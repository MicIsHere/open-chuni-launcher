<script setup lang="ts">
import { Input } from "@/components/ui/input";

const model = defineModel<string>({ required: true });

defineProps<{ placeholder?: string }>();
</script>

<template>
  <Input v-model="model" :placeholder="placeholder" class="w-full sm:w-56" />
</template>
