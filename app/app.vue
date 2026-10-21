<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { animate } from 'motion-v'

const search = ref('')
const tab = ref<Tab>('history')
const filter = ref<ClipFilter>({ kind: null, source: null })
const layout = ref<'grid' | 'list'>('grid')
// The clips wait for it, the history skeletons don't: they are in the prerendered page
const layoutLoaded = ref(false)
onMounted(async () => {
  const settings = await invoke<{ layout: 'grid' | 'list' | null }>('get_settings').catch(() => null)
  layout.value = settings?.layout ?? 'grid'
  layoutLoaded.value = true
  watch(layout, value => invoke('set_layout', { layout: value }))
})

const {
  historyClips,
  tabClips,
  shownTab,
  searching,
  loading,
  historyLoading,
  copy,
  remove,
  addToCollection,
  removeFromCollection,
  prefetch,
  toggleFavorite,
  removeFromFavorites,
  removeFromHistory,
  moveInCollection,
  saveCollectionOrder,
  dropIntoCollection
} = useClips(search, tab, filter)

function emptyMessage(clips: readonly unknown[], shown: Tab) {
  if (clips.length) return null
  const query = search.value.trim()
  if (searching.value) return query ? `No results for “${query}”` : 'No clips match these filters'
  if (loading.value || searching.value) return null
  if (shown === 'favorites') return 'No favorites yet. Click the star of a clip to add it here.'
  if (typeof shown === 'number') return 'This collection is empty. Right-click a clip in History to add it here.'
  return null
}
// Out of the grid flow: tiles leaving still take their place for the time of their exit.
// Faded in once they're gone, but not when the grid shows up with it
const emptyClass = 'absolute inset-x-0 top-2 py-16 text-center text-sm'
const emptyFade = {
  initial: { opacity: 0 },
  animate: { opacity: 0.6, transition: { delay: 0.15, duration: 0.2 } },
  exit: { opacity: 0, transition: { duration: 0 } }
}
const isHistoryShown = computed(() => shownTab.value === 'history')
const shownCollection = computed(() => (typeof shownTab.value === 'number' ? shownTab.value : null))

const { collections } = useCollections()
function tabIndex(id: Tab) {
  return id === 'favorites' ? 0 : id === 'history' ? 1 : collections.value.findIndex(c => c.id === id) + 2
}
const swipeForward = ref(true)
const historySwipes = ref(true)
watch(shownTab, (shown, previous) => {
  swipeForward.value = tabIndex(shown) >= tabIndex(previous)
  historySwipes.value = shown === 'history' || previous === 'history'
})
const swipeTransition = { type: 'spring', bounce: 0, duration: 0.35 } as const
function swipeIn(el: Element, done: () => void) {
  animate(el, { x: [swipeForward.value ? '100%' : '-100%', 0] }, swipeTransition).then(done)
}
function swipeOut(el: Element, done: () => void) {
  animate(el, { x: swipeForward.value ? '-100%' : '100%' }, swipeTransition).then(done)
}
// History waits on the side of the shown tab, and jumps sides off-screen (Favorites is on its left)
const historyAnimation = computed(() => ({ x: isHistoryShown.value ? 0 : shownTab.value === 'favorites' ? '100%' : '-100%' }))
const historyTransition = computed(() => (historySwipes.value ? swipeTransition : { duration: 0 }))

const detailsClip = ref<Clip | null>(null)
const settingsOpen = ref(false)

const collectionName = (id: number) => collections.value.find(c => c.id === id)?.name ?? 'collection'
async function addTo(clip: Clip, collectionId: number) {
  const added = await addToCollection(clip, collectionId)
  notifyClip(clip.id, `${added ? 'Added to' : 'Already in'} ${collectionName(collectionId)}`)
}
async function removeFrom(clip: Clip, collectionId: number) {
  const shown = collectionId === shownCollection.value
  await removeFromCollection(clip, collectionId)
  // Removed from the open collection, the tile leaves: that's feedback enough
  if (!shown) notifyClip(clip.id, `Removed from ${collectionName(collectionId)}`)
}

