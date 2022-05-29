// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use tirocks::{
    db::{DefaultCfOnlyBuilder, RawColumnFamilyHandle},
    metadata::CfMetaData,
    option::{CompactionOptions, CompressionType, FlushOptions, WriteOptions},
    Db, OpenOptions,
};

use super::tempdir_with_prefix;

#[test]
fn test_metadata() {
    let path = tempdir_with_prefix("_rust_rocksdb_test_metadata");
    let mut builder = DefaultCfOnlyBuilder::default();
    builder
        .set_create_if_missing(true)
        .cf_options_mut()
        .set_disable_auto_compactions(true);
    let db = builder.open(path.path()).unwrap();
    let cf = db.default_cf();

    let write_opt = WriteOptions::default();
    let mut flush_opt = FlushOptions::default();
    flush_opt.set_wait(true);
    let num_files = 5;
    for i in 0..num_files {
        db.put(&write_opt, cf, &[i], &[i]).unwrap();
        db.flush(&flush_opt, cf).unwrap();
    }

    let live_files = db.live_files_metadata();
    let files_count = live_files.len();
    assert_eq!(files_count as u8, num_files);
    for i in 0..files_count {
        let f = live_files.get(i);
        assert!(f.name().unwrap().len() > 0);
        assert_eq!(f.smallest_key(), [num_files - 1 - i as u8]);
        assert_eq!(f.largest_key(), [num_files - 1 - i as u8]);
    }

    let mut cf_meta = CfMetaData::default();
    db.cf_metadata(cf, &mut cf_meta);
    let cf_levels = cf_meta.levels();
    assert_eq!(cf_levels.len(), 7);
    for (i, cf_level) in cf_levels.enumerate() {
        let files = cf_level.files();
        if i != 0 {
            assert_eq!(files.len(), 0);
            continue;
        }
        assert_eq!(files.len(), num_files as usize);
        for f in files {
            assert!(f.size() > 0);
            assert!(f.name().unwrap().len() > 0);
            assert!(f.smallest_key().len() > 0);
            assert!(f.largest_key().len() > 0);
        }
    }
}

fn get_files_cf(db: &Db, cf: &RawColumnFamilyHandle, max_level: usize) -> Vec<String> {
    let mut files = Vec::new();
    let mut cf_meta = CfMetaData::default();
    db.cf_metadata(cf, &mut cf_meta);
    for (i, level) in cf_meta.levels().enumerate() {
        if i > max_level {
            break;
        }
        for f in level.files() {
            files.push(f.name().unwrap().to_string());
        }
    }
    files
}

#[test]
fn test_compact_files() {
    let path = tempdir_with_prefix("_rust_rocksdb_test_metadata");
    let mut builder = DefaultCfOnlyBuilder::default();
    builder
        .set_create_if_missing(true)
        .cf_options_mut()
        .set_disable_auto_compactions(true);
    let db = builder.open(path.path()).unwrap();
    let cf = db.default_cf();

    let cf_opts = db.cf_options(cf);
    let output_file_size = cf_opts.cf_options().target_file_size_base();

    let mut opts = CompactionOptions::default();
    opts.set_compression(CompressionType::kZSTD)
        .set_output_file_size_limit(output_file_size);

    let write_opt = WriteOptions::default();
    let mut flush_opt = FlushOptions::default();
    flush_opt.set_wait(true);
    let num_files = 5;
    for i in 0..num_files {
        let b = &[i as u8];
        db.put(&write_opt, cf, b, b).unwrap();
        db.flush(&flush_opt, cf).unwrap();
    }
    let input_files = get_files_cf(&db, cf, 0);
    assert_eq!(input_files.len(), num_files);
    db.compact_files(&opts, cf, &input_files, 0).unwrap();
    assert_eq!(get_files_cf(&db, cf, 0).len(), 1);
}
