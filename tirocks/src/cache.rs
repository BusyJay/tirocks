// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use tirocks_sys::{crocksdb_cache_t, rocksdb_LRUCacheOptions};

use crate::util::simple_access;
use crate::{Result, Status};

pub type JemallocAllocatorOptions = tirocks_sys::rocksdb_JemallocAllocatorOptions;

pub struct LruCacheBuilder {
    ptr: *mut rocksdb_LRUCacheOptions,
}

impl LruCacheBuilder {
    simple_access! {
        crocksdb_lru_cache_options

        /// Capacity of the cache.
        capacity: usize

        /// Cache is sharded into 2^num_shard_bits shards,
        /// by hash of key. Refer to NewLRUCache for further
        /// information.
        num_shard_bits: i32

        // If strict_capacity_limit is set,
        // insert to the cache will fail when cache is full.
        strict_capacity_limit: bool

        // Percentage of cache reserved for high priority entries.
        // If greater than zero, the LRU list will be split into a high-pri
        // list and a low-pri list. High-pri entries will be insert to the
        // tail of high-pri list, while low-pri entries will be first inserted to
        // the low-pri list (the midpoint). This is refered to as
        // midpoint insertion strategy to make entries never get hit in cache
        // age out faster.
        //
        // See also
        // BlockBasedTableOptions::cache_index_and_filter_blocks_with_high_priority.
        high_pri_pool_ratio: f64
    }

    /// Uses jemalloc allocator instead of system allocator to allocate memory for cache.
    #[inline]
    pub fn set_use_jemalloc(&mut self, opt: &mut JemallocAllocatorOptions) -> Result<&mut Self> {
        unsafe {
            let mut s = Status::default();
            tirocks_sys::crocksdb_lru_cache_options_set_use_jemalloc(self.ptr, opt, s.as_mut_ptr());
            if s.ok() {
                Ok(self)
            } else {
                Err(s)
            }
        }
    }

    pub fn build(&self) -> SysCache {
        unsafe {
            let ptr = tirocks_sys::crocksdb_cache_create_lru(self.ptr);
            SysCache { ptr }
        }
    }
}

impl Drop for LruCacheBuilder {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            tirocks_sys::crocksdb_lru_cache_options_destroy(self.ptr);
        }
    }
}

#[derive(Debug)]
pub struct SysCache {
    ptr: *mut crocksdb_cache_t,
}

impl SysCache {
    #[inline]
    pub(crate) fn get(&self) -> *mut crocksdb_cache_t {
        self.ptr
    }
}

impl Drop for SysCache {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            tirocks_sys::crocksdb_cache_destroy(self.ptr);
        }
    }
}
