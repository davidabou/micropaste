import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

export interface Clip {
  id: number
  kind: 'text' | 'image' | 'files'
  /** The text, the image hash (file `images/<hash>.png` in the app data folder), or the file paths, one per line */
  content: string
  /** Image size in pixels */
  width: number | null
  height: number | null
  /** Size of the saved image file, or total size of the copied files, in bytes */
  byteSize: number | null
  /** Display name of the app the clip was copied from, e.g. "Google Chrome" */
  sourceAppName: string | null
  /** Executable path of that app */
  sourceAppPath: string | null
  /** Unix timestamp in milliseconds */
  createdAt: number
  /** Starred by the user: kept forever, and listed by the favorites filter */
  favorite: boolean
}

/** Delay after the last keystroke before searching, so typing stays smooth */
const SEARCH_DELAY_MS = 120

/**
 * Clipboard history, newest first, kept in sync with every new copy, or the clips of
 * `collection`, last added first. `clips` only holds the clips matching `query` while it isn't empty.
 */
export function useClips(
  query: MaybeRefOrGetter<string> = '',
  collection: MaybeRefOrGetter<number | null> = null,
  /** Shows only the favorite clips of the history or collection */
  favoritesOnly: MaybeRefOrGetter<boolean> = false
) {
  const history = ref<Clip[]>([])
  // Tab counts change with every deletion and collection change
  const { refresh: refreshCounts } = useCollections()
  /**
   * Clips of every collection opened (or hovered) so far: opening it again shows them
   * right away, while they are reloaded in the background
   */
  const collectionCache = shallowReactive(new Map<number, Clip[]>())
  /**
   * Collection whose clips are shown. Follows `collection` once its clips are loaded,
   * so the previous tab stays on screen meanwhile instead of an empty one
   */
  const shownCollection = ref<number | null>(toValue(collection))
  /** Clips of the shown collection, `null` on the history */
  const collectionClips = computed(() =>
    shownCollection.value === null ? null : collectionCache.get(shownCollection.value) ?? null
  )
  /** Applies a change to the cached clips of every collection */
  const updateCache = (change: (clips: Clip[]) => Clip[]) => {
    for (const [id, clips] of collectionCache) collectionCache.set(id, change(clips))
  }
  /** Clips matching the query, `null` when not searching */
  const results = ref<Clip[] | null>(null)
  // The history and the open collection are listed apart: the history grid stays alive
  // while a collection is open, so coming back to it is instant
  const historyClips = computed(() => (shownCollection.value === null && results.value) || history.value)
  const openCollectionClips = computed(() =>
    shownCollection.value === null ? [] : results.value ?? collectionClips.value ?? []
  )
  /** Clips of the shown tab */
  const clips = computed(() => (shownCollection.value === null ? historyClips.value : openCollectionClips.value))
  const unlisteners: UnlistenFn[] = []

  // Searched (or filtered to favorites) in the whole history or collection by the backend,
  // not only in the loaded clips
  let searchTimer: ReturnType<typeof setTimeout> | undefined
  let searchId = 0
  const search = (delay = SEARCH_DELAY_MS) => {
    clearTimeout(searchTimer)
    const current = toValue(query).trim()
    const favorites = toValue(favoritesOnly)
    if (!current && !favorites) {
      searchId++
      results.value = null
      return
    }
    searchTimer = setTimeout(async () => {
      const id = ++searchId
      const found = await invoke<Clip[]>('search_clips', { query: current, collection: toValue(collection), favorites })
      // A newer search started meanwhile: its results win
      if (id === searchId) results.value = found
    }, delay)
  }
  watch(() => toValue(query), () => search())
  // The favorites filter applies at once, no typing to wait for
  watch(() => toValue(favoritesOnly), () => search(0))

  const fetchCollection = async (id: number) => {
    const clips = await invoke<Clip[]>('list_clips', { collection: id })
    collectionCache.set(id, clips)
  }

  /** Loads a collection ahead, e.g. when its tab is hovered, so opening it is instant */
  const prefetch = (id: number) => {
    if (!collectionCache.has(id)) fetchCollection(id).catch(() => {})
  }

  // Opening a collection shows its cached clips at once (reloaded in the background),
  // or waits for them the first time. It also searches in it instead
  let loadId = 0
  const loadCollection = async () => {
    const id = ++loadId
    const current = toValue(collection)
    if (current === null || collectionCache.has(current)) {
      shownCollection.value = current
      if (current !== null) fetchCollection(current).catch(() => {})
      return
    }
    await fetchCollection(current)
    // Switched again meanwhile: the newer tab wins
    if (id === loadId) shownCollection.value = current
  }
  watch(() => toValue(collection), () => {
    loadCollection()
    if (results.value) search()
  })

  const onClipAdded = (clip: Clip) => {
    history.value = [clip, ...history.value.filter(c => c.id !== clip.id)]
    // In a collection, a clip copied again keeps its place, only its data changes
    updateCache(clips => clips.map(c => (c.id === clip.id ? clip : c)))
    // The new copy shows up in the results if it matches
    if (results.value) search()
  }

  // Clips unused for 30 days leave the history (collections aside), cleaned up by the backend
  const onClipsRemoved = (ids: number[]) => {
    const removed = new Set(ids)
    const keep = (list: Clip[]) => list.filter(c => !removed.has(c.id))
    history.value = keep(history.value)
    updateCache(keep)
    if (results.value) results.value = keep(results.value)
  }

  onMounted(async () => {
    // Listen before loading so a copy made in between is not lost
    unlisteners.push(
      await listen<Clip>('clip-added', ({ payload }) => onClipAdded(payload)),
      await listen<number[]>('clips-removed', ({ payload }) => onClipsRemoved(payload)),
      // Dropped files are saved by the backend, which alone gets their paths
      await listen<{ collection: number, clip: Clip }>('clip-dropped', ({ payload }) => onClipDropped(payload.clip, payload.collection))
    )
    const saved = await invoke<Clip[]>('list_clips')
    const live = history.value
    history.value = [...live, ...saved.filter(s => !live.some(c => c.id === s.id))]
    loadCollection()
  })

  onBeforeUnmount(() => {
    unlisteners.forEach(unlisten => unlisten())
    clearTimeout(searchTimer)
  })

  const without = (list: Clip[] | null, clip: Clip) => list && list.filter(c => c.id !== clip.id)

  /** Copies the clip again, ready to paste with Ctrl+V */
  const copy = (clip: Clip) => invoke('copy_clip', { id: clip.id })

  /** Deletes the clip from the history, and so from every collection */
  const remove = async (clip: Clip) => {
    await invoke('delete_clip', { id: clip.id })
    refreshCounts()
    history.value = without(history.value, clip)!
    updateCache(clips => without(clips, clip)!)
    results.value = without(results.value, clip)
  }

  /** Content dropped from another app into a collection: shown at the top of it */
  const onClipDropped = (clip: Clip, collectionId: number) => {
    refreshCounts()
    const cached = collectionCache.get(collectionId)
    if (cached) collectionCache.set(collectionId, [clip, ...without(cached, clip)!])
  }

  /** Text, or a picture's bytes, dropped from another app into a collection */
  const dropIntoCollection = async (collectionId: number, content: { text: string } | { image: Uint8Array }) => {
    const clip = 'text' in content
      ? await invoke<Clip>('drop_text', { collection: collectionId, text: content.text })
      : await invoke<Clip>('drop_image', { collection: collectionId, bytes: Array.from(content.image) })
    onClipDropped(clip, collectionId)
  }

  const addToCollection = async (clip: Clip, collectionId: number) => {
    await invoke('add_to_collection', { collection: collectionId, clip: clip.id })
    refreshCounts()
    const cached = collectionCache.get(collectionId)
    if (cached) collectionCache.set(collectionId, [clip, ...without(cached, clip)!])
  }

  /** Takes the clip out of a collection, it stays in the history */
  const removeFromCollection = async (clip: Clip, collectionId: number) => {
    await invoke('remove_from_collection', { collection: collectionId, clip: clip.id })
    refreshCounts()
    const cached = collectionCache.get(collectionId)
    if (cached) collectionCache.set(collectionId, without(cached, clip)!)
    if (collectionId === shownCollection.value) results.value = without(results.value, clip)
  }

  /**
   * Moves a clip of the shown collection to the place of another one, live while it is dragged.
   * Saved by `saveCollectionOrder` once the drag is over
   */
  const moveInCollection = (clip: Clip, target: Clip) => {
    const id = shownCollection.value
    const clips = id === null ? undefined : collectionCache.get(id)
    if (id === null || !clips) return
    const from = clips.findIndex(c => c.id === clip.id)
    const to = clips.findIndex(c => c.id === target.id)
    if (from < 0 || to < 0 || from === to) return
    const moved = [...clips]
    moved.splice(to, 0, ...moved.splice(from, 1))
    collectionCache.set(id, moved)
  }

  const saveCollectionOrder = () => {
    const id = shownCollection.value
    const clips = id === null ? undefined : collectionCache.get(id)
    if (id !== null && clips) invoke('reorder_collection', { collection: id, clips: clips.map(c => c.id) })
  }

  /** Stars or unstars a clip, everywhere it is shown */
  const toggleFavorite = async (clip: Clip) => {
    const updated = await invoke<Clip>('set_favorite', { id: clip.id, favorite: !clip.favorite })
    const replace = (list: Clip[]) => list.map(c => (c.id === updated.id ? updated : c))
    history.value = replace(history.value)
    updateCache(replace)
    if (results.value) {
      // Unstarred while only favorites are shown: it leaves the list
      results.value = toValue(favoritesOnly) && !updated.favorite ? without(results.value, updated) : replace(results.value)
    }
    return updated
  }

  /** Whether `clips` holds search results */
  const searching = computed(() => results.value !== null)
  /** Whether the open collection is still loading */
  const loading = computed(() => toValue(collection) !== shownCollection.value)

  return {
    clips,
    historyClips,
    openCollectionClips,
    shownCollection: readonly(shownCollection),
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
  }
}
