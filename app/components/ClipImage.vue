<script lang="ts">
import { convertFileSrc } from '@tauri-apps/api/core'
import { appDataDir, join } from '@tauri-apps/api/path'

let imagesDir: Promise<string> | undefined
const getImagesDir = () => (imagesDir ??= appDataDir().then(dir => join(dir, 'images')))
</script>

<script setup lang="ts">
const props = defineProps<{
  file: string
}>()

const src = ref<string>()
const loaded = ref(false)

watchEffect(async () => {
  const file = props.file
  const path = await join(await getImagesDir(), file)
  if (props.file !== file) return
  loaded.value = false
  src.value = convertFileSrc(path)
})
</script>

<template>
  <img :src="src" alt="" draggable="false" :class="!loaded && 'skeleton'" @load="loaded = true" @error="loaded = true">
</template>
