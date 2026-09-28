import { ref } from "vue";
import type { Ref } from "vue";
import { themes } from "@/lib/themes";
import type { ThemeDefinition } from "@/lib/themes";

const STORAGE_KEY = "theme";

const currentTheme: Ref<ThemeDefinition> = ref(resolveInitialTheme());
let initialized = false;

function findTheme(id: string): ThemeDefinition | undefined {
  return themes.find((theme) => theme.id === id);
}

function findThemeByDarkness(dark: boolean): ThemeDefinition | undefined {
  return themes.find((theme) => theme.dark === dark);
}

function prefersDark(): boolean {
  return window.matchMedia("(prefers-color-scheme: dark)").matches;
}

function resolveInitialTheme(): ThemeDefinition {
  const stored = localStorage.getItem(STORAGE_KEY);
  return findTheme(stored ?? "") ?? findThemeByDarkness(prefersDark()) ?? themes[0];
}

const THEME_SWITCH_DURATION_MS = 250;
let themeSwitchTimer: number | undefined;

function applyTheme(theme: ThemeDefinition): void {
  const root = document.documentElement;
  root.dataset.theme = theme.id;
  root.classList.toggle("dark", theme.dark);

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

    window
      .matchMedia("(prefers-color-scheme: dark)")
      .addEventListener("change", (event) => {
        if (localStorage.getItem(STORAGE_KEY)) return;
        const next = findThemeByDarkness(event.matches);
        if (next) {
          currentTheme.value = next;
          applyTheme(next);
        }
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
