// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::ffi::CStr;
use std::sync::mpsc::{self, SyncSender, Sender};
use std::sync::*;
use std::thread::{self, JoinHandle};

use tirocks::option::{WriteOptions, ReadOptions, FlushOptions, CompactRangeOptions, BottommostLevelCompaction};
use tirocks::properties::table::user::SequenceNumber;
use tirocks::table::SysTableFactory;
use tirocks::table::block::BlockBasedTableOptions;
use tirocks::table::block::filter_policy::SysFilterPolicy;
use tirocks::{RawIterator, OpenOptions, Iterator, Snapshot, Iterable, CloneFactory};
use tirocks::compaction_filter::{CompactionFilter, Decision, ValueType, SysCompactionFilterFactory};
use tirocks::db::DefaultCfOnlyBuilder;
use tirocks::slice_transform::{SliceTransform, SysSliceTransform};

struct FixedPrefixTransform {
    pub prefix_len: usize,
}

impl SliceTransform for FixedPrefixTransform {
    fn name(&self) -> &CStr {
        unsafe {
            CStr::from_bytes_with_nul_unchecked(b"fixed prefix\0")
        }
    }

    fn transform<'a>(&self, key: &'a [u8]) -> &'a [u8] {
        &key[..self.prefix_len]
    }

    fn in_domain(&self, key: &[u8]) -> bool {
        key.len() >= self.prefix_len
    }
}

struct FixedSuffixTransform {
    pub suffix_len: usize,
}

impl SliceTransform for FixedSuffixTransform {
    fn name(&self) -> &CStr {
        unsafe {
            CStr::from_bytes_with_nul_unchecked(b"fixed suffix\0")
        }
    }

    fn transform<'a>(&self, key: &'a [u8]) -> &'a [u8] {
        &key[..(key.len() - self.suffix_len)]
    }

    fn in_domain(&self, key: &[u8]) -> bool {
        key.len() >= self.suffix_len
    }
}

#[allow(deprecated)]
fn prev_collect(iter: &mut RawIterator) -> Vec<(Vec<u8>, Vec<u8>)> {
    let mut buf = vec![];
    while iter.valid() {
        let k = iter.key().to_vec();
        let v = iter.value().to_vec();
        buf.push((k, v));
        iter.prev();
    }
    buf
}

fn next_collect(iter: &mut RawIterator) -> Vec<(Vec<u8>, Vec<u8>)> {
    let mut buf = vec![];
    while iter.valid() {
        let k = iter.key().to_vec();
        let v = iter.value().to_vec();
        buf.push((k, v));
        let _ = iter.next();
    }
    buf
}

