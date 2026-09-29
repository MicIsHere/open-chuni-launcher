import { ref, watch } from "vue";
import { loadPlugins } from "@/lib/plugins";
import type { GamePlugin } from "@/lib/plugins";
import { useNotifications } from "@/composables/useNotifications";
import { useI18n } from "@/i18n";

export interface AppSettings {
  gamePath: string;
  launchArgs: string;
  launchTimeoutSeconds: number;
  plugins: GamePlugin[];
}

const DEFAULTS: AppSettings = {
  gamePath: "",
  launchArgs: "",
  launchTimeoutSeconds: 30,
  plugins: loadPlugins(),
};

const STORAGE_KEY = "settings";

function load(): AppSettings {
  try {
    const stored = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? "{}");
    if (typeof stored !== "object" || stored === null || Array.isArray(stored)) {
      useNotifications().notify("warning", useI18n().t("notifications.settingsReset"));
      return { ...DEFAULTS, plugins: loadPlugins() };
    }
    const { gameDlls, plugins, ...savedSettings } = stored;
    return { ...DEFAULTS, ...savedSettings, plugins: loadPlugins(plugins, gameDlls) };
  } catch (error) {
    useNotifications().notifyError(error, useI18n().t("notifications.settingsLoadError"));
    return { ...DEFAULTS, plugins: loadPlugins() };
  }
}

const settings = ref<AppSettings>(load());

watch(
  settings,
  (value) => {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(value));
    } catch (error) {
      useNotifications().notifyError(error, useI18n().t("notifications.settingsSaveError"));
    }
  },
  { deep: true },
);

export function useSettings() {
  return { settings };
}
