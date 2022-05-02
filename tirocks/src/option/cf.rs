// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::{
    mem::{self},
    ops::{Deref, DerefMut},
    sync::Arc,
};

use crate::{
    cache::SysCache, compaction_filter::SysCompactionFilterFactory, comparator::SysComparator,
    mem_table::SysMemTableRepFactory, slice_transform::SysSliceTransform,
    sst_partitioner::SysSstParitionerFactory, table::SysTableFactory,
    table_properties::user::SysTablePropertiesCollectorFactory, util::simple_access,
};
use tirocks_sys::{rocksdb_ColumnFamilyOptions, rocksdb_titandb_TitanCFOptions};

use super::CompressionType;

pub type CompactionStyle = tirocks_sys::rocksdb_CompactionStyle;
pub type CompactionPriority = tirocks_sys::rocksdb_CompactionPri;
pub type CompressionOptions = tirocks_sys::rocksdb_CompressionOptions;
pub type FifoCompactionOptions = tirocks_sys::rocksdb_CompactionOptionsFIFO;
pub type UniversalCompactionOptions = tirocks_sys::rocksdb_CompactionOptionsUniversal;
pub type TitanBlobRunMode = tirocks_sys::rocksdb_titandb_TitanBlobRunMode;

#[repr(transparent)]
pub struct RawCfOptions(rocksdb_ColumnFamilyOptions);

#[derive(Debug)]
#[repr(C)]
pub struct CfOptions {
    ptr: *mut RawCfOptions,
    comparator: Option<Arc<SysComparator>>,
}

impl Deref for CfOptions {
    type Target = RawCfOptions;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}

impl DerefMut for CfOptions {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.ptr }
    }
}

impl Default for CfOptions {
    #[inline]
    fn default() -> CfOptions {
        unsafe {
            let ptr = tirocks_sys::crocksdb_cfoptions_create();
            CfOptions {
                ptr: ptr as _,
                comparator: None,
            }
        }
    }
}

impl Drop for CfOptions {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            tirocks_sys::crocksdb_cfoptions_destroy(self.ptr as _);
        }
    }
}