#[test]
pub fn test_iterator() {
    let path = super::tempdir_with_prefix("_rust_rocksdb_iteratortest");

    let k1 = b"k1";
    let k2 = b"k2";
    let k3 = b"k3";
    let k4 = b"k4";
    let v1 = b"v1111";
    let v2 = b"v2222";
    let v3 = b"v3333";
    let v4 = b"v4444";
    let mut builder = DefaultCfOnlyBuilder::default();
    builder.options_mut().db_options_mut().set_create_if_missing(true);
    let db = builder.open(path.path()).unwrap();
    let write_opts = WriteOptions::default();
    let cf = db.default_cf();
    db.put(&write_opts, cf, k1, v1).unwrap();
    db.put(&write_opts, cf, k2, v2).unwrap();
    db.put(&write_opts, cf, k3, v3).unwrap();
    let expected = vec![
        (k1.to_vec(), v1.to_vec()),
        (k2.to_vec(), v2.to_vec()),
        (k3.to_vec(), v3.to_vec()),
    ];

    let mut read_opt = ReadOptions::default();
    let mut iter = db.iter(&mut read_opt, cf);

    iter.seek_to_first();
    assert_eq!((&mut iter).collect::<Vec<_>>(), expected);

    // Test that it's idempotent
    iter.seek_to_first();
    assert_eq!((&mut iter).collect::<Vec<_>>(), expected);

    // Test it in reverse a few times
    iter.seek_to_last();
    let mut tmp_vec = prev_collect(&mut iter);
    tmp_vec.reverse();
    assert_eq!(tmp_vec, expected);

    iter.seek_to_last();
    let mut tmp_vec = prev_collect(&mut iter);
    tmp_vec.reverse();
    assert_eq!(tmp_vec, expected);

    // Try it forward again
    iter.seek_to_first();
    assert_eq!((&mut iter).collect::<Vec<_>>(), expected);

    iter.seek_to_first();
    assert_eq!((&mut iter).collect::<Vec<_>>(), expected);

    let mut read_opt2 = ReadOptions::default();
    let mut old_iterator = db.iter(&mut read_opt2, cf);
    old_iterator.seek_to_first();
    db.put(&write_opts, cf, k4, v4).unwrap();
    let expected2 = vec![
        (k1.to_vec(), v1.to_vec()),
        (k2.to_vec(), v2.to_vec()),
        (k3.to_vec(), v3.to_vec()),
        (k4.to_vec(), v4.to_vec()),
    ];
    assert_eq!(old_iterator.collect::<Vec<_>>(), expected);
    drop(iter);

    let mut iter = db.iter(&mut read_opt, cf);
    iter.seek_to_first();
    assert_eq!((&mut iter).collect::<Vec<_>>(), expected2);

    iter.seek(k2);
    assert!(iter.valid());
    let expected = vec![
        (k2.to_vec(), v2.to_vec()),
        (k3.to_vec(), v3.to_vec()),
        (k4.to_vec(), v4.to_vec()),
    ];
    assert_eq!((&mut iter).collect::<Vec<_>>(), expected);

    iter.seek(k2);
    assert!(iter.valid());
    let expected = vec![(k2.to_vec(), v2.to_vec()), (k1.to_vec(), v1.to_vec())];
    assert_eq!(prev_collect(&mut iter), expected);

    let cases: Vec<(&[u8], bool)> = vec![
    (b"k0", true), 
    (b"k1", true), 
    (b"k11", true), 
    (b"k5", false),
    (b"k0", true), 
    (b"k1", true), 
    (b"k11", true), 
    (b"k5", false)
    ];
    for (key, valid) in cases {
        iter.seek(key);
        assert_eq!(iter.valid(), valid);
    }

    iter.seek(b"k4");
    assert!(iter.valid());
    iter.prev();
    assert!(iter.valid());
    iter.next();
    assert!(iter.valid());
    iter.next();
    assert!(!iter.valid());
}

fn make_checker<D: Iterable + Send + 'static>(mut iter: Iterator<'static, D>) -> (Sender<()>, JoinHandle<()>) {
    let (tx, rx) = mpsc::channel();
    let j = thread::spawn(move || {
        rx.recv().unwrap();
        iter.seek_to_first();
        assert!(iter.valid());
        assert_eq!(iter.key(), b"k1");
        assert_eq!(iter.value(), b"v1");
    });
    (tx, j)
}

#[test]
fn test_send_iterator() {
    let path = super::tempdir_with_prefix("_rust_rocksdb_iteratortest_send");
    
    let mut builder = DefaultCfOnlyBuilder::default();
    builder.options_mut().db_options_mut().set_create_if_missing(true);
    let db = Arc::new(builder.open(path.path()).unwrap());
    let write_opt = WriteOptions::default();
    let cf = db.default_cf();
    db.put(&write_opt, cf, b"k1", b"v1").unwrap();

    let read_opt = ReadOptions::default();
    let iter = Iterator::new(db.clone(), read_opt, cf);

    let (tx, handle) = make_checker(iter);
    drop(db);
    tx.send(()).unwrap();
    handle.join().unwrap();

    let db = Arc::new(builder.open(path.path()).unwrap());
    let cf = db.default_cf();
    db.flush(FlushOptions::default().set_wait(true), cf).unwrap();

    let snap = Snapshot::new(db.clone());
    let opts = ReadOptions::default();
    let iter = Iterator::new(db.clone(), opts, cf);
    db.put(&write_opt, cf, b"k1", b"v2").unwrap();
    db.flush(FlushOptions::default().set_wait(true), cf).unwrap();
    db.compact_range(&CompactRangeOptions::default(), cf, None, None).unwrap();

    let (tx, handle) = make_checker(iter);
    // iterator still holds the sst file, so it should be able to read the old value.
    drop(snap);
    drop(db);
    tx.send(()).unwrap();
    handle.join().unwrap();
}

