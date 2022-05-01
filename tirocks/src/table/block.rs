// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

pub mod filter_policy;

use tirocks_sys::rocksdb_BlockBasedTableOptions;

use crate::cache::SysCache;
use crate::util::simple_access;

use self::filter_policy::SysFilterPolicy;

pub type IndexType = tirocks_sys::rocksdb_BlockBasedTableOptions_IndexType;

pub struct BlockBasedTableOptions {
    ptr: *mut rocksdb_BlockBasedTableOptions,
}

impl BlockBasedTableOptions {
    simple_access! {
        crocksdb_block_based_options

        /// Indicating if we'd put index/filter blocks to the block cache.
        /// If not specified, each "table reader" object will pre-load index/filter
        /// block during table initialization.
        cache_index_and_filter_blocks: bool

        /// If cache_index_and_filter_blocks is enabled, cache index and filter
        /// blocks with high priority. If set to true, depending on implementation of
        /// block cache, index and filter blocks may be less likely to be evicted
        /// than data blocks.
        cache_index_and_filter_blocks_with_high_priority: bool

        /// if cache_index_and_filter_blocks is true and the below is true, then
        /// filter and index blocks are stored in the cache, but a reference is
        /// held in the "table reader" object so the blocks are pinned and only
        /// evicted from cache when the table reader is freed.
        pin_l0_filter_and_index_blocks_in_cache: bool

        /// If cache_index_and_filter_blocks is true and the below is true, then
        /// the top-level index of partitioned filter and index blocks are stored in
        /// the cache, but a reference is held in the "table reader" object so the
        /// blocks are pinned and only evicted from cache when the table reader is
        /// freed. This is not limited to l0 in LSM tree.
        pin_top_level_index_and_filter: bool

        /// The index type that will be used for this table.
        index_type: IndexType

        /// This option is now deprecated. No matter what value it is set to,
        /// it will behave as if hash_index_allow_collision=true.
        hash_index_allow_collision: bool

        /// Disable block cache. If this is set to true,
        /// then no block cache should be used, and the block_cache should
        /// point to a nullptr object.
        no_block_cache: bool

        /// If non-NULL use the specified cache for blocks.
        /// If NULL, rocksdb will automatically create and use an 8MB internal cache.
        block_cache: &SysCache [ .get() ]

        /// If non-NULL use the specified cache for compressed blocks.
        /// If NULL, rocksdb will not use a compressed block cache.
        /// Note: though it looks similar to `block_cache`, RocksDB doesn't put the
        ///       same type of object there.
        block_cache_compressed: &SysCache [ .get() ]

        /// Approximate size of user data packed per block.  Note that the
        /// block size specified here corresponds to uncompressed data.  The
        /// actual size of the unit read from disk may be smaller if
        /// compression is enabled.  This parameter can be changed dynamically.
        block_size: usize

        /// This is used to close a block before it reaches the configured
        /// 'block_size'. If the percentage of free space in the current block is less
        /// than this specified number and adding a new record to the block will
        /// exceed the configured block size, then this block will be closed and the
        /// new record will be written to the next block.
        block_size_deviation: i32

        /// Number of keys between restart points for delta encoding of keys.
        /// This parameter can be changed dynamically.  Most clients should
        /// leave this parameter alone.  The minimum value allowed is 1.  Any smaller
        /// value will be silently overwritten with 1.
        block_restart_interval: i32

        /// Block size for partitioned metadata. Currently applied to indexes when
        /// kTwoLevelIndexSearch is used and to filters when partition_filters is used.
        /// Note: Since in the current implementation the filters and index partitions
        /// are aligned, an index/filter block is created when either index or filter
        /// block size reaches the specified limit.
        /// Note: this limit is currently applied to only index blocks; a filter
        /// partition is cut right after an index block is cut
        /// TODO(myabandeh): remove the note above when filter partitions are cut
        /// separately
        metadata_block_size: u64

        /// Note: currently this option requires kTwoLevelIndexSearch to be set as
        /// well.
        ///
        /// Use partitioned full filters for each SST file. This option is
        /// incompatible with block-based filters.
        partition_filters: bool

        /// If non-nullptr, use the specified filter policy to reduce disk reads.
        /// Many applications will benefit from passing the result of
        /// new_bloom_filter_policy() here.
        filter_policy: &SysFilterPolicy [ .get() ]

        /// If true, place whole keys in the filter (not just prefixes).
        /// This must generally be true for gets to be efficient.
        whole_key_filtering: bool

        /// If used, For every data block we load into memory, we will create a bitmap
        /// of size ((block_size / `read_amp_bytes_per_bit`) / 8) bytes. This bitmap
        /// will be used to figure out the percentage we actually read of the blocks.
        ///
        /// When this feature is used Tickers::READ_AMP_ESTIMATE_USEFUL_BYTES and
        /// Tickers::READ_AMP_TOTAL_READ_BYTES can be used to calculate the
        /// read amplification using this formula
        /// (READ_AMP_TOTAL_READ_BYTES / READ_AMP_ESTIMATE_USEFUL_BYTES)
        ///
        /// value  =>  memory usage (percentage of loaded blocks memory)
        /// 1      =>  12.50 %
        /// 2      =>  06.25 %
        /// 4      =>  03.12 %
        /// 8      =>  01.56 %
        /// 16     =>  00.78 %
        ///
        /// Note: This number must be a power of 2, if not it will be sanitized
        /// to be the next lowest power of 2, for example a value of 7 will be
        /// treated as 4, a value of 19 will be treated as 16.
        ///
        /// Default: 0 (disabled)
        read_amp_bytes_per_bit: u32

        /// We currently have five versions:
        /// 0 -- This version is currently written out by all RocksDB's versions by
        /// default.  Can be read by really old RocksDB's. Doesn't support changing
        /// checksum (default is CRC32).
        /// 1 -- Can be read by RocksDB's versions since 3.0. Supports non-default
        /// checksum, like xxHash. It is written by RocksDB when
        /// BlockBasedTableOptions::checksum is something other than kCRC32c. (version
        /// 0 is silently upconverted)
        /// 2 -- Can be read by RocksDB's versions since 3.10. Changes the way we
        /// encode compressed blocks with LZ4, BZip2 and Zlib compression. If you
        /// don't plan to run RocksDB before version 3.10, you should probably use
        /// this.
        /// 3 -- Can be read by RocksDB's versions since 5.15. Changes the way we
        /// encode the keys in index blocks. If you don't plan to run RocksDB before
        /// version 5.15, you should probably use this.
        /// This option only affects newly written tables. When reading existing
        /// tables, the information about version is read from the footer.
        /// 4 -- Can be read by RocksDB's versions since 5.16. Changes the way we
        /// encode the values in index blocks. If you don't plan to run RocksDB before
        /// version 5.16 and you are using index_block_restart_interval > 1, you should
        /// probably use this as it would reduce the index size.
        /// This option only affects newly written tables. When reading existing
        /// tables, the information about version is read from the footer.
        format_version: u32
    }

    #[inline]
    pub(crate) fn get(&self) -> *mut rocksdb_BlockBasedTableOptions {
        self.ptr
    }
}

impl Drop for BlockBasedTableOptions {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            tirocks_sys::crocksdb_block_based_options_destroy(self.ptr);
        }
    }
}
