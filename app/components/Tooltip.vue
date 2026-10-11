<script setup lang="ts">
const props = withDefaults(defineProps<{
  /** Text shown on hover, multi-line with `\n`. Without it, only the slot is rendered */
  content?: string | null
  side?: 'top' | 'right' | 'bottom' | 'left'
  /** Keeps the trigger as is but never shows the tooltip */
  disabled?: boolean
  /**
   * For paths: wider, and wrapped only at the edge, anywhere in the text, instead of at its spaces
   * ("C:\Users\Master Race" doesn't break after "Master")
   */
  path?: boolean
}>(), {
  side: 'top'
})

const open = ref(false)

// The bubble (portal, content, animation) is built the first time it opens only: a tile holds several
// tooltips, most never shown. The trigger stays the same element all along, never rebuilt
const opened = ref(false)
watch(open, (isOpen) => {
  if (isOpen) opened.value = true
})

// Pops in on a bouncy spring, sliding away from its trigger, and pops out without bounce
const offset = computed(() => ({
  x: props.side === 'left' ? 4 : props.side === 'right' ? -4 : 0,
  y: props.side === 'top' ? 4 : props.side === 'bottom' ? -4 : 0
}))
const pop = computed(() => ({
  initial: { opacity: 0, scale: 0.85, ...offset.value },
  animate: { opacity: 1, scale: 1, x: 0, y: 0, transition: { type: 'spring' as const, bounce: 0.4, visualDuration: 0.18 } },
  exit: { opacity: 0, scale: 0.9, transition: { type: 'spring' as const, bounce: 0, visualDuration: 0.1 } }
}))
</script>

<template>
  <TooltipRoot v-if="content" v-model:open="open" :disabled="disabled">
    <!-- The slot element itself is the trigger, no extra wrapper -->
    <TooltipTrigger as-child>
      <slot />
    </TooltipTrigger>
    <!-- force-mount: Motion decides when it leaves, once its exit animation is over -->
    <TooltipPortal v-if="opened" force-mount>
      <AnimatePresence>
        <TooltipContent
          v-if="open"
          force-mount
          as-child
          :side="side"
          :side-offset="6"
          :collision-padding="8"
        >
          <!-- Colors of the Windows 11 tooltips -->
          <Motion
            :initial="pop.initial"
            :animate="pop.animate"
            :exit="pop.exit"
            :class="path ? 'max-w-[min(36rem,calc(100vw-1rem))] [line-break:anywhere]' : 'max-w-80'"
            class="z-50 origin-(--reka-tooltip-content-transform-origin) rounded-md border border-black/[5.78%] bg-[#f9f9f9] px-2 py-1 font-ui text-xs whitespace-pre-line wrap-anywhere shadow-md select-none dark:border-black/20 dark:bg-[#2c2c2c]"
          >
            {{ content }}
          </Motion>
        </TooltipContent>
      </AnimatePresence>
    </TooltipPortal>
  </TooltipRoot>
  <slot v-else />
</template>
