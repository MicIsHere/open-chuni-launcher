<script setup lang="ts">
import {
  NumberField,
  NumberFieldContent,
  NumberFieldDecrement,
  NumberFieldIncrement,
  NumberFieldInput,
} from "@/components/ui/number-field";

const model = defineModel<number>({ required: true });

defineProps<{ min?: number; max?: number; step?: number }>();
</script>

<template>
  <NumberField
    v-model="model"
    :min="min"
    :max="max"
    :step="step ?? 1"
    class="w-32"
  >
    <NumberFieldContent>
      <NumberFieldDecrement />
      <NumberFieldInput />
      <NumberFieldIncrement />
    </NumberFieldContent>
  </NumberField>
</template>
