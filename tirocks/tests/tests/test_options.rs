// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::{ffi::CStr, path::Path, sync::Arc, time::Duration};

use tirocks::{
    cache::LruCacheBuilder,
    db::DefaultCfOnlyBuilder,
    env::{
        logger::{LogLevel, SysInfoLogger},
        Env,
    },
    mem_table::SysMemTableRepFactory,
    option::{
        CompactRangeOptions, CompactionPriority, CompactionStyle, CompressionType, FlushOptions,
        ReadOptions, ReadTier, WriteOptions,
    },
    properties::{table::user::SysTablePropertiesCollectorFactory, PropNumFilesAtLevel},
    rate_limiter::{RateLimiterBuilder, RateLimiterMode},
    slice_transform::{SliceTransform, SysSliceTransform},
    statistics::{Histograms, Tickers},
    table::{
        block::{filter_policy::SysFilterPolicy, BlockBasedTableOptions, IndexType},
        SysTableFactory,
    },
    CfOptions, DbOptions, OpenOptions, Statistics,
};

use super::tempdir_with_prefix;

#[test]
fn test_enable_statistics() {
    let mut opt = DbOptions::default();
    let s = Statistics::default();
    opt.set_statistics(&s);
    opt.set_stats_dump_period(Duration::from_secs(60));
    s.histogram(Histograms::DB_SEEK);
    assert_eq!(s.ticker(Tickers::BLOCK_CACHE_MISS), 0);
    assert_eq!(s.take_ticker(Tickers::BLOCK_CACHE_MISS), 0);
    assert_eq!(s.ticker(Tickers::BLOCK_CACHE_MISS), 0);
}

struct FixedPrefixTransform {
    pub prefix_len: usize,
}

impl SliceTransform for FixedPrefixTransform {
    fn transform<'a>(&self, key: &'a [u8]) -> &'a [u8] {
        &key[..self.prefix_len]
    }

    fn in_domain(&self, key: &[u8]) -> bool {
        key.len() >= self.prefix_len
    }

    fn name(&self) -> &std::ffi::CStr {
        CStr::from_bytes_with_nul(b"FixedPrefixTransform\0").unwrap()
    }
}

#[test]
fn test_memtable_insert_hint_prefix_extractor() {
    let path = tempdir_with_prefix("_rust_rocksdb_memtable_insert_hint_prefix_extractor");
    let mut builder = DefaultCfOnlyBuilder::default();
    builder
        .set_create_if_missing(true)
        .cf_options_mut()
        .set_memtable_insert_with_hint_prefix_extractor(&SysSliceTransform::new(
            FixedPrefixTransform { prefix_len: 2 },
        ));
    let db = builder.open(path.path()).unwrap();
    let write_opts = WriteOptions::default();
    let read_opts = ReadOptions::default();
    let cf = db.default_cf();

    db.put(&write_opts, cf, b"k0-1", b"a").unwrap();
    db.put(&write_opts, cf, b"k0-2", b"b").unwrap();
    db.put(&write_opts, cf, b"k0-3", b"c").unwrap();
    assert_eq!(db.get(&read_opts, cf, b"k0-1"), Ok(Some(b"a".to_vec())));
    assert_eq!(db.get(&read_opts, cf, b"k0-2"), Ok(Some(b"b".to_vec())));
    assert_eq!(db.get(&read_opts, cf, b"k0-3"), Ok(Some(b"c".to_vec())));
}

#[test]
fn test_set_ratelimiter() {
    let path = tempdir_with_prefix("_rust_rocksdb_test_set_rate_limiter");
    let mut builder = DefaultCfOnlyBuilder::default();
    // compaction and flush rate limited below 100MB/sec
    let rate_limiter = RateLimiterBuilder::new(100 * 1024 * 1024).build_generic();
    builder
        .set_create_if_missing(true)
        .db_options_mut()
        .set_rate_limiter(&rate_limiter);
    builder.open(path.path()).unwrap().close().unwrap();
}

