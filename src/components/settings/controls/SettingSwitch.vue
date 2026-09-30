<script setup lang="ts">
import { Switch } from "@/components/ui/switch";

const model = defineModel<boolean>({ required: true });
</script>

<template>
  <Switch v-model="model" />
</template>
