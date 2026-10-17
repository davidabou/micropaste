<script setup lang="ts">
import type { Clip } from '@/utils/clip'

defineProps<{ clip: Clip }>()
</script>

<template>
  <div class="relative flex h-44 flex-col overflow-hidden rounded-lg border border-black/[5.78%] bg-black/[3.73%] text-xs transition-[background-color,box-shadow] duration-100 hover:bg-black/[5.78%] hover:ring-2 hover:ring-black/10">
    <img v-if="clip.kind === 'link'" :src="clip.image" alt="" draggable="false" class="absolute inset-0 size-full object-cover object-left">

    <div v-else-if="clip.kind === 'color'" class="absolute inset-0 bg-[conic-gradient(#ccc_25%,#fff_0_50%,#ccc_0_75%,#fff_0)] bg-size-[16px_16px]">
      <div class="size-full" :style="{ background: clip.value }" />
    </div>

    <img v-else-if="clip.kind === 'image'" :src="clip.src" alt="" draggable="false" class="absolute inset-0 size-full object-cover">

    <div v-else-if="clip.kind === 'files'" class="absolute inset-0 flex items-center justify-center pb-10">
      <FileStack :icons="clip.icons" />
    </div>

    <div v-else-if="clip.kind === 'svg'" class="absolute inset-0 flex items-center justify-center">
      <svg viewBox="0 0 24 24" class="size-24" fill="currentColor"><path :d="clip.path" /></svg>
    </div>

    <div v-else class="min-h-0 flex-1">
      <p class="line-clamp-5 px-3 pt-9 text-sm whitespace-pre-wrap wrap-anywhere">{{ clip.content }}</p>
    </div>

    <footer
      class="absolute inset-x-0 bottom-0 z-10 px-3 py-2"
      :class="clip.kind === 'color' ? 'text-white' : (clip.kind === 'link' || clip.kind === 'image') && 'bg-linear-to-t from-white/90 via-white/60 to-transparent pt-8'"
    >
      <p v-if="clip.kind === 'link'" class="mb-1 truncate text-sm font-medium">
        {{ clip.title }}
      </p>
      <p v-else-if="clip.kind === 'color'" class="mb-1 truncate text-sm font-medium">
        {{ clip.value }}
      </p>
      <p v-else-if="clip.kind === 'files'" class="mb-1 truncate text-sm font-medium">
        {{ clip.icons.length }} items
      </p>

      <div class="flex items-center gap-1.5">
        <Icon :name="clip.app" class="shrink-0 text-base" />
        <span class="truncate" :class="clip.kind === 'text' || clip.kind === 'files' || clip.kind === 'svg' ? 'opacity-60' : 'opacity-80'">
          {{ clip.time }}
        </span>
        <span v-if="clip.kind === 'link'" class="ml-auto truncate opacity-70">{{ clip.domain }}</span>
        <span v-else-if="clip.kind === 'color'" class="ml-auto shrink-0 opacity-70">{{ clip.format }}</span>
        <span v-else-if="clip.kind === 'image'" class="ml-auto shrink-0 opacity-70">{{ clip.size }}</span>
        <span v-else-if="clip.kind === 'files'" class="ml-auto shrink-0 opacity-60">{{ clip.icons.length }} items</span>
        <span v-else-if="clip.kind === 'svg'" class="ml-auto shrink-0 opacity-60">SVG</span>
      </div>
    </footer>
  </div>
</template>
