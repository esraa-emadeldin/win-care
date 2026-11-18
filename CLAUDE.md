# CLAUDE.md

win-care: Windows PC care app. Rust + Tauri 2 + Vue 3/TS. Design and plan live in
`docs/superpowers/` and are the source of truth.

## Layout

- `crates/core`: scanners, `make_plan`, `apply`, guard. Pure Rust, tested on temp folders with
  fakes (`FileOps` trait). **Planning never writes; every write is in `apply`.**
- `crates/winsys`: Windows API implementations (Recycle Bin, registry, WMI, startup).
- `src-tauri`: commands, events, tray, settings. `ui/`: Vue frontend that only renders.

## Commands

`npm run check` (fmt, clippy -D warnings, cargo test, eslint, prettier, vue-tsc, vitest),
`npm run test:e2e` (Playwright, mocked backend), `npm run dev` (app).
Shells started before the Rust install need `RUSTUP_HOME`/`CARGO_HOME` set (user env vars).

## Safety rules (never break)

- No admin actions in v1; nothing elevates.
- Default is the Recycle Bin; never fall back to a permanent delete silently.
- Protected: Windows, Program Files, other users, Documents/Desktop/Pictures/Music/Videos.
  Never follow junctions/symlinks.
- No telemetry, no network except the opt-in update check.

## Habits

TDD: failing test first. Conventional one-line commits, no trailers.
