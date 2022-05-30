// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::{collections::HashMap, ffi::CStr, ops, time::Duration};

use rand::Rng;
use tirocks::{
    cache::LruCacheBuilder,
    db::{MultiCfTitanBuilder, RawColumnFamilyHandle, DEFAULT_CF_NAME},
    option::{
        BottommostLevelCompaction, CompactRangeOptions, CompressionType, FlushOptions, ReadOptions,
        TitanCfOptions, WriteOptions,
    },
    properties::{
        table::{
            builtin::OwnedTablePropertiesCollection,
            user::{
                Context, EntryType, SysTablePropertiesCollectorFactory, TablePropertiesCollector,
                TablePropertiesCollectorFactory, UserCollectedProperties,
            },
        },
        PropTitanNumBlobFilesAtLevel, PropTitanNumDiscardableRatioLe0File,
    },
    statistics::{TitanHistograms, TitanTickers},
    titan::TitanBlobIndex,
    CfOptions, Db, OpenOptions, Result, Statistics,
};

use super::tempdir_with_prefix;

fn encode_u32(x: u32) -> Vec<u8> {
    x.to_le_bytes().to_vec()
}

fn decode_u32(x: &[u8]) -> u32 {
    let mut dst = [0u8; 4];
    dst.copy_from_slice(&x[..4]);
    u32::from_le_bytes(dst)
}

#[derive(Default)]
struct TitanCollector {
    num_blobs: u32,
    num_entries: u32,
}

impl TitanCollector {
    fn add(&mut self, other: &TitanCollector) {
        self.num_blobs += other.num_blobs;
        self.num_entries += other.num_entries;
    }

    fn encode(&self, properties: &mut UserCollectedProperties) {
        properties.add(&[0], &encode_u32(self.num_blobs));
        properties.add(&[1], &encode_u32(self.num_entries));
    }

    fn decode(props: &UserCollectedProperties) -> TitanCollector {
        let mut c = TitanCollector::default();
        c.num_blobs = decode_u32(&props[&[0]]);
        c.num_entries = decode_u32(&props[&[1]]);
        c
    }
}

impl TablePropertiesCollector for TitanCollector {
    fn add(&mut self, _: &[u8], value: &[u8], entry_type: EntryType, _: u64, _: u64) -> Result<()> {
        self.num_entries += 1;
        if let EntryType::kEntryBlobIndex = entry_type {
            self.num_blobs += 1;
            let index = TitanBlobIndex::decode(value).unwrap();
            assert!(index.file_number > 0);
            assert!(index.blob_size > 0);
        }
        Ok(())
    }

    fn finish(&mut self, properties: &mut UserCollectedProperties) -> Result<()> {
        self.encode(properties);
        Ok(())
    }

    fn name(&self) -> &CStr {
        CStr::from_bytes_with_nul(b"titancollector\0").unwrap()
    }
}

#[derive(Default)]
struct TitanCollectorFactory {}

impl TablePropertiesCollectorFactory for TitanCollectorFactory {
    type Collector = TitanCollector;
    fn create_table_properties_collector(&self, _: Context) -> TitanCollector {
        TitanCollector::default()
    }

    fn name(&self) -> &CStr {
        CStr::from_bytes_with_nul(b"TitanCollectorFactory\0").unwrap()
    }
}

fn check_table_properties(db: &Db, num_blobs: u32, num_entries: u32) {
    let cf = db.default_cf();
    let mut collection = OwnedTablePropertiesCollection::default();
    db.properties_of_all_tables(cf, &mut collection).unwrap();
    let mut res = TitanCollector::default();
    let props: HashMap<_, _> = collection.into_iter().collect();
    for (_, v) in &props {
        res.add(&TitanCollector::decode(v.user_collected_properties()));
    }
    assert_eq!(res.num_blobs, num_blobs);
    assert_eq!(res.num_entries, num_entries);
}

