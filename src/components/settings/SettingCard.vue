<script setup lang="ts">
import type { Component } from "vue";

defineProps<{
  icon: Component;
  title: string;
  description?: string;
}>();
</script>

<template>
  <div class="setting-card">
    <div class="flex min-w-0 items-center gap-3">
      <div class="setting-card-icon">
        <component :is="icon" class="size-4.5" aria-hidden="true" />
      </div>
      <div class="min-w-0">
        <p class="setting-card-title">{{ title }}</p>
        <p v-if="description" class="setting-card-description">
          {{ description }}
        </p>
      </div>
    </div>

    <div class="shrink-0 pl-12 sm:pl-0">
      <slot />
    </div>
  </div>
</template>