#[test]
fn test_set_ratelimiter_with_auto_tuned() {
    let path = tempdir_with_prefix("_rust_rocksdb_test_set_rate_limiter_with_auto_tuned");
    let mut builder = DefaultCfOnlyBuilder::default();
    let rate_limiter = RateLimiterBuilder::new(100 * 1024 * 1024)
        .set_refill_period(Duration::from_secs(1))
        .set_mode(RateLimiterMode::kAllIo)
        .set_auto_tuned(true)
        .build_generic();
    builder
        .set_create_if_missing(true)
        .db_options_mut()
        .set_rate_limiter(&rate_limiter);
    builder.open(path.path()).unwrap().close().unwrap();
}

#[test]
fn test_set_writeampbasedratelimiter_with_auto_tuned() {
    let path =
        tempdir_with_prefix("_rust_rocksdb_test_set_write_amp_based_rate_limiter_with_auto_tuned");
    let mut builder = DefaultCfOnlyBuilder::default();
    let rate_limiter = RateLimiterBuilder::new(100 * 1024 * 1024)
        .set_refill_period(Duration::from_secs(1))
        .set_mode(RateLimiterMode::kAllIo)
        .set_auto_tuned(true)
        .build_write_amp_based();
    builder
        .set_create_if_missing(true)
        .db_options_mut()
        .set_rate_limiter(&rate_limiter);
    builder.open(path.path()).unwrap().close().unwrap();
}

#[test]
fn test_set_ratelimiter_bytes_per_second() {
    let path = tempdir_with_prefix("_rust_rocksdb_test_set_rate_limiter_bytes_per_second");
    let mut builder = DefaultCfOnlyBuilder::default();
    let rate_limiter = RateLimiterBuilder::new(100 * 1024 * 1024).build_generic();
    builder
        .set_create_if_missing(true)
        .db_options_mut()
        .set_rate_limiter(&rate_limiter);
    let db = builder.open(path.path()).unwrap();
    rate_limiter.set_bytes_per_second(200 * 1024 * 1024);
    assert_eq!(rate_limiter.bytes_per_second(), 200 * 1024 * 1024);
    db.close().unwrap();
}

#[cfg(not(windows))]
#[test]
fn test_flush_wal() {
    let path = tempdir_with_prefix("_rust_rocksdb_test_flush_wal");
    let mut builder = DefaultCfOnlyBuilder::default();
    builder
        .set_create_if_missing(true)
        .db_options_mut()
        .set_manual_wal_flush(true);
    let db = builder.open(path.path()).unwrap();
    let write_opt = WriteOptions::default();
    let cf = db.default_cf();
    db.put(&write_opt, cf, b"key", b"value").unwrap();
    db.flush_wal(false).unwrap();
    db.close().unwrap();
}

#[cfg(not(windows))]
#[test]
fn test_sync_wal() {
    let path = tempdir_with_prefix("_rust_rocksdb_test_sync_wal");
    let db = DefaultCfOnlyBuilder::default()
        .set_create_if_missing(true)
        .open(path.path())
        .unwrap();
    let write_opt = WriteOptions::default();
    let cf = db.default_cf();
    db.put(&write_opt, cf, b"key", b"value").unwrap();
    db.sync_wal().unwrap();
    db.close().unwrap();
}

#[test]
fn test_create_info_log() {
    let path = tempdir_with_prefix("_rust_rocksdb_test_create_info_log_opt");
    let mut builder = DefaultCfOnlyBuilder::default();
    builder
        .set_create_if_missing(true)
        .db_options_mut()
        .set_info_log_level(LogLevel::DEBUG_LEVEL)
        .set_log_file_time_to_roll(Duration::from_secs(1));

    let info_dir = tempdir_with_prefix("_rust_rocksdb_test_info_log_dir");
    let logger = SysInfoLogger::from_options(builder.db_options_mut(), info_dir.path()).unwrap();
    builder.db_options_mut().set_info_log(&logger);

    let db = builder.open(path.path()).unwrap();

    assert!(Path::new(info_dir.path().join("LOG").to_str().unwrap()).is_file());

    std::thread::sleep(Duration::from_secs(2));

    let write_opt = WriteOptions::default();
    let mut flush_opt = FlushOptions::default();
    let cf = db.default_cf();
    flush_opt.set_wait(true);
    for i in 0..200 {
        db.put(&write_opt, cf, format!("k_{}", i).as_bytes(), b"v")
            .unwrap();
        db.flush(&flush_opt, cf).unwrap();
    }

    db.close().unwrap();

    // The LOG must be rolled many times.
    let count = info_dir.path().read_dir().unwrap().count();
    assert!(count > 1);
}

