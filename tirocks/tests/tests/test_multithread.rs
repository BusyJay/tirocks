// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::{sync::Arc, thread};

use tirocks::{
    db::DefaultCfOnlyBuilder,
    option::{ReadOptions, WriteOptions},
    OpenOptions,
};

use super::tempdir_with_prefix;

const N: usize = 100_000;

#[test]
fn test_multi_thread() {
    let path = tempdir_with_prefix("_rust_rocksdb_multithreadtest");

    let db = DefaultCfOnlyBuilder::default()
        .set_create_if_missing(true)
        .open(path.path())
        .unwrap();
    let db = Arc::new(db);

    let write_opt = WriteOptions::default();
    let cf = db.default_cf();
    db.put(&write_opt, &cf, b"key", b"value1").unwrap();

    let db1 = db.clone();
    let j1 = thread::spawn(move || {
        let write_opt = WriteOptions::default();
        let cf = db1.default_cf();
        for _ in 1..N {
            db1.put(&write_opt, cf, b"key", b"value1").unwrap();
        }
    });

    let db2 = db.clone();
    let j2 = thread::spawn(move || {
        let write_opt = WriteOptions::default();
        let cf = db2.default_cf();
        for _ in 1..N {
            db2.put(&write_opt, cf, b"key", b"value2").unwrap();
        }
    });

    let db3 = db.clone();
    let j3 = thread::spawn(move || {
        let read_opt = ReadOptions::default();
        let cf = db3.default_cf();
        for _ in 1..N {
            let res = db3.get(&read_opt, cf, b"key");
            assert!(
                res == Ok(Some(b"value1".to_vec())) || res == Ok(Some(b"value2".to_vec())),
                "{:?}",
                res
            );
        }
    });

    j1.join().unwrap();
    j2.join().unwrap();
    j3.join().unwrap();
}
