//! Cleanup scanners: what counts as junk, and what is pre-selected.

use std::fs::{self, FileTimes};
use std::os::windows::fs::{FileTimesExt, OpenOptionsExt};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use wincare_core::{CancelToken, CategoryId, CleanupResult, Env, scan_cleanup};

const DAY: Duration = Duration::from_secs(24 * 60 * 60);

fn write_aged(path: &Path, bytes: usize, age: Duration) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, vec![b'x'; bytes]).unwrap();
    let when = SystemTime::now() - age;
    let file = fs::File::options().write(true).open(path).unwrap();
    file.set_times(FileTimes::new().set_modified(when).set_created(when))
        .unwrap();
}

/// Backdates a folder's creation time (folders need the backup-semantics flag).
fn age_dir(path: &Path, age: Duration) {
    const FILE_WRITE_ATTRIBUTES: u32 = 0x100;
    const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
    let when = SystemTime::now() - age;
    let dir = fs::OpenOptions::new()
        .access_mode(FILE_WRITE_ATTRIBUTES)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS)
        .open(path)
        .unwrap();
    dir.set_times(FileTimes::new().set_modified(when).set_created(when))
        .unwrap();
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

fn scan(env: &Env, running: &[&str]) -> CleanupResult {
    let running: Vec<String> = running.iter().map(|s| s.to_string()).collect();
    scan_cleanup(
        env,
        now_ms(),
        &|name| running.iter().any(|r| r == name),
        &CancelToken::new(),
        &mut |_, _| {},
    )
}

fn paths(result: &CleanupResult, category: CategoryId) -> Vec<PathBuf> {
    result
        .findings
        .iter()
        .filter(|f| f.category == category)
        .map(|f| f.path.clone())
        .collect()
}

#[test]
fn temp_only_old_items() {
    let tmp = tempfile::tempdir().unwrap();
    let env = Env::for_tests(tmp.path());
    write_aged(&env.temp.join("old.tmp"), 10, 2 * DAY);
    write_aged(&env.temp.join("new.tmp"), 10, Duration::ZERO);

    let result = scan(&env, &[]);

    assert_eq!(
        paths(&result, CategoryId::UserTemp),
        vec![env.temp.join("old.tmp")]
    );
    assert!(result.findings[0].preselected);
}

#[test]
fn browser_cache_skipped_while_running() {
    let tmp = tempfile::tempdir().unwrap();
    let env = Env::for_tests(tmp.path());
    let cache = env
        .local_app_data
        .join(r"Google\Chrome\User Data\Default\Cache");
    for i in 0..3 {
        write_aged(&cache.join(format!("f{i}")), 100, DAY);
    }
    let edge = env
        .local_app_data
        .join(r"Microsoft\Edge\User Data\Default\Cache");
    write_aged(&edge.join("e0"), 50, DAY);

    let result = scan(&env, &["chrome.exe"]);

    assert!(paths(&result, CategoryId::BrowserCache).contains(&edge));
    assert!(!paths(&result, CategoryId::BrowserCache).contains(&cache));
    assert!(
        result.notes.iter().any(|n| n.contains("Chrome is running")),
        "notes: {:?}",
        result.notes
    );
}

#[test]
fn old_installers_never_preselected() {
    let tmp = tempfile::tempdir().unwrap();
    let env = Env::for_tests(tmp.path());
    write_aged(&env.downloads.join("setup.exe"), 10, 40 * DAY);
    write_aged(&env.downloads.join("fresh.msi"), 10, DAY);
    write_aged(&env.downloads.join("photo.jpg"), 10, 40 * DAY);

    let result = scan(&env, &[]);

    let found: Vec<_> = result
        .findings
        .iter()
        .filter(|f| f.category == CategoryId::OldInstallers)
        .collect();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].path, env.downloads.join("setup.exe"));
    assert!(!found[0].preselected);
}

