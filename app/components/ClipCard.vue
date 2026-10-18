<script setup lang="ts">
import type { Clip, PartialCopy } from '@/composables/useClips'
import { invoke } from '@tauri-apps/api/core'
import { copiedText } from '@/composables/useClips'

const props = defineProps<{
  clip: Clip
  onCopy?: () => Promise<PartialCopy | null>
  inCollection?: boolean
  confirm?: 'delete' | 'remove' | null
  layout?: 'grid' | 'list'
}>()

defineEmits<{
  favorite: []
  info: []
  askDelete: []
  delete: []
  cancelDelete: []
}>()

const cornerButton = 'flex size-7 cursor-pointer items-center justify-center rounded-md bg-white/80 backdrop-blur-md transition-[background-color] duration-100 hover:bg-neutral-200/90 active:bg-neutral-300/90 focus-visible:outline-2 focus-visible:outline-current dark:bg-black/60 dark:hover:bg-neutral-700/80 dark:active:bg-neutral-800/80'

const hovered = ref(false)
const focused = ref(false)
const actionsShown = computed(() => hovered.value || focused.value)
const cornerHidden = { opacity: 0, y: -10, scale: 0.85 }
function cornerAnimation(index: number) {
  return actionsShown.value
    ? { opacity: 1, y: 0, scale: 1, transition: { type: 'spring' as const, bounce: 0.45, duration: 0.4, delay: index * 0.05 } }
    : { ...cornerHidden, transition: { duration: 0.12 } }
}
const actionsMounted = ref(false)
const { start: unmountActionsLater, stop: keepActions } = useTimeoutFn(() => (actionsMounted.value = false), 150, { immediate: false })
watch(actionsShown, (shown) => {
  if (!shown) return unmountActionsLater()
  keepActions()
  actionsMounted.value = true
})

const notice = ref<{ text: string, error: boolean } | null>(null)
const copying = ref(false)
const { start: hideNoticeLater } = useTimeoutFn(() => (notice.value = null), () => (notice.value?.error ? 1800 : 1200), { immediate: false })

function showNotice(text: string, error = false) {
  notice.value = { text, error }
  hideNoticeLater()
}

const errorText = (error: unknown) => (typeof error === 'string' ? error : 'Couldn’t copy')

async function copy() {
  copying.value = true
  try {
    showNotice(copiedText(await props.onCopy?.() ?? null))
  } catch (error) {
    showNotice(errorText(error), true)
  } finally {
    copying.value = false
  }
}

watch(clipNotice, (shown) => {
  if (shown?.clip === props.clip.id) showNotice(shown.text)
})

defineExpose({ copy })

const dragging = ref(false)
const tileSpring = { type: 'spring', bounce: 0, duration: 0.2 } as const
const watchingDragEnd = ref(false)
useEventListener(() => (watchingDragEnd.value ? window : null), 'pointermove', () => (dragging.value = false))

// Not Motion's `whilePress`: once a drag starts, the button never gets its `pointerup` and would stay shrunk
const pressed = ref(false)
const release = () => (pressed.value = false)
useEventListener(() => (pressed.value ? window : null), ['pointerup', 'pointercancel'], release)
function press(event: PointerEvent) {
  if (event.button === 0) pressed.value = true
}

watch(dragging, (isDragging) => {
  tileDragging.value = isDragging
  if (isDragging) return
  draggedClip.value = null
  watchingDragEnd.value = false
})

// The webview snapshots the tile wrongly on its own (the whole page moves along), and without
// the Mica behind it: the tile is made opaque, in the same color, for the snapshot only
function setDragImage(event: DragEvent) {
  const tile = event.currentTarget as HTMLElement
  const rect = tile.getBoundingClientRect()
  Object.assign(tile.style, {
    transition: 'none',
    backgroundColor: prefersDark.value ? '#2e2e2e' : '#eaeaea',
    boxShadow: 'none'
  })
  event.dataTransfer!.setDragImage(tile, event.clientX - rect.left, event.clientY - rect.top)
  requestAnimationFrame(() => Object.assign(tile.style, { transition: '', backgroundColor: '', boxShadow: '' }))
}

