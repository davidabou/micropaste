<script setup lang="ts">
defineProps<{ icons: string[] }>()

const STACK = [{ rotate: 0, x: 0 }, { rotate: -9, x: -22 }, { rotate: 9, x: 22 }]
</script>

<template>
  <div class="relative isolate size-20">
    <div
      v-for="(icon, i) in icons"
      :key="icon"
      class="absolute inset-0 flex items-center justify-center rounded-md bg-white shadow-md ring-1 ring-black/10"
      :style="{ rotate: `${STACK[i]!.rotate}deg`, translate: `${STACK[i]!.x}px 0`, zIndex: icons.length - i }"
    >
      <Icon :name="icon" class="text-[44px]" />
    </div>
  </div>
</template>