#[test]
fn test_auto_roll_max_size_info_log() {
    let path = tempdir_with_prefix("_rust_rocksdb_test_max_size_info_log_opt");
    let mut builder = DefaultCfOnlyBuilder::default();
    builder
        .set_create_if_missing(true)
        .db_options_mut()
        .set_max_log_file_size(10);

    let info_dir = tempdir_with_prefix("_rust_rocksdb_max_size_info_log_dir");
    let log = SysInfoLogger::from_options(builder.db_options_mut(), info_dir.path()).unwrap();
    builder.db_options_mut().set_info_log(&log);

    builder.open(path.path()).unwrap().close().unwrap();

    // The LOG must be rolled many times.
    let count = info_dir.path().read_dir().unwrap().count();
    assert!(count > 1);
}

#[test]
fn test_set_pin_l0_filter_and_index_blocks_in_cache() {
    let path = tempdir_with_prefix("_rust_rocksdb_set_cache_and_index");
    let mut builder = DefaultCfOnlyBuilder::default();
    let mut bbto = BlockBasedTableOptions::default();
    bbto.set_pin_l0_filter_and_index_blocks_in_cache(true);
    let table_factory = SysTableFactory::new_block_based(&bbto);
    builder
        .set_create_if_missing(true)
        .cf_options_mut()
        .set_table_factory(&table_factory);
    builder.open(path.path()).unwrap().close().unwrap();
}

#[test]
fn test_partitioned_index_filters() {
    let path = tempdir_with_prefix("_rust_rocksdb_set_cache_and_index");
    // See https://github.com/facebook/rocksdb/wiki/Partitioned-Index-Filters#how-to-use-it
    let mut bbto = BlockBasedTableOptions::default();
    bbto.set_index_type(IndexType::kTwoLevelIndexSearch)
        .set_partition_filters(true)
        .set_filter_policy(&SysFilterPolicy::new_bloom_filter_policy(10., false))
        .set_metadata_block_size(4096)
        .set_cache_index_and_filter_blocks(true)
        .set_pin_top_level_index_and_filter(true)
        .set_cache_index_and_filter_blocks_with_high_priority(true)
        .set_pin_l0_filter_and_index_blocks_in_cache(true);
    let mut builder = DefaultCfOnlyBuilder::default();
    builder
        .set_create_if_missing(true)
        .cf_options_mut()
        .set_table_factory(&SysTableFactory::new_block_based(&bbto));
    builder.open(path.path()).unwrap().close().unwrap();
}

#[test]
fn test_set_lru_cache() {
    let path = tempdir_with_prefix("_rust_rocksdb_set_set_lru_cache");
    let mut builder = DefaultCfOnlyBuilder::default();
    builder
        .set_create_if_missing(true)
        .cf_options_mut()
        .set_table_factory(&SysTableFactory::new_block_based(
            BlockBasedTableOptions::default()
                .set_block_cache(&LruCacheBuilder::default().set_capacity(8388608).build()),
        ));
    builder.open(path.path()).unwrap().close().unwrap();
}

