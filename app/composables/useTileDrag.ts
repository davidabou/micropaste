import type { Clip } from './useClips'

/** Whether a tile of the window is being dragged, shared by every tile and the window. */
export const tileDragging = ref(false)

/** Clip of the dragged tile, so a collection tab can take it when it is dropped there */
export const draggedClip = shallowRef<Clip | null>(null)
