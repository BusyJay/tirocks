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

pub use bindings::*;

impl crocksdb_slice_t {
    pub fn to_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.data as _, self.size) }
    }
}

#[repr(transparent)]
pub struct Status(ManuallyDrop<rocksdb_Status>);

impl Drop for Status {
    #[inline]
    fn drop(&mut self) {
        if !self.0.state_.is_null() {
            unsafe {
                crocksdb_status_destroy()
            }
        }
    }
}

/// Convert a Rust slice to crocksdb_slice_t.
/// 
/// # Safety
/// 
/// Caller should guarantee returned value should not outlive `src`.
#[inline]
pub unsafe fn s(src: &[u8]) -> crocksdb_slice_t {
    crocksdb_slice_t {
        data: src.as_ptr() as _,
        size: src.len(),
    }
}

use std::{ffi::CStr, mem::ManuallyDrop};
use libc::{c_char, c_void};

/// # Safety
///
/// ptr must point to a valid CStr value
pub unsafe fn error_message(ptr: *mut c_char) -> String {
    let c_str = CStr::from_ptr(ptr);
    let s = format!("{}", c_str.to_string_lossy());
    libc::free(ptr as *mut c_void);
    s
}

pub struct Error(pub String);

#[macro_export]
macro_rules! ffi_try {
    ($func:ident($($arg:expr),+)) => ({
        use std::ptr;
        let mut err = ptr::null_mut();
        let res = $crate::$func($($arg),+, &mut err);
        if err.is_null() {
            res
        } else {
            return Err($crate::Error($crate::error_message(err)).into());
        }
    });
    ($func:ident()) => ({
        use std::ptr;
        let mut err = ptr::null_mut();
        let res = $crate::$func(&mut err);
        if err.is_null() {
            res
        } else {
            return Err($crate::Error($crate::error_message(err)).into());
        }
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_smoke() {
        assert_eq!(b"rocksdb.cfstats-no-file-histogram", unsafe {
            super::crocksdb_property_name_cf_stats_no_file_histogram.to_bytes()
        });
    }
}
