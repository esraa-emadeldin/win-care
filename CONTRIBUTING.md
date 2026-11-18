# Contributing to win-care

Thank you for helping! This guide takes you from a fresh clone to a merged pull request.
Questions are welcome in [Discussions](https://github.com/esraa-emadeldin/win-care/discussions);
security problems go to [SECURITY.md](SECURITY.md), never to a public issue.

## Ways to help

- **Report a bug** with the [bug template](https://github.com/esraa-emadeldin/win-care/issues/new?template=bug_report.yml).
- **Suggest a feature** with the [feature template](https://github.com/esraa-emadeldin/win-care/issues/new?template=feature_request.yml)
  — before writing code, so we agree on the shape first.
- **Pick an issue** labelled `good first issue` or `help wanted`, and comment that you're on it.
- **Improve the docs**, screenshots or wording. Plain, friendly English is part of the product.
- **Add a cleanup category** — see the recipe below.

## Set up

You need **Windows 10/11**, [Rust](https://rustup.rs) (stable, MSVC toolchain, with the
"Desktop development with C++" build tools), [Node.js 24](https://nodejs.org) and git.

```powershell
gh repo fork esraa-emadeldin/win-care --clone
cd win-care
npm ci
npm run dev        # the app with hot reload
```

> **Smart App Control** (Windows 11) blocks every program a compiler produces, so Rust cannot
> build while it is on. A development PC needs it off: Windows Security → App & browser control
> → Smart App Control. It cannot be turned back on without resetting Windows, so decide
> deliberately — a VM is a fine alternative.

Work on the UI alone, no Rust needed, against a fake backend:

```powershell
npx vite ui --mode mock
```

## Everyday commands

```powershell
npm run check      # everything CI runs except browser tests: fmt, clippy, cargo test, eslint, prettier, types, vitest
npm run test:e2e   # Playwright UI tests against the mocked backend
npm run format     # prettier; `cargo fmt --all` for Rust
npx tauri build    # the installer, in target/release/bundle/nsis
```

A pre-commit hook checks formatting and types. Never skip it with `--no-verify`; fix the cause.

## Project layout

```
crates/core     scanners, the guard, make_plan/apply — pure Rust, tested on temp folders
crates/winsys   Windows APIs: Recycle Bin, registry, WMI, Task Scheduler, known folders
src-tauri       Tauri commands, settings, tray, updater
ui/             Vue 3 + TypeScript frontend (renders only; all work happens in Rust)
tests/e2e       Playwright tests
docs/           architecture, safety model, design spec and plans
```

[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) explains how the pieces fit.

## Making a change

1. Branch from `main`: `feat/<topic>`, `fix/<topic>`, `docs/<topic>` or `chore/<topic>`.
2. **Write a failing test first**, then the code. Every behaviour change needs a test that would
   catch its regression. Test the failure cases — a locked file, a file that changed since the
   scan, a junction — not just the happy path.
3. Run `npm run check` and `npm run test:e2e`.
4. Commit with [Conventional Commits](https://www.conventionalcommits.org):
   `feat(core): skip browser caches while the browser runs` — imperative, lowercase, no period,
   under 72 characters. Scopes: `core`, `winsys`, `app`, `ui`, `docs`, `ci`.
5. Open a pull request and fill in the template. CI must be green.

Small, focused pull requests are reviewed fastest.

## The rules that keep users safe

These are not up for debate in a pull request; they are the product.

- **Scanning never writes.** Every change goes scan → plan → review → apply, and `apply` re-checks
  each item (still there, unchanged, not in use, the Recycle Bin can take it).
- **Recycle Bin by default, never a silent permanent delete.** The only permanent delete is
  "Empty Recycle Bin", behind a confirmation.
- **The guard decides.** Anything outside the user's profile, inside a protected folder, or behind
  a junction/symlink is refused — in `make_plan` and again in `apply`.
- **No admin actions** until the planned elevated helper (see the roadmap).
- **No telemetry, no network** except the opt-in update check.
- **Honest UI.** Real measured sizes, plain words, no fake urgency.

## Recipe: add a cleanup category

1. Add a variant to `CategoryId` in `crates/core/src/model.rs`.
2. Add its rule in `crates/core/src/cleanup.rs` (`scan_cleanup`): which folders, which items,
   pre-selected or not. Everything must stay inside the user's profile.
3. Test it in `crates/core/tests/cleanup.rs` with `Env::for_tests` — include a case that must
   **not** be picked up.
4. Give it a label in `CATEGORY_LABELS` (`ui/src/stores/scan.ts`) and add it to the mock
   (`ui/src/mock.ts`).

## Code style

- Rust: `cargo fmt`, `clippy -D warnings`, small functions, a doc comment on anything public.
  Comments explain **why**, not what.
- TypeScript/Vue: ESLint + Prettier, `<script setup lang="ts">`, no `any`.
- User-facing text: short, friendly, specific ("changed since the scan", not "error 0x80").

## Licence

By contributing you agree that your contribution is licensed under the [MIT licence](LICENSE).