impl RawCfOptions {
    simple_access! {
        /// The maximum number of write buffers that are built up in memory.
        /// The default and the minimum number is 2, so that when 1 write buffer
        /// is being flushed to storage, new writes can continue to the other
        /// write buffer.
        /// If max_write_buffer_number > 3, writing will be slowed down to
        /// options.delayed_write_rate if we are writing to the last write buffer
        /// allowed.
        ///
        /// Default: 2
        ///
        /// Dynamically changeable through SetOptions() API
        max_write_buffer_number: i32

        /// The minimum number of write buffers that will be merged together
        /// before writing to storage.  If set to 1, then
        /// all write buffers are flushed to L0 as individual files and this increases
        /// read amplification because a get request has to check in all of these
        /// files. Also, an in-memory merge may result in writing lesser
        /// data to storage if there are duplicate records in each of these
        /// individual write buffers.  Default: 1
        min_write_buffer_number_to_merge: i32

        /// The total maximum number of write buffers to maintain in memory including
        /// copies of buffers that have already been flushed.  Unlike
        /// max_write_buffer_number, this parameter does not affect flushing.
        /// This controls the minimum amount of write history that will be available
        /// in memory for conflict checking when Transactions are used.
        ///
        /// When using an OptimisticTransactionDB:
        /// If this value is too low, some transactions may fail at commit time due
        /// to not being able to determine whether there were any write conflicts.
        ///
        /// When using a TransactionDB:
        /// If Transaction::SetSnapshot is used, TransactionDB will read either
        /// in-memory write buffers or SST files to do write-conflict checking.
        /// Increasing this value can reduce the number of reads to SST files
        /// done for conflict detection.
        ///
        /// Setting this value to 0 will cause write buffers to be freed immediately
        /// after they are flushed.
        /// If this value is set to -1, 'max_write_buffer_number' will be used.
        ///
        /// Default:
        /// If using a TransactionDB/OptimisticTransactionDB, the default value will
        /// be set to the value of 'max_write_buffer_number' if it is not explicitly
        /// set by the user.  Otherwise, the default is 0.
        max_write_buffer_number_to_maintain: i32

        /// Allows thread-safe inplace updates. If this is true, there is no way to
        /// achieve point-in-time consistency using snapshot or iterator (assuming
        /// concurrent updates). Hence iterator and multi-get will return results
        /// which are not consistent as of any point-in-time.
        /// If inplace_callback function is not set,
        ///   Put(key, new_value) will update inplace the existing_value iff
        ///   * key exists in current memtable
        ///   * new sizeof(new_value) <= sizeof(existing_value)
        ///   * existing_value for that key is a put i.e. kTypeValue
        /// If inplace_callback function is set, check doc for inplace_callback.
        /// Default: false.
        inplace_update_support: bool

        /// Number of locks used for inplace update
        /// Default: 10000, if inplace_update_support = true, else 0.
        ///
        /// Dynamically changeable through SetOptions() API
        inplace_update_num_locks: usize

        /// if prefix_extractor is set and memtable_prefix_bloom_size_ratio is not 0,
        /// create prefix bloom for memtable with the size of
        /// write_buffer_size * memtable_prefix_bloom_size_ratio.
        /// If it is larger than 0.25, it is sanitized to 0.25.
        ///
        /// Default: 0 (disable)
        ///
        /// Dynamically changeable through SetOptions() API
        memtable_prefix_bloom_size_ratio: f64

        /// Page size for huge page for the arena used by the memtable. If <=0, it
        /// won't allocate from huge page but from malloc.
        /// Users are responsible to reserve huge pages for it to be allocated. For
        /// example:
        ///      sysctl -w vm.nr_hugepages=20
        /// See linux doc Documentation/vm/hugetlbpage.txt
        /// If there isn't enough free huge page available, it will fall back to
        /// malloc.
        ///
        /// Dynamically changeable through SetOptions() API
        memtable_huge_page_size: usize

        /// If non-nullptr, memtable will use the specified function to extract
        /// prefixes for keys, and for each prefix maintain a hint of insert location
        /// to reduce CPU usage for inserting keys with the prefix. Keys out of
        /// domain of the prefix extractor will be insert without using hints.
        ///
        /// Currently only the default skiplist based memtable implements the feature.
        /// All other memtable implementation will ignore the option. It incurs ~250
        /// additional bytes of memory overhead to store a hint for each prefix.
        /// Also concurrent writes (when allow_concurrent_memtable_write is true) will
        /// ignore the option.
        ///
        /// The option is best suited for workloads where keys will likely to insert
        /// to a location close the last inserted key with the same prefix.
        /// One example could be inserting keys of the form (prefix + timestamp),
        /// and keys of the same prefix always comes in with time order. Another
        /// example would be updating the same key over and over again, in which case
        /// the prefix can be the key itself.
        ///
        /// Default: nullptr (disable)
        memtable_insert_with_hint_prefix_extractor: &SysSliceTransform [ .get() ]

        /// Control locality of bloom filter probes to improve cache miss rate.
        /// This option only applies to memtable prefix bloom and plaintable
        /// prefix bloom. It essentially limits every bloom checking to one cache line.
        /// This optimization is turned off when set to 0, and positive number to turn
        /// it on.
        /// Default: 0
        bloom_locality: u32

        /// size of one block in arena memory allocation.
        /// If <= 0, a proper value is automatically calculated (usually 1/8 of
        /// writer_buffer_size, rounded up to a multiple of 4KB).
        ///
        /// There are two additional restriction of the specified size:
        /// (1) size should be in the range of [4096, 2 << 30] and
        /// (2) be the multiple of the CPU word (which helps with the memory
        /// alignment).
        ///
        /// We'll automatically check and adjust the size number to make sure it
        /// conforms to the restrictions.
        ///
        /// Default: 0
        ///
        /// Dynamically changeable through SetOptions() API
        arena_block_size: usize
    }

    /// Different levels can have different compression policies. There
    /// are cases where most lower levels would like to use quick compression
    /// algorithms while the higher levels (which have more data) use
    /// compression algorithms that have better compression but could
    /// be slower. This array, if non-empty, should have an entry for
    /// each level of the database; these override the value specified in
    /// the previous field 'compression'.
    ///
    /// NOTICE if level_compaction_dynamic_level_bytes=true,
    /// compression_per_level[0] still determines L0, but other elements
    /// of the array are based on base level (the level L0 files are merged
    /// to), and may not match the level users see from info log for metadata.
    /// If L0 files are merged to level-n, then, for i>0, compression_per_level[i]
    /// determines compaction type for level n+i-1.
    /// For example, if we have three 5 levels, and we determine to merge L0
    /// data to L4 (which means L1..L3 will be empty), then the new files go to
    /// L4 uses compression type compression_per_level[1].
    /// If now L0 is merged to L2. Data goes to L2 will be compressed
    /// according to compression_per_level[1], L3 using compression_per_level[2]
    /// and L4 using compression_per_level[3]. Compaction for each level can
    /// change when data grows.
    #[inline]
    pub fn set_compression_per_level(
        &mut self,
        compression_per_level: &[CompressionType],
    ) -> &mut Self {
        unsafe {
            tirocks_sys::crocksdb_options_set_compression_per_level(
                self.as_mut_ptr(),
                compression_per_level.as_ptr(),
                compression_per_level.len(),
            );
        }
        self
    }

