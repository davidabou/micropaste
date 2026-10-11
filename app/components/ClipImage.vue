<script lang="ts">
import { convertFileSrc } from '@tauri-apps/api/core'
import { appDataDir, join } from '@tauri-apps/api/path'

// Resolved once for every instance
let imagesDir: Promise<string> | undefined
const getImagesDir = () => (imagesDir ??= appDataDir().then(dir => join(dir, 'images')))
</script>

<script setup lang="ts">
const props = defineProps<{
  /** File name in the images folder of the app data, e.g. `<hash>.png` */
  file: string
}>()

const src = ref<string>()

watchEffect(async () => {
  const file = props.file
  const path = await join(await getImagesDir(), file)
  if (props.file === file) src.value = convertFileSrc(path)
})
</script>

<template>
  <img v-if="src" :src="src" alt="" draggable="false">
</template>
