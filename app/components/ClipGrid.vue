<script setup lang="ts">
import type { Clip, PartialCopy } from '@/composables/useClips'
import { invoke } from '@tauri-apps/api/core'
import { animate } from 'motion-v'

const props = defineProps<{
  clips: Clip[]
  collection: number | null
  onCopy: (clip: Clip) => Promise<PartialCopy | null>
  reorderable?: boolean
  layout?: 'grid' | 'list'
  loading?: boolean
}>()

const emit = defineEmits<{
  info: [clip: Clip]
  favorite: [clip: Clip]
  delete: [clip: Clip]
  add: [clip: Clip, collection: number]
  removeFrom: [clip: Clip, collection: number]
  move: [clip: Clip, target: Clip]
  moveEnd: []
}>()

const container = useTemplateRef<HTMLElement>('container')
function columns(tiles: HTMLElement[]) {
  if (props.layout === 'list') return 1
  const top = (tile?: HTMLElement) => Math.round(tile?.getBoundingClientRect().top ?? 0)
  const count = tiles.findIndex(tile => top(tile) !== top(tiles[0]))
  return count === -1 ? tiles.length : count
}
function onKeydown(event: KeyboardEvent) {
  const tiles = [...(container.value?.querySelectorAll<HTMLElement>('[data-tile]') ?? [])]
  const index = tiles.indexOf(event.target as HTMLElement)
  if (index === -1 || event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return
  const step = columns(tiles)
  const last = tiles.length - 1
  const targets: Record<string, number> = {
    ArrowLeft: index - 1,
    ArrowRight: index + 1,
    ArrowUp: index - step,
    // From the row above the last one, even without a tile right below
    ArrowDown: Math.floor(index / step) < Math.floor(last / step) ? Math.min(last, index + step) : index
  }
  const target = targets[event.key]
  if (target === undefined) return
  event.preventDefault()
  const tile = tiles[Math.min(last, Math.max(0, target))]
  tile?.focus({ preventScroll: true })
  tile?.scrollIntoView({ block: 'nearest' })
}

const cards = new Map<number, { copy: () => Promise<void> }>()
function setCard(id: number, card: unknown) {
  if (card) cards.set(id, card as { copy: () => Promise<void> })
  else cards.delete(id)
}

async function paste(clip: Clip) {
  await invoke('paste_clip', { id: clip.id })
    .catch(error => notifyClip(clip.id, typeof error === 'string' ? error : 'Couldn’t paste'))
}

async function copyImageText(clip: Clip) {
  await invoke('copy_image_text', { id: clip.id })
    .then(() => notifyClip(clip.id, 'Copied text'))
    .catch(error => notifyClip(clip.id, typeof error === 'string' ? error : 'Couldn’t copy'))
}

async function saveAs(clip: Clip) {
  await invoke<{ saved: number, total: number } | null>('save_clip_as', { id: clip.id })
    .then((saved) => {
      if (saved) notifyClip(clip.id, saved.saved < saved.total ? `Saved (${saved.saved}/${saved.total})` : 'Saved')
    })
    .catch(error => notifyClip(clip.id, typeof error === 'string' ? error : 'Couldn’t save'))
}

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

const MIN_PAGE_SIZE = 24
const end = useTemplateRef<HTMLElement>('end')
const scroller = () => end.value?.closest<HTMLElement>('[data-reka-scroll-area-viewport]') ?? null
function pageSize() {
  const grid = end.value?.parentElement
  const height = scroller()?.clientHeight
  if (!grid || height === undefined) return MIN_PAGE_SIZE
  const width = grid.clientWidth
  const count = props.layout === 'list'
    ? Math.ceil(height / 60) + 2
    : Math.max(1, Math.floor((width - 16 + 8) / 208)) * (Math.ceil(height / 184) + 2)
  return Math.max(MIN_PAGE_SIZE, count)
}
const renderedCount = ref(MIN_PAGE_SIZE)
const renderedClips = computed(() => props.clips.slice(0, renderedCount.value))

watch(() => props.layout, async () => {
  const grid = end.value?.parentElement
  scroller()?.scrollTo({ top: 0 })
  renderedCount.value = pageSize()
  await nextTick()
  if (grid && !props.loading) animate(grid, { opacity: [0, 1], y: [12, 0] }, { type: 'spring', bounce: 0, visualDuration: 0.25 })
  checkEnd()
})

const shownSince = Date.now()
let observer: IntersectionObserver | undefined

// The observer only reports changes: observing again reports whether the end is in view now
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
  }, { root: scroller(), rootMargin: '400px' })
  renderedCount.value = pageSize()
  if (end.value) observer.observe(end.value)
})
onBeforeUnmount(() => observer?.disconnect())

