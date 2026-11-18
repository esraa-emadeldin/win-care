const UNITS = ['B', 'KB', 'MB', 'GB', 'TB']

/** Bytes as a short label with 1024-based units: 1932735283 -> "1.8 GB". */
export function formatBytes(n: number): string {
  if (n < 1024) return `${n} B`
  let value = n
  let unit = 0
  while (value >= 1024 && unit < UNITS.length - 1) {
    value /= 1024
    unit++
  }
  return `${value.toFixed(1)} ${UNITS[unit]}`
}
