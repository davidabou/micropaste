<script setup lang="ts">
import { animate } from 'motion-v'
import type { Clip } from '~/composables/useClips'

const props = defineProps<{
  /** Clip to detail, `null` when the panel is closed */
  clip: Clip | null
  /** Bound with `@copy`: the Copy button shows "Copied" once the returned promise resolves */
  onCopy: (clip: Clip) => Promise<unknown>
}>()

const emit = defineEmits<{
  close: []
  favorite: [clip: Clip]
}>()

// The last clip shown stays while the panel slides out
const shown = ref<Clip | null>(null)
watch(() => props.clip, (clip) => {
  if (clip) shown.value = clip
}, { immediate: true })
const open = computed(() => props.clip !== null)

const prefersDark = usePrefersDark()
const text = computed(() => (shown.value?.kind === 'text' ? shown.value.content : null))
const color = computed(() => (text.value ? copiedColor(text.value) : null))
const svgSrc = computed(() => (text.value ? svgToDataUrl(text.value, prefersDark.value ? '#fff' : '#000') : null))
const link = computed(() => (text.value ? copiedLink(text.value) : null))
const linkPreview = useLinkPreview(link, open)
const files = computed(() => (shown.value?.kind === 'files' ? shown.value.content.split('\n') : null))
const isGroup = computed(() => (files.value?.length ?? 0) > 1)
/** Folder holding every file, `null` when they come from different folders */
const commonFolder = computed(() => {
  const folders = new Set(files.value?.map(folderOf))
  return folders.size === 1 ? [...folders][0]! : null
})

// Several files: a carousel, one file at a time. Arrows (buttons or keys) slide to the previous or
// next one, wrapping around, and the next one is loaded ahead so it shows at once
const slide = ref(0)
watch(() => shown.value?.id, () => (slide.value = 0))
const slidePath = computed(() => files.value?.[slide.value] ?? null)
const filePreview = useFilePreview(slidePath, open)
useFilePreview(() => (isGroup.value ? files.value![(slide.value + 1) % files.value!.length]! : null), open)
let slideForward = true
function go(step: number) {
  if (!files.value || !isGroup.value) return
  slideForward = step > 0
  slide.value = (slide.value + step + files.value.length) % files.value.length
}
/** Jumps to a file from its dot, sliding toward it */
function goTo(index: number) {
  if (index === slide.value) return
  slideForward = index > slide.value
  slide.value = index
}
// One dot per file, the current one wider. Past 7 files, like Instagram: a window of 7 dots follows
// the current file, and the dots at its edges shrink to tell more lie beyond
const DOT = 6
const ACTIVE_DOT = 16
const DOT_GAP = 4
const VISIBLE_DOTS = 7
const dotCount = computed(() => files.value?.length ?? 0)
const dotOffset = computed(() =>
  Math.min(Math.max(slide.value - Math.floor(VISIBLE_DOTS / 2), 0), Math.max(dotCount.value - VISIBLE_DOTS, 0))
)
const dotsWidth = computed(() => {
  const shown = Math.min(dotCount.value, VISIBLE_DOTS)
  return (shown - 1) * (DOT + DOT_GAP) + ACTIVE_DOT
})
function dotScale(index: number) {
  const position = index - dotOffset.value
  const hiddenBefore = dotOffset.value > 0
  const hiddenAfter = dotOffset.value + VISIBLE_DOTS < dotCount.value
  if (position < 0 || position >= VISIBLE_DOTS) return 0
  if ((position === 0 && hiddenBefore) || (position === VISIBLE_DOTS - 1 && hiddenAfter)) return 0.5
  if ((position === 1 && hiddenBefore) || (position === VISIBLE_DOTS - 2 && hiddenAfter)) return 0.75
  return 1
}
const dotSpring = { type: 'spring' as const, bounce: 0.2, visualDuration: 0.3 }
// Through Transition hooks: they read the direction when the slide starts, the leaving one included
const slideSpring = { type: 'spring' as const, bounce: 0, visualDuration: 0.3 }
function slideIn(el: Element, done: () => void) {
  animate(el, { x: [slideForward ? '100%' : '-100%', 0] }, slideSpring).then(done)
}
function slideOut(el: Element, done: () => void) {
  animate(el, { x: slideForward ? '-100%' : '100%' }, slideSpring).then(done)
}

