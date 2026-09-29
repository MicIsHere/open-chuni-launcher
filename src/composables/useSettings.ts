import { ref, watch } from "vue";

export interface AppSettings {
  gamePath: string;
  launchArgs: string;
  launchTimeoutSeconds: number;
  gameDlls: string[];
}

const DEFAULTS: AppSettings = {
  gamePath: "",
  launchArgs: "",
  launchTimeoutSeconds: 30,
  gameDlls: ["chusanhook.dll", "duolinguo.dll", "hook.dll"],
};

const STORAGE_KEY = "settings";


function load(): AppSettings {
  try {
    const stored = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? "{}");
    return { ...DEFAULTS, ...stored };
  } catch {
    return { ...DEFAULTS };
  }
}

const settings = ref<AppSettings>(load());

watch(
  settings,
  (value) => localStorage.setItem(STORAGE_KEY, JSON.stringify(value)),
  { deep: true },
);

export function useSettings() {
  return { settings };
}
