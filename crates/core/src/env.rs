//! The folders scanners read from. Built from the real system in the app,
//! and from a temp folder in tests, so every scanner is testable.

use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct Env {
    pub temp: PathBuf,
    pub local_app_data: PathBuf,
    pub app_data: PathBuf,
    pub user_profile: PathBuf,
    pub downloads: PathBuf,
    pub windir: PathBuf,
    pub program_files: Vec<PathBuf>,
    pub program_data: PathBuf,
    /// Documents, Desktop, Pictures, Music, Videos: never offered.
    pub known_protected: Vec<PathBuf>,
}

impl Env {
    /// A fake system under `root`, laid out like the real one, folders created.
    pub fn for_tests(root: &Path) -> Env {
        let user = root.join("Users").join("tester");
        let local = user.join("AppData").join("Local");
        let env = Env {
            temp: local.join("Temp"),
            app_data: user.join("AppData").join("Roaming"),
            downloads: user.join("Downloads"),
            local_app_data: local,
            windir: root.join("Windows"),
            program_files: vec![root.join("Program Files"), root.join("Program Files (x86)")],
            program_data: root.join("ProgramData"),
            known_protected: ["Documents", "Desktop", "Pictures", "Music", "Videos"]
                .iter()
                .map(|n| user.join(n))
                .collect(),
            user_profile: user,
        };
        let fixed = [
            &env.temp,
            &env.app_data,
            &env.downloads,
            &env.windir,
            &env.program_data,
        ];
        for dir in fixed
            .into_iter()
            .chain(&env.program_files)
            .chain(&env.known_protected)
        {
            fs::create_dir_all(dir).expect("create test folder");
        }
        env
    }
}
