<script setup lang="ts">
import type { InertiaOptions } from 'motion-v'
import { animate } from 'motion-v'

type Card
  = | { kind: 'color', value: string }
    | { kind: 'image', src: string }
    | { kind: 'text', lines: number[] }
    | { kind: 'svg' }
    | { kind: 'files' }

const cards: Record<string, Card> = {
  wallpaper: { kind: 'image', src: '/background.jpg' },
  indigo: { kind: 'color', value: '#5B5BD6' },
  query: { kind: 'text', lines: [80, 55, 70] },
  nuxt: { kind: 'image', src: '/nuxt.jpg' },
  bolt: { kind: 'svg' },
  notes: { kind: 'text', lines: [60, 85, 45] },
  invoices: { kind: 'files' },
  mint: { kind: 'color', value: 'oklch(0.78 0.15 162)' }
}

const ids = Object.keys(cards)
const entries = ids.map(id => ({ id, card: cards[id]! }))
const COLUMNS = 4
const ROWS = Math.ceil(ids.length / COLUMNS)

const tabs = ref([
  { id: 'history', name: 'History', cards: ids },
  { id: 'snippets', name: 'Snippets', cards: ['query', 'notes'] },
  { id: 'design', name: 'Design', cards: ['indigo', 'mint', 'bolt', 'wallpaper'] },
  { id: 'research', name: 'Research', cards: ['nuxt'] }
])

const active = ref('history')
const members = computed(() => tabs.value.find(t => t.id === active.value)!.cards)

const dragging = ref<string | null>(null)

// Hidden cards keep their last slot, so they fade out where they were.
// The dragged card keeps its own until dropped, or it would jump away from the pointer.
const slots = ref<Record<string, number>>({})
watch(members, (list) => {
  const moved = list.map((id, i) => [id, i] as const).filter(([id]) => id !== dragging.value)
  slots.value = { ...slots.value, ...Object.fromEntries(moved) }
}, { immediate: true })

function placement(id: string) {
  const slot = slots.value[id] ?? 0
  const shown = members.value.includes(id)
  return {
    // Not x and y: drag measures cards without their parents' transforms, and reads % ones as px
    left: `${(slot % COLUMNS) * (100 / COLUMNS)}%`,
    top: `${Math.floor(slot / COLUMNS) * (100 / ROWS)}%`,
    opacity: shown ? 1 : 0,
    scale: shown ? 1 : 0.8
  }
}

const root = ref<HTMLElement>()
const stage = ref<HTMLElement>()
const lifted = ref<string | null>(null)
const dropTarget = ref<string | null>(null)
const flash = ref<string | null>(null)

// Touch drags would block the page scroll
const canDrag = ref(false)
onMounted(() => {
  canDrag.value = window.matchMedia('(pointer: fine)').matches
})

let liftTimer: ReturnType<typeof setTimeout> | undefined
let flashTimer: ReturnType<typeof setTimeout> | undefined
onBeforeUnmount(() => {
  clearTimeout(liftTimer)
  clearTimeout(flashTimer)
})

function tabAt(x: number, y: number) {
  for (const el of root.value?.querySelectorAll<HTMLElement>('[data-collection]') ?? []) {
    const rect = el.getBoundingClientRect()
    if (x >= rect.left && x <= rect.right && y >= rect.top - 12 && y <= rect.bottom + 12) return el.dataset.collection!
  }
  return null
}

function slotAt(x: number, y: number) {
  const rect = stage.value?.getBoundingClientRect()
  if (!rect || x < rect.left || x > rect.right || y < rect.top || y > rect.bottom) return null
  const column = Math.min(Math.floor((x - rect.left) / rect.width * COLUMNS), COLUMNS - 1)
  const row = Math.min(Math.floor((y - rect.top) / rect.height * ROWS), ROWS - 1)
  return Math.min(row * COLUMNS + column, members.value.length - 1)
}

function onDragStart(id: string) {
  clearTimeout(liftTimer)
  lifted.value = id
  dragging.value = id
}

function onDrag(event: PointerEvent, id: string) {
  dropTarget.value = tabAt(event.clientX, event.clientY)
  if (active.value === 'history' || dropTarget.value) return

  const tab = tabs.value.find(t => t.id === active.value)!
  const slot = slotAt(event.clientX, event.clientY)
  if (slot === null || tab.cards[slot] === id) return
  const cards = tab.cards.filter(card => card !== id)
  cards.splice(slot, 0, id)
  tab.cards = cards
}

function onDragEnd(id: string) {
  dragging.value = null
  slots.value = { ...slots.value, [id]: members.value.indexOf(id) }

  const target = tabs.value.find(t => t.id === dropTarget.value)
  dropTarget.value = null
  // Stays above the other cards while it springs back
  liftTimer = setTimeout(() => (lifted.value = null), 500)
  if (!target) return

  if (target.cards.includes(id)) {
    const label = root.value?.querySelector(`[data-collection="${target.id}"]`)
    if (label) animate(label, { x: [0, -4, 4, -3, 2, 0] }, { duration: 0.4 })
    return
  }

  target.cards = [id, ...target.cards]
  flash.value = target.id
  clearTimeout(flashTimer)
  flashTimer = setTimeout(() => (flash.value = null), 700)
}

