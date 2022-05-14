// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use tirocks::{
    db::DefaultCfOnlyBuilder,
    metadata::{CfMetaData, SizeApproximationOptions},
    option::{
        BottommostLevelCompaction, CompactRangeOptions, FlushOptions, ReadOptions, WriteOptions,
    },
    properties::PropNumFilesAtLevel,
    OpenOptions, Options,
};

use super::tempdir_with_prefix;

#[test]
fn test_compact_range() {
    let path = tempdir_with_prefix("_rust_rocksdb_test_compact_range");
    let mut opts = Options::default();
    opts.db_options_mut().set_create_if_missing(true);
    let db = DefaultCfOnlyBuilder::default()
        .set_options(opts)
        .open(path.path())
        .unwrap();
    let samples = vec![
        (b"k1".to_vec(), b"value--------1".to_vec()),
        (b"k2".to_vec(), b"value--------2".to_vec()),
        (b"k3".to_vec(), b"value--------3".to_vec()),
        (b"k4".to_vec(), b"value--------4".to_vec()),
        (b"k5".to_vec(), b"value--------5".to_vec()),
    ];
    let default_cf = db.default_cf();
    let write_opt = WriteOptions::default();
    let read_opt = ReadOptions::default();
    for (k, v) in &samples {
        db.put(&write_opt, default_cf, k, v).unwrap();
        assert_eq!(
            v.as_slice(),
            &*db.get(&read_opt, default_cf, k).unwrap().unwrap()
        );
    }

    // flush memtable to sst file
    db.flush(FlushOptions::default().set_wait(true), default_cf)
        .unwrap();

    let size_opt = SizeApproximationOptions::default();
    let old_size = db
        .approximate_sizes(&size_opt, default_cf, &[(b"k0", b"k6")])
        .unwrap()[0];

    // delete all and compact whole range
    for (ref k, _) in &samples {
        db.delete(&write_opt, default_cf, k).unwrap()
    }
    db.compact_range(&CompactRangeOptions::default(), default_cf, None, None)
        .unwrap();
    let new_size = db
        .approximate_sizes(&size_opt, default_cf, &[(b"k0", b"k6")])
        .unwrap()[0];
    assert!(old_size > new_size);
}

#[test]
fn test_compact_range_change_level() {
    let path = tempdir_with_prefix("_rust_rocksdb_test_compact_range_change_level");
    let mut opts = Options::default();
    opts.db_options_mut().set_create_if_missing(true);
    opts.cf_options_mut()
        .set_level0_file_num_compaction_trigger(10);
    let db = DefaultCfOnlyBuilder::default()
        .set_options(opts)
        .open(path.path())
        .unwrap();
    let samples = vec![
        (b"k1".to_vec(), b"value--------1".to_vec()),
        (b"k2".to_vec(), b"value--------2".to_vec()),
        (b"k3".to_vec(), b"value--------3".to_vec()),
        (b"k4".to_vec(), b"value--------4".to_vec()),
        (b"k5".to_vec(), b"value--------5".to_vec()),
    ];
    let default_cf = db.default_cf();
    let write_opt = WriteOptions::default();
    let mut flush_opt = FlushOptions::default();
    flush_opt.set_wait(true);
    for (k, v) in &samples {
        db.put(&write_opt, default_cf, k, v).unwrap();
        db.flush(&flush_opt, default_cf).unwrap();
    }

    let compact_level = 1;
    let mut compact_opts = CompactRangeOptions::default();
    compact_opts
        .set_change_level(true)
        .set_target_level(compact_level);
    db.compact_range(&compact_opts, default_cf, None, None)
        .unwrap();
    let prop = PropNumFilesAtLevel::new(compact_level as usize);
    assert_eq!(
        db.property(default_cf, &prop)
            .unwrap()
            .unwrap()
            .parse::<u64>()
            .unwrap(),
        samples.len() as u64
    );
}

#[test]
fn test_compact_range_bottommost_level_compaction() {
    let path = tempdir_with_prefix("test_compact_range_bottommost_level_compaction");
    let mut opts = Options::default();
    opts.db_options_mut().set_create_if_missing(true);

    let db = DefaultCfOnlyBuilder::default()
        .set_options(opts)
        .open(path.path())
        .unwrap();
    let default_cf = db.default_cf();
    let write_opt = WriteOptions::default();
    db.put(&write_opt, default_cf, &[0], &[0]).unwrap();
    let mut flush_opt = FlushOptions::default();
    flush_opt.set_wait(true);
    db.flush(&flush_opt, default_cf).unwrap();

    // Compact to bottommost level
    let cf_opts = db.cf_options(default_cf);
    let bottommost_level = (cf_opts.cf_options().num_levels() - 1) as i32;
    let mut compact_opts = CompactRangeOptions::default();
    compact_opts
        .set_change_level(true)
        .set_target_level(bottommost_level);
    db.compact_range(&compact_opts, default_cf, None, None)
        .unwrap();

    let mut metadata = CfMetaData::default();
    db.cf_metadata(default_cf, &mut metadata);
    let mut bottommost_files = metadata.levels().last().unwrap().files();
    assert_eq!(bottommost_files.len(), 1);
    let bottommost_filename = bottommost_files.next().unwrap().name().unwrap().to_string();
    drop(bottommost_files);

    // Skip bottommost level compaction
    compact_opts.set_bottommost_level_compaction(BottommostLevelCompaction::kSkip);
    db.compact_range(&compact_opts, default_cf, None, None)
        .unwrap();
    db.cf_metadata(default_cf, &mut metadata);
    let mut bottommost_files = metadata.levels().last().unwrap().files();
    assert_eq!(bottommost_files.len(), 1);
    assert_eq!(
        bottommost_filename,
        bottommost_files.next().unwrap().name().unwrap()
    );
    drop(bottommost_files);

    // Force bottommost level compaction
    compact_opts.set_bottommost_level_compaction(BottommostLevelCompaction::kForce);
    db.compact_range(&compact_opts, default_cf, None, None)
        .unwrap();
    db.cf_metadata(default_cf, &mut metadata);
    let mut bottommost_files = metadata.levels().last().unwrap().files();
    assert_eq!(bottommost_files.len(), 1);
    assert_ne!(
        bottommost_filename,
        bottommost_files.next().unwrap().name().unwrap()
    );
}