function onDropClip(collectionId: number) {
  if (draggedClip.value) addTo(draggedClip.value, collectionId)
}

const searchInput = useTemplateRef<{ focus: () => void }>('searchInput')
function onKeydown(event: KeyboardEvent) {
  if (event.ctrlKey && event.key.toLowerCase() === 'f') {
    event.preventDefault()
    searchInput.value?.focus()
  }
  if (event.key === 'Escape' && !event.isComposing) hideOnEscape(event)
}

function hideOnEscape(event: KeyboardEvent) {
  // Reka layers listen on the window too, after this listener, and don't mark the event
  const layerOpen = document.querySelector('[data-dismissable-layer]') !== null
  setTimeout(() => {
    if (!layerOpen && !event.defaultPrevented) invoke('hide_window')
  })
}
function onSearchEscape(event: KeyboardEvent) {
  if (!search.value) return
  event.preventDefault()
  search.value = ''
}
const externalDrop = ref(false)
const shownCollectionName = computed(() => collections.value.find(c => c.id === shownCollection.value)?.name)
function acceptsExternal(dataTransfer: DataTransfer | null) {
  if (!dataTransfer || tileDragging.value || shownCollection.value === null) return false
  return dataTransfer.types.includes('Files') || dataTransfer.types.includes('text/plain')
}
function onDragOver(event: DragEvent) {
  event.preventDefault()
  const external = acceptsExternal(event.dataTransfer)
  externalDrop.value = external
  if (event.dataTransfer) event.dataTransfer.dropEffect = tileDragging.value || external ? 'copy' : 'none'
}
function onDragLeave(event: DragEvent) {
  if (!event.relatedTarget) externalDrop.value = false
}
async function onDrop(event: DragEvent) {
  event.preventDefault()
  const accepted = externalDrop.value
  externalDrop.value = false
  const id = shownCollection.value
  const data = event.dataTransfer
  if (!accepted || id === null || !data) return

  const files = [...data.files]
  const fromBrowser = data.types.includes('text/uri-list')
  if (files.length && !fromBrowser) {
    const webview = (window as unknown as { chrome?: { webview?: { postMessageWithAdditionalObjects: (message: string, objects: FileList) => void } } }).chrome?.webview
    webview?.postMessageWithAdditionalObjects(`micropaste-drop-files:${id}`, data.files)
    return
  }
  const picture = files.find(file => file.type.startsWith('image/'))
  if (picture) {
    await dropIntoCollection(id, { image: new Uint8Array(await picture.arrayBuffer()) }).catch(() => {})
    return
  }
  const text = data.getData('text/plain')
  if (text.trim()) await dropIntoCollection(id, { text }).catch(() => {})
}

function onContextMenu(event: MouseEvent) {
  if (import.meta.dev || (event.target as Element).closest('input, textarea')) return
  event.preventDefault()
}

useEventListener('contextmenu', onContextMenu)
useEventListener('keydown', onKeydown)
useEventListener('dragover', onDragOver)
useEventListener('drop', onDrop)
useEventListener('dragleave', onDragLeave)
</script>

