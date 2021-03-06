// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use tirocks_sys::{crocksdb_cache_t, rocksdb_LRUCacheOptions};

use crate::util::simple_access;

#[cfg(feature = "jemalloc")]
#[derive(Debug)]
#[repr(transparent)]
pub struct JemallocAllocatorOptions(tirocks_sys::rocksdb_JemallocAllocatorOptions);

#[cfg(feature = "jemalloc")]
mod imp {
    use super::JemallocAllocatorOptions;
    use std::mem::MaybeUninit;

    impl Default for JemallocAllocatorOptions {
        #[inline]
        fn default() -> Self {
            unsafe {
                let mut opt = MaybeUninit::uninit();
                tirocks_sys::crocksdb_jemallocallocatoroptions_init(opt.as_mut_ptr());
                JemallocAllocatorOptions(opt.assume_init())
            }
        }
    }

    impl JemallocAllocatorOptions {
        /// Jemalloc tcache cache allocations by size class. For each size class,
        /// it caches between 20 (for large size classes) to 200 (for small size
        /// classes). To reduce tcache memory usage in case the allocator is access
        /// by large number of threads, we can control whether to cache an allocation
        /// by its size.
        #[inline]
        pub fn set_limit_tcache_size(&mut self, limit: bool) -> &mut Self {
            self.0.limit_tcache_size = limit;
            self
        }

        /// Lower bound of allocation size to use tcache, if limit_tcache_size=true.
        /// When used with block cache, it is recommneded to set it to block_size/4.
        #[inline]
        pub fn set_tcache_size_lower_bound(&mut self, bound: usize) -> &mut Self {
            self.0.tcache_size_lower_bound = bound;
            self
        }

        /// Upper bound of allocation size to use tcache, if limit_tcache_size=true.
        /// When used with block cache, it is recommneded to set it to block_size.
        #[inline]
        pub fn set_tcache_size_upper_bound(&mut self, bound: usize) -> &mut Self {
            self.0.tcache_size_upper_bound = bound;
            self
        }
    }
}

pub struct LruCacheBuilder {
    ptr: *mut rocksdb_LRUCacheOptions,
}

impl Default for LruCacheBuilder {
    #[inline]
    fn default() -> Self {
        unsafe {
            let ptr = tirocks_sys::crocksdb_lru_cache_options_create();
            LruCacheBuilder { ptr }
        }
    }
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
    #[cfg(feature = "jemalloc")]
    #[inline]
    pub fn set_use_jemalloc(
        &mut self,
        opt: &mut JemallocAllocatorOptions,
    ) -> crate::Result<&mut Self> {
        unsafe {
            let mut s = crate::Status::default();
            tirocks_sys::crocksdb_lru_cache_options_set_use_jemalloc(
                self.ptr,
                &mut opt.0,
                s.as_mut_ptr(),
            );
            if s.ok() {
                Ok(self)
            } else {
                Err(s)
            }
        }
    }

    #[inline]
    fn as_mut_ptr(&mut self) -> *mut rocksdb_LRUCacheOptions {
        self.ptr
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
    pub(crate) fn get_ptr(&self) -> *mut crocksdb_cache_t {
        self.ptr
    }

    #[inline]
    pub(crate) fn as_ptr(&self) -> *const crocksdb_cache_t {
        self.ptr
    }

    /// sets the maximum configured capacity of the cache. When the new
    /// capacity is less than the old capacity and the existing usage is
    /// greater than new capacity, the implementation will do its best job to
    /// purge the released entries from the cache in order to lower the usage
    #[inline]
    pub fn set_capacity(&self, capacity: usize) -> &Self {
        unsafe {
            tirocks_sys::crocksdb_cache_set_capacity(self.ptr, capacity);
            self
        }
    }

    simple_access! {
        crocksdb_cache
        /// Returns the memory size for the entries residing in the cache.
        (<) usage: usize

        /// Returns the maximum configured capacity of the cache.
        (<) capacity: usize
    }
}

unsafe impl Send for SysCache {}
unsafe impl Sync for SysCache {}

impl Drop for SysCache {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            tirocks_sys::crocksdb_cache_destroy(self.ptr);
        }
    }
}
