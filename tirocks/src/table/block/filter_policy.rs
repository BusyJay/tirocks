// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use tirocks_sys::crocksdb_filterpolicy_t;

pub struct SysFilterPolicy {
    ptr: *mut crocksdb_filterpolicy_t,
}

unsafe impl Send for SysFilterPolicy {}
unsafe impl Sync for SysFilterPolicy {}

impl SysFilterPolicy {
    #[inline]
    pub fn new_bloom_filter_policy(bits_per_key: f64, use_block_based_builder: bool) -> Self {
        let ptr = unsafe {
            tirocks_sys::crocksdb_filterpolicy_create_bloom_format(
                bits_per_key,
                use_block_based_builder,
            )
        };
        SysFilterPolicy { ptr }
    }

    #[inline]
    pub(crate) fn get_ptr(&self) -> *mut crocksdb_filterpolicy_t {
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
