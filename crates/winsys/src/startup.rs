//! What starts with Windows: Run keys, Startup folders and logon tasks.
//! Only per-user items are switched, the way Task Manager does it.

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use windows::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
};
use windows::Win32::System::TaskScheduler::{
    ITaskFolder, ITaskService, TASK_ENUM_HIDDEN, TASK_TRIGGER_LOGON, TaskScheduler,
};
use windows::Win32::System::Variant::VARIANT;
use windows::core::BSTR;
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WRITE, REG_BINARY};
use winreg::{RegKey, RegValue};

use wincare_core::{StartupItem, StartupSource, approved_is_enabled, approved_value};

const RUN: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run";
const RUN_32: &str = r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Run";
const APPROVED: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved";
/// Seconds between 1601-01-01 (FILETIME) and 1970-01-01 (Unix), in 100 ns.
const FILETIME_UNIX_OFFSET: u64 = 116_444_736_000_000_000;

pub fn list_startup() -> Vec<StartupItem> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let mut items = Vec::new();
    items.extend(run_items(&hkcu, RUN, "Run", StartupSource::HkcuRun));
    items.extend(run_items(&hklm, RUN, "Run", StartupSource::HklmRun));
    items.extend(run_items(&hklm, RUN_32, "Run32", StartupSource::Hklm32Run));
    if let Some(dir) = std::env::var_os("APPDATA").map(PathBuf::from) {
        items.extend(folder_items(
            &hkcu,
            dir.join(r"Microsoft\Windows\Start Menu\Programs\Startup"),
            StartupSource::UserFolder,
        ));
    }
    if let Some(dir) = std::env::var_os("ProgramData").map(PathBuf::from) {
        items.extend(folder_items(
            &hklm,
            dir.join(r"Microsoft\Windows\Start Menu\Programs\StartUp"),
            StartupSource::CommonFolder,
        ));
    }
    items.extend(logon_tasks());
    items
}

/// Switches a per-user item on or off. Machine-wide items and tasks need
/// admin rights, which v1 never asks for.
pub fn set_startup_enabled(item: &StartupItem, enabled: bool) -> Result<(), String> {
    let approved_list = match item.source {
        StartupSource::HkcuRun => "Run",
        StartupSource::UserFolder => "StartupFolder",
        _ => return Err("needs admin".into()),
    };
    let (_, value_name) = item
        .id
        .split_once(':')
        .ok_or_else(|| format!("bad startup id {}", item.id))?;
    let (key, _) = RegKey::predef(HKEY_CURRENT_USER)
        .create_subkey_with_flags(format!(r"{APPROVED}\{approved_list}"), KEY_WRITE)
        .map_err(|e| e.to_string())?;
    let value = RegValue {
        bytes: approved_value(enabled, now_filetime()).to_vec().into(),
        vtype: REG_BINARY,
    };
    key.set_raw_value(value_name, &value)
        .map_err(|e| e.to_string())
}

fn run_items(
    hive: &RegKey,
    path: &str,
    approved_list: &str,
    source: StartupSource,
) -> Vec<StartupItem> {
    let Ok(run) = hive.open_subkey_with_flags(path, KEY_READ) else {
        return Vec::new();
    };
    let approved = hive
        .open_subkey_with_flags(format!(r"{APPROVED}\{approved_list}"), KEY_READ)
        .ok();
    run.enum_values()
        .flatten()
        .map(|(name, value)| {
            let command = value.to_string().trim_matches('"').to_string();
            StartupItem {
                id: format!("{}:{name}", tag(source)).to_lowercase(),
                enabled: is_enabled(approved.as_ref(), &name),
                name,
                command,
                publisher: None,
                source,
                can_toggle: source.can_toggle(),
            }
        })
        .collect()
}

