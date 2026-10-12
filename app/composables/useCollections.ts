import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

export interface Collection {
  id: number
  name: string
  count: number
}

const collections = ref<Collection[]>([])
const historyCount = ref(0)
let loaded = false

async function refresh() {
  const [list, count] = await Promise.all([
    invoke<Collection[]>('list_collections'),
    invoke<number>('count_clips')
  ])
  collections.value = list
  historyCount.value = count
}

export function useCollections() {
  if (import.meta.client && !loaded) {
    loaded = true
    refresh()
    listen('clip-added', () => refresh())
    listen('clips-removed', () => refresh())
  }

  const create = async (name: string) => {
    const collection = await invoke<Collection>('create_collection', { name })
    collections.value = [...collections.value, collection]
    return collection
  }

  const rename = async (id: number, name: string) => {
    await invoke('rename_collection', { id, name })
    collections.value = collections.value.map(c => (c.id === id ? { ...c, name: name.trim() } : c))
  }

  const reorder = async (ordered: readonly Collection[]) => {
    collections.value = [...ordered]
    await invoke('reorder_collections', { ids: ordered.map(c => c.id) })
  }

  const remove = async (id: number) => {
    await invoke('delete_collection', { id })
    collections.value = collections.value.filter(c => c.id !== id)
  }

  return { collections: readonly(collections), historyCount: readonly(historyCount), create, rename, reorder, remove, refresh }
}
