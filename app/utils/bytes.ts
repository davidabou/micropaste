const UNITS = ['B', 'KB', 'MB', 'GB']

/** Formats a file size, e.g. "245 KB" or "1.2 MB". */
export function formatBytes(bytes: number): string {
  let value = bytes
  let unit = 0
  while (value >= 1024 && unit < UNITS.length - 1) {
    value /= 1024
    unit++
  }
  // One decimal below 10 ("1.2 MB"), none above ("245 KB")
  const digits = unit > 0 && value < 10 ? 1 : 0
  return `${value.toLocaleString('en-US', { maximumFractionDigits: digits })} ${UNITS[unit]}`
}