    simple_access! {
        /// Number of levels for this database
        num_levels: i32

        /// Soft limit on number of level-0 files. We start slowing down writes at this
        /// point. A value <0 means that no writing slow down will be triggered by
        /// number of files in level-0.
        ///
        /// Default: 20
        ///
        /// Dynamically changeable through SetOptions() API
        level0_slowdown_writes_trigger: i32

        /// Maximum number of level-0 files.  We stop writes at this point.
        ///
        /// Default: 36
        ///
        /// Dynamically changeable through SetOptions() API
        level0_stop_writes_trigger: i32

        /// Target file size for compaction.
        /// target_file_size_base is per-file size for level-1.
        /// Target file size for level L can be calculated by
        /// target_file_size_base * (target_file_size_multiplier ^ (L-1))
        /// For example, if target_file_size_base is 2MB and
        /// target_file_size_multiplier is 10, then each file on level-1 will
        /// be 2MB, and each file on level 2 will be 20MB,
        /// and each file on level-3 will be 200MB.
        ///
        /// Default: 64MB.
        ///
        /// Dynamically changeable through SetOptions() API
        target_file_size_base: u64

        /// By default target_file_size_multiplier is 1, which means
        /// by default files in different levels will have similar size.
        ///
        /// Dynamically changeable through SetOptions() API
        target_file_size_multiplier: i32

        /// If true, RocksDB will pick target size of each level dynamically.
        /// We will pick a base level b >= 1. L0 will be directly merged into level b,
        /// instead of always into level 1. Level 1 to b-1 need to be empty.
        /// We try to pick b and its target size so that
        /// 1. target size is in the range of
        ///   (max_bytes_for_level_base / max_bytes_for_level_multiplier,
        ///    max_bytes_for_level_base]
        /// 2. target size of the last level (level num_levels-1) equals to extra size
        ///    of the level.
        /// At the same time max_bytes_for_level_multiplier and
        /// max_bytes_for_level_multiplier_additional are still satisfied.
        /// (When L0 is too large, we make some adjustment. See below.)
        ///
        /// With this option on, from an empty DB, we make last level the base level,
        /// which means merging L0 data into the last level, until it exceeds
        /// max_bytes_for_level_base. And then we make the second last level to be
        /// base level, to start to merge L0 data to second last level, with its
        /// target size to be 1/max_bytes_for_level_multiplier of the last level's
        /// extra size. After the data accumulates more so that we need to move the
        /// base level to the third last one, and so on.
        ///
        /// For example, assume max_bytes_for_level_multiplier=10, num_levels=6,
        /// and max_bytes_for_level_base=10MB.
        /// Target sizes of level 1 to 5 starts with:
        /// [- - - - 10MB]
        /// with base level is level. Target sizes of level 1 to 4 are not applicable
        /// because they will not be used.
        /// Until the size of Level 5 grows to more than 10MB, say 11MB, we make
        /// base target to level 4 and now the targets looks like:
        /// [- - - 1.1MB 11MB]
        /// While data are accumulated, size targets are tuned based on actual data
        /// of level 5. When level 5 has 50MB of data, the target is like:
        /// [- - - 5MB 50MB]
        /// Until level 5's actual size is more than 100MB, say 101MB. Now if we keep
        /// level 4 to be the base level, its target size needs to be 10.1MB, which
        /// doesn't satisfy the target size range. So now we make level 3 the target
        /// size and the target sizes of the levels look like:
        /// [- - 1.01MB 10.1MB 101MB]
        /// In the same way, while level 5 further grows, all levels' targets grow,
        /// like
        /// [- - 5MB 50MB 500MB]
        /// Until level 5 exceeds 1000MB and becomes 1001MB, we make level 2 the
        /// base level and make levels' target sizes like this:
        /// [- 1.001MB 10.01MB 100.1MB 1001MB]
        /// and go on...
        ///
        /// By doing it, we give max_bytes_for_level_multiplier a priority against
        /// max_bytes_for_level_base, for a more predictable LSM tree shape. It is
        /// useful to limit worse case space amplification.
        ///
        ///
        /// If the compaction from L0 is lagged behind, a special mode will be turned
        /// on to prioritize write amplification against max_bytes_for_level_multiplier
        /// or max_bytes_for_level_base. The L0 compaction is lagged behind by looking
        /// at number of L0 files and total L0 size. If number of L0 files is at least
        /// the double of level0_file_num_compaction_trigger, or the total size is
        /// at least max_bytes_for_level_base, this mode is on. The target of L1 grows
        /// to the actual data size in L0, and then determine the target for each level
        /// so that each level will have the same level multiplier.
        ///
        /// For example, when L0 size is 100MB, the size of last level is 1600MB,
        /// max_bytes_for_level_base = 80MB, and max_bytes_for_level_multiplier = 10.
        /// Since L0 size is larger than max_bytes_for_level_base, this is a L0
        /// compaction backlogged mode. So that the L1 size is determined to be 100MB.
        /// Based on max_bytes_for_level_multiplier = 10, at least 3 non-0 levels will
        /// be needed. The level multiplier will be calculated to be 4 and the three
        /// levels' target to be [100MB, 400MB, 1600MB].
        ///
        /// In this mode, The number of levels will be no more than the normal mode,
        /// and the level multiplier will be lower. The write amplification will
        /// likely to be reduced.
        ///
        ///
        /// max_bytes_for_level_multiplier_additional is ignored with this flag on.
        ///
        /// Turning this feature on or off for an existing DB can cause unexpected
        /// LSM tree structure so it's not recommended.
        ///
        /// Default: false
        level_compaction_dynamic_level_bytes: bool

        /// Default: 10.
        ///
        /// Dynamically changeable through SetOptions() API
        max_bytes_for_level_multiplier: f64
    }

