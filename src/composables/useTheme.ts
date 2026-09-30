import { ref } from "vue";
import type { Ref } from "vue";
import { themes } from "@/lib/themes";
import type { ThemeDefinition } from "@/lib/themes";

const STORAGE_KEY = "theme";
const SYSTEM_THEME_ID = "system";

const currentTheme: Ref<ThemeDefinition> = ref(resolveInitialTheme());
let initialized = false;

function findTheme(id: string): ThemeDefinition | undefined {
  return themes.find((theme) => theme.id === id);
}

function prefersDark(): boolean {
  return window.matchMedia("(prefers-color-scheme: dark)").matches;
}

function resolveInitialTheme(): ThemeDefinition {
  // 未选择过主题（或存值失效）时默认跟随系统
  return findTheme(localStorage.getItem(STORAGE_KEY) ?? "") ?? themes[0];
}

const THEME_SWITCH_DURATION_MS = 250;
let themeSwitchTimer: number | undefined;

function applyTheme(theme: ThemeDefinition): void {
  const root = document.documentElement;
  root.dataset.theme = theme.id;
  root.classList.toggle("dark", theme.followsSystem ? prefersDark() : theme.dark);

  root.classList.add("theme-switching");
  window.clearTimeout(themeSwitchTimer);
  themeSwitchTimer = window.setTimeout(() => {
    root.classList.remove("theme-switching");
  }, THEME_SWITCH_DURATION_MS);
}

export function useTheme() {
  if (!initialized) {
    initialized = true;
    applyTheme(currentTheme.value);

    window.matchMedia("(prefers-color-scheme: dark)").addEventListener("change", () => {
      const stored = localStorage.getItem(STORAGE_KEY);
      // 固定了具体主题后不再跟随系统；「跟随系统」或从未选择时继续跟随
      if (stored && stored !== SYSTEM_THEME_ID) return;
      applyTheme(currentTheme.value);
    });
  }

  function setTheme(id: string): void {
    const theme = findTheme(id);
    if (!theme) return;
    currentTheme.value = theme;
    applyTheme(theme);
    localStorage.setItem(STORAGE_KEY, id);
  }

  return { themes, currentTheme, setTheme };
}
