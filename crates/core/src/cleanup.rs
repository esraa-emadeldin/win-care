//! Cleanup scanners. Each category names its folders and its rule; a scan only
//! reads and returns findings, one per top-level item, with measured sizes.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::env::Env;
use crate::guard::Guard;
use crate::model::{CategoryId, Finding};
use crate::walk::{CancelToken, created_ms, fresh_ms, is_reparse_point, modified_ms, walk};

const DAY_MS: u64 = 24 * 60 * 60 * 1000;
/// Temp items untouched for this long are considered left behind.
const TEMP_MIN_AGE_MS: u64 = DAY_MS;
/// Installers in Downloads older than this are offered (never pre-selected).
const INSTALLER_MIN_AGE_MS: u64 = 30 * DAY_MS;
const INSTALLER_EXTENSIONS: [&str; 3] = ["exe", "msi", "msix"];

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct CleanupResult {
    pub findings: Vec<Finding>,
    /// Plain-language notes, e.g. "Chrome is running; close it to clean its cache".
    pub notes: Vec<String>,
    pub unreadable_dirs: u64,
}

struct Browser {
    name: &'static str,
    process: &'static str,
    /// Folder under LocalAppData holding one sub-folder per profile.
    profiles: &'static str,
    cache: &'static str,
}

const BROWSERS: [Browser; 4] = [
    Browser {
        name: "Edge",
        process: "msedge.exe",
        profiles: r"Microsoft\Edge\User Data",
        cache: "Cache",
    },
    Browser {
        name: "Chrome",
        process: "chrome.exe",
        profiles: r"Google\Chrome\User Data",
        cache: "Cache",
    },
    Browser {
        name: "Brave",
        process: "brave.exe",
        profiles: r"BraveSoftware\Brave-Browser\User Data",
        cache: "Cache",
    },
    Browser {
        name: "Firefox",
        process: "firefox.exe",
        profiles: r"Mozilla\Firefox\Profiles",
        cache: "cache2",
    },
];

/// Scans every cleanup category. Reads only; changes nothing.
pub fn scan_cleanup(
    env: &Env,
    now_ms: u64,
    is_running: &dyn Fn(&str) -> bool,
    cancel: &CancelToken,
    progress: &mut dyn FnMut(CategoryId, u64),
) -> CleanupResult {
    let guard = Guard::new(env);
    let mut s = Scan {
        guard: &guard,
        cancel,
        result: CleanupResult::default(),
    };

    // User temp: whole top-level items whose newest file is older than a day.
    for child in children(&env.temp) {
        s.add(CategoryId::UserTemp, &child, true, |m| {
            now_ms.saturating_sub(m.newest_ms) > TEMP_MIN_AGE_MS
        });
    }
    s.report(CategoryId::UserTemp, progress);

    // Browser caches: one finding per profile cache folder, skipped while running.
    for b in &BROWSERS {
        let caches: Vec<PathBuf> = children(&env.local_app_data.join(b.profiles))
            .into_iter()
            .map(|profile| profile.join(b.cache))
            .filter(|c| c.is_dir())
            .collect();
        if caches.is_empty() {
            continue;
        }
        if is_running(b.process) {
            s.result.notes.push(format!(
                "{} is running; close it to clean its cache",
                b.name
            ));
            continue;
        }
        for cache in caches {
            s.add(CategoryId::BrowserCache, &cache, true, |_| true);
        }
    }
    s.report(CategoryId::BrowserCache, progress);

    // Thumbnail and icon caches: the database files themselves.
    for file in children(&env.local_app_data.join(r"Microsoft\Windows\Explorer")) {
        let name = file_name(&file);
        if (name.starts_with("thumbcache_") || name.starts_with("iconcache_"))
            && name.ends_with(".db")
        {
            s.add(CategoryId::ThumbnailCache, &file, true, |_| true);
        }
    }
    s.report(CategoryId::ThumbnailCache, progress);

    // Crash dumps and error reports in the user's profile.
    for root in [
        r"CrashDumps",
        r"Microsoft\Windows\WER\ReportArchive",
        r"Microsoft\Windows\WER\ReportQueue",
    ] {
        for child in children(&env.local_app_data.join(root)) {
            s.add(CategoryId::CrashDumps, &child, true, |_| true);
        }
    }
    s.report(CategoryId::CrashDumps, progress);

    // Old installers in Downloads: offered, never pre-selected.
    for file in children(&env.downloads) {
        let ext = file
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        if file.is_file() && INSTALLER_EXTENSIONS.contains(&ext.as_str()) {
            s.add(CategoryId::OldInstallers, &file, false, |m| {
                now_ms.saturating_sub(m.newest_ms) > INSTALLER_MIN_AGE_MS
            });
        }
    }
    s.report(CategoryId::OldInstallers, progress);

    s.result
}