    /// Different max-size multipliers for different levels.
    /// These are multiplied by max_bytes_for_level_multiplier to arrive
    /// at the max-size of each level.
    ///
    /// Default: 1
    ///
    /// Dynamically changeable through SetOptions() API
    #[inline]
    pub fn set_max_bytes_for_level_multiplier_additional(&mut self, level: &[i32]) -> &mut Self {
        unsafe {
            tirocks_sys::crocksdb_options_set_max_bytes_for_level_multiplier_additional(
                self.as_mut_ptr(),
                level.as_ptr(),
                level.len(),
            );
        }
        self
    }

    simple_access! {
        /// We try to limit number of bytes in one compaction to be lower than this
        /// threshold. But it's not guaranteed.
        /// Value 0 will be sanitized.
        ///
        /// Default: target_file_size_base * 25
        ///
        /// Dynamically changeable through SetOptions() API
        max_compaction_bytes: u64

        /// All writes will be slowed down to at least delayed_write_rate if estimated
        /// bytes needed to be compaction exceed this threshold.
        ///
        /// Default: 64GB
        ///
        /// Dynamically changeable through SetOptions() API
        soft_pending_compaction_bytes_limit: u64

        /// All writes are stopped if estimated bytes needed to be compaction exceed
        /// this threshold.
        ///
        /// Default: 256GB
        ///
        /// Dynamically changeable through SetOptions() API
        hard_pending_compaction_bytes_limit: u64

        /// The compaction style. Default: kCompactionStyleLevel
        compaction_style: CompactionStyle

        /// If level compaction_style = kCompactionStyleLevel, for each level,
        /// which files are prioritized to be picked to compact.
        /// Default: kMinOverlappingRatio
        compaction_priority: CompactionPriority
    }

    /// The options needed to support Universal Style compactions
    ///
    /// Dynamically changeable through SetOptions() API
    /// Dynamic change example:
    /// SetOptions("compaction_options_universal", "{size_ratio=2;}")
    #[inline]
    pub fn compaction_options_universal_mut(&mut self) -> &mut UniversalCompactionOptions {
        unsafe {
            &mut *tirocks_sys::crocksdb_options_get_universal_compaction_options(self.as_mut_ptr())
        }
    }

    /// The options for FIFO compaction style
    ///
    /// Dynamically changeable through SetOptions() API
    /// Dynamic change example:
    /// SetOptions("compaction_options_fifo", "{max_table_files_size=100;}")
    #[inline]
    pub fn compaction_options_fifo_mut(&mut self) -> &mut FifoCompactionOptions {
        unsafe {
            &mut *tirocks_sys::crocksdb_options_get_fifo_compaction_options(self.as_mut_ptr())
        }
    }