#[cfg(feature = "jemalloc")]
#[test]
fn test_set_jemalloc_nodump_allocator_for_lru_cache() {
    use tirocks::cache::JemallocAllocatorOptions;

    let path = tempdir_with_prefix("_rust_rocksdb_set_jemalloc_nodump_allocator");
    let cache = LruCacheBuilder::default()
        .set_use_jemalloc(&mut JemallocAllocatorOptions::default())
        .unwrap()
        .set_capacity(838608)
        .build();
    let mut builder = DefaultCfOnlyBuilder::default();
    builder
        .set_create_if_missing(true)
        .cf_options_mut()
        .set_table_factory(&SysTableFactory::new_block_based(
            BlockBasedTableOptions::default().set_block_cache(&cache),
        ));
    builder.open(path.path()).unwrap().close().unwrap();
}

#[test]
fn test_basic_configuration() {
    let path = tempdir_with_prefix("_rust_rocksdb_pending_compaction_bytes_limit");
    let wal_dir = tempdir_with_prefix("_rust_rocksdb_test_set_wal_dir");
    let mut builder = DefaultCfOnlyBuilder::default();
    builder
        .set_create_if_missing(true)
        .cf_options_mut()
        .set_soft_pending_compaction_bytes_limit(64 * 1024 * 1024 * 1024)
        .set_hard_pending_compaction_bytes_limit(256 * 1024 * 1024 * 1024)
        .set_optimize_filters_for_hits(true)
        .set_force_consistency_checks(true)
        .set_level_compaction_dynamic_level_bytes(true)
        .set_disable_write_stall(true)
        .set_compaction_priority(CompactionPriority::kMinOverlappingRatio)
        .set_compression(CompressionType::kSnappyCompression)
        .set_num_levels(2);
    builder
        .db_options_mut()
        .set_max_subcompactions(4)
        .set_bytes_per_sync(1024 * 1024)
        .set_wal_bytes_per_sync(1024 * 1024)
        .set_use_direct_reads(true)
        .set_use_direct_io_for_flush_and_compaction(true)
        .set_writable_file_max_buffer_size(1024 * 1024)
        .set_max_background_jobs(4)
        .set_compaction_readahead_size(2048)
        .set_max_manifest_file_size(20 * 1024 * 1024)
        .set_max_log_file_size(100 * 1024 * 1024)
        .set_keep_log_file_num(10)
        .set_recycle_log_file_num(10)
        .set_delayed_write_rate(2 * 1024 * 1024)
        .set_wal_ttl(Duration::from_secs(86400))
        .set_wal_size_limit(10 * 1024 * 1024)
        .set_wal_dir(wal_dir.path());
    let db = builder.open(path.path()).unwrap();
    let opt = db.cf_options(db.default_cf());
    assert!(opt.cf_options().disable_write_stall());
    assert_eq!(
        opt.cf_options().compression(),
        CompressionType::kSnappyCompression
    );
    assert_eq!(opt.cf_options().num_levels(), 2);
    db.close().unwrap();
}

#[test]
fn test_get_block_cache_usage() {
    let path = tempdir_with_prefix("_rust_rocksdb_set_cache_and_index");

    let cache = LruCacheBuilder::default()
        .set_capacity(16 * 1024 * 1024)
        .build();
    let mut builder = DefaultCfOnlyBuilder::default();
    builder
        .set_create_if_missing(true)
        .cf_options_mut()
        .set_table_factory(&SysTableFactory::new_block_based(
            BlockBasedTableOptions::default().set_block_cache(&cache),
        ));
    assert_eq!(cache.usage(), 0);
    assert_eq!(cache.capacity(), 16 * 1024 * 1024);

    let db = builder.open(path.path()).unwrap();
    let write_opt = WriteOptions::default();
    let read_opt = ReadOptions::default();
    let cf = db.default_cf();

    for i in 0..200 {
        db.put(&write_opt, cf, format!("k_{}", i).as_bytes(), b"v")
            .unwrap();
    }
    db.flush(FlushOptions::default().set_wait(true), cf)
        .unwrap();
    for i in 0..200 {
        db.get(&read_opt, cf, format!("k_{}", i).as_bytes())
            .unwrap();
    }

    assert_ne!(cache.usage(), 0);
    cache.set_capacity(32 * 1024 * 1024);
    assert_eq!(cache.capacity(), 32 * 1024 * 1024);
}

