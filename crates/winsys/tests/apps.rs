//! Reads the real list of installed apps (read-only).

use wincare_winsys::list_apps;

#[test]
fn list_apps_finds_something() {
    let apps = list_apps();
    assert!(!apps.is_empty(), "no installed apps found");
    assert!(apps.iter().all(|a| !a.name.trim().is_empty()));
    let mut ids: Vec<_> = apps.iter().map(|a| a.id.clone()).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), apps.len(), "app ids must be unique");
}