async function onDragStart(event: DragEvent) {
  release()
  dragging.value = true
  // Right away, not on the next update: the window checks it from the first `dragover`
  tileDragging.value = true
  draggedClip.value = props.clip
  if (props.clip.kind === 'text') {
    event.dataTransfer?.setData('text/plain', props.clip.content)
    if (link.value) event.dataTransfer?.setData('text/uri-list', link.value)
    if (event.dataTransfer) setDragImage(event)
    // `dragend` doesn't always reach the page once dropped in another app. The page gets
    // no mouse moves during a webview drag, so the first one back means the drag is over.
    // Next frame: skips a move that might still come along with the drag start
    requestAnimationFrame(() => (watchingDragEnd.value = dragging.value))
    return
  }
  event.preventDefault()
  await invoke('start_drag', { id: props.clip.id }).catch(error => showNotice(errorText(error), true))
  dragging.value = false
}

const timeAgo = useTimeAgo(() => props.clip.createdAt, compactTimeAgo)
const isImage = computed(() => props.clip.kind === 'image')

const prefersDark = usePrefersDark()
const svgSrc = computed(() =>
  props.clip.kind === 'text'
    ? svgToDataUrl(props.clip.content, prefersDark.value ? '#fff' : '#000')
    : null
)

const color = computed(() => (props.clip.kind === 'text' ? copiedColor(props.clip.content) : null))

const link = computed(() => (props.clip.kind === 'text' ? copiedLink(props.clip.content) : null))
const root = useTemplateRef<HTMLElement>('root')
// Relative to the grid: it slides in from off-screen
const scroller = computed(() => root.value?.closest<HTMLElement>('[data-clip-scroller]') ?? null)
const visible = ref(false)
const { stop: stopWatchingVisibility } = useIntersectionObserver(root, ([entry]) => {
  if (!entry?.isIntersecting) return
  visible.value = true
  stopWatchingVisibility()
}, { root: scroller, rootMargin: '200px' })
const linkPreview = useLinkPreview(link, visible)
const preview = computed(() =>
  linkPreview.value && (linkPreview.value.title || linkPreview.value.image) ? linkPreview.value : null
)

const files = computed(() => (props.clip.kind === 'files' ? props.clip.content.split('\n') : null))
const isFileGroup = computed(() => (files.value?.length ?? 0) > 1)
const filePreviews = [0, 1, 2].map(i => useFilePreview(() => files.value?.[i] ?? null, visible))
const filePreview = filePreviews[0]!
const fileThumbnail = computed(() =>
  !isFileGroup.value && filePreview.value?.thumbnail ? filePreview.value.image : null
)
const fileTitle = computed(() => {
  if (!files.value) return null
  return isFileGroup.value ? `${files.value.length} items` : fileName(files.value[0]!)
})

const STACK_LAYOUTS: Record<number, { rotate: number, x: number }[]> = {
  2: [{ rotate: 5, x: 10 }, { rotate: -5, x: -10 }],
  3: [{ rotate: 0, x: 0 }, { rotate: -9, x: -22 }, { rotate: 9, x: 22 }]
}
const fileStack = computed(() => {
  if (!files.value || !isFileGroup.value) return []
  const layout = STACK_LAYOUTS[Math.min(files.value.length, 3)]!
  return layout.map((position, i) => ({ ...position, preview: filePreviews[i]!.value, z: layout.length - i }))
})

const coverImage = computed(() => (isImage.value ? `${props.clip.content}.png` : preview.value?.image ?? null))
const hasCover = computed(() => coverImage.value !== null || fileThumbnail.value !== null || color.value !== null)
const footerText = computed(() => (color.value ? (isLightColor(color.value.css) ? 'text-black/90' : 'text-white') : null))
const pageTitle = computed(() => preview.value?.title ?? null)
const title = computed(() => (link.value ? linkLabel(link.value) : fileTitle.value ?? color.value?.value))
const isList = computed(() => props.layout === 'list')
const listTitle = computed(() => {
  if (title.value) return title.value
  if (isImage.value) return props.clip.width && props.clip.height ? `Image ${props.clip.width} × ${props.clip.height}` : 'Image'
  return props.clip.content.trim().replace(/\s+/g, ' ')
})
const listTextMask = computed(() => {
  const covered = props.inCollection ? 96 : 64
  return `linear-gradient(to left, transparent ${covered}px, black ${covered + 32}px)`
})
const copiedAt = computed(() => new Date(props.clip.createdAt).toLocaleString('en-US', { dateStyle: 'medium', timeStyle: 'short' }))
</script>

