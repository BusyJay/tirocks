// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

#![cfg_attr(feature = "nightly", feature(io_error_more))]
#![cfg_attr(feature = "nightly", feature(test))]
// TODO: remove this when options are all implemented.
#![allow(unused)]

#[cfg(feature = "nightly")]
extern crate test;

pub mod cache;
pub mod compaction_filter;
pub mod comparator;
pub mod db;
#[cfg(feature = "encryption")]
pub mod encryption;
pub mod env;
mod error;
mod iterator;
pub mod listener;
pub mod mem_table;
pub mod merge_operator;
pub mod metadata;
pub mod option;
pub mod perf_context;
mod pin_slice;
pub mod properties;
pub mod rate_limiter;
pub mod slice_transform;
mod snapshot;
pub mod sst_partitioner;
pub mod statistics;
pub mod table;
pub mod table_filter;
mod util;
pub mod write_batch;

pub use db::{Db, RawDb};
pub use error::{Code, Result, Severity, Status, SubCode};
pub use iterator::{Iterable, Iterator, RawIterator};
pub use option::{CfOptions, DbOptions, Options};
pub use pin_slice::PinSlice;
pub use snapshot::{RawSnapshot, Snapshot};
pub use statistics::Statistics;
pub use util::{CloneFactory, DefaultFactory, RustRange};
pub use write_batch::{WriteBatch, WriteBatchIter};

#[cfg(test)]
mod tests {
    use super::*;
    use db::OpenOptions;

    #[cfg(feature = "nightly")]
    #[bench]
    fn bench_write_with_callback(b: &mut test::Bencher) {
        let td = tempfile::tempdir().unwrap();
        let db = db::DefaultCfOnlyBuilder::default()
            .set_create_if_missing(true)
            .open(td.path())
            .unwrap();
        let mut opts = option::WriteOptions::default();
        opts.set_sync(false);
        opts.set_disable_wal(true);
        let mut value = 0;
        let mut batch = WriteBatch::default();
        b.iter(|| {
            db.write_with_callback(&opts, &mut batch, &mut || {
                value += 1;
            })
            .unwrap();
        });
        assert_ne!(value, 0);
    }

    #[cfg(feature = "nightly")]
    #[bench]
    fn bench_write_without_callback(b: &mut test::Bencher) {
        let td = tempfile::tempdir().unwrap();
        let db = db::DefaultCfOnlyBuilder::default()
            .set_create_if_missing(true)
            .open(td.path())
            .unwrap();
        let mut opts = option::WriteOptions::default();
        opts.set_sync(false);
        opts.set_disable_wal(true);
        let mut value = 0;
        let mut batch = WriteBatch::default();
        b.iter(|| {
            db.write(&opts, &mut batch).unwrap();
            value += 1;
        });
        assert_ne!(value, 0);
    }
}
