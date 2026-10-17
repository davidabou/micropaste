<script setup lang="ts">
defineProps<{
  name: string
  count: number
  active?: boolean
}>()
</script>

<template>
  <span class="flex h-8 items-baseline gap-1.5 rounded-md px-3 pt-1.75 text-sm font-medium" :class="active ? 'bg-black/[5.78%]' : 'opacity-60'">
    {{ name }}
    <span class="text-xs font-normal tabular-nums opacity-60">{{ count }}</span>
  </span>
</template>
