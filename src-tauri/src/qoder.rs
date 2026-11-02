use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use rusqlite::{params, Connection, OpenFlags};
use serde_json::Value;
use tauri::{AppHandle, Emitter};

use crate::events::{
    include_task, is_paused, report_status, sanitize_task_title, ActivityPayload, BubblePayload,
    TokenUsage,
};

const SOURCE: &str = "qoder";
const POLL_INTERVAL: Duration = Duration::from_secs(1);
const NOT_FOUND_INTERVAL: Duration = Duration::from_secs(10);
const ERROR_INTERVAL: Duration = Duration::from_secs(3);
const BUSY_TIMEOUT: Duration = Duration::from_millis(250);
const MAX_ROWS_PER_POLL: usize = 200;

#[derive(Debug)]
struct ChatMessage {
    usage: TokenUsage,
    session_id: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Watermark {
    last_rowid: i64,
}

#[derive(Debug)]
enum PollError {
    Busy,
    Other(String),
}

impl std::fmt::Display for PollError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PollError::Busy => write!(f, "database is busy"),
            PollError::Other(message) => write!(f, "{message}"),
        }
    }
}

impl From<rusqlite::Error> for PollError {
    fn from(error: rusqlite::Error) -> Self {
        if let rusqlite::Error::SqliteFailure(code, _) = &error {
            if matches!(
                code.code,
                rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked
            ) {
                return PollError::Busy;
            }
        }
        PollError::Other(error.to_string())
    }
}

impl From<std::io::Error> for PollError {
    fn from(error: std::io::Error) -> Self {
        PollError::Other(error.to_string())
    }
}

impl From<serde_json::Error> for PollError {
    fn from(error: serde_json::Error) -> Self {
        PollError::Other(error.to_string())
    }
}

pub fn start(app: AppHandle, config_dir: PathBuf) {
    std::thread::spawn(move || {
        let _ = fs::create_dir_all(&config_dir);
        let watermark_path = std::env::var_os("MOYUYU_QODER_WATERMARK")
            .map(PathBuf::from)
            .unwrap_or_else(|| config_dir.join("qoder-watermark.json"));
        let mut last_status = String::new();

        // 给前端监听器留出挂载时间
        std::thread::sleep(POLL_INTERVAL);

        loop {
            let Some(db_path) = find_database() else {
                report_status(
                    &app,
                    SOURCE,
                    "not_found",
                    "未检测到 Qoder 本地数据库",
                    &mut last_status,
                );
                std::thread::sleep(NOT_FOUND_INTERVAL);
                continue;
            };

            match poll_once(&app, &db_path, &watermark_path) {
                Ok(()) => {
                    report_status(&app, SOURCE, "connected", "Qoder 已连接", &mut last_status);
                    std::thread::sleep(POLL_INTERVAL);
                }
                Err(PollError::Busy) => {
                    report_status(&app, SOURCE, "busy", "Qoder 数据库忙，稍后重试", &mut last_status);
                    std::thread::sleep(ERROR_INTERVAL);
                }
                Err(PollError::Other(message)) => {
                    report_status(&app, SOURCE, "error", &message, &mut last_status);
                    std::thread::sleep(ERROR_INTERVAL);
                }
            }
        }
    });
}

fn find_database() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("MOYUYU_QODER_DB") {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Some(path);
        }
    }

    let appdata = std::env::var_os("APPDATA")?;
    let base = PathBuf::from(appdata);

    ["QoderCN", "Qoder", "Qoder CN"]
        .iter()
        .map(|name| {
            base.join(name)
                .join("SharedClientCache")
                .join("cache")
                .join("db")
                .join("local.db")
        })
        .find(|path| path.is_file())
}

