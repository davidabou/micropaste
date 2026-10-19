import { invoke } from '@tauri-apps/api/core'

export interface FilePreview {
  image: string | null
  thumbnail: boolean
  typeName: string | null
  width: number | null
  height: number | null
  media: 'animation' | 'video' | 'audio' | null
}

const previews = new Map<string, Promise<FilePreview | null>>()
const loadedPreviews = new Map<string, FilePreview>()

const cacheKey = (path: string, large: boolean) => (large ? `large:${path}` : path)

/** `large` for the details panel: a bigger thumbnail, showing the tile's one while it loads. */
export function useFilePreview(path: MaybeRefOrGetter<string | null>, enabled: MaybeRefOrGetter<boolean> = true, large = false) {
  const preview = ref<FilePreview | null>(null)

  // Only a new path reloads: a refreshed clip with the same files keeps its preview
  watch(() => {
    const current = toValue(path)
    return current && (toValue(enabled) || loadedPreviews.has(cacheKey(current, large))) ? current : null
  }, async (current) => {
    if (!current) return (preview.value = null)
    const key = cacheKey(current, large)
    preview.value = loadedPreviews.get(key) ?? (large ? loadedPreviews.get(current) : null) ?? null
    if (loadedPreviews.has(key)) return

    if (!previews.has(key)) {
      previews.set(key, invoke<FilePreview>('get_file_preview', { path: current, large }).catch(() => null))
    }
    const loaded = await previews.get(key)!
    // A missing file may be restored: only previews that worked are kept
    if (loaded?.image) loadedPreviews.set(key, loaded)
    else previews.delete(key)
    // Keeps the tile's thumbnail if the larger one fails
    if (toValue(path) === current && (loaded?.image || !preview.value)) preview.value = loaded
  }, { immediate: true })

  return preview
}
