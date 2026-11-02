mod interaction;

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use tauri::{
    menu::{Menu, MenuItem, Submenu},
    tray::TrayIconBuilder,
    AppHandle, Emitter, Manager, WindowEvent,
};

#[derive(Default)]
struct PetState {
    paused: AtomicBool,
}

#[derive(Clone, serde::Serialize)]
struct ActivityPayload {
    state: &'static str,
    intensity: f32,
    token_rate: f64,
    source: &'static str,
}

#[derive(Clone, serde::Serialize)]
struct BubblePayload {
    kind: &'static str,
    tool: Option<&'static str>,
    tokens: Option<u64>,
}

fn emit_demo(app: &AppHandle, kind: &str) {
    let (state, intensity, rate, bubble) = match kind {
        "working" => (
            "working",
            0.8,
            140.0,
            Some(("working", Some("AI"), Some(140))),
        ),
        "celebrate" => ("celebrate", 0.5, 0.0, Some(("celebrate", None, None))),
        "error" => ("error", 0.6, 0.0, Some(("error", None, None))),
        "sleep" => ("sleep", 0.0, 0.0, Some(("idle", None, None))),
        "bubble" => {
            let _ = app.emit(
                "pet://bubble",
                BubblePayload {
                    kind: "idle",
                    tool: None,
                    tokens: None,
                },
            );
            return;
        }
        _ => ("idle", 0.0, 0.0, Some(("idle", None, None))),
    };

    let _ = app.emit(
        "pet://activity",
        ActivityPayload {
            state,
            intensity,
            token_rate: rate,
            source: "demo",
        },
    );

    if let Some((bubble_kind, tool, tokens)) = bubble {
        let _ = app.emit(
            "pet://bubble",
            BubblePayload {
                kind: bubble_kind,
                tool,
                tokens,
            },
        );
    }
}

fn toggle_pause(app: &AppHandle) {
    let state = app.state::<PetState>();
    let paused = !state.paused.load(Ordering::Relaxed);
    state.paused.store(paused, Ordering::Relaxed);
    let _ = app.emit("pet://paused", paused);
}

fn toggle_visibility(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("pet") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
        }
    }
}

fn schedule_position_clamp(window: tauri::WebviewWindow) {
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(600));

        let (Ok(position), Ok(size)) = (window.outer_position(), window.outer_size()) else {
            return;
        };
        let Ok(monitors) = window.available_monitors() else {
            return;
        };

        let visible = monitors.iter().any(|monitor| {
            let monitor_position = monitor.position();
            let monitor_size = monitor.size();
            position.x + size.width as i32 > monitor_position.x
                && position.x < monitor_position.x + monitor_size.width as i32
                && position.y + size.height as i32 > monitor_position.y
                && position.y < monitor_position.y + monitor_size.height as i32
        });

        if !visible {
            let _ = window.center();
        }
    });
}

fn setup_tray(app: &tauri::App) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "显示 / 隐藏", true, None::<&str>)?;
    let pause = MenuItem::with_id(app, "pause", "暂停 / 恢复", true, None::<&str>)?;
    let demo_working = MenuItem::with_id(app, "demo_working", "工作", true, None::<&str>)?;
    let demo_celebrate = MenuItem::with_id(app, "demo_celebrate", "庆祝", true, None::<&str>)?;
    let demo_error = MenuItem::with_id(app, "demo_error", "错误", true, None::<&str>)?;
    let demo_sleep = MenuItem::with_id(app, "demo_sleep", "睡眠", true, None::<&str>)?;
    let demo_bubble = MenuItem::with_id(app, "demo_bubble", "随机气泡", true, None::<&str>)?;

    let demo = Submenu::with_items(
        app,
        "调试",
        true,
        &[
            &demo_working,
            &demo_celebrate,
            &demo_error,
            &demo_sleep,
            &demo_bubble,
        ],
    )?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &pause, &demo, &quit])?;

    let mut builder = TrayIconBuilder::with_id("main-tray")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => toggle_visibility(app),
            "pause" => toggle_pause(app),
            "demo_working" => emit_demo(app, "working"),
            "demo_celebrate" => emit_demo(app, "celebrate"),
            "demo_error" => emit_demo(app, "error"),
            "demo_sleep" => emit_demo(app, "sleep"),
            "demo_bubble" => emit_demo(app, "bubble"),
            "quit" => app.exit(0),
            _ => {}
        });

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }

    builder.build(app)?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .manage(PetState::default())
        .setup(|app| {
            setup_tray(app)?;

            if let Some(window) = app.get_webview_window("pet") {
                let _ = window.set_ignore_cursor_events(true);
                interaction::watch_pet_interaction(window.clone());
                schedule_position_clamp(window);
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "pet" {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
