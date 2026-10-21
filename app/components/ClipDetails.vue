<script setup lang="ts">
import type { Clip, PartialCopy } from '@/composables/useClips'
import { convertFileSrc, invoke } from '@tauri-apps/api/core'
import { openUrl } from '@tauri-apps/plugin-opener'
import { animate } from 'motion-v'
import { copiedText } from '@/composables/useClips'

const props = defineProps<{
  clip: Clip | null
  onCopy: (clip: Clip) => Promise<PartialCopy | null>
}>()

const emit = defineEmits<{
  close: []
}>()

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
const { text: imageText } = useImageText(shown, open)
const files = computed(() => (shown.value?.kind === 'files' ? shown.value.content.split('\n') : null))
const isGroup = computed(() => (files.value?.length ?? 0) > 1)
const commonFolder = computed(() => {
  const folders = new Set(files.value?.map(folderOf))
  return folders.size === 1 ? [...folders][0]! : null
})

const missing = ref(new Set<string>())
async function checkFiles() {
  const clip = shown.value
  if (!clip || clip.kind !== 'files' || !open.value) return (missing.value = new Set())
  const paths = await invoke<string[]>('missing_files', { id: clip.id }).catch(() => [])
  if (shown.value?.id === clip.id) missing.value = new Set(paths)
}
watch([() => shown.value?.id, open], checkFiles, { immediate: true })
// The files may have been deleted or restored while the window was in the background
useEventListener('focus', () => checkFiles())
const allMissing = computed(() => !!files.value && files.value.every(path => missing.value.has(path)))
const existing = (path: string | null | undefined) => (path && !missing.value.has(path) ? path : null)

const slide = ref(0)
watch(() => shown.value?.id, () => (slide.value = 0))
const slidePath = computed(() => files.value?.[slide.value] ?? null)
const slideMissing = computed(() => !!slidePath.value && missing.value.has(slidePath.value))
// Reloaded when a missing file comes back
const filePreview = useFilePreview(() => existing(slidePath.value), open, true)
useFilePreview(() => (isGroup.value ? existing(files.value![(slide.value + 1) % files.value!.length]) : null), open, true)

type MediaKind = 'animation' | 'video' | 'audio'
const media = ref<{ path: string, kind: MediaKind, src: string } | null>(null)
// Only changes once the new slide's kind is known: the dots don't jump while it loads
const playable = ref(false)
const isPlayable = (kind: MediaKind | null | undefined) => kind === 'video'
watch([() => shown.value?.id, () => existing(slidePath.value), open], async ([id, path, isOpen]) => {
  if (!path) playable.value = false
  if (!id || !path || !isOpen) return
  if (media.value?.path === path) return (playable.value = isPlayable(media.value.kind))
  const kind = await invoke<MediaKind | null>('allow_media', { id, path }).catch(() => null)
  if (shown.value?.id !== id || slidePath.value !== path) return
  playable.value = isPlayable(kind)
  if (kind) media.value = { path, kind, src: convertFileSrc(path) }
}, { immediate: true })
const slideMedia = computed(() => (media.value && media.value.path === existing(slidePath.value) ? media.value : null))
const failedMedia = ref(new Set<string>())
function mediaFailed() {
  if (media.value) failedMedia.value = new Set(failedMedia.value).add(media.value.path)
  media.value = null
  playable.value = false
}
// Laid out from the preview, before the file is allowed: no jump once it's playable
const audioShown = computed(() =>
  filePreview.value?.media === 'audio' && !!slidePath.value && !slideMissing.value && !failedMedia.value.has(slidePath.value)
)
let slideForward = true
function go(step: number) {
  if (!files.value || !isGroup.value) return
  slideForward = step > 0
  slide.value = (slide.value + step + files.value.length) % files.value.length
}
function goTo(index: number) {
  if (index === slide.value) return
  slideForward = index > slide.value
  slide.value = index
}
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
  if (files.value) return isGroup.value ? 'Files' : (filePreview.value?.typeName ?? 'File')
  if (color.value) return 'Color'
  if (svgSrc.value) return 'SVG'
  if (link.value) return 'Link'
  return 'Text'
})
const title = computed(() => {
  if (linkPreview.value?.title) return linkPreview.value.title
  if (files.value) return files.value.map(fileName).join(', ')
  if (color.value) return color.value.value
  return type.value
})
const dimensions = computed(() => {
  if (shown.value?.width && shown.value.height) return { width: shown.value.width, height: shown.value.height }
  if (svgSrc.value) return svgSize(text.value!)
  const preview = filePreview.value
  return files.value && !isGroup.value && preview?.width && preview.height ? { width: preview.width, height: preview.height } : null
})
const textStats = computed(() => {
  if (!text.value || color.value || link.value || svgSrc.value) return null
  return {
    characters: text.value.length,
    words: text.value.trim().split(/\s+/).filter(Boolean).length,
    lines: text.value.split('\n').length
  }
})
const byteSize = computed(() => shown.value?.byteSize ?? (svgSrc.value ? new TextEncoder().encode(text.value!).length : null))
const lastCopied = computed(() =>
  shown.value ? new Date(shown.value.createdAt).toLocaleString('en-US', { dateStyle: 'full', timeStyle: 'short' }) : ''
)
const formatNumber = (value: number) => value.toLocaleString('en-US')

