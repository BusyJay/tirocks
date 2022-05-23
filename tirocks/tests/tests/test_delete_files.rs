// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::{ops, path::Path};

use rand::RngCore;
use tirocks::{
    db::DefaultCfOnlyBuilder,
    option::{CompactRangeOptions, FlushOptions, ReadOptions, WriteOptions},
    Db, Iterator, OpenOptions, Snapshot,
};

use super::tempdir_with_prefix;

fn initial_data(path: &Path) -> Db {
    let mut builder = DefaultCfOnlyBuilder::default();
    builder
        .set_create_if_missing(true)
        // We will control the compaction manually.
        .cf_options_mut()
        .set_disable_auto_compactions(true);
    let db = builder.open(path).unwrap();

    generate_file_bottom_level(&db, 0..3);
    generate_file_bottom_level(&db, 3..6);
    generate_file_bottom_level(&db, 6..9);

    db
}

/// Generates a file with `range` and put it to the bottommost level.
fn generate_file_bottom_level(db: &Db, range: ops::Range<u32>) {
    let cf = db.default_cf();
    let write_opt = WriteOptions::default();
    for i in range {
        let k = format!("key{}", i);
        let v = format!("value{}", i);
        db.put(&write_opt, cf, k.as_bytes(), v.as_bytes()).unwrap();
    }
    db.flush(FlushOptions::default().set_wait(true), cf)
        .unwrap();

    let opts = db.cf_options(cf);
    let mut compact_opts = CompactRangeOptions::default();
    compact_opts
        .set_change_level(true)
        .set_target_level(opts.cf_options().num_levels() - 1)
        .set_bottommost_level_compaction(tirocks_sys::rocksdb_BottommostLevelCompaction::kSkip);
    db.compact_range(&compact_opts, cf, None, None).unwrap();
}

#[test]
fn test_delete_files_in_range_with_iter() {
    let path = tempdir_with_prefix("_rust_rocksdb_test_delete_files_in_range_with_iter");
    let db = initial_data(path.path());

    // construct iterator before DeleteFilesInRange
    let mut read_opt = ReadOptions::default();
    let cf = db.default_cf();
    let mut iter = db.iter(&mut read_opt, cf);

    // delete sst2
    db.delete_files_in_range(cf, Some(b"key2"), Some(b"key7"), false)
        .unwrap();

    let mut count = 0;
    iter.seek_to_first();
    while iter.valid() {
        iter.next();
        count += 1;
    }
    iter.check().unwrap();

    // iterator will pin all sst files.
    assert_eq!(count, 9);
}

#[test]
fn test_delete_files_in_range_with_snap() {
    let path = tempdir_with_prefix("_rust_rocksdb_test_delete_files_in_range_with_snap");
    let db = initial_data(path.path());

    // construct snapshot before DeleteFilesInRange
    let snap = Snapshot::new(&db);

    // delete sst2
    let cf = db.default_cf();
    db.delete_files_in_range(cf, Some(b"key2"), Some(b"key7"), false)
        .unwrap();

    let read_opt = ReadOptions::default();
    let mut iter = Iterator::new(snap, read_opt, cf);
    iter.seek_to_first();

    let mut count = 0;
    while iter.valid() {
        iter.next();
        count = count + 1;
    }
    iter.check().unwrap();

    // sst2 has been dropped.
    assert_eq!(count, 6);
}

