const UNITS = ['B', 'KB', 'MB', 'GB']

export function formatBytes(bytes: number): string {
  let value = bytes
  let unit = 0
  while (value >= 1024 && unit < UNITS.length - 1) {
    value /= 1024
    unit++
  }
  const digits = unit > 0 && value < 10 ? 1 : 0
  return `${value.toLocaleString('en-US', { maximumFractionDigits: digits })} ${UNITS[unit]}`
}
