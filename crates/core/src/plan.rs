//! Plan and apply. A plan is the exact list the user reviews; apply is the only
//! code that changes anything, and it re-checks every item first.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::cleanup::measure;
use crate::guard::Guard;
use crate::model::{Finding, SkipReason};
use crate::walk::CancelToken;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Action {
    Recycle,
    DeletePermanently,
}

/// What the Recycle Bin of a path's drive can take.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BinInfo {
    pub enabled: bool,
    pub max_bytes: u64,
}

/// The file operations apply needs. Windows implements it in wincare-winsys;
/// tests use a fake, so no test ever recycles a real file by accident.
pub trait FileOps {
    fn bin_info(&self, path: &Path) -> BinInfo;
    fn in_use(&self, path: &Path) -> bool;
    /// Must never fall back to a permanent delete.
    fn recycle(&self, path: &Path) -> Result<(), String>;
    fn delete(&self, path: &Path) -> Result<(), String>;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PlanItem {
    pub finding: Finding,
    /// False when the drive's Recycle Bin is off or too small for this item.
    pub recyclable: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    pub id: String,
    pub action: Action,
    pub items: Vec<PlanItem>,
    pub total_bytes: u64,
    /// Selected paths the guard refused, with the reason, so Review can show them.
    pub rejected: Vec<Skipped>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Skipped {
    pub path: PathBuf,
    pub reason: SkipReason,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ApplyReport {
    pub done: Vec<PathBuf>,
    pub freed_bytes: u64,
    pub skipped: Vec<Skipped>,
}

/// Builds the plan the user reviews. Drops anything not selectable or not
/// allowed by the guard; reads only.
pub fn make_plan(findings: Vec<Finding>, action: Action, ops: &dyn FileOps, guard: &Guard) -> Plan {
    let mut items = Vec::new();
    let mut rejected = Vec::new();
    for finding in findings.into_iter().filter(|f| f.selectable) {
        match guard.check(&finding.path) {
            Err(reason) => rejected.push(Skipped {
                path: finding.path,
                reason,
            }),
            Ok(()) => {
                let recyclable = fits_in_bin(ops.bin_info(&finding.path), finding.bytes);
                items.push(PlanItem {
                    finding,
                    recyclable,
                });
            }
        }
    }
    Plan {
        id: next_plan_id(),
        action,
        total_bytes: items.iter().map(|i| i.finding.bytes).sum(),
        items,
        rejected,
    }
}

/// Carries out a reviewed plan. Every item is checked again, in this order:
/// guard, still there, unchanged, not in use, Recycle Bin can take it.
pub fn apply(plan: &Plan, ops: &dyn FileOps, guard: &Guard) -> ApplyReport {
    let mut report = ApplyReport::default();
    for item in &plan.items {
        let f = &item.finding;
        match check_item(plan.action, f, ops, guard) {
            Err(reason) => report.skipped.push(Skipped {
                path: f.path.clone(),
                reason,
            }),
            Ok(()) => {
                let result = match plan.action {
                    Action::Recycle => ops.recycle(&f.path),
                    Action::DeletePermanently => ops.delete(&f.path),
                };
                match result {
                    Ok(()) => {
                        report.done.push(f.path.clone());
                        report.freed_bytes += f.bytes;
                    }
                    Err(e) => report.skipped.push(Skipped {
                        path: f.path.clone(),
                        reason: SkipReason::Failed(e),
                    }),
                }
            }
        }
    }
    report
}

fn check_item(
    action: Action,
    f: &Finding,
    ops: &dyn FileOps,
    guard: &Guard,
) -> Result<(), SkipReason> {
    guard.check(&f.path)?;
    let now = measure(&f.path, &CancelToken::new()).ok_or(SkipReason::Missing)?;
    if now.bytes != f.bytes || now.newest_ms != f.modified_ms {
        return Err(SkipReason::Changed);
    }
    if ops.in_use(&f.path) {
        return Err(SkipReason::InUse);
    }
    if action == Action::Recycle && !fits_in_bin(ops.bin_info(&f.path), f.bytes) {
        return Err(SkipReason::NotRecyclable);
    }
    Ok(())
}

fn fits_in_bin(bin: BinInfo, bytes: u64) -> bool {
    bin.enabled && bytes <= bin.max_bytes
}

fn next_plan_id() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis());
    format!("plan-{now}-{}", COUNTER.fetch_add(1, Ordering::Relaxed))
}