const type = computed(() => {
  if (!shown.value) return ''
  if (shown.value.kind === 'image') return 'Image'
  if (files.value) return files.value.length > 1 ? `${files.value.length} files` : 'File'
  if (color.value) return 'Color'
  if (svgSrc.value) return 'SVG'
  if (link.value) return 'Link'
  return 'Text'
})
const title = computed(() => {
  if (linkPreview.value?.title) return linkPreview.value.title
  if (files.value) return isGroup.value ? `${files.value.length} files` : fileName(files.value[0]!)
  if (color.value) return color.value.value
  return type.value
})
// Characters, words and lines of a plain text
const textStats = computed(() => {
  if (!text.value || color.value || link.value) return null
  return {
    characters: text.value.length,
    words: text.value.trim().split(/\s+/).filter(Boolean).length,
    lines: text.value.split('\n').length
  }
})
const lastCopied = computed(() =>
  shown.value ? new Date(shown.value.createdAt).toLocaleString('en-US', { dateStyle: 'full', timeStyle: 'short' }) : ''
)
const formatNumber = (value: number) => value.toLocaleString('en-US')

// The Copy button turns into "Copied" for a moment, or tells why it couldn't copy
const copyLabel = ref<string | null>(null)
const copyFailed = ref(false)
let copiedTimer: ReturnType<typeof setTimeout> | undefined
async function copy() {
  if (!shown.value) return
  try {
    await props.onCopy(shown.value)
    copyFailed.value = false
    copyLabel.value = 'Copied'
  } catch (error) {
    copyFailed.value = true
    // The copied files no longer exist…
    copyLabel.value = typeof error === 'string' ? error : 'Couldn’t copy'
  }
  clearTimeout(copiedTimer)
  copiedTimer = setTimeout(() => (copyLabel.value = null), copyFailed.value ? 1800 : 1200)
}
onBeforeUnmount(() => clearTimeout(copiedTimer))

// Slides in from the right on a spring, over a fading backdrop, and slides back out. It stays
// rendered while it slides out, and is removed once that animation is over
const rendered = ref(false)
let unmountTimer: ReturnType<typeof setTimeout> | undefined
watch(open, (isOpen) => {
  clearTimeout(unmountTimer)
  if (isOpen) rendered.value = true
  // In case the end of the animation isn't reported: removed anyway once it surely is over
  else unmountTimer = setTimeout(() => (rendered.value = false), 500)
}, { immediate: true })
onBeforeUnmount(() => clearTimeout(unmountTimer))
const panelAnimation = computed(() =>
  open.value
    ? { x: 0, transition: { type: 'spring' as const, bounce: 0.15, visualDuration: 0.3 } }
    : { x: '105%', transition: { type: 'spring' as const, bounce: 0, visualDuration: 0.2 } }
)
function onPanelAnimated() {
  if (!open.value) rendered.value = false
}

const label = 'shrink-0 opacity-60'
// Previous and next buttons, over the sides of the carousel
// Shown while the pointer is over the carousel (or a button has the keyboard focus)
const carouselButton = 'absolute top-1/2 flex size-7 -translate-y-1/2 cursor-pointer items-center justify-center rounded-full bg-white/80 opacity-0 shadow-sm transition-[opacity,background-color] duration-150 group-hover/carousel:opacity-100 focus-visible:opacity-100 hover:bg-white focus-visible:outline-2 focus-visible:outline-current dark:bg-black/60 dark:hover:bg-black/80'
</script>

