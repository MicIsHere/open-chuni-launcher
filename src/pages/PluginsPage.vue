<script setup lang="ts">
import { computed, nextTick, onMounted, ref } from "vue";
import { FilePlus2, FolderInput, LockKeyhole, Plus, Puzzle, Trash2, X } from "@lucide/vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { Switch } from "@/components/ui/switch";
import { useSettings } from "@/composables/useSettings";
import { useNotifications } from "@/composables/useNotifications";
import { useI18n } from "@/i18n";
import { resolveLocalizedText } from "@/i18n";
import { addPluginDlls, BUILTIN_PLUGINS, createPlugin, dllFileName, isBuiltinPlugin, mergePluginInfos } from "@/lib/plugins";
import type { GamePlugin, PluginInfo } from "@/lib/plugins";

const { t, locale } = useI18n();
const { settings } = useSettings();
const { notify, notifyError } = useNotifications();
const isTauri = "__TAURI_INTERNALS__" in window;
const adding = ref(false);
const importing = ref(false);
const newDll = ref("");
const newDescription = ref("");

// 进入页面时刷新内置插件目录：补全来源路径与清单信息
onMounted(async () => {
  if (!isTauri) return;
  try {
    const infos = await invoke<PluginInfo[]>("list_plugin_dlls");
    mergePluginInfos(settings.value.plugins, infos);
  } catch (error) {
    notifyError(error, t("plugins.importError"));
  }
});

const groups = computed(() => [
  { id: "builtin", titleKey: "plugins.builtin", plugins: settings.value.plugins.filter(isBuiltinPlugin) },
  { id: "custom", titleKey: "plugins.custom", plugins: settings.value.plugins.filter((plugin) => !isBuiltinPlugin(plugin)) },
]);
const enabledCount = computed(() => settings.value.plugins.filter((plugin) => plugin.enabled).length);

function pluginName(plugin: GamePlugin): string {
  // 名称来自清单（只读，随语言切换）；清单缺失时回落到 i18n 名称或文件名
  return (
    resolveLocalizedText(plugin.name, locale.value).trim() || defaultPluginName(plugin)
  );
}

function pluginDescription(plugin: GamePlugin): string {
  return resolveLocalizedText(plugin.description, locale.value);
}

// 可编辑描述：自定义插件写当前语言的条目；语言中立字符串保持原形态
function editableDescription(plugin: GamePlugin): string {
  return resolveLocalizedText(plugin.description, locale.value);
}

function setEditableDescription(plugin: GamePlugin, value: string) {
  if (plugin.description === undefined || typeof plugin.description === "string") {
    plugin.description = value;
    return;
  }
  plugin.description = { ...plugin.description, [locale.value]: value };
}

function defaultPluginName(plugin: GamePlugin): string {
  const builtin = BUILTIN_PLUGINS.find((item) => item.id === plugin.id);
  return builtin ? t(builtin.nameKey) : dllFileName(plugin.dll).slice(0, -4);
}

async function showAddForm() {
  adding.value = true;
  await nextTick();
  document.getElementById("plugin-new-dll")?.focus();
}

function cancelAdd() {
  adding.value = false;
  newDll.value = "";
  newDescription.value = "";
}

function addDll() {
  const plugin = createPlugin(newDll.value);
  if (!plugin) {
    const isCore = dllFileName(newDll.value.trim()).toLowerCase() === "chusanhook.dll";
    notify(isCore ? "info" : "error", t(isCore ? "plugins.coreDll" : "plugins.invalidDll"));
    return;
  }
  if (addPluginDlls(settings.value.plugins, [newDll.value], newDescription.value.trim()) === 0) {
    notify("warning", t("plugins.duplicate"));
    return;
  }
  notify("success", t("notifications.pluginAdded", { name: pluginName(plugin) }));
  newDll.value = "";
  newDescription.value = "";
  adding.value = false;
}

function removePlugin(plugin: GamePlugin) {
  if (isBuiltinPlugin(plugin)) return;
  settings.value.plugins = settings.value.plugins.filter((item) => item.id !== plugin.id);
  notify("info", t("notifications.pluginRemoved", { name: pluginName(plugin) }));
}

async function importDlls(fromFolder: boolean) {
  if (!isTauri || importing.value) return;
  importing.value = true;
  try {
    const selected = await open(fromFolder
      ? { directory: true, multiple: false, title: t("plugins.importFolder") }
      : { multiple: true, filters: [{ name: t("plugins.dllFiles"), extensions: ["dll"] }], title: t("plugins.importFiles") });
    if (!selected) return;
    const infos = fromFolder
      ? await invoke<PluginInfo[]>("list_plugin_dlls", { directory: selected })
      : (Array.isArray(selected) ? selected : [selected]).map((path) => ({
          file: dllFileName(path),
          path,
          name: dllFileName(path).slice(0, -4),
          description: "",
          hasManifest: false,
        }));
    if (infos.length === 0) {
      notify("info", t("plugins.emptyFolder"));
      return;
    }
    const added = mergePluginInfos(settings.value.plugins, infos);
    notify(added > 0 ? "success" : "warning", t("plugins.importResult", {
      added,
      skipped: infos.length - added,
    }));
  } catch (cause) {
    notifyError(cause, t("notifications.importError"));
  } finally {
    importing.value = false;
  }
}
</script>

