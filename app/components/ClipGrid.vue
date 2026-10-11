<script setup lang="ts">
import { animate } from 'motion-v'
import type { Clip } from '~/composables/useClips'

const props = defineProps<{
  clips: Clip[]
  /** Collection the clips belong to, `null` for the history */
  collection: number | null
  /** Bound with `@copy`: the tile shows "Copied" once the returned promise resolves */
  onCopy: (clip: Clip) => Promise<unknown>
  /** Tiles can be rearranged by dragging them over each other (a collection, not filtered) */
  reorderable?: boolean
  /** Tiles in a grid, or rows in a list */
  layout?: 'grid' | 'list'
  /** The clips aren't loaded yet: placeholder tiles stand in for them */
  loading?: boolean
}>()

// Tiles by clip id: the context menu copies through its tile, which then shows "Copied"
const cards = new Map<number, { copy: () => Promise<void> }>()
const setCard = (id: number, card: unknown) => {
  if (card) cards.set(id, card as { copy: () => Promise<void> })
  else cards.delete(id)
}

const emit = defineEmits<{
  info: [clip: Clip]
  favorite: [clip: Clip]
  delete: [clip: Clip]
  add: [clip: Clip, collection: number]
  removeFrom: [clip: Clip, collection: number]
  /** A dragged tile passes over another one: it takes its place */
  move: [clip: Clip, target: Clip]
  /** The drag that rearranged the tiles is over: the order can be saved */
  moveEnd: []
}>()

// Dragging a tile over another one of the same grid moves it there, live: the others slide
// out of the way. Leaving the window still drops it in another app
const REORDER_PAUSE_MS = 150
let lastMove = 0
let moved = false
function onTileOver(event: DragEvent, target: Clip) {
  const clip = draggedClip.value
  if (!props.reorderable || !clip || !props.clips.some(c => c.id === clip.id)) return
  event.preventDefault()
  event.stopPropagation()
  if (event.dataTransfer) event.dataTransfer.dropEffect = 'move'
  // Pause after a move: the sliding tiles pass under the pointer and would move it back
  if (target.id === clip.id || Date.now() - lastMove < REORDER_PAUSE_MS) return
  lastMove = Date.now()
  moved = true
  emit('move', clip, target)
}
watch(tileDragging, (dragging) => {
  if (!dragging && moved) {
    moved = false
    emit('moveEnd')
  }
})

// Tiles are rendered a page at a time, the next ones once scrolling gets near the end: a long history
// opens as fast as a short one. A page fills the window and a bit more, in one go: smaller pages
// would make a large window fill up in visible steps
const MIN_PAGE_SIZE = 24
const end = useTemplateRef<HTMLElement>('end')
function pageSize() {
  const grid = end.value?.parentElement
  if (!grid) return MIN_PAGE_SIZE
  const { clientWidth: width, clientHeight: height } = grid
  // Tiles: at least 200px wide, 176px tall. Rows: 56px tall. Plus their gaps, and two more rows
  const count = props.layout === 'list'
    ? Math.ceil(height / 60) + 2
    : Math.max(1, Math.floor((width - 16 + 8) / 208)) * (Math.ceil(height / 184) + 2)
  return Math.max(MIN_PAGE_SIZE, count)
}
const renderedCount = ref(MIN_PAGE_SIZE)
const renderedClips = computed(() => props.clips.slice(0, renderedCount.value))

// Switching between grid and list rebuilds the tiles from the first page, from the top: moving
// hundreds of tiles to their new place at once would hold the window. The rest follows on scroll.
// The whole grid fades in and rises a little instead, once built: one animation, whatever the
// number of tiles
watch(() => props.layout, async () => {
  const grid = end.value?.parentElement
  grid?.scrollTo({ top: 0 })
  renderedCount.value = pageSize()
  await nextTick()
  if (grid) animate(grid, { opacity: [0, 1], y: [12, 0] }, { type: 'spring', bounce: 0, visualDuration: 0.25 })
  checkEnd()
})

// Only clips copied while the grid is shown pop in: tiles added by scrolling, or by a whole new
// grid, just appear
const shownSince = Date.now()
let observer: IntersectionObserver | undefined

// The observer only reports changes: observing again makes it report whether the end is
// in view right now, e.g. on a tall window that still shows it after a new page
async function checkEnd() {
  await nextTick()
  if (!observer || !end.value) return
  observer.unobserve(end.value)
  observer.observe(end.value)
}

onMounted(() => {
  observer = new IntersectionObserver(([entry]) => {
    if (!entry?.isIntersecting || renderedCount.value >= props.clips.length) return
    renderedCount.value += pageSize()
    checkEnd()
  }, { root: end.value?.parentElement, rootMargin: '400px' })
  renderedCount.value = pageSize()
  if (end.value) observer.observe(end.value)
})
onBeforeUnmount(() => observer?.disconnect())

// The clips arrive after the grid is shown (history loading, search results…): the end may
// already be in view with fewer tiles than the window holds
watch(() => props.clips.length, checkEnd)

// Quick and barely bouncy, like the Windows 11 animations
const tileTransition = { type: 'spring', bounce: 0.15, duration: 0.45 } as const
// A removed tile shrinks and fades in place, quickly, before the others slide over its spot
const tileExit = { opacity: 0, scale: 0.85, transition: { duration: 0.2, ease: 'easeIn' as const } }