#[test]
fn test_compact_options() {
    let path = tempdir_with_prefix("_rust_rocksdb_compact_options");
    let mut builder = DefaultCfOnlyBuilder::default();
    builder
        .set_create_if_missing(true)
        .cf_options_mut()
        .set_disable_auto_compactions(true);
    let db = builder.open(path.path()).unwrap();
    let write_opt = WriteOptions::default();
    let cf = db.default_cf();
    db.put(&write_opt, cf, b"k1", b"v1").unwrap();
    db.put(&write_opt, cf, b"k2", b"v2").unwrap();
    db.flush(FlushOptions::default().set_wait(true), cf)
        .unwrap();

    let mut compact_opts = CompactRangeOptions::default();
    compact_opts.set_exclusive_manual_compaction(false);
    db.compact_range(&compact_opts, cf, None, None).unwrap();
}

#[test]
fn test_basic_configuration_with_write() {
    let path = tempdir_with_prefix("_rust_rocksdb_basic_configuration_with_write");
    let mut builder = DefaultCfOnlyBuilder::default();
    builder.options_mut().set_env(Arc::new(Env::default()));
    builder
        .set_create_if_missing(true)
        .db_options_mut()
        .set_allow_concurrent_memtable_write(false)
        .set_manual_wal_flush(true);
    builder
        .cf_options_mut()
        .set_max_bytes_for_level_multiplier(8.0)
        .set_compaction_style(CompactionStyle::kCompactionStyleFIFO)
        .compaction_options_fifo_mut()
        .set_allow_compaction(true)
        .set_max_table_files_size(100000);
    let db = builder.open(path.path()).unwrap();
    let write_opt = WriteOptions::default();
    let cf = db.default_cf();
    for i in 0..200 {
        db.put(&write_opt, cf, format!("k_{}", i).as_bytes(), b"v")
            .unwrap();
    }
    let opt = db.cf_options(cf);
    assert_eq!(opt.cf_options().max_bytes_for_level_multiplier(), 8.0);
}

#[test]
fn test_enable_pipelined_write() {
    let path = tempdir_with_prefix("_rust_rocksdb_enable_pipelined_write");
    let tmp_path = tempdir_with_prefix("_rust_rocksdb_test_db_path");
    let mut builder = DefaultCfOnlyBuilder::default();
    builder
        .set_create_if_missing(true)
        .db_options_mut()
        .set_enable_pipelined_write(true)
        .add_db_path(tmp_path.path(), 1073741824);
    builder
        .cf_options_mut()
        .set_bottommost_compression(CompressionType::kNoCompression);
    let db = builder.open(path.path()).unwrap();
    let write_opt = WriteOptions::default();
    let cf = db.default_cf();
    for i in 0..200 {
        db.put(&write_opt, cf, format!("k_{}", i).as_bytes(), b"v")
            .unwrap();
    }
}

#[test]
fn test_get_compressions() {
    let mut cf_opts = CfOptions::default();
    let compressions = &[
        CompressionType::kNoCompression,
        CompressionType::kSnappyCompression,
    ];
    cf_opts.set_compression_per_level(compressions);
    let v = cf_opts.compression_per_level();
    assert_eq!(v, compressions);
    let mut cf_opts2 = CfOptions::default();
    cf_opts2.set_compression_per_level(&[]);
    let v2 = cf_opts2.compression_per_level();
    assert!(v2.is_empty());
}

