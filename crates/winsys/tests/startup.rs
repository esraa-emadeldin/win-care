//! Startup items on the real system. The toggle test only writes a value for a
//! made-up item name, and removes it again.

use winreg::RegKey;
use winreg::enums::{HKEY_CURRENT_USER, KEY_ALL_ACCESS};

use wincare_core::{StartupItem, StartupSource, approved_is_enabled};
use wincare_winsys::{list_startup, set_startup_enabled};

const APPROVED_RUN: &str =
    r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run";
const TEST_NAME: &str = "wincare-test-startup-item";

/// Deletes the test value even when an assertion fails.
struct Cleanup;
impl Drop for Cleanup {
    fn drop(&mut self) {
        if let Ok(k) =
            RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(APPROVED_RUN, KEY_ALL_ACCESS)
        {
            let _ = k.delete_value(TEST_NAME);
        }
    }
}

fn item(source: StartupSource) -> StartupItem {
    StartupItem {
        id: format!("{}:{TEST_NAME}", source_tag(source)),
        name: TEST_NAME.into(),
        command: "notepad.exe".into(),
        publisher: None,
        source,
        enabled: true,
        can_toggle: source.can_toggle(),
    }
}

fn source_tag(source: StartupSource) -> &'static str {
    match source {
        StartupSource::HkcuRun => "hkcurun",
        StartupSource::HklmRun => "hklmrun",
        _ => "other",
    }
}

#[test]
fn list_startup_has_unique_ids() {
    let items = list_startup();
    let mut ids: Vec<_> = items.iter().map(|i| i.id.clone()).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), items.len());
    assert!(items.iter().all(|i| i.can_toggle == i.source.can_toggle()));
}

#[test]
fn toggle_writes_task_manager_value() {
    let _cleanup = Cleanup;
    let it = item(StartupSource::HkcuRun);

    set_startup_enabled(&it, false).unwrap();
    let key = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(APPROVED_RUN)
        .unwrap();
    let raw = key.get_raw_value(TEST_NAME).unwrap();
    assert!(!approved_is_enabled(&raw.bytes));

    set_startup_enabled(&it, true).unwrap();
    let raw = key.get_raw_value(TEST_NAME).unwrap();
    assert!(approved_is_enabled(&raw.bytes));
}

#[test]
fn machine_wide_items_need_admin() {
    let err = set_startup_enabled(&item(StartupSource::HklmRun), false).unwrap_err();
    assert_eq!(err, "needs admin");
}
