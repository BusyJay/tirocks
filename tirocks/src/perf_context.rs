// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::ops::{BitOr, BitOrAssign};

use tirocks_sys::{
    rocksdb_IOStatsContext, rocksdb_PerfContext, rocksdb_PerfFlag, rocksdb_PerfFlags,
    rocksdb_PerfLevel,
};

use crate::util::simple_access;

pub type PerfLevel = rocksdb_PerfLevel;
pub type PerfFlag = rocksdb_PerfFlag;

/// Get current perf stats level for current thread
pub fn get_perf_level() -> PerfLevel {
    unsafe { tirocks_sys::crocksdb_get_perf_level() }
}

/// Set the perf stats level for current thread
pub fn set_perf_level(level: PerfLevel) {
    unsafe { tirocks_sys::crocksdb_set_perf_level(level) }
}

#[derive(Debug)]
pub struct PerfFlags {
    ptr: *mut rocksdb_PerfFlags,
}

unsafe impl Sync for PerfFlags {}

unsafe impl Send for PerfFlags {}

impl Default for PerfFlags {
    #[inline]
    fn default() -> Self {
        unsafe {
            let ptr = tirocks_sys::crocksdb_create_perf_flags();
            PerfFlags { ptr }
        }
    }
}

impl Drop for PerfFlags {
    fn drop(&mut self) {
        unsafe {
            tirocks_sys::crocksdb_destroy_perf_flags(self.ptr);
        }
    }
}

impl BitOrAssign<PerfFlag> for PerfFlags {
    #[inline]
    fn bitor_assign(&mut self, rhs: PerfFlag) {
        unsafe {
            tirocks_sys::crocksdb_perf_flags_set(self.ptr, rhs);
        }
    }
}

impl BitOr<PerfFlag> for PerfFlags {
    type Output = PerfFlags;

    #[inline]
    fn bitor(mut self, rhs: PerfFlag) -> PerfFlags {
        self |= rhs;
        self
    }
}

impl From<PerfFlag> for PerfFlags {
    #[inline]
    fn from(flag: PerfFlag) -> Self {
        Self::default() | flag
    }
}

impl<'a> From<&'a [PerfFlag]> for PerfFlags {
    #[inline]
    fn from(fs: &'a [PerfFlag]) -> Self {
        fs.iter().fold(Self::default(), |l, r| l | *r)
    }
}

pub fn set_perf_flags(flags: &PerfFlags) {
    unsafe {
        tirocks_sys::crocksdb_set_perf_flags(flags.ptr);
    }
}

/// A context provides performance insights.
///
/// A perf context is thread local, it can't be sent across threads.
/// ```compile_fail
/// use tirocks::perf_context::PerfContext;
/// let p = PerfContext::get();
/// std::thread::spawn(move || p.reset());
/// ```
pub struct PerfContext {
    ptr: *mut rocksdb_PerfContext,
}

impl PerfContext {
    /// reset all performance counters to zero
    #[inline]
    pub fn reset(&mut self) {
        unsafe {
            tirocks_sys::crocksdb_perf_context_reset(self.ptr);
        }
    }