const spring = { type: 'spring' as const, bounce: 0.15, visualDuration: 0.4 }
// Typed as inertia, but spread over it: dropped cards snap back on the same curve as their slot change,
// so the two cancel out instead of overshooting
const snapBack = spring as unknown as InertiaOptions
const plain = 'flex items-center justify-center bg-white ring-1 ring-black/[6%] ring-inset'
</script>

<template>
  <div ref="root" class="relative flex min-h-96 flex-col items-center justify-center gap-10 overflow-hidden rounded-2xl border border-black/6 bg-white/50 px-6 py-12 select-none">
    <div aria-hidden="true" class="absolute inset-0 bg-dots" />

    <div role="tablist" class="relative flex max-w-full gap-1 overflow-x-auto rounded-xl bg-white p-1 shadow-[0_8px_24px_-12px_rgba(24,38,84,0.3)] ring-1 ring-black/6 scrollbar-none">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        role="tab"
        :aria-selected="active === tab.id"
        :data-collection="tab.id === 'history' ? undefined : tab.id"
        class="relative flex h-8 shrink-0 items-baseline gap-1.5 rounded-lg px-3 pt-1.75 text-sm font-medium transition-[opacity,background-color,color] duration-200"
        :class="flash === tab.id ? 'bg-accent/12 text-accent' : dropTarget === tab.id ? 'bg-black/8' : active !== tab.id && 'opacity-50 hover:opacity-80'"
        @click="active = tab.id"
      >
        <Motion v-if="active === tab.id" layout-id="collections-pill" class="absolute inset-0 rounded-lg bg-black/[5.78%]" :transition="spring" />
        <span class="relative">{{ tab.name }}</span>
        <Motion
          :key="tab.cards.length"
          as="span"
          class="relative inline-block text-xs font-normal tabular-nums opacity-60"
          :initial="{ y: -6, opacity: 0 }"
          :animate="{ y: 0, opacity: 1 }"
          :transition="{ type: 'spring', bounce: 0.5, visualDuration: 0.3 }"
        >
          {{ tab.cards.length }}
        </Motion>
      </button>
    </div>

    <div ref="stage" aria-hidden="true" class="relative w-full max-w-lg">
      <!-- Sizes the stage for the full history -->
      <div class="invisible grid grid-cols-4">
        <div v-for="id in ids" :key="id" class="p-1.5">
          <div class="aspect-4/3" />
        </div>
      </div>

      <Motion
        v-for="{ id, card } in entries"
        :key="id"
        class="absolute w-1/4 p-1.5"
        :class="lifted === id ? 'z-30' : members.includes(id) ? 'z-10' : 'pointer-events-none'"
        :initial="false"
        :animate="placement(id)"
        :transition="{ ...spring, opacity: { duration: 0.2 } }"
      >
        <Motion
          :drag="canDrag && members.includes(id)"
          :drag-snap-to-origin="true"
          :drag-momentum="false"
          :drag-transition="snapBack"
          :while-drag="{ scale: 1.08, rotate: -4 }"
          class="aspect-4/3 overflow-hidden rounded-lg shadow-[0_12px_28px_-14px_rgba(24,38,84,0.4)]"
          :class="canDrag && 'cursor-grab active:cursor-grabbing'"
          @drag-start="onDragStart(id)"
          @drag="onDrag($event, id)"
          @drag-end="onDragEnd(id)"
        >
          <div v-if="card.kind === 'color'" class="size-full" :style="{ background: card.value }" />

          <img v-else-if="card.kind === 'image'" :src="card.src" alt="" draggable="false" class="size-full object-cover object-left">

          <div v-else-if="card.kind === 'text'" class="flex size-full flex-col justify-center gap-2 bg-white px-3 ring-1 ring-black/6 ring-inset">
            <span v-for="(width, j) in card.lines" :key="j" class="h-1.5 rounded-full bg-black/10" :style="{ width: `${width}%` }" />
          </div>

          <div v-else-if="card.kind === 'svg'" class="size-full" :class="[plain]">
            <svg viewBox="0 0 24 24" class="size-8 text-black/80" fill="currentColor"><path :d="BOLT" /></svg>
          </div>

          <div v-else class="size-full" :class="[plain]">
            <Icon name="vscode-icons:file-type-pdf2" class="text-3xl" />
          </div>
        </Motion>
      </Motion>
    </div>

    <p class="relative -mt-4 flex items-center gap-2 text-sm text-neutral-400">
      <Icon :name="canDrag ? 'gravity-ui:hand' : 'gravity-ui:hand-point-up'" />
      {{ !canDrag ? "Tap a tab to see what's inside" : active === 'history' ? 'Drag a card onto a tab to file it' : 'Drag cards to reorder them, or onto a tab' }}
    </p>
  </div>
</template>