    simple_access! {
        /// An iteration->Next() sequentially skips over keys with the same
        /// user-key unless this option is set. This number specifies the number
        /// of keys (with the same userkey) that will be sequentially
        /// skipped before a reseek is issued.
        ///
        /// Default: 8
        ///
        /// Dynamically changeable through SetOptions() API
        max_sequential_skip_in_iterations: u64

        /// This is a factory that provides MemTableRep objects.
        /// Default: a factory that provides a skip-list-based implementation of
        /// MemTableRep.
        memtable_factory: &SysMemTableRepFactory [ .get() ]

        /// Block-based table related options are moved to BlockBasedTableOptions.
        /// Related options that were originally here but now moved include:
        ///   no_block_cache
        ///   block_cache
        ///   block_cache_compressed
        ///   block_size
        ///   block_size_deviation
        ///   block_restart_interval
        ///   filter_policy
        ///   whole_key_filtering
        /// If you'd like to customize some of these options, you will need to
        /// use NewBlockBasedTableFactory() to construct a new table factory.
        ///
        /// This option allows user to collect their own interested statistics of
        /// the tables.
        /// Default: empty vector -- no user-defined statistics collection will be
        /// performed.
        <add> table_properties_collector_factory: &SysTablePropertiesCollectorFactory [ .get() ]

        /// Maximum number of successive merge operations on a key in the memtable.
        ///
        /// When a merge operation is added to the memtable and the maximum number of
        /// successive merges is reached, the value of the key will be calculated and
        /// inserted into the memtable instead of the merge operation. This will
        /// ensure that there are never more than max_successive_merges merge
        /// operations in the memtable.
        ///
        /// Default: 0 (disabled)
        ///
        /// Dynamically changeable through SetOptions() API
        max_successive_merges: usize

        /// This flag specifies that the implementation should optimize the filters
        /// mainly for cases where keys are found rather than also optimize for keys
        /// missed. This would be used in cases where the application knows that
        /// there are very few misses or the performance in the case of misses is not
        /// important.
        ///
        /// For now, this flag allows us to not store filters for the last level i.e
        /// the largest level which contains data of the LSM store. For keys which
        /// are hits, the filters in this level are not useful because we will search
        /// for the data anyway. NOTE: the filters in other levels are still useful
        /// even for key hit because they tell us whether to look in that level or go
        /// to the higher level.
        ///
        /// Default: false
        optimize_filters_for_hits: bool

        /// In debug mode, RocksDB run consistency checks on the LSM every time the LSM
        /// change (Flush, Compaction, AddFile). These checks are disabled in release
        /// mode, use this option to enable them in release mode as well.
        /// Default: false
        force_consistency_checks: bool

        /// Measure IO stats in compactions and flushes, if true.
        ///
        /// Default: false
        ///
        /// Dynamically changeable through SetOptions() API
        report_bg_io_stats: bool
    }

    /// Use this if you don't need to keep the data sorted, i.e. you'll never use
    /// an iterator, only Put() and Get() API calls
    ///
    /// size is wiped up to MiB.
    #[inline]
    pub fn optimize_for_point_lookup(&mut self, block_cache_size: u64) -> &mut Self {
        unsafe {
            tirocks_sys::crocksdb_options_optimize_for_point_lookup(
                self.as_mut_ptr(),
                block_cache_size / 1024 / 1024,
            );
        }
        self
    }

    /// Default values for some parameters in ColumnFamilyOptions are not optimized for heavy
    /// workloads and big datasets, which means you might observe write stalls under some
    /// conditions. As a starting point for tuning RocksDB options, use
    /// `optimize_level_style_compaction` to optimize level style compaction.
    ///
    /// You can learn more about the different styles here:
    /// https://github.com/facebook/rocksdb/wiki/Rocksdb-Architecture-Guide
    /// Make sure to also call `increase_parallelism`, which will provide the
    /// biggest performance gains.
    /// Note: we might use more memory than memtable_memory_budget during high write rate period
    #[inline]
    pub fn optimize_level_style_compaction(&mut self, memtable_memory_budget: u64) -> &mut Self {
        unsafe {
            tirocks_sys::crocksdb_options_optimize_level_style_compaction(
                self.as_mut_ptr(),
                memtable_memory_budget,
            );
        }
        self
    }

    /// Default values for some parameters in ColumnFamilyOptions are not optimized for heavy
    /// workloads and big datasets, which means you might observe write stalls under some
    /// conditions. As a starting point for tuning RocksDB options, use
    /// `optimize_universal_style_compaction` to optimize universal style compaction
    /// Universal style compaction is focused on reducing Write Amplification
    /// Factor for big data sets, but increases Space Amplification. You can learn
    /// more about the different styles here:
    /// https://github.com/facebook/rocksdb/wiki/Rocksdb-Architecture-Guide
    /// Make sure to also call `increase_parallelism`, which will provide the
    /// biggest performance gains.
    /// Note: we might use more memory than memtable_memory_budget during high write rate period
    #[inline]
    pub fn optimize_universal_style_compaction(
        &mut self,
        memtable_memory_budget: u64,
    ) -> &mut Self {
        unsafe {
            tirocks_sys::crocksdb_options_optimize_universal_style_compaction(
                self.as_mut_ptr(),
                memtable_memory_budget,
            );
        }
        self
    }

