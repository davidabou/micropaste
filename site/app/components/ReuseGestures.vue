<script setup lang="ts">
import ReuseClick from '@/components/ReuseClick.vue'
import ReuseDrag from '@/components/ReuseDrag.vue'
import ReuseMenu from '@/components/ReuseMenu.vue'

const DURATION = 6

const gestures = [
  {
    title: 'Click',
    icon: 'gravity-ui:hand-point-up',
    description: 'Click a card to put it back on the clipboard, ready for Ctrl+V.',
    scene: ReuseClick
  },
  {
    title: 'Drag',
    icon: 'gravity-ui:hand',
    description: 'Drag a card straight into any app: text, images and files land where you need them.',
    scene: ReuseDrag
  },
  {
    title: 'Right-click',
    icon: 'gravity-ui:square-list-ul',
    description: 'The full menu: copy, details, favorite, add to collections, delete.',
    scene: ReuseMenu
  }
]

const active = ref(0)

const root = ref<HTMLElement>()
const inView = useInView(root, { amount: 0.4 })

function next() {
  active.value = (active.value + 1) % gestures.length
}
</script>

<template>
  <div ref="root" class="grid w-full gap-8 md:grid-cols-[2fr_3fr] md:items-center">
    <div class="grid">
      <!-- Reserves the longest description, so switching tabs never moves the page -->
      <div aria-hidden="true" class="invisible col-start-1 row-start-1 flex flex-col">
        <span v-for="gesture in gestures" :key="gesture.title" class="py-4 text-lg font-medium tracking-tight">{{ gesture.title }}</span>
        <div class="-mb-4 grid">
          <p v-for="gesture in gestures" :key="gesture.title" class="col-start-1 row-start-1 max-w-sm pt-1 pl-7">
            {{ gesture.description }}
          </p>
        </div>
      </div>

      <div role="tablist" aria-orientation="vertical" class="col-start-1 row-start-1 flex flex-col">
        <button
          v-for="(gesture, i) in gestures"
          :key="gesture.title"
          role="tab"
          :aria-selected="i === active"
          class="group relative py-4 text-left"
          @click="active = i"
        >
          <!-- From under this icon to the next one -->
          <span
            v-if="i < gestures.length - 1 || i === active"
            class="absolute top-11.5 -bottom-3.5 left-2 w-0.5 overflow-hidden rounded-full bg-black/10"
          >
            <Motion
              v-if="i === active && inView"
              :key="active"
              class="size-full origin-top bg-neutral-800"
              :initial="{ scaleY: 0 }"
              :animate="{ scaleY: 1 }"
              :transition="{ duration: DURATION, ease: 'linear' }"
              @animation-complete="next"
            />
          </span>

          <span
            class="flex items-center gap-2.5 text-lg font-medium tracking-tight transition-colors duration-200"
            :class="i === active ? 'text-neutral-800' : 'text-neutral-400 group-hover:text-neutral-600'"
          >
            <Icon :name="gesture.icon" size="18" class="shrink-0" />
            {{ gesture.title }}
          </span>

          <AnimatePresence :initial="false">
            <Motion
              v-if="i === active"
              class="overflow-hidden"
              :initial="{ height: 0, opacity: 0, marginBottom: 0 }"
              :animate="{ height: 'auto', opacity: 1, marginBottom: -16 }"
              :exit="{ height: 0, opacity: 0, marginBottom: 0 }"
              :transition="{ duration: 0.4, ease }"
            >
              <p class="max-w-sm pt-1 pl-7 text-neutral-500">
                {{ gesture.description }}
              </p>
            </Motion>
          </AnimatePresence>
        </button>
      </div>
    </div>

    <div aria-hidden="true" class="relative h-96 overflow-hidden rounded-2xl border border-black/6 bg-white/50">
      <div class="absolute inset-0 bg-dots" />

      <div class="absolute inset-0 flex items-center justify-center">
        <AnimatePresence mode="wait">
          <Motion
            v-if="inView"
            :key="active"
            class="absolute inset-0 max-sm:scale-75"
            :initial="{ opacity: 0, y: 12, filter: 'blur(6px)' }"
            :animate="{ opacity: 1, y: 0, filter: 'blur(0px)' }"
            :exit="{ opacity: 0, y: -12, filter: 'blur(6px)' }"
            :transition="{ duration: 0.4, ease }"
          >
            <component :is="gestures[active]!.scene" />
          </Motion>
        </AnimatePresence>
      </div>
    </div>
  </div>
</template>
