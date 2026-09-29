<script setup lang="ts">
import { nextTick, ref, watch } from "vue";
import { Play, Square } from "@lucide/vue";
import { Button } from "@/components/ui/button";
import { useLaunch } from "@/composables/useLaunch";
import { useI18n } from "@/i18n";

const { t } = useI18n();
const { logs, running, launch, stop } = useLaunch();

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
  <div class="flex h-full flex-col gap-4">
    <div class="flex items-center gap-2">
      <Button :disabled="running" @click="launch">
        <Play class="size-4" aria-hidden="true" />
        {{ t("home.launch") }}
      </Button>
      <Button variant="outline" :disabled="!running" @click="stop">
        <Square class="size-4" aria-hidden="true" />
        {{ t("home.stop") }}
      </Button>
    </div>

    <div
      ref="logPanel"
      class="min-h-0 flex-1 overflow-auto rounded-lg border bg-card p-3 font-mono text-xs leading-relaxed text-muted-foreground"
    >
      <p v-for="(line, index) in logs" :key="index">{{ line }}</p>
    </div>
  </div>
</template>
