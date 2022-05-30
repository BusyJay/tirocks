// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::ffi::CStr;

use tirocks::{
    db::DefaultCfOnlyBuilder,
    option::{ReadOptions, WriteOptions},
    slice_transform::{SliceTransform, SysSliceTransform},
    table::{
        block::{filter_policy::SysFilterPolicy, BlockBasedTableOptions},
        SysTableFactory,
    },
    OpenOptions,
};

use super::tempdir_with_prefix;

struct FixedPostfixTransform {
    pub postfix_len: usize,
}

impl SliceTransform for FixedPostfixTransform {
    fn transform<'a>(&self, key: &'a [u8]) -> &'a [u8] {
        let mid = key.len() - self.postfix_len;
        &key[..mid]
    }

    fn in_domain(&self, key: &[u8]) -> bool {
        key.len() >= self.postfix_len
    }

    fn name(&self) -> &std::ffi::CStr {
        CStr::from_bytes_with_nul(b"FixedPostfixTransform\0").unwrap()
    }
}

#[test]
fn test_slice_transform() {
    let path = tempdir_with_prefix("_rust_rocksdb_slice_transform_test");
    let mut bbto = BlockBasedTableOptions::default();
    bbto.set_filter_policy(&SysFilterPolicy::new_bloom_filter_policy(10, false))
        .set_whole_key_filtering(false);
    let mut builder = DefaultCfOnlyBuilder::default();
    builder
        .set_create_if_missing(true)
        .cf_options_mut()
        .set_table_factory(&SysTableFactory::new_block_based(&bbto))
        .set_memtable_prefix_bloom_size_ratio(0.25)
        .set_prefix_extractor(&SysSliceTransform::new(FixedPostfixTransform {
            postfix_len: 2,
        }));
    let db = builder.open(path.path()).unwrap();
    let samples = vec![
        (b"key_01".to_vec(), b"1".to_vec()),
        (b"key_02".to_vec(), b"2".to_vec()),
        (b"key_0303".to_vec(), b"3".to_vec()),
        (b"key_0404".to_vec(), b"4".to_vec()),
    ];

    let write_opt = WriteOptions::default();
    let mut read_opt = ReadOptions::default();
    let cf = db.default_cf();
    for &(ref k, ref v) in &samples {
        db.put(&write_opt, cf, k, v).unwrap();
        assert_eq!(Ok(Some(v.clone())), db.get(&read_opt, cf, k));
    }

    let mut it = db.iter(&mut read_opt, cf);

    let invalid_seeks = vec![
        b"key_".to_vec(),
        b"key_0".to_vec(),
        b"key_030".to_vec(),
        b"key_03000".to_vec(),
    ];

    for key in invalid_seeks {
        it.seek(&key);
        assert!(!it.valid());
        it.check().unwrap();
    }

    let valid_seeks = vec![
        (b"key_00".to_vec(), b"key_01".to_vec()),
        (b"key_03".to_vec(), b"key_0303".to_vec()),
        (b"key_0301".to_vec(), b"key_0303".to_vec()),
    ];

    for (key, expect_key) in valid_seeks {
        it.seek(&key);
        assert!(it.valid());
        assert_eq!(it.key(), &*expect_key);
    }

    // TODO: support total_order mode and add test later.
}