fn poll_once(app: &AppHandle, db_path: &Path, watermark_path: &Path) -> Result<(), PollError> {
    let connection = open_connection(db_path)?;
    let max_rowid: i64 = connection.query_row(
        "SELECT COALESCE(MAX(rowid), 0)
         FROM chat_message
         WHERE role = 'assistant'
           AND token_info IS NOT NULL
           AND token_info <> ''",
        [],
        |row| row.get(0),
    )?;

    let last_rowid = match load_watermark(watermark_path) {
        Some(mark) if mark <= max_rowid => mark,
        Some(_) => {
            save_watermark(watermark_path, max_rowid)?;
            max_rowid
        }
        None => {
            save_watermark(watermark_path, max_rowid)?;
            return Ok(());
        }
    };

    if last_rowid >= max_rowid {
        return Ok(());
    }

    let (messages, new_rowid) = read_new_messages(&connection, last_rowid)?;
    if new_rowid > last_rowid {
        save_watermark(watermark_path, new_rowid)?;
    }

    let total: u64 = messages.iter().map(|message| message.usage.total()).sum();
    if total > 0 && !is_paused(app) {
        let task = if include_task(app) {
            messages
                .iter()
                .rev()
                .find_map(|message| message.session_id.as_deref())
                .and_then(|session_id| load_session_title(&connection, session_id))
                .and_then(|title| sanitize_task_title(&title))
        } else {
            None
        };

        let intensity = (total as f32 / 500.0).clamp(0.25, 1.0);
        let _ = app.emit(
            "pet://activity",
            ActivityPayload {
                state: "working",
                intensity,
                token_rate: total as f64,
                source: SOURCE,
            },
        );
        let _ = app.emit(
            "pet://bubble",
            BubblePayload {
                kind: "working",
                tool: Some("Qoder"),
                tokens: Some(total),
                task,
            },
        );
    }

    Ok(())
}

fn open_connection(path: &Path) -> Result<Connection, PollError> {
    match open_read_only(path) {
        Ok(connection) => Ok(connection),
        Err(primary_error) => open_snapshot(path).map_err(|_| primary_error),
    }
}

fn open_read_only(path: &Path) -> Result<Connection, PollError> {
    let flags = OpenFlags::SQLITE_OPEN_READ_ONLY
        | OpenFlags::SQLITE_OPEN_URI
        | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let connection = Connection::open_with_flags(path, flags)?;
    connection.busy_timeout(BUSY_TIMEOUT)?;
    Ok(connection)
}

fn open_snapshot(path: &Path) -> Result<Connection, PollError> {
    let snapshot_dir = std::env::temp_dir().join("moyuyu-pet-qoder");
    fs::create_dir_all(&snapshot_dir)?;
    let target = snapshot_dir.join("local.db");
    fs::copy(path, &target)?;

    for suffix in ["-wal", "-shm"] {
        let source = with_suffix(path, suffix);
        if source.is_file() {
            let _ = fs::copy(&source, with_suffix(&target, suffix));
        }
    }

    open_read_only(&target)
}

fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut value = path.as_os_str().to_os_string();
    value.push(suffix);
    PathBuf::from(value)
}

fn load_watermark(path: &Path) -> Option<i64> {
    let raw = fs::read(path).ok()?;
    let watermark: Watermark = serde_json::from_slice(&raw).ok()?;
    (watermark.last_rowid >= 0).then_some(watermark.last_rowid)
}

fn save_watermark(path: &Path, last_rowid: i64) -> Result<(), PollError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temporary = with_suffix(path, ".tmp");
    let raw = serde_json::to_vec(&Watermark { last_rowid })?;
    fs::write(&temporary, raw)?;
    fs::rename(&temporary, path)?;
    Ok(())
}

fn read_new_messages(
    connection: &Connection,
    after_rowid: i64,
) -> Result<(Vec<ChatMessage>, i64), PollError> {
    let mut statement = connection.prepare(
        "SELECT rowid, token_info, session_id
         FROM chat_message
         WHERE rowid > ?1
           AND role = 'assistant'
           AND token_info IS NOT NULL
           AND token_info <> ''
         ORDER BY rowid ASC
         LIMIT ?2",
    )?;

    let rows = statement.query_map(params![after_rowid, MAX_ROWS_PER_POLL as i64], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, Option<String>>(2)?,
        ))
    })?;

    let mut messages = Vec::new();
    let mut new_rowid = after_rowid;
    for row in rows {
        let (rowid, raw, session_id) = row?;
        if rowid > new_rowid {
            new_rowid = rowid;
        }
        if let Some(usage) = parse_token_info(&raw) {
            messages.push(ChatMessage { usage, session_id });
        }
    }

    Ok((messages, new_rowid))
}

