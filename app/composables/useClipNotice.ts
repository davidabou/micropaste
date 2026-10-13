export const clipNotice = shallowRef<{ clip: number, text: string } | null>(null)

export function notifyClip(clip: number, text: string) {
  clipNotice.value = { clip, text }
}
