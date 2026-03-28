use crate::error::AppResult;
use crate::models::{TokenUsageData, WatcherStatus};
use notify_debouncer_mini::notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebouncedEvent};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

const DEBOUNCE_MS: u64 = 100;

/// token-usage.json のパスを解決
pub fn resolve_token_file_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
        .join("token-usage.json")
}

/// ファイルを読み取り、パースして返す
pub fn read_and_parse(path: &Path) -> AppResult<TokenUsageData> {
    let contents = std::fs::read_to_string(path)?;
    let data = serde_json::from_str::<TokenUsageData>(&contents)?;
    Ok(data)
}

/// ファイルウォッチャーを起動し、変更時にイベントを emit する
pub fn start_file_watcher(
    app_handle: AppHandle,
    token_file_path: PathBuf,
) -> AppResult<notify_debouncer_mini::Debouncer<notify_debouncer_mini::notify::RecommendedWatcher>> {
    let (tx, rx) = mpsc::channel();

    let mut debouncer = new_debouncer(
        Duration::from_millis(DEBOUNCE_MS),
        move |result: Result<Vec<DebouncedEvent>, notify_debouncer_mini::notify::Error>| {
            let _ = tx.send(result);
        },
    )?;

    // 親ディレクトリを監視（ファイル未存在でも対応可能）
    let Some(watch_dir) = token_file_path.parent() else {
        return Err(crate::error::AppError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Token file path has no parent directory",
        )));
    };
    debouncer
        .watcher()
        .watch(watch_dir, RecursiveMode::NonRecursive)?;

    // イベントループをバックグラウンドスレッドで開始
    let handle = app_handle.clone();
    let path = token_file_path.clone();

    tauri::async_runtime::spawn_blocking(move || {
        let _ = handle.emit("watcher-status", WatcherStatus { active: true });

        loop {
            match rx.recv() {
                Ok(Ok(events)) => {
                    // .claudeディレクトリ内の変更を検知したらファイルを再読み込み
                    let relevant = events.iter().any(|e| {
                        e.path == path
                            || e.path.file_name() == path.file_name()
                            || e.path.file_name().map_or(false, |n| n.to_string_lossy().contains("token-usage"))
                    });
                    if relevant && path.exists() {
                        match read_and_parse(&path) {
                            Ok(data) => {
                                if let Some(state) =
                                    handle.try_state::<crate::state::AppState>()
                                {
                                    if let Ok(mut guard) = state.lock() {
                                        guard.token_data = Some(data.clone());
                                    }
                                }
                                let _ = handle.emit("token-updated", &data);

                                // アラートチェック
                                crate::alerts::check_and_notify(&handle, &data);
                                // ヒートマップ記録
                                crate::heatmap::record_usage(&handle, &data);
                                // トレイ更新
                                crate::tray::update_tray(&handle, &data);
                            }
                            Err(e) => {
                                eprintln!("Failed to parse token file: {e}");
                            }
                        }
                    }
                }
                Ok(Err(errors)) => {
                    eprintln!("Watch errors: {errors:?}");
                    let _ = handle.emit("watcher-error", format!("{errors:?}"));
                }
                Err(_) => break,
            }
        }

        let _ = handle.emit("watcher-status", WatcherStatus { active: false });
    });

    Ok(debouncer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_token_file_path_ends_with_correct_path() {
        let path = resolve_token_file_path();
        let path_str = path.to_string_lossy();
        assert!(
            path_str.ends_with(".claude/token-usage.json")
                || path_str.ends_with(".claude\\token-usage.json"),
            "expected path ending with .claude/token-usage.json, got: {path_str}"
        );
    }

    #[test]
    fn read_and_parse_valid_json() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let file_path = dir.path().join("token-usage.json");
        std::fs::write(
            &file_path,
            r#"{
                "context_window": {
                    "total_input_tokens": 100,
                    "total_output_tokens": 50,
                    "used_percentage": 25.0
                },
                "cost": {
                    "total_cost_usd": 0.05
                }
            }"#,
        )
        .expect("write file");

        let data = read_and_parse(&file_path).expect("should parse");
        assert_eq!(data.context_window.total_input_tokens, 100);
        assert_eq!(data.context_window.total_output_tokens, 50);
        assert!((data.context_window.used_percentage - 25.0).abs() < f64::EPSILON);
        assert!((data.cost.total_cost_usd - 0.05).abs() < f64::EPSILON);
    }

    #[test]
    fn read_and_parse_invalid_json() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let file_path = dir.path().join("token-usage.json");
        std::fs::write(&file_path, "not valid json {{{").expect("write file");

        let result = read_and_parse(&file_path);
        assert!(result.is_err());
    }

    #[test]
    fn read_and_parse_real_world_f64_percentage() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let file_path = dir.path().join("token-usage.json");
        std::fs::write(
            &file_path,
            r#"{
                "context_window": {
                    "used_percentage": 57.99999999999999
                },
                "rate_limits": {
                    "five_hour": {
                        "used_percentage": 83.33333333333334,
                        "resets_at": 1700000000
                    }
                }
            }"#,
        )
        .expect("write file");

        let data = read_and_parse(&file_path).expect("should parse f64 percentages");
        assert!(
            (data.context_window.used_percentage - 57.99999999999999).abs() < f64::EPSILON
        );
        let five_hour = data
            .rate_limits
            .expect("rate_limits")
            .five_hour
            .expect("five_hour");
        assert!(
            (five_hour.used_percentage - 83.33333333333334).abs() < f64::EPSILON
        );
    }

    #[test]
    fn read_and_parse_missing_file() {
        let path = Path::new("/nonexistent/path/token-usage.json");
        let result = read_and_parse(path);
        assert!(result.is_err());
    }
}
