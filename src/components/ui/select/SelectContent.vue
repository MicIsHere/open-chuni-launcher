<script setup lang="ts">
import type { SelectContentEmits, SelectContentProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import {
  SelectContent,
  SelectPortal,
  SelectViewport,
  useForwardPropsEmits,
} from 'reka-ui'
import { cn } from '@/lib/utils'

defineOptions({
  inheritAttrs: false,
})

// 为全部弹层定位属性给出与 reka-ui 运行时一致的默认值，避免 IDE 误判为必填
const props = withDefaults(
  defineProps<SelectContentProps & { class?: HTMLAttributes['class'] }>(),
  {
    position: 'popper',
    side: 'bottom',
    align: 'start',
    sideOffset: 4,
    alignOffset: 0,
    alignFlip: true,
    sideFlip: true,
    arrowPadding: 0,
    avoidCollisions: true,
    collisionBoundary: () => [],
    collisionPadding: 0,
    sticky: 'partial',
    hideWhenDetached: false,
    hideShiftedArrow: true,
    prioritizePosition: false,
    positionStrategy: 'fixed',
    updatePositionStrategy: 'optimized',
  },
)
const emits = defineEmits<SelectContentEmits>()

const delegatedProps = reactiveOmit(props, 'class')

const forwarded = useForwardPropsEmits(delegatedProps, emits)
</script>

<template>
  <SelectPortal>
    <SelectContent
      data-slot="select-content"
      v-bind="{ ...$attrs, ...forwarded }"
      :class="cn(
        'select-content',
        props.class,
      )
      "
    >
      <SelectViewport class="select-viewport">
        <slot />
      </SelectViewport>
    </SelectContent>
  </SelectPortal>
</template>