<template>
  <div class="plugins-page">
    <div class="plugins-toolbar">
      <div class="plugins-toolbar-actions">
        <Button :disabled="adding" @click="showAddForm">
          <Plus aria-hidden="true" />
          {{ t("plugins.add") }}
        </Button>
        <Button variant="outline" :disabled="!isTauri || importing" @click="importDlls(false)">
          <FilePlus2 aria-hidden="true" />
          {{ t("plugins.importFiles") }}
        </Button>
        <Button variant="outline" :disabled="!isTauri || importing" @click="importDlls(true)">
          <FolderInput aria-hidden="true" />
          {{ t("plugins.importFolder") }}
        </Button>
      </div>
      <span class="plugins-count">
        {{ t("plugins.enabledCount", { count: enabledCount, total: settings.plugins.length }) }}
      </span>
    </div>

    <form v-if="adding" class="plugins-add-form" @submit.prevent="addDll" @keydown.esc.prevent="cancelAdd">
      <div class="plugins-add-fields">
        <div class="plugin-field">
          <label for="plugin-new-dll" class="plugin-field-label">{{ t("plugins.dllPath") }}</label>
          <Input id="plugin-new-dll" v-model="newDll" :placeholder="t('plugins.dllPlaceholder')" required />
        </div>
        <div class="plugin-field">
          <label for="plugin-new-description" class="plugin-field-label">{{ t("plugins.description") }}</label>
          <Input id="plugin-new-description" v-model="newDescription" :placeholder="t('plugins.noDescription')" />
        </div>
      </div>
      <div class="plugins-toolbar-actions">
        <Button type="submit" :disabled="!newDll.trim()">
          <Plus aria-hidden="true" />
          {{ t("plugins.add") }}
        </Button>
        <Button type="button" variant="ghost" @click="cancelAdd">
          <X aria-hidden="true" />
          {{ t("plugins.cancel") }}
        </Button>
      </div>
    </form>

    <section v-for="group in groups" :key="group.id" class="plugins-section" :aria-labelledby="`plugins-${group.id}`">
      <h2 :id="`plugins-${group.id}`" class="plugins-section-title">
        {{ t(group.titleKey) }} <span class="plugins-count">{{ group.plugins.length }}</span>
      </h2>
      <p v-if="group.plugins.length === 0" class="plugins-empty">{{ t("plugins.empty") }}</p>
      <article v-for="plugin in group.plugins" :key="plugin.id" class="plugin-card" :data-enabled="plugin.enabled" :data-plugin-id="plugin.id">
        <div class="plugin-card-header">
          <img v-if="plugin.icon" :src="plugin.icon" class="plugin-card-icon" alt="" />
          <Puzzle v-else class="plugin-card-icon" aria-hidden="true" />
          <div class="plugin-card-heading">
            <!-- 内置插件名称与描述均来自同名 JSON 清单，只读 -->
            <h3 class="plugin-card-name">{{ pluginName(plugin) }}</h3>
            <code class="plugin-card-dll">{{ plugin.dll }}</code>
          </div>
          <div class="plugin-card-actions">
            <Tooltip v-if="isBuiltinPlugin(plugin)">
              <TooltipTrigger as-child>
                <span class="plugin-builtin-mark" tabindex="0" :aria-label="t('plugins.builtin')">
                  <LockKeyhole class="size-3.5" aria-hidden="true" />
                </span>
              </TooltipTrigger>
              <TooltipContent>{{ t("plugins.builtin") }}</TooltipContent>
            </Tooltip>
            <Switch
              v-model="plugin.enabled"
              :aria-label="t('plugins.toggle', { name: pluginName(plugin) })"
            />
            <Tooltip v-if="!isBuiltinPlugin(plugin)">
              <TooltipTrigger as-child>
                <Button variant="ghost" size="icon" :aria-label="t('plugins.remove', { name: pluginName(plugin) })" @click="removePlugin(plugin)">
                  <Trash2 aria-hidden="true" />
                </Button>
              </TooltipTrigger>
              <TooltipContent>{{ t("plugins.remove", { name: pluginName(plugin) }) }}</TooltipContent>
            </Tooltip>
          </div>
        </div>
        <div class="plugin-field">
          <!-- 内置插件描述来自清单，不可更改且随语言切换；自定义插件可编辑 -->
          <p
            v-if="plugin.descriptionLocked"
            class="plugin-description-readonly"
            :title="t('plugins.descriptionLocked')"
          >
            {{ pluginDescription(plugin) || t("plugins.noDescription") }}
          </p>
          <textarea
            v-else
            :value="editableDescription(plugin)"
            class="plugin-description-field"
            rows="2"
            :placeholder="t('plugins.noDescription')"
            :aria-label="t('plugins.editDescription', { name: pluginName(plugin) })"
            @input="setEditableDescription(plugin, ($event.target as HTMLTextAreaElement).value)"
          />
        </div>
      </article>
    </section>
  </div>
</template>
