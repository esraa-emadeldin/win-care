//! The Recycle Bin, through the Windows shell. The progress sink below vetoes
//! any delete the shell would do permanently (bin off, item too big), so
//! "move to Recycle Bin" can never silently become a permanent delete.

use std::cell::Cell;
use std::fs::{self, OpenOptions};
use std::os::windows::fs::OpenOptionsExt;
use std::path::Path;

use windows::Win32::Foundation::E_ABORT;
use windows::Win32::Storage::FileSystem::{
    GetDiskFreeSpaceExW, GetVolumeNameForVolumeMountPointW, GetVolumePathNameW,
};
use windows::Win32::System::Com::{
    CLSCTX_ALL, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
};
use windows::Win32::UI::Shell::{
    FOF_ALLOWUNDO, FOF_NOCONFIRMATION, FOF_NOERRORUI, FOF_SILENT, FOFX_RECYCLEONDELETE,
    FileOperation, IFileOperation, IFileOperationProgressSink, IFileOperationProgressSink_Impl,
    IShellItem, SHCreateItemFromParsingName, SHERB_NOCONFIRMATION, SHERB_NOPROGRESSUI,
    SHERB_NOSOUND, SHEmptyRecycleBinW, SHQUERYRBINFO, SHQueryRecycleBinW,
};
use windows::core::{HRESULT, HSTRING, PCWSTR, Ref};
use windows_core::implement;
use winreg::RegKey;
use winreg::enums::HKEY_CURRENT_USER;

use wincare_core::{BinInfo, FileOps};

/// `TSF_DELETE_RECYCLE_IF_POSSIBLE`: the shell will recycle, not destroy.
const TSF_DELETE_RECYCLE_IF_POSSIBLE: u32 = 0x80;
const ERROR_SHARING_VIOLATION: i32 = 32;
const ERROR_LOCK_VIOLATION: i32 = 33;

/// `FileOps` for the real system.
pub struct WinFileOps;

impl FileOps for WinFileOps {
    fn bin_info(&self, path: &Path) -> BinInfo {
        bin_info(path)
    }

    fn in_use(&self, path: &Path) -> bool {
        if path.is_dir() {
            return false;
        }
        // Opening with no sharing fails while another program has it open.
        match OpenOptions::new().read(true).share_mode(0).open(path) {
            Ok(_) => false,
            Err(e) => matches!(
                e.raw_os_error(),
                Some(ERROR_SHARING_VIOLATION | ERROR_LOCK_VIOLATION)
            ),
        }
    }

    fn recycle(&self, path: &Path) -> Result<(), String> {
        recycle(path).map_err(|e| e.message())?;
        if fs::symlink_metadata(path).is_ok() {
            return Err("could not be moved to the Recycle Bin".into());
        }
        Ok(())
    }

    fn delete(&self, path: &Path) -> Result<(), String> {
        let meta = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
        // remove_dir_all does not follow junctions or symlinks.
        let result = if meta.is_dir() {
            fs::remove_dir_all(path)
        } else {
            fs::remove_file(path)
        };
        result.map_err(|e| e.to_string())
    }
}

fn recycle(path: &Path) -> windows::core::Result<()> {
    unsafe {
        // Ignore "already initialised" results; the shell works in either mode.
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let op: IFileOperation = CoCreateInstance(&FileOperation, None, CLSCTX_ALL)?;
        op.SetOperationFlags(
            FOF_ALLOWUNDO | FOF_NOCONFIRMATION | FOF_SILENT | FOF_NOERRORUI | FOFX_RECYCLEONDELETE,
        )?;
        let item: IShellItem = SHCreateItemFromParsingName(&HSTRING::from(path), None)?;
        let sink: IFileOperationProgressSink = VetoPermanentDelete {
            vetoed: Cell::new(false),
        }
        .into();
        op.DeleteItem(&item, &sink)?;
        op.PerformOperations()
    }
}

#[implement(IFileOperationProgressSink)]
struct VetoPermanentDelete {
    vetoed: Cell<bool>,
}

impl IFileOperationProgressSink_Impl for VetoPermanentDelete_Impl {
    fn PreDeleteItem(&self, dwflags: u32, _item: Ref<IShellItem>) -> windows::core::Result<()> {
        if dwflags & TSF_DELETE_RECYCLE_IF_POSSIBLE == 0 {
            self.vetoed.set(true);
            return Err(E_ABORT.into());
        }
        Ok(())
    }

