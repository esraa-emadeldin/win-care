//! Plan and apply: the only place anything is ever changed.

use std::cell::RefCell;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use wincare_core::{
    Action, ActivityEntry, BinInfo, CategoryId, Env, FileOps, Finding, Guard, Plan, PlanItem,
    SkipReason, append_activity, apply, fresh_ms, make_plan, read_activity,
};

/// Records what would have been recycled or deleted; touches nothing else.
struct FakeOps {
    bin: BinInfo,
    in_use: HashSet<PathBuf>,
    recycled: RefCell<Vec<PathBuf>>,
    deleted: RefCell<Vec<PathBuf>>,
}

impl FakeOps {
    fn new() -> Self {
        FakeOps {
            bin: BinInfo {
                enabled: true,
                max_bytes: u64::MAX,
            },
            in_use: HashSet::new(),
            recycled: RefCell::default(),
            deleted: RefCell::default(),
        }
    }
}

impl FileOps for FakeOps {
    fn bin_info(&self, _: &Path) -> BinInfo {
        self.bin
    }
    fn in_use(&self, path: &Path) -> bool {
        self.in_use.contains(path)
    }
    fn recycle(&self, path: &Path) -> Result<(), String> {
        self.recycled.borrow_mut().push(path.to_path_buf());
        Ok(())
    }
    fn delete(&self, path: &Path) -> Result<(), String> {
        self.deleted.borrow_mut().push(path.to_path_buf());
        Ok(())
    }
}

fn file(env: &Env, name: &str, bytes: usize) -> Finding {
    let path = env.temp.join(name);
    fs::write(&path, vec![b'x'; bytes]).unwrap();
    let meta = fs::metadata(&path).unwrap();
    Finding {
        id: name.to_string(),
        category: CategoryId::UserTemp,
        path,
        bytes: bytes as u64,
        is_dir: false,
        modified_ms: fresh_ms(&meta),
        preselected: true,
        selectable: true,
        note: None,
    }
}

#[test]
fn apply_skips_changed_and_missing() {
    let tmp = tempfile::tempdir().unwrap();
    let env = Env::for_tests(tmp.path());
    let guard = Guard::new(&env);
    let ops = FakeOps::new();
    let findings = vec![file(&env, "a", 5), file(&env, "b", 5), file(&env, "c", 5)];
    let plan = make_plan(findings, Action::Recycle, &ops, &guard);

    // The world moves on between the scan and the click.
    let b = env.temp.join("b");
    fs::write(&b, b"grown since the scan").unwrap();
    fs::File::options()
        .write(true)
        .open(&b)
        .unwrap()
        .set_modified(SystemTime::now() + Duration::from_secs(60))
        .unwrap();
    fs::remove_file(env.temp.join("c")).unwrap();

    let report = apply(&plan, &ops, &guard);

    assert_eq!(report.done, vec![env.temp.join("a")]);
    assert_eq!(report.freed_bytes, 5);
    let reasons: Vec<_> = report.skipped.iter().map(|s| s.reason.clone()).collect();
    assert_eq!(reasons, vec![SkipReason::Changed, SkipReason::Missing]);
    assert_eq!(*ops.recycled.borrow(), vec![env.temp.join("a")]);
}

#[test]
fn apply_never_deletes_unrecyclable() {
    let tmp = tempfile::tempdir().unwrap();
    let env = Env::for_tests(tmp.path());
    let guard = Guard::new(&env);
    let mut ops = FakeOps::new();
    ops.bin.enabled = false;

    let plan = make_plan(vec![file(&env, "a", 5)], Action::Recycle, &ops, &guard);
    assert!(!plan.items[0].recyclable);

    let report = apply(&plan, &ops, &guard);

    assert_eq!(report.skipped[0].reason, SkipReason::NotRecyclable);
    assert!(ops.recycled.borrow().is_empty());
    assert!(ops.deleted.borrow().is_empty());
}

#[test]
fn too_large_for_bin() {
    let tmp = tempfile::tempdir().unwrap();
    let env = Env::for_tests(tmp.path());
    let guard = Guard::new(&env);
    let mut ops = FakeOps::new();
    ops.bin.max_bytes = 10;

    let plan = make_plan(vec![file(&env, "big", 20)], Action::Recycle, &ops, &guard);

    assert!(!plan.items[0].recyclable);
}

#[test]
fn apply_skips_in_use() {
    let tmp = tempfile::tempdir().unwrap();
    let env = Env::for_tests(tmp.path());
    let guard = Guard::new(&env);
    let mut ops = FakeOps::new();
    ops.in_use.insert(env.temp.join("open.log"));

    let plan = make_plan(
        vec![file(&env, "open.log", 5)],
        Action::Recycle,
        &ops,
        &guard,
    );
    let report = apply(&plan, &ops, &guard);

    assert_eq!(report.skipped[0].reason, SkipReason::InUse);
    assert!(ops.recycled.borrow().is_empty());
}

