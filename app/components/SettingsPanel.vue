<script setup lang="ts">
import type { UnlistenFn } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

const props = defineProps<{ open: boolean }>()
const emit = defineEmits<{ close: [] }>()

interface Settings {
  shortcut: string
  retentionDays: number | null
  launchAtStartup: boolean
  linkPreviews: boolean
}
const settings = ref<Settings | null>(null)
watch(() => props.open, async (open) => {
  if (open) settings.value = await invoke<Settings>('get_settings')
}, { immediate: true })

let unlisten: UnlistenFn | undefined
onMounted(async () => {
  unlisten = await listen<boolean>('launch-at-startup-changed', ({ payload }) => {
    if (settings.value) settings.value.launchAtStartup = payload
  })
})

const retentionOptions = [
  { value: '1', label: '1 day' },
  { value: '7', label: '1 week' },
  { value: '30', label: '1 month' },
  { value: '90', label: '3 months' },
  { value: '365', label: '1 year' },
  { value: 'forever', label: 'Forever' }
]
const retentionLabel = computed(() => retentionOptions.find(option => option.value === retention.value)?.label)
const retentionMenuOpen = ref(false)
const retentionMenuOpened = ref(false)
watch(retentionMenuOpen, (open) => {
  if (open) retentionMenuOpened.value = true
})
const retention = computed({
  get: () => String(settings.value?.retentionDays ?? 'forever'),
  set: value => chooseRetention(value)
})
const pendingRetention = ref<{ days: number, count: number } | null>(null)
const confirmShown = ref(false)
const retentionRow = useTemplateRef<HTMLElement>('retentionRow')
const retentionRowWidth = ref(0)
const CONTROL_WIDTH = 144
const morph = { type: 'spring', bounce: 0.2, duration: 0.4 } as const
const labelHidden = { opacity: 0, filter: 'blur(4px)', x: -8, transition: { duration: 0.2, ease: 'easeOut' as const } }
const labelShown = { opacity: 1, filter: 'blur(0px)', x: 0, transition: { duration: 0.25, ease: 'easeOut' as const, delay: 0.1 } }
const buttonContentHidden = { opacity: 0, scale: 0.9, transition: { duration: 0.12 } }
const buttonContentShown = { opacity: 1, scale: 1, transition: { duration: 0.15, delay: 0.15 } }
function cancelRetentionOnEscape(event: KeyboardEvent) {
  if (!pendingRetention.value) return
  event.preventDefault()
  pendingRetention.value = null
}
// The click that cancels the confirmation must not also close the panel
let cancellingClick: PointerEvent | null = null
function cancelRetentionOnOutsideClick(event: PointerEvent) {
  if ((event.target as Element | null)?.closest('[data-retention-confirm]')) return
  pendingRetention.value = null
  cancellingClick = event
}
function keepOpenOnCancellingClick(event: CustomEvent<{ originalEvent: PointerEvent }>) {
  if (event.detail.originalEvent === cancellingClick) event.preventDefault()
}
useEventListener(() => (pendingRetention.value ? window : null), 'pointerdown', cancelRetentionOnOutsideClick, { capture: true })
function onConfirmAnimated() {
  if (!pendingRetention.value) confirmShown.value = false
}

async function chooseRetention(value: string) {
  if (!settings.value) return
  const days = value === 'forever' ? null : Number(value)
  const current = settings.value.retentionDays
  if (days !== null && (current === null || days < current)) {
    const count = await invoke<number>('count_expiring_clips', { days })
    if (count > 0) {
      retentionRowWidth.value = retentionRow.value?.offsetWidth ?? CONTROL_WIDTH
      pendingRetention.value = { days, count }
      confirmShown.value = true
      return
    }
  }
  applyRetention(days)
}
const clipCount = (count: number) => `${count.toLocaleString('en-US')} ${count === 1 ? 'clip' : 'clips'}`
function applyRetention(days: number | null) {
  if (!settings.value) return
  pendingRetention.value = null
  settings.value.retentionDays = days
  invoke('set_retention_days', { days })
}

