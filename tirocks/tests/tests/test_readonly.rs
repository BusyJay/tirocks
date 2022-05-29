// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use tirocks::{
    db::{DefaultCfOnlyBuilder, MultiCfBuilder, DEFAULT_CF_NAME},
    option::{ReadOptions, WriteOptions},
    CfOptions, OpenOptions,
};

use super::tempdir_with_prefix;

#[test]
fn test_open_for_read_only() {
    let temp = tempdir_with_prefix("_rust_rocksdb_test_open_for_read_only");

    let rw = DefaultCfOnlyBuilder::default()
        .set_create_if_missing(true)
        .open(temp.path())
        .unwrap();
    let write_opt = WriteOptions::default();
    let read_opt = ReadOptions::default();
    let rw_cf = rw.default_cf();

    rw.put(&write_opt, rw_cf, b"k1", b"v1").unwrap();
    rw.put(&write_opt, rw_cf, b"k2", b"v2").unwrap();
    rw.put(&write_opt, rw_cf, b"k3", b"v3").unwrap();
    assert_eq!(rw.get(&read_opt, rw_cf, b"k1"), Ok(Some(b"v1".to_vec())));
    assert_eq!(rw.get(&read_opt, rw_cf, b"k2"), Ok(Some(b"v2".to_vec())));
    assert_eq!(rw.get(&read_opt, rw_cf, b"k3"), Ok(Some(b"v3".to_vec())));

    let mut builder = DefaultCfOnlyBuilder::default();
    builder.set_read_only(true, false);
    let r1 = builder.open(temp.path()).unwrap();
    let r1_cf = r1.default_cf();
    assert_eq!(r1.get(&read_opt, r1_cf, b"k1"), Ok(Some(b"v1".to_vec())));
    assert_eq!(r1.get(&read_opt, r1_cf, b"k2"), Ok(Some(b"v2".to_vec())));
    assert_eq!(r1.get(&read_opt, r1_cf, b"k3"), Ok(Some(b"v3".to_vec())));

    let r2 = builder.open(temp.path()).unwrap();
    let r2_cf = r2.default_cf();
    assert_eq!(r2.get(&read_opt, r2_cf, b"k1"), Ok(Some(b"v1".to_vec())));
    assert_eq!(r2.get(&read_opt, r2_cf, b"k2"), Ok(Some(b"v2".to_vec())));
    assert_eq!(r2.get(&read_opt, r2_cf, b"k3"), Ok(Some(b"v3".to_vec())));

    let r3 = builder.open(temp.path()).unwrap();
    let r3_cf = r3.default_cf();
    assert_eq!(r3.get(&read_opt, r3_cf, b"k1"), Ok(Some(b"v1".to_vec())));
    assert_eq!(r3.get(&read_opt, r3_cf, b"k2"), Ok(Some(b"v2".to_vec())));
    assert_eq!(r3.get(&read_opt, r3_cf, b"k3"), Ok(Some(b"v3".to_vec())));

    drop(rw);
    drop(r1);
    drop(r2);
    drop(r3);
}

#[test]
fn test_open_cf_for_read_only() {
    let temp = tempdir_with_prefix("_rust_rocksdb_test_open_cf_for_read_only");

    let mut rw = DefaultCfOnlyBuilder::default()
        .set_create_if_missing(true)
        .open(temp.path())
        .unwrap();
    rw.create_cf("cf1", CfOptions::default()).unwrap();
    rw.create_cf("cf2", CfOptions::default()).unwrap();
    rw.close().unwrap();

    let rw = MultiCfBuilder::default()
        .add_cf(DEFAULT_CF_NAME, CfOptions::default())
        .add_cf("cf1", CfOptions::default())
        .add_cf("cf2", CfOptions::default())
        .open(temp.path())
        .unwrap();
    let cf1 = rw.cf("cf1").unwrap();
    let write_opt = WriteOptions::default();
    let read_opt = ReadOptions::default();
    rw.put(&write_opt, cf1, b"cf1_k1", b"cf1_v1").unwrap();
    rw.put(&write_opt, cf1, b"cf1_k2", b"cf1_v2").unwrap();
    rw.put(&write_opt, cf1, b"cf1_k3", b"cf1_v3").unwrap();
    assert_eq!(
        rw.get(&read_opt, cf1, b"cf1_k1"),
        Ok(Some(b"cf1_v1".to_vec()))
    );
    assert_eq!(
        rw.get(&read_opt, cf1, b"cf1_k2"),
        Ok(Some(b"cf1_v2".to_vec()))
    );
    assert_eq!(
        rw.get(&read_opt, cf1, b"cf1_k3"),
        Ok(Some(b"cf1_v3".to_vec()))
    );
    let cf2 = rw.cf("cf2").unwrap();
    rw.put(&write_opt, cf2, b"cf2_k1", b"cf2_v1").unwrap();
    rw.put(&write_opt, cf2, b"cf2_k2", b"cf2_v2").unwrap();
    rw.put(&write_opt, cf2, b"cf2_k3", b"cf2_v3").unwrap();
    assert_eq!(
        rw.get(&read_opt, cf2, b"cf2_k1"),
        Ok(Some(b"cf2_v1".to_vec()))
    );
    assert_eq!(
        rw.get(&read_opt, cf2, b"cf2_k2"),
        Ok(Some(b"cf2_v2".to_vec()))
    );
    assert_eq!(
        rw.get(&read_opt, cf2, b"cf2_k3"),
        Ok(Some(b"cf2_v3".to_vec()))
    );
    rw.close().unwrap();

    let r1 = MultiCfBuilder::default()
        .add_cf(DEFAULT_CF_NAME, CfOptions::default())
        .add_cf("cf1", CfOptions::default())
        .set_read_only(true, false)
        .open(temp.path())
        .unwrap();
    let cf1 = r1.cf("cf1").unwrap();
    assert_eq!(
        r1.get(&read_opt, cf1, b"cf1_k1"),
        Ok(Some(b"cf1_v1".to_vec()))
    );
    assert_eq!(
        r1.get(&read_opt, cf1, b"cf1_k2"),
        Ok(Some(b"cf1_v2".to_vec()))
    );
    assert_eq!(
        r1.get(&read_opt, cf1, b"cf1_k3"),
        Ok(Some(b"cf1_v3".to_vec()))
    );

    let r2 = MultiCfBuilder::default()
        .add_cf(DEFAULT_CF_NAME, CfOptions::default())
        .add_cf("cf2", CfOptions::default())
        .set_read_only(true, false)
        .open(temp.path())
        .unwrap();
    let cf2 = r2.cf("cf2").unwrap();
    assert_eq!(
        r2.get(&read_opt, cf2, b"cf2_k1"),
        Ok(Some(b"cf2_v1".to_vec()))
    );
    assert_eq!(
        r2.get(&read_opt, cf2, b"cf2_k2"),
        Ok(Some(b"cf2_v2".to_vec()))
    );
    assert_eq!(
        r2.get(&read_opt, cf2, b"cf2_k3"),
        Ok(Some(b"cf2_v3".to_vec()))
    );
}
