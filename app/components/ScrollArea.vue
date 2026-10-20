<script setup lang="ts">
const props = withDefaults(defineProps<{
  orientation?: 'vertical' | 'horizontal'
  viewportClass?: string
  layoutScroll?: boolean
}>(), {
  orientation: 'vertical'
})

const viewport = 'size-full rounded-[inherit] focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-current'
// Thin until hovered, like Windows 11 scrollbars
const scrollbar = computed(() =>
  props.orientation === 'vertical'
    ? 'w-2.5 py-0.5 pr-0.5 pl-1.5 hover:pl-0.5'
    : 'h-2.5 flex-col px-0.5 pt-1.5 pb-0.5 hover:pt-0.5'
)
</script>

<template>
  <ScrollAreaRoot type="hover" :scroll-hide-delay="600" class="overflow-hidden">
    <LayoutScrollViewport v-if="layoutScroll" :class="[viewport, viewportClass]">
      <slot />
    </LayoutScrollViewport>
    <ScrollAreaViewport v-else :class="[viewport, viewportClass]">
      <slot />
    </ScrollAreaViewport>
    <ScrollAreaScrollbar
      :orientation="orientation"
      class="z-10 flex touch-none transition-[padding] duration-100 select-none data-[state=hidden]:animate-fade-out data-[state=visible]:animate-fade-in"
      :class="scrollbar"
    >
      <ScrollAreaThumb class="relative flex-1 rounded-full bg-black/45 dark:bg-white/45" />
    </ScrollAreaScrollbar>
  </ScrollAreaRoot>
</template>
