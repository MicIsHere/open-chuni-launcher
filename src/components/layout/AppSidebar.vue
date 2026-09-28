<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from "vue";
import { ChevronsLeft, ChevronsRight, Disc3 } from "@lucide/vue";
import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { useI18n } from "@/i18n";
import { navSections } from "@/lib/navigation";
import { cn } from "@/lib/utils";

const { t } = useI18n();

const active = defineModel<string>({ required: true });

/**
 * 侧边栏按钮的共享样式。必须留在模板中（不能放进 styles/）：它要覆盖
 * buttonVariants 自带的工具类，只有 cn()/tw-merge 能做到——级联层级中
 * components 层恒输给 utilities 层。
 */
const sidebarItemClass =
  "w-full justify-start gap-2.5 px-2 text-sidebar-foreground/80 hover:bg-sidebar-accent hover:text-sidebar-foreground";

const COLLAPSE_KEY = "sidebar-collapsed";
const NARROW_QUERY = "(max-width: 639.98px)";

const manualCollapsed = ref(localStorage.getItem(COLLAPSE_KEY) === "1");
const mediaQuery = window.matchMedia(NARROW_QUERY);
const narrow = ref(mediaQuery.matches);

/** Icon-only rail when manually collapsed or when the window is too narrow. */
const collapsed = computed(() => manualCollapsed.value || narrow.value);

function toggleCollapsed() {
  manualCollapsed.value = !manualCollapsed.value;
}

watch(manualCollapsed, (value) => {
  localStorage.setItem(COLLAPSE_KEY, value ? "1" : "0");
});

function onNarrowChange(event: MediaQueryListEvent) {
  narrow.value = event.matches;
}

mediaQuery.addEventListener("change", onNarrowChange);
onBeforeUnmount(() => mediaQuery.removeEventListener("change", onNarrowChange));
</script>

<template>
  <!-- Collapse styling (width, label visibility) lives in .app-sidebar CSS,
       driven by the data-collapsed attribute. -->
  <aside
    :aria-label="t('sidebar.navLabel')"
    class="app-sidebar"
    :data-collapsed="collapsed || undefined"
  >
    <div class="flex h-12 shrink-0 items-center gap-2 px-3">
      <div class="sidebar-brand-icon">
        <Disc3 class="size-4.5" aria-hidden="true" />
      </div>
      <span class="sidebar-label text-sm font-semibold tracking-tight">
        Open Chunithm Launcher
      </span>
    </div>

    <nav class="sidebar-nav">
      <Tooltip v-for="item in navSections" :key="item.id" :disabled="!collapsed">
        <TooltipTrigger as-child>
          <Button
            variant="ghost"
            :class="cn(sidebarItemClass, active === item.id && 'bg-sidebar-accent text-sidebar-accent-foreground')"
            :aria-label="t(item.labelKey)"
            :aria-current="active === item.id ? 'page' : undefined"
            @click="active = item.id"
          >
            <component :is="item.icon" class="size-4.5 shrink-0" aria-hidden="true" />
            <span class="sidebar-label">{{ t(item.labelKey) }}</span>
          </Button>
        </TooltipTrigger>
        <TooltipContent side="right">{{ t(item.labelKey) }}</TooltipContent>
      </Tooltip>
    </nav>

    <div class="sidebar-footer">
      <Tooltip :disabled="!collapsed">
        <TooltipTrigger as-child>
          <Button
            variant="ghost"
            :class="sidebarItemClass"
            :aria-label="collapsed ? t('sidebar.expand') : t('sidebar.collapse')"
            @click="toggleCollapsed"
          >
            <ChevronsRight v-if="collapsed" class="size-4.5 shrink-0" aria-hidden="true" />
            <ChevronsLeft v-else class="size-4.5 shrink-0" aria-hidden="true" />
            <span class="sidebar-label">{{ t("sidebar.collapse") }}</span>
          </Button>
        </TooltipTrigger>
        <TooltipContent side="right">{{ t("sidebar.expand") }}</TooltipContent>
      </Tooltip>
    </div>
  </aside>
</template>
