//! The StartupApproved values Task Manager reads and writes.

use wincare_core::{approved_is_enabled, approved_value};

#[test]
fn approved_decoding() {
    let mut v = [0u8; 12];
    v[0] = 2;
    assert!(approved_is_enabled(&v));
    v[0] = 3;
    assert!(!approved_is_enabled(&v));
    v[0] = 6;
    assert!(approved_is_enabled(&v));
    assert!(approved_is_enabled(&[]), "no value means enabled");
}

#[test]
fn approved_encoding_roundtrip() {
    let off = approved_value(false, 123);
    assert!(!approved_is_enabled(&off));
    assert_eq!(&off[4..12], &123u64.to_le_bytes());

    let on = approved_value(true, 123);
    assert!(approved_is_enabled(&on));
    assert_eq!(on, [2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
}
