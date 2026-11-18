//! The real folder layout, from Windows' known folders.

use wincare_core::{Guard, SkipReason};
use wincare_winsys::system_env;

#[test]
fn system_env_points_at_real_folders() {
    let env = system_env().unwrap();
    assert!(env.user_profile.is_dir());
    assert!(env.temp.is_dir());
    assert!(env.windir.is_dir());
    assert!(env.known_protected.len() >= 5);
    let guard = Guard::new(&env);
    let docs = &env.known_protected[0];
    assert_eq!(guard.check(&docs.join("x.txt")), Err(SkipReason::Protected));
    assert_eq!(
        guard.check(&env.windir.join("x")),
        Err(SkipReason::Protected)
    );
}

/// The same scan the Scan button runs, on the real system. Reads only.
#[test]
fn real_cleanup_scan_runs() {
    use std::time::{SystemTime, UNIX_EPOCH};
    use wincare_core::{CancelToken, scan_cleanup};

    let env = system_env().unwrap();
    let guard = Guard::new(&env);
    let running = wincare_winsys::running_process_names();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    let result = scan_cleanup(
        &env,
        now,
        &|n| running.iter().any(|r| r == n),
        &CancelToken::new(),
        &mut |_, _| {},
    );

    // Everything offered must pass the guard: nothing protected, nothing outside the profile.
    assert!(result.findings.iter().all(|f| guard.check(&f.path).is_ok()));
    eprintln!(
        "real scan: {} findings, {} MB, notes: {:?}",
        result.findings.len(),
        result.findings.iter().map(|f| f.bytes).sum::<u64>() / (1024 * 1024),
        result.notes
    );
}

/// Review I2: OneDrive roots and other personal folders are protected too.
#[test]
fn system_env_protects_onedrive_and_personal_folders() {
    let env = system_env().unwrap();
    let guard = Guard::new(&env);
    let profile = &env.user_profile;
    for name in ["Saved Games", "Favorites", "Contacts", ".ssh"] {
        assert_eq!(
            guard.check(&profile.join(name)),
            Err(SkipReason::Protected),
            "{name}"
        );
    }
    for var in ["OneDrive", "OneDriveConsumer", "OneDriveCommercial"] {
        if let Some(root) = std::env::var_os(var) {
            let root = std::path::PathBuf::from(root);
            assert_eq!(guard.check(&root), Err(SkipReason::Protected), "{var}");
            assert_eq!(
                guard.check(&root.join("anything")),
                Err(SkipReason::Protected),
                "{var} contents"
            );
        }
    }
}
