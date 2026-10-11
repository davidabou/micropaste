// One clock shared by every component, refreshed every 30 seconds
const now = ref(Date.now())
let timer: ReturnType<typeof setInterval> | undefined

const MINUTE = 60_000
const HOUR = 60 * MINUTE
const DAY = 24 * HOUR

/** Formats a timestamp as "now", "36 min ago", "12 hr ago", "3 d ago"… */
export function formatTimeAgo(timestamp: number, from = Date.now()) {
  const elapsed = Math.max(0, from - timestamp)
  if (elapsed < MINUTE) return 'now'
  if (elapsed < HOUR) return `${Math.floor(elapsed / MINUTE)} min ago`
  if (elapsed < DAY) return `${Math.floor(elapsed / HOUR)} hr ago`
  if (elapsed < 7 * DAY) return `${Math.floor(elapsed / DAY)} d ago`
  if (elapsed < 30 * DAY) return `${Math.floor(elapsed / (7 * DAY))} wk ago`
  if (elapsed < 365 * DAY) return `${Math.floor(elapsed / (30 * DAY))} mo ago`
  return `${Math.floor(elapsed / (365 * DAY))} yr ago`
}

/** Relative time that stays up to date while the app is open. */
export function useTimeAgo(timestamp: MaybeRefOrGetter<number>) {
  if (import.meta.client && !timer) {
    timer = setInterval(() => (now.value = Date.now()), 30_000)
  }
  return computed(() => formatTimeAgo(toValue(timestamp), now.value))
}
