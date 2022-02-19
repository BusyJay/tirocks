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

    pub fn set_state(&mut self, state: &[u8]) {
        self.clear_state();
        if !state.is_empty() {
            self.state_ = unsafe { crocksdb_to_cplus_array(r(state)) };
        }
    }

    pub fn set_message(&mut self, msg: &str) {
        self.set_state(msg.as_bytes())
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
    use super::s;

    #[test]
    fn test_smoke() {
        assert_eq!(b"rocksdb.cfstats-no-file-histogram", unsafe {
            s(super::crocksdb_property_name_cf_stats_no_file_histogram)
        });
    }
}
