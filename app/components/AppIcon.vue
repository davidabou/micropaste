<script lang="ts">
import { invoke } from '@tauri-apps/api/core'

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
  path: string | null
}>()

const src = ref<string | null>(null)
const loading = ref(false)

watchEffect(async () => {
  const path = props.path
  src.value = null
  if (!path) return
  loading.value = true
  const icon = await loadIcon(path)
  if (props.path !== path) return
  src.value = icon
  loading.value = false
})
</script>

<template>
  <img v-if="src" :src="src" alt="" class="size-4 shrink-0" draggable="false">
  <span v-else-if="loading" class="skeleton size-4 shrink-0 rounded-sm" />
  <Icon v-else name="gravity-ui:square-dashed" class="size-4 shrink-0 opacity-60" />
</template>
