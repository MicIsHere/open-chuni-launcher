<script setup lang="ts">
import type { SelectItemProps } from 'reka-ui'

import type { HTMLAttributes } from 'vue'
import { CheckIcon } from '@lucide/vue'
import { reactiveOmit } from '@vueuse/core'
import {
  SelectItem,
  SelectItemIndicator,
  SelectItemText,
  useForwardProps,
} from 'reka-ui'
import { cn } from '@/lib/utils'

const props = defineProps<SelectItemProps & { class?: HTMLAttributes['class'] }>()

const delegatedProps = reactiveOmit(props, 'class')

const forwardedProps = useForwardProps(delegatedProps)
</script>

<template>
  <SelectItem
    data-slot="select-item"
    v-bind="forwardedProps"
    :class="
      cn(
        'select-item',
        props.class,
      )
    "
  >
    <SelectItemText>
      <span class="select-item-text">
        <slot />
      </span>
    </SelectItemText>

    <span class="select-item-indicator">
      <SelectItemIndicator>
        <CheckIcon class="pointer-events-none" />
      </SelectItemIndicator>
    </span>
  </SelectItem>
</template>