const copySwap = {
  initial: { opacity: 0, scale: 0.8, filter: 'blur(4px)' },
  animate: { opacity: 1, scale: 1, filter: 'blur(0px)' },
  exit: { opacity: 0, scale: 0.8, filter: 'blur(4px)' }
}
const copySwapTransition = { type: 'spring', bounce: 0, duration: 0.3 } as const
const copyLabel = ref<string | null>(null)
const copyFailed = ref(false)
const { start: clearCopyLabelLater } = useTimeoutFn(() => (copyLabel.value = null), () => (copyFailed.value ? 1800 : 1200), { immediate: false })
async function copy() {
  if (!shown.value) return
  try {
    const partial = await props.onCopy(shown.value)
    copyFailed.value = false
    copyLabel.value = copiedText(partial)
  } catch (error) {
    copyFailed.value = true
    copyLabel.value = typeof error === 'string' ? error : 'Couldn’t copy'
    checkFiles()
  }
  clearCopyLabelLater()
}

const openLinkError = ref<string | null>(null)
const { start: clearOpenLinkErrorLater } = useTimeoutFn(() => (openLinkError.value = null), 1800, { immediate: false })
async function openLink(url: string) {
  await openUrl(url).catch(() => {
    openLinkError.value = 'Couldn’t open the link'
    clearOpenLinkErrorLater()
  })
}

const revealError = ref<string | null>(null)
const { start: clearRevealErrorLater } = useTimeoutFn(() => (revealError.value = null), 1800, { immediate: false })
async function revealFile(path: string) {
  if (!shown.value || missing.value.has(path)) return
  await invoke('reveal_file', { id: shown.value.id, path }).catch((error) => {
    checkFiles()
    revealError.value = typeof error === 'string' ? error : 'Couldn’t open the folder'
    clearRevealErrorLater()
  })
}

const fileCopy = ref<{ path: string, label: string, failed: boolean } | null>(null)
const { start: clearFileStatusLater } = useTimeoutFn(() => (fileCopy.value = null), () => (fileCopy.value?.failed ? 1800 : 1200), { immediate: false })
function showFileStatus(path: string, label: string, failed = false) {
  fileCopy.value = { path, label, failed }
  clearFileStatusLater()
}
async function copyFile(path: string) {
  if (!shown.value) return
  await invoke('copy_file', { id: shown.value.id, path })
    .then(() => showFileStatus(path, 'Copied'))
    .catch((error) => {
      showFileStatus(path, typeof error === 'string' ? error : 'Couldn’t copy', true)
      checkFiles()
    })
}
function fileCopyIcon(path: string) {
  if (fileCopy.value?.path !== path) return 'gravity-ui:copy'
  return fileCopy.value.failed ? 'gravity-ui:circle-xmark' : 'gravity-ui:check'
}

const textCopy = ref<{ label: string, failed: boolean } | null>(null)
const { start: clearTextCopyLater } = useTimeoutFn(() => (textCopy.value = null), () => (textCopy.value?.failed ? 1800 : 1200), { immediate: false })
async function copyImageText() {
  if (!shown.value) return
  await invoke('copy_image_text', { id: shown.value.id })
    .then(() => (textCopy.value = { label: 'Copied', failed: false }))
    .catch(error => (textCopy.value = { label: typeof error === 'string' ? error : 'Couldn’t copy', failed: true }))
  clearTextCopyLater()
}
watch(() => shown.value?.id, () => (textCopy.value = null))

async function dragFile(event: DragEvent, path: string) {
  event.preventDefault()
  if (!shown.value || missing.value.has(path)) return
  // Keeps the window from taking it as an external drop
  tileDragging.value = true
  await invoke('start_file_drag', { id: shown.value.id, path })
    .catch((error) => {
      showFileStatus(path, typeof error === 'string' ? error : 'Couldn’t drag', true)
      checkFiles()
    })
  tileDragging.value = false
}

