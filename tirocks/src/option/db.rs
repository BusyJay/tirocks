// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::{
    mem,
    ops::{Deref, DerefMut},
    path::Path,
    sync::Arc,
    time::Duration,
};
use tirocks_sys::{rocksdb_DBOptions, rocksdb_titandb_TitanDBOptions};

use crate::{
    env::{
        logger::{LogLevel, SysInfoLogger},
        Env,
    },
    listener::SysEventListener,
    rate_limiter::RateLimiter,
    util::simple_access,
    Statistics,
};

use super::PathToSlice;

pub type AccessHint = tirocks_sys::rocksdb_DBOptions_AccessHint;
pub type WalRecoveryMode = tirocks_sys::rocksdb_WALRecoveryMode;

#[derive(Debug)]
#[repr(C)]
pub struct DbOptions {
    ptr: *mut rocksdb_DBOptions,
    env: Option<Arc<Env>>,
}

impl Default for DbOptions {
    #[inline]
    fn default() -> DbOptions {
        let ptr = unsafe { tirocks_sys::crocksdb_dboptions_create() };
        DbOptions { ptr, env: None }
    }
}

impl Drop for DbOptions {
    #[inline]
    fn drop(&mut self) {
        unsafe { tirocks_sys::crocksdb_dboptions_destroy(self.ptr) }
    }
}

impl DbOptions {
    simple_access! {
        /// If true, the database will be created if it is missing.
        /// Default: false
        create_if_missing: bool

        /// If true, missing column families will be automatically created.
        /// Default: false
        create_missing_column_families: bool

        /// If true, an error is raised if the database already exists.
        /// Default: false
        error_if_exists: bool

        /// If true, RocksDB will aggressively check consistency of the data.
        /// Also, if any of the  writes to the database fails (Put, Delete, Merge,
        /// Write), the database will switch to read-only mode and fail all other
        /// Write operations.
        /// In most cases you want this to be set to true.
        /// Default: true
        paranoid_checks: bool
    }

    /// Use the specified object to interact with the environment,
    /// e.g. to read/write files, schedule background work, etc.
    /// Default: Env::Default()
    #[inline]
    pub fn set_env(&mut self, env: Arc<Env>) -> &mut Self {
        unsafe {
            tirocks_sys::crocksdb_options_set_env(self.ptr, env.as_mut_ptr());
        }
        // It may be unsafe to drop the old env if the options is still used for other db.
        self.env = Some(env);
        self
    }

    simple_access! {
        /// Use to control write rate of flush and compaction. Flush has higher
        /// priority than compaction. Rate limiting is disabled by default.
        /// If rate limiter is specified, bytes_per_sync is set to 1MB by default.
        ///
        /// Multiple DBs can share the same limiter.
        rate_limiter / ratelimiter / : &RateLimiter [ .as_mut_ptr() ]

        /// Any internal progress/error information generated by the db will
        /// be written to logger.
        ///
        /// If it is not set, the informations will be written to a file stored
        /// in the same directory as the DB contents.
        info_log: &SysInfoLogger [ .get() ]

        /// Set the log level.
        info_log_level: LogLevel

        /// Number of open files that can be used by the DB.  You may need to
        /// increase this if your database has a large working set. Value -1 means
        /// files opened are always kept open. You can estimate number of files based
        /// on target_file_size_base and target_file_size_multiplier for level-based
        /// compaction. For universal-style compaction, you can usually set it to -1.
        ///
        /// Default: -1
        ///
        /// Dynamically changeable through set_db_options() API.
        max_open_files: i32

        /// Once write-ahead logs exceed this size, we will start forcing the flush of
        /// column families whose memtables are backed by the oldest live WAL file
        /// (i.e. the ones that are causing all the space amplification). If set to 0
        /// (default), we will dynamically choose the WAL size limit to be
        /// [sum of all write_buffer_size * max_write_buffer_number] * 4
        /// This option takes effect only when there are more than one column family as
        /// otherwise the wal size is dictated by the write_buffer_size.
        ///
        /// Default: 0
        ///
        /// Dynamically changeable through SetDBOptions() API.
        max_total_wal_size: u64

        /// Collect metrics about database operations.
        ///
        /// Several DB can share the same statistics and they will be aggregated.
        statistics: &Statistics [ .as_mut_ptr() ]

        /// By default, writes to stable storage use fdatasync (on platforms
        /// where this function is available). If this option is true,
        /// fsync is used instead.
        ///
        /// fsync and fdatasync are equally safe for our purposes and fdatasync is
        /// faster, so it is rarely necessary to set this option. It is provided
        /// as a workaround for kernel/filesystem bugs, such as one that affected
        /// fdatasync with ext4 in kernel versions prior to 3.7.
        use_fsync: bool
    }

