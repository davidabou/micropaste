import type { UnlistenFn } from '@tauri-apps/api/event'
import type { Clip } from '@/composables/useClips'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

/** Text recognized in an image or a single image file, `null` until the background recognition is done. */
export function useImageText(clip: MaybeRefOrGetter<Clip | null>, enabled: MaybeRefOrGetter<boolean> = true) {
  const text = ref<string | null>(null)
  const imageId = () => {
    const current = toValue(clip)
    return current && current.kind !== 'text' && toValue(enabled) ? current.id : null
  }

  watch(imageId, async (id) => {
    text.value = null
    if (id === null) return
    const loaded = await invoke<string | null>('get_image_text', { id }).catch(() => '')
    if (imageId() !== id || text.value !== null) return
    text.value = loaded
  }, { immediate: true })

  let unlisten: Promise<UnlistenFn> | null = null
  onMounted(() => {
    unlisten = listen<{ id: number, text: string }>('image-text-recognized', ({ payload }) => {
      if (payload.id !== imageId()) return
      text.value = payload.text
    })
  })
  onBeforeUnmount(() => unlisten?.then(stop => stop()))

  return { text }
}