const rendered = ref(false)
// In case the end of the animation isn't reported
const { start: unmountLater, stop: keepMounted } = useTimeoutFn(() => (rendered.value = false), 500, { immediate: false })
watch(open, (isOpen) => {
  if (isOpen) {
    keepMounted()
    rendered.value = true
  } else if (rendered.value) {
    unmountLater()
  }
}, { immediate: true })
const panelAnimation = computed(() =>
  open.value
    ? { x: 0, transition: { type: 'spring' as const, bounce: 0.15, visualDuration: 0.3 } }
    : { x: '105%', transition: { type: 'spring' as const, bounce: 0, visualDuration: 0.2 } }
)
function onPanelAnimated() {
  if (!open.value) rendered.value = false
}

const label = 'shrink-0 opacity-60'
const imageButton = 'absolute right-2 bottom-2 flex size-7 cursor-pointer items-center justify-center rounded-md bg-white/80 shadow-sm backdrop-blur-md transition-[background-color] duration-100 hover:bg-white focus-visible:outline-2 focus-visible:outline-current dark:bg-black/60 dark:hover:bg-black/80'
const carouselButton = 'absolute top-1/2 flex size-7 -translate-y-1/2 cursor-pointer items-center justify-center rounded-full bg-white/80 opacity-0 shadow-sm transition-[opacity,background-color] duration-150 group-hover/carousel:opacity-100 focus-visible:opacity-100 hover:bg-white focus-visible:outline-2 focus-visible:outline-current dark:bg-black/60 dark:hover:bg-black/80'
</script>

