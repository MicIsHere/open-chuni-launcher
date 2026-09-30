import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useI18n } from "@/i18n";
import { useSettings } from "@/composables/useSettings";
import { useNotifications } from "@/composables/useNotifications";
import type { GamePlugin, PluginInfo } from "@/lib/plugins";
import { dllFileName, getGameDlls, mergePluginInfos } from "@/lib/plugins";

/** 日志上限：超出后丢弃最早的记录 */
const MAX_LOG_LINES = 500;

interface LaunchReport {
  missing_dlls: string[];
}

/**
 * 会话日志与运行状态是全局单例：切换页面、重新挂载都不会丢失。
 * 日志不持久化——启动器重启、每次启动游戏时自动清空。
 */
const logs = ref<string[]>([]);
const running = ref(false);
const unlisteners: Array<() => void> = [];
let initPromise: Promise<void> | undefined;

function appendLog(message: string) {
  logs.value.push(message);
  if (logs.value.length > MAX_LOG_LINES) {
    logs.value.splice(0, logs.value.length - MAX_LOG_LINES);
  }
}

function isTauri(): boolean {
  return "__TAURI_INTERNALS__" in window;
}

/* 注册后端事件监听并同步初始状态。应用生命周期内只执行一次，
   监听器随进程存续，不需要手动解除。 */
async function init(): Promise<void> {
  if (!isTauri()) return;
  const { notify } = useNotifications();
  const { t } = useI18n();
  const { settings } = useSettings();

  unlisteners.push(
    await listen<string>("launch://log", (event) => appendLog(event.payload)),
    await listen<boolean>("launch://state", (event) => {
      running.value = event.payload;
      notify(event.payload ? "success" : "info", t(event.payload
        ? "notifications.gameStarted" : "notifications.gameExited"));
    }),
  );
  running.value = await invoke<boolean>("is_running");

  // 启动即扫描内置插件目录，刷新插件的来源路径与清单信息（供启动复制用）
  const infos = await invoke<PluginInfo[]>("list_plugin_dlls");
  mergePluginInfos(settings.value.plugins, infos);
}

export function useLaunch() {
  const { t } = useI18n();
  const { settings } = useSettings();
  const { notify, notifyError } = useNotifications();

  // 初始化只执行一次；显式接住 Promise 避免悬空 Promise 告警
  initPromise ??= init().catch((error) => {
    appendLog(String(error));
    notifyError(error, t("notifications.launchInitError"));
  });

  async function launch() {
    // 每次启动游戏前清空上一局的日志
    logs.value = [];
    if (!settings.value.gamePath) {
      const message = t("home.needGamePath");
      appendLog(message);
      notify("warning", message);
      return;
    }
    if (!isTauri()) {
      notify("info", t("notifications.desktopOnly"));
      return;
    }
    running.value = true;
    try {
      // 内置插件 DLL 位于资源目录，启动时由后端复制到游戏目录
      const builtinPlugins = settings.value.plugins
        .filter((plugin): plugin is GamePlugin & { source: string } => Boolean(plugin.source))
        .map((plugin) => ({ dll: dllFileName(plugin.dll), source: plugin.source }));
      const report = await invoke<LaunchReport>("launch_game", {
        gameDir: settings.value.gamePath,
        dlls: getGameDlls(settings.value.plugins),
        builtinPlugins,
        launchTimeoutSecs: settings.value.launchTimeoutSeconds,
      });
      if (report.missing_dlls.length > 0) {
        notify("warning", t("notifications.missingDlls", { dlls: report.missing_dlls.join(", ") }));
      }
    } catch (error) {
      appendLog(String(error));
      notifyError(error, t("notifications.launchError"));
    } finally {
      running.value = false;
    }
  }

  function stop() {
    if (!isTauri()) return;
    invoke("stop_game").catch((error) => {
      appendLog(String(error));
      notifyError(error, t("notifications.stopError"));
    });
  }

  return { logs, running, launch, stop };
}