    /// Comparator used to define the order of keys in the table.
    /// Default: a comparator that uses lexicographic byte-wise ordering
    ///
    /// REQUIRES: The client must ensure that the comparator supplied
    /// here has the same name and orders keys *exactly* the same as the
    /// comparator provided to previous open calls on the same DB.
    ///
    /// # Safety
    /// It's undefinied behavior if `RawCfOptions` outlives `SysComparator`.
    #[inline]
    pub unsafe fn set_comparator(&mut self, c: &SysComparator) -> &mut Self {
        tirocks_sys::crocksdb_options_set_comparator(self.as_mut_ptr(), c.get());
        self
    }

    // REQUIRES: The client must provide a merge operator if Merge operation
    // needs to be accessed. Calling Merge on a DB without a merge operator
    // would result in Status::NotSupported. The client must ensure that the
    // merge operator supplied here has the same name and *exactly* the same
    // semantics as the merge operator provided to previous open calls on
    // the same DB. The only exception is reserved for upgrade, where a DB
    // previously without a merge operator is introduced to Merge operation
    // for the first time. It's necessary to specify a merge operator when
    // opening the DB in this case.
    // Default: nullptr
    // TODO: support merge
    //merge_operator: &SysMergeOperator [ .get() ]

    simple_access! {
        /// This is a factory that provides `CompactionFilter` objects which allow
        /// an application to modify/delete a key-value during table file creation.
        ///
        /// Unlike the `compaction_filter` option, which is used when compaction
        /// creates a table file, this factory allows using a `CompactionFilter` when a
        /// table file is created for various reasons. The factory can decide what
        /// `TableFileCreationReason`s use a `CompactionFilter`. For compatibility, by
        /// default the decision is to use a `CompactionFilter` for
        /// `TableFileCreationReason::kCompaction` only.
        ///
        /// Each thread of work involving creating table files will create a new
        /// `CompactionFilter` when it will be used according to the above
        /// `TableFileCreationReason`-based decision. This allows the application to
        /// know about the different ongoing threads of work and makes it unnecessary
        /// for `CompactionFilter` to provide thread-safety.
        compaction_filter_factory: &SysCompactionFilterFactory [ .get() ]

        /// Amount of data to build up in memory (backed by an unsorted log
        /// on disk) before converting to a sorted on-disk file.
        ///
        /// Larger values increase performance, especially during bulk loads.
        /// Up to max_write_buffer_number write buffers may be held in memory
        /// at the same time,
        /// so you may wish to adjust this parameter to control memory usage.
        /// Also, a larger write buffer will result in a longer recovery time
        /// the next time the database is opened.
        ///
        /// Note that write_buffer_size is enforced per column family.
        /// See db_write_buffer_size for sharing memory across column families.
        ///
        /// Default: 64MB
        ///
        /// Dynamically changeable through SetOptions() API
        write_buffer_size: usize

        /// Compress blocks using the specified compression algorithm.
        ///
        /// Default: kSnappyCompression, if it's supported. If snappy is not linked
        /// with the library, the default is kNoCompression.
        ///
        /// Typical speeds of kSnappyCompression on an Intel(R) Core(TM)2 2.4GHz:
        ///    ~200-500MB/s compression
        ///    ~400-800MB/s decompression
        ///
        /// Note that these speeds are significantly faster than most
        /// persistent storage speeds, and therefore it is typically never
        /// worth switching to kNoCompression.  Even if the input data is
        /// incompressible, the kSnappyCompression implementation will
        /// efficiently detect that and will switch to uncompressed mode.
        ///
        /// If you do not set `compression_opts.level`, or set it to
        /// `CompressionOptions::kDefaultCompressionLevel`, we will attempt to pick the
        /// default corresponding to `compression` as follows:
        ///
        /// - kZSTD: 3
        /// - kZlibCompression: Z_DEFAULT_COMPRESSION (currently -1)
        /// - kLZ4HCCompression: 0
        /// - For all others, we do not specify a compression level
        ///
        /// Dynamically changeable through SetOptions() API
        compression: CompressionType

        /// Compression algorithm that will be used for the bottommost level that
        /// contain files.
        ///
        /// Default: kDisableCompressionOption (Disabled)
        bottommost_compression: CompressionType
    }

    /// different options for compression algorithms used by bottommost_compression
    /// if it is enabled. To enable it, please see the definition of
    /// CompressionOptions.
    #[inline]
    pub fn bottommost_compression_opts_mut(&mut self) -> &mut CompressionOptions {
        unsafe {
            &mut *tirocks_sys::crocksdb_options_get_bottommost_compression_options(
                self.as_mut_ptr(),
            )
        }
    }

    /// Different options for compression algorithms
    #[inline]
    pub fn compression_opts_mut(&mut self) -> &mut CompressionOptions {
        unsafe { &mut *tirocks_sys::crocksdb_options_get_compression_options(self.as_mut_ptr()) }
    }

