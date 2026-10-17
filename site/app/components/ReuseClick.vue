<script setup lang="ts">
// 2 press, 3 release + notice, 4 notice gone + paste UI, 5 Ctrl, 6 V, 7 pasted
const step = ref(0)

const stage = ref<HTMLElement>()
const card = ref()
const editor = ref()

const pointer = usePointer(stage)

useSequence(async (wait) => {
  await wait(0.2)
  await pointer.moveTo(card.value, [0.55, 0.5])
  step.value = 2
  await wait(0.15)
  step.value = 3
  await wait(1.3)
  // Let the card and the editor settle before heading to the editor
  pointer.hold()
  step.value = 4
  await wait(0.5)
  await pointer.moveTo(editor.value, [1.08, 1.15])
  await wait(0.1)
  step.value = 5
  await wait(0.2)
  step.value = 6
  await wait(0.25)
  step.value = 7
})

const keys = computed(() => [
  { label: 'Ctrl', down: step.value === 5 || step.value === 6 },
  { label: 'V', down: step.value === 6 }
])

const code = 'export function useClips() {\n  const clips = ref<Clip[]>([])\n  return { clips }\n}'

const KEYWORD = '#af00db'
const FUNCTION = '#795e26'
const TYPE = '#267f99'
const STRING = '#a31515'
const PLAIN = 'rgb(0 0 0 / 0.8)'

const caret = 'mt-0.5 h-3.5 w-px shrink-0 animate-pulse bg-black/70'

// VS Code light colors; line 1 is already in the file, the rest is pasted
const lines = [
  [{ text: 'import', color: KEYWORD }, { text: ' { ref } ', color: PLAIN }, { text: 'from', color: KEYWORD }, { text: ' \'vue\'', color: STRING }],
  [{ text: 'export function', color: KEYWORD }, { text: ' useClips', color: FUNCTION }, { text: '() {', color: PLAIN }],
  [{ text: '  const', color: KEYWORD }, { text: ' clips = ', color: PLAIN }, { text: 'ref', color: FUNCTION }, { text: '<', color: PLAIN }, { text: 'Clip', color: TYPE }, { text: '[]>([])', color: PLAIN }],
  [{ text: '  return', color: KEYWORD }, { text: ' { clips }', color: PLAIN }],
  [{ text: '}', color: PLAIN }]
]
</script>

<template>
  <div ref="stage" class="absolute inset-0 flex items-center justify-center text-black/90">
    <div class="flex flex-col items-center gap-3">
      <Motion
        ref="card"
        layout
        :data-step="step"
        class="relative z-10 flex h-44 w-50 flex-col overflow-hidden rounded-lg border border-black/[5.78%] bg-[#f9f9f9] text-xs shadow-card"
        :class="step >= 2 && step < 4 && 'ring-2 ring-accent'"
        :animate="{ scale: step === 2 ? 0.97 : 1 }"
        :transition="{ ...layoutSpring, scale: { type: 'spring', bounce: 0.4, visualDuration: 0.2 } }"
      >
        <div class="flex min-h-0 flex-1 flex-col transition-[filter] duration-200" :class="step === 3 && 'blur-sm'">
          <p class="line-clamp-5 px-3 pt-3 text-sm whitespace-pre-wrap">{{ code }}</p>

          <footer class="mt-auto flex items-center gap-1.5 px-3 py-2">
            <Icon name="logos:visual-studio-code" class="shrink-0 text-base" />
            <span class="opacity-60">5 min ago</span>
          </footer>
        </div>

        <AnimatePresence>
          <Motion
            v-if="step === 3"
            class="absolute inset-0 flex items-center justify-center bg-white/50"
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
              <Icon name="gravity-ui:circle-check-fill" class="text-base" />
              Copied
            </Motion>
          </Motion>
        </AnimatePresence>
      </Motion>

      <Motion
        v-if="step >= 4"
        ref="editor"
        layout
        class="w-64 overflow-hidden rounded-lg border border-black/[5.78%] bg-white shadow-card"
        :initial="{ opacity: 0, y: -16, scale: 0.95 }"
        :animate="{ opacity: 1, y: 0, scale: 1 }"
        :transition="{ ...layoutSpring, opacity: { duration: 0.25, delay: 0.1 } }"
      >
        <div class="flex h-8 items-center gap-1.5 border-b border-black/[5.78%] bg-[#f9f9f9] pr-1.5 pl-3 text-xs">
          <Icon name="vscode-icons:file-type-typescript" class="shrink-0 text-sm" />
          <span class="text-black/70">clips.ts</span>

          <span class="ml-auto flex gap-1">
            <Motion
              v-for="key in keys"
              :key="key.label"
              as="kbd"
              class="flex h-5 min-w-5 items-center justify-center rounded border border-black/10 bg-white px-1.5 font-sans text-[10px] font-medium text-black/60"
              :animate="{ y: key.down ? 1 : 0, boxShadow: key.down ? '0 0px 0 rgba(0,0,0,0.12)' : '0 1px 0 rgba(0,0,0,0.12)' }"
              :transition="{ duration: 0.08 }"
            >
              {{ key.label }}
            </Motion>
          </span>
        </div>

        <div class="py-2 font-mono text-[11px] leading-4.5">
          <Motion
            v-for="(line, i) in lines"
            :key="i"
            class="flex h-4.5"
            :initial="false"
            :animate="{ opacity: i < 2 || step >= 7 ? 1 : 0 }"
            :transition="{ duration: 0.2, delay: (i - 1) * 0.04 }"
          >
            <span class="w-7 shrink-0 pr-2.5 text-right text-black/25 select-none">{{ i + 1 }}</span>
            <span v-if="i === 1 && step < 7" :class="caret" />
            <Motion
              as="span"
              class="whitespace-pre"
              :initial="false"
              :animate="i === 0 || step >= 7 ? { opacity: 1, filter: 'blur(0px)' } : { opacity: 0, filter: 'blur(4px)' }"
              :transition="{ duration: 0.25, delay: i === 0 ? 0 : (i - 1) * 0.04 }"
            >
              <span v-for="(token, j) in line" :key="j" :style="{ color: token.color }">{{ token.text }}</span>
            </Motion>
            <span v-if="i === lines.length - 1 && step >= 7" :class="caret" />
          </Motion>
        </div>
      </Motion>
    </div>

    <GestureCursor :x="pointer.x" :y="pointer.y" :pressed="step === 2" />
  </div>
</template>
