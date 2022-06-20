// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

#![cfg_attr(feature = "nightly", feature(io_error_more))]
// TODO: remove this when options are all implemented.
#![allow(unused)]

pub mod cache;
pub mod compaction_filter;
pub mod comparator;
#[cfg(feature = "encryption")]
pub mod encryption;
pub mod env;
mod error;
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
pub mod sst_partitioner;
pub mod statistics;
pub mod table;
pub mod table_filter;
mod util;

pub use error::{Code, Result, Severity, Status, SubCode};
pub use option::{CfOptions, DbOptions, Options};
pub use pin_slice::PinSlice;
pub use statistics::Statistics;
use tirocks_sys::rocksdb_DB;
pub use util::{CloneFactory, DefaultFactory};

// TODO: define later.
pub struct RawDb;

impl RawDb {
    pub(crate) unsafe fn from_ptr<'a>(_: *mut rocksdb_DB) -> &'a RawDb {
        unimplemented!()
    }

    pub(crate) fn get_ptr(&self) -> *mut rocksdb_DB {
        unimplemented!()
    }
}
