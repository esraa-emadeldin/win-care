//! Everything the commands share between calls: the last scan results (so a
//! plan can only contain things a scan found), pending plans and settings.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use wincare_core::{App, CancelToken, CleanupResult, Env, Finding, Plan, StartupItem};
use wincare_winsys::VitalsSampler;

use crate::settings::{self, Settings};

pub struct AppState {
    pub env: Env,
    pub data_dir: PathBuf,
    pub settings: Mutex<Settings>,
    pub last_cleanup: Mutex<Option<CleanupResult>>,
    pub last_leftovers: Mutex<Vec<Finding>>,
    /// Paths shown in the last space map, which the user may send to Review.
    pub space_paths: Mutex<Vec<PathBuf>>,
    pub space_cancel: Mutex<CancelToken>,
    pub plans: Mutex<HashMap<String, Plan>>,
    pub apps: Mutex<Vec<App>>,
    pub startup: Mutex<Vec<StartupItem>>,
    pub sampler: Mutex<Option<VitalsSampler>>,
}

impl AppState {
    pub fn new(env: Env, data_dir: PathBuf) -> Self {
        let settings = settings::load(&data_dir.join("settings.json"));
        AppState {
            env,
            settings: Mutex::new(settings),
            data_dir,
            last_cleanup: Mutex::default(),
            last_leftovers: Mutex::default(),
            space_paths: Mutex::default(),
            space_cancel: Mutex::default(),
            plans: Mutex::default(),
            apps: Mutex::default(),
            startup: Mutex::default(),
            sampler: Mutex::default(),
        }
    }

    pub fn settings_file(&self) -> PathBuf {
        self.data_dir.join("settings.json")
    }

    pub fn activity_file(&self) -> PathBuf {
        self.data_dir.join("activity.jsonl")
    }
}