watch(() => props.clips.length, checkEnd)

const tileTransition = { type: 'spring', bounce: 0.15, duration: 0.45 } as const
const tileExit = { opacity: 0, scale: 0.85, transition: { duration: 0.2, ease: 'easeIn' as const } }

const pending = ref<{ clip: number, action: 'delete' | 'remove' } | null>(null)
const askDelete = (clip: Clip) => (pending.value = { clip: clip.id, action: props.collection === null ? 'delete' : 'remove' })
function confirmDelete(clip: Clip) {
  const action = pending.value?.action
  pending.value = null
  if (action === 'delete') emit('delete', clip)
  else if (action === 'remove' && props.collection !== null) emit('removeFrom', clip, props.collection)
}
const asking = () => (pending.value ? window : null)
onKeyStroke('Escape', (event) => {
  event.preventDefault()
  pending.value = null
}, { target: asking })
useEventListener(asking, 'pointerdown', (event: PointerEvent) => {
  if (!(event.target as Element | null)?.closest('[data-delete-confirm]')) pending.value = null
}, { capture: true })
function onRemoveFrom(clip: Clip, collection: number) {
  if (collection === props.collection) pending.value = { clip: clip.id, action: 'remove' }
  else emit('removeFrom', clip, collection)
}
</script>

<template>
  <Motion class="absolute inset-0">
    <ScrollArea layout-scroll class="size-full">
      <div
        ref="container"
        class="relative grid auto-rows-min p-2 text-xs"
        :class="layout === 'list' ? 'grid-cols-1 gap-1' : 'grid-cols-[repeat(auto-fill,minmax(200px,1fr))] gap-2'"
        @keydown="onKeydown"
      >
        <LayoutGroup :key="layout">
          <AnimatePresence :initial="false">
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
              <ClipContextMenu
                :clip="clip"
                :collection="collection"
                @copy="cards.get(clip.id)?.copy()"
                @paste="paste(clip)"
                @copy-text="copyImageText(clip)"
                @save="saveAs(clip)"
                @info="emit('info', clip)"
                @favorite="emit('favorite', clip)"
                @delete="pending = { clip: clip.id, action: 'delete' }"
                @add="emit('add', clip, $event)"
                @remove-from="onRemoveFrom(clip, $event)"
              >
                <ClipCard
                  :ref="card => setCard(clip.id, card)"
                  :clip="clip"
                  :in-collection="collection !== null"
                  :layout="layout"
                  class="w-full"
                  :confirm="pending?.clip === clip.id ? pending.action : null"
                  @copy="onCopy(clip)"
                  @info="emit('info', clip)"
                  @favorite="emit('favorite', clip)"
                  @ask-delete="askDelete(clip)"
                  @delete="confirmDelete(clip)"
                  @cancel-delete="pending = null"
                />
              </ClipContextMenu>
            </Motion>
          </AnimatePresence>
        </LayoutGroup>

        <template v-if="loading && !clips.length">
          <div
            v-for="i in (layout === 'list' ? 10 : 12)"
            :key="i"
            class="flex rounded-lg border border-black/[5.78%] bg-black/[3.73%] dark:border-white/6 dark:bg-white/[6.05%]"
            :class="layout === 'list' ? 'h-14 items-center gap-3 pr-3 pl-2' : 'h-44 flex-col justify-end p-3'"
          >
            <div v-if="layout === 'list'" class="skeleton size-10 shrink-0 rounded-md" />
            <div class="flex min-w-0 flex-1 flex-col gap-2" :class="layout !== 'list' && 'justify-end'">
              <div class="skeleton h-3.5 rounded-sm" :style="{ width: `${45 + (i * 37) % 40}%` }" />
              <div class="flex items-center gap-1.5">
                <div class="skeleton size-4 rounded-sm" />
                <div class="skeleton h-2.5 w-12 rounded-sm" />
              </div>
            </div>
          </div>
        </template>

        <div ref="end" class="col-span-full h-px" />

        <slot />
      </div>
    </ScrollArea>
  </Motion>
</template>
