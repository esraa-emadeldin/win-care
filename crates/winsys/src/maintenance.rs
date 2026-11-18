//! One-click maintenance tasks. None of them needs admin rights.

use std::os::windows::process::CommandExt;
use std::process::Command;

/// Keeps a console window from flashing up.
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Restarts Windows Explorer (taskbar, Start, file windows). Open Explorer
/// windows close; files and other apps are not affected.
pub fn restart_explorer() -> Result<String, String> {
    Command::new("taskkill")
        .args(["/f", "/im", "explorer.exe"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|e| e.to_string())?;
    Command::new("explorer.exe")
        .spawn()
        .map_err(|e| format!("Explorer did not start again: {e}"))?;
    Ok("Windows Explorer restarted".into())
}
