mod alerts;
mod commands;
mod error;
mod heatmap;
mod models;
mod scheduler;
mod state;
mod tray;
mod watcher;

use state::AppState;
use std::sync::Mutex;
use tauri::{Emitter, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_notification::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::get_current_token,
            commands::get_watcher_status,
            commands::get_settings,
            commands::update_settings,
            commands::resize_window,
            commands::set_always_on_top,
            commands::get_heatmap_data,
        ])
        .setup(|app| {
            // システムトレイ
            tray::setup_tray(app)?;

            let app_handle = app.handle().clone();
            let token_file_path = watcher::resolve_token_file_path();

            // 初期データ読み取り
            if token_file_path.exists() {
                if let Ok(data) = watcher::read_and_parse(&token_file_path) {
                    if let Ok(mut guard) = app.state::<AppState>().lock() {
                        guard.token_data = Some(data.clone());
                        guard.watcher_active = true;
                    }
                    tray::update_tray(app.handle(), &data);
                    let handle = app.handle().clone();
                    let init_data = data;
                    tauri::async_runtime::spawn(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                        let _ = handle.emit("token-updated", &init_data);
                    });
                }
            }

            // ウォッチャー起動
            if let Ok(debouncer) = watcher::start_file_watcher(app_handle.clone(), token_file_path)
            {
                app.manage(Mutex::new(Some(debouncer)));
            }

            // スケジューラ起動
            scheduler::start_scheduler(app_handle);

            // ウィンドウ表示
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
