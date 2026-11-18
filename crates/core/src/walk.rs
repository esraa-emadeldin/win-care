//! Walks a folder tree safely: never follows junctions or symlinks, and keeps
//! going past folders it cannot read.

use std::fs::{self, Metadata};
use std::io::ErrorKind;
use std::os::windows::fs::MetadataExt;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::UNIX_EPOCH;

const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;

/// Shared flag the UI sets to stop a long scan.
#[derive(Clone, Debug, Default)]
pub struct CancelToken(Arc<AtomicBool>);

impl CancelToken {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Relaxed);
    }
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WalkStats {
    pub files: u64,
    pub bytes: u64,
    pub unreadable_dirs: u64,
}

/// Calls `on_file` for every regular file under `root`. Junctions and
/// symlinks are skipped entirely, not followed.
pub fn walk(
    root: &Path,
    cancel: &CancelToken,
    on_file: &mut dyn FnMut(&Path, &Metadata),
) -> WalkStats {
    let mut stats = WalkStats::default();
    if fs::symlink_metadata(root).is_ok_and(|m| is_reparse_point(&m)) {
        return stats;
    }
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        if cancel.is_cancelled() {
            break;
        }
        let entries = match fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(e) if e.kind() == ErrorKind::NotFound => continue,
            Err(_) => {
                stats.unreadable_dirs += 1;
                continue;
            }
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(meta) = fs::symlink_metadata(&path) else {
                continue;
            };
            if is_reparse_point(&meta) {
                continue;
            }
            if meta.is_dir() {
                stack.push(path);
            } else {
                stats.files += 1;
                stats.bytes += meta.len();
                on_file(&path, &meta);
            }
        }
    }
    stats
}

/// True for junctions and symlinks (and other reparse points).
pub fn is_reparse_point(meta: &Metadata) -> bool {
    meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

/// Modification time in ms since the Unix epoch (0 when unknown).
pub fn modified_ms(meta: &Metadata) -> u64 {
    meta.modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_millis() as u64)
}

/// Creation time in ms since the Unix epoch (0 when unknown).
pub fn created_ms(meta: &Metadata) -> u64 {
    meta.created()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_millis() as u64)
}

/// When a file last became new here: the later of modified and created.
/// A file extracted or copied a minute ago keeps an old modified time but has
/// a fresh creation time, and must not look abandoned.
pub fn fresh_ms(meta: &Metadata) -> u64 {
    modified_ms(meta).max(created_ms(meta))
}
