## Product

A free, open-source PC care app for Windows 11: it finds junk, frees disk space, removes apps
cleanly and keeps an eye on the PC's health. Working name only; it needs its own identity and
must not imitate CleanMyMac or any existing product's name, logo or look.

The attached screenshots are a macOS app by the same family of tools, given only as a reference
for the feature set and the "cards + one big Run button" idea. Redesign each screen as a native
Windows 11 app (Fluent: Mica background, rounded corners, Segoe UI Variable, light and dark).

Two rules shape every screen:
- Nothing is deleted before the user reviews the exact list of items.
- Deletes go to the Recycle Bin by default, so they can be undone.

## Users

Everyday Windows users who are not technical: students, families, office workers. They want a
faster, cleaner PC in one click, plain words instead of jargon, and to feel safe that nothing
important is removed.

## Stack

Rust with Tauri (a web-view UI), so the design should be buildable with HTML/CSS components.
A tray icon opens a small popup; a main window holds the full tools.

## Screens

1. Smart Scan (home): one scan across Cleanup, Apps, Startup and Health, results as cards, one
   primary "Clean" button.
2. Tray popup: live CPU, RAM, disk, battery, network; one recommendation; "Open app".
3. Apps: installed apps with size and last used, Uninstall that also finds leftovers.
4. Performance: startup apps you can switch off, and one-click maintenance tasks.
5. (new, no screenshot) Review before delete: grouped list of files with sizes and checkboxes,
   "Move to Recycle Bin".
6. (new, no screenshot) Space map: what uses the disk, as a treemap or sunburst.

## Current tokens

None yet. Propose a palette, type scale, radius and spacing that fit Windows 11 and a
friendly, trustworthy brand. Avoid purple/pink gradients that read as a copy of Mac cleaners.
