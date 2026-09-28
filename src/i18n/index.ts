import { computed, ref } from "vue";
import type { Ref } from "vue";
import { parseLang } from "@/i18n/parser";

const STORAGE_KEY = "language";
const FALLBACK_LOCALE = "zh-CN";

export interface LocaleDefinition {
  id: string;
  label: string;
}

const sources = import.meta.glob<string>("../locales/*.lang", {
  query: "?raw",
  import: "default",
  eager: true,
});

const messages: Record<string, Record<string, string>> = {};
for (const [path, source] of Object.entries(sources)) {
  const id = /([^/]+)\.lang$/.exec(path)?.[1];
  if (id) messages[id] = parseLang(source);
}

export const locales: LocaleDefinition[] = Object.keys(messages)
  .sort()
  .map((id) => ({ id, label: messages[id]["language.name"] ?? id }));

function findLocale(candidate: string | undefined): string | undefined {
  if (!candidate) return undefined;
  const exact = locales.some((locale) => locale.id === candidate);
  if (exact) return candidate;
  return locales.find((locale) => locale.id.split("-")[0] === candidate.split("-")[0])?.id;
}

function resolveInitialLocale(): string {
  const stored = findLocale(localStorage.getItem(STORAGE_KEY) ?? undefined);
  if (stored) return stored;
  const system = findLocale(typeof navigator !== "undefined" ? navigator.language : undefined);
  return system ?? FALLBACK_LOCALE;
}

const currentLocale: Ref<string> = ref(resolveInitialLocale());
document.documentElement.lang = currentLocale.value;

export function useI18n() {
  function setLocale(id: string): void {
    if (!messages[id]) return;
    currentLocale.value = id;
    localStorage.setItem(STORAGE_KEY, id);
    document.documentElement.lang = id;
  }

  function t(key: string, params?: Record<string, string | number>): string {
    const value =
      messages[currentLocale.value]?.[key] ?? messages[FALLBACK_LOCALE]?.[key] ?? key;
    if (!params) return value;
    return value.replace(/\{(\w+)}/g, (match, name: string) =>
      name in params ? String(params[name]) : match,
    );
  }

  return {
    locales,
    locale: computed(() => currentLocale.value),
    setLocale,
    t,
  };
}
