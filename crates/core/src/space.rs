//! The space map: a size tree of a folder or drive, for the treemap.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::walk::{CancelToken, is_reparse_point, walk};

/// Children smaller than this share of their parent (in per mille) are folded
/// into one "Other" node, so the treemap stays readable.
const FOLD_BELOW_PER_MILLE: u64 = 5;
pub const OTHER: &str = "Other";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SpaceNode {
    pub name: String,
    /// Empty for the folded "Other" node, which is not a real folder.
    pub path: PathBuf,
    pub bytes: u64,
    pub children: Vec<SpaceNode>,
}

/// Builds the tree to `max_depth` levels (deeper sizes still counted).
/// `on_partial` gets each top-level child as soon as it is measured, so the
/// map can fill in while a big drive is still being scanned.
pub fn scan_space(
    root: &Path,
    max_depth: u8,
    cancel: &CancelToken,
    on_partial: &mut dyn FnMut(&SpaceNode),
) -> SpaceNode {
    let mut children = Vec::new();
    for child in entries(root) {
        if cancel.is_cancelled() {
            break;
        }
        if let Some(node) = build(&child, 1, max_depth, cancel) {
            on_partial(&node);
            children.push(node);
        }
    }
    finish(name_of(root), root.to_path_buf(), children)
}

fn build(path: &Path, depth: u8, max_depth: u8, cancel: &CancelToken) -> Option<SpaceNode> {
    let meta = fs::symlink_metadata(path).ok()?;
    if is_reparse_point(&meta) {
        return None;
    }
    if !meta.is_dir() {
        return Some(SpaceNode {
            name: name_of(path),
            path: path.to_path_buf(),
            bytes: meta.len(),
            children: Vec::new(),
        });
    }
    if depth >= max_depth {
        let stats = walk(path, cancel, &mut |_, _| {});
        return Some(SpaceNode {
            name: name_of(path),
            path: path.to_path_buf(),
            bytes: stats.bytes,
            children: Vec::new(),
        });
    }
    let children = entries(path)
        .iter()
        .take_while(|_| !cancel.is_cancelled())
        .filter_map(|c| build(c, depth + 1, max_depth, cancel))
        .collect();
    Some(finish(name_of(path), path.to_path_buf(), children))
}

/// Totals the children, sorts them largest first and folds the tiny ones.
fn finish(name: String, path: PathBuf, mut children: Vec<SpaceNode>) -> SpaceNode {
    let bytes: u64 = children.iter().map(|c| c.bytes).sum();
    children.sort_by_key(|c| std::cmp::Reverse(c.bytes));
    let (keep, tiny): (Vec<_>, Vec<_>) = children
        .into_iter()
        .partition(|c| bytes == 0 || c.bytes * 1000 >= bytes * FOLD_BELOW_PER_MILLE);
    let mut children = keep;
    if !tiny.is_empty() {
        children.push(SpaceNode {
            name: OTHER.into(),
            path: PathBuf::new(),
            bytes: tiny.iter().map(|c| c.bytes).sum(),
            children: Vec::new(),
        });
    }
    SpaceNode {
        name,
        path,
        bytes,
        children,
    }
}

fn entries(dir: &Path) -> Vec<PathBuf> {
    fs::read_dir(dir)
        .map(|rd| rd.flatten().map(|e| e.path()).collect())
        .unwrap_or_default()
}

fn name_of(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string())
}
