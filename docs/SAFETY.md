# Safety model

win-care's one job is to remove things without ever removing the wrong thing. These are the
rules, and where in the code each one lives.

## 1. Scanning only reads

Scanners (`crates/core/src/cleanup.rs`, `apps.rs`, `space.rs`) return **findings**: a path, its
measured size and newest modification time. They never write. Walking a folder never follows a
junction or symlink (`walk.rs`), and a folder it cannot read is counted and skipped, not fatal.

## 2. You review a plan

`make_plan` (`plan.rs`) turns the findings you ticked into a **plan**. Items you cannot pick
(for example leftovers outside your profile, which would need admin) never enter a plan.
Review shows every item, its size, and a badge on anything the Recycle Bin cannot take.

## 3. Apply re-checks everything

`apply` (`plan.rs`) is the only code that changes files. For every item, in order:

| Check                                          | If it fails                              |
| ---------------------------------------------- | ---------------------------------------- |
| The guard allows the path                      | skipped: _protected folder_              |
| It still exists                                | skipped: _already gone_                  |
| Same size and modification time as in the scan | skipped: _changed since the scan_        |
| No other program has it open                   | skipped: _in use_                        |
| The drive's Recycle Bin is on and big enough   | skipped: _too large for the Recycle Bin_ |

Then it is moved to the Recycle Bin, and the result is written to the activity log
(`%APPDATA%\win-care\activity.jsonl`).

## 4. The guard

`Guard::check` (`guard.rs`) allows a path only if it is **inside your user profile**, **not** in
Documents, Desktop, Pictures, Music, Videos, Windows, Program Files or ProgramData, contains no
`..`, and **no folder on the way to it is a junction or symlink**. It runs in `make_plan` _and_
again in `apply`, so even a hand-made plan cannot reach a protected file. The real folder
locations come from Windows' known-folder API, so a moved Documents folder is still protected.

## 5. Never a silent permanent delete

Moving to the Recycle Bin uses the Windows shell (`crates/winsys/src/recycle.rs`). Windows
normally deletes permanently when an item does not fit in the Recycle Bin. win-care attaches a
progress sink whose `PreDeleteItem` **refuses** any delete that would not go to the Recycle Bin,
and afterwards checks the item really left its folder.

The only permanent delete is explicit: **Empty Recycle Bin**, behind a "cannot be undone"
confirmation. Everything else goes to the Recycle Bin.

## 6. No admin, no network

v0.1 never asks for admin rights; Windows itself refuses changes outside your profile. The
only network request is the opt-in daily update check, and updates are signature-checked
before install.

Found a way around any of this? Please report it privately — see [SECURITY.md](../SECURITY.md).
