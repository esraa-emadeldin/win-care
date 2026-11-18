//! Installed apps and what they leave behind after an uninstall.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::cleanup::measure;
use crate::env::Env;
use crate::guard::Guard;
use crate::model::{CategoryId, Finding};
use crate::walk::CancelToken;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum AppKind {
    Win32 {
        uninstall_cmd: String,
        quiet_cmd: Option<String>,
        install_location: Option<PathBuf>,
    },
    Msix {
        full_name: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct App {
    pub id: String,
    pub name: String,
    pub publisher: Option<String>,
    pub version: Option<String>,
    pub bytes: Option<u64>,
    /// Only when Windows records it; never guessed.
    pub last_used_ms: Option<u64>,
    pub kind: AppKind,
}

impl App {
    fn install_location(&self) -> Option<&Path> {
        match &self.kind {
            AppKind::Win32 {
                install_location, ..
            } => install_location.as_deref(),
            AppKind::Msix { .. } => None,
        }
    }
}

/// Folders an uninstalled app left behind, matched by its name (or
/// publisher\name). A folder holding another installed app is never offered.
/// Folders outside the user's profile are shown but not selectable in v1.
pub fn find_leftovers(app: &App, others: &[App], env: &Env) -> Vec<Finding> {
    let guard = Guard::new(env);
    let name = normalize(&app.name);
    let publisher = app.publisher.as_deref().map(normalize);
    if name.is_empty() {
        return Vec::new();
    }
    let roots =
        env.program_files
            .iter()
            .chain([&env.program_data, &env.local_app_data, &env.app_data]);

    let mut found = Vec::new();
    for root in roots {
        for child in subfolders(root) {
            let child_name = folder_name(&child);
            if child_name == name {
                found.push(child);
            } else if publisher.as_deref() == Some(child_name.as_str()) {
                found.extend(
                    subfolders(&child)
                        .into_iter()
                        .filter(|g| folder_name(g) == name),
                );
            }
        }
    }

    found
        .into_iter()
        .filter(|dir| !holds_other_app(dir, others))
        .filter_map(|dir| {
            let m = measure(&dir, &CancelToken::new())?;
            let removable = guard.check(&dir).is_ok();
            Some(Finding {
                id: dir.to_string_lossy().to_lowercase(),
                category: CategoryId::AppLeftovers,
                bytes: m.bytes,
                is_dir: true,
                modified_ms: m.newest_ms,
                preselected: removable,
                selectable: removable,
                note: (!removable).then(|| "Needs admin; not removed in v1".to_string()),
                path: dir,
            })
        })
        .collect()
}

/// True if an app with the same name is still installed (any version, any
/// registration): uninstallers often leave a second entry, or have not
/// finished yet.
pub fn still_installed(target: &App, after: &[App]) -> bool {
    let name = normalize(&target.name);
    after.iter().any(|a| normalize(&a.name) == name)
}

/// The folder belongs to another installed app: it holds that app's install
/// location, or carries that app's name.
fn holds_other_app(dir: &Path, others: &[App]) -> bool {
    let name = folder_name(dir);
    if others.iter().any(|a| normalize(&a.name) == name) {
        return true;
    }
    let dir = dir.to_string_lossy().to_lowercase();
    others.iter().filter_map(App::install_location).any(|loc| {
        let loc = loc.to_string_lossy().to_lowercase();
        loc == dir || loc.starts_with(&format!("{dir}\\"))
    })
}

/// Lowercase letters and digits only: "Contoso Notes" -> "contosonotes".
fn normalize(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn folder_name(path: &Path) -> String {
    path.file_name()
        .map(|n| normalize(&n.to_string_lossy()))
        .unwrap_or_default()
}

fn subfolders(dir: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = fs::read_dir(dir)
        .map(|rd| {
            rd.flatten()
                .map(|e| e.path())
                .filter(|p| p.is_dir())
                .collect()
        })
        .unwrap_or_default();
    out.sort();
    out
}
