//! The activity log: one JSON line per apply, so the user can see what was
//! moved where. A damaged line is skipped, never fatal.

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::plan::Action;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ActivityEntry {
    pub at_ms: u64,
    pub action: Action,
    pub done: u64,
    pub freed_bytes: u64,
    pub skipped: u64,
    pub paths: Vec<PathBuf>,
}

pub fn append_activity(file: &Path, entry: &ActivityEntry) -> io::Result<()> {
    if let Some(dir) = file.parent() {
        fs::create_dir_all(dir)?;
    }
    let line = serde_json::to_string(entry).map_err(io::Error::other)?;
    let mut f = OpenOptions::new().create(true).append(true).open(file)?;
    writeln!(f, "{line}")
}

/// Newest first, at most `limit` entries; unreadable lines are ignored.
pub fn read_activity(file: &Path, limit: usize) -> Vec<ActivityEntry> {
    let text = fs::read_to_string(file).unwrap_or_default();
    text.lines()
        .rev()
        .filter_map(|line| serde_json::from_str(line).ok())
        .take(limit)
        .collect()
}
