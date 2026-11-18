//! Space map tree: sizes, order, folding tiny items, and no junctions.

use std::fs;
use std::path::Path;

use wincare_core::{CancelToken, SpaceNode, scan_space};

fn file(path: &Path, bytes: usize) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, vec![b'x'; bytes]).unwrap();
}

fn scan(root: &Path) -> SpaceNode {
    scan_space(root, 3, &CancelToken::new(), &mut |_| {})
}

fn names(node: &SpaceNode) -> Vec<&str> {
    node.children.iter().map(|c| c.name.as_str()).collect()
}

#[test]
fn space_tree_sizes_and_order() {
    let tmp = tempfile::tempdir().unwrap();
    file(&tmp.path().join("a").join("f"), 300);
    file(&tmp.path().join("b").join("f"), 100);
    file(&tmp.path().join("c").join("deep").join("f"), 200);

    let root = scan(tmp.path());

    assert_eq!(root.bytes, 600);
    assert_eq!(names(&root), vec!["a", "c", "b"]);
    assert_eq!(root.children[1].children[0].name, "deep");
}

#[test]
fn space_folds_tiny_children() {
    let tmp = tempfile::tempdir().unwrap();
    file(&tmp.path().join("big").join("f"), 10 * 1024);
    for i in 0..50 {
        file(&tmp.path().join(format!("tiny{i}.txt")), 1);
    }

    let root = scan(tmp.path());

    assert_eq!(names(&root), vec!["big", "Other"]);
    assert_eq!(root.children[1].bytes, 50);
    assert_eq!(root.bytes, 10 * 1024 + 50);
}

#[test]
fn space_does_not_follow_junctions() {
    let tmp = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    file(&outside.path().join("huge"), 1024 * 1024);
    file(&tmp.path().join("real").join("f"), 10);
    junction::create(outside.path(), tmp.path().join("link")).unwrap();

    let root = scan(tmp.path());

    assert_eq!(root.bytes, 10);
    assert_eq!(names(&root), vec!["real"]);
}

#[test]
fn space_reports_each_top_level_child() {
    let tmp = tempfile::tempdir().unwrap();
    file(&tmp.path().join("a").join("f"), 5);
    file(&tmp.path().join("b").join("f"), 5);
    let mut partials = 0;

    scan_space(tmp.path(), 2, &CancelToken::new(), &mut |_| partials += 1);

    assert_eq!(partials, 2);
}
