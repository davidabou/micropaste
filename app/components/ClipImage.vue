<script lang="ts">
import { convertFileSrc } from '@tauri-apps/api/core'
import { appDataDir, join } from '@tauri-apps/api/path'

let imagesDir: Promise<string> | undefined
const getImagesDir = () => (imagesDir ??= appDataDir().then(dir => join(dir, 'images')))
// A remounted tile shows its image at once, without the skeleton
const srcs = new Map<string, string>()
const loadedSrcs = new Set<string>()
</script>

<script setup lang="ts">
const props = defineProps<{
  file: string
}>()

const src = ref(srcs.get(props.file))
const loadedSrc = ref<string>()
const loaded = computed(() => !!src.value && (loadedSrc.value === src.value || loadedSrcs.has(src.value)))

watch(() => props.file, async (file) => {
  if (!srcs.has(file)) srcs.set(file, convertFileSrc(await join(await getImagesDir(), file)))
  if (props.file === file) src.value = srcs.get(file)
}, { immediate: true })

function onLoad() {
  if (src.value) loadedSrcs.add(src.value)
  loadedSrc.value = src.value
}
</script>

<template>
  <img :src="src" alt="" draggable="false" :class="!loaded && 'skeleton'" @load="onLoad" @error="loadedSrc = src">
</template>