    simple_access! {
        /// Number of files to trigger level-0 compaction. A value <0 means that
        /// level-0 compaction will not be triggered by number of files at all.
        ///
        /// Default: 4
        ///
        /// Dynamically changeable through SetOptions() API
        level0_file_num_compaction_trigger: i32

        /// If non-nullptr, use the specified function to determine the
        /// prefixes for keys.  These prefixes will be placed in the filter.
        /// Depending on the workload, this can reduce the number of read-IOP
        /// cost for scans when a prefix is passed via ReadOptions to
        /// db.NewIterator().  For prefix filtering to work properly,
        /// "prefix_extractor" and "comparator" must be such that the following
        /// properties hold:
        ///
        /// 1) key.starts_with(prefix(key))
        /// 2) Compare(prefix(key), key) <= 0.
        /// 3) If Compare(k1, k2) <= 0, then Compare(prefix(k1), prefix(k2)) <= 0
        /// 4) prefix(prefix(key)) == prefix(key)
        prefix_extractor: &SysSliceTransform [ .get() ]

        /// Control maximum total data size for a level.
        /// max_bytes_for_level_base is the max total for level-1.
        /// Maximum number of bytes for level L can be calculated as
        /// (max_bytes_for_level_base) * (max_bytes_for_level_multiplier ^ (L-1))
        /// For example, if max_bytes_for_level_base is 200MB, and if
        /// max_bytes_for_level_multiplier is 10, total data size for level-1
        /// will be 200MB, total file size for level-2 will be 2GB,
        /// and total file size for level-3 will be 20GB.
        ///
        /// Default: 256MB.
        ///
        /// Dynamically changeable through SetOptions() API
        max_bytes_for_level_base: u64

        /// Disable automatic compactions. Manual compactions can still
        /// be issued on this column family
        ///
        /// Dynamically changeable through SetOptions() API
        disable_auto_compactions: bool

        /// Disable write stall mechanism.
        ///
        /// Dynamically changeable through SetOptions() API
        disable_write_stall: bool

        /// This is a factory that provides TableFactory objects.
        /// Default: a block-based table factory that provides a default
        /// implementation of TableBuilder and TableReader with default
        /// BlockBasedTableOptions.
        table_factory: &SysTableFactory [ .get() ]

        /// If non-nullptr, use the specified factory for a function to determine the
        /// partitioning of sst files. This helps compaction to split the files
        /// on interesting boundaries (key prefixes) to make propagation of sst
        /// files less write amplifying (covering the whole key space).
        /// THE FEATURE IS STILL EXPERIMENTAL
        sst_partitioner_factory: &SysSstParitionerFactory [ .get() ]
    }

    pub(crate) unsafe fn from_ptr<'a>(ptr: *const rocksdb_ColumnFamilyOptions) -> &'a Self {
        &*(ptr as *const RawCfOptions)
    }

    pub(crate) unsafe fn from_ptr_mut<'a>(ptr: *mut rocksdb_ColumnFamilyOptions) -> &'a mut Self {
        &mut *(ptr as *mut RawCfOptions)
    }

    pub(crate) fn as_mut_ptr(&mut self) -> *mut rocksdb_ColumnFamilyOptions {
        self as *mut _ as _
    }
}

impl CfOptions {
    /// Same as `RawCfOptions::set_comparator` but manages the lifetime of `c`.
    #[inline]
    pub fn set_comparator(&mut self, c: Arc<SysComparator>) -> &mut Self {
        unsafe {
            (**self).set_comparator(&c);
        }
        self.comparator = Some(c);
        self
    }

    #[inline]
    pub fn comparator(&self) -> Option<&Arc<SysComparator>> {
        self.comparator.as_ref()
    }

    #[inline]
    pub(crate) fn as_ptr(&self) -> *const rocksdb_ColumnFamilyOptions {
        self.ptr as _
    }
}

#[repr(transparent)]
pub struct RawTitanCfOptions(rocksdb_titandb_TitanCFOptions);

impl Deref for RawTitanCfOptions {
    type Target = RawCfOptions;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe {
            // TitanCFOptions inherits CFOptions, so the two structs are identical.
            mem::transmute(self)
        }
    }
}

impl DerefMut for RawTitanCfOptions {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            // TitanCFOptions inherits CFOptions, so the two structs are identical.
            mem::transmute(self)
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct TitanCfOptions {
    ptr: *mut RawTitanCfOptions,
    comparator: Option<Arc<SysComparator>>,
}

impl Deref for TitanCfOptions {
    type Target = RawTitanCfOptions;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}

impl DerefMut for TitanCfOptions {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.ptr }
    }
}

impl RawTitanCfOptions {
    simple_access! {
        ctitandb_options

        /// The smallest value to store in blob files. Value smaller than
        /// this threshold will be inlined in base DB.
        ///
        /// Default: 4096
        min_blob_size: u64

        /// The compression algorithm used to compress data in blob files.
        ///
        /// Default: kNoCompression
        blob_file_compression: CompressionType
    }

