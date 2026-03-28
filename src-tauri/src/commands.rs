use crate::error::{AppError, AppResult};
use crate::models::{HeatmapEntry, TokenUsageData, WidgetSettings};
use crate::state::AppState;
use crate::watcher;
use tauri::{Manager, State};
use tauri_plugin_store::StoreExt;

/// 現在のトークンデータを取得（初期ロード用）
/// State にデータがなければファイルから直接読み取る
#[tauri::command]
pub fn get_current_token(state: State<'_, AppState>) -> AppResult<Option<TokenUsageData>> {
    let guard = state.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    if guard.token_data.is_some() {
        return Ok(guard.token_data.clone());
    }
    drop(guard);

    // State にデータがない場合、ファイルから直接読み取り
    let path = watcher::resolve_token_file_path();
    if path.exists() {
        match watcher::read_and_parse(&path) {
            Ok(data) => {
                // State にも保存
                if let Ok(mut guard) = state.lock() {
                    guard.token_data = Some(data.clone());
                }
                Ok(Some(data))
            }
            Err(_) => Ok(None),
        }
    } else {
        Ok(None)
    }
}

/// ウォッチャーの稼働状態を取得
#[tauri::command]
pub fn get_watcher_status(state: State<'_, AppState>) -> AppResult<bool> {
    let guard = state.lock().map_err(|e| AppError::Lock(e.to_string()))?;
    Ok(guard.watcher_active)
}

/// 現在のウィジェット設定を取得
#[tauri::command]
pub fn get_settings(app_handle: tauri::AppHandle) -> AppResult<WidgetSettings> {
    let store = app_handle
        .store("settings.json")
        .map_err(|e| AppError::Store(e.to_string()))?;

    let settings = match store.get("settings") {
        Some(value) => serde_json::from_value(value).unwrap_or_default(),
        None => WidgetSettings::default(),
    };

    Ok(settings)
}

/// Always on Top を切り替え
#[tauri::command]
pub fn set_always_on_top(app_handle: tauri::AppHandle, value: bool) -> AppResult<()> {
    if let Some(window) = app_handle.get_webview_window("main") {
        window
            .set_always_on_top(value)
            .map_err(|e| AppError::Window(e.to_string()))?;
    }
    Ok(())
}

/// ウィンドウサイズを変更（resizable:false でも動作するように Rust 側で実行）
#[tauri::command]
pub fn resize_window(app_handle: tauri::AppHandle, width: f64, height: f64) -> AppResult<()> {
    let width = width.clamp(100.0, 800.0);
    let height = height.clamp(50.0, 2000.0);
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.set_resizable(true);
        let size = tauri::LogicalSize::new(width, height);
        let _ = window.set_size(tauri::Size::Logical(size));
        let _ = window.set_resizable(false);
    }
    Ok(())
}

/// ウィジェット設定を更新・保存
#[tauri::command]
pub fn update_settings(
    app_handle: tauri::AppHandle,
    settings: WidgetSettings,
) -> AppResult<()> {
    let value =
        serde_json::to_value(&settings).map_err(|e| AppError::Store(e.to_string()))?;

    let store = app_handle
        .store("settings.json")
        .map_err(|e| AppError::Store(e.to_string()))?;
    store.set("settings", value);
    store.save().map_err(|e| AppError::Store(e.to_string()))?;

    // スケジューラ設定が変更された場合は再起動
    if settings.schedule_settings.is_some() {
        crate::scheduler::restart_scheduler(app_handle);
    }

    Ok(())
}

/// ヒートマップデータを取得
#[tauri::command]
pub fn get_heatmap_data(
    app_handle: tauri::AppHandle,
    days: Option<u32>,
) -> AppResult<Vec<HeatmapEntry>> {
    let days = days.unwrap_or(365).min(3650);
    Ok(crate::heatmap::get_data(&app_handle, days))
}
