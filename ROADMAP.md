# Roadmap

Where win-care is going. Plans change; the [issues](https://github.com/esraa-emadeldin/win-care/issues)
and [milestones](https://github.com/esraa-emadeldin/win-care/milestones) are the live view.
Ideas welcome — open a feature request.

## ✅ v0.1 — first release (now)

- Smart Scan, Review before deleting, Recycle Bin by default with no silent permanent delete
- Applications (uninstall + leftovers), Performance (startup, maintenance, health), Space Map
- Tray popup with live vitals, light/dark themes, start-up splash
- Per-user installer, no admin, opt-in update check

## 🔜 v0.2 — trust and reach

- **Code signing**, so Windows SmartScreen and Smart App Control accept win-care
- **A final name and logo** (win-care is a working name)
- Install with `winget install win-care`
- More safe cleanup categories inside the user's profile (app caches like Teams, Discord, Spotify)
- Translations: the UI text moves to language files — help wanted
- Accessibility pass: screen reader labels, keyboard-only use, high contrast

## 🧭 v0.3 — deeper cleaning, still safe

- An optional, separately installed **admin helper** in Program Files for tasks that need it:
  `C:\Windows\Temp`, the Windows Update cache, system error reports, Flush DNS.
  One UAC prompt per action, never a running elevated process.
- **Developer cleanup**: old `node_modules`, Gradle/Maven/npm/pip caches, Docker, old SDKs
- Duplicate and large-file finder
- "Last used" for apps, when Windows records it

## 💡 Later

- Scheduled scans with a notification ("2.1 GB to clean — review?"), never automatic deletes
- Battery and disk health history
- ARM64 builds
- A companion project for setting up a developer PC from one config file

## 🚫 Not planned

These would break the trust the app is built on:

- Registry "cleaners", "RAM boosters", driver updaters
- Deleting anything without showing it first
- Telemetry, accounts, ads or bundled offers
