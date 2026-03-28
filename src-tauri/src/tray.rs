use crate::models::TokenUsageData;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::Manager;

const TRAY_ID: &str = "main-tray";

pub fn setup_tray(app: &tauri::App) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Show / Hide", true, None::<&str>)?;
    let on_top = MenuItem::with_id(app, "on_top", "Always on Top", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &on_top, &quit])?;

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(app.default_window_icon().cloned().unwrap_or_else(|| {
            tauri::image::Image::new_owned(vec![0u8; 16 * 16 * 4], 16, 16)
        }))
        .tooltip("Claude Code Token Widget")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            let window = app.get_webview_window("main");
            match event.id.as_ref() {
                "show" => {
                    if let Some(w) = window {
                        if w.is_visible().unwrap_or(false) {
                            let _ = w.hide();
                        } else {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                }
                "on_top" => {
                    if let Some(w) = window {
                        if let Ok(current) = w.is_always_on_top() {
                            let _ = w.set_always_on_top(!current);
                        }
                    }
                }
                "quit" => app.exit(0),
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                if let Some(window) = tray.app_handle().get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}

/// トークンデータ更新時にツールチップを更新
pub fn update_tray(app_handle: &tauri::AppHandle, data: &TokenUsageData) {
    let Some(tray) = app_handle.tray_by_id(TRAY_ID) else {
        return;
    };

    let five_h = data
        .rate_limits
        .as_ref()
        .and_then(|r| r.five_hour.as_ref())
        .map_or(0.0, |e| e.used_percentage);
    let seven_d = data
        .rate_limits
        .as_ref()
        .and_then(|r| r.seven_day.as_ref())
        .map_or(0.0, |e| e.used_percentage);
    let ctx = data.context_window.used_percentage;

    let tooltip = format!("5H: {:.0}% | 7D: {:.0}% | Ctx: {:.0}%", five_h, seven_d, ctx);
    let _ = tray.set_tooltip(Some(&tooltip));
}