#[test]
fn test_write_options() {
    let path = tempdir_with_prefix("_rust_rocksdb_write_options");
    let db = DefaultCfOnlyBuilder::default()
        .set_create_if_missing(true)
        .open(path.path())
        .unwrap();

    let mut write_opts = WriteOptions::default();
    write_opts
        .set_ignore_missing_column_families(true)
        .set_no_slowdown(true)
        .set_low_priority(true);
    let cf = db.default_cf();
    db.put(&write_opts, cf, b"k1", b"a").unwrap();
    write_opts.set_sync(true).set_disable_wal(false);
    db.put(&write_opts, cf, b"k2", b"b").unwrap();
    write_opts.set_sync(false).set_disable_wal(true);
    db.put(&write_opts, cf, b"k3", b"c").unwrap();
    let read_opts = ReadOptions::default();
    assert_eq!(db.get(&read_opts, cf, b"k1").unwrap().unwrap(), b"a");
    assert_eq!(db.get(&read_opts, cf, b"k2").unwrap().unwrap(), b"b");
    assert_eq!(db.get(&read_opts, cf, b"k3").unwrap().unwrap(), b"c");
}

#[test]
fn test_read_options() {
    let path = tempdir_with_prefix("_rust_rocksdb_write_options");
    let db = DefaultCfOnlyBuilder::default()
        .set_create_if_missing(true)
        .open(path.path())
        .unwrap();

    let mut read_opts = ReadOptions::default();
    read_opts
        .set_verify_checksums(true)
        .set_fill_cache(true)
        .set_tailing(true)
        .set_pin_data(true)
        .set_background_purge_on_iterator_cleanup(true)
        .set_ignore_range_deletions(true)
        .set_max_skippable_internal_keys(0)
        .set_readahead_size(0)
        .set_read_tier(ReadTier::kMemtableTier);

    let cf = db.default_cf();
    let write_opts = WriteOptions::default();
    db.put(&write_opts, cf, b"k1", b"a").unwrap();
    db.put(&write_opts, cf, b"k2", b"b").unwrap();
    db.put(&write_opts, cf, b"k3", b"c").unwrap();

    let keys = vec![b"k1", b"k2", b"k3"];
    let mut iter = db.iter(&mut read_opts, cf);
    iter.seek(b"k1");
    let mut key_count = 0;
    while iter.valid() {
        assert_eq!(keys[key_count], iter.key());
        key_count = key_count + 1;
        iter.next();
    }
    assert!(key_count == 3);
    iter.check().unwrap();
}

#[test]
fn test_readoptions_lower_bound() {
    let path = tempdir_with_prefix("_rust_rocksdb_readoptions_lower_bound");
    let db = DefaultCfOnlyBuilder::default()
        .set_create_if_missing(true)
        .open(path.path())
        .unwrap();

    let cf = db.default_cf();
    let write_opts = WriteOptions::default();
    db.put(&write_opts, cf, b"k1", b"b").unwrap();
    db.put(&write_opts, cf, b"k2", b"a").unwrap();
    db.put(&write_opts, cf, b"k3", b"a").unwrap();

    let mut read_opts = ReadOptions::default();
    let lower_bound = b"k2".to_vec();
    read_opts.set_iterate_lower_bound(lower_bound);
    let mut iter = db.iter(&mut read_opts, cf);
    iter.seek(b"k3");
    let mut count = 0;
    while iter.valid() {
        count += 1;
        iter.prev();
    }
    assert_eq!(count, 2);
    iter.check().unwrap();
}

#[test]
fn test_block_based_options() {
    let path = tempdir_with_prefix("_rust_rocksdb_block_based_options");
    let stats = Statistics::default();
    let mut builder = DefaultCfOnlyBuilder::default();
    builder
        .set_create_if_missing(true)
        .db_options_mut()
        .set_statistics(&stats)
        .set_stats_dump_period(Duration::from_secs(60));
    builder
        .cf_options_mut()
        .set_table_factory(&SysTableFactory::new_block_based(
            BlockBasedTableOptions::default().set_read_amp_bytes_per_bit(4),
        ));
    let db = builder.open(path.path()).unwrap();

    // RocksDB use randomness for the read amplification statistics,
    // we should use a bigger enough value (> `bytes_per_bit`) to make
    // sure the statistics will not be 0.
    let cf = db.default_cf();
    let write_opts = WriteOptions::default();
    db.put(&write_opts, cf, b"a", b"abcdef").unwrap();
    db.flush(FlushOptions::default().set_wait(true), cf)
        .unwrap();
    let read_opts = ReadOptions::default();
    db.get(&read_opts, cf, b"a").unwrap();
    assert_ne!(stats.ticker(Tickers::READ_AMP_TOTAL_READ_BYTES), 0);
    assert_ne!(stats.ticker(Tickers::READ_AMP_ESTIMATE_USEFUL_BYTES), 0);
}