struct Scan<'a> {
    guard: &'a Guard,
    cancel: &'a CancelToken,
    result: CleanupResult,
}

/// Size and newest modification time of a file or a whole folder.
pub(crate) struct Measured {
    pub bytes: u64,
    pub newest_ms: u64,
    pub is_dir: bool,
    pub unreadable_dirs: u64,
}

impl Scan<'_> {
    fn add(
        &mut self,
        category: CategoryId,
        path: &Path,
        preselected: bool,
        keep: impl Fn(&Measured) -> bool,
    ) {
        if self.cancel.is_cancelled() || self.guard.check(path).is_err() {
            return;
        }
        let Some(m) = measure(path, self.cancel) else {
            return;
        };
        self.result.unreadable_dirs += m.unreadable_dirs;
        if m.bytes == 0 && !m.is_dir || !keep(&m) {
            return;
        }
        self.result.findings.push(Finding {
            id: path.to_string_lossy().to_lowercase(),
            category,
            path: path.to_path_buf(),
            bytes: m.bytes,
            is_dir: m.is_dir,
            modified_ms: m.newest_ms,
            // Contents we could not read may be recent or unknown: offer, never pre-select.
            preselected: preselected && m.unreadable_dirs == 0,
            selectable: true,
            note: (m.unreadable_dirs > 0)
                .then(|| "Some folders inside could not be read".to_string()),
        });
    }

    fn report(&self, category: CategoryId, progress: &mut dyn FnMut(CategoryId, u64)) {
        let bytes = self
            .result
            .findings
            .iter()
            .filter(|f| f.category == category)
            .map(|f| f.bytes)
            .sum();
        progress(category, bytes);
    }
}

/// A finding for a path the user picked by hand (e.g. in the space map),
/// measured now. `None` if it is gone or is a junction/symlink. The guard is
/// still applied later by `make_plan` and `apply`.
pub fn finding_for_path(path: &Path, category: CategoryId) -> Option<Finding> {
    let m = measure(path, &CancelToken::new())?;
    Some(Finding {
        id: path.to_string_lossy().to_lowercase(),
        category,
        path: path.to_path_buf(),
        bytes: m.bytes,
        is_dir: m.is_dir,
        modified_ms: m.newest_ms,
        preselected: true,
        selectable: true,
        note: None,
    })
}

/// Measures a file or folder without following junctions. `None` if it is gone
/// or is itself a junction/symlink.
pub(crate) fn measure(path: &Path, cancel: &CancelToken) -> Option<Measured> {
    let meta = fs::symlink_metadata(path).ok()?;
    if is_reparse_point(&meta) {
        return None;
    }
    if !meta.is_dir() {
        return Some(Measured {
            bytes: meta.len(),
            newest_ms: fresh_ms(&meta),
            is_dir: false,
            unreadable_dirs: 0,
        });
    }
    // Age a folder by its files and by when the folder itself was created.
    // Not by the folder's modified time: that changes whenever anything inside
    // is added or removed, which says nothing about whether it is used.
    let mut newest = created_ms(&meta);
    let stats = walk(path, cancel, &mut |_, m| newest = newest.max(fresh_ms(m)));
    if stats.files == 0 {
        newest = newest.max(modified_ms(&meta));
    }
    Some(Measured {
        bytes: stats.bytes,
        newest_ms: newest,
        is_dir: true,
        unreadable_dirs: stats.unreadable_dirs,
    })
}

/// Direct children of a folder, sorted; empty if it does not exist.
fn children(dir: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = fs::read_dir(dir)
        .map(|rd| rd.flatten().map(|e| e.path()).collect())
        .unwrap_or_default();
    out.sort();
    out
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().to_lowercase())
        .unwrap_or_default()
}
