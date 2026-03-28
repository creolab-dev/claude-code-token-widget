use crate::models::{HeatmapEntry, TokenUsageData};
use crate::state::AppState;
use tauri::{AppHandle, Manager};
use tauri_plugin_store::StoreExt;

const HEATMAP_STORE: &str = "heatmap-data.json";

/// コスト差分を計算する。セッションが変わった場合は全額、同一セッションでは差分のみ。
/// 戻り値: (cost_delta, session_changed)
pub fn compute_cost_delta(
    current_cost: f64,
    last_session_cost: f64,
    current_session_id: Option<&str>,
    last_session_id: Option<&str>,
) -> (f64, bool) {
    let session_changed = last_session_id != current_session_id;
    let cost_delta = if session_changed {
        current_cost
    } else {
        (current_cost - last_session_cost).max(0.0)
    };
    (cost_delta, session_changed)
}

/// トークン更新時に日次データを蓄積
pub fn record_usage(app_handle: &AppHandle, data: &TokenUsageData) {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    let Some(state) = app_handle.try_state::<AppState>() else {
        return;
    };

    let (cost_delta, input_tokens, output_tokens, new_session, guard_session_id, guard_session_cost) = {
        let Ok(mut guard) = state.lock() else {
            return;
        };

        // ウィジェット再起動時: ストアから前回のセッション追跡状態を復元
        if guard.last_session_id.is_none() {
            if let Ok(store) = app_handle.store(HEATMAP_STORE) {
                if let Some(val) = store.get("_last_session_id") {
                    guard.last_session_id = serde_json::from_value(val).ok();
                }
                if let Some(val) = store.get("_last_session_cost") {
                    guard.last_session_cost = serde_json::from_value(val).unwrap_or(0.0);
                }
            }
        }

        let current_cost = data.cost.total_cost_usd;
        let current_session = data.session_id.clone();

        let (cost_delta, session_changed) = compute_cost_delta(
            current_cost,
            guard.last_session_cost,
            current_session.as_deref(),
            guard.last_session_id.as_deref(),
        );

        guard.last_session_id = current_session;
        guard.last_session_cost = current_cost;

        let sid = guard.last_session_id.clone();
        let cost = guard.last_session_cost;

        (
            cost_delta,
            data.context_window.total_input_tokens,
            data.context_window.total_output_tokens,
            session_changed,
            sid,
            cost,
        )
    };

    // 何も変化がなければスキップ
    if cost_delta <= 0.0 && !new_session {
        return;
    }

    let Ok(store) = app_handle.store(HEATMAP_STORE) else {
        return;
    };

    let mut entry = match store.get(&today) {
        Some(value) => serde_json::from_value::<HeatmapEntry>(value).unwrap_or_else(|_| new_entry(&today)),
        None => new_entry(&today),
    };

    entry.total_cost_usd += cost_delta;
    // トークン数は最新値で上書き（累積ではなくスナップショット）
    entry.total_input_tokens = input_tokens;
    entry.total_output_tokens = output_tokens;
    if new_session {
        entry.session_count += 1;
    }

    if let Ok(value) = serde_json::to_value(&entry) {
        store.set(&today, value);
    }

    // セッション追跡状態をストアに永続化
    if let Ok(sid_val) = serde_json::to_value(&guard_session_id) {
        store.set("_last_session_id", sid_val);
    }
    if let Ok(cost_val) = serde_json::to_value(guard_session_cost) {
        store.set("_last_session_cost", cost_val);
    }
    let _ = store.save();
}

fn new_entry(date: &str) -> HeatmapEntry {
    HeatmapEntry {
        date: date.to_string(),
        total_input_tokens: 0,
        total_output_tokens: 0,
        total_cost_usd: 0.0,
        session_count: 0,
    }
}

/// ヒートマップデータを取得（過去 N 日分）
pub fn get_data(app_handle: &AppHandle, days: u32) -> Vec<HeatmapEntry> {
    let Ok(store) = app_handle.store(HEATMAP_STORE) else {
        return Vec::new();
    };

    let mut entries = Vec::new();
    let today = chrono::Local::now().date_naive();

    for i in 0..days {
        let date = today - chrono::Duration::days(i64::from(i));
        let key = date.format("%Y-%m-%d").to_string();

        if let Some(value) = store.get(&key) {
            if let Ok(entry) = serde_json::from_value::<HeatmapEntry>(value) {
                entries.push(entry);
            }
        }
    }

    entries.reverse();
    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_session_returns_delta() {
        let (delta, changed) = compute_cost_delta(
            1.5,                    // current_cost
            1.0,                    // last_session_cost
            Some("session-1"),      // current_session_id
            Some("session-1"),      // last_session_id
        );
        assert!(!changed);
        assert!((delta - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn same_session_no_increase() {
        let (delta, changed) = compute_cost_delta(
            1.0,                    // current_cost (same as last)
            1.0,                    // last_session_cost
            Some("session-1"),
            Some("session-1"),
        );
        assert!(!changed);
        assert!((delta - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn same_session_negative_delta_clamped_to_zero() {
        // Cost decreased (shouldn't happen in practice but handled)
        let (delta, changed) = compute_cost_delta(
            0.5,
            1.0,
            Some("session-1"),
            Some("session-1"),
        );
        assert!(!changed);
        assert!((delta - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn new_session_returns_full_cost() {
        let (delta, changed) = compute_cost_delta(
            2.0,                    // current_cost
            1.0,                    // last_session_cost (from previous session)
            Some("session-2"),      // current_session_id
            Some("session-1"),      // last_session_id
        );
        assert!(changed);
        assert!((delta - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn first_session_no_previous() {
        let (delta, changed) = compute_cost_delta(
            0.5,
            0.0,
            Some("session-1"),
            None,
        );
        assert!(changed); // None != Some
        assert!((delta - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn both_sessions_none() {
        let (delta, changed) = compute_cost_delta(
            0.3,
            0.1,
            None,
            None,
        );
        assert!(!changed); // None == None
        assert!((delta - 0.2).abs() < f64::EPSILON);
    }

    #[test]
    fn current_none_previous_some() {
        let (delta, changed) = compute_cost_delta(
            0.8,
            0.5,
            None,
            Some("session-1"),
        );
        assert!(changed); // Some != None
        assert!((delta - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn zero_cost_new_session() {
        let (delta, changed) = compute_cost_delta(
            0.0,
            1.0,
            Some("session-2"),
            Some("session-1"),
        );
        assert!(changed);
        assert!((delta - 0.0).abs() < f64::EPSILON);
    }
}
