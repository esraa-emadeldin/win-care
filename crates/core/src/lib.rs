//! wincare-core: scanners, planner and applier. Planning never writes;
//! every write happens in `apply`, after the user reviewed the plan.

mod activity;
mod apps;
mod cleanup;
mod env;
mod guard;
mod health;
mod model;
mod plan;
mod space;
mod startup;
mod walk;

pub use activity::{ActivityEntry, append_activity, read_activity};
pub use apps::{App, AppKind, find_leftovers, still_installed};
pub use cleanup::{CleanupResult, finding_for_path, scan_cleanup};
pub use env::Env;
pub use guard::Guard;
pub use health::{Check, CheckStatus, battery_wear_check, disk_space_check};
pub use model::{CategoryId, Finding, SkipReason};
pub use plan::{Action, ApplyReport, BinInfo, FileOps, Plan, PlanItem, Skipped, apply, make_plan};
pub use space::{OTHER, SpaceNode, scan_space};
pub use startup::{StartupItem, StartupSource, approved_is_enabled, approved_value};
pub use walk::{CancelToken, WalkStats, fresh_ms, is_reparse_point, modified_ms, walk};

/// The crate version, shown in the About screen.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    #[test]
    fn workspace_builds() {
        assert_eq!(crate::VERSION, env!("CARGO_PKG_VERSION"));
    }
}