    simple_access! {
        crocksdb_perf_context

        /// Total number of user key comparisons
        (<) user_key_comparison_count: u64

        /// Total number of block cache hits
        (<) block_cache_hit_count: u64

        /// Total number of block reads (with IO)
        (<) block_read_count: u64

        /// Total number of bytes from block reads
        (<) block_read_byte: u64

        /// Total nanos spent on block reads
        (<) block_read_time: u64

        /// Total number of index block hits
        (<) block_cache_index_hit_count: u64

        /// Total number of index block reads
        (<) index_block_read_count: u64

        /// Total number of filter block hits
        (<) block_cache_filter_hit_count: u64

        /// Total number of filter block reads
        (<) filter_block_read_count: u64

        /// Total number of compression dictionary block reads
        (<) compression_dict_block_read_count: u64

        /// Total nanos spent on block checksum
        (<) block_checksum_time: u64

        /// Total nanos spent on block decompression
        (<) block_decompress_time: u64

        /// Bytes for vals returned by Get
        (<) get_read_bytes: u64

        /// Bytes for vals returned by MultiGet
        (<) multiget_read_bytes: u64

        /// Bytes for keys/vals decoded by iterator
        (<) iter_read_bytes: u64

        /// Total number of internal keys skipped over during iteration.
        /// There are several reasons for it:
        /// 1. when calling Next(), the iterator is in the position of the previous
        ///    key, so that we'll need to skip it. It means this counter will always
        ///    be incremented in Next().
        /// 2. when calling Next(), we need to skip internal entries for the previous
        ///    keys that are overwritten.
        /// 3. when calling Next(), Seek() or SeekToFirst(), after previous key
        ///    before calling Next(), the seek key in Seek() or the beginning for
        ///    SeekToFirst(), there may be one or more deleted keys before the next
        ///    valid key that the operation should place the iterator to. We need
        ///    to skip both of the tombstone and updates hidden by the tombstones. The
        ///    tombstones are not included in this counter, while previous updates
        ///    hidden by the tombstones will be included here.
        /// 4. symmetric cases for Prev() and SeekToLast()
        /// internal_recent_skipped_count is not included in this counter.
        (<) internal_key_skipped_count: u64

        /// Total number of deletes and single deletes skipped over during iteration
        /// When calling Next(), Seek() or SeekToFirst(), after previous position
        /// before calling Next(), the seek key in Seek() or the beginning for
        /// SeekToFirst(), there may be one or more deleted keys before the next valid
        /// key. Every deleted key is counted once. We don't recount here if there are
        /// still older updates invalidated by the tombstones.
        (<) internal_delete_skipped_count: u64

        /// How many times iterators skipped over internal keys that are more recent
        /// than the snapshot that iterator is using.
        (<) internal_recent_skipped_count: u64

        /// How many values were fed into merge operator by iterators.
        (<) internal_merge_count: u64

        /// Total nanos spent on getting snapshot
        (<) get_snapshot_time: u64

        /// Total nanos spent on querying memtables
        (<) get_from_memtable_time: u64

        /// Number of mem tables queried
        (<) get_from_memtable_count: u64

        /// Total nanos spent after Get() finds a key
        (<) get_post_process_time: u64

        /// Total nanos reading from output files
        (<) get_from_output_files_time: u64

        /// Total nanos spent on seeking memtable
        (<) seek_on_memtable_time: u64

        /// Number of seeks issued on memtable
        /// (including SeekForPrev but not SeekToFirst and SeekToLast)
        (<) seek_on_memtable_count: u64

        /// Number of Next()s issued on memtable
        (<) next_on_memtable_count: u64

        /// Number of Prev()s issued on memtable
        (<) prev_on_memtable_count: u64

        /// Total nanos spent on seeking child iters
        (<) seek_child_seek_time: u64

        /// Number of seek issued in child iterators
        (<) seek_child_seek_count: u64

        /// Total nanos spent on the merge min heap
        (<) seek_min_heap_time: u64

        /// Total nanos spent on the merge max heap
        (<) seek_max_heap_time: u64

        /// Total nanos spent on seeking the internal entries
        (<) seek_internal_seek_time: u64

        /// Total nanos spent on iterating internal entries to find the next user entry
        (<) find_next_user_entry_time: u64

        /// This group of stats provide a breakdown of time spent by Write().
        /// May be inaccurate when 2PC, two_write_queues or enable_pipelined_write
        /// are enabled.
        ///
        /// total nanos spent on writing to WAL
        (<) write_wal_time: u64

        /// total nanos spent on writing to mem tables
        (<) write_memtable_time: u64

        /// total nanos spent on delaying or throttling write
        (<) write_delay_time: u64

        /// total nanos spent on switching memtable/wal and scheduling
        /// flushes/compactions.
        (<) write_scheduling_flushes_compactions_time: u64

        /// total nanos spent on writing a record, excluding the above four things
        (<) write_pre_and_post_process_time: u64

        /// time spent waiting for other threads of the batch group
        (<) write_thread_wait_nanos: u64

        /// time spent on acquiring DB mutex.
        (<) db_mutex_lock_nanos: u64

        /// Time spent on waiting with a condition variable created with DB mutex.
        (<) db_condition_wait_nanos: u64

        /// Time spent on merge operator.
        (<) merge_operator_time_nanos: u64

        /// Time spent on reading index block from block cache or SST file
        (<) read_index_block_nanos: u64

        /// Time spent on reading filter block from block cache or SST file
        (<) read_filter_block_nanos: u64

        /// Time spent on creating data block iterator
        (<) new_table_block_iter_nanos: u64

        /// Time spent on creating a iterator of an SST file.
        (<) new_table_iterator_nanos: u64

        /// Time spent on seeking a key in data/index blocks
        (<) block_seek_nanos: u64

        /// Time spent on finding or creating a table reader
        (<) find_table_nanos: u64

        /// Total number of mem table bloom hits
        (<) bloom_memtable_hit_count: u64

        /// Total number of mem table bloom misses
        (<) bloom_memtable_miss_count: u64

        /// Total number of SST table bloom hits
        (<) bloom_sst_hit_count: u64

        /// Total number of SST table bloom misses
        (<) bloom_sst_miss_count: u64

        /// Time spent waiting on key locks in transaction lock manager.
        (<) key_lock_wait_time: u64

        /// Number of times acquiring a lock was blocked by another transaction.
        (<) key_lock_wait_count: u64

        /// This is only populated when TimedEnv is used.
        (<) env_new_sequential_file_nanos: u64

        /// This is only populated when TimedEnv is used.
        (<) env_new_random_access_file_nanos: u64

        /// This is only populated when TimedEnv is used.
        (<) env_new_writable_file_nanos: u64

        /// This is only populated when TimedEnv is used.
        (<) env_reuse_writable_file_nanos: u64

        /// This is only populated when TimedEnv is used.
        (<) env_new_random_rw_file_nanos: u64

        /// This is only populated when TimedEnv is used.
        (<) env_new_directory_nanos: u64

        /// This is only populated when TimedEnv is used.
        (<) env_file_exists_nanos: u64

        /// This is only populated when TimedEnv is used.
        (<) env_get_children_nanos: u64

        /// This is only populated when TimedEnv is used.
        (<) env_get_children_file_attributes_nanos: u64

        /// This is only populated when TimedEnv is used.
        (<) env_delete_file_nanos: u64

        /// This is only populated when TimedEnv is used.
        (<) env_create_dir_nanos: u64

        /// This is only populated when TimedEnv is used.
        (<) env_create_dir_if_missing_nanos: u64

        /// This is only populated when TimedEnv is used.
        (<) env_delete_dir_nanos: u64

        /// This is only populated when TimedEnv is used.
        (<) env_get_file_size_nanos: u64

        /// This is only populated when TimedEnv is used.
        (<) env_get_file_modification_time_nanos: u64

        /// This is only populated when TimedEnv is used.
        (<) env_rename_file_nanos: u64

        /// This is only populated when TimedEnv is used.
        (<) env_link_file_nanos: u64

        /// This is only populated when TimedEnv is used.
        (<) env_lock_file_nanos: u64

        /// This is only populated when TimedEnv is used.
        (<) env_unlock_file_nanos: u64

        /// This is only populated when TimedEnv is used.
        (<) env_new_logger_nanos: u64

        /// Total nanos spent on get
        (<) get_cpu_nanos: u64

        /// Total nanos spent on next
        (<) iter_next_cpu_nanos: u64

        /// Total nanos spent on prev
        (<) iter_prev_cpu_nanos: u64

        /// Total nanos spent on seek
        (<) iter_seek_cpu_nanos: u64

        /// Time spent in encrypting data. Populated when EncryptedEnv is used.
        (<) encrypt_data_nanos: u64

        /// Time spent in decrypting data. Populated when EncryptedEnv is used.
        (<) decrypt_data_nanos: u64
    }

