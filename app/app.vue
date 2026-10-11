<script setup lang="ts">
import { animate } from 'motion-v'

const search = ref('')
/** Open collection, `null` for the history */
const collection = ref<number | null>(null)
/** Tiles in a grid or rows in a list, remembered from one launch to the next */
const layout = ref<'grid' | 'list'>('grid')
onMounted(() => {
  if (localStorage.getItem('micropaste:layout') === 'list') layout.value = 'list'
})
watch(layout, value => localStorage.setItem('micropaste:layout', value))

/** The star of the title bar: shows only the favorites of the open tab */
const favoritesOnly = ref(false)
const {
  historyClips,
  openCollectionClips,
  shownCollection,
  searching,
  loading,
  copy,
  remove,
  addToCollection,
  removeFromCollection,
  prefetch,
  toggleFavorite,
  moveInCollection,
  saveCollectionOrder,
  dropIntoCollection
} = useClips(search, collection, favoritesOnly)

// Shown in a grid without clips, depending on what filters it
function emptyMessage(clips: readonly unknown[], inCollection: boolean) {
  if (clips.length) return null
  const query = search.value.trim()
  if (searching.value && query) return `No results for “${query}”`
  if (searching.value && favoritesOnly.value) return 'No favorites here yet. Click the star of a clip to add it.'
  if (inCollection && !loading.value && !searching.value) return 'This collection is empty. Right-click a clip in History to add it here.'
  return null
}
const isHistoryShown = computed(() => shownCollection.value === null)

// Changing tab swipes the grids side by side: toward the left when the new tab is on the right,
// and back. Motion keeps the direction right and can turn around mid-swipe
const { collections } = useCollections()
const tabIndex = (id: number | null) => (id === null ? 0 : collections.value.findIndex(c => c.id === id) + 1)
const swipeForward = ref(true)
watch(shownCollection, (shown, previous) => {
  swipeForward.value = tabIndex(shown) >= tabIndex(previous)
})
const swipeTransition = { type: 'spring', bounce: 0, duration: 0.35 } as const
// Collection grids swipe through Transition hooks: they read the direction when the swipe starts,
// even for the leaving grid, whose props no longer update
function swipeIn(el: Element, done: () => void) {
  animate(el, { x: [swipeForward.value ? '100%' : '-100%', 0] }, swipeTransition).then(done)
}
function swipeOut(el: Element, done: () => void) {
  animate(el, { x: swipeForward.value ? '-100%' : '100%' }, swipeTransition).then(done)
}
// The history is always the first tab: it leaves toward the left, and comes back from there.
// Out of view (clipped by the tab area), it stays alive for an instant return
const historyAnimation = computed(() => ({ x: isHistoryShown.value ? 0 : '-100%' }))

// Details of a clip, in a panel sliding over the right side. Starring it there updates the panel too
const detailsClip = ref<Clip | null>(null)
async function toggleDetailsFavorite(clip: Clip) {
  const updated = await toggleFavorite(clip)
  if (detailsClip.value?.id === updated.id) detailsClip.value = updated
}

// A tile dropped on a collection tab goes into that collection
function onDropClip(collectionId: number) {
  if (draggedClip.value) addToCollection(draggedClip.value, collectionId)
}

// Ctrl+F focuses the search field, instead of opening the webview's find bar
const searchInput = useTemplateRef<{ focus: () => void }>('searchInput')
function onKeydown(event: KeyboardEvent) {
  if (event.ctrlKey && event.key.toLowerCase() === 'f') {
    event.preventDefault()
    searchInput.value?.focus()
  }
}
// Drags are handled by the page, not by Tauri (`dragDropEnabled: false`). A tile dragged over
// the window shows the normal copy cursor instead of a blocked one. Content from other apps (files,
// selected text, pictures) can be dropped into an open collection only: dropping isn't copying, so
// it doesn't go in the history. Anything else can't be dropped: the webview would open a dropped
// file in place of the app
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
  // Left the window
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
  // Pictures dragged from a browser come with their link: their bytes are taken. Files from the
  // Explorer have no link: WebView2 hands their paths to the backend, the page can't read them
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

onMounted(() => {
  window.addEventListener('keydown', onKeydown)
  window.addEventListener('dragover', onDragOver)
  window.addEventListener('drop', onDrop)
  window.addEventListener('dragleave', onDragLeave)
})
onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKeydown)
  window.removeEventListener('dragover', onDragOver)
  window.removeEventListener('drop', onDrop)
  window.removeEventListener('dragleave', onDragLeave)
})
</script>