    /// Paths where SST files can be put into, with its target size.
    /// Newer data is placed into paths specified earlier while
    /// older data gradually moves to paths specified later.
    ///
    /// For example, you have a flash device with 10GB allocated for the DB,
    /// as well as a hard drive of 2TB, you should config it to be:
    ///   [{"/flash_path", 10GB}, {"/hard_drive", 2TB}]
    ///
    /// The system will try to guarantee data under each path is close to but
    /// not larger than the target size. But current and future file sizes used
    /// by determining where to place a file are based on best-effort estimation,
    /// which means there is a chance that the actual size under the directory
    /// is slightly more than target size under some workloads. User should give
    /// some buffer room for those cases.
    ///
    /// If none of the paths has sufficient room to place a file, the file will
    /// be placed to the last path anyway, despite to the target size.
    ///
    /// Placing newer data to earlier paths is also best-efforts. User should
    /// expect user files to be placed in higher levels in some extreme cases.
    ///
    /// If not specified, only one path will be used, which is db_name passed when
    /// opening the DB.
    #[inline]
    pub fn add_db_path(&mut self, path: impl AsRef<Path>, target_size: u64) -> &mut Self {
        unsafe {
            tirocks_sys::crocksdb_options_add_db_paths(self.ptr, path.path_to_slice(), target_size);
        }
        self
    }

