// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use tirocks::{option::{CompressionType, WriteOptions, FlushOptions, ReadOptions}, Options, db::DefaultCfOnlyBuilder, OpenOptions, Statistics, statistics::{Tickers, Histograms}};

#[test]
// Make sure all compression types are supported.
fn test_compression() {
    let path = super::tempdir_with_prefix("_rust_rocksdb_test_metadata");
    let compression_types = [
        CompressionType::kSnappyCompression,
        CompressionType::kZlibCompression,
        CompressionType::kBZip2Compression,
        CompressionType::kLZ4Compression,
        CompressionType::kLZ4HCCompression,
        CompressionType::kZSTD,
    ];
    for compression_type in compression_types {
        let mut opts = Options::default();
        opts.db_options_mut().set_create_if_missing(true);
        opts.cf_options_mut().set_compression(compression_type);
        // DB open will fail if compression type is not supported.
        DefaultCfOnlyBuilder::default().set_options(opts).open(path.path()).unwrap();
    }
}

#[test]
fn test_db_statistics() {
    let path = super::tempdir_with_prefix("_rust_rocksdb_statistics");
    let mut builder = DefaultCfOnlyBuilder::default();
    let stats = Statistics::default();
    builder.options_mut().db_options_mut().set_create_if_missing(true).set_statistics(&stats);
    let db = builder.open(path.path()).unwrap();
    
    let wopts = WriteOptions::default();
    let cf = db.default_cf();

    db.put(&wopts, cf, b"k0", b"a").unwrap();
    db.put(&wopts, cf, b"k1", b"b").unwrap();
    db.put(&wopts, cf, b"k2", b"c").unwrap();
    db.flush(FlushOptions::default().set_wait(true), cf).unwrap(); // flush memtable to sst file.

    let mut read_opts = ReadOptions::default();
    assert_eq!(db.get(&mut read_opts, cf, b"k0"), Ok(Some(b"a".to_vec())));
    assert_eq!(db.get(&mut read_opts, cf, b"k1"), Ok(Some(b"b".to_vec())));
    assert_eq!(db.get(&mut read_opts, cf, b"k2"), Ok(Some(b"c".to_vec())));

    assert!(stats.ticker(Tickers::BLOCK_CACHE_HIT) > 0);
    assert!(stats.take_ticker(Tickers::BLOCK_CACHE_HIT) > 0);
    assert_eq!(stats.ticker(Tickers::BLOCK_CACHE_HIT), 0);
    let mut get_his = stats.histogram(Histograms::DB_GET);
    assert_ne!(get_his.max, 0f64);

    stats.reset();
    get_his = stats.histogram(Histograms::DB_GET);
    assert_eq!(get_his.max, 0f64);
}
