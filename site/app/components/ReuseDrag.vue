<script setup lang="ts">
// 2 press + lift + mail window, 3 drag, 4 over the drop zone, 5 drop, 6 move away
const step = ref(0)

const stage = ref<HTMLElement>()
const source = ref<HTMLElement>()
const zone = ref<HTMLElement>()

const pointer = usePointer(stage)

useSequence(async (wait) => {
  await wait(0.2)
  await pointer.moveTo(source.value)
  step.value = 2
  await wait(0.7)
  step.value = 3
  await pointer.moveTo(zone.value, [0.5, 0.5], 0.8)
  step.value = 4
  await wait(0.4)
  step.value = 5
  await wait(0.5)
  step.value = 6
  void pointer.moveTo(zone.value, [1.1, 1.6])
})

const ghost = computed(() => {
  if (step.value < 2) return { opacity: 0, scale: 1, rotate: 0 }
  if (step.value < 5) return { opacity: 1, scale: 1.05, rotate: -4 }
  return { opacity: 0, scale: 0.5, rotate: 0 }
})
</script>

<template>
  <div ref="stage" class="absolute inset-0 flex items-center justify-center text-black/90">
    <div class="flex items-center gap-8">
      <Motion
        layout
        :data-step="step"
        class="relative h-55 w-42.5 shrink-0 overflow-hidden rounded-lg border border-black/10 bg-[#f3f3f3] shadow-card"
        :transition="layoutSpring"
      >
        <div class="flex h-10 items-center px-2">
          <span class="flex h-6 items-center rounded-md bg-black/[5.78%] px-2.5 text-xs font-medium">History</span>
        </div>

        <div ref="source" class="absolute top-13 left-3 h-27.5 w-36.5 overflow-hidden rounded-lg border border-black/[5.78%]">
          <img src="/background.jpg" alt="" draggable="false" class="size-full object-cover">
        </div>

        <div class="absolute top-42.5 left-3 flex h-9.5 w-36.5 flex-col justify-center gap-1.5 rounded-lg border border-black/[5.78%] bg-black/[3.73%] px-3">
          <span class="h-1.5 w-24 rounded-full bg-black/10" />
          <span class="h-1.5 w-16 rounded-full bg-black/10" />
        </div>
      </Motion>

      <Motion
        v-if="step >= 2"
        layout
        class="flex h-60 w-60 shrink-0 flex-col rounded-xl border border-black/[5.78%] bg-white/90 p-4 text-sm shadow-card"
        :initial="{ opacity: 0, x: 24 }"
        :animate="{ opacity: 1, x: 0 }"
        :transition="layoutSpring"
      >
        <p class="border-b border-black/[5.78%] pb-2 text-black/50">
          To: <span class="text-black/80">team</span>
        </p>
        <p class="border-b border-black/[5.78%] py-2 text-black/50">
          Subject: <span class="text-black/80">New wallpaper</span>
        </p>

        <div
          ref="zone"
          class="mt-3 flex h-25 items-center justify-center rounded-lg border border-dashed transition-colors duration-200"
          :class="step === 4 ? 'border-accent bg-accent/8 text-accent' : 'border-black/20 text-black/40'"
        >
          <AnimatePresence mode="wait">
            <Motion
              v-if="step < 5"
              key="hint"
              as="span"
              class="font-medium"
              :exit="{ opacity: 0, scale: 0.9, transition: { duration: 0.1 } }"
            >
              Drop to attach
            </Motion>
            <Motion
              v-else
              key="file"
              class="flex items-center gap-2 rounded-md bg-white px-2 py-1.5 shadow-sm ring-1 ring-black/10"
              :initial="{ opacity: 0, scale: 0.8 }"
              :animate="{ opacity: 1, scale: 1 }"
              :transition="{ type: 'spring', bounce: 0.35, visualDuration: 0.3 }"
            >
              <img src="/background.jpg" alt="" class="h-7 w-10 rounded object-cover">
              <span class="flex flex-col leading-tight">
                <span class="text-xs font-medium text-black/80">background.jpg</span>
                <span class="text-[11px] text-black/50">2.4 MB</span>
              </span>
            </Motion>
          </AnimatePresence>
        </div>
      </Motion>
    </div>

    <!-- Follows the pointer, centered on it -->
    <Motion
      class="pointer-events-none absolute top-0 left-0 z-10"
      :style="{ x: pointer.x, y: pointer.y }"
    >
      <Motion
        class="h-27.5 w-36.5 -translate-x-1/2 -translate-y-1/2 overflow-hidden rounded-lg border border-black/[5.78%] shadow-[0_20px_40px_-12px_rgba(24,38,84,0.45)]"
        :initial="false"
        :animate="ghost"
        :transition="{ type: 'spring', bounce: 0.2, visualDuration: 0.3, opacity: { duration: 0.2 } }"
      >
        <img src="/background.jpg" alt="" draggable="false" class="size-full object-cover">
      </Motion>
    </Motion>

    <GestureCursor :x="pointer.x" :y="pointer.y" :pressed="step >= 2 && step < 5" />
  </div>
</template>
