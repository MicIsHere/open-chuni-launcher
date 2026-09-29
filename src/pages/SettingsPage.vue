<script setup lang="ts">
import { computed, ref } from "vue";
import { FolderOpen, Languages, Palette, Plus, Syringe, Terminal, Timer, X } from "@lucide/vue";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import SettingCard from "@/components/settings/SettingCard.vue";
import SettingNumber from "@/components/settings/controls/SettingNumber.vue";
import SettingPath from "@/components/settings/controls/SettingPath.vue";
import SettingSelect from "@/components/settings/controls/SettingSelect.vue";
import SettingText from "@/components/settings/controls/SettingText.vue";
import { useSettings } from "@/composables/useSettings";
import { useTheme } from "@/composables/useTheme";
import { useI18n } from "@/i18n";

const { t, locales, locale, setLocale } = useI18n();
const { themes, currentTheme, setTheme } = useTheme();
const { settings } = useSettings();

const newDll = ref("");

function addDll() {
  const name = newDll.value.trim();
  if (!name || settings.value.gameDlls.includes(name)) return;
  settings.value.gameDlls.push(name);
  newDll.value = "";
}

function removeDll(dll: string) {
  const index = settings.value.gameDlls.indexOf(dll);
  if (index !== -1) settings.value.gameDlls.splice(index, 1);
}

const themeId = computed({
  get: () => currentTheme.value.id,
  set: (id) => setTheme(id),
});

const themeOptions = computed(() =>
  themes.map((theme) => ({
    value: theme.id,
    label: t(`theme.${theme.id}`),
    icon: theme.icon,
  })),
);

const localeId = computed({
  get: () => locale.value,
  set: (id) => setLocale(id),
});

const localeOptions = locales.map((locale) => ({ value: locale.id, label: locale.label }));
</script>

<template>
  <div class="settings-page">
    <section class="settings-section">
      <h2 class="settings-section-title">
        {{ t("settings.appearance.title") }}
      </h2>

      <SettingCard
        :icon="Palette"
        :title="t('settings.theme.title')"
        :description="t('settings.theme.description')"
      >
        <SettingSelect v-model="themeId" :options="themeOptions" />
      </SettingCard>

      <SettingCard
        :icon="Languages"
        :title="t('settings.language.title')"
        :description="t('settings.language.description')"
      >
        <SettingSelect v-model="localeId" :options="localeOptions" />
      </SettingCard>
    </section>

    <section class="settings-section">
      <h2 class="settings-section-title">
        {{ t("settings.game.title") }}
      </h2>

      <SettingCard
        :icon="FolderOpen"
        :title="t('settings.gamePath.title')"
        :description="t('settings.gamePath.description')"
      >
        <SettingPath
          v-model="settings.gamePath"
          :placeholder="t('settings.gamePath.placeholder')"
          :browse-label="t('settings.gamePath.browse')"
        />
      </SettingCard>

      <SettingCard
        :icon="Syringe"
        :title="t('settings.gameDlls.title')"
        :description="t('settings.gameDlls.description')"
      >
        <div class="flex w-full flex-col gap-1.5 sm:w-64">
          <div
            v-for="dll in settings.gameDlls"
            :key="dll"
            class="flex h-7 items-center gap-1 rounded-md border bg-background px-2"
          >
            <span class="flex-1 truncate text-xs">{{ dll }}</span>
            <Button
              variant="ghost"
              class="size-5 rounded-sm p-0 text-muted-foreground hover:text-foreground"
              :aria-label="t('settings.gameDlls.remove')"
              @click="removeDll(dll)"
            >
              <X class="size-3.5" aria-hidden="true" />
            </Button>
          </div>
          <form class="flex gap-1" @submit.prevent="addDll">
            <Input
              v-model="newDll"
              :placeholder="t('settings.gameDlls.placeholder')"
              class="h-7 flex-1 text-xs"
            />
            <Button
              type="submit"
              variant="outline"
              class="size-7 shrink-0 p-0"
              :disabled="!newDll.trim()"
              :aria-label="t('settings.gameDlls.add')"
            >
              <Plus class="size-3.5" aria-hidden="true" />
            </Button>
          </form>
        </div>
      </SettingCard>

      <SettingCard
        :icon="Terminal"
        :title="t('settings.launchArgs.title')"
        :description="t('settings.launchArgs.description')"
      >
        <SettingText
          v-model="settings.launchArgs"
          :placeholder="t('settings.launchArgs.placeholder')"
        />
      </SettingCard>

      <SettingCard
        :icon="Timer"
        :title="t('settings.launchTimeout.title')"
        :description="t('settings.launchTimeout.description')"
      >
        <SettingNumber v-model="settings.launchTimeoutSeconds" :min="1" :max="300" />
      </SettingCard>
    </section>
  </div>
</template>
