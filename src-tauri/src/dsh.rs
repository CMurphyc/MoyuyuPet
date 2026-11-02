use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use serde_json::Value;
use tauri::{AppHandle, Emitter};

use crate::events::{is_paused, report_status, ActivityPayload, BubblePayload};

const SOURCE: &str = "dsh";
const POLL_INTERVAL: Duration = Duration::from_secs(2);
const NOT_FOUND_INTERVAL: Duration = Duration::from_secs(10);
const ERROR_INTERVAL: Duration = Duration::from_secs(3);

type FileStamp = (SystemTime, u64);

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
struct Watermarks {
    #[serde(default)]
    initialized: bool,
    #[serde(default)]
    files: BTreeMap<String, i64>,
}

#[derive(Debug, serde::Deserialize)]
struct RawEvent {
    #[serde(default)]
    seq: i64,
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    data: Value,
}

#[derive(Debug, PartialEq)]
enum DshAction {
    Working { intensity: f32, tokens: Option<u64> },
    Celebrate { tokens: u64 },
    Error,
    Idle,
}

pub fn start(app: AppHandle, config_dir: PathBuf) {
    std::thread::spawn(move || {
        let _ = fs::create_dir_all(&config_dir);
        let watermark_path = std::env::var_os("MOYUYU_DSH_WATERMARK")
            .map(PathBuf::from)
            .unwrap_or_else(|| config_dir.join("dsh-watermark.json"));
        let mut mtimes: HashMap<PathBuf, FileStamp> = HashMap::new();
        let mut last_status = String::new();

        // 给前端监听器留出挂载时间
        std::thread::sleep(Duration::from_secs(1));

        loop {
            let Some(root) = find_root() else {
                report_status(
                    &app,
                    SOURCE,
                    "not_found",
                    "未检测到 dsh 会话目录",
                    &mut last_status,
                );
                std::thread::sleep(NOT_FOUND_INTERVAL);
                continue;
            };

            match poll_once(&app, &root, &watermark_path, &mut mtimes) {
                Ok(()) => {
                    report_status(&app, SOURCE, "connected", "dsh 已连接", &mut last_status);
                    std::thread::sleep(POLL_INTERVAL);
                }
                Err(message) => {
                    report_status(&app, SOURCE, "error", &message, &mut last_status);
                    std::thread::sleep(ERROR_INTERVAL);
                }
            }
        }
    });
}

fn find_root() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("MOYUYU_DSH_ROOT") {
        let path = PathBuf::from(path);
        if path.is_dir() {
            return Some(path);
        }
    }

    let home = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME"))?;
    let root = PathBuf::from(home).join(".dsh").join("sessions");
    root.is_dir().then_some(root)
}

fn session_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let Ok(projects) = fs::read_dir(root) else {
        return files;
    };

    for project in projects.flatten() {
        let project_path = project.path();
        if !project_path.is_dir() {
            continue;
        }

        let Ok(sessions) = fs::read_dir(project_path) else {
            continue;
        };
        for session in sessions.flatten() {
            let session_path = session.path();
            if !session_path.is_dir() {
                continue;
            }

            let file = session_path.join("session.v4.jsonl.zstd");
            if file.is_file() {
                files.push(file);
            }
        }
    }

    files
}

