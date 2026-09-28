import type { Component } from "vue";
import { House, Settings } from "@lucide/vue";

export interface NavSection {
  id: string;
  labelKey: string;
  icon: Component;
}

export const navSections: NavSection[] = [
  { id: "home", labelKey: "nav.home", icon: House },
  { id: "settings", labelKey: "nav.settings", icon: Settings },
];
