import { createApp } from "vue";
import "./style.css";
import App from "./App.vue";
import { useTheme } from "@/composables/useTheme";
import { useNotifications } from "@/composables/useNotifications";

const { notifyError } = useNotifications();
const app = createApp(App);

app.config.errorHandler = (error) => {
  console.error(error);
  notifyError(error);
};
window.addEventListener("error", (event) => notifyError(event.error ?? event.message));
window.addEventListener("unhandledrejection", (event) => notifyError(event.reason));

useTheme();

app.mount("#app");
