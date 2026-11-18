// Types mirroring the Rust structs (serde camelCase). Keep in sync with
// crates/core and src-tauri/src/commands.rs.

export type CategoryId =
  | 'userTemp'
  | 'browserCache'
  | 'thumbnailCache'
  | 'crashDumps'
  | 'oldInstallers'
  | 'appLeftovers'
  | 'spaceMap'

export interface Finding {
  id: string
  category: CategoryId
  path: string
  bytes: number
  isDir: boolean
  modifiedMs: number
  preselected: boolean
  selectable: boolean
  note: string | null
}

export type SkipReason =
  | 'missing'
  | 'changed'
  | 'inUse'
  | 'protected'
  | 'notRecyclable'
  | 'needsAdmin'
  | { failed: string }

export type Action = 'recycle' | 'deletePermanently'

export interface PlanItem {
  finding: Finding
  recyclable: boolean
}

export interface Plan {
  id: string
  action: Action
  items: PlanItem[]
  totalBytes: number
  /** Picked paths the backend refused, with the reason. */
  rejected: Skipped[]
}

export interface Skipped {
  path: string
  reason: SkipReason
}

export interface ApplyReport {
  done: string[]
  freedBytes: number
  skipped: Skipped[]
}

export interface CleanupResult {
  findings: Finding[]
  notes: string[]
  unreadableDirs: number
}

export type CheckStatus = 'ok' | 'warn' | 'unknown'

export interface Check {
  id: string
  title: string
  status: CheckStatus
  detail: string
  fixUri: string | null
}

export interface SmartScan {
  cleanup: CleanupResult
  cleanupBytes: number
  appsCount: number
  appsBytes: number
  startupCount: number
  health: Check[]
}

export interface App {
  id: string
  name: string
  publisher: string | null
  version: string | null
  bytes: number | null
  lastUsedMs: number | null
  kind: unknown
}

export type StartupSource =
  'hkcuRun' | 'hklmRun' | 'hklm32Run' | 'userFolder' | 'commonFolder' | 'task'

export interface StartupItem {
  id: string
  name: string
  command: string
  publisher: string | null
  source: StartupSource
  enabled: boolean
  canToggle: boolean
}

export interface SpaceNode {
  name: string
  /** Empty for the folded "Other" node. */
  path: string
  bytes: number
  children: SpaceNode[]
}

export interface Drive {
  letter: string
  total: number
  free: number
}

export interface Vitals {
  cpuPct: number
  ramUsed: number
  ramTotal: number
  diskUsed: number
  diskTotal: number
  batteryPct: number | null
  charging: boolean | null
  netRxBps: number
  netTxBps: number
}

export interface Settings {
  startWithWindows: boolean
  theme: 'system' | 'light' | 'dark'
  permanentDeleteCaches: boolean
  lowDiskAlertPct: number
  checkUpdates: boolean
}

export interface ActivityEntry {
  atMs: number
  action: Action
  done: number
  freedBytes: number
  skipped: number
  paths: string[]
}

export interface ScanProgress {
  area: 'cleanup' | 'apps' | 'startup' | 'health'
  label: string
}