async function setLaunchAtStartup(enabled: boolean) {
  if (!settings.value) return
  settings.value.launchAtStartup = await invoke<boolean>('set_launch_at_startup', { enabled }).catch(() => !enabled)
}

function setLinkPreviews(enabled: boolean) {
  if (!settings.value) return
  settings.value.linkPreviews = enabled
  invoke('set_link_previews', { enabled })
}

const recording = ref(false)
const shortcutError = ref<string | null>(null)
const shortcutLabel = computed(() => settings.value?.shortcut.split('+').map(key => (key === 'Super' ? 'Win' : key)).join(' + '))
const MODIFIERS = ['Control', 'Alt', 'Shift', 'Meta']

function startRecording() {
  shortcutError.value = null
  recording.value = true
  invoke('set_recording_shortcut', { recording: true })
}
function stopRecording() {
  if (!recording.value) return
  recording.value = false
  invoke('set_recording_shortcut', { recording: false })
}
useEventListener(() => (recording.value ? window : null), 'keydown', onRecordKey, { capture: true })
async function onRecordKey(event: KeyboardEvent) {
  event.preventDefault()
  event.stopPropagation()
  if (MODIFIERS.includes(event.key)) return
  const hasModifier = event.ctrlKey || event.altKey || event.shiftKey || event.metaKey
  if (event.key === 'Escape' && !hasModifier) return stopRecording()
  const shortcut = [
    event.ctrlKey && 'Ctrl',
    event.altKey && 'Alt',
    event.shiftKey && 'Shift',
    event.metaKey && 'Super',
    event.code.replace(/^(Key|Digit)/, '')
  ].filter(Boolean).join('+')
  stopRecording()
  try {
    await invoke('set_shortcut', { shortcut })
    if (settings.value) settings.value.shortcut = shortcut
  } catch (error) {
    shortcutError.value = typeof error === 'string' ? error : 'Couldn’t change the shortcut'
  }
}
watch(() => props.open, (open) => {
  if (!open) {
    stopRecording()
    shortcutError.value = null
    pendingRetention.value = null
    confirmShown.value = false
  }
})
onBeforeUnmount(() => {
  stopRecording()
  unlisten?.()
})

const rendered = ref(false)
// In case the end of the animation isn't reported
const { start: unmountLater, stop: keepMounted } = useTimeoutFn(() => (rendered.value = false), 500, { immediate: false })
watch(() => props.open, (open) => {
  if (open) {
    keepMounted()
    rendered.value = true
  } else if (rendered.value) {
    unmountLater()
  }
}, { immediate: true })
const panelAnimation = computed(() =>
  props.open
    ? { x: 0, transition: { type: 'spring' as const, bounce: 0.15, visualDuration: 0.3 } }
    : { x: '105%', transition: { type: 'spring' as const, bounce: 0, visualDuration: 0.2 } }
)
function onPanelAnimated() {
  if (!props.open) rendered.value = false
}

const thumbSpring = { type: 'spring', bounce: 0.15, duration: 0.3 } as const

const { currentVersion, readyVersion, status: updateStatus, check: checkForUpdate, install: installUpdate } = useUpdate()
const updateDescription = computed(() => {
  if (readyVersion.value) return `Version ${readyVersion.value} is ready to install`
  if (updateStatus.value === 'upToDate') return `Micropaste ${currentVersion.value} is up to date`
  return `Version ${currentVersion.value}`
})
</script>

