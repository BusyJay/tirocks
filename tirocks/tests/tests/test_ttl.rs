// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use tirocks::{
    db::{DefaultCfOnlyBuilder, MultiCfBuilder, DEFAULT_CF_NAME},
    option::{ReadOptions, WriteOptions},
    CfOptions, OpenOptions,
};

use super::tempdir_with_prefix;

#[test]
pub fn test_ttl() {
    let path = tempdir_with_prefix("_rust_rocksdb_ttl_test");

    // should be able to open db with ttl
    let mut db = DefaultCfOnlyBuilder::default()
        .set_create_if_missing(true)
        .set_ttl(10)
        .open(path.path())
        .unwrap();
    db.create_cf("cf1", CfOptions::default()).unwrap();
    let mut c: Vec<_> = db.cfs().map(|cf| cf.name().unwrap().to_string()).collect();
    assert_eq!(c, vec!["default", "cf1"]);

    db.create_cf("cf2", CfOptions::default()).unwrap();
    c = db.cfs().map(|cf| cf.name().unwrap().to_string()).collect();
    assert_eq!(c, vec!["default", "cf1", "cf2"]);
    drop(db);

    // should be able to write, read over a cf with the length of ttls equals to that of cfs
    let db = MultiCfBuilder::default()
        .add_cf_ttl("cf1", CfOptions::default(), 10)
        .add_cf_ttl("cf2", CfOptions::default(), 10)
        .add_cf_ttl(DEFAULT_CF_NAME, CfOptions::default(), 10)
        .open(path.path())
        .unwrap();

    let cf1 = db.cf("cf1").unwrap();
    let write_opts = WriteOptions::default();
    let read_opts = ReadOptions::default();
    db.put(&write_opts, cf1, b"k1", b"v1").unwrap();
    assert_eq!(db.get(&read_opts, cf1, b"k1"), Ok(Some(b"v1".to_vec())));
    db.put(&write_opts, cf1, b"k1", b"a").unwrap();
    drop(db);

    // should be able to write, read over a cf with the length of ttls equals to that of cfs.
    // default cf could be with ttl 0 if it is not in cfds
    let db = MultiCfBuilder::default()
        .add_cf_ttl("cf1", CfOptions::default(), 10)
        .add_cf_ttl("cf2", CfOptions::default(), 10)
        .add_cf_ttl(DEFAULT_CF_NAME, CfOptions::default(), 0)
        .open(path.path())
        .unwrap();

    let cf1 = db.cf("cf1").unwrap();
    let write_opts = WriteOptions::default();
    let read_opts = ReadOptions::default();
    db.put(&write_opts, cf1, b"k1", b"v1").unwrap();
    assert_eq!(db.get(&read_opts, cf1, b"k1"), Ok(Some(b"v1".to_vec())));
    db.put(&write_opts, cf1, b"k1", b"a").unwrap();
    drop(db);
}
