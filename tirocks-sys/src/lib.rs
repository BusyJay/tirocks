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
            self.state_ = unsafe { crocksdb_to_cplus_array(state.as_ptr() as _, state.len()) };
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
    #[test]
    fn test_smoke() {
        assert_eq!(b"rocksdb.cfstats-no-file-histogram", unsafe {
            super::crocksdb_property_name_cf_stats_no_file_histogram.to_bytes()
        });
    }
}
