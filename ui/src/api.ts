// Typed wrappers for every Tauri command. The UI never calls invoke directly.
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type {
  Action,
  ActivityEntry,
  App,
  ApplyReport,
  Check,
  Drive,
  Finding,
  Plan,
  ScanProgress,
  Settings,
  SmartScan,
  SpaceNode,
  StartupItem,
  Vitals
} from './types'

export const api = {
  smartScan: () => invoke<SmartScan>('smart_scan'),
  makePlan: (paths: string[], action: Action = 'recycle') =>
    invoke<Plan>('make_plan', { paths, action }),
  applyPlan: (planId: string) => invoke<ApplyReport>('apply_plan', { planId }),
  listApps: () => invoke<App[]>('list_apps'),
  uninstallApp: (id: string) => invoke<Finding[]>('uninstall_app', { id }),
  listStartup: () => invoke<StartupItem[]>('list_startup'),
  setStartup: (id: string, enabled: boolean) => invoke<void>('set_startup', { id, enabled }),
  health: () => invoke<Check[]>('health'),
  runMaintenance: (task: 'restartExplorer' | 'emptyRecycleBin') =>
    invoke<string>('run_maintenance', { task }),
  recycleBinSize: () => invoke<number>('recycle_bin_size'),
  drives: () => invoke<Drive[]>('drives'),
  spaceScan: (root: string) => invoke<SpaceNode>('space_scan', { root }),
  cancelSpaceScan: () => invoke<void>('cancel_space_scan'),
  vitals: () => invoke<Vitals>('vitals'),
  getSettings: () => invoke<Settings>('get_settings'),
  setSettings: (settings: Settings) => invoke<void>('set_settings', { settings }),
  activity: (limit: number) => invoke<ActivityEntry[]>('activity', { limit }),
  openTarget: (target: string) => invoke<void>('open_target', { target }),
  revealPath: (path: string) => invoke<void>('reveal_path', { path }),
  checkUpdate: () => invoke<string | null>('check_update'),
  installUpdate: () => invoke<void>('install_update')
}

export function onScanProgress(cb: (p: ScanProgress) => void): Promise<UnlistenFn> {
  return listen<ScanProgress>('scan-progress', (e) => cb(e.payload))
}

export function onSpacePartial(cb: (n: SpaceNode) => void): Promise<UnlistenFn> {
  return listen<SpaceNode>('space-partial', (e) => cb(e.payload))
}