#[test]
fn test_delete_files_in_range_with_delete_range() {
    // Regression test for https://github.com/facebook/rocksdb/issues/2833.
    let path = tempdir_with_prefix("_rocksdb_test_delete_files_in_range_with_delete_range");

    let sst_size = 1 << 10;
    let value_size = 8 << 10;
    let mut builder = DefaultCfOnlyBuilder::default();
    builder
        .set_create_if_missing(true)
        .cf_options_mut()
        .set_target_file_size_base(sst_size)
        .set_level0_file_num_compaction_trigger(10);
    let db = builder.open(path.path()).unwrap();
    let cf = db.default_cf();

    // Flush 5 files in level 0.
    // File i will contain keys i and i+1.
    let write_opt = WriteOptions::default();
    for i in 0..5 {
        let k1 = format!("{}", i);
        let k2 = format!("{}", i + 1);
        let mut v = vec![0; value_size];
        rand::thread_rng().fill_bytes(&mut v);
        db.put(&write_opt, cf, k1.as_bytes(), v.as_slice()).unwrap();
        db.put(&write_opt, cf, k2.as_bytes(), v.as_slice()).unwrap();
        db.flush(FlushOptions::default().set_wait(true), cf)
            .unwrap();
    }

    // Hold a snapshot to prevent the following delete range from dropping keys above.
    let snapshot = Snapshot::new(&db);
    db.delete_range(&write_opt, cf, b"0", b"6").unwrap();
    db.flush(FlushOptions::default().set_wait(true), cf)
        .unwrap();
    // After this, we will have 3 files in level 1.
    // File i will contain keys i and i+1, and the delete range [0, 6).
    let compact_opt = CompactRangeOptions::default();
    db.compact_range(&compact_opt, cf, None, None).unwrap();
    drop(snapshot);

    // Before the fix, the file in the middle with keys 2 and 3 will be deleted,
    // which can be a problem when we compact later. After the fix, no file will
    // be deleted since they have an overlapped delete range [0, 6).
    db.delete_files_in_range(cf, Some(b"1"), Some(b"4"), false)
        .unwrap();

    // Flush a file with keys 4 and 5 to level 0.
    for i in 4..5 {
        let k1 = format!("{}", i);
        let k2 = format!("{}", i + 1);
        let mut v = vec![0; value_size];
        rand::thread_rng().fill_bytes(&mut v);
        db.put(&write_opt, cf, k1.as_bytes(), v.as_slice()).unwrap();
        db.put(&write_opt, cf, k2.as_bytes(), v.as_slice()).unwrap();
        db.flush(FlushOptions::default().set_wait(true), cf)
            .unwrap();
    }

    // After this, the delete range [0, 6) will drop all entries before it, so
    // we should have only keys 4 and 5.
    db.compact_range(&compact_opt, cf, None, None).unwrap();

    let mut read_opt = ReadOptions::default();
    let mut it = db.iter(&mut read_opt, cf);
    it.seek_to_first();
    assert!(it.valid());
    assert_eq!(it.key(), b"4");
    it.next();
    assert!(it.valid());
    assert_eq!(it.key(), b"5");
    it.next();
    assert!(!it.valid());
    it.check().unwrap();
}

#[test]
fn test_delete_files_in_ranges() {
    let path = tempdir_with_prefix("_rust_rocksdb_test_delete_files_in_multi_ranges");
    let db = initial_data(path.path());

    // Delete files in multiple overlapped ranges.
    // File ["key0", "key2"], ["key3", "key5"] should have been deleted,
    // but file ["key6", "key8"] should not be deleted because "key8" is exclusive.
    let mut ranges: Vec<(Option<&[u8]>, Option<&[u8]>)> = Vec::new();
    ranges.push((Some(b"key0"), Some(b"key4")));
    ranges.push((Some(b"key2"), Some(b"key6")));
    ranges.push((Some(b"key4"), Some(b"key8")));

    let cf = db.default_cf();
    db.delete_files_in_ranges(cf, &ranges, false).unwrap();

    // Check that ["key0", "key5"] have been deleted, but ["key6", "key8"] still exist.
    let mut read_opt = ReadOptions::default();
    let mut iter = db.iter(&mut read_opt, cf);
    iter.seek_to_first();
    for i in 6..9 {
        assert!(iter.valid());
        let k = format!("key{}", i);
        assert_eq!(iter.key(), k.as_bytes());
        iter.next();
    }
    assert!(!iter.valid());
    iter.check().unwrap();
    drop(iter);

    // Delete the last file.
    let ranges: Vec<(Option<&[u8]>, Option<&[u8]>)> = vec![(Some(b"key6"), Some(b"key8"))];
    db.delete_files_in_ranges(cf, &ranges, true).unwrap();
    let mut iter = db.iter(&mut read_opt, cf);
    iter.seek_to_first();
    assert!(!iter.valid());
    iter.check().unwrap();
}
