// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

// bzip2 requires a user-defined `bz_internal_error` hook to handle errors
// as it's an alloc free library. bzip2_sys provides a default implementation.
extern crate bzip2_sys;

#[allow(clippy::all)]
mod bindings {
    include!(env!("BINDING_PATH"));
}

use std::{ffi::CStr, str::Utf8Error};

pub use bindings::*;

/// Convert a rust slice to rocksdb slice.
///
/// ### Safety
///
/// Caller should guarantee the data lives as long as the returned slice.
// Because the layout of rocksdb_Slice is exact the same as &[u8], this function
// will be optimized away by compiler.
#[inline]
pub unsafe fn r(src: &[u8]) -> rocksdb_Slice {
    rocksdb_Slice {
        data_: src.as_ptr() as _,
        size_: src.len(),
    }
}

/// Convert a rocksdb slice to rust slice.
///
/// ### Safety
///
/// Caller should guarantee the data lives as long as the returned slice.
// Because the layout of rocksdb_Slice is exact the same as &[u8], this function
// will be optimized away by compiler.
#[inline]
pub unsafe fn s<'a>(src: rocksdb_Slice) -> &'a [u8] {
    std::slice::from_raw_parts(src.data_ as _, src.size_)
}

impl rocksdb_Status {
    #[inline]
    pub fn with_code(code: rocksdb_Status_Code) -> rocksdb_Status {
        rocksdb_Status {
            code_: code,
            subcode_: rocksdb_Status_SubCode::kNone,
            sev_: rocksdb_Status_Severity::kNoError,
            state_: std::ptr::null(),
        }
    }

    #[inline]
    fn clear_state(&mut self) {
        if self.state_.is_null() {
            return;
        }
        unsafe {
            crocksdb_free_cplus_array(self.state_);
            self.state_ = std::ptr::null();
        }
    }

    #[inline]
    pub fn set_state(&mut self, state: &[u8]) {
        self.clear_state();
        if !state.is_empty() {
            self.state_ = unsafe { crocksdb_to_cplus_array(r(state)) };
        }
    }

    #[inline]
    pub fn set_message(&mut self, msg: &str) {
        self.set_state(msg.as_bytes())
    }

    #[inline]
    pub fn ok(&self) -> bool {
        self.code_ == rocksdb_Status_Code::kOk
    }

    #[inline]
    pub fn state(&self) -> Option<&[u8]> {
        if self.state_.is_null() {
            None
        } else {
            unsafe { Some(CStr::from_ptr(self.state_).to_bytes()) }
        }
    }

    #[inline]
    pub fn message(&self) -> Result<Option<&str>, Utf8Error> {
        if self.state_.is_null() {
            Ok(None)
        } else {
            unsafe { CStr::from_ptr(self.state_).to_str().map(Some) }
        }
    }

    #[inline]
    pub fn code(&self) -> rocksdb_Status_Code {
        self.code_
    }
}

impl Drop for rocksdb_Status {
    #[inline]
    fn drop(&mut self) {
        self.clear_state();
    }
}

#[macro_export]
macro_rules! ffi_try {
    ($func:ident($($arg:expr),+)) => ({
        let mut status = $crate::rocksdb_Status::with_code($crate::rocksdb_Status_Code::kOk);
        let res = $crate::$func($($arg),+, &mut status);
        if status.ok() {
            res
        } else {
            return Err(status.into());
        }
    });
    ($func:ident()) => ({
        let mut status = $crate::rocksdb_Status::with_code($crate::rocksdb_Status_Code::kOk);
        let res = $crate::$func(&mut status);
        if status.ok() {
            res
        } else {
            return Err(status.into());
        }
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        crocksdb_close, crocksdb_options_create, crocksdb_options_destroy,
        crocksdb_options_set_create_if_missing, rocksdb_Status_Code,
    };

    use super::s;

    #[test]
    fn test_smoke() {
        assert_eq!(b"rocksdb.cfstats-no-file-histogram", unsafe {
            s(super::crocksdb_property_name_cf_stats_no_file_histogram)
        });
    }

    #[test]
    fn test_status() {
        let td = tempfile::tempdir().unwrap();
        let path = td.path().to_str().unwrap();
        unsafe {
            let opt = crocksdb_options_create();
            crocksdb_options_set_create_if_missing(opt, 0);
            let s: Result<_, super::rocksdb_Status> =
                (|| Ok(ffi_try!(crocksdb_open(opt, path.as_ptr() as _))))();
            let e = s.unwrap_err();
            assert!(!e.ok());
            assert_eq!(e.code_, rocksdb_Status_Code::kInvalidArgument);
            let msg = e.message().unwrap().unwrap();
            assert!(msg.contains("does not exist"), "{}", msg);

            crocksdb_options_set_create_if_missing(opt, 1);
            let s: Result<_, super::rocksdb_Status> =
                (|| Ok(ffi_try!(crocksdb_open(opt, path.as_ptr() as _))))();
            let e = s.unwrap();
            crocksdb_close(e);
            crocksdb_options_destroy(opt);
        }
    }
}
