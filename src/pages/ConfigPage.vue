<script setup lang="ts">
import { computed } from "vue";
import { Server } from "@lucide/vue";
import { Input } from "@/components/ui/input";
import SettingSelect from "@/components/settings/controls/SettingSelect.vue";
import { useSettings } from "@/composables/useSettings";
import { useI18n } from "@/i18n";
import { isValidKeychip, SERVER_PRESETS } from "@/lib/servers";

const { t } = useI18n();
const { settings } = useSettings();

const presetOptions = [
  ...SERVER_PRESETS.map((preset) => ({ value: preset.id, label: preset.label })),
  { value: "custom", label: t("config.server.custom") },
];

const keychipInvalid = computed(
  () =>
    settings.value.server.keychip.trim() !== "" &&
    !isValidKeychip(settings.value.server.keychip),
);
</script>

<template>
  <div class="settings-page">
    <section class="settings-section">
      <div class="config-server-card">
        <div class="config-server-header">
          <div class="config-server-icon">
            <Server class="size-5" aria-hidden="true" />
          </div>
          <div>
            <h3 class="text-base font-semibold">{{ t("config.server.title") }}</h3>
            <p class="text-xs text-muted-foreground">{{ t("config.server.description") }}</p>
          </div>
        </div>

        <div class="config-field">
          <p class="config-field-label">{{ t("config.server.preset") }}</p>
          <SettingSelect v-model="settings.server.preset" :options="presetOptions" />
        </div>

        <template v-if="settings.server.preset === 'custom'">
          <div class="config-field">
            <label class="config-field-label" for="config-dns">{{ t("config.server.dns") }}</label>
            <Input
              id="config-dns"
              v-model="settings.server.customDns"
              placeholder="dns server host"
            />
          </div>
          <div class="config-field">
            <label class="config-field-label" for="config-aimedb">{{ t("config.server.aimedb") }}</label>
            <Input
              id="config-aimedb"
              v-model="settings.server.customAimeDb"
              placeholder="aimeDB server host"
            />
            <p class="config-field-hint">{{ t("config.server.aimedbHint") }}</p>
          </div>
        </template>

        <div class="config-divider" role="presentation" />

        <div class="config-field">
          <label class="config-field-label" for="config-keychip">{{ t("config.keychip.title") }}</label>
          <Input
            id="config-keychip"
            v-model="settings.server.keychip"
            class="font-mono"
            :placeholder="t('config.keychip.placeholder')"
            :aria-invalid="keychipInvalid"
            :class="keychipInvalid ? 'border-destructive' : ''"
          />
          <p v-if="keychipInvalid" class="text-xs text-destructive">
            {{ t("config.keychip.invalid") }}
          </p>
        </div>
      </div>
    </section>
  </div>
</template>