#[test]
fn test_seek_for_prev() {
    let path = super::tempdir_with_prefix("_rust_rocksdb_seek_for_prev");
    let mut builder = DefaultCfOnlyBuilder::default();
    builder.options_mut().db_options_mut().set_create_if_missing(true);
    
    let db = builder.open(path.path()).unwrap();
    let cf = db.default_cf();
    let write_opts = WriteOptions::default();
    db.put(&write_opts, cf, b"k1-0", b"a").unwrap();
    db.put(&write_opts, cf, b"k1-1", b"b").unwrap();
    db.put(&write_opts, cf, b"k1-3", b"d").unwrap();

    let mut read_opts = ReadOptions::default();
    let mut iter = db.iter(&mut read_opts, cf);
    iter.seek_for_prev(b"k1-2");
    assert!(iter.valid());
    assert_eq!(iter.key(), b"k1-1");
    assert_eq!(iter.value(), b"b");
    drop(iter);

    let mut iter = db.iter(&mut read_opts, cf);
    iter.seek_for_prev(b"k1-3");
    assert!(iter.valid());
    assert_eq!(iter.key(), b"k1-3");
    assert_eq!(iter.value(), b"d");
    drop(iter);

    let mut iter = db.iter(&mut read_opts, cf);
    iter.seek_to_first();
    assert!(iter.valid());
    assert_eq!(iter.key(), b"k1-0");
    assert_eq!(iter.value(), b"a");
    drop(iter);

    let mut iter = db.iter(&mut read_opts, cf);
    iter.seek_to_last();
    assert!(iter.valid());
    assert_eq!(iter.key(), b"k1-3");
    assert_eq!(iter.value(), b"d");
    drop(iter);

    let mut iter = db.iter(&mut read_opts, cf);
    iter.seek_for_prev(b"k0-0");
    assert!(!iter.valid());
    drop(iter);

    let mut iter = db.iter(&mut read_opts, cf);
    iter.seek_for_prev(b"k2-0");
    assert!(iter.valid());
    assert_eq!(iter.key(), b"k1-3");
    assert_eq!(iter.value(), b"d");
}

#[test]
fn read_with_upper_bound() {
    let path = super::tempdir_with_prefix("_rust_rocksdb_read_with_upper_bound_test");
    let mut builder = DefaultCfOnlyBuilder::default();
    builder.options_mut().db_options_mut().set_create_if_missing(true);
    
    let db = builder.open(path.path()).unwrap();
    let write_opts = WriteOptions::default();
    let cf = db.default_cf();
    db.put(&write_opts, cf, b"k1-0", b"a").unwrap();
    db.put(&write_opts, cf, b"k1-1", b"b").unwrap();
    db.put(&write_opts, cf, b"k2-0", b"c").unwrap();

    let mut readopts = ReadOptions::default();
    let upper_bound = b"k2".to_vec();
    readopts.set_iterate_upper_bound(upper_bound);
    assert_eq!(readopts.iterate_upper_bound(), Some(b"k2" as &[u8]));
    let mut iter = db.iter(&mut readopts, cf);
    iter.seek_to_first();
    let vec = next_collect(&mut iter);
    assert_eq!(vec.len(), 2);
}

