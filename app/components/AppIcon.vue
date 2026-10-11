<script lang="ts">
import { invoke } from '@tauri-apps/api/core'

// Shared by every instance: each app icon is requested only once
const cache = new Map<string, Promise<string | null>>()

function loadIcon(path: string) {
  if (!cache.has(path)) {
    cache.set(path, invoke<string | null>('get_app_icon', { path }).catch(() => null))
  }
  return cache.get(path)!
}
</script>

<script setup lang="ts">
const props = defineProps<{
  /** Executable path of the app */
  path: string | null
}>()

const src = ref<string | null>(null)

watchEffect(async () => {
  const path = props.path
  src.value = null
  if (!path) return
  const icon = await loadIcon(path)
  // Ignore the result if the path changed while loading
  if (props.path === path) src.value = icon
})
</script>

<template>
  <img v-if="src" :src="src" alt="" class="size-4 shrink-0" draggable="false">
  <Icon v-else name="gravity-ui:square-dashed" class="size-4 shrink-0 opacity-60" />
</template>