#[test]
fn long_and_unicode_paths() {
    let tmp = tempfile::tempdir().unwrap();
    let env = Env::for_tests(tmp.path());
    let mut deep = env.temp.join("deep");
    while deep.as_os_str().len() < 300 {
        deep = deep.join("nested-folder-name");
    }
    write_aged(&deep.join("résumé-日本.tmp"), 42, 2 * DAY);
    age_dir(&env.temp.join("deep"), 2 * DAY);

    let result = scan(&env, &[]);

    let found = result
        .findings
        .iter()
        .find(|f| f.path == env.temp.join("deep"))
        .expect("deep folder found");
    assert_eq!(found.bytes, 42);
    assert!(found.is_dir);
}

#[test]
fn sizes_are_measured() {
    let tmp = tempfile::tempdir().unwrap();
    let env = Env::for_tests(tmp.path());
    let dir = env.local_app_data.join("CrashDumps");
    write_aged(&dir.join("a.dmp"), 10, DAY);
    write_aged(&dir.join("b.dmp"), 20, DAY);
    write_aged(&dir.join("c.dmp"), 30, DAY);

    let result = scan(&env, &[]);

    let total: u64 = result
        .findings
        .iter()
        .filter(|f| f.category == CategoryId::CrashDumps)
        .map(|f| f.bytes)
        .sum();
    assert_eq!(total, 60);
}

/// Review I3: a zip opened a minute ago extracts files with their old dates.
/// The folder is new, so it is in use, not junk.
#[test]
fn temp_folder_extracted_just_now_is_not_offered() {
    let tmp = tempfile::tempdir().unwrap();
    let env = Env::for_tests(tmp.path());
    let folder = env.temp.join("Temp1_photos.zip");
    fs::create_dir_all(&folder).unwrap();
    fs::write(folder.join("readme.txt"), b"old content").unwrap();
    fs::File::options()
        .write(true)
        .open(folder.join("readme.txt"))
        .unwrap()
        .set_modified(SystemTime::now() - 10 * DAY)
        .unwrap();

    let result = scan(&env, &[]);

    assert!(
        paths(&result, CategoryId::UserTemp).is_empty(),
        "{:?}",
        result.findings
    );
}

/// A file copied into temp just now keeps its old modified date but is new.
#[test]
fn temp_file_created_just_now_is_not_offered() {
    let tmp = tempfile::tempdir().unwrap();
    let env = Env::for_tests(tmp.path());
    let file = env.temp.join("setup-helper.dll");
    fs::write(&file, b"x").unwrap();
    fs::File::options()
        .write(true)
        .open(&file)
        .unwrap()
        .set_modified(SystemTime::now() - 10 * DAY)
        .unwrap();

    assert!(paths(&scan(&env, &[]), CategoryId::UserTemp).is_empty());
}

/// Review I5: a folder whose contents cannot all be read may hold recent or
/// unknown files, so it is offered but never pre-selected.
#[test]
fn folder_with_unreadable_part_is_not_preselected() {
    let tmp = tempfile::tempdir().unwrap();
    let env = Env::for_tests(tmp.path());
    let folder = env.temp.join("old-tool");
    write_aged(&folder.join("a.log"), 10, 3 * DAY);
    let locked = folder.join("locked");
    write_aged(&locked.join("b.log"), 10, 3 * DAY);
    age_dir(&locked, 3 * DAY);
    age_dir(&folder, 3 * DAY);
    let user = std::env::var("USERNAME").unwrap();
    let deny = |on: bool| {
        let args = if on {
            vec!["/deny".to_string(), format!("{user}:(RX)")]
        } else {
            vec!["/remove:d".to_string(), user.clone()]
        };
        std::process::Command::new("icacls")
            .arg(&locked)
            .args(args)
            .output()
            .unwrap();
    };
    deny(true);
    let result = scan(&env, &[]);
    deny(false);

    let f = result
        .findings
        .iter()
        .find(|f| f.path == folder)
        .expect("folder still offered");
    assert!(!f.preselected);
    assert!(
        f.note
            .as_deref()
            .unwrap_or("")
            .contains("could not be read")
    );
    assert_eq!(result.unreadable_dirs, 1);
}
