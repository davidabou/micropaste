<script setup lang="ts">
const themes = [
  {
    dark: false,
    window: 'border-black/10 bg-[#f3f3f3]/80 text-black/90',
    card: 'border-black/[5.78%] bg-[#f9f9f9]',
    selected: 'bg-black/[5.78%]',
    line: 'bg-black/10'
  },
  {
    dark: true,
    window: 'border-white/10 bg-[#202020]/80 text-white/90',
    card: 'border-white/[6%] bg-[#2b2b2b]',
    selected: 'bg-white/[6%]',
    line: 'bg-white/15'
  }
]

const cards = [
  { kind: 'image', src: '/nuxt.jpg', app: 'logos:chrome' },
  { kind: 'color', value: '#5B5BD6', app: 'logos:figma' },
  { kind: 'text', lines: [80, 55, 70], app: 'logos:visual-studio-code' },
  { kind: 'svg', app: 'logos:figma' },
  { kind: 'files', icon: 'vscode-icons:file-type-pdf2', app: 'flat-color-icons:folder' },
  { kind: 'image', src: '/background.jpg', app: 'logos:microsoft-edge' }
] as const

// Where the diagonal crosses the top edge, in %; it leans 20% over the height
const REST = 62
const SMOOTHING = 90 // ms, lower follows the pointer more tightly

const root = ref<HTMLElement>()
const line = ref<SVGLineElement>()
const darkShown = ref(false)
const lightShown = ref(true)

let current = 125
let goal = 125
let frame = 0
let last = 0

// Eases towards the goal every frame, so a new goal never restarts the motion
function tick(now: number) {
  const dt = Math.min(now - last, 64)
  last = now
  current += (goal - current) * (1 - Math.exp(-dt / SMOOTHING))
  if (Math.abs(goal - current) < 0.01) current = goal

  // Straight to the DOM: a Vue render per frame would stutter
  root.value?.style.setProperty('--split', String(current))
  line.value?.setAttribute('x1', String(current))
  line.value?.setAttribute('x2', String(current - 20))
  darkShown.value = current < 100
  lightShown.value = current > 20

  frame = current === goal ? 0 : requestAnimationFrame(tick)
}

function moveTo(value: number) {
  goal = value
  if (frame) return
  last = performance.now()
  frame = requestAnimationFrame(tick)
}

onBeforeUnmount(() => cancelAnimationFrame(frame))

const inView = useInView(root, { once: true, amount: 0.5 })
watch(inView, seen => seen && moveTo(REST))

// Follows the pointer, at its height
function onPointerMove(event: PointerEvent) {
  if (event.pointerType !== 'mouse') return
  const rect = root.value!.getBoundingClientRect()
  const x = ((event.clientX - rect.left) / rect.width) * 100
  const y = (event.clientY - rect.top) / rect.height
  moveTo(Math.min(Math.max(x + 20 * y, 0), 120))
}

const darkHalf = 'polygon(calc(var(--split) * 1%) 0, 100% 0, 100% 100%, calc((var(--split) - 20) * 1%) 100%)'
</script>

<template>
  <div
    ref="root"
    aria-hidden="true"
    class="relative h-full min-h-96 overflow-hidden rounded-2xl border border-black/6 select-none [--split:125]"
    @pointermove="onPointerMove"
    @pointerleave="moveTo(REST)"
  >
    <img src="/background.jpg" alt="" draggable="false" class="absolute inset-0 size-full scale-110 object-cover blur-2xl">

    <div
      v-for="theme in themes"
      :key="String(theme.dark)"
      class="absolute inset-0 flex items-center justify-center p-8"
      :class="theme.dark && 'bg-black/40'"
      :style="theme.dark ? { clipPath: darkHalf } : undefined"
    >
      <div class="flex w-full max-w-md flex-col overflow-hidden rounded-lg border shadow-[0_32px_64px_rgb(0_0_0/0.18),0_2px_21px_rgb(0_0_0/0.14)]" :class="theme.window">
        <header class="flex h-8 items-center justify-between">
          <div class="flex items-center gap-2 pl-3">
            <Icon name="gravity-ui:magnifier" class="text-sm opacity-60" />
            <span class="text-xs opacity-50">Search…</span>
          </div>
          <CaptionButtons small />
        </header>

        <div class="flex gap-1 px-2 pb-1 text-xs font-medium">
          <span class="rounded-md px-2.5 py-1" :class="theme.selected">History</span>
          <span class="px-2.5 py-1 opacity-60">Snippets</span>
          <span class="px-2.5 py-1 opacity-60">Design</span>
        </div>

        <div class="grid grid-cols-3 gap-1.5 p-2">
          <div v-for="(card, i) in cards" :key="i" class="relative aspect-4/3 overflow-hidden rounded-md border" :class="theme.card">
            <div v-if="card.kind === 'color'" class="size-full" :style="{ background: card.value }" />
            <img v-else-if="card.kind === 'image'" :src="card.src" alt="" draggable="false" class="size-full object-cover">
            <div v-else-if="card.kind === 'text'" class="flex size-full flex-col justify-center gap-1.5 px-3 pb-3">
              <span v-for="(width, j) in card.lines" :key="j" class="h-1 rounded-full" :class="theme.line" :style="{ width: `${width}%` }" />
            </div>
            <div v-else-if="card.kind === 'svg'" class="flex size-full items-center justify-center pb-3">
              <svg viewBox="0 0 24 24" class="size-7" fill="currentColor"><path :d="BOLT" /></svg>
            </div>
            <div v-else class="flex size-full items-center justify-center pb-3">
              <Icon :name="card.icon" class="text-2xl" />
            </div>

            <Icon :name="card.app" class="absolute bottom-1.5 left-1.5 text-xs" />
          </div>
        </div>
      </div>
    </div>

    <svg class="pointer-events-none absolute inset-0 size-full" viewBox="0 0 100 100" preserveAspectRatio="none">
      <line ref="line" x1="125" y1="0" x2="105" y2="100" stroke="white" stroke-opacity="0.7" stroke-width="1.5" vector-effect="non-scaling-stroke" />
    </svg>

    <Motion
      as="span"
      class="absolute top-4 left-4 flex items-center gap-1.5 rounded-full bg-white/70 px-2.5 py-1 text-xs font-medium text-black/70 backdrop-blur-md"
      :initial="false"
      :animate="{ opacity: lightShown ? 1 : 0 }"
    >
      <Icon name="gravity-ui:sun" class="text-sm" />
      Light
    </Motion>
    <Motion
      as="span"
      class="absolute right-4 bottom-4 flex items-center gap-1.5 rounded-full bg-black/50 px-2.5 py-1 text-xs font-medium text-white/80 backdrop-blur-md"
      :initial="{ opacity: 0 }"
      :animate="{ opacity: darkShown ? 1 : 0 }"
    >
      <Icon name="gravity-ui:moon" class="text-sm" />
      Dark
    </Motion>
  </div>
</template>
