use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use tauri::{AppHandle, Emitter, Manager};

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
}

pub fn is_paused(app: &AppHandle) -> bool {
    app.try_state::<PetState>()
        .map(|state| state.is_paused())
        .unwrap_or(false)
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
