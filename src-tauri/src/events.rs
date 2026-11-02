use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use tauri::{AppHandle, Emitter, Manager};

use crate::preferences::Preferences;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct TokenUsage {
    pub input: u64,
    pub output: u64,
    pub cache_read: u64,
    pub cache_write: u64,
}

impl TokenUsage {
    pub fn total(self) -> u64 {
        self.input.saturating_add(self.output)
    }
}

#[derive(Clone, serde::Serialize)]
pub struct ActivityPayload {
    pub state: &'static str,
    pub intensity: f32,
    pub token_rate: f64,
    pub source: &'static str,
}

#[derive(Clone, serde::Serialize)]
pub struct BubblePayload {
    pub kind: &'static str,
    pub tool: Option<&'static str>,
    pub tokens: Option<u64>,
    pub task: Option<String>,
}

#[derive(Clone, serde::Serialize)]
pub struct AdapterStatus {
    pub source: &'static str,
    pub status: &'static str,
    pub message: String,
}

#[derive(Default)]
pub struct AdapterRegistry(pub Mutex<HashMap<&'static str, AdapterStatus>>);

#[derive(Default)]
pub struct PetState {
    paused: AtomicBool,
}

impl PetState {
    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::Relaxed)
    }

    pub fn set_paused(&self, paused: bool) {
        self.paused.store(paused, Ordering::Relaxed);
    }
}

#[derive(Clone, serde::Serialize)]
pub struct SettingsPayload {
    pub paused: bool,
    pub autostart: bool,
    pub include_task: bool,
}

pub fn is_paused(app: &AppHandle) -> bool {
    app.try_state::<PetState>()
        .map(|state| state.is_paused())
        .unwrap_or(false)
}

pub fn include_task(app: &AppHandle) -> bool {
    app.try_state::<Preferences>()
        .map(|preferences| preferences.include_task())
        .unwrap_or(false)
}

pub fn sanitize_task_title(raw: &str) -> Option<String> {
    let collapsed = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    let trimmed = collapsed.trim();
    if trimmed.is_empty() {
        return None;
    }

    let mut clipped: String = trimmed.chars().take(24).collect();
    if trimmed.chars().count() > 24 {
        clipped.push('…');
    }
    Some(clipped)
}

pub fn report_status(
    app: &AppHandle,
    source: &'static str,
    status: &'static str,
    message: &str,
    last_status: &mut String,
) {
    let key = format!("{status}|{message}");
    if *last_status == key {
        return;
    }
    *last_status = key;

    eprintln!("[{source}] {message}");
    let payload = AdapterStatus {
        source,
        status,
        message: message.to_string(),
    };

    if let Some(registry) = app.try_state::<AdapterRegistry>() {
        if let Ok(mut statuses) = registry.0.lock() {
            statuses.insert(source, payload.clone());
        }
    }

    let _ = app.emit("pet://adapter-status", payload);
}

#[cfg(test)]
mod tests {
    use super::sanitize_task_title;

    #[test]
    fn sanitizes_and_truncates_titles() {
        assert_eq!(
            sanitize_task_title("  hello\n world  ").as_deref(),
            Some("hello world")
        );
        assert_eq!(sanitize_task_title("   "), None);

        let long = "任务".repeat(20);
        let title = sanitize_task_title(&long).expect("title");
        assert_eq!(title.chars().count(), 25);
        assert!(title.ends_with('…'));
    }
}
