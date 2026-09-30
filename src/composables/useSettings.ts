import { ref, watch } from "vue";
import { loadPlugins } from "@/lib/plugins";
import type { GamePlugin } from "@/lib/plugins";
import type { ServerConfig } from "@/lib/servers";
import {
  DEFAULT_AIME,
  DEFAULT_GFX,
  DEFAULT_GPIO,
  DEFAULT_IO3,
  DEFAULT_VFS,
} from "@/lib/segatools";
import type {
  AimeConfig,
  GfxConfig,
  GpioConfig,
  Io3Config,
  VfsConfig,
} from "@/lib/segatools";
import { useNotifications } from "@/composables/useNotifications";
import { useI18n } from "@/i18n";

export interface AppSettings {
  gamePath: string;
  launchArgs: string;
  launchTimeoutSeconds: number;
  plugins: GamePlugin[];
  server: ServerConfig;
  vfs: VfsConfig;
  gpio: GpioConfig;
  gfx: GfxConfig;
  aime: AimeConfig;
  io3: Io3Config;
}

const DEFAULTS: AppSettings = {
  gamePath: "",
  launchArgs: "",
  launchTimeoutSeconds: 30,
  plugins: loadPlugins(),
  server: { preset: "custom", customDns: "", customAimeDb: "", keychip: "" },
  vfs: DEFAULT_VFS,
  gpio: DEFAULT_GPIO,
  gfx: DEFAULT_GFX,
  aime: DEFAULT_AIME,
  io3: DEFAULT_IO3,
};

const STORAGE_KEY = "settings";

/** 把存档对象合并到默认值上，缺失字段回落默认，新增字段随存档保留。 */
function mergeConfig<T extends object>(defaults: T, saved: unknown): T {
  if (typeof saved !== "object" || saved === null || Array.isArray(saved)) {
    return { ...defaults };
  }
  return { ...defaults, ...(saved as Partial<T>) };
}

function load(): AppSettings {
  try {
    const stored = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? "{}");
    if (typeof stored !== "object" || stored === null || Array.isArray(stored)) {
      useNotifications().notify("warning", useI18n().t("notifications.settingsReset"));
      return { ...DEFAULTS, plugins: loadPlugins() };
    }
    const { gameDlls, plugins, ...savedSettings } = stored;
    return {
      ...DEFAULTS,
      ...savedSettings,
      plugins: loadPlugins(plugins, gameDlls),
      server: mergeConfig(DEFAULTS.server, savedSettings.server),
      vfs: mergeConfig(DEFAULTS.vfs, savedSettings.vfs),
      gpio: mergeConfig(DEFAULTS.gpio, savedSettings.gpio),
      gfx: mergeConfig(DEFAULTS.gfx, savedSettings.gfx),
      aime: mergeConfig(DEFAULTS.aime, savedSettings.aime),
      io3: mergeConfig(DEFAULTS.io3, savedSettings.io3),
    };
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
