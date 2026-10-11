<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import type { Clip } from '~/composables/useClips'

const props = defineProps<{
  clip: Clip
  /** Bound with `@copy`: copies the clip, the tile shows "Copied" once it succeeds */
  onCopy?: () => Promise<unknown>
  /** Shown in a collection: the delete button takes the clip out of it instead */
  inCollection?: boolean
  /** Asking to confirm a deletion ("delete") or a removal from the open collection ("remove") */
  confirm?: 'delete' | 'remove' | null
  /** "grid": a tall tile with a large preview. "list": a compact row with a small thumbnail */
  layout?: 'grid' | 'list'
}>()

defineEmits<{
  favorite: []
  /** The info button was clicked: details of the clip */
  info: []
  /** The delete button was clicked: confirmation asked */
  askDelete: []
  /** The deletion (or removal) was confirmed */
  delete: []
  cancelDelete: []
}>()

// Buttons over the top right corner of the tile, readable over pictures too
const cornerButton = 'flex size-7 cursor-pointer items-center justify-center rounded-md bg-white/80 backdrop-blur-md transition-[background-color] duration-100 hover:bg-neutral-200/90 active:bg-neutral-300/90 focus-visible:outline-2 focus-visible:outline-current dark:bg-black/60 dark:hover:bg-neutral-700/80 dark:active:bg-neutral-800/80'

// The corner buttons drop in from above with a little bounce, one after the other, when the tile
// is hovered (or focused with the keyboard, not by a click). They leave quickly, both at once
const hovered = ref(false)
const focused = ref(false)
const actionsShown = computed(() => hovered.value || focused.value)
const cornerHidden = { opacity: 0, y: -10, scale: 0.85 }
function cornerAnimation(index: number) {
  return actionsShown.value
    ? { opacity: 1, y: 0, scale: 1, transition: { type: 'spring' as const, bounce: 0.45, duration: 0.4, delay: index * 0.05 } }
    : { ...cornerHidden, transition: { duration: 0.12 } }
}
// The buttons only exist while needed: built when the tile is hovered, removed once they have
// left. A grid of resting tiles holds none of them
const actionsMounted = ref(false)
let unmountActions: ReturnType<typeof setTimeout> | undefined
watch(actionsShown, (shown) => {
  clearTimeout(unmountActions)
  if (shown) actionsMounted.value = true
  else unmountActions = setTimeout(() => (actionsMounted.value = false), 150)
})
onBeforeUnmount(() => clearTimeout(unmountActions))

/** Message shown over the tile for a moment: "Copied", or why it couldn't be copied or dragged */
const notice = ref<{ text: string, error: boolean } | null>(null)
/** From the click until "Copied" shows: the delete button stays hidden in between */
const copying = ref(false)
let noticeTimer: ReturnType<typeof setTimeout> | undefined

function showNotice(text: string, error = false) {
  // Showing it again restarts the message instead of hiding it early
  clearTimeout(noticeTimer)
  notice.value = { text, error }
  noticeTimer = setTimeout(() => (notice.value = null), error ? 1800 : 1200)
}

// Copy and drag fail with a message ready to show (the copied files no longer exist…)
const errorText = (error: unknown) => (typeof error === 'string' ? error : 'Couldn’t copy')

async function copy() {
  copying.value = true
  try {
    await props.onCopy?.()
    showNotice('Copied')
  } catch (error) {
    showNotice(errorText(error), true)
  } finally {
    copying.value = false
  }
}

onBeforeUnmount(() => clearTimeout(noticeTimer))

// The context menu copies through the tile too, so it shows "Copied" the same way
defineExpose({ copy })

// Dragging a tile drops its content in another app: text through the webview,
// images and files through a native drag, the only way to hand files to other apps
// The tile tilts while it is being dragged, to show the drag was picked up
const dragging = ref(false)
/** Quick and without bounce, for the tilt and the press */
const tileSpring = { type: 'spring', bounce: 0, duration: 0.2 } as const
const stopDragging = () => (dragging.value = false)

// Shrinks a little while pressed. Followed by hand rather than with Motion's `whilePress`: once
// a drag starts, the button never gets its `pointerup`, and `whilePress` would stay shrunk after it
const pressed = ref(false)
const release = () => {
  pressed.value = false
  window.removeEventListener('pointerup', release)
  window.removeEventListener('pointercancel', release)
}
function press(event: PointerEvent) {
  if (event.button !== 0) return
  pressed.value = true
  window.addEventListener('pointerup', release)
  window.addEventListener('pointercancel', release)
}
onBeforeUnmount(release)

