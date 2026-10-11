import { invoke } from '@tauri-apps/api/core'

export interface FilePreview {
  /** PNG data URL, `null` if the file no longer exists */
  image: string | null
  /** Whether `image` is a thumbnail of the content (picture, video, PDF…), rather than the icon of the file type */
  thumbnail: boolean
}

// Shared by every card: each file is previewed at most once per session
const previews = new Map<string, Promise<FilePreview | null>>()

/**
 * Thumbnail or icon of a copied file, as the Explorer shows it, `null` while loading.
 * Nothing is loaded until `enabled` is true, e.g. once the card is on screen.
 */
export function useFilePreview(path: MaybeRefOrGetter<string | null>, enabled: MaybeRefOrGetter<boolean> = true) {
  const preview = ref<FilePreview | null>(null)

  watchEffect(async () => {
    const current = toValue(path)
    preview.value = null
    if (!current || !toValue(enabled)) return

    if (!previews.has(current)) {
      previews.set(current, invoke<FilePreview>('get_file_preview', { path: current }).catch(() => null))
    }
    const loaded = await previews.get(current)!
    if (toValue(path) === current) preview.value = loaded
  })

  return preview
}