    /// Get Thread-local PerfContext object pointer
    #[inline]
    pub fn get() -> PerfContext {
        PerfContext {
            ptr: unsafe { tirocks_sys::crocksdb_get_perf_context() },
        }
    }

    #[inline]
    fn as_ptr(&self) -> *const rocksdb_PerfContext {
        self.ptr
    }
}

/// A thread local context for gathering io-stats efficiently and transparently.
/// Use SetPerfLevel(PerfLevel::kEnableTime) to enable time stats.
pub struct IoStatsContext {
    ptr: *mut rocksdb_IOStatsContext,
}

impl IoStatsContext {
    /// Reset all io-stats counter to zero
    #[inline]
    pub fn reset(&mut self) {
        unsafe { tirocks_sys::crocksdb_iostats_context_reset(self.ptr) }
    }

    simple_access! {
        crocksdb_iostats_context

        /// The thread pool id
        (<) thread_pool_id: u64

        /// Number of bytes that has been written.
        (<) bytes_written: u64

        /// Number of bytes that has been read.
        (<) bytes_read: u64

        /// Time spent in open() and fopen().
        (<) open_nanos: u64

        /// Time spent in fallocate().
        (<) allocate_nanos: u64

        /// Time spent in write() and pwrite().
        (<) write_nanos: u64

        /// Time spent in read() and pread()
        (<) read_nanos: u64

        /// Time spent in sync_file_range().
        (<) range_sync_nanos: u64

        /// Time spent in fsync
        (<) fsync_nanos: u64

        /// Time spent in preparing write (fallocate etc).
        (<) prepare_write_nanos: u64

        /// Time spent in Logger::Logv().
        (<) logger_nanos: u64

        /// CPU time spent in write() and pwrite()
        (<) cpu_write_nanos: u64

        /// CPU time spent in read() and pread()
        (<) cpu_read_nanos: u64

    }

    /// Get Thread-local IOStatsContext object pointer
    #[inline]
    pub fn get() -> IoStatsContext {
        IoStatsContext {
            ptr: unsafe { tirocks_sys::crocksdb_get_iostats_context() },
        }
    }

    #[inline]
    fn as_ptr(&self) -> *const rocksdb_IOStatsContext {
        self.ptr
    }
}
