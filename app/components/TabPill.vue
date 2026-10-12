<script setup lang="ts">
const transition = { type: 'spring', bounce: 0.15, duration: 0.4 } as const
</script>

<template>
  <!-- Radius in style: Motion keeps it round while resizing -->
  <Motion
    layout-id="tab-pill"
    :transition="transition"
    :style="{ borderRadius: '6px' }"
    class="absolute inset-0 -z-10 bg-black/[5.78%] dark:bg-white/[8.37%]"
  />
</template>
