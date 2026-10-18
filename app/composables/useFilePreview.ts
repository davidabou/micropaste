import { invoke } from '@tauri-apps/api/core'

export interface FilePreview {
  image: string | null
  thumbnail: boolean
  typeName: string | null
}

const previews = new Map<string, Promise<FilePreview | null>>()
const loadedPreviews = new Map<string, FilePreview>()

export function useFilePreview(path: MaybeRefOrGetter<string | null>, enabled: MaybeRefOrGetter<boolean> = true) {
  const preview = ref<FilePreview | null>(null)

  // Only a new path reloads: a refreshed clip with the same files keeps its preview
  watch(() => {
    const current = toValue(path)
    return current && (toValue(enabled) || loadedPreviews.has(current)) ? current : null
  }, async (current) => {
    preview.value = (current ? loadedPreviews.get(current) : null) ?? null
    if (!current || preview.value) return

    if (!previews.has(current)) {
      previews.set(current, invoke<FilePreview>('get_file_preview', { path: current }).catch(() => null))
    }
    const loaded = await previews.get(current)!
    // A missing file may be restored: only previews that worked are kept
    if (loaded?.image) loadedPreviews.set(current, loaded)
    else previews.delete(current)
    if (toValue(path) === current) preview.value = loaded
  }, { immediate: true })

  return preview
}