#[test]
fn test_titandb() {
    let max_value_size = 10;

    let path = tempdir_with_prefix("test_titandb");
    let tdb_path = path.path().join("titandb");
    let f = SysTablePropertiesCollectorFactory::new(TitanCollectorFactory::default());
    let mut builder = MultiCfTitanBuilder::default();
    builder
        .db_options_mut()
        .set_directory(tdb_path)
        .set_disable_background_gc(true)
        .set_purge_obsolete_files_period(Duration::from_secs(10));
    let mut cf_opts = TitanCfOptions::default();
    cf_opts
        .set_min_blob_size(max_value_size / 2 + 1)
        .set_blob_file_compression(CompressionType::kZSTD)
        .set_level_merge(false)
        .set_range_merge(false)
        .set_max_sorted_runs(20)
        .add_table_properties_collector_factory(&f)
        .compression_opts_mut()
        .set_window_bits(-14)
        .set_level(0)
        .set_max_dict_bytes(0)
        .set_strategy(0)
        .set_zstd_max_train_bytes(0);

    let mut db = builder
        .set_create_if_missing(true)
        .add_cf(DEFAULT_CF_NAME, cf_opts)
        .open(path.path())
        .unwrap();
    let write_opts = WriteOptions::default();
    let mut flush_opts = FlushOptions::default();
    flush_opts.set_wait(true);
    let cf = db.default_cf();

    let n = 10;
    for i in 0..n {
        for size in 0..max_value_size {
            let k = (i * n + size) as u8;
            let v = vec![k; (size + 1) as usize];
            db.put(&write_opts, cf, &[k], &v).unwrap();
        }
        db.flush(&flush_opts, cf).unwrap();
    }

    let mut cf_opts = CfOptions::default();
    cf_opts.set_num_levels(4);
    db.create_cf("cf1", cf_opts).unwrap();
    let cf = db.default_cf();
    let cf1 = db.cf("cf1").unwrap();
    assert_eq!(db.cf_options(cf1).cf_options().num_levels(), 4);

    let mut read_opts = ReadOptions::default();
    let mut iter = db.iter(&mut read_opts, cf);
    iter.seek_to_first();
    let get_opts = ReadOptions::default();
    for i in 0..n {
        for j in 0..n {
            let k = (i * n + j) as u8;
            let v = vec![k; (j + 1) as usize];
            assert_eq!(db.get(&get_opts, cf, &[k]), Ok(Some(v.clone())));
            assert!(iter.valid());
            assert_eq!(iter.key(), &[k]);
            assert_eq!(iter.value(), v.as_slice());
            iter.next();
        }
    }
    iter.check().unwrap();
    drop(iter);

    read_opts.set_key_only(true);
    let mut iter = db.iter(&mut read_opts, cf);
    iter.seek_to_first();
    for i in 0..n {
        for j in 0..n {
            let k = (i * n + j) as u8;
            let v = vec![k; (j + 1) as usize];
            assert_eq!(db.get(&get_opts, cf, &[k]), Ok(Some(v.clone())));
            assert!(iter.valid());
            assert_eq!(iter.key(), &[k]);
            iter.next();
        }
    }
    iter.check().unwrap();

    let num_entries = n as u32 * max_value_size as u32;
    check_table_properties(&db, num_entries / 2, num_entries);
}

#[test]
fn test_titan_sequence_number() {
    let path = tempdir_with_prefix("test_titan_sequence_number");

    let tdb_path = path.path().join("titandb");
    let mut builder = MultiCfTitanBuilder::default();
    builder
        .set_create_if_missing(true)
        .db_options_mut()
        .set_directory(tdb_path);

    let db = builder
        .add_cf(DEFAULT_CF_NAME, TitanCfOptions::default())
        .open(path.path())
        .unwrap();

    let snap = db.snapshot();
    let snap_seq = snap.sequence_number();
    let seq1 = db.latest_sequence_number();
    assert_eq!(snap_seq, seq1);

    db.put(&WriteOptions::default(), db.default_cf(), b"key", b"value")
        .unwrap();
    let seq2 = db.latest_sequence_number();
    assert!(seq2 > seq1);
}

#[test]
fn test_titan_blob_index() {
    let mut rng = rand::thread_rng();
    let index = TitanBlobIndex::new(rng.gen(), rng.gen(), rng.gen());
    let value = index.encode();
    let index2 = TitanBlobIndex::decode(&value).unwrap();
    assert_eq!(index2.file_number, index.file_number);
    assert_eq!(index2.blob_size, index.blob_size);
    assert_eq!(index2.blob_offset, index.blob_offset);
}

// Generates a file with `range` and put it to the bottommost level.
fn generate_file_bottom_level(db: &Db, handle: &RawColumnFamilyHandle, range: ops::Range<u32>) {
    let write_opt = WriteOptions::default();
    for i in range {
        let k = format!("key{}", i);
        let v = format!("value{}", i);
        db.put(&write_opt, handle, k.as_bytes(), v.as_bytes())
            .unwrap();
    }
    db.flush(FlushOptions::default().set_wait(true), handle)
        .unwrap();

    let opts = db.cf_options(handle);
    let mut compact_opts = CompactRangeOptions::default();
    compact_opts
        .set_change_level(true)
        .set_target_level(opts.cf_options().num_levels() - 1)
        .set_bottommost_level_compaction(BottommostLevelCompaction::kSkip);
    db.compact_range(&compact_opts, handle, None, None).unwrap();
}

