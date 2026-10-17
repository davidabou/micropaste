<script setup lang="ts">
function cellReveal(i: number) {
  return reveal(0.08 * i, 0.3)
}

const colors = [
  { value: '#5B5BD6', format: 'HEX' },
  { value: 'rgb(255 122 69)', format: 'RGB' },
  { value: 'oklch(0.78 0.15 162)', format: 'OKLCH' }
]

const apps = [
  'logos:chrome',
  'logos:visual-studio-code',
  'logos:figma',
  'logos:microsoft-edge',
  'flat-color-icons:folder',
  'vscode-icons:file-type-powershell'
]

const tabs = [
  { name: 'History', count: 281 },
  { name: 'Snippets', count: 24 },
  { name: 'Design', count: 12 },
  { name: 'Research', count: 7 }
]

const cell = 'flex flex-col overflow-hidden rounded-2xl border border-black/[6%] bg-white/50'
const stage = 'relative flex h-44 items-center justify-center'
const tile = 'rounded-lg border border-black/[5.78%] bg-[#f9f9f9] shadow-card'
</script>

<template>
  <div class="grid w-full grid-cols-1 gap-4 text-black/90 md:grid-cols-6">
    <Motion class="md:col-span-4" :class="[cell]" v-bind="cellReveal(0)">
      <div aria-hidden="true" :class="stage">
        <div class="relative h-36 w-72 overflow-hidden" :class="[tile]">
          <img src="/nuxt.jpg" alt="" draggable="false" class="size-full object-cover">
          <footer class="absolute inset-x-0 bottom-0 flex items-center gap-1.5 bg-linear-to-t from-white/90 via-white/60 to-transparent px-3 pt-6 pb-2 text-xs text-black/90">
            <Icon name="logos:chrome" class="shrink-0 text-base" />
            <span class="truncate font-medium">Nuxt: The Full-Stack Vue Framework</span>
            <span class="ml-auto shrink-0 opacity-70">nuxt.com</span>
          </footer>
        </div>
      </div>
      <div class="p-6 pt-0">
        <h3 class="text-lg font-medium tracking-tight">Links</h3>
        <p class="text-neutral-500">The page's title, image and domain, so you know what a link is before you open it.</p>
      </div>
    </Motion>

    <Motion class="md:col-span-2" :class="[cell]" v-bind="cellReveal(1)">
      <div aria-hidden="true" :class="stage">
        <div class="flex -space-x-6">
          <div
            v-for="(color, i) in colors"
            :key="color.format"
            class="flex h-28 w-20 items-end rounded-lg p-2 text-[10px] font-medium text-white shadow-card ring-1 ring-black/5"
            :style="{ background: color.value, rotate: `${(i - 1) * 8}deg`, translate: `0 ${i === 1 ? -6 : 0}px` }"
          >
            {{ color.format }}
          </div>
        </div>
      </div>
      <div class="p-6 pt-0">
        <h3 class="text-lg font-medium tracking-tight">Colors</h3>
        <p class="text-neutral-500">HEX, RGB, HSL or OKLCH become swatches.</p>
      </div>
    </Motion>

    <Motion class="md:col-span-2" :class="[cell]" v-bind="cellReveal(0)">
      <div aria-hidden="true" :class="stage">
        <div class="relative h-28 w-40 overflow-hidden" :class="[tile]">
          <img src="/background.jpg" alt="" draggable="false" class="size-full object-cover">
          <span class="absolute right-2 bottom-1.5 text-[10px] font-medium text-white/90">2.4 MB</span>
        </div>
      </div>
      <div class="p-6 pt-0">
        <h3 class="text-lg font-medium tracking-tight">Images & screenshots</h3>
        <p class="text-neutral-500">Full previews, ready to paste or drag out.</p>
      </div>
    </Motion>

    <Motion class="md:col-span-2" :class="[cell]" v-bind="cellReveal(1)">
      <div aria-hidden="true" :class="stage">
        <div class="relative h-32 w-40 text-xs" :class="[tile]">
          <div class="absolute inset-0 flex items-center justify-center pb-6">
            <svg viewBox="0 0 24 24" class="size-14" fill="currentColor">
              <path :d="BOLT" />
            </svg>
          </div>
          <footer class="absolute inset-x-0 bottom-0 flex items-center gap-1.5 px-3 py-2">
            <Icon name="logos:figma" class="shrink-0 text-base" />
            <span class="opacity-60">5 min ago</span>
            <span class="ml-auto shrink-0 opacity-60">SVG</span>
          </footer>
        </div>
      </div>
      <div class="p-6 pt-0">
        <h3 class="text-lg font-medium tracking-tight">SVG</h3>
        <p class="text-neutral-500">Copied SVG markup shows the drawing, not the code.</p>
      </div>
    </Motion>

    <Motion class="md:col-span-2" :class="[cell]" v-bind="cellReveal(2)">
      <div aria-hidden="true" :class="stage">
        <FileStack :icons="FILE_ICONS" />
      </div>
      <div class="p-6 pt-0">
        <h3 class="text-lg font-medium tracking-tight">Files</h3>
        <p class="text-neutral-500">Thumbnails as the Explorer shows them, one file or a whole group.</p>
      </div>
    </Motion>

    <Motion class="md:col-span-3" :class="[cell]" v-bind="cellReveal(0)">
      <div aria-hidden="true" :class="stage">
        <div class="grid grid-cols-3 gap-3">
          <div v-for="icon in apps" :key="icon" class="flex size-14 items-center justify-center bg-white" :class="[tile]">
            <Icon :name="icon" class="text-2xl" />
          </div>
        </div>
      </div>
      <div class="p-6 pt-0">
        <h3 class="text-lg font-medium tracking-tight">Source app</h3>
        <p class="text-neutral-500">Every clip remembers the app it came from, and when.</p>
      </div>
    </Motion>

    <Motion class="md:col-span-3" :class="[cell]" v-bind="cellReveal(1)">
      <div aria-hidden="true" :class="stage">
        <div class="flex items-center gap-1 bg-[#f3f3f3] p-1.5" :class="[tile]">
          <TabPill v-for="tab in tabs" :key="tab.name" :name="tab.name" :count="tab.count" :active="tab.name === 'Snippets'" />
        </div>
      </div>
      <div class="p-6 pt-0">
        <h3 class="text-lg font-medium tracking-tight">Collections & favorites</h3>
        <p class="text-neutral-500">Sort clips into your own collections; they stay as long as you keep them.</p>
      </div>
    </Motion>
  </div>
</template>
