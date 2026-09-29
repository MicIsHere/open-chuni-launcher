import type { Component } from "vue";
import { House, Puzzle, ScrollText, Settings } from "@lucide/vue";

export interface NavSection {
  id: string;
  /** i18n key of the section title. */
  labelKey: string;
  icon: Component;
}

export const navSections: NavSection[] = [
  { id: "home", labelKey: "nav.home", icon: House },
  { id: "plugins", labelKey: "nav.plugins", icon: Puzzle },
  { id: "logs", labelKey: "nav.logs", icon: ScrollText },
  { id: "settings", labelKey: "nav.settings", icon: Settings },
];
