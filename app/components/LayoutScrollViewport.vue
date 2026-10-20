<script setup lang="ts">
import { injectScrollAreaRootContext } from 'reka-ui'

// ScrollAreaViewport, as a Motion element with layout-scroll: the layout animations inside account for its scroll
const context = injectScrollAreaRootContext()

const viewport = useTemplateRef<{ $el: HTMLElement }>('viewport')
const content = useTemplateRef<HTMLElement>('content')
onMounted(() => {
  context.onViewportChange(viewport.value?.$el ?? null)
  if (content.value) context.onContentChange(content.value)
})
</script>

<template>
  <Motion
    ref="viewport"
    layout-scroll
    data-reka-scroll-area-viewport
    :style="{
      overflowX: context.scrollbarXEnabled.value ? 'scroll' : 'hidden',
      overflowY: context.scrollbarYEnabled.value ? 'scroll' : 'hidden'
    }"
  >
    <div ref="content">
      <slot />
    </div>
  </Motion>
</template>
