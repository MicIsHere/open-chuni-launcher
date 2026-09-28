import type { Component } from "vue";
import { Moon, Sun } from "@lucide/vue";

export interface ThemeDefinition {
  id: string;
  icon: Component;
  dark: boolean;
}

export const themes: ThemeDefinition[] = [
  { id: "light", icon: Sun, dark: false },
  { id: "dark", icon: Moon, dark: true },
];
