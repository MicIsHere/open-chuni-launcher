<script setup lang="ts">
import type { TooltipContentEmits, TooltipContentProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { TooltipArrow, TooltipContent, TooltipPortal, useForwardPropsEmits } from 'reka-ui'
import { cn } from '@/lib/utils'

defineOptions({
  inheritAttrs: false,
})

// 为全部弹层定位属性给出与 reka-ui 运行时一致的默认值，避免 IDE 误判为必填
const props = withDefaults(defineProps<TooltipContentProps & { class?: HTMLAttributes['class'] }>(), {
  side: 'top',
  sideOffset: 0,
  align: 'center',
  alignOffset: 0,
  arrowPadding: 0,
  avoidCollisions: true,
  collisionBoundary: () => [],
  collisionPadding: 0,
  sticky: 'partial',
  hideWhenDetached: false,
  positionStrategy: 'fixed',
  updatePositionStrategy: 'optimized',
})

const emits = defineEmits<TooltipContentEmits>()

const delegatedProps = reactiveOmit(props, 'class')
const forwarded = useForwardPropsEmits(delegatedProps, emits)
</script>

<template>
  <TooltipPortal>
    <TooltipContent
      data-slot="tooltip-content"
      v-bind="{ ...forwarded, ...$attrs }"
      :class="cn('tooltip-content', props.class)"
    >
      <slot />

      <TooltipArrow class="size-2.5 rotate-45 rounded-xs bg-foreground fill-foreground z-50 translate-y-[calc(-50%-2px)]" />
    </TooltipContent>
  </TooltipPortal>
</template>
