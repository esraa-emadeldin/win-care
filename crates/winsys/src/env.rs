//! The real folder layout, from Windows' known folders, so a moved Downloads
//! or Documents folder is still found (and still protected).

use std::path::PathBuf;

use windows::Win32::System::Com::CoTaskMemFree;
use windows::Win32::UI::Shell::{
    FOLDERID_Desktop, FOLDERID_Documents, FOLDERID_Downloads, FOLDERID_LocalAppData,
    FOLDERID_Music, FOLDERID_Pictures, FOLDERID_Profile, FOLDERID_ProgramData,
    FOLDERID_ProgramFiles, FOLDERID_ProgramFilesX86, FOLDERID_RoamingAppData, FOLDERID_Videos,
    FOLDERID_Windows, KF_FLAG_DEFAULT, SHGetKnownFolderPath,
};
use windows::core::GUID;

use wincare_core::Env;

fn known(id: &GUID) -> Result<PathBuf, String> {
    unsafe {
        let p = SHGetKnownFolderPath(id, KF_FLAG_DEFAULT, None).map_err(|e| e.message())?;
        let path = p.to_string().map_err(|e| e.to_string());
        CoTaskMemFree(Some(p.0 as *const _));
        path.map(PathBuf::from)
    }
}

/// Personal folders in the profile that are never offered, besides the
/// known folders (which may have moved, e.g. into OneDrive).
const PERSONAL: [&str; 10] = [
    "Documents",
    "Desktop",
    "Pictures",
    "Music",
    "Videos",
    "Saved Games",
    "Favorites",
    "Contacts",
    "Links",
    ".ssh",
];
/// Environment variables Windows sets to the OneDrive folders.
const ONEDRIVE_VARS: [&str; 3] = ["OneDrive", "OneDriveConsumer", "OneDriveCommercial"];

pub fn system_env() -> Result<Env, String> {
    let profile = known(&FOLDERID_Profile)?;
    // Where Windows says the personal folders are (they may live in OneDrive)…
    let mut protected: Vec<PathBuf> = [
        &FOLDERID_Documents,
        &FOLDERID_Desktop,
        &FOLDERID_Pictures,
        &FOLDERID_Music,
        &FOLDERID_Videos,
    ]
    .into_iter()
    .filter_map(|id| known(id).ok())
    .collect();
    // …plus their default places, so a failed lookup never drops protection…
    protected.extend(PERSONAL.iter().map(|name| profile.join(name)));
    // …plus every OneDrive root: everything in it is the user's own files.
    protected.extend(
        ONEDRIVE_VARS
            .iter()
            .filter_map(std::env::var_os)
            .map(PathBuf::from)
            .filter(|p| !p.as_os_str().is_empty()),
    );
    protected.sort();
    protected.dedup();

    Ok(Env {
        temp: std::env::temp_dir(),
        local_app_data: known(&FOLDERID_LocalAppData)?,
        app_data: known(&FOLDERID_RoamingAppData)?,
        downloads: known(&FOLDERID_Downloads).unwrap_or_else(|_| profile.join("Downloads")),
        windir: known(&FOLDERID_Windows)?,
        program_files: vec![
            known(&FOLDERID_ProgramFiles)?,
            known(&FOLDERID_ProgramFilesX86)?,
        ],
        program_data: known(&FOLDERID_ProgramData)?,
        known_protected: protected,
        user_profile: profile,
    })
}

/// Lower-case names of running programs, e.g. "chrome.exe".
pub fn running_process_names() -> Vec<String> {
    let Ok(mut system) = sysinfo::System::new() else {
        return Vec::new();
    };
    system.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    system
        .processes()
        .values()
        .map(|p| p.name().to_string_lossy().to_lowercase())
        .collect()
}
