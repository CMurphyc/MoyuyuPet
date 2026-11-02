use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_autostart::ManagerExt;

use crate::events::{AdapterRegistry, AdapterStatus, PetState, SettingsPayload};

#[tauri::command]
pub fn get_settings(app: AppHandle, state: State<'_, PetState>) -> SettingsPayload {
    SettingsPayload {
        paused: state.is_paused(),
        autostart: app.autolaunch().is_enabled().unwrap_or(false),
    }
}

#[tauri::command]
pub fn set_paused(app: AppHandle, state: State<'_, PetState>, paused: bool) {
    state.set_paused(paused);
    let _ = app.emit("pet://paused", paused);
}

#[tauri::command]
pub fn set_autostart(app: AppHandle, enabled: bool) -> Result<bool, String> {
    let manager = app.autolaunch();
    if enabled {
        manager.enable().map_err(|error| error.to_string())?;
    } else {
        manager.disable().map_err(|error| error.to_string())?;
    }
    manager.is_enabled().map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_adapter_statuses(registry: State<'_, AdapterRegistry>) -> Vec<AdapterStatus> {
    let mut statuses: Vec<_> = registry
        .0
        .lock()
        .map(|statuses| statuses.values().cloned().collect())
        .unwrap_or_default();

    statuses.sort_by_key(|status| status.source);
    statuses
}

pub fn open_settings_window(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.show();
        let _ = window.set_focus();
        return Ok(());
    }

    WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("index.html".into()))
        .title("MoyuyuPet 设置")
        .inner_size(560.0, 640.0)
        .min_inner_size(480.0, 520.0)
        .resizable(true)
        .always_on_top(true)
        .center()
        .build()?;

    Ok(())
}