#[test]
fn test_total_order_seek() {
    let path = super::tempdir_with_prefix("_rust_rocksdb_total_order_seek");
    let mut bbto = BlockBasedTableOptions::default();
    let policy = SysFilterPolicy::new_bloom_filter_policy(10, false);
    bbto.set_filter_policy(&policy).set_whole_key_filtering(false);
    let mut builder = DefaultCfOnlyBuilder::default();
    builder.options_mut().db_options_mut().set_create_if_missing(true);
    let factory = SysTableFactory::new_block_based(&bbto);
    let transform = SysSliceTransform::new(FixedPrefixTransform { prefix_len: 2 });
    builder.options_mut().cf_options_mut().set_table_factory(&factory).set_prefix_extractor(&transform).set_memtable_prefix_bloom_size_ratio(0.1);

    let keys = vec![
        b"k1-1", b"k1-2", b"k1-3", b"k2-1", b"k2-2", b"k2-3", b"k3-1", b"k3-2", b"k3-3",
    ];
    let db = builder.open(path.path()).unwrap();
    let cf = db.default_cf();
    let wopts = WriteOptions::default();

    // sst1
    db.put(&wopts, cf, b"k1-1", b"a").unwrap();
    db.put(&wopts, cf, b"k1-2", b"b").unwrap();
    db.put(&wopts, cf, b"k1-3", b"c").unwrap();
    db.put(&wopts, cf, b"k2-1", b"a").unwrap();
    db.flush(FlushOptions::default().set_wait(true), cf).unwrap();

    // sst2
    db.put(&wopts, cf, b"k2-2", b"b").unwrap();
    db.put(&wopts, cf, b"k2-3", b"c").unwrap();
    db.flush(FlushOptions::default().set_wait(true), cf).unwrap();

    // memtable
    db.put(&wopts, cf, b"k3-1", b"a").unwrap();
    db.put(&wopts, cf, b"k3-2", b"b").unwrap();
    db.put(&wopts, cf, b"k3-3", b"c").unwrap();

    let mut ropts = ReadOptions::default();
    ropts.set_prefix_same_as_start(true);
    let mut iter = db.iter(&mut ropts, cf);
    // only iterate sst files and memtables that contain keys with the same prefix as b"k1"
    // and the keys is iterated as valid when prefixed as b"k1"
    iter.seek(b"k1-0");
    let mut key_count = 0;
    while iter.valid() {
        assert_eq!(keys[key_count], iter.key());
        key_count = key_count + 1;
        iter.next();
    }
    assert!(key_count == 3);
    drop(iter);

    ropts = ReadOptions::default();
    let mut iter = db.iter(&mut ropts, cf);
    // only iterate sst files and memtables that contain keys with the same prefix as b"k1"
    // but it still can next/prev to the keys which is not prefixed as b"k1" with
    // prefix_same_as_start
    iter.seek(b"k1-0");
    let mut key_count = 0;
    while iter.valid() {
        assert_eq!(keys[key_count], iter.key());
        key_count += 1;
        iter.next();
    }
    assert_eq!(key_count, 4);
    drop(iter);

    let mut ropts = ReadOptions::default();
    ropts.set_total_order_seek(true);
    let mut iter = db.iter(&mut ropts, cf);
    iter.seek(b"k1-0");
    let mut key_count = 0;
    while iter.valid() {
        // iterator all sst files and memtables
        assert_eq!(keys[key_count], iter.key());
        key_count = key_count + 1;
        iter.next();
    }
    assert!(key_count == 9);
}

#[test]
fn test_fixed_suffix_seek() {
    let path = super::tempdir_with_prefix("_rust_rocksdb_fixed_suffix_seek");
    let filter = SysFilterPolicy::new_bloom_filter_policy(10, false);
    let mut bbto = BlockBasedTableOptions::default();
    bbto.set_filter_policy(&filter).set_whole_key_filtering(false);
    let mut builder = DefaultCfOnlyBuilder::default();
    builder.options_mut().db_options_mut().set_create_if_missing(true);
    let table_factory = SysTableFactory::new_block_based(&bbto);
    let transform = SysSliceTransform::new(FixedSuffixTransform { suffix_len: 2});
    builder.options_mut().cf_options_mut().set_table_factory(&table_factory).set_prefix_extractor(&transform);

    let write_opts = WriteOptions::default();
    let db = builder.open(path.path()).unwrap();
    let cf = db.default_cf();
    db.put(&write_opts, cf, b"k-eghe-5", b"a").unwrap();
    db.put(&write_opts, cf, b"k-24yfae-6", b"a").unwrap();
    db.put(&write_opts, cf, b"k-h1fwd-7", b"a").unwrap();
    db.flush(FlushOptions::default().set_wait(true), cf).unwrap();

    let mut read_opts = ReadOptions::default();
    let mut iter = db.iter(&mut read_opts, cf);
    iter.seek(b"k-24yfae-8");
    let vec = prev_collect(&mut iter);
    assert!(vec.len() == 2);
    drop(iter);

    let mut iter = db.iter(&mut read_opts, cf);
    iter.seek(b"k-24yfa-9");
    let vec = prev_collect(&mut iter);
    assert!(vec.len() == 0);
}

