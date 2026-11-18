//! Health rules: when a check warns, and what it tells the user.

use wincare_core::{CheckStatus, battery_wear_check, disk_space_check};

#[test]
fn disk_space_warns_below_threshold() {
    let low = disk_space_check('C', 9, 100, 10);
    assert_eq!(low.status, CheckStatus::Warn);
    assert_eq!(low.fix_uri.as_deref(), Some("ms-settings:storagesense"));
    assert_eq!(disk_space_check('C', 11, 100, 10).status, CheckStatus::Ok);
    assert_eq!(disk_space_check('C', 0, 0, 10).status, CheckStatus::Unknown);
}

#[test]
fn battery_wear_detail() {
    let worn = battery_wear_check(50_000, 32_000);
    assert_eq!(worn.status, CheckStatus::Warn);
    assert_eq!(worn.detail, "Holds 64 % of its original charge");
    assert_eq!(battery_wear_check(50_000, 45_000).status, CheckStatus::Ok);
    assert_eq!(battery_wear_check(0, 45_000).status, CheckStatus::Unknown);
}
