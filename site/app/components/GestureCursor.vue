<script setup lang="ts">
import type { MotionValue } from 'motion-v'

const { x, y, pressed = false } = defineProps<{
  x: MotionValue<number>
  y: MotionValue<number>
  pressed?: boolean
}>()

// Vertical left side, 45° top side, slightly hollow bottom side
const ARROW = 'M5 5L14 14Q9.2 15.6 5 19Z'
</script>

<template>
  <Motion class="pointer-events-none absolute top-0 left-0 z-20" :style="{ x, y }">
    <Motion
      class="origin-top-left drop-shadow-[0_2px_3px_rgba(0,0,0,0.25)]"
      :animate="{ scale: pressed ? 0.85 : 1 }"
      :transition="{ type: 'spring', bounce: 0.4, visualDuration: 0.15 }"
    >
      <!-- Round joins on a sharp shape give the rounded corners: dark outline below, white body above -->
      <svg width="24" height="24" viewBox="0 0 24 24" class="-mt-1 -ml-1" stroke-linejoin="round">
        <path :d="ARROW" fill="#171717" stroke="#171717" stroke-width="6.5" />
        <path :d="ARROW" fill="white" stroke="white" stroke-width="3.5" />
      </svg>
    </Motion>
  </Motion>
</template>
