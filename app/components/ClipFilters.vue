<script setup lang="ts">
import type { ClipFilter, ClipKind, Tab } from '@/composables/useClips'
import { invoke } from '@tauri-apps/api/core'
import { scopeOf } from '@/composables/useClips'

const props = defineProps<{
  tab: Tab
}>()

const filter = defineModel<ClipFilter>({ required: true })

interface Source {
  name: string
  path: string | null
}

const KINDS: { value: ClipKind, label: string, icon: string }[] = [
  { value: 'text', label: 'Text', icon: 'gravity-ui:text' },
  { value: 'link', label: 'Links', icon: 'gravity-ui:globe' },
  { value: 'image', label: 'Images', icon: 'gravity-ui:picture' },
  { value: 'files', label: 'Files', icon: 'gravity-ui:file' },
  { value: 'color', label: 'Colors', icon: 'gravity-ui:palette' }
]
const ALL = 'all'

const kind = computed({
  get: () => filter.value.kind ?? ALL,
  set: value => (filter.value = { ...filter.value, kind: value === ALL ? null : value as ClipKind })
})
const shownKind = computed(() => KINDS.find(k => k.value === filter.value.kind) ?? null)

const sources = ref<Source[]>([])
// Kept apart: the chosen app may not be in the list of another tab
const sourcePath = ref<string | null>(null)
const source = computed({
  get: () => filter.value.source ?? ALL,
  set: (name) => {
    sourcePath.value = sources.value.find(s => s.name === name)?.path ?? null
    filter.value = { ...filter.value, source: name === ALL ? null : name }
  }
})
async function loadSources() {
  sources.value = await invoke<Source[]>('list_sources', scopeOf(props.tab)).catch(() => sources.value)
}

const kindOpen = ref(false)
const kindOpened = ref(false)
watch(kindOpen, open => open && (kindOpened.value = true))
const sourceOpen = ref(false)
const sourceOpened = ref(false)
// The search term shows the chosen app until something is typed: only then is the list filtered
const typing = ref(false)
watch(sourceOpen, (open) => {
  if (!open) return (typing.value = false)
  sourceOpened.value = true
  loadSources()
})

// Same frame as the search bar
const fieldClass = 'flex h-8 w-32 shrink-0 items-center gap-1.5 rounded-md border border-black/[5.78%] bg-black/[3.73%] pl-2.5 pr-1.5 text-xs font-medium transition-colors duration-100 hover:bg-black/[5.78%] focus-within:bg-black/[5.78%] dark:border-white/6 dark:bg-white/[6.05%] dark:hover:bg-white/[8.37%] dark:focus-within:bg-white/[8.37%]'
</script>

<template>
  <SelectRoot v-model="kind" v-model:open="kindOpen">
    <SelectTrigger as-child>
      <button type="button" aria-label="Type" class="focus-visible:outline-2 focus-visible:outline-current" :class="fieldClass">
        <Icon :name="shownKind?.icon ?? 'gravity-ui:layers'" :class="menuIcon" />
        <span class="flex-1 truncate text-left">{{ shownKind?.label ?? 'All types' }}</span>
        <Icon name="gravity-ui:chevron-down" class="text-sm opacity-60" />
      </button>
    </SelectTrigger>
    <SelectPortal v-if="kindOpened" force-mount>
      <AnimatePresence>
        <SelectContent v-if="kindOpen" force-mount as-child position="popper" align="start" :side-offset="4">
          <Motion
            :class="menuContent"
            :style="{ transformOrigin: 'var(--reka-select-content-transform-origin)' }"
            v-bind="menuMotion"
          >
            <SelectViewport>
              <SelectItem :value="ALL" :class="menuItem">
                <Icon name="gravity-ui:layers" :class="menuIcon" />
                <SelectItemText>All types</SelectItemText>
                <SelectItemIndicator class="ml-auto flex pl-3">
                  <Icon name="gravity-ui:check" class="text-sm" />
                </SelectItemIndicator>
              </SelectItem>
              <SelectSeparator :class="menuSeparator" />
              <SelectItem v-for="option in KINDS" :key="option.value" :value="option.value" :class="menuItem">
                <Icon :name="option.icon" :class="menuIcon" />
                <SelectItemText>{{ option.label }}</SelectItemText>
                <SelectItemIndicator class="ml-auto flex pl-3">
                  <Icon name="gravity-ui:check" class="text-sm" />
                </SelectItemIndicator>
              </SelectItem>
            </SelectViewport>
          </Motion>
        </SelectContent>
      </AnimatePresence>
    </SelectPortal>
  </SelectRoot>

  <ComboboxRoot v-model="source" v-model:open="sourceOpen" open-on-click>
    <ComboboxAnchor :class="fieldClass" @pointerenter="loadSources">
      <AppIcon v-if="filter.source !== null" :path="sourcePath" />
      <Icon v-else name="gravity-ui:circles-4-square" :class="menuIcon" />
      <ComboboxInput
        class="h-full min-w-0 flex-1 bg-transparent p-0 outline-none placeholder:text-current"
        placeholder="All apps"
        spellcheck="false"
        :display-value="value => (value === ALL ? 'All apps' : String(value))"
        @focus="($event.target as HTMLInputElement).select()"
        @input="typing = true"
      />
      <ComboboxTrigger class="flex cursor-default items-center" aria-label="Show source apps">
        <Icon name="gravity-ui:chevron-down" class="text-sm opacity-60" />
      </ComboboxTrigger>
    </ComboboxAnchor>
    <ComboboxPortal v-if="sourceOpened" force-mount>
      <AnimatePresence>
        <ComboboxContent v-if="sourceOpen" force-mount as-child position="popper" align="start" :side-offset="4">
          <Motion
            :class="menuContent"
            class="max-h-80 overflow-y-auto"
            :style="{ transformOrigin: 'var(--reka-combobox-content-transform-origin)' }"
            v-bind="menuMotion"
          >
            <ComboboxViewport>
              <ComboboxItem :value="ALL" :class="menuItem">
                <Icon name="gravity-ui:circles-4-square" :class="menuIcon" />
                All apps
                <ComboboxItemIndicator class="ml-auto flex pl-3">
                  <Icon name="gravity-ui:check" class="text-sm" />
                </ComboboxItemIndicator>
              </ComboboxItem>
              <ComboboxSeparator v-if="!typing" :class="menuSeparator" />
              <ComboboxItem v-for="app in sources" :key="app.name" :value="app.name" :class="menuItem">
                <AppIcon :path="app.path" />
                <span class="max-w-48 truncate">{{ app.name }}</span>
                <ComboboxItemIndicator class="ml-auto flex pl-3">
                  <Icon name="gravity-ui:check" class="text-sm" />
                </ComboboxItemIndicator>
              </ComboboxItem>
              <ComboboxEmpty :class="menuItem" class="opacity-60">
                No apps found
              </ComboboxEmpty>
            </ComboboxViewport>
          </Motion>
        </ComboboxContent>
      </AnimatePresence>
    </ComboboxPortal>
  </ComboboxRoot>
</template>
