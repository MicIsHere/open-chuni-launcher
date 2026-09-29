import { computed, ref } from "vue";
import type { Ref } from "vue";
import { parseLang } from "@/i18n/parser";

const STORAGE_KEY = "language";
const FALLBACK_LOCALE = "zh-CN";

export interface LocaleDefinition {
  id: string;
  label: string;
}

/** 单条文本的两种形态：语言中立字符串，或 {locale: text} 多语言映射 */
export type LocalizedText = string | Record<string, string>;

/**
 * 解析多语言文本（用于插件清单等数据驱动的文案）：
 * 精确语言 → 同语言前缀（en-US 匹配 en-*）→ 回退语言 → 首个非空值。
 * 随 locale 变化响应式生效。
 */
export function resolveLocalizedText(
  value: LocalizedText | undefined,
  locale: string,
): string {
  if (!value) return "";
  if (typeof value === "string") return value;
  const entries = Object.entries(value).filter(
    ([, text]) => typeof text === "string" && text.trim() !== "",
  );
  if (entries.length === 0) return "";
  const exact = entries.find(([key]) => key === locale);
  if (exact) return exact[1];
  const base = locale.split("-")[0];
  const sameBase = entries.find(([key]) => key.split("-")[0] === base);
  if (sameBase) return sameBase[1];
  const fallback = entries.find(([key]) => key === FALLBACK_LOCALE);
  if (fallback) return fallback[1];
  return entries[0][1];
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
