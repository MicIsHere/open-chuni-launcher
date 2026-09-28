<script setup lang="ts">
import { computed } from "vue";
import type { Component } from "vue";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

export interface SelectOption {
  value: string;
  label: string;
  icon?: Component;
}

const model = defineModel<string>({ required: true });

const props = defineProps<{ options: SelectOption[] }>();

/**
 * Render the trigger text from `options` instead of relying on reka-ui's
 * cached item text, so label changes (e.g. switching language) apply to the
 * closed trigger immediately.
 */
const selectedLabel = computed(
  () => props.options.find((option) => option.value === model.value)?.label ?? "",
);
</script>

<template>
  <Select v-model="model">
    <SelectTrigger class="w-44">
      <SelectValue>{{ selectedLabel }}</SelectValue>
    </SelectTrigger>
    <SelectContent>
      <SelectItem
        v-for="option in options"
        :key="option.value"
        :value="option.value"
      >
        <component :is="option.icon" v-if="option.icon" class="size-4" aria-hidden="true" />
        {{ option.label }}
      </SelectItem>
    </SelectContent>
  </Select>
</template>