#[test]
fn test_titan_delete_files_in_ranges() {
    let path = tempdir_with_prefix("_rust_rocksdb_test_titan_delete_files_in_multi_ranges");
    let tdb_path = path.path().join("titandb");
    let f = SysTablePropertiesCollectorFactory::new(TitanCollectorFactory::default());
    let mut builder = MultiCfTitanBuilder::default();
    builder.db_options_mut().set_directory(tdb_path);
    let mut cf_opt = TitanCfOptions::default();
    cf_opt
        .set_min_blob_size(0)
        .add_table_properties_collector_factory(&f);
    let db = builder
        .set_create_if_missing(true)
        .add_cf(DEFAULT_CF_NAME, cf_opt)
        .open(path.path())
        .unwrap();

    let cf = db.default_cf();
    generate_file_bottom_level(&db, cf, 0..3);
    generate_file_bottom_level(&db, cf, 3..6);
    generate_file_bottom_level(&db, cf, 6..9);

    // Delete files in multiple overlapped ranges.
    // File ["key0", "key2"], ["key3", "key5"] should have been deleted,
    // but file ["key6", "key8"] should not be deleted because "key8" is exclusive.
    let mut ranges = Vec::new();
    ranges.push((Some(b"key0" as &[u8]), Some(b"key4" as &[u8])));
    ranges.push((Some(b"key2"), Some(b"key6")));
    ranges.push((Some(b"key4"), Some(b"key8")));

    db.delete_files_in_ranges(cf, &ranges, false).unwrap();
    db.delete_blob_files_in_ranges(cf, &ranges, false).unwrap();

    // Check that ["key0", "key5"] have been deleted, but ["key6", "key8"] still exist.
    let mut read_opts = ReadOptions::default();
    read_opts.set_key_only(true);
    let mut iter = db.iter(&mut read_opts, cf);
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
    let ranges = vec![(Some(b"key6" as &[u8]), Some(b"key8" as &[u8]))];
    db.delete_files_in_ranges(cf, &ranges, true).unwrap();
    db.delete_blob_files_in_ranges(cf, &ranges, true).unwrap();
    let mut iter = db.iter(&mut read_opts, cf);
    iter.seek_to_first();
    assert!(!iter.valid());
    iter.check().unwrap();
}

#[test]
fn test_get_blob_cache_usage() {
    let path = tempdir_with_prefix("_rust_rocksdb_set_blob_cache");
    let tdb_path = path.path().join("titandb");
    let mut cf_opts = TitanCfOptions::default();
    let cache = LruCacheBuilder::default()
        .set_capacity(16 * 1024 * 1024)
        .set_num_shard_bits(-1)
        .set_strict_capacity_limit(false)
        .set_high_pri_pool_ratio(0.0)
        .build();
    cf_opts.set_blob_cache(&cache).set_min_blob_size(0);
    let mut builder = MultiCfTitanBuilder::default();
    builder
        .set_create_if_missing(true)
        .db_options_mut()
        .set_directory(tdb_path);
    assert_eq!(cache.usage(), 0);

    let db = builder
        .add_cf(DEFAULT_CF_NAME, cf_opts)
        .open(path.path())
        .unwrap();
    let write_opts = WriteOptions::default();
    let cf = db.default_cf();
    let read_opts = ReadOptions::default();

    for i in 0..200 {
        db.put(&write_opts, cf, format!("k_{}", i).as_bytes(), b"v")
            .unwrap();
    }
    db.flush(FlushOptions::default().set_wait(true), cf)
        .unwrap();
    for i in 0..200 {
        db.get(&read_opts, cf, format!("k_{}", i).as_bytes())
            .unwrap();
    }

    assert!(cache.usage() > 0);
}

#[test]
fn test_titan_statistics() {
    let path = tempdir_with_prefix("_rust_rocksdb_statistics");
    let mut cf_opts = TitanCfOptions::default();
    cf_opts.set_min_blob_size(0);
    let mut builder = MultiCfTitanBuilder::default();
    let stats = Statistics::default();
    builder
        .set_create_if_missing(true)
        .add_cf(DEFAULT_CF_NAME, cf_opts)
        .db_options_mut()
        .set_statistics(&stats);
    let db = builder.open(path.path()).unwrap();

    let write_opts = WriteOptions::default();
    let read_opts = ReadOptions::default();
    let cf = db.default_cf();

    db.put(&write_opts, cf, b"k0", b"a").unwrap();
    db.put(&write_opts, cf, b"k1", b"b").unwrap();
    db.put(&write_opts, cf, b"k2", b"c").unwrap();
    db.flush(FlushOptions::default().set_wait(true), cf)
        .unwrap(); // flush memtable to sst file.
    assert_eq!(db.get(&read_opts, cf, b"k0"), Ok(Some(b"a".to_vec())));
    assert_eq!(db.get(&read_opts, cf, b"k1"), Ok(Some(b"b".to_vec())));
    assert_eq!(db.get(&read_opts, cf, b"k2"), Ok(Some(b"c".to_vec())));

    let opt = db.cf_options_titan(cf).unwrap();
    assert_eq!(opt.cf_options().min_blob_size(), 0);
    assert!(stats.ticker(TitanTickers::TITAN_NUM_GET) > 0);
    assert!(stats.take_ticker(TitanTickers::TITAN_NUM_GET) > 0);
    assert_eq!(stats.ticker(TitanTickers::TITAN_NUM_GET), 0);

    let get_micros = stats.histogram(TitanHistograms::TITAN_GET_MICROS);
    assert!(get_micros.max > 0.0);
    stats.reset();
    let get_micros = stats.histogram(TitanHistograms::TITAN_GET_MICROS);
    assert_eq!(get_micros.max, 0f64);
    assert_eq!(
        db.property_u64(cf, &PropTitanNumDiscardableRatioLe0File),
        Some(1)
    );

    assert_eq!(
        db.property_u64(cf, &PropTitanNumBlobFilesAtLevel::new(0)),
        Some(1)
    );
}