<template>
  <DialogRoot :open="open" @update:open="value => !value && emit('close')">
    <DialogPortal force-mount>
      <DialogOverlay v-if="rendered" force-mount as-child>
        <Motion
          class="fixed inset-0 z-40 bg-black/20"
          :class="!open && 'pointer-events-none'"
          :initial="{ opacity: 0 }"
          :animate="{ opacity: open ? 1 : 0 }"
          :transition="{ duration: 0.2 }"
        />
      </DialogOverlay>
      <DialogContent
        v-if="rendered"
        force-mount
        as-child
        :aria-describedby="undefined"
        @escape-key-down="cancelRetentionOnEscape"
        @pointer-down-outside="keepOpenOnCancellingClick"
      >
        <Motion
          :initial="{ x: '105%' }"
          :animate="panelAnimation"
          class="fixed inset-y-0 right-0 z-50 flex w-96 flex-col overflow-hidden border-l border-black/[5.78%] bg-[#f9f9f9] text-sm shadow-xl dark:border-black/20 dark:bg-[#2c2c2c]"
          @animation-complete="onPanelAnimated"
        >
          <header class="flex h-12 shrink-0 items-center gap-2 pr-2 pl-4">
            <DialogTitle class="min-w-0 flex-1 truncate text-base font-medium">
              Settings
            </DialogTitle>
            <DialogClose as-child>
              <Button icon="gravity-ui:xmark" size="sm" />
            </DialogClose>
          </header>

          <ScrollArea v-if="settings" class="min-h-0 flex-1">
            <div class="grid auto-rows-min grid-cols-[1fr_9rem] items-center gap-x-4 gap-y-5 px-4 pt-1 pb-4">
              <div class="min-w-0">
                <p class="font-medium">
                  Shortcut
                </p>
                <p class="text-xs opacity-60">
                  Shows or hides Micropaste from any app
                </p>
                <p v-if="shortcutError" class="mt-1 text-xs text-[#c42b1c] dark:text-[#ff99a4]">
                  {{ shortcutError }}
                </p>
              </div>
              <Button variant="filled" class="w-full" @click="recording ? stopRecording() : startRecording()" @blur="stopRecording">
                <span class="truncate font-normal" :class="recording && 'opacity-60'">{{ recording ? 'Press a shortcut…' : shortcutLabel }}</span>
              </Button>

              <div ref="retentionRow" class="relative col-span-2 grid grid-cols-subgrid items-center">
                <Motion class="min-w-0" :animate="pendingRetention ? labelHidden : labelShown">
                  <p class="font-medium">
                    Keep history
                  </p>
                  <p class="text-xs opacity-60">
                    Favorites and clips in collections are always kept
                  </p>
                </Motion>
                <DropdownMenuRoot v-model:open="retentionMenuOpen">
                  <DropdownMenuTrigger as-child>
                    <Button variant="filled" class="w-full" :class="confirmShown && 'invisible'">
                      <span class="flex-1 truncate text-left font-normal">{{ retentionLabel }}</span>
                      <Icon name="gravity-ui:chevron-down" class="-mr-1 text-sm opacity-60" />
                    </Button>
                  </DropdownMenuTrigger>
                  <DropdownMenuPortal v-if="retentionMenuOpened" force-mount>
                    <AnimatePresence>
                      <DropdownMenuContent v-if="retentionMenuOpen" force-mount as-child align="end" :side-offset="4">
                        <Motion
                          :class="menuContent"
                          :style="{ transformOrigin: 'var(--reka-dropdown-menu-content-transform-origin)' }"
                          v-bind="menuMotion"
                        >
                          <DropdownMenuRadioGroup v-model="retention">
                            <DropdownMenuRadioItem
                              v-for="option in retentionOptions"
                              :key="option.value"
                              :value="option.value"
                              :class="menuItem"
                            >
                              <span class="flex w-3 items-center justify-center">
                                <DropdownMenuItemIndicator class="flex">
                                  <Icon name="gravity-ui:check" class="text-sm" />
                                </DropdownMenuItemIndicator>
                              </span>
                              {{ option.label }}
                            </DropdownMenuRadioItem>
                          </DropdownMenuRadioGroup>
                        </Motion>
                      </DropdownMenuContent>
                    </AnimatePresence>
                  </DropdownMenuPortal>
                </DropdownMenuRoot>
                <Motion
                  v-if="confirmShown"
                  data-retention-confirm
                  class="absolute inset-y-0 right-0 my-auto flex items-center gap-2 overflow-hidden bg-black/[5.78%] pr-1.5 pl-3 dark:bg-white/[8.37%]"
                  :style="{ borderRadius: '6px' }"
                  :initial="{ width: CONTROL_WIDTH, height: 32 }"
                  :animate="pendingRetention ? { width: retentionRowWidth, height: 40 } : { width: CONTROL_WIDTH, height: 32 }"
                  :transition="morph"
                  @animation-complete="onConfirmAnimated"
                >
                  <Motion
                    class="pointer-events-none absolute inset-y-0 right-0 flex items-center gap-2 px-3 text-xs"
                    :style="{ width: `${CONTROL_WIDTH}px` }"
                    :initial="{ opacity: 1, scale: 1 }"
                    :animate="pendingRetention ? buttonContentHidden : buttonContentShown"
                  >
                    <span class="flex-1 truncate text-left">{{ retentionLabel }}</span>
                    <Icon name="gravity-ui:chevron-down" class="-mr-1 text-sm opacity-60" />
                  </Motion>
                  <Motion
                    v-if="pendingRetention"
                    class="flex min-w-0 flex-1 items-center gap-2"
                    :initial="{ opacity: 0 }"
                    :animate="{ opacity: 1, transition: { delay: 0.1, duration: 0.15 } }"
                  >
                    <span class="min-w-0 flex-1 truncate text-xs">This will delete {{ clipCount(pendingRetention.count) }}.</span>
                    <Button size="sm" @click="pendingRetention = null">
                      Cancel
                    </Button>
                    <Button size="sm" @click="applyRetention(pendingRetention.days)">
                      Apply
                    </Button>
                  </Motion>
                </Motion>
              </div>

              <div class="min-w-0">
                <p class="font-medium">
                  Launch at startup
                </p>
                <p class="text-xs opacity-60">
                  Starts in the background when you sign in to Windows
                </p>
              </div>
              <SwitchRoot
                :model-value="settings.launchAtStartup"
                class="flex h-5 w-9 shrink-0 cursor-pointer items-center justify-self-end rounded-full bg-black/9 transition-colors duration-150 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-current data-[state=checked]:bg-black/85 dark:bg-white/11 dark:data-[state=checked]:bg-white"
                @update:model-value="setLaunchAtStartup"
              >
                <Motion
                  class="size-3.5 rounded-full bg-white shadow-sm"
                  :class="settings.launchAtStartup ? 'dark:bg-[#2c2c2c]' : 'dark:bg-white/80'"
                  :initial="false"
                  :animate="{ x: settings.launchAtStartup ? 19 : 3 }"
                  :transition="thumbSpring"
                />
              </SwitchRoot>

              <div class="min-w-0">
                <p class="font-medium">
                  Link previews
                </p>
                <p class="text-xs opacity-60">
                  Loads the title and image of copied links from their site
                </p>
              </div>
              <SwitchRoot
                :model-value="settings.linkPreviews"
                class="flex h-5 w-9 shrink-0 cursor-pointer items-center justify-self-end rounded-full bg-black/9 transition-colors duration-150 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-current data-[state=checked]:bg-black/85 dark:bg-white/11 dark:data-[state=checked]:bg-white"
                @update:model-value="setLinkPreviews"
              >
                <Motion
                  class="size-3.5 rounded-full bg-white shadow-sm"
                  :class="settings.linkPreviews ? 'dark:bg-[#2c2c2c]' : 'dark:bg-white/80'"
                  :initial="false"
                  :animate="{ x: settings.linkPreviews ? 19 : 3 }"
                  :transition="thumbSpring"
                />
              </SwitchRoot>

              <div class="min-w-0">
                <p class="font-medium">
                  Updates
                </p>
                <p class="text-xs opacity-60">
                  {{ updateDescription }}
                </p>
                <p v-if="updateStatus === 'failed'" class="mt-1 text-xs text-[#c42b1c] dark:text-[#ff99a4]">
                  Couldn’t check for updates
                </p>
              </div>
              <Button v-if="readyVersion" variant="accent" class="w-full" @click="installUpdate">
                <span class="truncate">Restart to update</span>
              </Button>
              <Button v-else variant="filled" class="w-full" :disabled="updateStatus === 'checking'" @click="checkForUpdate">
                <span class="truncate font-normal">{{ updateStatus === 'checking' ? 'Checking…' : 'Check for updates' }}</span>
              </Button>
            </div>
          </ScrollArea>
        </Motion>
      </DialogContent>
    </DialogPortal>
  </DialogRoot>
</template>
