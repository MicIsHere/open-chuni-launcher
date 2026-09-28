<script setup lang="ts">
import type { NumberFieldIncrementProps } from 'reka-ui'

import type { HTMLAttributes } from 'vue'
import { PlusIcon } from '@lucide/vue'
import { reactiveOmit } from '@vueuse/core'
import { NumberFieldIncrement, useForwardProps } from 'reka-ui'
import { cn } from '@/lib/utils'

const props = defineProps<NumberFieldIncrementProps & { class?: HTMLAttributes['class'] }>()

const delegatedProps = reactiveOmit(props, 'class')

const forwarded = useForwardProps(delegatedProps)
</script>

<template>
  <NumberFieldIncrement data-slot="increment" v-bind="forwarded" :class="cn('number-field-step right-0', props.class)">
    <slot>
      <PlusIcon class="h-4 w-4" />
    </slot>
  </NumberFieldIncrement>
</template>
