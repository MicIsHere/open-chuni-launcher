<script setup lang="ts">
import type { NumberFieldDecrementProps } from 'reka-ui'

import type { HTMLAttributes } from 'vue'
import { MinusIcon } from '@lucide/vue'
import { reactiveOmit } from '@vueuse/core'
import { NumberFieldDecrement, useForwardProps } from 'reka-ui'
import { cn } from '@/lib/utils'

const props = defineProps<NumberFieldDecrementProps & { class?: HTMLAttributes['class'] }>()

const delegatedProps = reactiveOmit(props, 'class')

const forwarded = useForwardProps(delegatedProps)
</script>

<template>
  <NumberFieldDecrement data-slot="decrement" v-bind="forwarded" :class="cn('number-field-step left-0', props.class)">
    <slot>
      <MinusIcon class="h-4 w-4" />
    </slot>
  </NumberFieldDecrement>
</template>
