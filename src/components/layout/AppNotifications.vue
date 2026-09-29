<script setup lang="ts">
import { nextTick } from "vue";
import { CircleCheck, CircleX, Info, TriangleAlert, X } from "@lucide/vue";
import { ToastClose, ToastDescription, ToastProvider, ToastRoot, ToastTitle, ToastViewport } from "reka-ui";
import { Button } from "@/components/ui/button";
import { useNotifications } from "@/composables/useNotifications";
import { useI18n } from "@/i18n";

const { t } = useI18n();
const { notifications, dismiss, beginDismiss } = useNotifications();
const icons = { success: CircleCheck, info: Info, warning: TriangleAlert, error: CircleX };

async function dismissAfterAnimation(id: number) {
  beginDismiss(id);
  await nextTick();
  const animations = document.getElementById("notification-" + id)?.getAnimations() ?? [];
  await Promise.allSettled(animations.map((animation) => animation.finished));
  dismiss(id);
}
</script>

<template>
  <ToastProvider :label="t('notifications.region')" disable-swipe>
    <ToastRoot
      v-for="notification in notifications"
      :key="notification.id"
      :id="'notification-' + notification.id"
      :duration="notification.duration"
      :type="notification.type === 'error' || notification.type === 'warning' ? 'foreground' : 'background'"
      :data-type="notification.type"
      data-slot="notification"
      class="notification-card"
      @update:open="(open) => { if (!open) dismissAfterAnimation(notification.id); }"
    >
      <div class="notification-icon">
        <component :is="icons[notification.type]" class="size-4.5" aria-hidden="true" />
      </div>
      <div class="notification-content">
        <ToastTitle class="notification-title">
          {{ notification.title ?? t('notifications.' + notification.type) }}
        </ToastTitle>
        <ToastDescription class="notification-message">{{ notification.message }}</ToastDescription>
      </div>
      <ToastClose as-child>
        <Button
          variant="ghost"
          size="icon-xs"
          class="text-muted-foreground"
          :aria-label="t('notifications.dismiss')"
          :title="t('notifications.dismiss')"
        >
          <X class="size-3.5" aria-hidden="true" />
        </Button>
      </ToastClose>
    </ToastRoot>
    <Teleport to="body">
      <ToastViewport class="notification-viewport" :label="t('notifications.region')" />
    </Teleport>
  </ToastProvider>
</template>
