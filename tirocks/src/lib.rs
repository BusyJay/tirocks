// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

#![cfg_attr(feature = "nightly", feature(io_error_more))]

#[cfg(feature = "encryption")]
pub mod encryption;
pub mod env;
mod error;
// TODO: remove this when options are all implemented.
#[allow(unused)]
pub mod listener;
pub mod option;
pub mod rate_limiter;
// TODO: remove this when options are all implemented.
#[allow(unused)]
pub mod table_properties;
mod util;

pub use error::{Code, Result, Severity, Status, SubCode};