    simple_access! {
        /// This specifies the info LOG dir.
        /// If it is empty, the log files will be in the same dir as data.
        /// If it is non empty, the log files will be in the specified dir,
        /// and the db data dir's absolute path will be used as the log file
        /// name's prefix.
        db_log_dir: impl AsRef<Path> [ .path_to_slice() ]

        /// This specifies the absolute dir path for write-ahead logs (WAL).
        /// If it is empty, the log files will be in the same dir as data,
        ///   dbname is used as the data dir by default
        /// If it is non empty, the log files will be in kept the specified dir.
        /// When destroying the db,
        ///   all log files in wal_dir and the dir itself is deleted
        wal_dir: impl AsRef<Path> [ .path_to_slice() ]

        /// The periodicity when obsolete files get deleted. The default
        /// value is 6 hours. The files that get out of scope by compaction
        /// process will still get automatically delete on every compaction,
        /// regardless of this setting. Values is wiped up to micros.
        ///
        /// Default: 6 hours
        ///
        /// Dynamically changeable through SetDBOptions() API.
        delete_obsolete_files_period / delete_obsolete_files_period_micros / : Duration [ .as_micros() as u64 ]

        /// Maximum number of concurrent background jobs (compactions and flushes).
        ///
        /// Default: 2
        ///
        /// Dynamically changeable through SetDBOptions() API.
        max_background_jobs: i32

        /// NOT SUPPORTED ANYMORE: RocksDB automatically decides this based on the
        /// value of max_background_jobs. For backwards compatibility we will set
        /// `max_background_jobs = max_background_compactions + max_background_flushes`
        /// in the case where user sets at least one of `max_background_compactions` or
        /// `max_background_flushes` (we replace -1 by 1 in case one option is unset).
        ///
        /// Maximum number of concurrent background compaction jobs, submitted to
        /// the default LOW priority thread pool.
        ///
        /// If you're increasing this, also consider increasing number of threads in
        /// LOW priority thread pool. For more information, see
        /// Env::SetBackgroundThreads
        ///
        /// Default: -1
        ///
        /// Dynamically changeable through SetDBOptions() API.
        #[deprecated]
        max_background_compactions: i32

        /// This value represents the maximum number of threads that will
        /// concurrently perform a compaction job by breaking it into multiple,
        /// smaller ones that are run simultaneously.
        /// Default: 1 (i.e. no subcompactions)
        max_subcompactions: u32

        /// NOT SUPPORTED ANYMORE: RocksDB automatically decides this based on the
        /// value of max_background_jobs. For backwards compatibility we will set
        /// `max_background_jobs = max_background_compactions + max_background_flushes`
        /// in the case where user sets at least one of `max_background_compactions` or
        /// `max_background_flushes`.
        ///
        /// Maximum number of concurrent background memtable flush jobs, submitted by
        /// default to the HIGH priority thread pool. If the HIGH priority thread pool
        /// is configured to have zero threads, flush jobs will share the LOW priority
        /// thread pool with compaction jobs.
        ///
        /// It is important to use both thread pools when the same Env is shared by
        /// multiple db instances. Without a separate pool, long running compaction
        /// jobs could potentially block memtable flush jobs of other db instances,
        /// leading to unnecessary Put stalls.
        ///
        /// If you're increasing this, also consider increasing number of threads in
        /// HIGH priority thread pool. For more information, see
        /// Env::SetBackgroundThreads
        /// Default: -1
        #[deprecated]
        max_background_flushes: i32

        /// Specify the maximal size of the info log file. If the log file
        /// is larger than `max_log_file_size`, a new info log file will
        /// be created.
        /// If max_log_file_size == 0, all logs will be written to one
        /// log file.
        max_log_file_size: usize

        /// If specified with non-zero value, log file will be rolled
        /// if it has been active longer than `log_file_time_to_roll`.
        /// `time` will be wiped up to seconds.
        /// Default: 0 (disabled)
        log_file_time_to_roll: Duration [ .as_secs() as usize ]

        /// Maximal info log files to be kept.
        /// Default: 1000
        keep_log_file_num: usize

        /// Recycle log files.
        /// If non-zero, we will reuse previously written log files for new
        /// logs, overwriting the old data.  The value indicates how many
        /// such files we will keep around at any point in time for later
        /// use.  This is more efficient because the blocks are already
        /// allocated and fdatasync does not need to update the inode after
        /// each write.
        /// Default: 0
        recycle_log_file_num: usize

        /// manifest file is rolled over on reaching this limit.
        /// The older manifest file be deleted.
        /// The default value is 1GB so that the manifest file can grow, but not
        /// reach the limit of storage capacity.
        max_manifest_file_size: u64

        /// Number of shards used for table cache.
        table_cache_num_shard_bits / table_cache_numshardbits / : i32

        /// The following two fields affect how archived logs will be deleted.
        /// 1. If both set to 0, logs will be deleted asap and will not get into
        ///    the archive.
        /// 2. If WAL_ttl is 0 and WAL_size_limit_MB is not 0,
        ///    WAL files will be checked every 10 min and if total size is greater
        ///    then WAL_size_limit_MB, they will be deleted starting with the
        ///    earliest until size_limit is met. All empty files will be deleted.
        /// 3. If WAL_ttl is not 0 and WAL_size_limit_MB is 0, then
        ///    WAL files will be checked every WAL_ttl / 2 and those that
        ///    are older than WAL_ttl will be deleted.
        /// 4. If both are not 0, WAL files will be checked every 10 min and both
        ///    checks will be performed with ttl being first.
        /// `ttl` will be wiped up to seconds.
        wal_ttl / wal_ttl_seconds / : Duration [ .as_secs() as u64 ]

        /// The following two fields affect how archived logs will be deleted.
        /// 1. If both set to 0, logs will be deleted asap and will not get into
        ///    the archive.
        /// 2. If WAL_ttl is 0 and WAL_size_limit_MB is not 0,
        ///    WAL files will be checked every 10 min and if total size is greater
        ///    then WAL_size_limit_MB, they will be deleted starting with the
        ///    earliest until size_limit is met. All empty files will be deleted.
        /// 3. If WAL_ttl is not 0 and WAL_size_limit_MB is 0, then
        ///    WAL files will be checked every WAL_ttl / 2 and those that
        ///    are older than WAL_ttl will be deleted.
        /// 4. If both are not 0, WAL files will be checked every 10 min and both
        ///    checks will be performed with ttl being first.
        /// `size` will be wiped up to MiB.
        wal_size_limit / wal_size_limit_mb / : u64 [ / 1024 / 1024 ]

        /// Number of bytes to preallocate (via fallocate) the manifest
        /// files.  Default is 4mb, which is reasonable to reduce random IO
        /// as well as prevent overallocation for mounts that preallocate
        /// large amounts of data (such as xfs's allocsize option).
        manifest_preallocation_size: usize

        /// Allow the OS to mmap file for reading sst tables. Default: false
        allow_mmap_reads: bool

        /// Allow the OS to mmap file for writing.
        /// DB::SyncWAL() only works if this is set to false.
        /// Default: false
        allow_mmap_writes: bool

        /// Use O_DIRECT for user and compaction reads.
        /// When true, we also force new_table_reader_for_compaction_inputs to true.
        /// Default: false
        use_direct_reads: bool

        /// Use O_DIRECT for writes in background flush and compactions.
        /// Default: false
        use_direct_io_for_flush_and_compaction: bool

        /// Disable child process inherit open files. Default: true
        is_fd_close_on_exec: bool

        /// If not zero, dump rocksdb.stats to LOG every stats_dump_period_sec
        ///
        /// Default: 10 min
        ///
        /// Dynamically changeable through SetDBOptions() API.
        stats_dump_period / stats_dump_period_sec / : Duration [ .as_secs() as u32 ]

        /// If set true, will hint the underlying file system that the file
        /// access pattern is random, when a sst file is opened.
        /// Default: true
        advise_random_on_open: bool

        /// Amount of data to build up in memtables across all column
        /// families before writing to disk.
        ///
        /// This is distinct from write_buffer_size, which enforces a limit
        /// for a single memtable.
        ///
        /// This feature is disabled by default. Specify a non-zero value
        /// to enable it.
        ///
        /// Default: 0 (disabled)
        db_write_buffer_size: usize

        /// Specify the file access pattern once a compaction is started.
        /// It will be applied to all input files of a compaction.
        /// Default: NORMAL
        access_hint_on_compaction_start: AccessHint

        /// If non-zero, we perform bigger reads when doing compaction. If you're
        /// running RocksDB on spinning disks, you should set this to at least 2MB.
        /// That way RocksDB's compaction is doing sequential instead of random reads.
        ///
        /// When non-zero, we also force new_table_reader_for_compaction_inputs to
        /// true.
        ///
        /// Default: 0
        ///
        /// Dynamically changeable through SetDBOptions() API.
        compaction_readahead_size: usize

        /// This is the maximum buffer size that is used by WritableFileWriter.
        /// On Windows, we need to maintain an aligned buffer for writes.
        /// We allow the buffer to grow until it's size hits the limit in buffered
        /// IO and fix the buffer size when using direct IO to ensure alignment of
        /// write requests if the logical sector size is unusual
        ///
        /// Default: 1024 * 1024 (1 MB)
        ///
        /// Dynamically changeable through SetDBOptions() API.
        writable_file_max_buffer_size: usize

        /// Use adaptive mutex, which spins in the user space before resorting
        /// to kernel. This could reduce context switch when the mutex is not
        /// heavily contended. However, if the mutex is hot, we could end up
        /// wasting spin time.
        /// Default: false
        use_adaptive_mutex: bool

        /// Allows OS to incrementally sync files to disk while they are being
        /// written, asynchronously, in the background. This operation can be used
        /// to smooth out write I/Os over time. Users shouldn't rely on it for
        /// persistency guarantee.
        /// Issue one request for every bytes_per_sync written. 0 turns it off.
        ///
        /// You may consider using rate_limiter to regulate write rate to device.
        /// When rate limiter is enabled, it automatically enables bytes_per_sync
        /// to 1MB.
        ///
        /// This option applies to table files
        ///
        /// Default: 0, turned off
        ///
        /// Note: DOES NOT apply to WAL files. See wal_bytes_per_sync instead
        /// Dynamically changeable through SetDBOptions() API.
        bytes_per_sync: u64

        /// Same as bytes_per_sync, but applies to WAL files
        ///
        /// Default: 0, turned off
        ///
        /// Dynamically changeable through SetDBOptions() API.
        wal_bytes_per_sync: u64

        /// Add an EventListeners whose callback functions will be called
        /// when specific RocksDB event happens. Listener can be shared with multiple
        /// DBs.
        <add> event_listener / eventlistener/ : &SysEventListener [ .get() ]

        /// The limited write rate to DB if soft_pending_compaction_bytes_limit or
        /// level0_slowdown_writes_trigger is triggered, or we are writing to the
        /// last mem table allowed and we allow more than 3 mem tables. It is
        /// calculated using size of user write requests before compression.
        /// RocksDB may decide to slow down more if the compaction still
        /// gets behind further.
        /// If the value is 0, we will infer a value from `rater_limiter` value
        /// if it is not empty, or 16MB if `rater_limiter` is empty. Note that
        /// if users change the rate in `rate_limiter` after DB is opened,
        /// `delayed_write_rate` won't be adjusted.
        ///
        /// Unit: byte per second.
        ///
        /// Default: 0
        ///
        /// Dynamically changeable through SetDBOptions() API.
        delayed_write_rate: u64

        /// By default, a single write thread queue is maintained. The thread gets
        /// to the head of the queue becomes write batch group leader and responsible
        /// for writing to WAL and memtable for the batch group.
        ///
        /// If enable_pipelined_write is true, separate write thread queue is
        /// maintained for WAL write and memtable write. A write thread first enter WAL
        /// writer queue and then memtable writer queue. Pending thread on the WAL
        /// writer queue thus only have to wait for previous writers to finish their
        /// WAL writing but not the memtable writing. Enabling the feature may improve
        /// write throughput and reduce latency of the prepare phase of two-phase
        /// commit.
        ///
        /// Default: false
        enable_pipelined_write: bool

        /// Setting unordered_write to true trades higher write throughput with
        /// relaxing the immutability guarantee of snapshots. This violates the
        /// repeatability one expects from ::Get from a snapshot, as well as
        /// ::MultiGet and Iterator's consistent-point-in-time view property.
        /// If the application cannot tolerate the relaxed guarantees, it can implement
        /// its own mechanisms to work around that and yet benefit from the higher
        /// throughput. Using TransactionDB with WRITE_PREPARED write policy and
        /// two_write_queues=true is one way to achieve immutable snapshots despite
        /// unordered_write.
        ///
        /// By default, i.e., when it is false, rocksdb does not advance the sequence
        /// number for new snapshots unless all the writes with lower sequence numbers
        /// are already finished. This provides the immutability that we except from
        /// snapshots. Moreover, since Iterator and MultiGet internally depend on
        /// snapshots, the snapshot immutability results into Iterator and MultiGet
        /// offering consistent-point-in-time view. If set to true, although
        /// Read-Your-Own-Write property is still provided, the snapshot immutability
        /// property is relaxed: the writes issued after the snapshot is obtained (with
        /// larger sequence numbers) will be still not visible to the reads from that
        /// snapshot, however, there still might be pending writes (with lower sequence
        /// number) that will change the state visible to the snapshot after they are
        /// landed to the memtable.
        ///
        /// Default: false
        unordered_write: bool

        /// By default, a single write thread queue is maintained. The thread gets
        /// to the head of the queue becomes write batch group leader and responsible
        /// for writing to WAL.
        ///
        /// If enable_pipelined_commit is true, RocksDB will apply WriteBatch to
        /// memtable out of order but commit them in order.
        ///
        /// Default: false
        enable_pipelined_commit: bool

        /// If true, allow multi-writers to update mem tables in parallel.
        /// Only some memtable_factory-s support concurrent writes; currently it
        /// is implemented only for SkipListFactory.  Concurrent memtable writes
        /// are not compatible with inplace_update_support or filter_deletes.
        /// It is strongly recommended to set enable_write_thread_adaptive_yield
        /// if you are going to use this feature.
        ///
        /// Default: true
        allow_concurrent_memtable_write: bool

        /// If true, threads synchronizing with the write batch group leader will
        /// wait for up to write_thread_max_yield_usec before blocking on a mutex.
        /// This can substantially improve throughput for concurrent workloads,
        /// regardless of whether allow_concurrent_memtable_write is enabled.
        ///
        /// Default: true
        enable_write_thread_adaptive_yield: bool

        /// Recovery mode to control the consistency while replaying WAL
        /// Default: kPointInTimeRecovery
        wal_recovery_mode: WalRecoveryMode

        /// If true WAL is not flushed automatically after each write. Instead it
        /// relies on manual invocation of FlushWAL to write the WAL buffer to its
        /// file.
        manual_wal_flush: bool

        /// If true, RocksDB supports flushing multiple column families and committing
        /// their results atomically to MANIFEST. Note that it is not
        /// necessary to set atomic_flush to true if WAL is always enabled since WAL
        /// allows the database to be restored to the last persistent state in WAL.
        /// This option is useful when there are column families with writes NOT
        /// protected by WAL.
        /// For manual flush, application has to specify which column families to
        /// flush atomically in DB::Flush.
        /// For auto-triggered flush, RocksDB atomically flushes ALL column families.
        ///
        /// Currently, any WAL-enabled writes after atomic flush may be replayed
        /// independently if the process crashes later and tries to recover.
        atomic_flush: bool
    }

