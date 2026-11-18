//! wincare-winsys: Windows API implementations of the wincare-core traits.

mod apps;
mod env;
mod health;
mod maintenance;
mod recycle;
mod startup;
mod vitals;

pub use apps::{list_apps, uninstall};
pub use env::{running_process_names, system_env};
pub use health::health_checks;
pub use maintenance::restart_explorer;
pub use recycle::{WinFileOps, empty_recycle_bin, recycle_bin_size};
pub use startup::{list_startup, set_startup_enabled};
pub use vitals::{Drive, Vitals, VitalsSampler, drives};
