// Shared state: the last Smart Scan and what is being reviewed. A plain
// reactive object is enough; no store library needed.
import { reactive } from 'vue'
import type { CategoryId, Finding, SmartScan } from '../types'

export interface ReviewItem {
  finding: Finding
  selected: boolean
  /** False when the drive's Recycle Bin cannot take it. */
  recyclable: boolean
  /** Why the backend refused it (e.g. "protected folder"); null if allowed. */
  blocked: string | null
}

export type ReviewSource = 'cleanup' | 'leftovers' | 'space'

export const store = reactive({
  scan: null as SmartScan | null,
  scanning: false,
  progress: '',
  review: {
    source: 'cleanup' as ReviewSource,
    findings: [] as Finding[]
  }
})

/** Review rows: pre-selected only if selectable and the bin can take it. */
export function reviewItems(
  findings: Finding[],
  unrecyclable: Set<string>,
  blocked: Map<string, string> = new Map()
): ReviewItem[] {
  return findings.map((f) => {
    const recyclable = !unrecyclable.has(f.path)
    const why = blocked.get(f.path) ?? null
    return {
      finding: f,
      recyclable,
      blocked: why,
      selected: f.selectable && f.preselected && recyclable && why === null
    }
  })
}

/** Bytes that "Move to Recycle Bin" would move. */
export function selectedTotal(items: ReviewItem[]): number {
  return items
    .filter((i) => i.selected && i.finding.selectable && i.recyclable && i.blocked === null)
    .reduce((sum, i) => sum + i.finding.bytes, 0)
}

export const CATEGORY_LABELS: Record<CategoryId, string> = {
  userTemp: 'Temporary files',
  browserCache: 'Browser caches',
  thumbnailCache: 'Thumbnail cache',
  crashDumps: 'Crash reports',
  oldInstallers: 'Old installers in Downloads',
  appLeftovers: 'App leftovers',
  spaceMap: 'Picked in Space Map'
}

export function startReview(source: ReviewSource, findings: Finding[]): void {
  store.review.source = source
  store.review.findings = findings
}

export function baseName(path: string): string {
  return path.split(/[\\/]/).filter(Boolean).pop() ?? path
}
