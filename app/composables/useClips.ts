import type { UnlistenFn } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

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
  inHistory: boolean
}

export interface PartialCopy {
  copied: number
  total: number
}

export type Tab = 'history' | 'favorites' | number
type ListTab = Exclude<Tab, 'history'>

export type ClipKind = 'text' | 'link' | 'color' | 'image' | 'files'

export interface ClipFilter {
  kind: ClipKind | null
  source: string | null
}

export function copiedText(partial: PartialCopy | null) {
  return partial ? `Copied (${partial.copied}/${partial.total})` : 'Copied'
}

const SEARCH_DELAY_MS = 120

export const scopeOf = (tab: Tab) => (tab === 'history' ? {} : tab === 'favorites' ? { favorites: true } : { collection: tab })

export function useClips(
  query: MaybeRefOrGetter<string> = '',
  tab: MaybeRefOrGetter<Tab> = 'history',
  filter: MaybeRefOrGetter<ClipFilter> = { kind: null, source: null }
) {
  const history = ref<Clip[]>([])
  const historyLoading = ref(true)
  const { refresh: refreshCounts } = useCollections()
  const listCache = shallowReactive(new Map<ListTab, Clip[]>())
  const shownTab = ref<Tab>(toValue(tab))
  const cachedClips = computed(() => (shownTab.value === 'history' ? null : listCache.get(shownTab.value) ?? null))
  const updateCache = (change: (clips: Clip[]) => Clip[]) => {
    for (const [id, clips] of listCache) listCache.set(id, change(clips))
  }
  // History stays mounted beside the other tabs: its results are kept up to date apart
  const historyResults = ref<Clip[] | null>(null)
  const results = ref<Clip[] | null>(null)
  const historyClips = computed(() => historyResults.value ?? history.value)
  const tabClips = computed(() => (shownTab.value === 'history' ? [] : results.value ?? cachedClips.value ?? []))
  const unlisteners: UnlistenFn[] = []

  const searchIn = (shown: Tab) =>
    invoke<Clip[]>('search_clips', { query: toValue(query).trim(), ...scopeOf(shown), filter: toValue(filter) })
  const filtering = () => {
    const { kind, source } = toValue(filter)
    return toValue(query).trim() !== '' || kind !== null || source !== null
  }

  let searchId = 0
  const runSearch = async () => {
    const id = ++searchId
    const current = toValue(tab)
    const [inHistory, found] = await Promise.all([searchIn('history'), current === 'history' ? null : searchIn(current)])
    if (id !== searchId) return
    historyResults.value = inHistory
    // Left meanwhile: the new tab loads its own results
    if (toValue(tab) === current) results.value = found
  }
  const { start: searchLater, stop: cancelSearch } = useTimeoutFn(runSearch, SEARCH_DELAY_MS, { immediate: false })
  // The delay spares a search per typed letter: a filter is picked in one go
  const search = (now = false) => {
    cancelSearch()
    if (!filtering()) {
      searchId++
      historyResults.value = null
      results.value = null
      return
    }
    if (now) runSearch()
    else searchLater()
  }
  watch(() => toValue(query), () => search())
  watch(() => [toValue(filter).kind, toValue(filter).source], () => search(true))

  const fetchList = async (id: ListTab) => {
    const clips = await invoke<Clip[]>('list_clips', scopeOf(id))
    listCache.set(id, clips)
  }

  const prefetch = (id: ListTab) => {
    if (!listCache.has(id)) fetchList(id).catch(() => {})
  }

  let loadId = 0
  const loadTab = async () => {
    const id = ++loadId
    const current = toValue(tab)
    // The tab shows up with its own results, not the ones of the previous tab
    if (filtering() && current !== 'history') {
      // Ready for when the filters are cleared
      prefetch(current)
      const resultsId = searchId
      const found = await searchIn(current).catch(() => [])
      if (id !== loadId) return
      if (resultsId === searchId) results.value = found
      shownTab.value = current
      return
    }
    if (current === 'history' || listCache.has(current)) {
      shownTab.value = current
      if (current !== 'history') fetchList(current).catch(() => {})
      return
    }
    await fetchList(current)
    if (id === loadId) shownTab.value = current
  }
  watch(() => toValue(tab), () => {
    loadTab()
  })

  const onClipAdded = (clip: Clip) => {
    history.value = [clip, ...history.value.filter(c => c.id !== clip.id)]
    updateCache(clips => clips.map(c => (c.id === clip.id ? clip : c)))
    if (filtering()) search()
  }

  const onClipsRemoved = (ids: number[]) => {
    const removed = new Set(ids)
    const keep = (list: Clip[]) => list.filter(c => !removed.has(c.id))
    history.value = keep(history.value)
    updateCache(keep)
    if (historyResults.value) historyResults.value = keep(historyResults.value)
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
    loadTab()
  })

  onBeforeUnmount(() => unlisteners.forEach(unlisten => unlisten()))

  const without = (list: Clip[] | null, clip: Clip) => list && list.filter(c => c.id !== clip.id)

  const copy = (clip: Clip) => invoke<PartialCopy | null>('copy_clip', { id: clip.id })

  const remove = async (clip: Clip) => {
    await invoke('delete_clip', { id: clip.id })
    refreshCounts()
    history.value = without(history.value, clip)!
    updateCache(clips => without(clips, clip)!)
    historyResults.value = without(historyResults.value, clip)
    results.value = without(results.value, clip)
  }

  const onClipDropped = (clip: Clip, collectionId: number) => {
    const cached = listCache.get(collectionId)
    if (cached?.some(c => c.id === clip.id)) {
      listCache.set(collectionId, cached.map(c => (c.id === clip.id ? clip : c)))
      highlightClip(clip.id)
      return
    }
    refreshCounts()
    if (cached) listCache.set(collectionId, [clip, ...cached])
  }

  const dropIntoCollection = async (collectionId: number, content: { text: string } | { image: Uint8Array }) => {
    const clip = 'text' in content
      ? await invoke<Clip>('drop_text', { collection: collectionId, text: content.text })
      : await invoke<Clip>('drop_image', { collection: collectionId, bytes: Array.from(content.image) })
    onClipDropped(clip, collectionId)
  }

  /** Resolves to `false` when the clip was already in the collection. */
  const addToCollection = async (clip: Clip, collectionId: number) => {
    const cached = listCache.get(collectionId)
    const already = cached
      ? cached.some(c => c.id === clip.id)
      : (await invoke<number[]>('clip_collections', { clip: clip.id }).catch((): number[] => [])).includes(collectionId)
    if (already) return false
    await invoke('add_to_collection', { collection: collectionId, clip: clip.id })
    refreshCounts()
    if (cached) listCache.set(collectionId, [clip, ...cached])
    return true
  }

  const removeFromCollection = async (clip: Clip, collectionId: number) => {
    await invoke('remove_from_collection', { collection: collectionId, clip: clip.id })
    refreshCounts()
    const cached = listCache.get(collectionId)
    if (cached) listCache.set(collectionId, without(cached, clip)!)
    if (collectionId === shownTab.value) results.value = without(results.value, clip)
  }

  const moveInCollection = (clip: Clip, target: Clip) => {
    const id = shownTab.value
    const clips = typeof id === 'number' ? listCache.get(id) : undefined
    if (typeof id !== 'number' || !clips) return
    const from = clips.findIndex(c => c.id === clip.id)
    const to = clips.findIndex(c => c.id === target.id)
    if (from < 0 || to < 0 || from === to) return
    const moved = [...clips]
    moved.splice(to, 0, ...moved.splice(from, 1))
    listCache.set(id, moved)
  }

  const saveCollectionOrder = () => {
    const id = shownTab.value
    const clips = typeof id === 'number' ? listCache.get(id) : undefined
    if (typeof id === 'number' && clips) invoke('reorder_collection', { collection: id, clips: clips.map(c => c.id) })
  }

  const toggleFavorite = async (clip: Clip) => {
    const updated = await invoke<Clip>('set_favorite', { id: clip.id, favorite: !clip.favorite })
    const replace = (list: Clip[]) => list.map(c => (c.id === updated.id ? updated : c))
    history.value = replace(history.value)
    updateCache(replace)
    if (historyResults.value) historyResults.value = replace(historyResults.value)
    const favorites = listCache.get('favorites')
    if (favorites) listCache.set('favorites', updated.favorite ? [updated, ...without(favorites, updated)!] : without(favorites, updated)!)
    if (results.value) {
      results.value = shownTab.value === 'favorites' && !updated.favorite ? without(results.value, updated) : replace(results.value)
    }
    return updated
  }

  const searching = computed(() => historyResults.value !== null)
  const loading = computed(() => toValue(tab) !== shownTab.value)

  return {
    historyClips,
    tabClips,
    shownTab: readonly(shownTab),
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
