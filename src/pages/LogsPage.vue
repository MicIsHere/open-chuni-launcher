<script setup lang="ts">
import { nextTick, ref, watch } from "vue";
import { useLaunch } from "@/composables/useLaunch";
import { useI18n } from "@/i18n";

const { t } = useI18n();
const { logs } = useLaunch();

const logPanel = ref<HTMLElement | null>(null);

// 新日志到达时自动滚动到底部
watch(
  () => logs.value.length,
  async () => {
    await nextTick();
    if (logPanel.value) {
      logPanel.value.scrollTop = logPanel.value.scrollHeight;
    }
  },
);
</script>

<template>
  <div class="flex h-full flex-col">
    <div
      ref="logPanel"
      class="min-h-0 flex-1 overflow-auto rounded-lg border bg-card p-3 font-mono text-xs leading-relaxed text-muted-foreground"
    >
      <p v-if="logs.length === 0" class="py-6 text-center">
        {{ t("logs.empty") }}
      </p>
      <p v-for="(line, index) in logs" :key="index">{{ line }}</p>
    </div>
  </div>
</template>
