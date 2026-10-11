<script setup lang="ts">
// Spring of the background sliding from one tab to the other
const transition = { type: 'spring', bounce: 0.15, duration: 0.4 } as const
</script>

<template>
  <!-- Background of the open tab, behind its name. Rendered in the open tab only: the shared layoutId
       makes Motion slide it from the previous tab, resizing it to the new one. The radius is set
       in style so Motion keeps it round while resizing -->
  <Motion
    layout-id="tab-pill"
    :transition="transition"
    :style="{ borderRadius: '6px' }"
    class="absolute inset-0 -z-10 bg-black/[5.78%] dark:bg-white/[8.37%]"
  />
</template>
