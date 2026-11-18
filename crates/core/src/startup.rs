//! Startup items and the `StartupApproved` values Task Manager uses to switch
//! them on and off without deleting them (so every change is reversible).

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum StartupSource {
    HkcuRun,
    HklmRun,
    Hklm32Run,
    UserFolder,
    CommonFolder,
    Task,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StartupItem {
    pub id: String,
    pub name: String,
    pub command: String,
    pub publisher: Option<String>,
    pub source: StartupSource,
    pub enabled: bool,
    /// Only per-user items can be switched without admin rights.
    pub can_toggle: bool,
}

impl StartupSource {
    pub fn can_toggle(self) -> bool {
        matches!(self, StartupSource::HkcuRun | StartupSource::UserFolder)
    }
}

/// A missing value, or an even first byte, means enabled.
pub fn approved_is_enabled(bytes: &[u8]) -> bool {
    bytes.first().is_none_or(|b| b & 1 == 0)
}

/// The 12-byte value Task Manager writes: `02` + zeros when enabled,
/// `03 00 00 00` + the FILETIME it was disabled at when disabled.
pub fn approved_value(enabled: bool, now_filetime: u64) -> [u8; 12] {
    let mut v = [0u8; 12];
    if enabled {
        v[0] = 0x02;
    } else {
        v[0] = 0x03;
        v[4..12].copy_from_slice(&now_filetime.to_le_bytes());
    }
    v
}