#[test]
fn apply_refuses_protected_even_if_planned() {
    let tmp = tempfile::tempdir().unwrap();
    let env = Env::for_tests(tmp.path());
    let guard = Guard::new(&env);
    let ops = FakeOps::new();
    let doc = env.user_profile.join("Documents").join("thesis.docx");
    fs::write(&doc, b"precious").unwrap();
    let mut finding = file(&env, "x", 1);
    finding.path = doc.clone();
    // A hand-built plan, as if something bypassed make_plan.
    let plan = Plan {
        id: "forged".into(),
        action: Action::Recycle,
        items: vec![PlanItem {
            finding,
            recyclable: true,
        }],
        total_bytes: 1,
        rejected: vec![],
    };

    let report = apply(&plan, &ops, &guard);

    assert_eq!(report.skipped[0].reason, SkipReason::Protected);
    assert!(ops.recycled.borrow().is_empty());
    assert!(doc.exists());
}

#[test]
fn make_plan_drops_unselectable_and_protected() {
    let tmp = tempfile::tempdir().unwrap();
    let env = Env::for_tests(tmp.path());
    let guard = Guard::new(&env);
    let ops = FakeOps::new();
    let mut info_only = file(&env, "info", 1);
    info_only.selectable = false;
    let mut protected = file(&env, "p", 1);
    protected.path = env.windir.join("system.dll");

    let plan = make_plan(
        vec![info_only, protected, file(&env, "ok", 3)],
        Action::Recycle,
        &ops,
        &guard,
    );

    assert_eq!(plan.items.len(), 1);
    assert_eq!(plan.total_bytes, 3);
}

#[test]
fn activity_roundtrip_and_corrupt_line() {
    let tmp = tempfile::tempdir().unwrap();
    let log = tmp.path().join("activity.jsonl");
    let entry = |at_ms| ActivityEntry {
        at_ms,
        action: Action::Recycle,
        done: 1,
        freed_bytes: 10,
        skipped: 0,
        paths: vec![PathBuf::from(r"C:\x")],
    };
    append_activity(&log, &entry(1)).unwrap();
    append_activity(&log, &entry(2)).unwrap();
    let mut text = fs::read_to_string(&log).unwrap();
    text.push_str("{ not json\n");
    fs::write(&log, text).unwrap();
    append_activity(&log, &entry(3)).unwrap();

    let read = read_activity(&log, 10);

    let order: Vec<u64> = read.iter().map(|e| e.at_ms).collect();
    assert_eq!(order, vec![3, 2, 1]);
    assert_eq!(read_activity(&log, 2).len(), 2);
}

#[test]
fn finding_for_path_measures_a_folder() {
    let tmp = tempfile::tempdir().unwrap();
    let env = Env::for_tests(tmp.path());
    let dir = env.local_app_data.join("BigGame").join("shaders");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("a.bin"), vec![0u8; 70]).unwrap();
    fs::write(dir.join("b.bin"), vec![0u8; 30]).unwrap();

    let f = wincare_core::finding_for_path(&dir, CategoryId::SpaceMap).unwrap();

    assert_eq!(f.bytes, 100);
    assert!(f.is_dir && f.selectable && f.preselected);
    assert_eq!(f.category, CategoryId::SpaceMap);
    assert!(wincare_core::finding_for_path(&dir.join("missing"), CategoryId::SpaceMap).is_none());
}

/// Review C1 end to end: even a hand-made plan cannot recycle a folder that
/// contains a protected one.
#[test]
fn apply_refuses_parent_of_protected() {
    let tmp = tempfile::tempdir().unwrap();
    let mut env = Env::for_tests(tmp.path());
    let onedrive = env.user_profile.join("OneDrive");
    fs::create_dir_all(onedrive.join("Documents")).unwrap();
    fs::write(onedrive.join("Documents").join("thesis.docx"), b"x").unwrap();
    env.known_protected.push(onedrive.join("Documents"));
    let guard = Guard::new(&env);
    let ops = FakeOps::new();
    let finding = wincare_core::finding_for_path(&onedrive, CategoryId::SpaceMap).unwrap();

    assert!(
        make_plan(vec![finding.clone()], Action::Recycle, &ops, &guard)
            .items
            .is_empty()
    );
    let forged = Plan {
        id: "forged".into(),
        action: Action::Recycle,
        items: vec![PlanItem {
            finding,
            recyclable: true,
        }],
        total_bytes: 1,
        rejected: vec![],
    };
    let report = apply(&forged, &ops, &guard);
    assert_eq!(report.skipped[0].reason, SkipReason::Protected);
    assert!(ops.recycled.borrow().is_empty());
}

/// Review I1: what the guard refuses is reported with a reason, so Review can
/// show it instead of silently dropping it.
#[test]
fn make_plan_reports_what_it_refused() {
    let tmp = tempfile::tempdir().unwrap();
    let env = Env::for_tests(tmp.path());
    let guard = Guard::new(&env);
    let ops = FakeOps::new();
    let mut system = file(&env, "p", 1);
    system.path = env.windir.join("system.dll");

    let plan = make_plan(
        vec![system, file(&env, "ok", 3)],
        Action::Recycle,
        &ops,
        &guard,
    );

    assert_eq!(plan.items.len(), 1);
    assert_eq!(plan.rejected.len(), 1);
    assert_eq!(plan.rejected[0].path, env.windir.join("system.dll"));
    assert_eq!(plan.rejected[0].reason, SkipReason::Protected);
}
