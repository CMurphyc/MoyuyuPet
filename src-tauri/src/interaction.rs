use std::time::Duration;

use tauri::WebviewWindow;

#[cfg(target_os = "windows")]
fn cursor_position() -> Option<(i32, i32)> {
    use windows_sys::Win32::Foundation::POINT;
    use windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos;

    let mut point = POINT { x: 0, y: 0 };
    let ok = unsafe { GetCursorPos(&mut point) };
    if ok != 0 {
        Some((point.x, point.y))
    } else {
        None
    }
}

#[cfg(not(target_os = "windows"))]
fn cursor_position() -> Option<(i32, i32)> {
    None
}

pub fn watch_pet_interaction(window: WebviewWindow) {
    std::thread::spawn(move || {
        let mut ignoring = true;
        let _ = window.set_ignore_cursor_events(true);

        loop {
            std::thread::sleep(Duration::from_millis(100));

            if !window.is_visible().unwrap_or(false) {
                continue;
            }

            let (Ok(position), Ok(size)) = (window.outer_position(), window.outer_size()) else {
                continue;
            };
            let Some((cursor_x, cursor_y)) = cursor_position() else {
                continue;
            };

            let relative_x = (cursor_x - position.x) as f64;
            let relative_y = (cursor_y - position.y) as f64;
            let width = size.width as f64;
            let height = size.height as f64;

            // 只有宠物本体区域可交互，气泡和透明区域保持点击穿透。
            let inside_pet = relative_x > width * 0.22
                && relative_x < width * 0.78
                && relative_y > height * 0.42
                && relative_y < height * 0.98;

            let next_ignoring = !inside_pet;
            if next_ignoring != ignoring {
                ignoring = next_ignoring;
                let _ = window.set_ignore_cursor_events(ignoring);
            }
        }
    });
}