<template>
  <!-- Wrapper: the delete button can't sit inside the tile button.
       isolate: the footer z-index stays inside the tile -->
  <div
    ref="root"
    class="group relative isolate"
    @pointerenter="hovered = true"
    @pointerleave="hovered = false"
    @focusin="focused = ($event.target as Element).matches(':focus-visible')"
    @focusout="focused = false"
  >
    <Motion
      as="button"
      type="button"
      class="peer relative flex w-full cursor-pointer flex-col overflow-hidden rounded-lg border border-black/[5.78%] bg-black/[3.73%] dark:border-white/6 text-left transition-[background-color,box-shadow] duration-100 hover:bg-black/[5.78%] hover:ring-2 hover:ring-black/10 focus-visible:outline-2 focus-visible:outline-current dark:bg-white/[6.05%] dark:hover:bg-white/[8.37%] dark:hover:ring-white/15"
      :class="isList ? 'h-14' : 'h-44'"
      :animate="dragging ? { rotate: isList ? 0 : 1, scale: 0.96, opacity: 0.8 } : { rotate: 0, scale: pressed ? 0.98 : 1, opacity: 1 }"
      :transition="tileSpring"
      draggable="true"
      @pointerdown="press"
      @click="copy"
      @dragstart="onDragStart"
      @dragend="dragging = false"
    >
      <div
        v-if="isList"
        class="absolute inset-0 flex items-center gap-3 pr-3 pl-2 transition-[filter] duration-200"
        :class="(notice || confirm) && 'blur-sm'"
      >
        <div class="relative flex size-10 shrink-0 items-center justify-center overflow-hidden rounded-md bg-black/[3.73%] dark:bg-white/[6.05%]">
          <ClipImage v-if="linkPreview?.icon" :file="linkPreview.icon" class="size-6 object-contain" />
          <ClipImage v-else-if="coverImage && !link" :file="coverImage" class="size-full object-cover" />
          <img v-else-if="fileThumbnail" :src="fileThumbnail" alt="" draggable="false" class="animate-fade-in size-full object-cover">
          <div
            v-else-if="color"
            class="size-full bg-[conic-gradient(#ccc_25%,#fff_0_50%,#ccc_0_75%,#fff_0)] bg-size-[8px_8px]"
          >
            <div class="size-full" :style="{ background: color.css }" />
          </div>
          <img v-else-if="svgSrc" :src="svgSrc" alt="" draggable="false" class="size-6 object-contain">
          <Icon v-else-if="link" name="gravity-ui:globe" class="text-[20px] opacity-50" />
          <img v-else-if="filePreview?.image" :src="filePreview.image" alt="" draggable="false" class="animate-fade-in size-7 object-contain">
          <Icon v-else-if="files && filePreview" name="gravity-ui:file-xmark" class="animate-fade-in text-[20px] opacity-50" />
          <Icon v-else-if="files" name="gravity-ui:file" class="text-[20px] opacity-20" />
          <Icon v-else name="gravity-ui:text-align-left" class="text-[20px] opacity-50" />
        </div>
        <div class="min-w-0 flex-1" :style="actionsShown ? { maskImage: listTextMask } : undefined">
          <p class="truncate text-sm font-medium">
            {{ listTitle }}
          </p>
          <div class="mt-0.5 flex min-w-0 items-center gap-1.5">
            <Tooltip :content="clip.sourceAppName ?? 'Unknown app'">
              <AppIcon :path="clip.sourceAppPath" />
            </Tooltip>
            <Tooltip :content="copiedAt">
              <time class="shrink-0 opacity-60">{{ timeAgo }}</time>
            </Tooltip>
            <template v-if="clip.byteSize">
              <span class="opacity-40">·</span>
              <span class="shrink-0 opacity-60">{{ formatBytes(clip.byteSize) }}</span>
            </template>
            <template v-else-if="color || svgSrc || files || pageTitle">
              <span class="opacity-40">·</span>
              <span class="truncate opacity-60">
                {{ color ? color.format : svgSrc ? 'SVG' : files ? (files.length > 1 ? `${files.length} items` : 'Folder') : pageTitle }}
              </span>
            </template>
          </div>
        </div>
      </div>

      <template v-else>
        <div class="absolute inset-0 flex flex-col transition-[filter] duration-200" :class="(notice || confirm) && 'blur-sm'">
          <ClipImage
            v-if="coverImage"
            :file="coverImage"
            class="absolute inset-0 size-full object-cover"
          />
          <img
            v-else-if="fileThumbnail"
            :src="fileThumbnail"
            alt=""
            draggable="false"
            class="animate-fade-in absolute inset-0 size-full object-cover"
          >
          <div
            v-else-if="color"
            class="absolute inset-0 bg-[conic-gradient(#ccc_25%,#fff_0_50%,#ccc_0_75%,#fff_0)] bg-size-[16px_16px]"
          >
            <div class="size-full" :style="{ background: color.css }" />
          </div>
          <div v-else-if="svgSrc" class="absolute inset-0 flex items-center justify-center">
            <img :src="svgSrc" alt="" draggable="false" class="size-24 object-contain">
          </div>
          <div v-else-if="link" class="absolute inset-0 flex items-center justify-center">
            <!-- Nuxt Icon sets width and height to 1em, over Tailwind size utilities -->
            <Icon name="gravity-ui:globe" class="text-[64px] opacity-40" />
          </div>
          <div v-else-if="isFileGroup" class="absolute inset-0 flex items-center justify-center pb-10">
            <div class="relative isolate size-20">
              <div
                v-for="card in fileStack"
                :key="card.z"
                class="absolute inset-0 flex items-center justify-center overflow-hidden rounded-md bg-white shadow-md ring-1 ring-black/10 dark:bg-neutral-800 dark:ring-white/10"
                :style="{ rotate: `${card.rotate}deg`, translate: `${card.x}px 0`, zIndex: card.z }"
              >
                <img
                  v-if="card.preview?.image"
                  :src="card.preview.image"
                  alt=""
                  draggable="false"
                  class="animate-fade-in size-full"
                  :class="card.preview.thumbnail ? 'object-cover' : 'object-contain p-3'"
                >
                <Icon v-else-if="card.preview" name="gravity-ui:file-xmark" class="animate-fade-in text-[32px] opacity-40" />
                <div v-else class="skeleton size-full" />
              </div>
            </div>
          </div>
          <div v-else-if="files" class="absolute inset-0 flex items-center justify-center pb-10">
            <img v-if="filePreview?.image" :src="filePreview.image" alt="" draggable="false" class="animate-fade-in size-16 object-contain">
            <Icon v-else-if="filePreview" name="gravity-ui:file-xmark" class="animate-fade-in text-[64px] opacity-40" />
            <Icon v-else name="gravity-ui:file" class="text-[64px] opacity-15" />
          </div>
          <div v-else class="min-h-0 flex-1">
            <p class="line-clamp-5 px-3 pt-9 whitespace-pre-wrap wrap-anywhere text-sm">
              {{ clip.content }}
            </p>
          </div>

          <footer
            class="absolute inset-x-0 bottom-0 z-10 px-3 py-2"
            :class="[
              footerText ?? (hasCover && 'bg-linear-to-t from-white/90 via-white/60 to-transparent pt-8 dark:from-black/85 dark:via-black/50')
            ]"
          >
            <p v-if="title" class="mb-1 truncate font-medium text-sm">
              {{ title }}
            </p>
            <div class="flex items-center gap-1.5">
              <Tooltip :content="clip.sourceAppName ?? 'Unknown app'">
                <AppIcon :path="clip.sourceAppPath" />
              </Tooltip>
              <Tooltip :content="copiedAt">
                <time class="truncate" :class="hasCover ? 'opacity-80' : 'opacity-60'">{{ timeAgo }}</time>
              </Tooltip>
              <Tooltip v-if="clip.byteSize" :content="clip.width && clip.height ? `${clip.width} × ${clip.height}` : null">
                <span class="ml-auto shrink-0 opacity-70">
                  {{ formatBytes(clip.byteSize) }}
                </span>
              </Tooltip>
              <span v-else-if="color" class="ml-auto shrink-0 opacity-70">{{ color.format }}</span>
              <span v-else-if="svgSrc" class="ml-auto shrink-0 opacity-60">SVG</span>
              <span v-else-if="files" class="ml-auto shrink-0 opacity-60">{{ files.length > 1 ? `${files.length} items` : 'Folder' }}</span>
            </div>
          </footer>
        </div>
      </template>

      <AnimatePresence>
        <Motion
          v-if="notice"
          class="absolute inset-0 flex items-center justify-center bg-white/50 dark:bg-black/40"
          :initial="{ opacity: 0 }"
          :animate="{ opacity: 1, transition: { duration: 0.1 } }"
          :exit="{ opacity: 0, transition: { duration: 0.3 } }"
        >
          <Motion
            as="span"
            class="flex max-w-full min-w-0 items-center gap-1.5 px-3 text-sm font-semibold"
            :initial="{ scale: 0.85 }"
            :animate="{ scale: 1 }"
            :transition="{ type: 'spring', bounce: 0.4, duration: 0.35 }"
          >
            <Icon :name="notice.error ? 'gravity-ui:circle-xmark-fill' : 'gravity-ui:circle-check-fill'" class="shrink-0 text-base" />
            <span class="truncate">{{ notice.text }}</span>
          </Motion>
        </Motion>
      </AnimatePresence>
    </Motion>

    <AnimatePresence>
      <Motion
        v-if="confirm"
        data-delete-confirm
        class="absolute inset-0 z-20 flex items-center rounded-lg bg-white/50 dark:bg-black/40"
        :class="isList ? 'px-3' : 'flex-col justify-center'"
        :initial="{ opacity: 0 }"
        :animate="{ opacity: 1, transition: { duration: 0.1 } }"
        :exit="{ opacity: 0, transition: { duration: 0.15 } }"
      >
        <Motion
          class="flex items-center gap-3"
          :class="isList ? 'w-full justify-between' : 'flex-col'"
          :initial="{ scale: 0.85 }"
          :animate="{ scale: 1 }"
          :transition="{ type: 'spring', bounce: 0.4, duration: 0.35 }"
        >
          <p class="text-sm font-medium">
            {{ confirm === 'remove' ? 'Remove from this collection?' : 'Delete this clip?' }}
          </p>
          <div class="flex gap-2">
            <Button size="sm" @click="$emit('cancelDelete')">
              Cancel
            </Button>
            <Button size="sm" @click="$emit('delete')">
              {{ confirm === 'remove' ? 'Remove' : 'Delete' }}
            </Button>
          </div>
        </Motion>
      </Motion>
    </AnimatePresence>

    <div
      v-if="actionsMounted"
      v-show="!dragging && !copying && !notice && !confirm"
      class="absolute flex gap-1 peer-active:invisible"
      :class="[isList ? 'top-1/2 right-2 -translate-y-1/2' : 'top-1.5 right-1.5', !actionsShown && 'pointer-events-none']"
    >
      <Tooltip content="Details">
        <Motion
          as="button"
          type="button"
          :class="cornerButton"
          :initial="cornerHidden"
          :animate="cornerAnimation(0)"
          @click="$emit('info')"
        >
          <Icon name="gravity-ui:circle-info" class="text-sm" />
        </Motion>
      </Tooltip>
      <Tooltip :content="clip.favorite ? 'Remove from favorites' : 'Add to favorites'">
        <Motion
          as="button"
          type="button"
          :class="cornerButton"
          :initial="cornerHidden"
          :animate="cornerAnimation(1)"
          @click="$emit('favorite')"
        >
          <Icon v-if="clip.favorite" name="gravity-ui:star-fill" class="text-sm" />
          <Icon v-else name="gravity-ui:star" class="text-sm" />
        </Motion>
      </Tooltip>
      <Tooltip v-if="inCollection" content="Remove from collection">
        <Motion
          as="button"
          type="button"
          :class="cornerButton"
          :initial="cornerHidden"
          :animate="cornerAnimation(2)"
          @click="$emit('askDelete')"
        >
          <Icon name="gravity-ui:xmark" class="text-sm" />
        </Motion>
      </Tooltip>
    </div>
  </div>
</template>
