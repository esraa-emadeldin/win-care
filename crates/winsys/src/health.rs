//! Read-only health checks. Anything that cannot be read becomes `Unknown`,
//! never an error, so one broken source cannot hide the others.

use std::collections::HashMap;
use std::thread;

use serde::Deserialize;
use winreg::RegKey;
use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_READ};
use wmi::WMIConnection;

use wincare_core::{Check, CheckStatus, battery_wear_check, disk_space_check};

use crate::vitals::{drives, has_battery};

const FIREWALL: &str = r"SYSTEM\CurrentControlSet\Services\SharedAccess\Parameters\FirewallPolicy";
const REBOOT_REQUIRED: &str =
    r"SOFTWARE\Microsoft\Windows\CurrentVersion\WindowsUpdate\Auto Update\RebootRequired";

pub fn health_checks(alert_pct: u8) -> Vec<Check> {
    let mut checks: Vec<Check> = drives()
        .iter()
        .map(|d| disk_space_check(d.letter, d.free, d.total, alert_pct))
        .collect();
    // WMI wants its own COM mode; a fresh thread keeps it away from the
    // threads that use the shell for the Recycle Bin.
    let wmi_checks = thread::spawn(wmi_checks).join().unwrap_or_default();
    checks.extend(wmi_checks);
    checks.push(firewall_check());
    checks.push(restart_check());
    checks
}

fn wmi_checks() -> Vec<Check> {
    let mut out = disk_health();
    out.push(antivirus_check());
    if has_battery() {
        out.push(battery_check());
    }
    out
}

#[derive(Deserialize)]
#[serde(rename = "MSFT_PhysicalDisk", rename_all = "PascalCase")]
struct PhysicalDisk {
    friendly_name: Option<String>,
    health_status: Option<u16>,
}

fn disk_health() -> Vec<Check> {
    let disks: Option<Vec<PhysicalDisk>> =
        WMIConnection::with_namespace_path(r"root\Microsoft\Windows\Storage")
            .ok()
            .and_then(|c| c.query().ok());
    let Some(disks) = disks else {
        return vec![unknown("disk-health", "Disk health")];
    };
    disks
        .iter()
        .enumerate()
        .map(|(i, d)| {
            let name = d
                .friendly_name
                .clone()
                .unwrap_or_else(|| format!("Disk {i}"));
            // 0 Healthy, 1 Warning, 2 Unhealthy, 5 Unknown.
            let (status, detail) = match d.health_status {
                Some(0) => (CheckStatus::Ok, "Healthy".to_string()),
                Some(1 | 2) => (
                    CheckStatus::Warn,
                    "Windows reports a problem; back up your files".to_string(),
                ),
                _ => (CheckStatus::Unknown, "Status not reported".to_string()),
            };
            Check {
                id: format!("disk-health-{i}"),
                title: format!("Health of {name}"),
                status,
                detail,
                fix_uri: Some("ms-settings:disksandvolumes".into()),
            }
        })
        .collect()
}

#[derive(Deserialize)]
#[serde(rename = "AntiVirusProduct", rename_all = "camelCase")]
struct AntiVirusProduct {
    display_name: Option<String>,
    product_state: Option<u32>,
}

/// On when any registered antivirus (Defender or another) reports itself on.
fn antivirus_check() -> Check {
    let products: Option<Vec<AntiVirusProduct>> =
        WMIConnection::with_namespace_path(r"root\SecurityCenter2")
            .ok()
            .and_then(|c| c.query().ok());
    let title = "Antivirus protection".to_string();
    let fix_uri = Some("windowsdefender:".to_string());
    let Some(products) = products else {
        return Check {
            id: "antivirus".into(),
            ..unknown("antivirus", &title)
        };
    };
    // Bits 12-15 of productState: 1 means real-time protection is on.
    let on: Vec<String> = products
        .iter()
        .filter(|p| p.product_state.is_some_and(|s| (s >> 12) & 0xF == 1))
        .map(|p| p.display_name.clone().unwrap_or_default())
        .collect();
    let (status, detail) = if on.is_empty() {
        (CheckStatus::Warn, "No antivirus is turned on".to_string())
    } else {
        (CheckStatus::Ok, format!("On: {}", on.join(", ")))
    };
    Check {
        id: "antivirus".into(),
        title,
        status,
        detail,
        fix_uri,
    }
}

#[derive(Deserialize)]
#[serde(rename = "BatteryStaticData", rename_all = "PascalCase")]
struct BatteryStaticData {
    designed_capacity: Option<u32>,
}

#[derive(Deserialize)]
#[serde(rename = "BatteryFullChargedCapacity", rename_all = "PascalCase")]
struct BatteryFullChargedCapacity {
    full_charged_capacity: Option<u32>,
}

fn battery_check() -> Check {
    let Ok(conn) = WMIConnection::with_namespace_path(r"root\wmi") else {
        return battery_wear_check(0, 0);
    };
    let design = conn
        .query::<BatteryStaticData>()
        .ok()
        .and_then(|v| v.first().and_then(|b| b.designed_capacity));
    let full = conn
        .query::<BatteryFullChargedCapacity>()
        .ok()
        .and_then(|v| v.first().and_then(|b| b.full_charged_capacity));
    battery_wear_check(design.unwrap_or(0), full.unwrap_or(0))
}

fn firewall_check() -> Check {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let profiles = [
        ("DomainProfile", "domain"),
        ("StandardProfile", "private"),
        ("PublicProfile", "public"),
    ];
    let state: HashMap<&str, Option<u32>> = profiles
        .iter()
        .map(|(key, label)| {
            let on = hklm
                .open_subkey_with_flags(format!(r"{FIREWALL}\{key}"), KEY_READ)
                .and_then(|k| k.get_value::<u32, _>("EnableFirewall"))
                .ok();
            (*label, on)
        })
        .collect();
    let (status, detail) = if state.values().any(Option::is_none) {
        (
            CheckStatus::Unknown,
            "Could not read the firewall settings".to_string(),
        )
    } else {
        let off: Vec<&str> = profiles
            .iter()
            .map(|(_, l)| *l)
            .filter(|l| state[l] == Some(0))
            .collect();
        if off.is_empty() {
            (CheckStatus::Ok, "On for all networks".to_string())
        } else {
            (
                CheckStatus::Warn,
                format!("Off for {} networks", off.join(", ")),
            )
        }
    };
    Check {
        id: "firewall".into(),
        title: "Windows Firewall".into(),
        status,
        detail,
        fix_uri: Some("windowsdefender://network".into()),
    }
}

fn restart_check() -> Check {
    let pending = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey_with_flags(REBOOT_REQUIRED, KEY_READ)
        .is_ok();
    Check {
        id: "restart".into(),
        title: "Windows Update".into(),
        status: if pending {
            CheckStatus::Warn
        } else {
            CheckStatus::Ok
        },
        detail: if pending {
            "Restart to finish installing updates".into()
        } else {
            "No restart needed".into()
        },
        fix_uri: Some("ms-settings:windowsupdate".into()),
    }
}

fn unknown(id: &str, title: &str) -> Check {
    Check {
        id: id.into(),
        title: title.into(),
        status: CheckStatus::Unknown,
        detail: "Could not be read".into(),
        fix_uri: None,
    }
}
