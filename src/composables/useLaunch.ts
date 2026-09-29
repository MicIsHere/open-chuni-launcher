import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useI18n } from "@/i18n";
import { useSettings } from "@/composables/useSettings";

/** 日志上限：超出后丢弃最早的记录 */
const MAX_LOG_LINES = 500;
const LOGS_KEY = "launch-logs";

/** 会话日志与运行状态是全局单例：切换页面、重新挂载都不会丢失 */
const logs = ref<string[]>(loadLogs());
const running = ref(false);
const unlisteners: Array<() => void> = [];
let initPromise: Promise<void> | undefined;

function loadLogs(): string[] {
  try {
    const stored = JSON.parse(localStorage.getItem(LOGS_KEY) ?? "[]");
    return Array.isArray(stored) ? stored.slice(-MAX_LOG_LINES) : [];
  } catch {
    return [];
  }
}

function appendLog(message: string) {
  logs.value.push(message);
  if (logs.value.length > MAX_LOG_LINES) {
    logs.value.splice(0, logs.value.length - MAX_LOG_LINES);
  }
  localStorage.setItem(LOGS_KEY, JSON.stringify(logs.value));
}

function isTauri(): boolean {
  return "__TAURI_INTERNALS__" in window;
}

/* 注册后端事件监听并同步初始状态。应用生命周期内只执行一次，
   监听器随进程存续，不需要手动解除。 */
async function init(): Promise<void> {
  if (!isTauri()) return;

  unlisteners.push(
    await listen<string>("launch://log", (event) => appendLog(event.payload)),
    await listen<boolean>("launch://state", (event) => {
      running.value = event.payload;
    }),
  );
  running.value = await invoke<boolean>("is_running");
}

export function useLaunch() {
  const { t } = useI18n();
  const { settings } = useSettings();

  // 初始化只执行一次；显式接住 Promise 避免悬空 Promise 告警
  initPromise ??= init().catch((error) => appendLog(String(error)));

  async function launch() {
    if (!settings.value.gamePath) {
      appendLog(t("home.needGamePath"));
      return;
    }
    running.value = true;
    try {
      await invoke("launch_game", {
        gameDir: settings.value.gamePath,
        dlls: settings.value.gameDlls,
      });
    } catch (error) {
      appendLog(String(error));
    } finally {
      running.value = false;
    }
  }

  function stop() {
    invoke("stop_game").catch((error) => appendLog(String(error)));
  }

  return { logs, running, launch, stop };
}
