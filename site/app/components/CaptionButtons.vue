<script setup lang="ts">
const { small = false } = defineProps<{ small?: boolean }>()

const button = computed(() => ['flex items-center justify-center', small ? 'h-8 w-10' : 'h-10 w-[46px]'])
const icon = computed(() => small ? 'size-2' : 'size-2.5')
</script>

<template>
  <div class="flex shrink-0">
    <span :class="button">
      <svg viewBox="0 0 10 10" :class="icon" fill="none" stroke="currentColor"><path d="M0 5.5h10" /></svg>
    </span>
    <span :class="button">
      <svg viewBox="0 0 10 10" :class="icon" fill="none" stroke="currentColor"><rect x="0.5" y="0.5" width="9" height="9" rx="1" /></svg>
    </span>
    <span :class="button">
      <svg viewBox="0 0 10 10" :class="icon" fill="none" stroke="currentColor"><path d="M0.5 0.5l9 9M9.5 0.5l-9 9" /></svg>
    </span>
  </div>
</template>
