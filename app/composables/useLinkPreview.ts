import { invoke } from '@tauri-apps/api/core'

export interface LinkPreview {
  url: string
  title: string | null
  description: string | null
  siteName: string | null
  image: string | null
  icon: string | null
  stale: boolean
}

const loads = new Map<string, Promise<LinkPreview | null>>()
const refreshes = new Map<string, Promise<LinkPreview | null>>()
const shown = new Map<string, LinkPreview | null>()

function once(cache: Map<string, Promise<LinkPreview | null>>, command: string, url: string) {
  if (!cache.has(url)) {
    cache.set(url, invoke<LinkPreview | null>(command, { url }).catch(() => null))
  }
  return cache.get(url)!
}

export function useLinkPreview(url: MaybeRefOrGetter<string | null>, enabled: MaybeRefOrGetter<boolean> = true) {
  const preview = ref<LinkPreview | null>(null)

  watch(() => {
    const current = toValue(url)
    return current && (toValue(enabled) || shown.has(current)) ? current : null
  }, async (current) => {
    preview.value = (current ? shown.get(current) : null) ?? null
    if (!current || shown.has(current)) return

    const isCurrent = () => toValue(url) === current
    const cached = await once(loads, 'get_link_preview', current)
    if (!cached?.stale) shown.set(current, cached)
    if (!isCurrent()) return
    preview.value = cached

    if (cached?.stale) {
      const refreshed = await once(refreshes, 'refresh_link_preview', current)
      shown.set(current, refreshed ?? cached)
      if (isCurrent() && refreshed) preview.value = refreshed
    }
  }, { immediate: true })

  return preview
}