#[test]
fn test_vector_memtable_factory_options() {
    let path = tempdir_with_prefix("_rust_rocksdb_vector_memtable_factory_options");
    let mut builder = DefaultCfOnlyBuilder::default();
    builder
        .set_create_if_missing(true)
        .db_options_mut()
        .set_allow_concurrent_memtable_write(false);
    builder
        .cf_options_mut()
        .set_memtable_factory(&SysMemTableRepFactory::with_vector(4096));
    let db = builder.open(path.path()).unwrap();
    let write_opts = WriteOptions::default();
    let cf = db.default_cf();
    db.put(&write_opts, cf, b"k1", b"v1").unwrap();
    db.put(&write_opts, cf, b"k2", b"v2").unwrap();
    let mut read_opts = ReadOptions::default();
    assert_eq!(db.get(&read_opts, cf, b"k1"), Ok(Some(b"v1".to_vec())));
    assert_eq!(db.get(&read_opts, cf, b"k2"), Ok(Some(b"v2".to_vec())));
    db.flush(FlushOptions::default().set_wait(true), cf)
        .unwrap();

    let mut iter = db.iter(&mut read_opts, cf);
    iter.seek_to_first();
    assert!(iter.valid());
    assert_eq!(iter.key(), b"k1");
    assert_eq!(iter.value(), b"v1");
    iter.next();
    assert!(iter.valid());
    assert_eq!(iter.key(), b"k2");
    assert_eq!(iter.value(), b"v2");
    iter.next();
    assert!(!iter.valid());
}

#[test]
fn test_compact_on_deletion() {
    let num_keys = 1000;
    let window_size = 100;
    let dels_trigger = 90;

    let path = tempdir_with_prefix("_rust_rocksdb_compact_on_deletion_test");
    let mut builder = DefaultCfOnlyBuilder::default();
    builder
        .set_create_if_missing(true)
        .cf_options_mut()
        .add_table_properties_collector_factory(
            &SysTablePropertiesCollectorFactory::with_compact_on_deletion(
                window_size,
                dels_trigger,
            ),
        );
    let db = builder.open(path.path()).unwrap();
    let cf = db.default_cf();
    let write_opts = WriteOptions::default();
    db.put(&write_opts, cf, b"key0", b"value").unwrap();
    db.flush(FlushOptions::default().set_wait(true), cf)
        .unwrap();
    let mut opt = CompactRangeOptions::default();
    opt.set_change_level(true).set_target_level(1);
    db.compact_range(&opt, cf, None, None).unwrap();

    assert_eq!(
        db.property(cf, &PropNumFilesAtLevel::new(1)),
        Ok(Some("1".to_string()))
    );

    for i in 0..num_keys {
        if i >= num_keys - window_size && i < num_keys - window_size + dels_trigger {
            db.delete(&write_opts, cf, format!("key{}", i).as_ref())
                .unwrap();
        } else {
            db.put(&write_opts, cf, format!("key{}", i).as_ref(), b"value")
                .unwrap();
        }
    }
    db.flush(FlushOptions::default().set_wait(true), cf)
        .unwrap();

    let max_retry = 1000;
    for _ in 0..max_retry {
        if db.property(cf, &PropNumFilesAtLevel::new(0)) == Ok(Some("0".to_string())) {
            break;
        } else {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
    assert_eq!(
        db.property(cf, &PropNumFilesAtLevel::new(0)),
        Ok(Some("0".to_string()))
    );
    assert_eq!(
        db.property(cf, &PropNumFilesAtLevel::new(1)),
        Ok(Some("1".to_string()))
    );
}
