<script setup lang="ts">
const clips = Object.values(sampleClips)

const tabs = [
  { name: 'History', count: 281 },
  { name: 'Snippets', count: 24 },
  { name: 'Design', count: 12 }
]

const toolbarButton = 'flex size-7 items-center justify-center rounded-md'
</script>

<template>
  <div
    role="img"
    aria-label="The Micropaste window, showing copied links, code, colors, files, images and SVG as cards"
    class="flex h-113.5 w-full max-w-200 flex-col overflow-hidden rounded-lg border border-black/10 bg-[#f3f3f3] text-black/90 shadow-[0_32px_64px_rgb(0_0_0/0.14),0_2px_21px_rgb(0_0_0/0.14)] select-none"
  >
    <header class="flex h-10 shrink-0 items-stretch justify-between">
      <div class="flex min-w-0 items-center gap-1 pl-2">
        <div class="flex h-6 w-40 items-center gap-2 px-1 sm:w-60">
          <Icon name="gravity-ui:magnifier" class="shrink-0 text-base opacity-60" />
          <span class="truncate text-sm opacity-50">Search…</span>
        </div>
        <span :class="toolbarButton"><Icon name="gravity-ui:star" class="text-base opacity-60" /></span>
        <span :class="toolbarButton"><Icon name="gravity-ui:layout-cells-large" class="text-base opacity-60" /></span>
        <span :class="toolbarButton"><Icon name="gravity-ui:gear" class="text-base opacity-60" /></span>
      </div>

      <CaptionButtons />
    </header>

    <div class="flex shrink-0 items-center gap-1 px-2 pb-1">
      <TabPill v-for="(tab, i) in tabs" :key="tab.name" :name="tab.name" :count="tab.count" :active="i === 0" />
      <span class="flex size-8 items-center justify-center">
        <Icon name="gravity-ui:plus" class="text-base opacity-70" />
      </span>
    </div>

    <div class="flex min-h-0 flex-1">
      <div class="grid min-w-0 flex-1 auto-rows-min grid-cols-[repeat(auto-fill,minmax(200px,1fr))] gap-2 p-2 text-xs">
        <ClipTile v-for="clip in clips" :key="clip.kind" :clip="clip" />
      </div>
    </div>
  </div>
</template>