<template>
  <TooltipProvider :delay-duration="500" disable-hoverable-content>
    <div class="flex h-screen flex-col">
      <TitleBar>
        <div class="flex min-w-0 flex-1 items-center gap-1.5" data-tauri-drag-region>
          <div class="flex h-8 max-w-96 min-w-0 flex-1 items-center rounded-md border border-black/[5.78%] bg-black/[3.73%] dark:border-white/6 dark:bg-white/[6.05%]">
            <Input
              ref="searchInput"
              v-model="search"
              class="ml-1 min-w-0 flex-1"
              icon="gravity-ui:magnifier"
              placeholder="Search…"
              spellcheck="false"
              @keydown.esc="onSearchEscape"
            />
          </div>
          <ClipFilters v-model="filter" :tab="tab" />
        </div>
        <div class="flex gap-2">
          <Tooltip :content="layout === 'grid' ? 'Show as list' : 'Show as grid'">
            <Button
              :icon="layout === 'grid' ? 'gravity-ui:layout-cells-large' : 'gravity-ui:layout-list'"
              size="sm"
              @click="layout = layout === 'grid' ? 'list' : 'grid'"
            />
          </Tooltip>
          <Tooltip content="Settings">
            <Button icon="gravity-ui:gear" size="sm" @click="settingsOpen = true" />
          </Tooltip>
        </div>
      </TitleBar>

      <CollectionTabs v-model="tab" @prefetch="prefetch" @drop-clip="onDropClip" />

      <div class="relative flex min-h-0 flex-1 overflow-hidden">
        <ClipGrid
          :initial="false"
          :animate="historyAnimation"
          :inert="!isHistoryShown"
          :transition="historyTransition"
          :clips="layoutLoaded ? historyClips : []"
          :collection="null"
          :layout="layout"
          :loading="historyLoading || !layoutLoaded"
          @info="detailsClip = $event"
          @copy="copy"
          @favorite="toggleFavorite"
          @delete="remove"
          @remove-from-history="removeFromHistory"
          @add="addTo"
          @remove-from="removeFrom"
        >
          <AnimatePresence :initial="false">
            <Motion v-if="emptyMessage(historyClips, 'history')" as="p" :class="emptyClass" v-bind="emptyFade">
              {{ emptyMessage(historyClips, 'history') }}
            </Motion>
          </AnimatePresence>
        </ClipGrid>
        <Transition :css="false" @enter="swipeIn" @leave="swipeOut">
          <!-- Motion stops the animations of its own element when it unmounts: the swipe moves this wrapper -->
          <div
            v-if="layoutLoaded && !isHistoryShown"
            :key="shownTab"
            class="absolute inset-0 transition-[filter] duration-200"
            :class="externalDrop && 'blur-sm [clip-path:inset(0.5rem_round_0.5rem)]'"
          >
            <ClipGrid
              :clips="tabClips"
              :collection="shownCollection"
              :favorites="shownTab === 'favorites'"
              :layout="layout"
              :reorderable="!searching && shownCollection !== null"
              @info="detailsClip = $event"
              @move="moveInCollection"
              @move-end="saveCollectionOrder"
              @copy="copy"
              @favorite="shownTab === 'favorites' ? removeFromFavorites($event) : toggleFavorite($event)"
              @delete="remove"
              @add="addTo"
              @remove-from="removeFrom"
            >
              <AnimatePresence :initial="false">
                <Motion v-if="emptyMessage(tabClips, shownTab)" as="p" :class="emptyClass" v-bind="emptyFade">
                  {{ emptyMessage(tabClips, shownTab) }}
                </Motion>
              </AnimatePresence>
            </ClipGrid>
          </div>
        </Transition>

        <!-- A backdrop blur doesn't reach text: the collection blurs itself -->
        <AnimatePresence>
          <Motion
            v-if="externalDrop"
            class="pointer-events-none absolute inset-2 z-30 flex items-center justify-center rounded-lg border-2 border-dashed border-black/20 bg-white/50 dark:border-white/25 dark:bg-black/40"
            :initial="{ opacity: 0, scale: 0.98 }"
            :animate="{ opacity: 1, scale: 1 }"
            :exit="{ opacity: 0, scale: 0.98 }"
            :transition="{ type: 'spring', bounce: 0, visualDuration: 0.15 }"
          >
            <p class="flex items-center gap-2 text-sm font-medium">
              <Icon name="gravity-ui:folder-plus" class="text-base" />
              Drop to add to “{{ shownCollectionName }}”
            </p>
          </Motion>
        </AnimatePresence>
      </div>
    </div>

    <ClipDetails :clip="detailsClip" @close="detailsClip = null" @copy="copy" />
    <SettingsPanel :open="settingsOpen" @close="settingsOpen = false" />
  </TooltipProvider>
</template>
