// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::str::{self, Utf8Error};

use tirocks_sys::{crocksdb_memtablerepfactory_t, r, s};

#[derive(Debug)]
pub struct SysMemTableRepFactory {
    ptr: *mut crocksdb_memtablerepfactory_t,
}

impl SysMemTableRepFactory {
    #[inline]
    pub fn with_hash_skip_list(
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
    pub fn with_hash_link_list(bucket_count: usize) -> Self {
        let ptr =
            unsafe { tirocks_sys::crocksdb_memtablerepfactory_create_hash_link_list(bucket_count) };
        Self { ptr }
    }

    #[inline]
    pub fn with_doubly_skip_list(lookahead: usize) -> Self {
        let ptr =
            unsafe { tirocks_sys::crocksdb_memtablerepfactory_create_doubly_skip_list(lookahead) };
        Self { ptr }
    }

    #[inline]
    pub fn with_vector(reserved_bytes: u64) -> Self {
        let ptr = unsafe { tirocks_sys::crocksdb_memtablerepfactory_create_vector(reserved_bytes) };
        Self { ptr }
    }

    #[inline]
    pub fn name(&self) -> std::result::Result<&str, Utf8Error> {
        unsafe {
            let mut name = r(&[]);
            tirocks_sys::crocksdb_memtablerepfactory_name(self.ptr, &mut name);
            str::from_utf8(s(name))
        }
    }

    #[inline]
    pub(crate) fn get_ptr(&self) -> *mut crocksdb_memtablerepfactory_t {
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
