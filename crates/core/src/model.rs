//! Data shared by scanners, the planner, the applier and the UI.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Where a finding came from. Decides its description and whether permanent
/// delete is allowed for it (only the cache categories).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum CategoryId {
    UserTemp,
    BrowserCache,
    ThumbnailCache,
    CrashDumps,
    OldInstallers,
    AppLeftovers,
    SpaceMap,
}

/// One thing a scan found: a file or a folder with its total measured size.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
    pub id: String,
    pub category: CategoryId,
    pub path: PathBuf,
    pub bytes: u64,
    pub is_dir: bool,
    /// Newest modification time inside it, ms since the Unix epoch. Apply
    /// compares it again to notice a file that changed since the scan.
    pub modified_ms: u64,
    pub preselected: bool,
    /// False for things shown for information only (e.g. needs admin).
    pub selectable: bool,
    pub note: Option<String>,
}

/// Why an item was not changed. Shown to the user after apply.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SkipReason {
    Missing,
    Changed,
    InUse,
    Protected,
    NotRecyclable,
    NeedsAdmin,
    Failed(String),
}
