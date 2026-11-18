//! Live numbers for the tray popup, and the list of fixed drives.

use std::time::Instant;

use serde::{Deserialize, Serialize};
use sysinfo::{Disks, Networks, System};
use windows::Win32::Storage::FileSystem::{GetDiskFreeSpaceExW, GetDriveTypeW, GetLogicalDrives};
use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
use windows::core::HSTRING;

/// `DRIVE_FIXED` from GetDriveTypeW: internal disks, not USB sticks or DVDs.
const DRIVE_FIXED: u32 = 3;
/// `BatteryFlag` value meaning the PC has no battery.
const NO_BATTERY: u8 = 128;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Drive {
    pub letter: char,
    pub total: u64,
    pub free: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Vitals {
    pub cpu_pct: f32,
    pub ram_used: u64,
    pub ram_total: u64,
    pub disk_used: u64,
    pub disk_total: u64,
    pub battery_pct: Option<u8>,
    pub charging: Option<bool>,
    pub net_rx_bps: u64,
    pub net_tx_bps: u64,
}

/// Keeps the previous sample, because CPU and network are rates.
pub struct VitalsSampler {
    system: System,
    networks: Networks,
    last: Instant,
}

impl VitalsSampler {
    pub fn new() -> Result<Self, String> {
        let mut system = System::new().map_err(|e| e.to_string())?;
        system.refresh_cpu_usage();
        Ok(VitalsSampler {
            system,
            networks: Networks::new_with_refreshed_list().map_err(|e| e.to_string())?,
            last: Instant::now(),
        })
    }

    pub fn sample(&mut self) -> Vitals {
        // CPU usage needs two readings a short while apart.
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        self.system.refresh_cpu_usage();
        self.system.refresh_memory();
        self.networks.refresh(true);
        let secs = self.last.elapsed().as_secs_f64().max(0.001);
        self.last = Instant::now();
        let (rx, tx) = self
            .networks
            .values()
            .fold((0, 0), |(r, t), n| (r + n.received(), t + n.transmitted()));

        let system_drive = std::env::var("SystemDrive").unwrap_or_else(|_| "C:".into());
        let disks = Disks::new_with_refreshed_list().ok();
        let disk = disks.as_ref().and_then(|d| {
            d.iter()
                .find(|d| d.mount_point().to_string_lossy().starts_with(&system_drive))
        });
        let (battery_pct, charging) = battery();

        Vitals {
            cpu_pct: self.system.global_cpu_usage().clamp(0.0, 100.0),
            ram_used: self.system.used_memory(),
            ram_total: self.system.total_memory(),
            disk_used: disk.map_or(0, |d| d.total_space() - d.available_space()),
            disk_total: disk.map_or(0, |d| d.total_space()),
            battery_pct,
            charging,
            net_rx_bps: (rx as f64 / secs) as u64,
            net_tx_bps: (tx as f64 / secs) as u64,
        }
    }
}

fn power_status() -> Option<SYSTEM_POWER_STATUS> {
    let mut status = SYSTEM_POWER_STATUS::default();
    unsafe { GetSystemPowerStatus(&mut status) }.ok()?;
    Some(status)
}

pub(crate) fn has_battery() -> bool {
    power_status().is_some_and(|s| s.BatteryFlag != NO_BATTERY)
}

fn battery() -> (Option<u8>, Option<bool>) {
    match power_status() {
        Some(s) if s.BatteryFlag != NO_BATTERY && s.BatteryLifePercent <= 100 => {
            (Some(s.BatteryLifePercent), Some(s.ACLineStatus == 1))
        }
        _ => (None, None),
    }
}

/// Fixed drives with their size and free space.
pub fn drives() -> Vec<Drive> {
    let mask = unsafe { GetLogicalDrives() };
    (0..26u8)
        .filter(|i| mask & (1 << i) != 0)
        .filter_map(|i| {
            let letter = (b'A' + i) as char;
            let root = HSTRING::from(format!("{letter}:\\"));
            if unsafe { GetDriveTypeW(&root) } != DRIVE_FIXED {
                return None;
            }
            let (mut free, mut total) = (0u64, 0u64);
            unsafe { GetDiskFreeSpaceExW(&root, Some(&mut free), Some(&mut total), None) }.ok()?;
            Some(Drive {
                letter,
                total,
                free,
            })
        })
        .collect()
}
