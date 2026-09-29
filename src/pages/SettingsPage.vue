<script setup lang="ts">
import { computed } from "vue";
import { FolderOpen, Languages, Palette, Terminal, Timer } from "@lucide/vue";
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