#[derive(Clone)]
struct TestCompactionFilter(SyncSender<(i32, Vec<u8>, SequenceNumber, ValueType, Vec<u8>)>);

impl CompactionFilter for TestCompactionFilter {
    fn filter(&mut self, level: i32, key: &[u8], seqno: SequenceNumber, value_type: ValueType, existing_value: &[u8]) -> Decision {
        self.0.send((level, key.to_vec(), seqno, value_type, existing_value.to_vec())).unwrap();
        Decision::Keep
    }

    fn name(&mut self) -> &CStr {
        unsafe {
            CStr::from_bytes_with_nul_unchecked(b"TestCopactionFilter\0")
        }
    }
}

#[test]
fn test_iter_sequence_number() {
    let (tx, rx) = mpsc::sync_channel(8);
    let filter = TestCompactionFilter(tx);

    let path = super::tempdir_with_prefix("_rust_rocksdb_sequence_number");
    let mut builder = DefaultCfOnlyBuilder::default();
    builder.options_mut().db_options_mut().set_create_if_missing(true);
    let factory = SysCompactionFilterFactory::new(CloneFactory::new(filter));
    builder.options_mut().cf_options_mut().set_disable_auto_compactions(true).set_num_levels(7).set_compaction_filter_factory(&factory);
    let db = builder.open(path.path()).unwrap();

    let write_opt = WriteOptions::default();
    let mut flush_opt = FlushOptions::default();
    let cf = db.default_cf();
    db.put(&write_opt, cf, b"key1", b"value11").unwrap();
    db.flush(flush_opt.set_wait(false), cf).unwrap();
    db.put(&write_opt, cf, b"key1", b"value22").unwrap();
    db.flush(&flush_opt, cf).unwrap();
    db.put(&write_opt, cf, b"key2", b"value21").unwrap();
    db.flush(&flush_opt, cf).unwrap();
    db.put(&write_opt, cf, b"key2", b"value22").unwrap();

    let mut read_opt = ReadOptions::default();
    let mut iter = db.iter(&mut read_opt, cf);
    iter.seek(b"key1");
    assert!(iter.valid());
    assert_eq!(iter.key(), b"key1");
    assert_eq!(iter.value(), b"value22");
    assert_eq!(iter.sequence_number(), Some(2));

    iter.next();
    assert!(iter.valid());
    assert_eq!(iter.key(), b"key2");
    assert_eq!(iter.value(), b"value22");
    assert_eq!(iter.sequence_number(), Some(4));

    let mut compact_opts = CompactRangeOptions::default();
    compact_opts.set_bottommost_level_compaction(BottommostLevelCompaction::kForce).set_target_level(6);
    let cf = db.default_cf();
    db.compact_range(&compact_opts, cf, Some(b"a"), Some(b"z")).unwrap();

    let (k, key, seqno, vt, v) = rx.recv().unwrap();
    assert_eq!(k, 0);
    assert_eq!(key, b"key1");
    assert_eq!(v, b"value22");
    assert_eq!(vt, ValueType::kValue);
    assert_eq!(seqno, 2);

    let (k, key, seqno, vt, v) = rx.recv().unwrap();
    assert_eq!(k, 0);
    assert_eq!(key, b"key2");
    assert_eq!(v, b"value22");
    assert_eq!(vt, ValueType::kValue);
    assert_eq!(seqno, 4);
}