    /// By default, RocksDB uses only one background thread for flush and
    /// compaction. Calling this function will set it up such that total of
    /// `total_threads` is used. Good value for `total_threads` is the number of
    /// cores. You almost definitely want to call this function if your system is
    /// bottlenecked by RocksDB.
    #[inline]
    pub fn increase_parallelism(&mut self, total_threads: i32) -> &mut Self {
        unsafe {
            tirocks_sys::crocksdb_options_increase_parallelism(self.ptr, total_threads);
        }
        self
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct TitanDbOptions {
    ptr: *mut rocksdb_titandb_TitanDBOptions,
    env: Option<Arc<Env>>,
}

impl Deref for TitanDbOptions {
    type Target = DbOptions;

    #[inline]
    fn deref(&self) -> &DbOptions {
        unsafe {
            // TitanDBOptions inherits DBOptions, so the two structs are identical.
            mem::transmute(self)
        }
    }
}

impl DerefMut for TitanDbOptions {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            // TitanDBOptions inherits DBOptions, so the two structs are identical.
            mem::transmute(self)
        }
    }
}

impl TitanDbOptions {
    simple_access! {
        ctitandb_options

        /// The directory to store data specific to TitanDB alongside with
        /// the base DB.
        ///
        /// Default: {dbname}/titandb
        directory / dirname / : impl AsRef<Path> [ .path_to_slice() ]

        /// Disable background GC
        ///
        /// Default: false
        disable_background_gc: bool

        /// Max background GC thread
        ///
        /// Default: 1
        max_background_gc: i32

        /// How often to schedule delete obsolete blob files periods.
        /// If set zero, obsolete blob files won't be deleted.
        /// `period` will be wiped up to seconds.
        ///
        /// Default: 10s
        purge_obsolete_files_period / purge_obsolete_files_period_sec / : Duration [ .as_secs() as u32 ]
    }
}

impl Default for TitanDbOptions {
    #[inline]
    fn default() -> TitanDbOptions {
        let ptr = unsafe { tirocks_sys::ctitandb_dboptions_create() };
        TitanDbOptions { ptr, env: None }
    }
}

impl Drop for TitanDbOptions {
    #[inline]
    fn drop(&mut self) {
        unsafe { tirocks_sys::ctitandb_dboptions_destroy(self.ptr) }
    }
}
