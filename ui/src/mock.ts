// A fake backend for browser tests and UI work without Rust: `vite --mode mock`.
// The data is chosen to hit the interesting cases (file in use, installer not
// pre-selected, item too large for the Recycle Bin).
import { mockIPC } from '@tauri-apps/api/mocks'
import type {
  ActivityEntry,
  App,
  ApplyReport,
  Check,
  Finding,
  Plan,
  Settings,
  SmartScan,
  SpaceNode,
  StartupItem
} from './types'

const MB = 1024 * 1024
const GB = 1024 * MB
const TEMP = 'C:\\Users\\demo\\AppData\\Local\\Temp'

function finding(
  path: string,
  bytes: number,
  category: Finding['category'],
  extra: Partial<Finding> = {}
): Finding {
  return {
    id: path.toLowerCase(),
    category,
    path,
    bytes,
    isDir: false,
    modifiedMs: 0,
    preselected: true,
    selectable: true,
    note: null,
    ...extra
  }
}

const findings: Finding[] = [
  finding(`${TEMP}\\old-build.tmp`, 200 * MB, 'userTemp'),
  finding(`${TEMP}\\locked.log`, 10 * MB, 'userTemp'),
  finding(
    'C:\\Users\\demo\\AppData\\Local\\Google\\Chrome\\User Data\\Default\\session-cache',
    300 * MB,
    'browserCache',
    { isDir: true }
  ),
  finding('C:\\Users\\demo\\AppData\\Local\\CrashDumps\\huge.dmp', 3 * GB, 'crashDumps'),
  finding('C:\\Users\\demo\\Downloads\\setup-old.exe', 50 * MB, 'oldInstallers', {
    preselected: false
  })
]

const health: Check[] = [
  {
    id: 'disk-c',
    title: 'Free space on C:',
    status: 'ok',
    detail: '120.4 GB free of 476.3 GB',
    fixUri: 'ms-settings:storagesense'
  },
  {
    id: 'antivirus',
    title: 'Antivirus protection',
    status: 'ok',
    detail: 'On: Microsoft Defender',
    fixUri: 'windowsdefender:'
  },
  {
    id: 'restart',
    title: 'Windows Update',
    status: 'warn',
    detail: 'Restart to finish installing updates',
    fixUri: 'ms-settings:windowsupdate'
  }
]

const apps: App[] = [
  {
    id: 'hklm:chrome',
    name: 'Google Chrome',
    publisher: 'Google LLC',
    version: '131.0',
    bytes: 1.5 * GB,
    lastUsedMs: null,
    kind: {}
  },
  {
    id: 'hkcu:contoso',
    name: 'Contoso Notes',
    publisher: 'Contoso',
    version: '2.1',
    bytes: 120 * MB,
    lastUsedMs: null,
    kind: {}
  },
  {
    id: 'msix:spotify',
    name: 'Spotify',
    publisher: 'Spotify AB',
    version: '1.2',
    bytes: null,
    lastUsedMs: null,
    kind: {}
  }
]

const startup: StartupItem[] = [
  {
    id: 'hkcurun:spotify',
    name: 'Spotify',
    command: 'Spotify.exe /minimized',
    publisher: null,
    source: 'hkcuRun',
    enabled: true,
    canToggle: true
  },
  {
    id: 'hklmrun:securityhealth',
    name: 'SecurityHealth',
    command: 'SecurityHealthSystray.exe',
    publisher: null,
    source: 'hklmRun',
    enabled: true,
    canToggle: false
  },
  {
    id: 'task:\\updater',
    name: 'Updater',
    command: '\\Updater',
    publisher: null,
    source: 'task',
    enabled: true,
    canToggle: false
  }
]

const space: SpaceNode = {
  name: 'C:\\',
  path: 'C:\\',
  bytes: 300 * GB,
  children: [
    { name: 'Windows', path: 'C:\\Windows', bytes: 120 * GB, children: [] },
    {
      name: 'Users',
      path: 'C:\\Users',
      bytes: 100 * GB,
      children: [{ name: 'demo', path: 'C:\\Users\\demo', bytes: 100 * GB, children: [] }]
    },
    { name: 'Program Files', path: 'C:\\Program Files', bytes: 70 * GB, children: [] },
    { name: 'Other', path: '', bytes: 10 * GB, children: [] }
  ]
}

let settings: Settings = {
  startWithWindows: false,
  theme: 'system',
  permanentDeleteCaches: false,
  lowDiskAlertPct: 10,
  checkUpdates: false
}