fn folder_items(hive: &RegKey, dir: PathBuf, source: StartupSource) -> Vec<StartupItem> {
    let approved = hive
        .open_subkey_with_flags(format!(r"{APPROVED}\StartupFolder"), KEY_READ)
        .ok();
    let Ok(entries) = fs::read_dir(&dir) else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter_map(|e| {
            let file = e.file_name().to_string_lossy().to_string();
            if file.eq_ignore_ascii_case("desktop.ini") {
                return None;
            }
            let name = file.strip_suffix(".lnk").unwrap_or(&file).to_string();
            Some(StartupItem {
                id: format!("{}:{file}", tag(source)).to_lowercase(),
                enabled: is_enabled(approved.as_ref(), &file),
                name,
                command: e.path().to_string_lossy().to_string(),
                publisher: None,
                source,
                can_toggle: source.can_toggle(),
            })
        })
        .collect()
}

fn is_enabled(approved: Option<&RegKey>, name: &str) -> bool {
    approved
        .and_then(|k| k.get_raw_value(name).ok())
        .is_none_or(|v| approved_is_enabled(&v.bytes))
}

/// Enabled scheduled tasks with a logon trigger, outside Windows' own
/// `\Microsoft\` folder. Read-only: shown so the user knows they exist.
fn logon_tasks() -> Vec<StartupItem> {
    let mut out = Vec::new();
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let Ok(service) =
            CoCreateInstance::<_, ITaskService>(&TaskScheduler, None, CLSCTX_INPROC_SERVER)
        else {
            return out;
        };
        let none = VARIANT::default();
        if service.Connect(&none, &none, &none, &none).is_err() {
            return out;
        }
        if let Ok(root) = service.GetFolder(&BSTR::from("\\")) {
            visit_folder(&root, &mut out);
        }
    }
    out
}

unsafe fn visit_folder(folder: &ITaskFolder, out: &mut Vec<StartupItem>) {
    unsafe {
        let path = folder.Path().map(|p| p.to_string()).unwrap_or_default();
        if path.to_lowercase().starts_with(r"\microsoft") {
            return;
        }
        if let Ok(tasks) = folder.GetTasks(TASK_ENUM_HIDDEN.0) {
            let count = tasks.Count().unwrap_or(0);
            for i in 1..=count {
                let Ok(task) = tasks.get_Item(&VARIANT::from(i)) else {
                    continue;
                };
                if !task.Enabled().is_ok_and(|e| e.as_bool()) {
                    continue;
                }
                let Ok(triggers) = task.Definition().and_then(|d| d.Triggers()) else {
                    continue;
                };
                // These two methods hand their result back through a pointer.
                let mut n = 0i32;
                let _ = triggers.Count(&mut n);
                let at_logon = (1..=n).any(|j| {
                    let Ok(trigger) = triggers.get_Item(j) else {
                        return false;
                    };
                    let mut kind = Default::default();
                    trigger.Type(&mut kind).is_ok() && kind == TASK_TRIGGER_LOGON
                });
                if !at_logon {
                    continue;
                }
                let task_path = task.Path().map(|p| p.to_string()).unwrap_or_default();
                out.push(StartupItem {
                    id: format!("task:{task_path}").to_lowercase(),
                    name: task.Name().map(|n| n.to_string()).unwrap_or_default(),
                    command: task_path,
                    publisher: None,
                    source: StartupSource::Task,
                    enabled: true,
                    can_toggle: false,
                });
            }
        }
        if let Ok(folders) = folder.GetFolders(0) {
            let count = folders.Count().unwrap_or(0);
            for k in 1..=count {
                if let Ok(sub) = folders.get_Item(&VARIANT::from(k)) {
                    visit_folder(&sub, out);
                }
            }
        }
    }
}

fn tag(source: StartupSource) -> &'static str {
    match source {
        StartupSource::HkcuRun => "hkcurun",
        StartupSource::HklmRun => "hklmrun",
        StartupSource::Hklm32Run => "hklm32run",
        StartupSource::UserFolder => "userfolder",
        StartupSource::CommonFolder => "commonfolder",
        StartupSource::Task => "task",
    }
}

fn now_filetime() -> u64 {
    let unix_100ns = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos() as u64 / 100);
    unix_100ns + FILETIME_UNIX_OFFSET
}
