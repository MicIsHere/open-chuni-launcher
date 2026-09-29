export interface GamePlugin {
  id: string;
  dll: string;
  enabled: boolean;
  description: string;
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
      description: typeof entry?.description === "string" ? entry.description : "",
    };
  });

  for (const entry of saved) {
    if (BUILTIN_PLUGINS.some((builtin) => builtin.id === entry.id)) continue;
    if (typeof entry.dll !== "string") continue;
    const plugin = createPlugin(entry.dll);
    if (!plugin || plugins.some((existing) => existing.id === plugin.id)) continue;
    plugin.enabled = entry.enabled !== false;
    plugin.description = typeof entry.description === "string" ? entry.description : "";
    plugins.push(plugin);
  }
  return plugins;
}

export function getGameDlls(plugins: GamePlugin[]): string[] {
  return [CORE_DLL, ...plugins.filter((plugin) => plugin.enabled).map((plugin) => plugin.dll)];
}
