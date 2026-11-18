# win-care v1: design

Status: draft for approval · 2026-11-18 · working name `win-care` (final name open, see §10)

## 1. Intent and why

A free, open-source PC care app for Windows: it finds junk, frees disk space, removes apps
cleanly and keeps an eye on the PC's health, from a tray icon and one main window.

It takes the cleanup and tools ideas from [zc-care](https://github.com/ZoolCoder/zc-care)'s macOS
app (ZC Care), not its code: zc-care is macOS-only and its roadmap lists Windows as not planned.
The owner wants a real product for their own PCs and for everyday users, kept apart from coursework.
Machine setup (installing and managing developer apps) is a **separate, later project**.

Users: everyday, non-technical Windows users (students, families, office workers). They want a
faster, cleaner PC in one click, in plain words, and to trust that nothing important is removed.

## 2. Goals, non-goals, anti-goals

Goals for v1:

- Six screens, as in the approved mockup (§6): Smart Scan, tray popup, Applications,
  Performance, Review before deleting, Space map.
- Every destructive action is reviewable, logged, and undoable where Windows allows it.
- Installs in under a minute without admin rights; starts with Windows if the user wants.

Non-goals for v1: machine setup / app installs (separate project); app _updates_; browser
history/cookie cleaning; duplicate/similar-file finder; scheduled automatic cleaning; languages
other than English; ARM64 and 32-bit builds; macOS/Linux.

Anti-goals (would make it a failure even if it works):

- A registry "cleaner", driver updater, "RAM booster" or any scare-ware: fake urgency, inflated
  junk numbers, red alerts to push a click.
- Deleting anything the user did not see in a list first.
- Telemetry, accounts, ads, or any required network service.
- Looking like, or being named like, CleanMyMac/CleanMyPC or any existing product.

## 3. Constraints

- Windows 11, and Windows 10 22H2, x64. WebView2 is required (built into 11; the installer
  bootstraps it on 10).
- Rust for all system work; UI in the web view (decision §5).
- Runs as a normal user. **v1 has no admin actions at all** (builder-check finding, Claude's
  call): re-launching a per-user, user-writable exe elevated would let any user-level program
  swap it and gain admin, and on standard accounts the elevated process would be a different
  user with a different Recycle Bin and log. Admin-only features move to v2 with a per-machine
  install and a helper in Program Files.
- Fully offline. The only network use is the optional update check (§8).
- Open source, MIT licence (Claude's call, matches zc-care; owner can change it).
- Owner's quality rules apply: tests, lint and build in CI, small commits.

## 4. Safety model (applies to every feature)

Borrowed from zc-care's core rules:

1. **Scan never writes.** A scan only reads and returns a list of findings with paths and sizes.
2. **Plan, review, apply.** Any change is a _plan_: the exact items, sizes and the action for each.
   The Review screen shows it; nothing happens until the user presses the action button.
   Applying re-checks each item (still exists, same size and type, not in use) and skips and
   reports anything that changed since the scan.
3. **Recycle Bin by default.** Deletes go to the Recycle Bin. Permanent delete is a per-category
   setting for caches only, off by default, and labelled as permanent in the button text.
4. **Emptying the Recycle Bin is its own action**, never inside "Move to Recycle Bin", with a **Update after the final review: v1 ships without the permanent-delete setting; everything goes to the Recycle Bin. Planned for v0.2 with its own clearly labelled button.**
   plain "this cannot be undone" warning. (Fixes a flaw in the ChatGPT mockup.)
5. **Protected paths.** Never offered: `C:\Windows` (except listed temp/log folders),
   `Program Files` (except leftovers of an app the user just uninstalled), other users' profiles,
   and anything under Documents, Desktop, Pictures, Music, Videos. The Downloads folder is shown
   for review but never pre-selected.
6. **Files in use are skipped**, not forced, and reported ("12 files skipped: in use").
7. **Activity log.** Every apply is written to a local log (what, size, where it went), shown in
   the app, with "Open Recycle Bin" to restore.
8. **Honest numbers.** Sizes are real bytes on disk, measured, never estimated upward.
9. **Nothing is ever deleted permanently by accident.** Before apply, each item is checked
   against its drive's Recycle Bin (bin turned off, or item larger than the bin's maximum size).
   Such items are marked in Review ("too large for the Recycle Bin") and skipped and reported on
   apply. The only permanent deletes are the explicit ones: the cache setting in §4.3 and
   "Empty Recycle Bin" in §4.4. The shell operation must never be allowed to fall back to a
   permanent delete silently.

## 5. Approaches (decided)

| Option                                | Trade-off                                                                                                                                                  |
| ------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **A. Rust + Tauri 2** (chosen)        | Rust core does the work; UI is HTML/CSS in WebView2. Mockups map directly to HTML/CSS; tray, autostart and MSI/NSIS installers are built in; ~5–10 MB app. |
| B. Rust + native WinUI (`windows-rs`) | Most native look, but Rust WinUI support is immature; slow to build and hard to match the mockups.                                                         |
| C. C# + WinUI 3                       | Best Windows tooling, but not Rust, which the owner chose.                                                                                                 |

The owner chose Rust; Claude chose Tauri within that, the owner did not object.
Frontend framework inside Tauri: **Vue 3 + TypeScript + Vite** (Claude's call: matches the
owner's default web stack and quality setup: ESLint, Prettier, Playwright).

## 6. Features (v1)

Visual reference: the approved mockup board
`design/chatgpt/v1-mockups/images/01.png` (direction "Windows-native, calm, clear and
reassuring", tokens in §7). Light and dark themes follow the Windows setting.

### 6.1 Smart Scan (home)

One scan across four cards: **Cleanup** (bytes of junk), **Apps** (count of large apps not used
recently, when "last used" is known), **Startup** (items that start with Windows), **Health**
(problems found). Each card has "View details". One primary **Clean** button opens Review with
the Cleanup findings pre-selected by the rules in §4. A full scan should finish in under
30 seconds on a typical SSD PC; progress is shown per card.

### 6.2 Cleanup categories

Each category is a scanner with a name, a plain description, the paths it reads, and whether it
needs admin:

- User temp (`%TEMP%`), files older than 24 h.
- Browser caches only (Edge, Chrome, Firefox, Brave: cache folders, not history, cookies or
  passwords). Skipped with a note while the browser is running.
- Thumbnail and icon cache (`%LOCALAPPDATA%\Microsoft\Windows\Explorer\thumbcache_*`).
- Crash dumps and Windows Error Reporting reports in the user's profile
  (`%LOCALAPPDATA%\CrashDumps`, `%LOCALAPPDATA%\Microsoft\Windows\WER`).
- v2 (needs admin, so not in v1): `C:\Windows\Temp`, system WER reports, Windows Update
  download cache.
- Old installers in Downloads (`.exe`, `.msi`, older than 30 days): shown, never pre-selected.
- Recycle Bin: shown as a size, cleared only through its own action (§4.4).

### 6.3 Tray popup

Tray icon with a small popup: CPU %, RAM %, system disk used %, battery % (if present),
network throughput, each with a short sparkline; one recommendation (the single most useful
finding from the last scan, or none); **Open app**. Sampling every 2 s only while the popup is
open; every 60 s in the background for alerts (low disk space below 10 % free, by default).
Closing the main window leaves the tray running; Quit is in the tray menu.

### 6.4 Applications

Lists installed apps from the Windows uninstall registry keys (HKLM, HKLM WOW6432Node, HKCU)
and MSIX/Store apps. Columns: name, publisher, size, last used (shown only when Windows records
it; otherwise "unknown", never guessed). Search and sort by size/name/last used.
**Uninstall** runs the app's own uninstaller (or removes the MSIX package), waits for it, then
scans for leftovers (folders named for the app/publisher in Program Files, ProgramData,
AppData) and opens Review with them. Leftover **registry keys are reported only, never deleted**
in v1.

### 6.5 Performance

- **Startup apps**: listed from the HKCU and HKLM Run keys, the user and common Startup
  folders, and logon scheduled tasks. **Only per-user items get a switch** (HKCU Run and the
  user Startup folder), using the same `StartupApproved` values Task Manager uses, so it is
  reversible and Task Manager shows the same state. Machine-wide items (HKLM, common Startup
  folder) are shown read-only with "needs admin, open Task Manager"; scheduled tasks are shown
  read-only with "Open Task Scheduler" (Task Manager does not manage them either). Shows
  publisher where known.
- **Maintenance tasks**, each with a one-line explanation and its own Run button, none needing
  admin: restart Windows Explorer; empty Recycle Bin (permanent, §4.4). Flush DNS cache moved to v2: ipconfig /flushdns needs admin on current Windows 11 (found in Task 7). The mockup's "Free Up RAM" and "Repair file associations" are **dropped**:
  the first kills user programs or does nothing useful, the second is vague.
- **Health checks** (read-only, each with a "How to fix" that opens the right Settings page):
  free space on each fixed drive; disk health status (from Windows storage APIs); Defender
  real-time protection on; firewall on for all profiles; pending Windows Update restart;
  battery wear (design vs full-charge capacity) on laptops.

### 6.6 Review before deleting

Shared by Cleanup, Space map and app leftovers. Groups (category or folder) with size,
item count, checkbox and expand to file level. Shows the total and the action:
"Move to Recycle Bin (3.3 GB)". A "Back" button changes nothing. After apply: a result summary
(moved, skipped and why, freed) and a link to the activity log.

### 6.7 Space map

Pick a fixed drive (default C:). Shows a treemap of folders by size plus a donut summary
(Windows, Apps, Users, Other). Drill into folders, "Open in Explorer", and "Review for deletion"
which sends the chosen items to Review (§4 protected paths still apply). Scanning streams
results so the map fills in progressively; it can be cancelled.

### 6.8 Settings and About

Start with Windows (on/off), theme (system/light/dark), permanent delete for caches (off),
low-disk alert threshold, check for updates (on/off), open activity log, licence, version.

## 7. Design tokens (from the mockup)

Primary `#2563EB`, accent `#06B6D4`, success `#16A34A`, warning `#F59E0B`, error `#EF4444`,
surface `#FFFFFF`, surface alt `#F3F6FF`, text `#0F172A`, text secondary `#64748B`.
Font Segoe UI Variable; 8 px spacing unit; 12 px corner radius. Dark theme values are to be
derived by the builder (same roles, WCAG AA contrast). Mica backdrop on Windows 11 where Tauri
supports it, solid surface on 10.

## 8. Architecture, install and update

```
win-care/
  crates/core      scanners, planners, appliers; no UI; unit-tested against temp dirs
  crates/winsys    thin wrappers over Windows APIs (registry, shell file ops, WMI, startup)
  src-tauri/       Tauri app: commands, events, tray, autostart, updater
  ui/              Vue 3 + TS + Vite frontend
```

- `core` keeps zc-care's split: **planning performs no writes; every write lives in an apply
  step**. Scans run off the UI thread and stream progress events; every scan is cancellable.
- Deleting to the Recycle Bin uses the Windows shell file operation with undo.
- Elevation: none in v1 (§3). v2 design: per-machine install in Program Files with a separate
  helper exe there, applying one saved plan from a protected location and exiting.
- Install: **one installer in v1, the NSIS setup.exe in per-user (`currentUser`) mode**, no
  admin needed. The MSI is deferred to v2 (Tauri's WiX MSI is per-machine and needs admin).
  Development builds are unsigned. **Code signing is required before the first public release**
  (finding on 2026-11-18): Windows 11 Smart App Control blocks unsigned, unknown executables
  outright, not just with a SmartScreen warning, so an unsigned win-care would not start at
  all on PCs that have it on. Options to evaluate: SignPath Foundation (free for OSS), a
  low-cost open-source code-signing certificate, Azure Artifact Signing where eligible. A winget manifest follows the first public release.
- Update: Tauri updater against GitHub Releases, NSIS bundle, signed update manifests; check at
  most once a day, only if enabled; the user confirms every update.
- Data: settings and activity log as JSON in `%APPDATA%\<app>`; nothing leaves the PC.

## 9. Quality

- `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`; ESLint, Prettier, `vue-tsc`,
  Playwright for the UI against mocked Tauri commands.
- Scanners are tested against generated temp folder trees, including the failure cases: file in
  use, file changed between scan and apply, symlink/junction pointing outside the target,
  protected path.
- GitHub Actions on `windows-latest`: lint, test, build both installers; Dependabot; a
  pre-commit hook for fmt and lint.

## 10. Decided vs left to the builder

Decided by the owner: Windows product, separate from zc-care; cleanup + tools scope; machine
setup is a separate project; Rust; open source; the mockup direction; audience "anyone's PC";
lives in `D:\Development` (not `D:\Study`).

Claude's calls (owner may override): Tauri 2; Vue 3 + TS; MIT; the cleanup category list;
dropping Free Up RAM and Repair file associations; registry leftovers report-only; per-user
install; unsigned v1. After the builder check: no admin actions in v1; NSIS-only installer;
machine-wide startup items and scheduled tasks read-only; items the Recycle Bin cannot take are
skipped and reported, never deleted permanently. The owner approved the design on 2026-11-18
and asked Claude to proceed through all steps autonomously.

Left to the builder: crate choices (e.g. `sysinfo`, `windows` crate features, treemap library),
exact scan heuristics within the rules above, dark-theme values, icon set, file layout inside the
crates, and the order of implementation.

Open: **the product name** (avoid "CleanMy*" and the common "Nexa"); needed before the first
public release, not before coding (the working name is used until then).