watch(dragging, (isDragging) => {
  // Lets the window accept the drag while it passes over it
  tileDragging.value = isDragging
  if (!isDragging) draggedClip.value = null
  if (!isDragging) window.removeEventListener('pointermove', stopDragging)
})
onBeforeUnmount(() => window.removeEventListener('pointermove', stopDragging))

// Image under the pointer while dragging text: a compact label with the start of the text (and the
// color of a color). Left to itself, the webview snapshots the tile wrongly, the whole page moving along
const dragLabelClass = 'fixed -top-96 left-0 flex max-w-64 items-center gap-2 rounded-lg border border-black/[5.78%] bg-[#f9f9f9] px-3 py-2 font-ui text-sm shadow-lg dark:border-black/20 dark:bg-[#2c2c2c]'
function setDragLabel(dataTransfer: DataTransfer) {
  const label = document.createElement('div')
  label.className = dragLabelClass
  if (color.value) {
    const swatch = document.createElement('span')
    swatch.className = 'size-4 shrink-0 rounded-sm border border-black/10'
    swatch.style.background = color.value.css
    label.append(swatch)
  }
  const text = document.createElement('span')
  text.className = 'truncate'
  text.textContent = listTitle.value
  label.append(text)
  // Snapshotted by the webview right away, then no longer needed
  document.body.append(label)
  dataTransfer.setDragImage(label, 12, 12)
  requestAnimationFrame(() => label.remove())
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
    if (event.dataTransfer) setDragLabel(event.dataTransfer)
    // `dragend` doesn't always reach the page once dropped in another app. The page gets
    // no mouse moves during a webview drag, so the first one back means the drag is over.
    // Next frame: skips a move that might still come along with the drag start
    requestAnimationFrame(() => window.addEventListener('pointermove', stopDragging, { once: true }))
    return
  }
  // The webview drag is cancelled for a native one, which hands real files to other apps.
  // Mouse moves reach the page again once the webview drag is cancelled: they can't tell when it ends
  event.preventDefault()
  // Resolves once the native drag is dropped or cancelled
  await invoke('start_drag', { id: props.clip.id }).catch(error => showNotice(errorText(error), true))
  dragging.value = false
}

const timeAgo = useTimeAgo(() => props.clip.createdAt)
const isImage = computed(() => props.clip.kind === 'image')

// Copied SVG code is previewed as an image, in the text color of the current theme
const prefersDark = usePrefersDark()
const svgSrc = computed(() =>
  props.clip.kind === 'text'
    ? svgToDataUrl(props.clip.content, prefersDark.value ? '#fff' : '#000')
    : null
)

// Copied colors (hex, rgb(), hsl(), oklch()…) fill the tile
const color = computed(() => (props.clip.kind === 'text' ? copiedColor(props.clip.content) : null))

// Copied links show the website preview once loaded, the plain link until then
const link = computed(() => (props.clip.kind === 'text' ? copiedLink(props.clip.content) : null))
// Loaded only once the card is on screen, not for the whole history at once
const root = useTemplateRef<HTMLElement>('root')
const visible = useVisible(root)
const linkPreview = useLinkPreview(link, visible)
const preview = computed(() =>
  linkPreview.value && (linkPreview.value.title || linkPreview.value.image) ? linkPreview.value : null
)

// A copied file shows as the Explorer does: its thumbnail, or else its icon.
// Several files show as a fanned stack of the first ones
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

// Position of each card in the stack, the first file on top
const STACK_LAYOUTS: Record<number, { rotate: number, x: number }[]> = {
  2: [{ rotate: 5, x: 10 }, { rotate: -5, x: -10 }],
  3: [{ rotate: 0, x: 0 }, { rotate: -9, x: -22 }, { rotate: 9, x: 22 }]
}
const fileStack = computed(() => {
  if (!files.value || !isFileGroup.value) return []
  const layout = STACK_LAYOUTS[Math.min(files.value.length, 3)]!
  return layout.map((position, i) => ({ ...position, preview: filePreviews[i]!.value, z: layout.length - i }))
})