    /// The compression options. The `blob_file_compression.enabled` option is
    /// ignored, we only use `blob_file_compression` above to determine wether the
    /// blob file is compressed. We use this options mainly to configure the
    /// compression dictionary.
    pub fn blob_file_compression_options_mut(&mut self) -> &mut CompressionOptions {
        unsafe {
            &mut *tirocks_sys::ctitandb_options_get_blob_file_compression_options(self.as_mut_ptr())
        }
    }

    simple_access! {
        ctitandb_options

        /// If non-NULL use the specified cache for blob records.
        ///
        /// Default: nullptr
        blob_cache: &SysCache [ .get() ]

        /// Max batch size for GC.
        ///
        /// Default: 1GB
        max_gc_batch_size: u64

        /// Min batch size for GC.
        ///
        /// Default: 512MB
        min_gc_batch_size: u64

        /// The ratio of how much discardable size of a blob file can be GC.
        ///
        /// Default: 0.5
        blob_file_discardable_ratio: f64

        /// The ratio of how much size of a blob file need to be sample before GC.
        ///
        /// Default: 0.1
        sample_file_size_ratio: f64

        /// The blob file size less than this option will be mark GC.
        ///
        /// Default: 8MB
        merge_small_file_threshold: u64

        /// The mode used to process blob file.
        ///
        /// Default: kNormal
        blob_run_mode: TitanBlobRunMode

        /// If set true, values in blob file will be merged to a new blob file while
        /// their corresponding keys are compacted to last two level in LSM-Tree.
        ///
        /// With this feature enabled, Titan could get better scan performance, and
        /// better write performance during GC, but will suffer around 1.1 space
        /// amplification and 3 more write amplification if no GC needed (eg. uniformly
        /// distributed keys) under default rocksdb setting.
        ///
        /// Requirement: level_compaction_dynamic_level_base = true
        /// Default: false
        level_merge: bool

        /// With level merge enabled, we expect there are no more than 10 sorted runs
        /// of blob files in both of last two levels. But since last level blob files
        /// won't be merged again, sorted runs in last level will increase infinitely.
        ///
        /// With this feature enabled, Titan will check sorted runs of compaction range
        /// after each last level compaction and mark related blob files if there are
        /// too many. These marked blob files will be merged to a new sorted run in
        /// next compaction.
        ///
        /// Default: false
        range_merge: bool

        /// Max sorted runs to trigger range merge. Decrease this value will increase
        /// write amplification but get better short range scan performance.
        ///
        /// Default: 20
        max_sorted_runs: i32

        /// If set true, Titan will rewrite valid blob index from GC output as merge
        /// operands back to data store.
        ///
        /// With this feature enabled, Titan background GC won't block online write,
        /// trade-off being read performance slightly reduced compared to normal
        /// rewrite mode.
        ///
        /// Default: false
        gc_merge_rewrite: bool
    }

    pub(crate) unsafe fn from_ptr<'a>(ptr: *const rocksdb_titandb_TitanCFOptions) -> &'a Self {
        &*(ptr as *const RawTitanCfOptions)
    }

    pub(crate) unsafe fn from_ptr_mut<'a>(
        ptr: *mut rocksdb_titandb_TitanCFOptions,
    ) -> &'a mut Self {
        &mut *(ptr as *mut RawTitanCfOptions)
    }

    pub(crate) fn as_mut_ptr(&mut self) -> *mut rocksdb_titandb_TitanCFOptions {
        self as *mut _ as _
    }
}

impl TitanCfOptions {
    /// Same as `CfOptions::set_comparator`.
    #[inline]
    pub fn set_comparator(&mut self, c: Arc<SysComparator>) -> &mut Self {
        unsafe {
            (**self).set_comparator(&c);
        }
        self.comparator = Some(c);
        self
    }

    #[inline]
    pub fn comparator(&self) -> Option<&Arc<SysComparator>> {
        self.comparator.as_ref()
    }

    #[inline]
    pub(crate) fn as_ptr(&self) -> *const rocksdb_titandb_TitanCFOptions {
        self.ptr as _
    }
}

impl Default for TitanCfOptions {
    #[inline]
    fn default() -> Self {
        unsafe {
            let ptr = tirocks_sys::ctitandb_cfoptions_create();
            Self {
                ptr: ptr as _,
                comparator: None,
            }
        }
    }
}

impl Drop for TitanCfOptions {
    #[inline]
    fn drop(&mut self) {
        unsafe { tirocks_sys::ctitandb_cfoptions_destroy(self.ptr as _) }
    }
}
