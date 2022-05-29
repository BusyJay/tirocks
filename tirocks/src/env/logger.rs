// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::path::Path;

use libc::c_void;
use tirocks_sys::{rocksdb_Slice, s};

use crate::{
    option::RawDbOptions,
    util::{check_status, PathToSlice},
    Result, Status,
};

pub type LogLevel = tirocks_sys::rocksdb_InfoLogLevel;

/// An interface for writing log messages.
pub trait Logger: Send + Sync {
    /// Write an entry to the log file with the specified log level.
    fn logv(&self, log_level: LogLevel, data: &[u8]);
}

extern "C" fn destructor<L: Logger>(ctx: *mut c_void) {
    unsafe {
        Box::from_raw(ctx as *mut L);
    }
}

extern "C" fn logv<L: Logger>(ctx: *mut c_void, log_level: LogLevel, log: rocksdb_Slice) {
    unsafe {
        let logger = &*(ctx as *mut L);
        logger.logv(log_level, s(log));
    }
}

/// A wrapped info logger that can be shared by multiple DBs.
pub struct SysInfoLogger {
    ptr: *mut tirocks_sys::crocksdb_logger_t,
}

unsafe impl Send for SysInfoLogger {}
unsafe impl Sync for SysInfoLogger {}

impl SysInfoLogger {
    pub fn new<L: Logger>(l: L) -> SysInfoLogger {
        let ptr = unsafe {
            let p = Box::new(l);
            tirocks_sys::crocksdb_logger_create(
                Box::into_raw(p) as *mut c_void,
                Some(destructor::<L>),
                Some(logv::<L>),
            )
        };
        SysInfoLogger { ptr }
    }

    /// Use the configuration to create an info log at given directory.
    ///
    /// If there is already an logger set to option, the logger is used directly.
    pub fn from_options(opt: &RawDbOptions, dir: impl AsRef<Path>) -> Result<SysInfoLogger> {
        unsafe {
            let mut s = Status::default();
            let ptr = tirocks_sys::crocksdb_create_log_from_options(
                dir.path_to_slice(),
                opt.as_ptr(),
                s.as_mut_ptr(),
            );
            check_status!(s)?;
            Ok(SysInfoLogger { ptr })
        }
    }

    pub(crate) fn get(&self) -> *mut tirocks_sys::crocksdb_logger_t {
        self.ptr
    }
}

impl Drop for SysInfoLogger {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            tirocks_sys::crocksdb_log_destroy(self.ptr);
        }
    }
}
