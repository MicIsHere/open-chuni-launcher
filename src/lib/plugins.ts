import type { LocalizedText } from "@/i18n";

export interface GamePlugin {
  id: string;
  dll: string;
  enabled: boolean;
  /** 清单提供的名称与描述（只读，可为多语言映射） */
  name?: LocalizedText;
  description?: LocalizedText;
  /** 内置插件 DLL 在资源目录中的来源路径，启动时复制到游戏目录 */
  source?: string;
  /** 图标 data URL（来自同名 JSON 清单） */
  icon?: string;
  /** 描述来自清单时锁定，不可编辑 */
  descriptionLocked?: boolean;
}

export interface PluginInfo {
  file: string;
  name?: LocalizedText;
  description?: LocalizedText;
  version?: string;
  author?: string;
  path: string;
  iconDataUrl?: string;
  hasManifest: boolean;
}

/** 清单字段兼容守卫：字符串或多语言映射（过滤非字符串值） */
function toLocalizedText(value: unknown): LocalizedText | undefined {
  if (typeof value === "string") return value;
  if (typeof value === "object" && value !== null && !Array.isArray(value)) {
    const entries = Object.entries(value).filter(([, text]) => typeof text === "string");
    return entries.length > 0 ? Object.fromEntries(entries) : undefined;
  }
  return undefined;
}

export const BUILTIN_PLUGINS = [
  { id: "duolinguo", dll: "duolinguo.dll", nameKey: "plugins.duolinguo.name" },
  { id: "hook", dll: "hook.dll", nameKey: "plugins.hook.name" },
] as const;

const CORE_DLL = "chusanhook.dll";

export function dllFileName(dll: string): string {
  return dll.replace(/\\/g, "/").split("/").pop() ?? "";
}

export function createPlugin(dll: string): GamePlugin | undefined {
  const path = dll.trim();
  const fileName = dllFileName(path);
  if (!/^.+\.dll$/i.test(fileName) || fileName.toLowerCase() === CORE_DLL) return;

  const builtin = BUILTIN_PLUGINS.find((plugin) => plugin.dll === fileName.toLowerCase());
  return {
    id: builtin?.id ?? `custom:${fileName.toLowerCase()}`,
    dll: builtin?.dll ?? path,
    enabled: true,
    description: "",
  };
}

export function isBuiltinPlugin(plugin: GamePlugin): boolean {
  return BUILTIN_PLUGINS.some((builtin) => builtin.id === plugin.id);
}

export function addPluginDlls(plugins: GamePlugin[], dlls: string[], description = ""): number {
  let added = 0;
  for (const dll of dlls) {
    const plugin = createPlugin(dll);
    if (!plugin || plugins.some((existing) => existing.id === plugin.id)) continue;
    plugin.description = description;
    plugins.push(plugin);
    added++;
  }
  return added;
}

/** 用清单信息填充插件字段（名称与描述均来自清单，只读） */
function applyPluginInfo(plugin: GamePlugin, info: PluginInfo) {
  plugin.name = toLocalizedText(info.name);
  plugin.description = toLocalizedText(info.description);
  plugin.descriptionLocked = info.hasManifest || isBuiltinPlugin(plugin);
  plugin.icon = info.iconDataUrl;
  plugin.source = info.path;
}

/** 把后端扫描到的插件清单合并进插件列表：新插件导入，已有插件刷新名称/来源/图标/描述 */
export function mergePluginInfos(plugins: GamePlugin[], infos: PluginInfo[]): number {
  let added = 0;
  for (const info of infos) {
    const id = createPlugin(info.path)?.id;
    if (!id) continue;
    const existing = plugins.find((item) => item.id === id);
    if (existing) {
      applyPluginInfo(existing, info);
    } else {
      const plugin = createPlugin(info.path);
      if (!plugin) continue;
      applyPluginInfo(plugin, info);
      plugins.push(plugin);
      added++;
    }
  }
  return added;
}

export function loadPlugins(savedPlugins?: unknown, legacyDlls?: unknown): GamePlugin[] {
  const legacy = Array.isArray(legacyDlls)
    ? legacyDlls.filter((dll): dll is string => typeof dll === "string")
    : undefined;
  const source: unknown[] = Array.isArray(savedPlugins)
    ? savedPlugins
    : legacy?.map((dll) => ({ dll })) ?? [];
  const saved = source.filter((item): item is Record<string, unknown> =>
    typeof item === "object" && item !== null && !Array.isArray(item),
  );

  // 内置插件始终恢复固定标识与文件名，旧列表中的缺席项迁移为关闭。
  const plugins: GamePlugin[] = BUILTIN_PLUGINS.map((builtin) => {
    const entry = saved.find((item) => item.id === builtin.id || (
      typeof item.dll === "string" && dllFileName(item.dll).toLowerCase() === builtin.dll
    ));
    return {
      id: builtin.id,
      dll: builtin.dll,
      enabled: entry ? entry.enabled !== false : Array.isArray(savedPlugins) || !legacy,
      description: toLocalizedText(entry?.description),
      name: toLocalizedText(entry?.name),
      source: typeof entry?.source === "string" ? entry.source : undefined,
      icon: typeof entry?.icon === "string" ? entry.icon : undefined,
      descriptionLocked: entry?.descriptionLocked !== false,
    };
  });

  for (const entry of saved) {
    if (BUILTIN_PLUGINS.some((builtin) => builtin.id === entry.id)) continue;
    if (typeof entry.dll !== "string") continue;
    const plugin = createPlugin(entry.dll);
    if (!plugin || plugins.some((existing) => existing.id === plugin.id)) continue;
    plugin.enabled = entry.enabled !== false;
    plugin.description = typeof entry.description === "string" ? entry.description : "";
    plugin.name = toLocalizedText(entry.name);
    plugin.source = typeof entry.source === "string" ? entry.source : undefined;
    plugin.icon = typeof entry.icon === "string" ? entry.icon : undefined;
    plugin.descriptionLocked = entry.descriptionLocked === true;
    plugins.push(plugin);
  }
  return plugins;
}

export function getGameDlls(plugins: GamePlugin[]): string[] {
  return [CORE_DLL, ...plugins.filter((plugin) => plugin.enabled).map((plugin) => plugin.dll)];
}
