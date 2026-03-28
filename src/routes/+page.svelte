<script lang="ts">
  import Widget from '$lib/components/Widget.svelte';
  import { tokenState } from '$lib/state/token-state.svelte';
  import { settingsState } from '$lib/state/settings-state.svelte';
  import { getCurrentToken } from '$lib/tauri/commands';
  import { onTokenUpdated, onWatcherStatus, onWatcherError } from '$lib/tauri/events';

  // 初期化 + ポーリングフォールバック
  $effect(() => {
    // 初回取得（複数回リトライ）
    let retries = 0;
    let pollInterval: ReturnType<typeof setInterval> | null = null;

    function startPolling() {
      if (pollInterval) return;
      pollInterval = setInterval(() => {
        getCurrentToken()
          .then((data) => {
            if (data) {
              tokenState.update(data);
            }
          })
          .catch(() => {});
      }, 5000);
    }

    function stopPolling() {
      if (pollInterval) {
        clearInterval(pollInterval);
        pollInterval = null;
      }
    }

    function tryLoad() {
      getCurrentToken()
        .then((data) => {
          if (data) {
            tokenState.update(data);
          } else if (retries < 3) {
            retries++;
            setTimeout(tryLoad, 1000);
          } else {
            tokenState.setDisconnected();
            startPolling();
          }
        })
        .catch((e) => {
          console.error('Failed to get token data:', e);
          if (retries < 3) {
            retries++;
            setTimeout(tryLoad, 1000);
          } else {
            tokenState.setDisconnected();
            startPolling();
          }
        });
    }
    tryLoad();

    settingsState.load();

    const unlistenToken = onTokenUpdated((data) => {
      tokenState.update(data);
      stopPolling();
    });
    const unlistenStatus = onWatcherStatus((status) => {
      if (!status.active) {
        tokenState.setDisconnected();
        startPolling();
      }
    });
    const unlistenError = onWatcherError((error) => {
      tokenState.setError(error);
    });

    // Start polling only if not connected
    if (tokenState.status !== 'connected') {
      startPolling();
    }

    return () => {
      unlistenToken.then((fn) => fn());
      unlistenStatus.then((fn) => fn());
      unlistenError.then((fn) => fn());
      stopPolling();
    };
  });

  // テーマ同期
  $effect(() => {
    tokenState.isDark = settingsState.theme === 'dark';
  });
</script>

<Widget />