fn load_session_title(connection: &Connection, session_id: &str) -> Option<String> {
    connection
        .query_row(
            "SELECT session_title FROM chat_session WHERE session_id = ?1",
            params![session_id],
            |row| row.get::<_, Option<String>>(0),
        )
        .ok()
        .flatten()
}

fn parse_token_info(raw: &str) -> Option<TokenUsage> {
    let value: Value = serde_json::from_str(raw).ok()?;
    if !value.is_object() {
        return None;
    }

    let read = |key: &str| value.get(key).and_then(Value::as_u64).unwrap_or(0);
    let usage = TokenUsage {
        input: read("prompt_tokens"),
        output: read("completion_tokens"),
        cache_read: read("cached_tokens"),
        cache_write: 0,
    };

    if usage.input == 0 && usage.output == 0 && usage.cache_read == 0 {
        None
    } else {
        Some(usage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_token_info() {
        let usage = parse_token_info(
            r#"{"prompt_tokens":120,"completion_tokens":30,"cached_tokens":20,"max_input_tokens":1000}"#,
        )
        .expect("usage should parse");

        assert_eq!(usage.input, 120);
        assert_eq!(usage.output, 30);
        assert_eq!(usage.cache_read, 20);
        assert_eq!(usage.total(), 150);
    }

    #[test]
    fn ignores_invalid_or_zero_usage() {
        assert!(parse_token_info("not json").is_none());
        assert!(parse_token_info(r#"{"prompt_tokens":0,"completion_tokens":0}"#).is_none());
    }

    #[test]
    fn reads_only_new_assistant_rows() {
        let connection = Connection::open_in_memory().expect("in-memory db");
        connection
            .execute_batch(
                "CREATE TABLE chat_message (id INTEGER PRIMARY KEY, role TEXT, token_info TEXT, session_id TEXT);
                 INSERT INTO chat_message (role, token_info, session_id) VALUES ('user', '{\"prompt_tokens\":99}', 's1');
                 INSERT INTO chat_message (role, token_info, session_id) VALUES ('assistant', '{\"prompt_tokens\":10,\"completion_tokens\":5}', 's1');
                 INSERT INTO chat_message (role, token_info, session_id) VALUES ('assistant', NULL, 's1');
                 INSERT INTO chat_message (role, token_info, session_id) VALUES ('assistant', '{\"prompt_tokens\":20,\"completion_tokens\":2}', 's1');",
            )
            .expect("fixture");

        let (messages, new_rowid) = read_new_messages(&connection, 1).expect("rows");

        assert_eq!(new_rowid, 4);
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].usage.total(), 15);
        assert_eq!(messages[0].session_id.as_deref(), Some("s1"));
        assert_eq!(messages[1].usage.total(), 22);
    }

    #[test]
    fn loads_session_title() {
        let connection = Connection::open_in_memory().expect("in-memory db");
        connection
            .execute_batch(
                "CREATE TABLE chat_session (session_id TEXT PRIMARY KEY, session_title TEXT);
                 INSERT INTO chat_session (session_id, session_title) VALUES ('s1', '重构登录流程');",
            )
            .expect("fixture");

        assert_eq!(
            load_session_title(&connection, "s1").as_deref(),
            Some("重构登录流程")
        );
        assert_eq!(load_session_title(&connection, "missing"), None);
    }

    #[test]
    fn watermark_roundtrip() {
        let directory = tempfile::tempdir().expect("temp dir");
        let path = directory.path().join("qoder-watermark.json");

        assert_eq!(load_watermark(&path), None);
        save_watermark(&path, 42).expect("save watermark");
        assert_eq!(load_watermark(&path), Some(42));
    }
}
