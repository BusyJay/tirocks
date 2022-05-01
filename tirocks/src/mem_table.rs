// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use tirocks_sys::crocksdb_memtablerepfactory_t;

#[derive(Debug)]
pub struct SysMemTableRepFactory {
    ptr: *mut crocksdb_memtablerepfactory_t,
}

impl SysMemTableRepFactory {
    #[inline]
    pub fn new_hash_skip_list(
        bucket_count: usize,
        skip_list_height: i32,
        skip_list_branching_factor: i32,
    ) -> Self {
        let ptr = unsafe {
            tirocks_sys::crocksdb_memtablerepfactory_create_hash_skip_list(
                bucket_count,
                skip_list_height,
                skip_list_branching_factor,
            )
        };
        Self { ptr }
    }

    #[inline]
    pub fn new_hash_link_list(bucket_count: usize) -> Self {
        let ptr =
            unsafe { tirocks_sys::crocksdb_memtablerepfactory_create_hash_link_list(bucket_count) };
        Self { ptr }
    }

    #[inline]
    pub fn new_doubly_skip_list(lookahead: usize) -> Self {
        let ptr =
            unsafe { tirocks_sys::crocksdb_memtablerepfactory_create_doubly_skip_list(lookahead) };
        Self { ptr }
    }

    #[inline]
    pub fn new_vector(reserved_bytes: u64) -> Self {
        let ptr = unsafe { tirocks_sys::crocksdb_memtablerepfactory_create_vector(reserved_bytes) };
        Self { ptr }
    }

    #[inline]
    pub(crate) fn get(&self) -> *mut crocksdb_memtablerepfactory_t {
        self.ptr
    }
}

impl Drop for SysMemTableRepFactory {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            tirocks_sys::crocksdb_memtablerepfactory_destroy(self.ptr);
        }
    }
}
