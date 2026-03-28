use crate::models::{AlertSettings, TokenUsageData, WidgetSettings};
use crate::state::AppState;
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_store::StoreExt;

/// 超過した閾値を検出する（通知済みのものは除外）
pub fn find_crossed_thresholds(
    used_percentage: f64,
    already_notified: &std::collections::HashSet<u8>,
    thresholds: &[u8],
) -> Vec<u8> {
    thresholds
        .iter()
        .filter(|&&t| used_percentage >= f64::from(t) && !already_notified.contains(&t))
        .copied()
        .collect()
}

/// レートリミットの閾値をチェックし、超過時にOS通知を送信
pub fn check_and_notify(app_handle: &AppHandle, data: &TokenUsageData) {
    let settings = load_alert_settings(app_handle);
    if !settings.enabled {
        return;
    }

    let Some(rate_limits) = &data.rate_limits else {
        return;
    };

    let Some(state) = app_handle.try_state::<AppState>() else {
        return;
    };

    // ロック内で通知リストを構築し、ロック外で送信
    let notifications: Vec<(String, String)> = {
        let Ok(mut guard) = state.lock() else {
            return;
        };

        let mut pending = Vec::new();

        // 5-Hour リミット
        if let Some(five_hour) = &rate_limits.five_hour {
            if guard.last_5h_resets_at != five_hour.resets_at {
                guard.notified_thresholds_5h.clear();
                guard.last_5h_resets_at = five_hour.resets_at;
            }

            let crossed = find_crossed_thresholds(
                five_hour.used_percentage,
                &guard.notified_thresholds_5h,
                &settings.thresholds,
            );
            for threshold in crossed {
                guard.notified_thresholds_5h.insert(threshold);
                pending.push((
                    format!("5-Hour Limit: {:.0}%", five_hour.used_percentage),
                    format!(
                        "Rate limit usage reached {:.0}% (threshold: {}%)",
                        five_hour.used_percentage, threshold
                    ),
                ));
            }
        }

        // 7-Day リミット
        if let Some(seven_day) = &rate_limits.seven_day {
            if guard.last_7d_resets_at != seven_day.resets_at {
                guard.notified_thresholds_7d.clear();
                guard.last_7d_resets_at = seven_day.resets_at;
            }

            let crossed = find_crossed_thresholds(
                seven_day.used_percentage,
                &guard.notified_thresholds_7d,
                &settings.thresholds,
            );
            for threshold in crossed {
                guard.notified_thresholds_7d.insert(threshold);
                pending.push((
                    format!("7-Day Limit: {:.0}%", seven_day.used_percentage),
                    format!(
                        "Rate limit usage reached {:.0}% (threshold: {}%)",
                        seven_day.used_percentage, threshold
                    ),
                ));
            }
        }

        pending
    };
    // ロック解放後に通知を送信
    for (title, body) in &notifications {
        send_notification(app_handle, title, body);
    }
}

fn load_alert_settings(app_handle: &AppHandle) -> AlertSettings {
    let Ok(store) = app_handle.store("settings.json") else {
        return AlertSettings::default();
    };

    match store.get("settings") {
        Some(value) => {
            let ws: WidgetSettings = serde_json::from_value(value).unwrap_or_default();
            ws.alert_settings.unwrap_or_default()
        }
        None => AlertSettings::default(),
    }
}

fn send_notification(app_handle: &AppHandle, title: &str, body: &str) {
    let _ = app_handle
        .notification()
        .builder()
        .title(title)
        .body(body)
        .show();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn no_thresholds_crossed_when_below_all() {
        let notified = HashSet::new();
        let thresholds = [50, 75, 90];
        let result = find_crossed_thresholds(30.0, &notified, &thresholds);
        assert!(result.is_empty());
    }

    #[test]
    fn one_threshold_crossed() {
        let notified = HashSet::new();
        let thresholds = [50, 75, 90];
        let result = find_crossed_thresholds(60.0, &notified, &thresholds);
        assert_eq!(result, vec![50]);
    }

    #[test]
    fn multiple_thresholds_crossed() {
        let notified = HashSet::new();
        let thresholds = [50, 75, 90];
        let result = find_crossed_thresholds(80.0, &notified, &thresholds);
        assert_eq!(result, vec![50, 75]);
    }

    #[test]
    fn all_thresholds_crossed() {
        let notified = HashSet::new();
        let thresholds = [50, 75, 90];
        let result = find_crossed_thresholds(95.0, &notified, &thresholds);
        assert_eq!(result, vec![50, 75, 90]);
    }

    #[test]
    fn already_notified_excluded() {
        let mut notified = HashSet::new();
        notified.insert(50);
        let thresholds = [50, 75, 90];
        let result = find_crossed_thresholds(80.0, &notified, &thresholds);
        assert_eq!(result, vec![75]);
    }

    #[test]
    fn exact_boundary_value_at_threshold() {
        let notified = HashSet::new();
        let thresholds = [50, 75, 90];
        // Exactly at 50% should trigger
        let result = find_crossed_thresholds(50.0, &notified, &thresholds);
        assert_eq!(result, vec![50]);
    }

    #[test]
    fn just_below_threshold() {
        let notified = HashSet::new();
        let thresholds = [50, 75, 90];
        let result = find_crossed_thresholds(49.999, &notified, &thresholds);
        assert!(result.is_empty());
    }

    #[test]
    fn all_already_notified() {
        let mut notified = HashSet::new();
        notified.insert(50);
        notified.insert(75);
        notified.insert(90);
        let thresholds = [50, 75, 90];
        let result = find_crossed_thresholds(100.0, &notified, &thresholds);
        assert!(result.is_empty());
    }

    #[test]
    fn empty_thresholds_list() {
        let notified = HashSet::new();
        let thresholds: [u8; 0] = [];
        let result = find_crossed_thresholds(100.0, &notified, &thresholds);
        assert!(result.is_empty());
    }

    #[test]
    fn zero_percentage() {
        let notified = HashSet::new();
        let thresholds = [50, 75, 90];
        let result = find_crossed_thresholds(0.0, &notified, &thresholds);
        assert!(result.is_empty());
    }
}
