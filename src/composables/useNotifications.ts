import { readonly, ref } from "vue";

export type NotificationType = "success" | "info" | "warning" | "error";

export interface AppNotification {
  id: number;
  type: NotificationType;
  message: string;
  title?: string;
  duration: number;
  closing: boolean;
}

interface NotificationOptions {
  title?: string;
  /** 设为 0 时保留通知，直到手动关闭。 */
  duration?: number;
}

const DEFAULT_DURATIONS: Record<NotificationType, number> = {
  success: 4000,
  info: 5000,
  warning: 7000,
  error: 10000,
};
const MAX_NOTIFICATIONS = 5;
const notifications = ref<AppNotification[]>([]);
let nextId = 0;

function notify(type: NotificationType, message: string, options: NotificationOptions = {}): number {
  const existing = notifications.value.find((item) =>
    !item.closing && item.type === type && item.message === message && item.title === options.title,
  );
  if (existing) return existing.id;

  const notification: AppNotification = {
    id: ++nextId,
    type,
    message,
    title: options.title,
    duration: options.duration ?? DEFAULT_DURATIONS[type],
    closing: false,
  };
  notifications.value = [...notifications.value, notification].slice(-MAX_NOTIFICATIONS);
  return notification.id;
}

function notifyError(error: unknown, title?: string): number {
  return notify("error", error instanceof Error ? error.message : String(error), { title });
}

function dismiss(id: number) {
  notifications.value = notifications.value.filter((item) => item.id !== id);
}

function beginDismiss(id: number) {
  const notification = notifications.value.find((item) => item.id === id);
  if (notification) notification.closing = true;
}

export function useNotifications() {
  return { notifications: readonly(notifications), notify, notifyError, dismiss, beginDismiss };
}
