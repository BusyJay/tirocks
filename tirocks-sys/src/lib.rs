// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

// bzip2 requires a user-defined `bz_internal_error` hook to handle errors
// as it's an alloc free library. bzip2_sys provides a default implementation.
extern crate bzip2_sys;

#[allow(clippy::all)]
mod bindings {
    // We don't want these types be generated as they have different sizes on different platforms
    // so we declare them manually.

    include!("pre_defined.rs");

    include!(env!("BINDING_PATH"));
}

use std::{ffi::CStr, mem, str::Utf8Error};

pub use bindings::*;

/// Check if it's safe to convert transmute from `&[u8]` to `rocksdb_Slice`.
///
/// Following code should be optimized away in release build.
pub fn rocks_slice_same_as_rust() -> bool {
    // Type is important.
    let rst: &[u8] = &[1, 2];
    unsafe {
        let rocks = r(rst);
        let size = mem::size_of_val(&rst);
        if mem::size_of_val(&rocks) != mem::size_of_val(&rst) || size != 16 {
            return false;
        }
        if mem::align_of_val(&rocks) != mem::align_of_val(&rst) {
            return false;
        }
        let rst_ptr = &rst as *const _ as *const u8;
        let rocks_ptr = &rocks as *const _ as *const u8;
        // libc::memcpy doesn't trigger optimization correctly.
        for i in 0..16 {
            if *rst_ptr.offset(i) != *rocks_ptr.offset(i) {
                return false;
            }
        }
        true
    }
}

/// Convert a rust slice to rocksdb slice.
///
/// # Safety
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
/// # Safety
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

    /// Build a status with given errors.
    ///
    /// # Panics
    ///
    /// `code` should not be `kOK`.
    #[inline]
    pub fn with_error(code: rocksdb_Status_Code, state: impl AsRef<[u8]>) -> rocksdb_Status {
        assert_ne!(code, rocksdb_Status_Code::kOk);
        let state = state.as_ref();
        let state_ = if !state.is_empty() {
            unsafe { crocksdb_to_cplus_array(r(state)) }
        } else {
            std::ptr::null()
        };
        rocksdb_Status {
            code_: code,
            subcode_: rocksdb_Status_SubCode::kNone,
            sev_: rocksdb_Status_Severity::kNoError,
            state_,
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

    #[inline]
    pub fn severity(&self) -> rocksdb_Status_Severity {
        self.sev_
    }

    #[inline]
    pub fn set_severity(&mut self, severity: rocksdb_Status_Severity) {
        self.sev_ = severity;
    }

    #[inline]
    pub fn sub_code(&self) -> rocksdb_Status_SubCode {
        self.subcode_
    }

    #[inline]
    pub fn set_sub_code(&mut self, sub_code: rocksdb_Status_SubCode) {
        self.subcode_ = sub_code;
    }
}

impl PartialEq for rocksdb_Status {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.code_ == other.code_
    }
}

impl Drop for rocksdb_Status {
    #[inline]
    fn drop(&mut self) {
        self.clear_state();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smoke() {
        assert_eq!(b"rocksdb.num-files-at-level", unsafe {
            let mut buf = r(&[]);
            super::crocksdb_property_name_num_files_at_level_prefix(&mut buf);
            s(buf)
        });
    }

    #[test]
    fn test_status() {
        let td = tempfile::tempdir().unwrap();
        let path = td.path().to_str().unwrap();
        unsafe {
            let opt = crocksdb_options_create();
            let db_opt = crocksdb_options_get_dboptions(opt);
            crocksdb_options_set_create_if_missing(db_opt, false);
            let mut status = rocksdb_Status::with_code(rocksdb_Status_Code::kOk);
            let s = crocksdb_open(opt, r(path.as_bytes()), &mut status);
            assert!(!status.ok());
            assert!(s.is_null());
            assert_eq!(status.code(), rocksdb_Status_Code::kInvalidArgument);
            let msg = status.message().unwrap().unwrap();
            assert!(msg.contains("does not exist"), "{}", msg);

            status.clear_state();
            crocksdb_options_set_create_if_missing(db_opt, true);
            let s = crocksdb_open(opt, r(path.as_bytes()), &mut status);
            assert!(status.ok(), "{:?}", status.message());
            crocksdb_close(s, &mut status);
            assert!(status.ok(), "{:?}", status.message());
            crocksdb_destroy(s);
            crocksdb_options_destroy(opt);
        }
    }
}
