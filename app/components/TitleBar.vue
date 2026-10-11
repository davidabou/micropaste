<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'

const isMaximized = ref(false)
let unlistenResize: (() => void) | undefined

onMounted(async () => {
  const appWindow = getCurrentWindow()
  isMaximized.value = await appWindow.isMaximized()
  unlistenResize = await appWindow.onResized(async () => {
    isMaximized.value = await appWindow.isMaximized()
  })
})

onBeforeUnmount(() => unlistenResize?.())

const minimize = () => getCurrentWindow().minimize()
const toggleMaximize = () => getCurrentWindow().toggleMaximize()
const close = () => getCurrentWindow().close()

// Hovering the maximize button a moment opens the Windows 11 Snap Layouts, like the native button.
// Windows only shows them over its own button: the backend opens them with their shortcut, Win+Z,
// which acts on the focused window. It closes them once the pointer has left the button and them,
// given where the button is on the screen
const SNAP_LAYOUTS_DELAY_MS = 600
let snapTimer: ReturnType<typeof setTimeout> | undefined
function startSnapLayouts(event: MouseEvent) {
  const button = event.currentTarget as HTMLElement
  clearTimeout(snapTimer)
  snapTimer = setTimeout(async () => {
    const appWindow = getCurrentWindow()
    const [position, scale] = await Promise.all([appWindow.innerPosition(), appWindow.scaleFactor()])
    const rect = button.getBoundingClientRect()
    await appWindow.setFocus()
    invoke('show_snap_layouts', {
      button: {
        x: position.x + rect.left * scale,
        y: position.y + rect.top * scale,
        width: rect.width * scale,
        height: rect.height * scale
      }
    })
  }, SNAP_LAYOUTS_DELAY_MS)
}
const cancelSnapLayouts = () => clearTimeout(snapTimer)
onBeforeUnmount(cancelSnapLayouts)

// Same size, glyphs and hover colors as the native Windows 11 caption buttons
const captionButton = 'h-10 w-[46px] font-icons text-[10px] transition-colors duration-100'
const neutralButton = `${captionButton} hover:bg-black/[3.73%] active:bg-black/[2.41%] dark:hover:bg-white/[6.05%] dark:active:bg-white/[4.19%]`
const closeButton = `${captionButton} hover:bg-[#c42b1c] hover:text-white active:bg-[#c42b1c]/90 active:text-white/70`
</script>

<template>
  <header class="flex h-10 items-stretch justify-between select-none" data-tauri-drag-region>
    <!-- Custom buttons go here -->
    <div class="flex items-center gap-1 pl-2">
      <slot />
    </div>

    <div class="flex">
      <button :class="neutralButton" aria-label="Minimize" @click="minimize">
        &#xE921;
      </button>
      <button
        :class="neutralButton"
        :aria-label="isMaximized ? 'Restore' : 'Maximize'"
        @click="cancelSnapLayouts(); toggleMaximize()"
        @mouseenter="startSnapLayouts"
        @mouseleave="cancelSnapLayouts"
      >
        <template v-if="isMaximized">&#xE923;</template>
        <template v-else>&#xE922;</template>
      </button>
      <button :class="closeButton" aria-label="Close" @click="close">
        &#xE8BB;
      </button>
    </div>
  </header>
</template>
