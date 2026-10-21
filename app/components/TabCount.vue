<script setup lang="ts">
import NumberFlow from '@number-flow/vue'

defineProps<{ value: number }>()
</script>

<template>
  <NumberFlow :value="value" locales="en-US" class="shrink-0 text-xs font-normal tabular-nums opacity-60" />
</template>
