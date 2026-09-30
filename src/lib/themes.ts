import type { Component } from "vue";
import { Moon, Sun, SunMoon } from "@lucide/vue";

export interface ThemeDefinition {
  id: string;
  icon: Component;
  dark: boolean;
  /** 跟随系统主题：dark 值在应用时按系统偏好动态解析 */
  followsSystem?: boolean;
}

export const themes: ThemeDefinition[] = [
  { id: "system", icon: SunMoon, dark: false, followsSystem: true },
  { id: "light", icon: Sun, dark: false },
  { id: "dark", icon: Moon, dark: true },
];