<template>
  <DialogRoot :open="open" @update:open="value => !value && emit('close')">
    <!-- force-mount: the panel decides when it leaves, once it has slid out -->
    <DialogPortal force-mount>
      <DialogOverlay v-if="rendered" force-mount as-child>
        <Motion
          class="fixed inset-0 z-40 bg-black/20"
          :class="!open && 'pointer-events-none'"
          :initial="{ opacity: 0 }"
          :animate="{ opacity: open ? 1 : 0 }"
          :transition="{ duration: 0.2 }"
        />
      </DialogOverlay>
      <DialogContent v-if="rendered && shown" force-mount as-child :aria-describedby="undefined">
        <!-- Left and right arrows browse the files of the carousel -->
        <Motion
          :initial="{ x: '105%' }"
          :animate="panelAnimation"
          @animation-complete="onPanelAnimated"
          @keydown.left="go(-1)"
          @keydown.right="go(1)"
          class="fixed inset-y-2 right-2 z-50 flex w-96 flex-col overflow-hidden rounded-lg border border-black/[5.78%] bg-[#f9f9f9] font-ui text-sm shadow-2xl dark:border-black/20 dark:bg-[#2c2c2c]"
        >
          <!-- Header: what it is, and the close button -->
          <header class="flex items-center gap-2 px-4 pt-4 pb-3">
            <DialogTitle class="min-w-0 flex-1 truncate text-base font-medium">
              {{ title }}
            </DialogTitle>
            <DialogClose as-child>
              <Button icon="gravity-ui:xmark" size="sm" />
            </DialogClose>
          </header>

          <div class="min-h-0 flex-1 overflow-y-auto px-4 pb-4">
            <!-- Large preview -->
            <div
              v-if="shown.kind === 'image' || linkPreview?.image || color || svgSrc || files"
              class="mb-4 flex h-48 items-center justify-center overflow-hidden rounded-lg bg-black/[3.73%] dark:bg-white/[6.05%]"
            >
              <ClipImage v-if="shown.kind === 'image'" :file="`${shown.content}.png`" class="size-full object-contain" />
              <ClipImage v-else-if="linkPreview?.image" :file="linkPreview.image" class="size-full object-cover" />
              <div
                v-else-if="color"
                class="size-full bg-[conic-gradient(#ccc_25%,#fff_0_50%,#ccc_0_75%,#fff_0)] bg-size-[16px_16px]"
              >
                <div class="size-full" :style="{ background: color.css }" />
              </div>
              <img v-else-if="svgSrc" :src="svgSrc" alt="" class="size-32 object-contain">
              <div v-else-if="files" class="group/carousel relative size-full">
                <Transition :css="false" @enter="slideIn" @leave="slideOut">
                  <div :key="slidePath ?? ''" class="absolute inset-0 flex items-center justify-center">
                    <img
                      v-if="filePreview?.image"
                      :src="filePreview.image"
                      alt=""
                      :class="filePreview.thumbnail ? 'size-full object-contain' : 'size-20 object-contain'"
                    >
                    <Icon v-else-if="filePreview" name="gravity-ui:file-xmark" class="text-[64px] opacity-40" />
                  </div>
                </Transition>
                <template v-if="isGroup">
                  <button type="button" :class="[carouselButton, 'left-2']" aria-label="Previous file" @click="go(-1)">
                    <Icon name="gravity-ui:chevron-left" class="text-sm" />
                  </button>
                  <button type="button" :class="[carouselButton, 'right-2']" aria-label="Next file" @click="go(1)">
                    <Icon name="gravity-ui:chevron-right" class="text-sm" />
                  </button>
                  <!-- Dots, shown with the arrows. Each one animates its own width, so the row stays smooth
                       both ways. Clicking a dot goes to its file -->
                  <div
                    class="absolute bottom-2 left-1/2 -translate-x-1/2 rounded-full bg-white/80 px-1.5 py-1 opacity-0 transition-opacity duration-150 group-hover/carousel:opacity-100 has-focus-visible:opacity-100 dark:bg-black/60"
                  >
                    <div class="overflow-hidden" :style="{ width: `${dotsWidth}px` }">
                      <Motion
                        class="flex items-center"
                        :style="{ gap: `${DOT_GAP}px` }"
                        :initial="false"
                        :animate="{ x: -dotOffset * (DOT + DOT_GAP) }"
                        :transition="dotSpring"
                      >
                        <Motion
                          v-for="(path, index) in files"
                          :key="path"
                          as="button"
                          type="button"
                          class="h-1.5 shrink-0 cursor-pointer rounded-full bg-current"
                          :aria-label="`File ${index + 1}`"
                          :initial="false"
                          :animate="{ width: index === slide ? ACTIVE_DOT : DOT, opacity: index === slide ? 1 : 0.3, scale: dotScale(index) }"
                          :transition="dotSpring"
                          @click="goTo(index)"
                        />
                      </Motion>
                    </div>
                  </div>
                </template>
              </div>
            </div>
            <!-- Name of the file shown by the carousel -->
            <p v-if="isGroup && slidePath" class="-mt-2 mb-4 truncate text-center opacity-70">
              {{ fileName(slidePath) }}
            </p>

            <!-- The full text, of a text, a link or a code -->
            <p
              v-if="text && !color"
              class="mb-4 max-h-64 overflow-y-auto rounded-lg bg-black/[3.73%] p-3 whitespace-pre-wrap wrap-anywhere select-text dark:bg-white/[6.05%]"
            >
              {{ text }}
            </p>
            <p v-if="linkPreview?.description" class="mb-4 opacity-80">
              {{ linkPreview.description }}
            </p>

            <!-- Information, one per line -->
            <dl class="flex flex-col gap-2.5">
              <div class="flex justify-between gap-4">
                <dt :class="label">Type</dt>
                <dd>{{ type }}</dd>
              </div>
              <div class="flex justify-between gap-4">
                <dt :class="label">Copied from</dt>
                <dd class="flex min-w-0 items-center gap-1.5">
                  <AppIcon :path="shown.sourceAppPath" />
                  <span class="truncate">{{ shown.sourceAppName ?? 'Unknown app' }}</span>
                </dd>
              </div>
              <div class="flex justify-between gap-4">
                <dt :class="label">Last copied</dt>
                <dd class="text-right">{{ lastCopied }}</dd>
              </div>
              <div v-if="shown.width && shown.height" class="flex justify-between gap-4">
                <dt :class="label">Dimensions</dt>
                <dd>{{ shown.width }} × {{ shown.height }}</dd>
              </div>
              <div v-if="shown.byteSize" class="flex justify-between gap-4">
                <dt :class="label">Size</dt>
                <dd>{{ formatBytes(shown.byteSize) }}</dd>
              </div>
              <div v-if="color" class="flex justify-between gap-4">
                <dt :class="label">Format</dt>
                <dd>{{ color.format }}</dd>
              </div>
              <div v-if="link" class="flex justify-between gap-4">
                <dt :class="label">Website</dt>
                <dd class="truncate">{{ linkDomain(link) }}</dd>
              </div>
              <template v-if="textStats">
                <div class="flex justify-between gap-4">
                  <dt :class="label">Characters</dt>
                  <dd>{{ formatNumber(textStats.characters) }}</dd>
                </div>
                <div class="flex justify-between gap-4">
                  <dt :class="label">Words</dt>
                  <dd>{{ formatNumber(textStats.words) }}</dd>
                </div>
                <div class="flex justify-between gap-4">
                  <dt :class="label">Lines</dt>
                  <dd>{{ formatNumber(textStats.lines) }}</dd>
                </div>
              </template>
              <!-- One line per file. A folder shared by every file shows once on top, else under each name.
                   The file shown by the carousel is highlighted, clicking another one slides to it -->
              <div v-if="files" class="flex flex-col gap-1.5">
                <dt :class="label">{{ files.length > 1 ? 'Files' : 'File' }}</dt>
                <dd v-if="commonFolder" class="flex min-w-0 items-center gap-1.5">
                  <Icon name="gravity-ui:folder" class="shrink-0 text-sm opacity-60" />
                  <Tooltip :content="commonFolder" path>
                    <span class="truncate opacity-80">{{ commonFolder }}</span>
                  </Tooltip>
                </dd>
                <dd class="flex flex-col">
                  <button
                    v-for="(path, index) in files"
                    :key="path"
                    type="button"
                    class="flex min-w-0 cursor-pointer flex-col rounded-md px-2 py-1.5 text-left transition-colors duration-100 hover:bg-black/[3.73%] dark:hover:bg-white/[6.05%]"
                    :class="isGroup && index === slide && 'bg-black/[5.78%] dark:bg-white/[8.37%]'"
                    @click="goTo(index)"
                  >
                    <span class="truncate">{{ fileName(path) }}</span>
                    <span v-if="!commonFolder" class="truncate text-xs opacity-60">{{ folderOf(path) }}</span>
                  </button>
                </dd>
              </div>
            </dl>
          </div>

          <!-- Actions -->
          <footer class="flex gap-2 border-t border-black/[5.78%] p-3 dark:border-white/6">
            <Button
              variant="filled"
              :icon="copyLabel && copyFailed ? 'gravity-ui:circle-xmark' : 'gravity-ui:copy'"
              class="flex-1"
              @click="copy"
            >
              {{ copyLabel ?? 'Copy' }}
            </Button>
            <!-- Icon only: the filled star tells it is a favorite -->
            <Tooltip :content="shown.favorite ? 'Remove from favorites' : 'Add to favorites'">
              <Button
                :icon="shown.favorite ? 'gravity-ui:star-fill' : 'gravity-ui:star'"
                :pressed="shown.favorite"
                @click="emit('favorite', shown)"
              />
            </Tooltip>
          </footer>
        </Motion>
      </DialogContent>
    </DialogPortal>
  </DialogRoot>
</template>
