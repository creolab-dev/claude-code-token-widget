use crate::models::TokenUsageData;
use std::collections::HashSet;
use std::sync::Mutex;

#[derive(Default)]
pub struct AppStateInner {
    pub token_data: Option<TokenUsageData>,
    pub watcher_active: bool,
    // アラート通知の重複防止
    pub notified_thresholds_5h: HashSet<u8>,
    pub notified_thresholds_7d: HashSet<u8>,
    pub last_5h_resets_at: Option<u64>,
    pub last_7d_resets_at: Option<u64>,
    // ヒートマップ用デルタ追跡
    pub last_session_id: Option<String>,
    pub last_session_cost: f64,
    // スケジューラ
    pub scheduler_handle: Option<tauri::async_runtime::JoinHandle<()>>,
}

/// Tauri managed state（Arc は Tauri が内部で管理）
pub type AppState = Mutex<AppStateInner>;
