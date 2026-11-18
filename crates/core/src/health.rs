//! Health checks: read-only, each with plain words and a Settings page to fix it.

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CheckStatus {
    Ok,
    Warn,
    Unknown,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Check {
    pub id: String,
    pub title: String,
    pub status: CheckStatus,
    pub detail: String,
    /// A `ms-settings:` (or similar) link that opens the right Settings page.
    pub fix_uri: Option<String>,
}

/// Battery capacity below this share of its design capacity is worth a mention.
const BATTERY_WEAR_WARN_PCT: u64 = 70;

pub fn disk_space_check(letter: char, free: u64, total: u64, alert_pct: u8) -> Check {
    // checked_div is None for a drive whose size could not be read (0).
    let (status, detail) = match (free * 100).checked_div(total) {
        None => (
            CheckStatus::Unknown,
            "Could not read the drive size".to_string(),
        ),
        Some(free_pct) => {
            let status = if free_pct < u64::from(alert_pct) {
                CheckStatus::Warn
            } else {
                CheckStatus::Ok
            };
            (status, format!("{} free of {}", gb(free), gb(total)))
        }
    };
    Check {
        id: format!("disk-{}", letter.to_ascii_lowercase()),
        title: format!("Free space on {letter}:"),
        status,
        detail,
        fix_uri: Some("ms-settings:storagesense".into()),
    }
}

pub fn battery_wear_check(design_mwh: u32, full_mwh: u32) -> Check {
    let (status, detail) = if design_mwh == 0 {
        (
            CheckStatus::Unknown,
            "Could not read the battery capacity".to_string(),
        )
    } else {
        let pct = u64::from(full_mwh) * 100 / u64::from(design_mwh);
        let status = if pct < BATTERY_WEAR_WARN_PCT {
            CheckStatus::Warn
        } else {
            CheckStatus::Ok
        };
        (status, format!("Holds {pct} % of its original charge"))
    };
    Check {
        id: "battery".into(),
        title: "Battery health".into(),
        status,
        detail,
        fix_uri: Some("ms-settings:batterysaver".into()),
    }
}

fn gb(bytes: u64) -> String {
    format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
}