// Deleting a clip, or taking it out of the open collection, waits for a confirmation on its tile.
// One tile asks at a time
const pending = ref<{ clip: number, action: 'delete' | 'remove' } | null>(null)
const askDelete = (clip: Clip) => (pending.value = { clip: clip.id, action: props.collection === null ? 'delete' : 'remove' })
function confirmDelete(clip: Clip) {
  const action = pending.value?.action
  pending.value = null
  if (action === 'delete') emit('delete', clip)
  else if (action === 'remove' && props.collection !== null) emit('removeFrom', clip, props.collection)
}
// Like a modal: Escape or a click anywhere outside the confirmation cancels it
const cancelOnEscape = (event: KeyboardEvent) => {
  if (event.key === 'Escape') pending.value = null
}
const cancelOnOutsideClick = (event: PointerEvent) => {
  if (!(event.target as Element | null)?.closest('[data-delete-confirm]')) pending.value = null
}
function stopListening() {
  window.removeEventListener('keydown', cancelOnEscape)
  window.removeEventListener('pointerdown', cancelOnOutsideClick, true)
}
watch(pending, (asking) => {
  stopListening()
  if (!asking) return
  window.addEventListener('keydown', cancelOnEscape)
  // Capture: runs even if what is clicked stops the event
  window.addEventListener('pointerdown', cancelOnOutsideClick, true)
})
onBeforeUnmount(stopListening)
// From the context menu: taking it out of another collection (unticking it) doesn't need asking
function onRemoveFrom(clip: Clip, collection: number) {
  if (collection === props.collection) pending.value = { clip: clip.id, action: 'remove' }
  else emit('removeFrom', clip, collection)
}
</script>

<template>
  <!-- Fills the tab area, where it swipes in and out (motion props come from the parent).
       layout-scroll: tiles keep animating to the right place even while the list is scrolled -->
  <Motion
    layout-scroll
    class="absolute inset-0 grid auto-rows-min overflow-y-auto p-2 font-ui text-xs"
    :class="layout === 'list' ? 'grid-cols-1 gap-1' : 'grid-cols-[repeat(auto-fill,minmax(200px,1fr))] gap-2'"
  >
    <!-- LayoutGroup: once a removed tile is gone, the others are told to slide into its place -->
    <!-- Keyed by the layout: a switch rebuilds the tiles instead of animating each one (see above) -->
    <LayoutGroup :key="layout">
      <!-- A removed tile keeps its spot while it fades out, then the next ones slide over it.
           initial false: a grid shown as a whole appears at once, not tile by tile -->
      <AnimatePresence :initial="false">
        <!-- A tile copied again moves to the top: it slides there and the others make room -->
        <Motion
          v-for="clip in renderedClips"
          :key="clip.id"
          layout="position"
          :initial="clip.createdAt > shownSince ? { opacity: 0, scale: 0.95 } : false"
          :animate="{ opacity: 1, scale: 1 }"
          :exit="tileExit"
          :transition="tileTransition"
          @dragover="onTileOver($event, clip)"
        >
          <!-- Right-click: copy, add to a collection, remove or delete -->
          <ClipContextMenu
            :clip="clip"
            :collection="collection"
            @copy="cards.get(clip.id)?.copy()"
            @info="emit('info', clip)"
            @favorite="emit('favorite', clip)"
            @delete="pending = { clip: clip.id, action: 'delete' }"
            @add="emit('add', clip, $event)"
            @remove-from="onRemoveFrom(clip, $event)"
          >
            <!-- In a collection, the delete button only takes the clip out of it. Both ask first -->
            <ClipCard
              :ref="card => setCard(clip.id, card)"
              :clip="clip"
              :in-collection="collection !== null"
              :layout="layout"
              class="w-full"
              @copy="onCopy(clip)"
              @info="emit('info', clip)"
              @favorite="emit('favorite', clip)"
              :confirm="pending?.clip === clip.id ? pending.action : null"
              @ask-delete="askDelete(clip)"
              @delete="confirmDelete(clip)"
              @cancel-delete="pending = null"
            />
          </ClipContextMenu>
        </Motion>
      </AnimatePresence>
    </LayoutGroup>

    <!-- Placeholder tiles while the clips load, shaped like the real ones: title and footer -->
    <template v-if="loading && !clips.length">
      <div
        v-for="i in (layout === 'list' ? 10 : 12)"
        :key="i"
        class="flex rounded-lg border border-black/[5.78%] bg-black/[3.73%] dark:border-white/6 dark:bg-white/[6.05%]"
        :class="layout === 'list' ? 'h-14 items-center gap-3 pr-3 pl-2' : 'h-44 flex-col justify-end p-3'"
      >
        <div v-if="layout === 'list'" class="skeleton size-10 shrink-0 rounded-md" />
        <div class="flex min-w-0 flex-1 flex-col gap-2" :class="layout !== 'list' && 'justify-end'">
          <!-- Titles of varied lengths, like real ones -->
          <div class="skeleton h-3.5 rounded-sm" :style="{ width: `${45 + (i * 37) % 40}%` }" />
          <div class="flex items-center gap-1.5">
            <div class="skeleton size-4 rounded-sm" />
            <div class="skeleton h-2.5 w-12 rounded-sm" />
          </div>
        </div>
      </div>
    </template>

    <!-- Reaching it renders the next page of tiles -->
    <div ref="end" class="col-span-full h-px" />

    <!-- Empty states -->
    <slot />
  </Motion>
</template>
