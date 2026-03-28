use crate::models::{ScheduleSettings, WidgetSettings};
use crate::state::AppState;
use chrono::Local;
use tauri::{AppHandle, Manager};
use tauri_plugin_store::StoreExt;

/// スケジューラを起動（設定に基づき定刻に Claude CLI を起動）
pub fn start_scheduler(app_handle: AppHandle) {
    let handle = app_handle.clone();

    let task = tauri::async_runtime::spawn(async move {
        loop {
            let settings = load_schedule_settings(&handle);
            if !settings.enabled {
                // 無効なら10秒ごとに再チェック
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                continue;
            }

            let now = Local::now();
            let target = now
                .date_naive()
                .and_hms_opt(
                    u32::from(settings.wake_hour),
                    u32::from(settings.wake_minute),
                    0,
                );

            let Some(target_time) = target else {
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                continue;
            };

            let target_dt = target_time
                .and_local_timezone(Local)
                .single();

            let Some(target_dt) = target_dt else {
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                continue;
            };

            let duration = if target_dt > now {
                (target_dt - now).to_std().unwrap_or(std::time::Duration::from_secs(60))
            } else {
                // 今日の時刻を過ぎていたら明日
                let tomorrow = target_dt + chrono::Duration::days(1);
                (tomorrow - now).to_std().unwrap_or(std::time::Duration::from_secs(60))
            };

            eprintln!(
                "Scheduler: next wake-up in {} seconds (at {:02}:{:02})",
                duration.as_secs(),
                settings.wake_hour,
                settings.wake_minute
            );

            tokio::time::sleep(duration).await;

            // 再度設定を確認（スリープ中に無効化された可能性）
            let settings = load_schedule_settings(&handle);
            if !settings.enabled {
                continue;
            }

            eprintln!("Scheduler: waking up Claude CLI...");
            spawn_claude(&settings);
        }
    });

    // ハンドルを保存
    if let Some(state) = app_handle.try_state::<AppState>() {
        if let Ok(mut guard) = state.lock() {
            if let Some(old) = guard.scheduler_handle.take() {
                old.abort();
            }
            guard.scheduler_handle = Some(task);
        }
    }
}

/// スケジューラを再起動（設定変更時に呼ぶ）
pub fn restart_scheduler(app_handle: AppHandle) {
    // 既存タスクをキャンセル
    if let Some(state) = app_handle.try_state::<AppState>() {
        if let Ok(mut guard) = state.lock() {
            if let Some(old) = guard.scheduler_handle.take() {
                old.abort();
            }
        }
    }
    start_scheduler(app_handle);
}

fn load_schedule_settings(app_handle: &AppHandle) -> ScheduleSettings {
    let Ok(store) = app_handle.store("settings.json") else {
        return ScheduleSettings::default();
    };

    match store.get("settings") {
        Some(value) => {
            let ws: WidgetSettings = serde_json::from_value(value).unwrap_or_default();
            ws.schedule_settings.unwrap_or_default()
        }
        None => ScheduleSettings::default(),
    }
}

/// コマンドが安全かどうかを判定（パス区切り・空白・メタ文字を拒否）
pub fn is_safe_command(cmd: &str) -> bool {
    !cmd.is_empty()
        && !cmd.contains('/')
        && !cmd.contains('\\')
        && !cmd.contains(' ')
        && !cmd.contains(';')
        && !cmd.contains('&')
}

/// プロンプトを500文字に制限
pub fn sanitize_prompt(prompt: &str) -> String {
    prompt.chars().take(500).collect()
}

fn spawn_claude(settings: &ScheduleSettings) {
    let cmd = &settings.command;

    if !is_safe_command(cmd) {
        eprintln!("Scheduler: Rejected unsafe command: {cmd}");
        return;
    }

    let cmd = cmd.clone();
    let prompt = sanitize_prompt(&settings.prompt);

    std::thread::spawn(move || {
        let result = std::process::Command::new(&cmd)
            .args(["-p", &prompt])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();

        match result {
            Ok(mut child) => {
                eprintln!("Scheduler: Claude CLI spawned (pid: {})", child.id());
                let _ = child.wait();
            }
            Err(e) => {
                eprintln!("Scheduler: Failed to spawn '{cmd}': {e}");
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_command_valid() {
        assert!(is_safe_command("claude"));
        assert!(is_safe_command("my-cli"));
        assert!(is_safe_command("claude.exe"));
    }

    #[test]
    fn safe_command_empty_rejected() {
        assert!(!is_safe_command(""));
    }

    #[test]
    fn safe_command_forward_slash_rejected() {
        assert!(!is_safe_command("/usr/bin/claude"));
        assert!(!is_safe_command("../claude"));
    }

    #[test]
    fn safe_command_backslash_rejected() {
        assert!(!is_safe_command("C:\\claude"));
        assert!(!is_safe_command("..\\claude"));
    }

    #[test]
    fn safe_command_semicolon_rejected() {
        assert!(!is_safe_command("claude;rm"));
        assert!(!is_safe_command(";malicious"));
    }

    #[test]
    fn safe_command_ampersand_rejected() {
        assert!(!is_safe_command("claude&rm"));
        assert!(!is_safe_command("&&bad"));
    }

    #[test]
    fn safe_command_space_rejected() {
        assert!(!is_safe_command("claude --flag"));
        assert!(!is_safe_command("some command"));
    }

    #[test]
    fn sanitize_prompt_short_unchanged() {
        let prompt = "hello world";
        assert_eq!(sanitize_prompt(prompt), "hello world");
    }

    #[test]
    fn sanitize_prompt_exactly_500_chars() {
        let prompt: String = "a".repeat(500);
        assert_eq!(sanitize_prompt(&prompt).len(), 500);
    }

    #[test]
    fn sanitize_prompt_truncates_at_500() {
        let prompt: String = "b".repeat(600);
        let sanitized = sanitize_prompt(&prompt);
        assert_eq!(sanitized.len(), 500);
        assert!(sanitized.chars().all(|c| c == 'b'));
    }

    #[test]
    fn sanitize_prompt_multibyte_chars() {
        // Each char is 3 bytes in UTF-8, but take(500) counts chars not bytes
        let prompt: String = "\u{3042}".repeat(600); // 'あ' repeated
        let sanitized = sanitize_prompt(&prompt);
        assert_eq!(sanitized.chars().count(), 500);
    }
}