const activity: ActivityEntry[] = [
  {
    atMs: Date.UTC(2026, 10, 17, 9),
    action: 'recycle',
    done: 4,
    freedBytes: 900 * MB,
    skipped: 0,
    paths: []
  }
]

let plans: Record<string, Plan> = {}

/** Space map picks become findings, like finding_for_path in Rust. */
function spacePicks(paths: string[]): Finding[] {
  const out: Finding[] = []
  const visit = (n: SpaceNode) => {
    if (n.path && paths.includes(n.path))
      out.push(finding(n.path, n.bytes, 'spaceMap', { isDir: true }))
    n.children.forEach(visit)
  }
  visit(space)
  return out
}
let leftovers: Finding[] = []

function all(): Finding[] {
  return [...findings, ...leftovers]
}

type Args = Record<string, unknown>

const handlers: Record<string, (a: Args) => unknown> = {
  smart_scan: (): SmartScan => ({
    cleanup: { findings, notes: [], unreadableDirs: 2 },
    cleanupBytes: findings.filter((f) => f.preselected).reduce((s, f) => s + f.bytes, 0),
    appsCount: apps.length,
    appsBytes: apps.reduce((s, a) => s + (a.bytes ?? 0), 0),
    startupCount: startup.filter((s) => s.enabled).length,
    health
  }),
  make_plan: (a): Plan => {
    const paths = a.paths as string[]
    // Like the real guard: only things inside the user's profile.
    const allowed = (p: string) => p.startsWith('C:\\Users\\demo\\')
    const known = [...all(), ...spacePicks(paths)]
    const rejected = known
      .filter((f) => paths.includes(f.path) && f.selectable && !allowed(f.path))
      .map((f) => ({ path: f.path, reason: 'protected' as const }))
    const items = known
      .filter((f) => paths.includes(f.path) && f.selectable && allowed(f.path))
      .map((f) => ({ finding: f, recyclable: !f.path.includes('huge') }))
    const plan: Plan = {
      id: `plan-${Object.keys(plans).length}`,
      action: 'recycle',
      items,
      totalBytes: items.reduce((s, i) => s + i.finding.bytes, 0),
      rejected
    }
    plans[plan.id] = plan
    return plan
  },
  apply_plan: (a): ApplyReport => {
    const plan = plans[a.planId as string]!
    const report: ApplyReport = { done: [], freedBytes: 0, skipped: [] }
    for (const { finding: f } of plan.items) {
      if (f.path.endsWith('locked.log')) report.skipped.push({ path: f.path, reason: 'inUse' })
      else {
        report.done.push(f.path)
        report.freedBytes += f.bytes
      }
    }
    return report
  },
  list_apps: () => apps,
  uninstall_app: () => {
    leftovers = [
      finding('C:\\Users\\demo\\AppData\\Roaming\\Contoso Notes', 40 * MB, 'appLeftovers', {
        isDir: true
      }),
      finding('C:\\Program Files\\Contoso Notes', 5 * MB, 'appLeftovers', {
        isDir: true,
        selectable: false,
        preselected: false,
        note: 'Needs admin; not removed in v1'
      })
    ]
    return leftovers
  },
  list_startup: () => startup,
  set_startup: (a) => {
    const item = startup.find((s) => s.id === a.id)
    if (item) item.enabled = a.enabled as boolean
  },
  health: () => health,
  run_maintenance: (a) =>
    a.task === 'emptyRecycleBin' ? 'Recycle Bin emptied' : 'Windows Explorer restarted',
  recycle_bin_size: () => 1.2 * GB,
  drives: () => [
    { letter: 'C', total: 476 * GB, free: 120 * GB },
    { letter: 'D', total: 931 * GB, free: 400 * GB }
  ],
  space_scan: () => space,
  cancel_space_scan: () => undefined,
  vitals: () => ({
    cpuPct: 12 + Math.random() * 10,
    ramUsed: (5.6 + Math.random() * 0.8) * GB,
    ramTotal: 16 * GB,
    diskUsed: 356 * GB,
    diskTotal: 476 * GB,
    batteryPct: 87,
    charging: false,
    netRxBps: 24 * 1024,
    netTxBps: 3 * 1024
  }),
  get_settings: () => settings,
  set_settings: (a) => {
    settings = a.settings as Settings
  },
  activity: () => activity,
  open_target: () => undefined,
  reveal_path: () => undefined,
  check_update: () => null,
  install_update: () => undefined
}

export function installMock(): void {
  plans = {}
  mockIPC(
    (cmd, payload) => {
      const handler = handlers[cmd]
      if (!handler) throw new Error(`mock: no handler for ${cmd}`)
      return handler((payload ?? {}) as Args)
    },
    { shouldMockEvents: true }
  )
}
