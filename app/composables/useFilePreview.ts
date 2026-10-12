import { invoke } from '@tauri-apps/api/core'

export interface FilePreview {
  image: string | null
  thumbnail: boolean
}

const previews = new Map<string, Promise<FilePreview | null>>()

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
