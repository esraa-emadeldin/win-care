//! Guard and walker: the two pieces that keep scans away from what must never be touched.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use wincare_core::{CancelToken, Env, Guard, SkipReason, walk};

fn write(path: &Path, bytes: usize) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, vec![b'x'; bytes]).unwrap();
}

fn documents(env: &Env) -> PathBuf {
    env.user_profile.join("Documents")
}

#[test]
fn guard_blocks_known_folders() {
    let tmp = tempfile::tempdir().unwrap();
    let env = Env::for_tests(tmp.path());
    let guard = Guard::new(&env);

    assert_eq!(
        guard.check(&documents(&env).join("a.txt")),
        Err(SkipReason::Protected)
    );
    assert_eq!(guard.check(&env.temp.join("a.tmp")), Ok(()));
    assert_eq!(
        guard.check(&env.windir.join("x")),
        Err(SkipReason::Protected)
    );
    // The profile itself and anything outside it are never deletable.
    assert_eq!(guard.check(&env.user_profile), Err(SkipReason::Protected));
    assert_eq!(
        guard.check(&env.program_data.join("x")),
        Err(SkipReason::Protected)
    );
    // ".." cannot be used to climb out of an allowed folder.
    assert_eq!(
        guard.check(&env.temp.join("..").join("..").join("..").join("Documents")),
        Err(SkipReason::Protected)
    );
}

#[test]
fn walk_skips_junctions() {
    let tmp = tempfile::tempdir().unwrap();
    let env = Env::for_tests(tmp.path());
    write(&documents(&env).join("big.bin"), 1024 * 1024);
    write(&env.temp.join("small.tmp"), 10);
    junction::create(documents(&env), env.temp.join("inner")).unwrap();

    let mut seen = Vec::new();
    let stats = walk(&env.temp, &CancelToken::new(), &mut |p, _| {
        seen.push(p.to_path_buf())
    });

    assert_eq!(stats.bytes, 10, "the junction's target must not be counted");
    assert_eq!(stats.files, 1);
    assert!(seen.iter().all(|p| !p.starts_with(env.temp.join("inner"))));
    let guard = Guard::new(&env);
    assert_eq!(
        guard.check(&env.temp.join("inner").join("big.bin")),
        Err(SkipReason::Protected)
    );
    assert_eq!(
        guard.check(&env.temp.join("inner")),
        Err(SkipReason::Protected)
    );
}

/// Denies the current user list access to a folder, and restores it on drop.
struct DenyList(PathBuf);

impl DenyList {
    fn new(dir: &Path) -> Self {
        let user = std::env::var("USERNAME").unwrap();
        let ok = Command::new("icacls")
            .arg(dir)
            .args(["/deny", &format!("{user}:(RX)")])
            .output()
            .unwrap()
            .status
            .success();
        assert!(ok, "icacls /deny failed");
        DenyList(dir.to_path_buf())
    }
}

impl Drop for DenyList {
    fn drop(&mut self) {
        let user = std::env::var("USERNAME").unwrap();
        let _ = Command::new("icacls")
            .arg(&self.0)
            .args(["/remove:d", &user])
            .output();
    }
}

#[test]
fn walk_counts_unreadable_dirs() {
    let tmp = tempfile::tempdir().unwrap();
    let env = Env::for_tests(tmp.path());
    write(&env.temp.join("ok.tmp"), 5);
    let locked = env.temp.join("locked");
    write(&locked.join("hidden.tmp"), 7);
    let _deny = DenyList::new(&locked);

    let stats = walk(&env.temp, &CancelToken::new(), &mut |_, _| {});

    assert_eq!(stats.unreadable_dirs, 1);
    assert_eq!(stats.files, 1);
    assert_eq!(stats.bytes, 5);
}

#[test]
fn walk_honours_cancel() {
    let tmp = tempfile::tempdir().unwrap();
    let env = Env::for_tests(tmp.path());
    write(&env.temp.join("a.tmp"), 5);
    let cancel = CancelToken::new();
    cancel.cancel();

    let stats = walk(&env.temp, &cancel, &mut |_, _| {});

    assert_eq!(stats.files, 0);
}

/// Review C1: with OneDrive folder backup, Documents lives at
/// profile\OneDrive\Documents. The OneDrive folder contains it, so it must be refused.
#[test]
fn guard_refuses_parent_of_protected_folder() {
    let tmp = tempfile::tempdir().unwrap();
    let mut env = Env::for_tests(tmp.path());
    let onedrive = env.user_profile.join("OneDrive");
    env.known_protected.push(onedrive.join("Documents"));
    fs::create_dir_all(onedrive.join("Documents")).unwrap();
    let guard = Guard::new(&env);

    assert_eq!(guard.check(&onedrive), Err(SkipReason::Protected));
    assert_eq!(
        guard.check(&env.user_profile.join("AppData")),
        Err(SkipReason::Protected)
    );
}

/// Review I2: the roots that hold everything (AppData, Local, Roaming, Temp,
/// Downloads) are never removed whole, but what is inside them may be.
#[test]
fn guard_refuses_container_roots_but_allows_their_contents() {
    let tmp = tempfile::tempdir().unwrap();
    let env = Env::for_tests(tmp.path());
    let guard = Guard::new(&env);
    let roots = [
        env.user_profile.join("AppData"),
        env.local_app_data.clone(),
        env.user_profile.join("AppData").join("LocalLow"),
        env.app_data.clone(),
        env.temp.clone(),
        env.downloads.clone(),
    ];
    for root in &roots {
        assert_eq!(guard.check(root), Err(SkipReason::Protected), "{root:?}");
        assert_eq!(guard.check(&root.join("child")), Ok(()), "{root:?} child");
    }
}
