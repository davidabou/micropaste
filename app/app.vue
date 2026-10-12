<script setup lang="ts">
import { animate } from 'motion-v'

const search = ref('')
const collection = ref<number | null>(null)
const layout = ref<'grid' | 'list'>('grid')
onMounted(() => {
  if (localStorage.getItem('micropaste:layout') === 'list') layout.value = 'list'
})
watch(layout, value => localStorage.setItem('micropaste:layout', value))

const favoritesOnly = ref(false)
const {
  historyClips,
  openCollectionClips,
  shownCollection,
  searching,
  loading,
  historyLoading,
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

function emptyMessage(clips: readonly unknown[], inCollection: boolean) {
  if (clips.length) return null
  const query = search.value.trim()
  if (searching.value && query) return `No results for “${query}”`
  if (searching.value && favoritesOnly.value) return 'No favorites here yet. Click the star of a clip to add it.'
  if (inCollection && !loading.value && !searching.value) return 'This collection is empty. Right-click a clip in History to add it here.'
  return null
}
const isHistoryShown = computed(() => shownCollection.value === null)

const { collections } = useCollections()
const tabIndex = (id: number | null) => (id === null ? 0 : collections.value.findIndex(c => c.id === id) + 1)
const swipeForward = ref(true)
watch(shownCollection, (shown, previous) => {
  swipeForward.value = tabIndex(shown) >= tabIndex(previous)
})
const swipeTransition = { type: 'spring', bounce: 0, duration: 0.35 } as const
function swipeIn(el: Element, done: () => void) {
  animate(el, { x: [swipeForward.value ? '100%' : '-100%', 0] }, swipeTransition).then(done)
}
function swipeOut(el: Element, done: () => void) {
  animate(el, { x: swipeForward.value ? '-100%' : '100%' }, swipeTransition).then(done)
}
const historyAnimation = computed(() => ({ x: isHistoryShown.value ? 0 : '-100%' }))

const detailsClip = ref<Clip | null>(null)
const settingsOpen = ref(false)
async function toggleDetailsFavorite(clip: Clip) {
  const updated = await toggleFavorite(clip)
  if (detailsClip.value?.id === updated.id) detailsClip.value = updated
}

function onDropClip(collectionId: number) {
  if (draggedClip.value) addToCollection(draggedClip.value, collectionId)
}

const searchInput = useTemplateRef<{ focus: () => void }>('searchInput')
function onKeydown(event: KeyboardEvent) {
  if (event.ctrlKey && event.key.toLowerCase() === 'f') {
    event.preventDefault()
    searchInput.value?.focus()
  }
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
  <TooltipProvider :delay-duration="500" disable-hoverable-content>
    <div class="flex h-screen flex-col">
      <TitleBar>
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
      </TitleBar>

      <CollectionTabs v-model="collection" @prefetch="prefetch" @drop-clip="onDropClip" />

      <div class="relative flex min-h-0 flex-1 overflow-hidden">
        <ClipGrid
          :initial="false"
          :animate="historyAnimation"
          :inert="!isHistoryShown"
          :transition="swipeTransition"
          :clips="historyClips"
          :collection="null"
          :layout="layout"
          :loading="historyLoading"
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
          <!-- Motion stops the animations of its own element when it unmounts: the swipe moves this wrapper -->
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
            <p class="flex items-center gap-2 font-ui text-sm font-medium">
              <Icon name="gravity-ui:folder-plus" class="text-base" />
              Drop to add to “{{ shownCollectionName }}”
            </p>
          </Motion>
        </AnimatePresence>
      </div>
    </div>

    <ClipDetails :clip="detailsClip" @close="detailsClip = null" @copy="copy" @favorite="toggleDetailsFavorite" />
    <SettingsPanel :open="settingsOpen" @close="settingsOpen = false" />
  </TooltipProvider>
</template>
