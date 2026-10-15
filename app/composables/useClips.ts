import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

export interface Clip {
  id: number
  kind: 'text' | 'image' | 'files'
  content: string
  width: number | null
  height: number | null
  byteSize: number | null
  sourceAppName: string | null
  sourceAppPath: string | null
  createdAt: number
  favorite: boolean
}

export interface PartialCopy {
  copied: number
  total: number
}

export const copiedText = (partial: PartialCopy | null) =>
  partial ? `Copied (${partial.copied}/${partial.total})` : 'Copied'

const SEARCH_DELAY_MS = 120

export function useClips(
  query: MaybeRefOrGetter<string> = '',
  collection: MaybeRefOrGetter<number | null> = null,
  favoritesOnly: MaybeRefOrGetter<boolean> = false
) {
  const history = ref<Clip[]>([])
  const historyLoading = ref(true)
  const { refresh: refreshCounts } = useCollections()
  const collectionCache = shallowReactive(new Map<number, Clip[]>())
  const shownCollection = ref<number | null>(toValue(collection))
  const collectionClips = computed(() =>
    shownCollection.value === null ? null : collectionCache.get(shownCollection.value) ?? null
  )
  const updateCache = (change: (clips: Clip[]) => Clip[]) => {
    for (const [id, clips] of collectionCache) collectionCache.set(id, change(clips))
  }
  const results = ref<Clip[] | null>(null)
  const historyClips = computed(() => (shownCollection.value === null && results.value) || history.value)
  const openCollectionClips = computed(() =>
    shownCollection.value === null ? [] : results.value ?? collectionClips.value ?? []
  )
  const clips = computed(() => (shownCollection.value === null ? historyClips.value : openCollectionClips.value))
  const unlisteners: UnlistenFn[] = []

  let searchId = 0
  const searchDelay = shallowRef(SEARCH_DELAY_MS)
  const { start: searchLater, stop: cancelSearch } = useTimeoutFn(async (current: string, favorites: boolean) => {
    const id = ++searchId
    const found = await invoke<Clip[]>('search_clips', { query: current, collection: toValue(collection), favorites })
    if (id === searchId) results.value = found
  }, searchDelay, { immediate: false })
  const search = (delay = SEARCH_DELAY_MS) => {
    cancelSearch()
    const current = toValue(query).trim()
    const favorites = toValue(favoritesOnly)
    if (!current && !favorites) {
      searchId++
      results.value = null
      return
    }
    searchDelay.value = delay
    searchLater(current, favorites)
  }
  watch(() => toValue(query), () => search())
  watch(() => toValue(favoritesOnly), () => search(0))

  const fetchCollection = async (id: number) => {
    const clips = await invoke<Clip[]>('list_clips', { collection: id })
    collectionCache.set(id, clips)
  }

  const prefetch = (id: number) => {
    if (!collectionCache.has(id)) fetchCollection(id).catch(() => {})
  }

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
    if (id === loadId) shownCollection.value = current
  }
  watch(() => toValue(collection), () => {
    loadCollection()
    if (results.value) search()
  })

  const onClipAdded = (clip: Clip) => {
    history.value = [clip, ...history.value.filter(c => c.id !== clip.id)]
    updateCache(clips => clips.map(c => (c.id === clip.id ? clip : c)))
    if (results.value) search()
  }

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
      await listen<{ collection: number, clip: Clip }>('clip-dropped', ({ payload }) => onClipDropped(payload.clip, payload.collection))
    )
    const saved = await invoke<Clip[]>('list_clips')
    const live = history.value
    history.value = [...live, ...saved.filter(s => !live.some(c => c.id === s.id))]
    historyLoading.value = false
    loadCollection()
  })

  onBeforeUnmount(() => unlisteners.forEach(unlisten => unlisten()))

  const without = (list: Clip[] | null, clip: Clip) => list && list.filter(c => c.id !== clip.id)

  const copy = (clip: Clip) => invoke<PartialCopy | null>('copy_clip', { id: clip.id })

  const remove = async (clip: Clip) => {
    await invoke('delete_clip', { id: clip.id })
    refreshCounts()
    history.value = without(history.value, clip)!
    updateCache(clips => without(clips, clip)!)
    results.value = without(results.value, clip)
  }

  const onClipDropped = (clip: Clip, collectionId: number) => {
    refreshCounts()
    const cached = collectionCache.get(collectionId)
    if (cached) collectionCache.set(collectionId, [clip, ...without(cached, clip)!])
  }

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

  const removeFromCollection = async (clip: Clip, collectionId: number) => {
    await invoke('remove_from_collection', { collection: collectionId, clip: clip.id })
    refreshCounts()
    const cached = collectionCache.get(collectionId)
    if (cached) collectionCache.set(collectionId, without(cached, clip)!)
    if (collectionId === shownCollection.value) results.value = without(results.value, clip)
  }

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

  const toggleFavorite = async (clip: Clip) => {
    const updated = await invoke<Clip>('set_favorite', { id: clip.id, favorite: !clip.favorite })
    const replace = (list: Clip[]) => list.map(c => (c.id === updated.id ? updated : c))
    history.value = replace(history.value)
    updateCache(replace)
    if (results.value) {
      results.value = toValue(favoritesOnly) && !updated.favorite ? without(results.value, updated) : replace(results.value)
    }
    return updated
  }

  const searching = computed(() => results.value !== null)
  const loading = computed(() => toValue(collection) !== shownCollection.value)

  return {
    clips,
    historyClips,
    openCollectionClips,
    shownCollection: readonly(shownCollection),
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
  }
}