fn poll_once(
    app: &AppHandle,
    root: &Path,
    watermark_path: &Path,
    mtimes: &mut HashMap<PathBuf, FileStamp>,
) -> Result<(), String> {
    let (mut watermarks, first_run) = match load_watermarks(watermark_path) {
        Some(store) => {
            let first_run = !store.initialized;
            (store, first_run)
        }
        None => (Watermarks::default(), true),
    };
    watermarks.initialized = true;

    let mut errors = Vec::new();
    let mut changed = first_run;

    for path in session_files(root) {
        let metadata = fs::metadata(&path).ok();
        let stamp = metadata
            .as_ref()
            .and_then(|meta| meta.modified().ok().map(|time| (time, meta.len())));

        if !first_run && stamp.is_some() && mtimes.get(&path) == stamp.as_ref() {
            continue;
        }

        let events = match read_events(&path) {
            Ok(events) => events,
            Err(error) => {
                errors.push(error);
                continue;
            }
        };

        let max_seq = events.iter().map(|event| event.seq).max().unwrap_or(0);
        let key = path.to_string_lossy().to_string();

        if first_run {
            watermarks.files.insert(key, max_seq);
        } else {
            let mut last_seq = watermarks.files.get(&key).copied().unwrap_or(0);
            if last_seq > max_seq {
                last_seq = 0;
            }

            if max_seq > last_seq {
                let (actions, new_seq) = actions_for_events(&events, last_seq);
                for action in actions {
                    emit_action(app, action);
                }
                watermarks.files.insert(key, new_seq);
                changed = true;
            }
        }

        if let Some(stamp) = stamp {
            mtimes.insert(path, stamp);
        }
    }

    if changed {
        save_watermarks(watermark_path, &watermarks)?;
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

fn read_events(path: &Path) -> Result<Vec<RawEvent>, String> {
    let file = fs::File::open(path)
        .map_err(|error| format!("读取 {} 失败: {error}", path.display()))?;
    let mut decoder = zstd::stream::read::Decoder::new(file)
        .map_err(|error| format!("解压 {} 失败: {error}", path.display()))?;

    let mut text = String::new();
    decoder
        .read_to_string(&mut text)
        .map_err(|error| format!("解码 {} 失败: {error}", path.display()))?;

    let mut events = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(event) = serde_json::from_str::<RawEvent>(line) {
            events.push(event);
        }
    }

    Ok(events)
}

fn actions_for_events(events: &[RawEvent], after_seq: i64) -> (Vec<DshAction>, i64) {
    let mut actions = Vec::new();
    let mut max_seq = after_seq;
    let mut turn_tokens = 0u64;

    // 如果应用在 turn 中途启动，先补算水位之前当前 turn 的用量，保证庆祝气泡数据完整。
    for event in events {
        if event.seq > after_seq {
            break;
        }
        match event.kind.as_str() {
            "turn/start" => turn_tokens = 0,
            "assistant/message" => {
                turn_tokens = turn_tokens.saturating_add(usage_total(&event.data).unwrap_or(0));
            }
            "turn/end" => turn_tokens = 0,
            _ => {}
        }
    }

    for event in events {
        max_seq = max_seq.max(event.seq);
        if event.seq <= after_seq {
            continue;
        }

        match event.kind.as_str() {
            "turn/start" => {
                turn_tokens = 0;
                actions.push(DshAction::Working {
                    intensity: 0.25,
                    tokens: None,
                });
            }
            "tool/call" => actions.push(DshAction::Working {
                intensity: 0.35,
                tokens: None,
            }),
            "assistant/message" => {
                if let Some(total) = usage_total(&event.data) {
                    turn_tokens = turn_tokens.saturating_add(total);
                    actions.push(DshAction::Working {
                        intensity: intensity_for(total),
                        tokens: Some(total),
                    });
                }
            }
            "turn/end" => {
                let kind = event
                    .data
                    .get("reason")
                    .and_then(|reason| reason.get("kind"))
                    .and_then(Value::as_str)
                    .unwrap_or_default();

                match kind {
                    "completed" => actions.push(DshAction::Celebrate {
                        tokens: turn_tokens,
                    }),
                    "failed" | "error" => actions.push(DshAction::Error),
                    _ => actions.push(DshAction::Idle),
                }
                turn_tokens = 0;
            }
            _ => {}
        }
    }

    (actions, max_seq)
}

fn usage_total(data: &Value) -> Option<u64> {
    let usage = data.get("usage")?;
    if let Some(total) = usage.get("totalTokens").and_then(Value::as_u64) {
        if total > 0 {
            return Some(total);
        }
    }

    let total: u64 = ["inputTokens", "outputTokens", "cacheReadTokens", "cacheWriteTokens"]
        .iter()
        .filter_map(|key| usage.get(*key).and_then(Value::as_u64))
        .sum();

    (total > 0).then_some(total)
}

fn intensity_for(total: u64) -> f32 {
    (total as f32 / 1000.0).clamp(0.25, 1.0)
}

fn emit_action(app: &AppHandle, action: DshAction) {
    if is_paused(app) {
        return;
    }

    match action {
        DshAction::Working { intensity, tokens } => {
            let _ = app.emit(
                "pet://activity",
                ActivityPayload {
                    state: "working",
                    intensity,
                    token_rate: tokens.unwrap_or(0) as f64,
                    source: SOURCE,
                },
            );
            if let Some(tokens) = tokens.filter(|tokens| *tokens > 0) {
                let _ = app.emit(
                    "pet://bubble",
                    BubblePayload {
                        kind: "working",
                        tool: Some(SOURCE),
                        tokens: Some(tokens),
                    },
                );
            }
        }
        DshAction::Celebrate { tokens } => {
            let _ = app.emit(
                "pet://activity",
                ActivityPayload {
                    state: "celebrate",
                    intensity: 0.5,
                    token_rate: 0.0,
                    source: SOURCE,
                },
            );
            let _ = app.emit(
                "pet://bubble",
                BubblePayload {
                    kind: "celebrate",
                    tool: Some(SOURCE),
                    tokens: Some(tokens),
                },
            );
        }
        DshAction::Error => {
            let _ = app.emit(
                "pet://activity",
                ActivityPayload {
                    state: "error",
                    intensity: 0.6,
                    token_rate: 0.0,
                    source: SOURCE,
                },
            );
            let _ = app.emit(
                "pet://bubble",
                BubblePayload {
                    kind: "error",
                    tool: Some(SOURCE),
                    tokens: None,
                },
            );
        }
        DshAction::Idle => {
            let _ = app.emit(
                "pet://activity",
                ActivityPayload {
                    state: "idle",
                    intensity: 0.0,
                    token_rate: 0.0,
                    source: SOURCE,
                },
            );
        }
    }
}

fn load_watermarks(path: &Path) -> Option<Watermarks> {
    let raw = fs::read(path).ok()?;
    serde_json::from_slice(&raw).ok()
}

fn save_watermarks(path: &Path, watermarks: &Watermarks) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("创建水位目录失败: {error}"))?;
    }

    let temporary = with_suffix(path, ".tmp");
    let raw = serde_json::to_vec(watermarks)
        .map_err(|error| format!("序列化水位失败: {error}"))?;
    fs::write(&temporary, raw).map_err(|error| format!("写入水位失败: {error}"))?;
    fs::rename(&temporary, path).map_err(|error| format!("替换水位失败: {error}"))?;
    Ok(())
}

fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut value = path.as_os_str().to_os_string();
    value.push(suffix);
    PathBuf::from(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_events(text: &str) -> Vec<RawEvent> {
        text.lines()
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect()
    }

    const SAMPLE: &str = r#"
{"type":"turn/start","seq":1,"data":{"turn":1}}
{"type":"assistant/message","seq":2,"data":{"turn":1,"step":1,"usage":{"inputTokens":100,"outputTokens":50,"cacheReadTokens":0,"cacheWriteTokens":0,"totalTokens":150}}}
{"type":"tool/call","seq":3,"data":{"turn":1,"step":1,"name":"bash"}}
{"type":"turn/end","seq":4,"data":{"turn":1,"reason":{"kind":"completed"}}}
"#;

    #[test]
    fn maps_turn_actions() {
        let events = parse_events(SAMPLE);
        let (actions, seq) = actions_for_events(&events, 0);

        assert_eq!(seq, 4);
        assert_eq!(actions.len(), 4);
        assert!(matches!(
            actions[0],
            DshAction::Working {
                intensity: 0.25,
                tokens: None
            }
        ));
        assert!(matches!(
            actions[1],
            DshAction::Working {
                tokens: Some(150),
                ..
            }
        ));
        assert!(matches!(
            actions[2],
            DshAction::Working {
                tokens: None,
                ..
            }
        ));
        assert!(matches!(
            actions[3],
            DshAction::Celebrate { tokens: 150 }
        ));
    }

    #[test]
    fn keeps_turn_total_when_starting_mid_turn() {
        let events = parse_events(SAMPLE);
        let (actions, seq) = actions_for_events(&events, 3);

        assert_eq!(seq, 4);
        assert_eq!(actions, vec![DshAction::Celebrate { tokens: 150 }]);
    }

    #[test]
    fn maps_failed_turn_to_error() {
        let events = parse_events(
            r#"
{"type":"turn/start","seq":1,"data":{"turn":1}}
{"type":"assistant/message","seq":2,"data":{"usage":{"inputTokens":10,"outputTokens":5}}}
{"type":"turn/end","seq":3,"data":{"reason":{"kind":"failed"}}}
"#,
        );

        let (actions, _) = actions_for_events(&events, 0);
        assert!(matches!(actions.last(), Some(DshAction::Error)));
    }

    #[test]
    fn falls_back_to_component_usage() {
        let data = serde_json::json!({
            "usage": {
                "inputTokens": 10,
                "outputTokens": 5,
                "cacheReadTokens": 2,
                "cacheWriteTokens": 1
            }
        });

        assert_eq!(usage_total(&data), Some(18));
    }

    #[test]
    fn skips_malformed_lines() {
        let events = parse_events(
            "not json\n{\"type\":\"turn/start\",\"seq\":1,\"data\":{}}\n{\"type\":\"turn/end\",\"seq\":2,\"data\":{\"reason\":{\"kind\":\"completed\"}}}",
        );

        assert_eq!(events.len(), 2);
        assert_eq!(events[0].seq, 1);
        assert_eq!(events[1].seq, 2);
    }

    #[test]
    fn watermark_roundtrip() {
        let directory = tempfile::tempdir().expect("temp dir");
        let path = directory.path().join("dsh-watermark.json");
        let mut store = Watermarks::default();
        store.initialized = true;
        store.files.insert("session-a".to_string(), 7);

        save_watermarks(&path, &store).expect("save watermarks");
        let loaded = load_watermarks(&path).expect("load watermarks");

        assert!(loaded.initialized);
        assert_eq!(loaded.files.get("session-a"), Some(&7));
    }
}
