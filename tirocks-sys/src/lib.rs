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

#[cfg(test)]
mod tests {
    #[test]
    fn test_smoke() {
        assert_eq!(b"rocksdb.cfstats-no-file-histogram", unsafe {
            super::crocksdb_property_name_cf_stats_no_file_histogram.to_bytes()
        });
    }
}
