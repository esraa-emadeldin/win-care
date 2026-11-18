//! Health, drives and vitals on the real system (read-only).

use wincare_core::CheckStatus;
use wincare_winsys::{VitalsSampler, drives, health_checks};

#[test]
fn drives_include_the_system_drive() {
    let system = std::env::var("SystemDrive").unwrap();
    let letter = system.chars().next().unwrap();
    let all = drives();
    let c = all
        .iter()
        .find(|d| d.letter == letter)
        .expect("system drive");
    assert!(c.total > 0 && c.free <= c.total);
}

#[test]
fn health_checks_cover_disk_space_and_security() {
    let checks = health_checks(10);
    let system = std::env::var("SystemDrive").unwrap().to_lowercase();
    let disk_id = format!("disk-{}", &system[..1]);
    assert!(checks.iter().any(|c| c.id == disk_id), "{checks:?}");
    for id in ["antivirus", "firewall", "restart"] {
        assert!(checks.iter().any(|c| c.id == id), "missing {id}");
    }
    assert!(checks.iter().all(|c| !c.title.is_empty()));
    // A check that cannot be read says so; it never errors.
    assert!(checks.iter().all(|c| matches!(
        c.status,
        CheckStatus::Ok | CheckStatus::Warn | CheckStatus::Unknown
    )));
}

#[test]
fn vitals_are_in_range() {
    let mut sampler = VitalsSampler::new().unwrap();
    let v = sampler.sample();
    assert!((0.0..=100.0).contains(&v.cpu_pct));
    assert!(v.ram_total > 0 && v.ram_used <= v.ram_total);
    assert!(v.disk_total > 0 && v.disk_used <= v.disk_total);
    if let Some(pct) = v.battery_pct {
        assert!(pct <= 100);
    }
}