<template>
  <DialogRoot :open="open" @update:open="value => !value && emit('close')">
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
        <Motion
          :initial="{ x: '105%' }"
          :animate="panelAnimation"
          class="fixed inset-y-0 right-0 z-50 flex w-96 flex-col overflow-hidden border-l border-black/[5.78%] bg-[#f9f9f9] text-sm shadow-xl dark:border-black/20 dark:bg-[#2c2c2c]"
          @animation-complete="onPanelAnimated"
          @keydown.left="go(-1)"
          @keydown.right="go(1)"
        >
          <header class="flex h-12 shrink-0 items-center gap-2 pr-2 pl-4">
            <DialogTitle class="min-w-0 flex-1 truncate text-base font-medium">
              {{ title }}
            </DialogTitle>
            <DialogClose as-child>
              <Button icon="gravity-ui:xmark" size="sm" />
            </DialogClose>
          </header>

          <ScrollArea class="min-h-0 flex-1">
            <div class="px-4 pb-4">
              <div
                v-if="shown.kind === 'image' || link || color || svgSrc || files"
                class="relative mb-4 flex h-48 items-center justify-center overflow-hidden rounded-lg bg-black/[3.73%] dark:bg-white/[6.05%]"
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
                <div v-else-if="link" class="flex min-w-0 flex-col items-center gap-3 px-4">
                  <div class="flex size-16 items-center justify-center rounded-2xl bg-white shadow-sm dark:bg-white/[8.37%]">
                    <ClipImage v-if="linkPreview?.icon" :file="linkPreview.icon" class="size-9 object-contain" />
                    <Icon v-else name="gravity-ui:globe" class="text-[32px] opacity-40" />
                  </div>
                  <span class="max-w-full truncate opacity-60">{{ linkDomain(link) }}</span>
                </div>
                <div v-else-if="files" class="group/carousel relative size-full">
                  <Transition :css="false" @enter="slideIn" @leave="slideOut">
                    <div
                      :key="slidePath ?? ''"
                      class="absolute inset-0 flex items-center justify-center"
                    >
                      <img
                        v-if="slideMedia?.kind === 'animation'"
                        :src="slideMedia.src"
                        alt=""
                        class="size-full object-contain"
                        @error="mediaFailed"
                      >
                      <MediaPlayer
                        v-else-if="slideMedia?.kind === 'video'"
                        video
                        :src="slideMedia.src"
                        :poster="filePreview?.image"
                        class="size-full"
                        @error="mediaFailed"
                      />
                      <MediaPlayer
                        v-else-if="audioShown"
                        :src="slideMedia?.kind === 'audio' ? slideMedia.src : null"
                        :poster="filePreview?.thumbnail ? filePreview.image : null"
                        class="size-full"
                        :class="isGroup ? 'px-11' : 'px-5'"
                        @error="mediaFailed"
                      />
                      <img
                        v-else-if="filePreview?.image"
                        :src="filePreview.image"
                        alt=""
                        :class="filePreview.thumbnail ? 'size-full object-contain' : 'size-20 object-contain'"
                      >
                      <Icon v-else-if="filePreview || slideMissing" name="gravity-ui:file-xmark" class="text-[64px] opacity-40" />
                    </div>
                  </Transition>
                  <template v-if="isGroup">
                    <button type="button" class="left-2" :class="[carouselButton]" aria-label="Previous file" @click="go(-1)">
                      <Icon name="gravity-ui:chevron-left" class="text-sm" />
                    </button>
                    <button type="button" class="right-2" :class="[carouselButton]" aria-label="Next file" @click="go(1)">
                      <Icon name="gravity-ui:chevron-right" class="text-sm" />
                    </button>
                    <div
                      class="absolute left-1/2 -translate-x-1/2 rounded-full bg-white/80 px-1.5 py-1 opacity-0 transition-opacity duration-150 group-hover/carousel:opacity-100 has-focus-visible:opacity-100 dark:bg-black/60"
                      :class="playable ? 'top-2' : 'bottom-2'"
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
                <Tooltip v-if="imageText" :content="textCopy?.label ?? 'Copy text'">
                  <button type="button" :class="imageButton" aria-label="Copy text" @click="copyImageText">
                    <Icon :name="textCopy ? (textCopy.failed ? 'gravity-ui:circle-xmark' : 'gravity-ui:check') : 'gravity-ui:square-dashed-text'" class="text-sm" />
                  </button>
                </Tooltip>
              </div>
              <p v-if="isGroup && slidePath" class="-mt-2 mb-4 truncate text-center opacity-70">
                {{ fileName(slidePath) }}
              </p>

              <ScrollArea
                v-if="text && !color"
                class="mb-4 rounded-lg bg-black/[3.73%] dark:bg-white/[6.05%]"
                viewport-class="max-h-64"
              >
                <p class="p-3 whitespace-pre-wrap wrap-anywhere select-text" :class="svgSrc && 'font-mono text-xs'">
                  {{ text }}
                </p>
              </ScrollArea>
              <p v-if="linkPreview?.description" class="mb-4 opacity-80">
                {{ linkPreview.description }}
              </p>

              <dl class="flex flex-col gap-2.5">
                <div class="flex justify-between gap-4">
                  <dt :class="label">Type</dt>
                  <dd>{{ type }}</dd>
                </div>
                <div class="flex justify-between gap-4">
                  <dt :class="label">Source</dt>
                  <dd class="flex min-w-0 items-center gap-1.5">
                    <AppIcon :path="shown.sourceAppPath" />
                    <span class="truncate">{{ shown.sourceAppName ?? 'Unknown app' }}</span>
                  </dd>
                </div>
                <div class="flex justify-between gap-4">
                  <dt :class="label">{{ shown.inHistory ? 'Copied' : 'Added' }}</dt>
                  <dd class="text-right">{{ lastCopied }}</dd>
                </div>
                <div v-if="dimensions" class="flex justify-between gap-4">
                  <dt :class="label">Dimensions</dt>
                  <dd>{{ dimensions.width }} × {{ dimensions.height }}</dd>
                </div>
                <div v-if="byteSize" class="flex justify-between gap-4">
                  <dt :class="label">Size</dt>
                  <dd>{{ formatBytes(byteSize) }}</dd>
                </div>
                <div v-if="color" class="flex justify-between gap-4">
                  <dt :class="label">Format</dt>
                  <dd>{{ color.format }}</dd>
                </div>
                <div v-if="link" class="flex justify-between gap-4">
                  <dt :class="label">Website</dt>
                  <dd class="flex min-w-0">
                    <Tooltip :content="openLinkError ?? 'Open in browser'">
                      <button
                        type="button"
                        class="-mx-1.5 -my-0.5 flex min-w-0 cursor-pointer items-center gap-1.5 rounded-md px-1.5 py-0.5 transition-colors duration-100 hover:bg-black/[3.73%] focus-visible:outline-2 focus-visible:outline-current dark:hover:bg-white/[6.05%]"
                        @click="openLink(link)"
                      >
                        <span class="truncate">{{ linkDomain(link) }}</span>
                        <Icon name="gravity-ui:arrow-up-right-from-square" class="shrink-0 text-sm opacity-60" />
                      </button>
                    </Tooltip>
                  </dd>
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
                <div v-if="files && !isGroup" class="flex justify-between gap-4">
                  <dt :class="label">Location</dt>
                  <dd class="flex min-w-0">
                    <Tooltip :content="revealError ?? (allMissing ? 'File not found' : 'Show in File Explorer')">
                      <!-- aria-disabled rather than disabled: a disabled button gets no hover, so no tooltip -->
                      <button
                        type="button"
                        :aria-disabled="allMissing"
                        class="-mx-1.5 -my-0.5 flex min-w-0 items-center gap-1.5 rounded-md px-1.5 py-0.5 transition-colors duration-100 focus-visible:outline-2 focus-visible:outline-current"
                        :class="allMissing ? 'opacity-50' : 'cursor-pointer hover:bg-black/[3.73%] dark:hover:bg-white/[6.05%]'"
                        @click="revealFile(files[0]!)"
                      >
                        <Icon name="gravity-ui:folder" class="shrink-0 text-sm opacity-60" />
                        <span class="truncate">{{ fileName(folderOf(files[0]!)) }}</span>
                      </button>
                    </Tooltip>
                  </dd>
                </div>
                <div v-if="files && isGroup" class="flex flex-col gap-1.5">
                  <dt :class="label">Files</dt>
                  <dd v-if="commonFolder" class="flex min-w-0 items-center gap-1.5">
                    <Icon name="gravity-ui:folder" class="shrink-0 text-sm opacity-60" />
                    <Tooltip :content="commonFolder" path>
                      <span class="truncate opacity-80">{{ commonFolder }}</span>
                    </Tooltip>
                  </dd>
                  <dd class="flex flex-col">
                    <div v-for="(path, index) in files" :key="path" class="group/file relative">
                      <button
                        type="button"
                        :draggable="!missing.has(path)"
                        class="flex w-full min-w-0 cursor-pointer flex-col rounded-md px-2 py-1.5 text-left transition-colors duration-100 hover:bg-black/[3.73%] dark:hover:bg-white/[6.05%]"
                        :class="[
                          missing.has(path) ? 'opacity-50' : 'group-hover/file:pr-8 group-has-focus-visible/file:pr-8',
                          fileCopy?.path === path && 'pr-8',
                          index === slide && 'bg-black/[5.78%] dark:bg-white/[8.37%]'
                        ]"
                        @click="goTo(index)"
                        @dragstart="dragFile($event, path)"
                      >
                        <span class="w-full truncate">{{ fileName(path) }}</span>
                        <span v-if="!commonFolder" class="w-full truncate text-xs opacity-60">{{ folderOf(path) }}</span>
                      </button>
                      <Tooltip v-if="!missing.has(path) || fileCopy?.path === path" :content="fileCopy?.path === path ? fileCopy.label : 'Copy this file'">
                        <button
                          type="button"
                          aria-label="Copy this file"
                          class="absolute top-1/2 right-1.5 flex size-6 -translate-y-1/2 cursor-pointer items-center justify-center rounded-md opacity-0 group-hover/file:opacity-60 hover:opacity-100! focus-visible:opacity-100 focus-visible:outline-2 focus-visible:outline-current"
                          :class="fileCopy?.path === path && 'opacity-100!'"
                          @click="copyFile(path)"
                        >
                          <Icon :name="fileCopyIcon(path)" class="text-base" />
                        </button>
                      </Tooltip>
                    </div>
                  </dd>
                </div>
              </dl>
            </div>
          </ScrollArea>

          <footer class="flex gap-2 border-t border-black/[5.78%] p-3 dark:border-white/6">
            <Button variant="filled" class="relative flex-1" :disabled="allMissing" @click="copy">
              <AnimatePresence mode="popLayout" :initial="false">
                <Motion
                  :key="copyLabel ?? 'Copy'"
                  v-bind="copySwap"
                  :transition="copySwapTransition"
                  class="flex min-w-0 items-center justify-center gap-2"
                >
                  <Icon
                    :name="copyLabel ? (copyFailed ? 'gravity-ui:circle-xmark' : 'gravity-ui:check') : 'gravity-ui:copy'"
                    class="shrink-0 text-base opacity-60 transition-opacity duration-100 group-hover:opacity-100"
                  />
                  <span class="truncate text-sm">{{ copyLabel ?? 'Copy' }}</span>
                </Motion>
              </AnimatePresence>
            </Button>
          </footer>
        </Motion>
      </DialogContent>
    </DialogPortal>
  </DialogRoot>
</template>