    fn StartOperations(&self) -> windows::core::Result<()> {
        Ok(())
    }
    fn FinishOperations(&self, _hr: HRESULT) -> windows::core::Result<()> {
        Ok(())
    }
    fn PreRenameItem(
        &self,
        _f: u32,
        _i: Ref<IShellItem>,
        _n: &PCWSTR,
    ) -> windows::core::Result<()> {
        Ok(())
    }
    fn PostRenameItem(
        &self,
        _f: u32,
        _i: Ref<IShellItem>,
        _n: &PCWSTR,
        _hr: HRESULT,
        _new: Ref<IShellItem>,
    ) -> windows::core::Result<()> {
        Ok(())
    }
    fn PreMoveItem(
        &self,
        _f: u32,
        _i: Ref<IShellItem>,
        _d: Ref<IShellItem>,
        _n: &PCWSTR,
    ) -> windows::core::Result<()> {
        Ok(())
    }
    fn PostMoveItem(
        &self,
        _f: u32,
        _i: Ref<IShellItem>,
        _d: Ref<IShellItem>,
        _n: &PCWSTR,
        _hr: HRESULT,
        _new: Ref<IShellItem>,
    ) -> windows::core::Result<()> {
        Ok(())
    }
    fn PreCopyItem(
        &self,
        _f: u32,
        _i: Ref<IShellItem>,
        _d: Ref<IShellItem>,
        _n: &PCWSTR,
    ) -> windows::core::Result<()> {
        Ok(())
    }
    fn PostCopyItem(
        &self,
        _f: u32,
        _i: Ref<IShellItem>,
        _d: Ref<IShellItem>,
        _n: &PCWSTR,
        _hr: HRESULT,
        _new: Ref<IShellItem>,
    ) -> windows::core::Result<()> {
        Ok(())
    }
    fn PostDeleteItem(
        &self,
        _f: u32,
        _i: Ref<IShellItem>,
        _hr: HRESULT,
        _new: Ref<IShellItem>,
    ) -> windows::core::Result<()> {
        Ok(())
    }
    fn PreNewItem(&self, _f: u32, _d: Ref<IShellItem>, _n: &PCWSTR) -> windows::core::Result<()> {
        Ok(())
    }
    fn PostNewItem(
        &self,
        _f: u32,
        _d: Ref<IShellItem>,
        _n: &PCWSTR,
        _t: &PCWSTR,
        _a: u32,
        _hr: HRESULT,
        _new: Ref<IShellItem>,
    ) -> windows::core::Result<()> {
        Ok(())
    }
    fn UpdateProgress(&self, _total: u32, _done: u32) -> windows::core::Result<()> {
        Ok(())
    }
    fn ResetTimer(&self) -> windows::core::Result<()> {
        Ok(())
    }
    fn PauseTimer(&self) -> windows::core::Result<()> {
        Ok(())
    }
    fn ResumeTimer(&self) -> windows::core::Result<()> {
        Ok(())
    }
}

/// Reads the per-volume Recycle Bin settings Explorer writes. With no stored
/// settings Windows uses its default: on, about 5 % of the volume. The veto
/// sink is the real guarantee; this only lets Review warn in advance.
fn bin_info(path: &Path) -> BinInfo {
    let Some((guid, volume_root)) = volume_of(path) else {
        return BinInfo {
            enabled: false,
            max_bytes: 0,
        };
    };
    let key = RegKey::predef(HKEY_CURRENT_USER).open_subkey(format!(
        r"Software\Microsoft\Windows\CurrentVersion\Explorer\BitBucket\Volume\{guid}"
    ));
    let nuke: u32 = key
        .as_ref()
        .ok()
        .and_then(|k| k.get_value("NukeOnDelete").ok())
        .unwrap_or(0);
    let max_mb: Option<u32> = key.ok().and_then(|k| k.get_value("MaxCapacity").ok());
    let max_bytes = match max_mb {
        Some(mb) => u64::from(mb) * 1024 * 1024,
        None => volume_size(&volume_root) / 20,
    };
    BinInfo {
        enabled: nuke == 0 && max_bytes > 0,
        max_bytes,
    }
}

/// The volume's `{GUID}` and its mount point (e.g. `C:\`).
fn volume_of(path: &Path) -> Option<(String, String)> {
    let wide = HSTRING::from(path);
    let mut root = [0u16; 1024];
    let mut name = [0u16; 128];
    unsafe {
        GetVolumePathNameW(&wide, &mut root).ok()?;
        GetVolumeNameForVolumeMountPointW(PCWSTR(root.as_ptr()), &mut name).ok()?;
    }
    let name = String::from_utf16_lossy(&name);
    let start = name.find('{')?;
    let end = name.find('}')?;
    let root = String::from_utf16_lossy(&root);
    Some((
        name[start..=end].to_string(),
        root.trim_end_matches('\0').to_string(),
    ))
}

fn volume_size(root: &str) -> u64 {
    let mut total = 0u64;
    unsafe {
        let _ = GetDiskFreeSpaceExW(&HSTRING::from(root), None, Some(&mut total), None);
    }
    total
}

/// Bytes currently in the Recycle Bins of all drives.
pub fn recycle_bin_size() -> u64 {
    let mut info = SHQUERYRBINFO {
        cbSize: size_of::<SHQUERYRBINFO>() as u32,
        ..Default::default()
    };
    unsafe {
        if SHQueryRecycleBinW(PCWSTR::null(), &mut info).is_err() {
            return 0;
        }
    }
    info.i64Size.max(0) as u64
}

/// Empties every Recycle Bin. Permanent: only after the user confirmed.
pub fn empty_recycle_bin() -> Result<(), String> {
    if recycle_bin_size() == 0 {
        return Ok(());
    }
    unsafe {
        SHEmptyRecycleBinW(
            None,
            PCWSTR::null(),
            SHERB_NOCONFIRMATION | SHERB_NOPROGRESSUI | SHERB_NOSOUND,
        )
    }
    .map_err(|e| e.message())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sink() -> IFileOperationProgressSink {
        VetoPermanentDelete {
            vetoed: Cell::new(false),
        }
        .into()
    }

    #[test]
    fn sink_vetoes_permanent_delete() {
        let err = unsafe { sink().PreDeleteItem(0, None) }.unwrap_err();
        assert_eq!(err.code(), E_ABORT);
    }

    #[test]
    fn sink_allows_recycling() {
        unsafe { sink().PreDeleteItem(TSF_DELETE_RECYCLE_IF_POSSIBLE, None) }.unwrap();
    }
}
