<script setup lang="ts">
const tiles = [
  { clip: sampleClips.link, side: 'left', x: '3%', y: '-9%', rotate: -8, depth: 1.2 },
  { clip: sampleClips.color, side: 'left', x: '8%', y: '34%', rotate: 6, depth: 0.6 },
  { clip: sampleClips.files, side: 'left', x: '-3%', y: '68%', rotate: -4, depth: 1.5 },
  { clip: sampleClips.image, side: 'right', x: '3%', y: '-7%', rotate: 7, depth: 1 },
  { clip: sampleClips.svg, side: 'right', x: '8%', y: '36%', rotate: -6, depth: 0.5 },
  { clip: sampleClips.text, side: 'right', x: '-3%', y: '70%', rotate: 5, depth: 1.3 }
] as const

const pointerX = useMotionValue(0)
const pointerY = useMotionValue(0)
const smoothX = useSpring(pointerX, { stiffness: 50, damping: 20 })
const smoothY = useSpring(pointerY, { stiffness: 50, damping: 20 })

const parallax = tiles.map(tile => ({
  x: useTransform(smoothX, v => v * tile.depth * 60),
  y: useTransform(smoothY, v => v * tile.depth * 60)
}))

const reducedMotion = useReducedMotion()

function onPointerMove(event: PointerEvent) {
  if (reducedMotion.value) return
  const box = (event.currentTarget as HTMLElement).getBoundingClientRect()
  pointerX.set((event.clientX - box.left) / box.width - 0.5)
  pointerY.set((event.clientY - box.top) / box.height - 0.5)
}

function onPointerLeave() {
  pointerX.set(0)
  pointerY.set(0)
}
</script>

<template>
  <section class="w-full px-6 mt-36">
    <div class="relative isolate flex min-h-112 flex-col items-center justify-center gap-8 overflow-hidden rounded-3xl border border-black/6 bg-white/50 px-6 py-20 text-center" @pointermove="onPointerMove" @pointerleave="onPointerLeave">
      <div aria-hidden="true" class="absolute inset-0 -z-10 bg-dots" />

      <!-- Tiles fly in from the middle, follow the pointer by depth, and drift on their own loop -->
      <Motion
        v-for="(tile, i) in tiles"
        :key="tile.clip.kind"
        aria-hidden="true"
        class="absolute hidden w-48 text-left lg:block"
        :style="{ top: tile.y, [tile.side]: tile.x }"
        :initial="{ opacity: 0, scale: 0.7, rotate: 0, x: tile.side === 'left' ? 120 : -120 }"
        :while-in-view="{ opacity: 1, scale: 1, rotate: tile.rotate, x: 0 }"
        :in-view-options="{ once: true, amount: 0.3 }"
        :transition="{ type: 'spring', bounce: 0.3, visualDuration: 0.8, delay: 0.2 + 0.08 * i }"
      >
        <Motion :style="{ x: parallax[i]!.x, y: parallax[i]!.y }">
          <Motion
            class="rounded-lg bg-[#f9f9f9] shadow-card"
            :animate="{ y: [0, -10, 0] }"
            :transition="{ duration: 5 + i * 0.6, ease: 'easeInOut', repeat: Infinity, delay: i * 0.4 }"
            :while-hover="{ scale: 1.06 }"
          >
            <ClipTile :clip="tile.clip" />
          </Motion>
        </Motion>
      </Motion>

      <Motion as="h2" class="font-medium text-5xl tracking-tight" v-bind="reveal()">
        Stop losing
        <br>
        <span class="font-title text-neutral-400">what you copy</span>
      </Motion>
      <Motion as="p" class="max-w-sm text-neutral-600" v-bind="reveal(0.1)">
        Free, open source and local-first. Install it once, and every copy is kept from then on.
      </Motion>
      <Motion class="flex flex-col items-center gap-3" v-bind="reveal(0.2)">
        <Button :to="RELEASES_URL" icon="mdi:microsoft-windows-classic">
          Download for Windows
        </Button>
        <small class="text-neutral-400">Windows 11 · 64-bit · No account needed</small>
      </Motion>
    </div>
  </section>
</template>
