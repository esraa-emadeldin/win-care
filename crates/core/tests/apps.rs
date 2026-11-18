//! Leftovers after an uninstall: found by name, never another app's folder.

use std::fs;
use std::path::{Path, PathBuf};

use wincare_core::{App, AppKind, CategoryId, Env, find_leftovers};

fn app(name: &str, publisher: Option<&str>, install: Option<PathBuf>) -> App {
    App {
        id: name.to_lowercase(),
        name: name.into(),
        publisher: publisher.map(Into::into),
        version: None,
        bytes: None,
        last_used_ms: None,
        kind: AppKind::Win32 {
            uninstall_cmd: "uninstall.exe".into(),
            quiet_cmd: None,
            install_location: install,
        },
    }
}

fn file(path: &Path, bytes: usize) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, vec![b'x'; bytes]).unwrap();
}

#[test]
fn leftovers_match_name_in_appdata() {
    let tmp = tempfile::tempdir().unwrap();
    let env = Env::for_tests(tmp.path());
    file(
        &env.app_data.join("Contoso Notes").join("settings.json"),
        12,
    );
    file(&env.app_data.join("Other App").join("x"), 1);

    let found = find_leftovers(&app("Contoso Notes", None, None), &[], &env);

    assert_eq!(found.len(), 1);
    assert_eq!(found[0].path, env.app_data.join("Contoso Notes"));
    assert_eq!(found[0].bytes, 12);
    assert_eq!(found[0].category, CategoryId::AppLeftovers);
    assert!(found[0].selectable && found[0].preselected);
}

#[test]
fn leftovers_under_publisher_folder() {
    let tmp = tempfile::tempdir().unwrap();
    let env = Env::for_tests(tmp.path());
    file(
        &env.local_app_data
            .join("Contoso")
            .join("Notes")
            .join("cache"),
        5,
    );

    let found = find_leftovers(&app("Notes", Some("Contoso"), None), &[], &env);

    assert_eq!(found.len(), 1);
    assert_eq!(
        found[0].path,
        env.local_app_data.join("Contoso").join("Notes")
    );
}

#[test]
fn leftovers_outside_profile_not_selectable() {
    let tmp = tempfile::tempdir().unwrap();
    let env = Env::for_tests(tmp.path());
    file(
        &env.program_files[0].join("Contoso Notes").join("app.exe"),
        3,
    );

    let found = find_leftovers(&app("Contoso Notes", None, None), &[], &env);

    assert_eq!(found.len(), 1);
    assert!(!found[0].selectable);
    assert!(!found[0].preselected);
    assert_eq!(
        found[0].note.as_deref(),
        Some("Needs admin; not removed in v1")
    );
}

#[test]
fn leftovers_skip_folder_of_other_app() {
    let tmp = tempfile::tempdir().unwrap();
    let env = Env::for_tests(tmp.path());
    let shared = env.program_data.join("Contoso");
    file(&shared.join("Paint").join("paint.exe"), 3);
    let other = app("Contoso Paint", Some("Contoso"), Some(shared.join("Paint")));

    let found = find_leftovers(&app("Contoso", None, None), &[other], &env);

    assert!(found.is_empty(), "found: {found:?}");
}

/// Review I6: the same app registered twice (per-machine and per-user) is
/// still installed after one entry is removed.
#[test]
fn still_installed_matches_by_name_not_id() {
    let target = app("Contoso Notes", Some("Contoso"), None);
    let mut other_copy = app("Contoso Notes", Some("Contoso"), None);
    other_copy.id = "hkcu:contoso-notes".into();
    other_copy.version = Some("2.0".into());

    assert!(wincare_core::still_installed(&target, &[other_copy]));
    assert!(!wincare_core::still_installed(
        &target,
        &[app("Other", None, None)]
    ));
}

/// Review I6: a folder named after another installed app is never a leftover,
/// even when that app records no install location.
#[test]
fn leftovers_skip_folder_named_after_installed_app() {
    let tmp = tempfile::tempdir().unwrap();
    let env = Env::for_tests(tmp.path());
    file(
        &env.app_data.join("Contoso Notes").join("settings.json"),
        12,
    );
    let installed_copy = app("Contoso Notes", None, None);

    let found = find_leftovers(&app("Contoso Notes", None, None), &[installed_copy], &env);

    assert!(found.is_empty(), "{found:?}");
}
