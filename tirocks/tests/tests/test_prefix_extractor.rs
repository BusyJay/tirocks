// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::ffi::CStr;

use tirocks::{
    db::DefaultCfOnlyBuilder,
    option::{FlushOptions, ReadOptions, WriteOptions},
    slice_transform::{SliceTransform, SysSliceTransform},
    table::{
        block::{filter_policy::SysFilterPolicy, BlockBasedTableOptions},
        SysTableFactory,
    },
    OpenOptions,
};

use super::tempdir_with_prefix;

struct FixedPrefixTransform {
    pub prefix_len: usize,
}

impl SliceTransform for FixedPrefixTransform {
    fn name(&self) -> &std::ffi::CStr {
        CStr::from_bytes_with_nul(b"FixedPrefixTransform\0").unwrap()
    }

    fn transform<'a>(&self, key: &'a [u8]) -> &'a [u8] {
        &key[..self.prefix_len]
    }

    fn in_domain(&self, key: &[u8]) -> bool {
        key.len() >= self.prefix_len
    }
}

#[test]
fn test_prefix_extractor_compatibility() {
    let path = tempdir_with_prefix("_rust_rocksdb_prefix_extractor_compatibility");
    let keys = vec![
        b"k1-0", b"k1-1", b"k1-2", b"k1-3", b"k1-4", b"k1-5", b"k1-6", b"k1-7", b"k1-8",
    ];

    // create db with no prefix extractor, and insert data
    let db = DefaultCfOnlyBuilder::default()
        .set_create_if_missing(true)
        .open(path.path())
        .unwrap();
    let write_opt = WriteOptions::default();
    let mut flush_opt = FlushOptions::default();
    flush_opt.set_wait(true);
    let cf = db.default_cf();

    // sst1 with no prefix bloom.
    db.put(&write_opt, cf, b"k1-0", b"a").unwrap();
    db.put(&write_opt, cf, b"k1-1", b"b").unwrap();
    db.put(&write_opt, cf, b"k1-2", b"c").unwrap();
    db.flush(&flush_opt, cf).unwrap(); // flush memtable to sst file.
    drop(db);

    // open db with prefix extractor, and insert data
    let mut bbto = BlockBasedTableOptions::default();
    let filter = SysFilterPolicy::new_bloom_filter_policy(10., false);
    bbto.set_filter_policy(&filter)
        .set_whole_key_filtering(false);
    let mut builder = DefaultCfOnlyBuilder::default();
    let table_factory = SysTableFactory::new_block_based(&bbto);
    let slice_transform = SysSliceTransform::new(FixedPrefixTransform { prefix_len: 2 });
    builder
        .set_create_if_missing(false)
        .cf_options_mut()
        .set_table_factory(&table_factory)
        .set_prefix_extractor(&slice_transform)
        // also create prefix bloom for memtable
        .set_memtable_prefix_bloom_size_ratio(0.1);
    let db = builder.open(path.path()).unwrap();

    let write_opt = WriteOptions::default();
    let mut flush_opt = FlushOptions::default();
    flush_opt.set_wait(true);
    let cf = db.default_cf();

    // sst2 with prefix bloom.
    db.put(&write_opt, cf, b"k1-3", b"a").unwrap();
    db.put(&write_opt, cf, b"k1-4", b"b").unwrap();
    db.put(&write_opt, cf, b"k1-5", b"c").unwrap();
    db.flush(&flush_opt, cf).unwrap(); // flush memtable to sst file.

    // memtable with prefix bloom.
    db.put(&write_opt, cf, b"k1-6", b"a").unwrap();
    db.put(&write_opt, cf, b"k1-7", b"b").unwrap();
    db.put(&write_opt, cf, b"k1-8", b"c").unwrap();

    let mut read_opt = ReadOptions::default();
    let mut iter = db.iter(&mut read_opt, cf);
    iter.seek(b"k1-0");
    let mut key_count = 0;
    while iter.valid() {
        // If sst file has no prefix bloom, don't use prefix seek model.
        assert_eq!(keys[key_count], iter.key());
        key_count = key_count + 1;
        iter.next();
    }
    assert!(key_count == 9);
    iter.check().unwrap();
}
