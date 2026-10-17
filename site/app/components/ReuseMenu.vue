<script setup lang="ts">
// 2 right-click + menu, 3 release, 4-7 down to Collections, 8 submenu, 9 on Design, 10 click, 11 release
const step = ref(0)

const ITEM = 32

const items = [
  { label: 'Copy', icon: 'gravity-ui:copy' },
  { label: 'Details', icon: 'gravity-ui:circle-info' },
  { label: 'Favorite', icon: 'gravity-ui:star' },
  { label: 'Collections', icon: 'gravity-ui:folder', sub: true },
  { label: 'Delete', icon: 'gravity-ui:trash-bin' }
]

const collections = ['Snippets', 'Design']

const highlighted = computed(() => (step.value < 4 ? -1 : Math.min(step.value - 4, 3)))

const stage = ref<HTMLElement>()
const card = ref<HTMLElement>()
const itemEls = ref<HTMLElement[]>([])
const design = ref<HTMLElement>()

const pointer = usePointer(stage)

useSequence(async (wait) => {
  await wait(0.2)
  // Where the menu's top left corner lands on the card
  await pointer.moveTo(card.value, [0.75, 0.36])
  step.value = 2
  await wait(0.15)
  step.value = 3
  await wait(0.6)
  for (let i = 0; i < 4; i++) {
    await pointer.moveTo(itemEls.value[i], [0.45, 0.5], 0.3)
    step.value = 4 + i
    await wait(0.15)
  }
  await wait(0.1)
  step.value = 8
  await wait(0.5)
  await nextTick()
  await pointer.moveTo(design.value, [0.45, 0.5], 0.5)
  step.value = 9
  await wait(0.3)
  step.value = 10
  await wait(0.15)
  step.value = 11
})

const menuSpring = { type: 'spring' as const, bounce: 0.35, visualDuration: 0.2 }
const menu = 'relative z-10 shrink-0 origin-top-left rounded-lg border border-black/[5.78%] bg-[#f9f9f9] p-1 text-sm shadow-lg'
const menuItem = 'relative flex h-8 items-center gap-2 px-2'
</script>

<template>
  <div ref="stage" class="absolute inset-0 flex items-center justify-center text-black/90">
    <div class="flex items-start">
      <Motion layout :data-step="step" class="shrink-0" :transition="layoutSpring">
        <div ref="card" class="flex h-35 w-40 flex-col justify-end overflow-hidden rounded-lg bg-[#5B5BD6] p-3 text-xs text-white shadow-card">
          <p class="mb-1 text-sm font-medium">#5B5BD6</p>
          <div class="flex items-center gap-1.5">
            <Icon name="logos:figma" class="shrink-0 text-base" />
            <span class="opacity-80">9 min ago</span>
            <span class="ml-auto opacity-70">HEX</span>
          </div>
        </div>
      </Motion>

      <Motion
        v-if="step >= 2"
        layout
        :data-step="step"
        :class="menu"
        class="mt-12.5 -ml-10 w-42"
        :initial="{ opacity: 0, scale: 0.85 }"
        :animate="{ opacity: 1, scale: 1 }"
        :transition="{ ...menuSpring, layout: layoutSpring }"
      >
        <Motion
          class="absolute inset-x-1 top-1 h-8 rounded-sm bg-black/[3.73%]"
          :initial="false"
          :animate="{ y: ITEM * Math.max(highlighted, 0), opacity: highlighted < 0 ? 0 : 1 }"
          :transition="{ type: 'spring', bounce: 0, visualDuration: 0.15, opacity: { duration: 0.1 } }"
        />

        <div
          v-for="(item, i) in items"
          :key="item.label"
          :ref="el => (itemEls[i] = el as HTMLElement)"
          :class="menuItem"
        >
          <Icon :name="item.icon" class="shrink-0 text-sm opacity-70" />
          {{ item.label }}
          <Icon v-if="item.sub" name="gravity-ui:chevron-right" class="ml-auto text-xs opacity-60" />
        </div>
      </Motion>

      <Motion
        v-if="step >= 8"
        layout
        :class="menu"
        class="mt-36.5 ml-1 w-37.5"
        :initial="{ opacity: 0, scale: 0.85 }"
        :animate="{ opacity: 1, scale: 1 }"
        :transition="{ ...menuSpring, layout: layoutSpring }"
      >
        <Motion
          class="absolute inset-x-1 top-1 h-8 rounded-sm bg-black/[3.73%]"
          :initial="{ opacity: 0, y: ITEM }"
          :animate="{ opacity: step >= 9 ? 1 : 0, y: ITEM }"
          :transition="{ duration: 0.1 }"
        />

        <div
          v-for="name in collections"
          :key="name"
          :ref="el => name === 'Design' && (design = el as HTMLElement)"
          :class="menuItem"
        >
          <span class="flex w-3 items-center justify-center">
            <Motion
              v-if="name === 'Design' && step >= 10"
              class="flex"
              :initial="{ scale: 0 }"
              :animate="{ scale: 1 }"
              :transition="{ type: 'spring', bounce: 0.5, visualDuration: 0.2 }"
            >
              <Icon name="gravity-ui:check" class="text-sm" />
            </Motion>
          </span>
          {{ name }}
        </div>
      </Motion>
    </div>

    <GestureCursor :x="pointer.x" :y="pointer.y" :pressed="step === 2 || step === 10" />
  </div>
</template>
