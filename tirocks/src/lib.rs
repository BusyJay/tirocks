// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

#![cfg_attr(feature = "nightly", feature(io_error_more))]
// TODO: remove this when options are all implemented.
#![allow(unused)]

pub mod cache;
pub mod comparator;
#[cfg(feature = "encryption")]
pub mod encryption;
pub mod env;
mod error;
pub mod listener;
pub mod option;
pub mod properties;
pub mod rate_limiter;
pub mod statistics;
// TODO: remove this when options are all implemented.
#[allow(unused)]
pub mod table;
pub mod table_filter;
mod util;

pub use error::{Code, Result, Severity, Status, SubCode};
pub use statistics::Statistics;
use tirocks_sys::rocksdb_DB;

// TODO: define later.
pub struct RawDb;

impl RawDb {
    pub(crate) unsafe fn from_ptr<'a>(_: *mut rocksdb_DB) -> &'a RawDb {
        unimplemented!()
    }
}
