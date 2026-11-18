//! The last line of defence: decides whether a path may ever be changed.
//! Planning and apply both ask it, so a faulty scanner cannot reach a
//! protected file.

use std::fs;
use std::path::{Component, Path};

use crate::env::Env;
use crate::model::SkipReason;
use crate::walk::is_reparse_point;

pub struct Guard {
    profile: String,
    /// Never touched, nor anything inside, nor anything that contains them.
    protected: Vec<String>,
    /// Never removed as a whole; what is inside them may be.
    roots: Vec<String>,
}

impl Guard {
    pub fn new(env: &Env) -> Guard {
        let protected = env
            .known_protected
            .iter()
            .chain(&env.program_files)
            .chain([&env.windir, &env.program_data])
            .map(|p| key(p))
            .collect();
        let app_data = env.user_profile.join("AppData");
        let roots = [
            app_data.join("LocalLow"),
            app_data,
            env.local_app_data.clone(),
            env.app_data.clone(),
            env.temp.clone(),
            env.downloads.clone(),
        ]
        .iter()
        .map(|p| key(p))
        .collect();
        Guard {
            profile: key(&env.user_profile),
            protected,
            roots,
        }
    }

    /// `Ok` only for a path strictly inside the user's profile, outside every
    /// protected folder, reached without passing through a junction or symlink.
    pub fn check(&self, path: &Path) -> Result<(), SkipReason> {
        if path.components().any(|c| c == Component::ParentDir) {
            return Err(SkipReason::Protected);
        }
        let k = key(path);
        if !is_inside(&k, &self.profile) {
            return Err(SkipReason::Protected);
        }
        // Inside a protected folder, or containing one (e.g. OneDrive holding
        // Documents): either way, removing it would remove protected files.
        if self
            .protected
            .iter()
            .any(|p| k == *p || is_inside(&k, p) || is_inside(p, &k))
        {
            return Err(SkipReason::Protected);
        }
        if self.roots.contains(&k) {
            return Err(SkipReason::Protected);
        }
        // The path itself and each folder above it, up to the profile, must be
        // a real folder: a junction could point anywhere.
        let mut current = Some(path);
        while let Some(p) = current {
            if key(p) == self.profile {
                break;
            }
            if fs::symlink_metadata(p).is_ok_and(|m| is_reparse_point(&m)) {
                return Err(SkipReason::Protected);
            }
            current = p.parent();
        }
        Ok(())
    }
}

/// Windows paths compare case-insensitively: one spelling for comparing.
fn key(path: &Path) -> String {
    let s = path.to_string_lossy().replace('/', "\\").to_lowercase();
    let s = s.strip_prefix(r"\\?\").unwrap_or(&s);
    s.trim_end_matches('\\').to_string()
}

/// True when `path_key` is below `base_key` (not equal to it).
fn is_inside(path_key: &str, base_key: &str) -> bool {
    path_key.len() > base_key.len()
        && path_key.starts_with(base_key)
        && path_key.as_bytes()[base_key.len()] == b'\\'
}