// Image shown across the whole tile: a copied image, or the image of a link preview
const coverImage = computed(() => (isImage.value ? `${props.clip.content}.png` : preview.value?.image ?? null))
// Pictures or colors behind the footer, which then needs a gradient to stay readable
const hasCover = computed(() => coverImage.value !== null || fileThumbnail.value !== null || color.value !== null)
const title = computed(() => preview.value?.title ?? fileTitle.value ?? color.value?.value)
// List rows show a single line of text for every clip
const isList = computed(() => props.layout === 'list')
const listTitle = computed(() => {
  if (title.value) return title.value
  if (isImage.value) return props.clip.width && props.clip.height ? `Image ${props.clip.width} × ${props.clip.height}` : 'Image'
  return link.value ?? props.clip.content.trim().replace(/\s+/g, ' ')
})
// Space the corner buttons take over the end of a row: info and star, plus the cross in a collection
const listTextMask = computed(() => {
  const covered = props.inCollection ? 96 : 64
  return `linear-gradient(to left, transparent ${covered}px, black ${covered + 32}px)`
})
const copiedAt = computed(() => new Date(props.clip.createdAt).toLocaleString('en-US', { dateStyle: 'medium', timeStyle: 'short' }))
</script>

<template>
  <!-- Wrapper: the delete button can't sit inside the tile button -->
  <div
    ref="root"
    class="group relative"
    @pointerenter="hovered = true"
    @pointerleave="hovered = false"
    @focusin="focused = ($event.target as Element).matches(':focus-visible')"
    @focusout="focused = false"
  >
    <!-- Tilts while dragged (a tile, not a list row) and shrinks a little while pressed, with Motion springs that can turn
         around mid-way. Hover colors stay in CSS -->
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
      <!-- List row: small thumbnail, then the title and the source app, time and size on one line each.
           Blurred under "Copied" like the tile -->
      <div
        v-if="isList"
        class="absolute inset-0 flex items-center gap-3 pr-3 pl-2 transition-[filter] duration-200"
        :class="(notice || confirm) && 'blur-sm'"
      >
        <div class="relative flex size-10 shrink-0 items-center justify-center overflow-hidden rounded-md bg-black/[3.73%] dark:bg-white/[6.05%]">
          <ClipImage v-if="coverImage" :file="coverImage" class="size-full object-cover" />
          <img v-else-if="fileThumbnail" :src="fileThumbnail" alt="" draggable="false" class="size-full object-cover">
          <div
            v-else-if="color"
            class="size-full bg-[conic-gradient(#ccc_25%,#fff_0_50%,#ccc_0_75%,#fff_0)] bg-size-[8px_8px]"
          >
            <div class="size-full" :style="{ background: color.css }" />
          </div>
          <img v-else-if="svgSrc" :src="svgSrc" alt="" draggable="false" class="size-6 object-contain">
          <Icon v-else-if="link" name="gravity-ui:globe" class="text-[20px] opacity-50" />
          <img v-else-if="filePreview?.image" :src="filePreview.image" alt="" draggable="false" class="size-7 object-contain">
          <Icon v-else-if="files" name="gravity-ui:file-xmark" class="text-[20px] opacity-50" />
          <Icon v-else name="gravity-ui:text-align-left" class="text-[20px] opacity-50" />
        </div>
        <!-- Fades out before the corner buttons while they show, instead of running under them -->
        <div class="min-w-0 flex-1" :style="actionsShown ? { maskImage: listTextMask } : undefined">
          <Tooltip :content="files?.map(fileName).join('\n') ?? listTitle">
            <p class="truncate text-sm font-medium">
              {{ listTitle }}
            </p>
          </Tooltip>
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
            <template v-else-if="color || svgSrc || files || link">
              <span class="opacity-40">·</span>
              <span class="truncate opacity-60">
                {{ color ? color.format : svgSrc ? 'SVG' : files ? (files.length > 1 ? `${files.length} items` : 'Folder') : linkDomain(link!) }}
              </span>
            </template>
          </div>
        </div>
      </div>

      <template v-else>
        <!-- Tile content (the footer aside), blurred under "Copied". Blurred itself rather than through
             a backdrop filter on the message, which didn't reach plain text -->
        <div class="absolute inset-0 flex flex-col transition-[filter] duration-200" :class="(notice || confirm) && 'blur-sm'">
          <!-- Images and SVGs fill the whole tile, the footer is drawn over them -->
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
            class="absolute inset-0 size-full object-cover"
          >
          <!-- Color over a checkerboard, which shows through semi-transparent colors -->
          <div
            v-else-if="color"
            class="absolute inset-0 bg-[conic-gradient(#ccc_25%,#fff_0_50%,#ccc_0_75%,#fff_0)] bg-size-[16px_16px]"
          >
            <div class="size-full" :style="{ background: color.css }" />
          </div>
          <div v-else-if="svgSrc" class="absolute inset-0 flex items-center justify-center">
            <!-- Same size for every SVG, whatever its own dimensions -->
            <img :src="svgSrc" alt="" draggable="false" class="size-24 object-contain">
          </div>
          <!-- Link without preview image: same layout, with an icon in place of the image -->
          <div v-else-if="link" class="absolute inset-0 flex items-center justify-center">
            <!-- Sized with the font size: Nuxt Icon sets width and height to 1em, over Tailwind size utilities -->
            <Icon name="gravity-ui:globe" class="text-[64px] opacity-40" />
          </div>
          <!-- Several files: the first ones as cards fanned out, each with its thumbnail or icon -->
          <div v-else-if="isFileGroup" class="absolute inset-0 flex items-center justify-center pb-10">
            <!-- isolate: the cards' z-index only orders them, they stay below the footer and "Copied" -->
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
                  class="size-full"
                  :class="card.preview.thumbnail ? 'object-cover' : 'object-contain p-3'"
                >
                <Icon v-else-if="card.preview" name="gravity-ui:file-xmark" class="text-[32px] opacity-40" />
              </div>
            </div>
          </div>
          <!-- File without thumbnail (document, archive, folder…): its icon, or a cross once it was moved or deleted -->
          <div v-else-if="files" class="absolute inset-0 flex items-center justify-center">
            <img v-if="filePreview?.image" :src="filePreview.image" alt="" draggable="false" class="size-16 object-contain">
            <Icon v-else-if="filePreview" name="gravity-ui:file-xmark" class="text-[64px] opacity-40" />
          </div>
          <!-- Plain text starts below the delete button, which never covers it -->
          <div v-else class="min-h-0 flex-1">
            <p class="line-clamp-5 px-3 pt-9 whitespace-pre-wrap wrap-anywhere text-sm">
              {{ clip.content }}
            </p>
          </div>
        </div>

        <!-- Source app and copy time, pinned to the bottom. Over images, a gradient (white or black,
             following the theme) keeps it readable. Kept sharp and above "Copied" -->
        <footer
          class="absolute inset-x-0 bottom-0 z-10 px-3 py-2"
          :class="[
            hasCover && 'bg-linear-to-t from-white/90 via-white/60 to-transparent pt-8 dark:from-black/85 dark:via-black/50',
          ]"
        >
          <!-- One line, cut with an ellipsis: the full title shows on hover -->
          <Tooltip v-if="title" :content="files?.map(fileName).join('\n') ?? title">
            <p class="mb-1 truncate font-medium text-sm">
              {{ title }}
            </p>
          </Tooltip>
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
            <!-- Only folders: no size to show -->
            <span v-else-if="files" class="ml-auto shrink-0 opacity-60">{{ files.length > 1 ? `${files.length} items` : 'Folder' }}</span>
            <Tooltip v-else-if="link" :content="link">
              <span class="ml-auto truncate opacity-70">{{ linkDomain(link) }}</span>
            </Tooltip>
          </div>
        </footer>
      </template>

      <!-- Confirmation over the whole tile: fades in quickly with a small pop, fades out on its own -->
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
            class="flex items-center gap-1.5 text-sm font-semibold"
            :initial="{ scale: 0.85 }"
            :animate="{ scale: 1 }"
            :transition="{ type: 'spring', bounce: 0.4, duration: 0.35 }"
          >
            <Icon :name="notice.error ? 'gravity-ui:circle-xmark-fill' : 'gravity-ui:circle-check-fill'" class="shrink-0 text-base" />
            {{ notice.text }}
          </Motion>
        </Motion>
      </AnimatePresence>
    </Motion>

    <!-- Asks before deleting, over the blurred tile like "Copied". Outside the tile button:
         a button can't hold other buttons -->
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
        <!-- On a list row, the question and the buttons share the line -->
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

    <!-- Over the top right corner of the tile, shown on hover. Hidden while the tile is pressed, dragged or shows "Copied".
         Not clickable while hidden -->
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
      <!-- Filled star for a favorite -->
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
      <!-- In a collection only: deleting from the history is a rare action, kept in the context menu -->
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
