// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use tirocks_sys::crocksdb_filterpolicy_t;

pub struct SysFilterPolicy {
    ptr: *mut crocksdb_filterpolicy_t,
}

impl SysFilterPolicy {
    #[inline]
    pub fn new_bloom_filter_policy(bits_per_key: i32, use_block_based_builder: bool) -> Self {
        let ptr = unsafe {
            tirocks_sys::crocksdb_filterpolicy_create_bloom_format(
                bits_per_key,
                use_block_based_builder,
            )
        };
        SysFilterPolicy { ptr }
    }

    #[inline]
    pub(crate) fn get(&self) -> *mut crocksdb_filterpolicy_t {
        self.ptr
    }
}

impl Drop for SysFilterPolicy {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            tirocks_sys::crocksdb_filterpolicy_destroy(self.ptr);
        }
    }
}
