//! The real Recycle Bin. These tests put a few tiny temp files in it.

use std::fs::{self, OpenOptions};
use std::os::windows::fs::OpenOptionsExt;
use std::path::PathBuf;

use wincare_core::FileOps;
use wincare_winsys::{WinFileOps, recycle_bin_size};

fn temp_file(dir: &tempfile::TempDir, name: &str, bytes: usize) -> PathBuf {
    let path = dir.path().join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, vec![b'x'; bytes]).unwrap();
    path
}

#[test]
fn recycle_moves_file_to_bin() {
    let dir = tempfile::tempdir().unwrap();
    let path = temp_file(&dir, "wincare-recycle-test.txt", 4096);
    let before = recycle_bin_size();

    WinFileOps.recycle(&path).unwrap();

    assert!(!path.exists());
    assert!(recycle_bin_size() > before, "bin did not grow");
}

#[test]
fn recycle_long_unicode_path() {
    let dir = tempfile::tempdir().unwrap();
    let mut rel = PathBuf::from("wincare-long");
    while dir.path().join(&rel).as_os_str().len() < 300 {
        rel = rel.join("nested-folder-name");
    }
    let path = temp_file(&dir, rel.join("日本.txt").to_str().unwrap(), 10);

    WinFileOps.recycle(&path).unwrap();

    assert!(!path.exists());
}

#[test]
fn in_use_detects_open_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = temp_file(&dir, "locked.log", 10);
    let free = temp_file(&dir, "free.log", 10);
    let _held = OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(&path)
        .unwrap();

    assert!(WinFileOps.in_use(&path));
    assert!(!WinFileOps.in_use(&free));
    assert!(!WinFileOps.in_use(dir.path()), "folders are never 'in use'");
}

#[test]
fn bin_info_for_system_drive() {
    let dir = tempfile::tempdir().unwrap();
    let info = WinFileOps.bin_info(dir.path());
    // The test machine keeps its Recycle Bin on for the system drive.
    assert!(info.enabled);
    assert!(info.max_bytes > 0);
}
