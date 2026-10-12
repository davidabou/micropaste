import type { Clip } from './useClips'

export const tileDragging = ref(false)

export const draggedClip = shallowRef<Clip | null>(null)
