<script setup lang="ts">
import { FolderOpen } from "@lucide/vue";
import { open } from "@tauri-apps/plugin-dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { useNotifications } from "@/composables/useNotifications";
import { useI18n } from "@/i18n";

const model = defineModel<string>({ required: true });

defineProps<{ placeholder?: string; browseLabel: string }>();

const isTauri = "__TAURI_INTERNALS__" in window;
const { notifyError } = useNotifications();
const { t } = useI18n();

async function browse() {
  if (!isTauri) return;
  try {
    const selected = await open({ directory: true, multiple: false });
    if (typeof selected === "string") {
      model.value = selected;
    }
  } catch (error) {
    notifyError(error, t("notifications.directoryError"));
  }
}
</script>

<template>
  <div class="flex w-full items-center gap-2 sm:w-auto">
    <Input
      :model-value="model"
      :placeholder="placeholder"
      readonly
      class="w-full sm:w-64"
    />
    <Button
      variant="outline"
      :disabled="!isTauri"
      class="shrink-0"
      @click="browse"
    >
      <FolderOpen class="size-4" aria-hidden="true" />
      {{ browseLabel }}
    </Button>
  </div>
</template>