<template>
  <!-- Shared by every tooltip: shows them after the same delay as Windows -->
  <TooltipProvider :delay-duration="500" disable-hoverable-content>
    <div class="flex h-screen flex-col">
      <TitleBar>
        <!-- Escape clears the search -->
        <Input
          ref="searchInput"
          v-model="search"
          class="w-60"
          icon="gravity-ui:magnifier"
          placeholder="Search…"
          spellcheck="false"
          @keydown.esc="search = ''"
        />
        <Tooltip :content="favoritesOnly ? 'Show all clips' : 'Show favorites only'">
          <Button
            :icon="favoritesOnly ? 'gravity-ui:star-fill' : 'gravity-ui:star'"
            size="sm"
            :pressed="favoritesOnly"
            @click="favoritesOnly = !favoritesOnly"
          />
        </Tooltip>
        <!-- Shows the layout it switches to -->
        <Tooltip :content="layout === 'grid' ? 'Show as list' : 'Show as grid'">
          <Button
            :icon="layout === 'grid' ? 'gravity-ui:layout-cells-large' : 'gravity-ui:layout-list'"
            size="sm"
            @click="layout = layout === 'grid' ? 'list' : 'grid'"
          />
        </Tooltip>
      </TitleBar>

      <CollectionTabs v-model="collection" @prefetch="prefetch" @drop-clip="onDropClip" />

      <!-- Changing tab swipes to the other grid once the new tab is loaded (instant for a collection
           already opened or hovered). The history grid stays alive while a collection is open:
           coming back to it is instant, at the same scroll position. overflow-hidden: grids swipe
           in and out of view -->
      <!-- Motion props on ClipGrid go to its root Motion element -->
      <div class="relative flex min-h-0 flex-1 overflow-hidden">
        <ClipGrid
          :initial="false"
          :animate="historyAnimation"
          :inert="!isHistoryShown"
          :transition="swipeTransition"
          :clips="historyClips"
          :collection="null"
          :layout="layout"
          @info="detailsClip = $event"
          @copy="copy"
          @favorite="toggleFavorite"
          @delete="remove"
          @add="addToCollection"
          @remove-from="removeFromCollection"
        >
          <p v-if="emptyMessage(historyClips, false)" class="col-span-full py-16 text-center text-sm opacity-60">
            {{ emptyMessage(historyClips, false) }}
          </p>
        </ClipGrid>
        <Transition :css="false" @enter="swipeIn" @leave="swipeOut">
          <!-- The swipe moves this plain wrapper: Motion stops every animation on its own element
               (the grid root) when it unmounts, which would cut the leaving swipe short -->
          <div
            v-if="shownCollection !== null"
            :key="shownCollection"
            class="absolute inset-0 transition-[filter] duration-200"
            :class="externalDrop && 'blur-sm [clip-path:inset(0.5rem_round_0.5rem)]'"
          >
            <ClipGrid
              :clips="openCollectionClips"
              :collection="shownCollection"
              :layout="layout"
              :reorderable="!searching"
              @info="detailsClip = $event"
              @move="moveInCollection"
              @move-end="saveCollectionOrder"
              @copy="copy"
              @favorite="toggleFavorite"
              @delete="remove"
              @add="addToCollection"
              @remove-from="removeFromCollection"
            >
              <p v-if="emptyMessage(openCollectionClips, true)" class="col-span-full py-16 text-center text-sm opacity-60">
                {{ emptyMessage(openCollectionClips, true) }}
              </p>
            </ClipGrid>
          </div>
        </Transition>

        <!-- Shown while content from another app is dragged over an open collection, which blurs
             itself below (a backdrop blur doesn't reach its text) -->
        <AnimatePresence>
          <Motion
            v-if="externalDrop"
            class="pointer-events-none absolute inset-2 z-30 flex items-center justify-center rounded-lg border-2 border-dashed border-black/20 bg-white/50 dark:border-white/25 dark:bg-black/40"
            :initial="{ opacity: 0, scale: 0.98 }"
            :animate="{ opacity: 1, scale: 1 }"
            :exit="{ opacity: 0, scale: 0.98 }"
            :transition="{ type: 'spring', bounce: 0, visualDuration: 0.15 }"
          >
            <p class="flex items-center gap-2 font-ui text-sm font-medium">
              <Icon name="gravity-ui:folder-plus" class="text-base" />
              Drop to add to “{{ shownCollectionName }}”
            </p>
          </Motion>
        </AnimatePresence>
      </div>
    </div>

    <ClipDetails :clip="detailsClip" @close="detailsClip = null" @copy="copy" @favorite="toggleDetailsFavorite" />
  </TooltipProvider>
</template>
