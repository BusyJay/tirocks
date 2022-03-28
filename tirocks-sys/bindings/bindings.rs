#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct rocksdb_Slice {
    pub data_: *const libc::c_char,
    pub size_: usize,
}
#[repr(C)]
#[derive(Debug)]
pub struct rocksdb_Status {
    pub code_: rocksdb_Status_Code,
    pub subcode_: rocksdb_Status_SubCode,
    pub sev_: rocksdb_Status_Severity,
    pub state_: *const libc::c_char,
}
#[repr(u8)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum rocksdb_Status_Code {
    kOk = 0,
    kNotFound = 1,
    kCorruption = 2,
    kNotSupported = 3,
    kInvalidArgument = 4,
    kIOError = 5,
    kMergeInProgress = 6,
    kIncomplete = 7,
    kShutdownInProgress = 8,
    kTimedOut = 9,
    kAborted = 10,
    kBusy = 11,
    kExpired = 12,
    kTryAgain = 13,
    kCompactionTooLarge = 14,
    kColumnFamilyDropped = 15,
    kMaxCode = 16,
}
#[repr(u8)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum rocksdb_Status_SubCode {
    kNone = 0,
    kMutexTimeout = 1,
    kLockTimeout = 2,
    kLockLimit = 3,
    kNoSpace = 4,
    kDeadlock = 5,
    kStaleFile = 6,
    kMemoryLimit = 7,
    kSpaceLimit = 8,
    kPathNotFound = 9,
    kMaxSubCode = 10,
}
#[repr(u8)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum rocksdb_Status_Severity {
    kNoError = 0,
    kSoftError = 1,
    kHardError = 2,
    kFatalError = 3,
    kUnrecoverableError = 4,
    kMaxSeverity = 5,
}
#[repr(C)]
#[repr(align(8))]
#[derive(Debug, Copy, Clone)]
pub struct rocksdb_Env {
    pub _bindgen_opaque_blob: [u64; 2usize],
}
#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum rocksdb_Env_WriteLifeTimeHint {
    WLTH_NOT_SET = 0,
    WLTH_NONE = 1,
    WLTH_SHORT = 2,
    WLTH_MEDIUM = 3,
    WLTH_LONG = 4,
    WLTH_EXTREME = 5,
}
#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum rocksdb_Env_Priority {
    BOTTOM = 0,
    LOW = 1,
    HIGH = 2,
    USER = 3,
    TOTAL = 4,
}
#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum rocksdb_Env_IOPriority {
    IO_LOW = 0,
    IO_HIGH = 1,
    IO_TOTAL = 2,
}
#[repr(u8)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum rocksdb_InfoLogLevel {
    DEBUG_LEVEL = 0,
    INFO_LEVEL = 1,
    WARN_LEVEL = 2,
    ERROR_LEVEL = 3,
    FATAL_LEVEL = 4,
    HEADER_LEVEL = 5,
    NUM_INFO_LOG_LEVELS = 6,
}
#[repr(i32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum rocksdb_encryption_EncryptionMethod {
    kUnknown = 0,
    kPlaintext = 1,
    kAES128_CTR = 2,
    kAES192_CTR = 3,
    kAES256_CTR = 4,
}
#[repr(C)]
pub struct rocksdb_encryption_KeyManager__bindgen_vtable(libc::c_void);
#[repr(C)]
#[derive(Debug)]
pub struct rocksdb_encryption_KeyManager {
    pub vtable_: *const rocksdb_encryption_KeyManager__bindgen_vtable,
}
pub type rocksdb_SequenceNumber = u64;
#[repr(i32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum rocksdb_TableFileCreationReason {
    kFlush = 0,
    kCompaction = 1,
    kRecovery = 2,
    kMisc = 3,
}
#[repr(i32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum rocksdb_BackgroundErrorReason {
    kFlush = 0,
    kCompaction = 1,
    kWriteCallback = 2,
    kMemTable = 3,
}
#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum rocksdb_CompactionStopStyle {
    kCompactionStopStyleSimilarSize = 0,
    kCompactionStopStyleTotalSize = 1,
}
#[repr(i8)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum rocksdb_CompactionStyle {
    kCompactionStyleLevel = 0,
    kCompactionStyleUniversal = 1,
    kCompactionStyleFIFO = 2,
    kCompactionStyleNone = 3,
}
#[repr(i8)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum rocksdb_CompactionPri {
    kByCompensatedSize = 0,
    kOldestLargestSeqFirst = 1,
    kOldestSmallestSeqFirst = 2,
    kMinOverlappingRatio = 3,
}
#[repr(i8)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum rocksdb_PartitionerResult {
    kNotRequired = 0,
    kRequired = 1,
}
#[repr(u32)]
#[doc = " Keep adding ticker's here."]
#[doc = "  1. Any ticker should be added before TICKER_ENUM_MAX."]
#[doc = "  2. Add a readable string in TickersNameMap below for the newly added ticker."]
#[doc = "  3. Add a corresponding enum value to TickerType.java in the java API"]
#[doc = "  4. Add the enum conversions from Java and C++ to portal.h's toJavaTickerType"]
#[doc = " and toCppTickers"]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum rocksdb_Tickers {
    BLOCK_CACHE_MISS = 0,
    BLOCK_CACHE_HIT = 1,
    BLOCK_CACHE_ADD = 2,
    BLOCK_CACHE_ADD_FAILURES = 3,
    BLOCK_CACHE_INDEX_MISS = 4,
    BLOCK_CACHE_INDEX_HIT = 5,
    BLOCK_CACHE_INDEX_ADD = 6,
    BLOCK_CACHE_INDEX_BYTES_INSERT = 7,
    BLOCK_CACHE_INDEX_BYTES_EVICT = 8,
    BLOCK_CACHE_FILTER_MISS = 9,
    BLOCK_CACHE_FILTER_HIT = 10,
    BLOCK_CACHE_FILTER_ADD = 11,
    BLOCK_CACHE_FILTER_BYTES_INSERT = 12,
    BLOCK_CACHE_FILTER_BYTES_EVICT = 13,
    BLOCK_CACHE_DATA_MISS = 14,
    BLOCK_CACHE_DATA_HIT = 15,
    BLOCK_CACHE_DATA_ADD = 16,
    BLOCK_CACHE_DATA_BYTES_INSERT = 17,
    BLOCK_CACHE_BYTES_READ = 18,
    BLOCK_CACHE_BYTES_WRITE = 19,
    BLOOM_FILTER_USEFUL = 20,
    BLOOM_FILTER_FULL_POSITIVE = 21,
    BLOOM_FILTER_FULL_TRUE_POSITIVE = 22,
    BLOOM_FILTER_MICROS = 23,
    PERSISTENT_CACHE_HIT = 24,
    PERSISTENT_CACHE_MISS = 25,
    SIM_BLOCK_CACHE_HIT = 26,
    SIM_BLOCK_CACHE_MISS = 27,
    MEMTABLE_HIT = 28,
    MEMTABLE_MISS = 29,
    GET_HIT_L0 = 30,
    GET_HIT_L1 = 31,
    GET_HIT_L2_AND_UP = 32,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    COMPACTION_KEY_DROP_NEWER_ENTRY = 33,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    COMPACTION_KEY_DROP_OBSOLETE = 34,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    COMPACTION_KEY_DROP_RANGE_DEL = 35,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    COMPACTION_KEY_DROP_USER = 36,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    COMPACTION_RANGE_DEL_DROP_OBSOLETE = 37,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    COMPACTION_OPTIMIZED_DEL_DROP_OBSOLETE = 38,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    COMPACTION_CANCELLED = 39,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    NUMBER_KEYS_WRITTEN = 40,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    NUMBER_KEYS_READ = 41,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    NUMBER_KEYS_UPDATED = 42,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    BYTES_WRITTEN = 43,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    BYTES_READ = 44,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    NUMBER_DB_SEEK = 45,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    NUMBER_DB_NEXT = 46,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    NUMBER_DB_PREV = 47,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    NUMBER_DB_SEEK_FOUND = 48,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    NUMBER_DB_NEXT_FOUND = 49,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    NUMBER_DB_PREV_FOUND = 50,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    ITER_BYTES_READ = 51,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    NO_FILE_CLOSES = 52,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    NO_FILE_OPENS = 53,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    NO_FILE_ERRORS = 54,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    STALL_L0_SLOWDOWN_MICROS = 55,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    STALL_MEMTABLE_COMPACTION_MICROS = 56,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    STALL_L0_NUM_FILES_MICROS = 57,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    STALL_MICROS = 58,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    DB_MUTEX_WAIT_MICROS = 59,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    RATE_LIMIT_DELAY_MILLIS = 60,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    NO_ITERATORS = 61,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    NUMBER_MULTIGET_CALLS = 62,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    NUMBER_MULTIGET_KEYS_READ = 63,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    NUMBER_MULTIGET_BYTES_READ = 64,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    NUMBER_FILTERED_DELETES = 65,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    NUMBER_MERGE_FAILURES = 66,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    BLOOM_FILTER_PREFIX_CHECKED = 67,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    BLOOM_FILTER_PREFIX_USEFUL = 68,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    NUMBER_OF_RESEEKS_IN_ITERATION = 69,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    GET_UPDATES_SINCE_CALLS = 70,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    BLOCK_CACHE_COMPRESSED_MISS = 71,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    BLOCK_CACHE_COMPRESSED_HIT = 72,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    BLOCK_CACHE_COMPRESSED_ADD = 73,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    BLOCK_CACHE_COMPRESSED_ADD_FAILURES = 74,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    WAL_FILE_SYNCED = 75,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    WAL_FILE_BYTES = 76,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    WRITE_DONE_BY_SELF = 77,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    WRITE_DONE_BY_OTHER = 78,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    WRITE_TIMEDOUT = 79,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    WRITE_WITH_WAL = 80,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    COMPACT_READ_BYTES = 81,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    COMPACT_WRITE_BYTES = 82,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    FLUSH_WRITE_BYTES = 83,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    NUMBER_DIRECT_LOAD_TABLE_PROPERTIES = 84,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    NUMBER_SUPERVERSION_ACQUIRES = 85,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    NUMBER_SUPERVERSION_RELEASES = 86,
    #[doc = " COMPACTION_KEY_DROP_* count the reasons for key drop during compaction"]
    #[doc = " There are 4 reasons currently."]
    NUMBER_SUPERVERSION_CLEANUPS = 87,
    NUMBER_BLOCK_COMPRESSED = 88,
    NUMBER_BLOCK_DECOMPRESSED = 89,
    NUMBER_BLOCK_NOT_COMPRESSED = 90,
    MERGE_OPERATION_TOTAL_TIME = 91,
    FILTER_OPERATION_TOTAL_TIME = 92,
    ROW_CACHE_HIT = 93,
    ROW_CACHE_MISS = 94,
    READ_AMP_ESTIMATE_USEFUL_BYTES = 95,
    READ_AMP_TOTAL_READ_BYTES = 96,
    NUMBER_RATE_LIMITER_DRAINS = 97,
    NUMBER_ITER_SKIP = 98,
    BLOB_DB_NUM_PUT = 99,
    BLOB_DB_NUM_WRITE = 100,
    BLOB_DB_NUM_GET = 101,
    BLOB_DB_NUM_MULTIGET = 102,
    BLOB_DB_NUM_SEEK = 103,
    BLOB_DB_NUM_NEXT = 104,
    BLOB_DB_NUM_PREV = 105,
    BLOB_DB_NUM_KEYS_WRITTEN = 106,
    BLOB_DB_NUM_KEYS_READ = 107,
    BLOB_DB_BYTES_WRITTEN = 108,
    BLOB_DB_BYTES_READ = 109,
    BLOB_DB_WRITE_INLINED = 110,
    BLOB_DB_WRITE_INLINED_TTL = 111,
    BLOB_DB_WRITE_BLOB = 112,
    BLOB_DB_WRITE_BLOB_TTL = 113,
    BLOB_DB_BLOB_FILE_BYTES_WRITTEN = 114,
    BLOB_DB_BLOB_FILE_BYTES_READ = 115,
    BLOB_DB_BLOB_FILE_SYNCED = 116,
    BLOB_DB_BLOB_INDEX_EXPIRED_COUNT = 117,
    BLOB_DB_BLOB_INDEX_EXPIRED_SIZE = 118,
    BLOB_DB_BLOB_INDEX_EVICTED_COUNT = 119,
    BLOB_DB_BLOB_INDEX_EVICTED_SIZE = 120,
    BLOB_DB_GC_NUM_FILES = 121,
    BLOB_DB_GC_NUM_NEW_FILES = 122,
    BLOB_DB_GC_FAILURES = 123,
    BLOB_DB_GC_NUM_KEYS_OVERWRITTEN = 124,
    BLOB_DB_GC_NUM_KEYS_EXPIRED = 125,
    BLOB_DB_GC_NUM_KEYS_RELOCATED = 126,
    BLOB_DB_GC_BYTES_OVERWRITTEN = 127,
    BLOB_DB_GC_BYTES_EXPIRED = 128,
    BLOB_DB_GC_BYTES_RELOCATED = 129,
    BLOB_DB_FIFO_NUM_FILES_EVICTED = 130,
    BLOB_DB_FIFO_NUM_KEYS_EVICTED = 131,
    BLOB_DB_FIFO_BYTES_EVICTED = 132,
    TXN_PREPARE_MUTEX_OVERHEAD = 133,
    TXN_OLD_COMMIT_MAP_MUTEX_OVERHEAD = 134,
    TXN_DUPLICATE_KEY_OVERHEAD = 135,
    TXN_SNAPSHOT_MUTEX_OVERHEAD = 136,
    NUMBER_MULTIGET_KEYS_FOUND = 137,
    NO_ITERATOR_CREATED = 138,
    NO_ITERATOR_DELETED = 139,
    BLOCK_CACHE_COMPRESSION_DICT_MISS = 140,
    BLOCK_CACHE_COMPRESSION_DICT_HIT = 141,
    BLOCK_CACHE_COMPRESSION_DICT_ADD = 142,
    BLOCK_CACHE_COMPRESSION_DICT_BYTES_INSERT = 143,
    BLOCK_CACHE_COMPRESSION_DICT_BYTES_EVICT = 144,
    TICKER_ENUM_MAX = 145,
}
#[repr(u32)]
#[doc = " Keep adding histogram's here."]
#[doc = " Any histogram should have value less than HISTOGRAM_ENUM_MAX"]
#[doc = " Add a new Histogram by assigning it the current value of HISTOGRAM_ENUM_MAX"]
#[doc = " Add a string representation in HistogramsNameMap below"]
#[doc = " And increment HISTOGRAM_ENUM_MAX"]
#[doc = " Add a corresponding enum value to HistogramType.java in the java API"]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum rocksdb_Histograms {
    DB_GET = 0,
    DB_WRITE = 1,
    COMPACTION_TIME = 2,
    COMPACTION_CPU_TIME = 3,
    SUBCOMPACTION_SETUP_TIME = 4,
    TABLE_SYNC_MICROS = 5,
    COMPACTION_OUTFILE_SYNC_MICROS = 6,
    WAL_FILE_SYNC_MICROS = 7,
    MANIFEST_FILE_SYNC_MICROS = 8,
    TABLE_OPEN_IO_MICROS = 9,
    DB_MULTIGET = 10,
    READ_BLOCK_COMPACTION_MICROS = 11,
    READ_BLOCK_GET_MICROS = 12,
    WRITE_RAW_BLOCK_MICROS = 13,
    STALL_L0_SLOWDOWN_COUNT = 14,
    STALL_MEMTABLE_COMPACTION_COUNT = 15,
    STALL_L0_NUM_FILES_COUNT = 16,
    HARD_RATE_LIMIT_DELAY_COUNT = 17,
    SOFT_RATE_LIMIT_DELAY_COUNT = 18,
    NUM_FILES_IN_SINGLE_COMPACTION = 19,
    DB_SEEK = 20,
    WRITE_STALL = 21,
    SST_READ_MICROS = 22,
    NUM_SUBCOMPACTIONS_SCHEDULED = 23,
    BYTES_PER_READ = 24,
    BYTES_PER_WRITE = 25,
    BYTES_PER_MULTIGET = 26,
    BYTES_COMPRESSED = 27,
    BYTES_DECOMPRESSED = 28,
    COMPRESSION_TIMES_NANOS = 29,
    DECOMPRESSION_TIMES_NANOS = 30,
    READ_NUM_MERGE_OPERANDS = 31,
    BLOB_DB_KEY_SIZE = 32,
    BLOB_DB_VALUE_SIZE = 33,
    BLOB_DB_WRITE_MICROS = 34,
    BLOB_DB_GET_MICROS = 35,
    BLOB_DB_MULTIGET_MICROS = 36,
    BLOB_DB_SEEK_MICROS = 37,
    BLOB_DB_NEXT_MICROS = 38,
    BLOB_DB_PREV_MICROS = 39,
    BLOB_DB_BLOB_FILE_WRITE_MICROS = 40,
    BLOB_DB_BLOB_FILE_READ_MICROS = 41,
    BLOB_DB_BLOB_FILE_SYNC_MICROS = 42,
    BLOB_DB_GC_MICROS = 43,
    BLOB_DB_COMPRESSION_MICROS = 44,
    BLOB_DB_DECOMPRESSION_MICROS = 45,
    FLUSH_TIME = 46,
    SST_BATCH_SIZE = 47,
    DB_WRITE_WAL_TIME = 48,
    HISTOGRAM_ENUM_MAX = 49,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct rocksdb_Snapshot {
    _unused: [u8; 0],
}
#[repr(u8)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum rocksdb_CompressionType {
    kNoCompression = 0,
    kSnappyCompression = 1,
    kZlibCompression = 2,
    kBZip2Compression = 3,
    kLZ4Compression = 4,
    kLZ4HCCompression = 5,
    kXpressCompression = 6,
    kZSTD = 7,
    kZSTDNotFinalCompression = 64,
    kDisableCompressionOption = 255,
}
#[repr(i8)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum rocksdb_WALRecoveryMode {
    kTolerateCorruptedTailRecords = 0,
    kAbsoluteConsistency = 1,
    kPointInTimeRecovery = 2,
    kSkipAnyCorruptedRecords = 3,
}
#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum rocksdb_ReadTier {
    kReadAllTier = 0,
    kBlockCacheTier = 1,
    kPersistedTier = 2,
    kMemtableTier = 3,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct rocksdb_ReadOptions {
    pub snapshot: *const rocksdb_Snapshot,
    pub iterate_lower_bound: *const rocksdb_Slice,
    pub iterate_upper_bound: *const rocksdb_Slice,
    pub readahead_size: usize,
    pub max_skippable_internal_keys: u64,
    pub read_tier: rocksdb_ReadTier,
    pub verify_checksums: bool,
    pub fill_cache: bool,
    pub tailing: bool,
    pub managed: bool,
    pub total_order_seek: bool,
    pub prefix_same_as_start: bool,
    pub pin_data: bool,
    pub background_purge_on_iterator_cleanup: bool,
    pub ignore_range_deletions: bool,
    pub table_filter: [u64; 4usize],
    pub iter_start_seqnum: rocksdb_SequenceNumber,
    pub timestamp: *const rocksdb_Slice,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct rocksdb_WriteOptions {
    pub sync: bool,
    pub disableWAL: bool,
    pub ignore_missing_column_families: bool,
    pub no_slowdown: bool,
    pub low_pri: bool,
    pub timestamp: *const rocksdb_Slice,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct rocksdb_FlushOptions {
    pub wait: bool,
    pub allow_write_stall: bool,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct rocksdb_CompactionOptions {
    pub compression: rocksdb_CompressionType,
    pub output_file_size_limit: u64,
    pub max_subcompactions: u32,
}
#[repr(i32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum rocksdb_BottommostLevelCompaction {
    kSkip = 0,
    kIfHaveCompactionFilter = 1,
    kForce = 2,
    kForceOptimized = 3,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct rocksdb_CompactRangeOptions {
    pub exclusive_manual_compaction: bool,
    pub change_level: bool,
    pub target_level: libc::c_int,
    pub target_path_id: u32,
    pub bottommost_level_compaction: rocksdb_BottommostLevelCompaction,
    pub allow_write_stall: bool,
    pub max_subcompactions: u32,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct rocksdb_IngestExternalFileOptions {
    pub move_files: bool,
    pub failed_move_fall_back_to_copy: bool,
    pub snapshot_consistency: bool,
    pub allow_global_seqno: bool,
    pub allow_blocking_flush: bool,
    pub ingest_behind: bool,
    pub write_global_seqno: bool,
    pub verify_checksums_before_ingest: bool,
}
#[repr(u8)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum rocksdb_PerfLevel {
    kUninitialized = 0,
    kDisable = 1,
    kEnableCount = 2,
    kEnableTimeExceptForMutex = 3,
    kEnableTimeAndCPUTimeExceptForMutex = 4,
    kEnableTime = 5,
    kOutOfBounds = 6,
}
#[repr(i32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum rocksdb_RateLimiter_OpType {
    kRead = 0,
    kWrite = 1,
}
#[repr(i32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum rocksdb_RateLimiter_Mode {
    kReadsOnly = 0,
    kWritesOnly = 1,
    kAllIo = 2,
}
#[repr(i8)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum rocksdb_BlockBasedTableOptions_IndexType {
    kBinarySearch = 0,
    kHashSearch = 1,
    kTwoLevelIndexSearch = 2,
    kBinarySearchWithFirstKey = 3,
}
#[repr(i32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum rocksdb_titandb_TitanBlobRunMode {
    kNormal = 0,
    kReadOnly = 1,
    kFallback = 2,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct rocksdb_titandb_TitanReadOptions {
    pub _base: rocksdb_ReadOptions,
    pub key_only: bool,
}
#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum rocksdb_titandb_TickerType {
    TITAN_NUM_GET = 145,
    TITAN_NUM_SEEK = 146,
    TITAN_NUM_NEXT = 147,
    TITAN_NUM_PREV = 148,
    TITAN_BLOB_FILE_NUM_KEYS_WRITTEN = 149,
    TITAN_BLOB_FILE_NUM_KEYS_READ = 150,
    TITAN_BLOB_FILE_BYTES_WRITTEN = 151,
    TITAN_BLOB_FILE_BYTES_READ = 152,
    TITAN_BLOB_FILE_SYNCED = 153,
    TITAN_GC_NUM_FILES = 154,
    TITAN_GC_NUM_NEW_FILES = 155,
    TITAN_GC_NUM_KEYS_OVERWRITTEN = 156,
    TITAN_GC_NUM_KEYS_RELOCATED = 157,
    TITAN_GC_BYTES_OVERWRITTEN = 158,
    TITAN_GC_BYTES_RELOCATED = 159,
    TITAN_GC_BYTES_WRITTEN = 160,
    TITAN_GC_BYTES_READ = 161,
    TITAN_BLOB_CACHE_HIT = 162,
    TITAN_BLOB_CACHE_MISS = 163,
    TITAN_GC_NO_NEED = 164,
    TITAN_GC_REMAIN = 165,
    TITAN_GC_DISCARDABLE = 166,
    TITAN_GC_SAMPLE = 167,
    TITAN_GC_SMALL_FILE = 168,
    TITAN_GC_FAILURE = 169,
    TITAN_GC_SUCCESS = 170,
    TITAN_GC_TRIGGER_NEXT = 171,
    TITAN_TICKER_ENUM_MAX = 172,
}
#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum rocksdb_titandb_HistogramType {
    TITAN_KEY_SIZE = 49,
    TITAN_VALUE_SIZE = 50,
    TITAN_GET_MICROS = 51,
    TITAN_SEEK_MICROS = 52,
    TITAN_NEXT_MICROS = 53,
    TITAN_PREV_MICROS = 54,
    TITAN_BLOB_FILE_WRITE_MICROS = 55,
    TITAN_BLOB_FILE_READ_MICROS = 56,
    TITAN_BLOB_FILE_SYNC_MICROS = 57,
    TITAN_MANIFEST_FILE_SYNC_MICROS = 58,
    TITAN_GC_MICROS = 59,
    TITAN_GC_INPUT_FILE_SIZE = 60,
    TITAN_GC_OUTPUT_FILE_SIZE = 61,
    TITAN_ITER_TOUCH_BLOB_FILE_COUNT = 62,
    TITAN_HISTOGRAM_ENUM_MAX = 63,
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_cloud_envoptions_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_backup_engine_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_backup_engine_info_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_restore_options_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_lru_cache_options_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_cache_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_memory_allocator_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_compactionfilter_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_compactionfiltercontext_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_compactionfilterfactory_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_comparator_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_fifo_compaction_options_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_filelock_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_filterpolicy_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_iterator_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_logger_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_logger_impl_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_mergeoperator_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_options_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_column_family_descriptor {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_block_based_table_options_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_cuckoo_table_options_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_randomfile_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_seqfile_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_slicetransform_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_snapshot_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_writablefile_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_writebatch_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_universal_compaction_options_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_livefiles_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_column_family_handle_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_envoptions_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_sequential_file_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_sstfilereader_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_sstfilewriter_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_externalsstfileinfo_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_ratelimiter_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_pinnableslice_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_user_collected_properties_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_user_collected_properties_iterator_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_table_properties_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_table_properties_collection_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_table_properties_collection_iterator_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_table_properties_collector_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_table_properties_collector_factory_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_flushjobinfo_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_compactionjobinfo_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_subcompactionjobinfo_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_externalfileingestioninfo_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_eventlistener_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_keyversions_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_column_family_meta_data_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_level_meta_data_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_sst_file_meta_data_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_perf_context_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_iostats_context_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_writestallinfo_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_writestallcondition_t {
    _unused: [u8; 0],
}
extern "C" {
    pub static crocksdb_property_name_num_files_at_level_prefix: rocksdb_Slice;
}
extern "C" {
    pub static crocksdb_property_name_compression_ratio_at_level_prefix: rocksdb_Slice;
}
extern "C" {
    pub static crocksdb_property_name_stats: rocksdb_Slice;
}
extern "C" {
    pub static crocksdb_property_name_ss_tables: rocksdb_Slice;
}
extern "C" {
    pub static crocksdb_property_name_cf_stats: rocksdb_Slice;
}
extern "C" {
    pub static crocksdb_property_name_cf_stats_no_file_histogram: rocksdb_Slice;
}
extern "C" {
    pub static crocksdb_property_name_cf_file_histogram: rocksdb_Slice;
}
extern "C" {
    pub static crocksdb_property_name_db_stats: rocksdb_Slice;
}
extern "C" {
    pub static crocksdb_property_name_level_stats: rocksdb_Slice;
}
extern "C" {
    pub static crocksdb_property_name_num_immutable_mem_table: rocksdb_Slice;
}
extern "C" {
    pub static crocksdb_property_name_num_immutable_mem_table_flushed: rocksdb_Slice;
}
extern "C" {
    pub static crocksdb_property_name_mem_table_flush_pending: rocksdb_Slice;
}
extern "C" {
    pub static crocksdb_property_name_num_running_flushes: rocksdb_Slice;
}
extern "C" {
    pub static crocksdb_property_name_compaction_pending: rocksdb_Slice;
}
extern "C" {
    pub static crocksdb_property_name_num_running_compactions: rocksdb_Slice;
}
extern "C" {
    pub static crocksdb_property_name_background_errors: rocksdb_Slice;
}
extern "C" {
    pub static crocksdb_property_name_cur_size_active_mem_table: rocksdb_Slice;
}
extern "C" {
    pub static crocksdb_property_name_cur_size_all_mem_tables: rocksdb_Slice;
}
extern "C" {
    pub static crocksdb_property_name_size_all_mem_tables: rocksdb_Slice;
}
extern "C" {
    pub static crocksdb_property_name_num_entries_active_mem_table: rocksdb_Slice;
}
extern "C" {
    pub static crocksdb_property_name_num_entries_imm_mem_tables: rocksdb_Slice;
}
extern "C" {
    pub static crocksdb_property_name_num_deletes_active_mem_table: rocksdb_Slice;
}
extern "C" {
    pub static crocksdb_property_name_num_deletes_imm_mem_tables: rocksdb_Slice;
}
extern "C" {
    pub static crocksdb_property_name_estimate_num_keys: rocksdb_Slice;
}
extern "C" {
    pub static crocksdb_property_name_estimate_table_readers_mem: rocksdb_Slice;
}
extern "C" {
    pub static crocksdb_property_name_is_file_deletions_enabled: rocksdb_Slice;
}
extern "C" {
    pub static crocksdb_property_name_num_snapshots: rocksdb_Slice;
}
extern "C" {
    pub static crocksdb_property_name_oldest_snapshot_time: rocksdb_Slice;
}
extern "C" {
    pub static crocksdb_property_name_oldest_snapshot_sequence: rocksdb_Slice;
}
extern "C" {
    pub static crocksdb_property_name_num_live_versions: rocksdb_Slice;
}
extern "C" {
    pub static crocksdb_property_name_current_super_version_number: rocksdb_Slice;
}
extern "C" {
    pub static crocksdb_property_name_estimate_live_data_size: rocksdb_Slice;
}
extern "C" {
    pub static crocksdb_property_name_min_log_number_to_keep: rocksdb_Slice;
}
extern "C" {
    pub static crocksdb_property_name_min_obsolete_sst_number_to_keep: rocksdb_Slice;
}
extern "C" {
    pub static crocksdb_property_name_total_sst_files_size: rocksdb_Slice;
}
extern "C" {
    pub static crocksdb_property_name_live_sst_files_size: rocksdb_Slice;
}
extern "C" {
    pub static crocksdb_property_name_base_level: rocksdb_Slice;
}
extern "C" {
    pub static crocksdb_property_name_estimate_pending_compaction_bytes: rocksdb_Slice;
}
extern "C" {
    pub static crocksdb_property_name_aggregated_table_properties: rocksdb_Slice;
}
extern "C" {
    pub static crocksdb_property_name_aggregated_table_properties_at_level: rocksdb_Slice;
}
extern "C" {
    pub static crocksdb_property_name_actual_delayed_write_rate: rocksdb_Slice;
}
extern "C" {
    pub static crocksdb_property_name_is_write_stopped: rocksdb_Slice;
}
extern "C" {
    pub static crocksdb_property_name_is_write_stalled: rocksdb_Slice;
}
extern "C" {
    pub static crocksdb_property_name_estimate_oldest_key_time: rocksdb_Slice;
}
extern "C" {
    pub static crocksdb_property_name_block_cache_capacity: rocksdb_Slice;
}
extern "C" {
    pub static crocksdb_property_name_block_cache_usage: rocksdb_Slice;
}
extern "C" {
    pub static crocksdb_property_name_block_cache_pinned_usage: rocksdb_Slice;
}
extern "C" {
    pub static crocksdb_property_name_options_statistics: rocksdb_Slice;
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_map_property_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_writebatch_iterator_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_sst_partitioner_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_sst_partitioner_request_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_sst_partitioner_context_t {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_sst_partitioner_factory_t {
    _unused: [u8; 0],
}
#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum crocksdb_table_property_t {
    kDataSize = 1,
    kIndexSize = 2,
    kFilterSize = 3,
    kRawKeySize = 4,
    kRawValueSize = 5,
    kNumDataBlocks = 6,
    kNumEntries = 7,
    kFormatVersion = 8,
    kFixedKeyLen = 9,
    kColumnFamilyID = 10,
    kColumnFamilyName = 11,
    kFilterPolicyName = 12,
    kComparatorName = 13,
    kMergeOperatorName = 14,
    kPrefixExtractorName = 15,
    kPropertyCollectorsNames = 16,
    kCompressionName = 17,
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_file_encryption_info_t {
    pub method: rocksdb_encryption_EncryptionMethod,
    pub key: *const libc::c_char,
    pub key_len: usize,
    pub iv: *const libc::c_char,
    pub iv_len: usize,
}
#[repr(C)]
#[derive(Debug)]
pub struct crocksdb_file_system_inspector_t {
    _unused: [u8; 0],
}
extern "C" {
    pub fn crocksdb_open(
        options: *const crocksdb_options_t,
        name: rocksdb_Slice,
        s: *mut rocksdb_Status,
    ) -> *mut crocksdb_t;
}
extern "C" {
    pub fn crocksdb_open_with_ttl(
        options: *const crocksdb_options_t,
        name: rocksdb_Slice,
        ttl: libc::c_int,
        s: *mut rocksdb_Status,
    ) -> *mut crocksdb_t;
}
extern "C" {
    pub fn crocksdb_open_for_read_only(
        options: *const crocksdb_options_t,
        name: rocksdb_Slice,
        error_if_log_file_exist: libc::c_uchar,
        s: *mut rocksdb_Status,
    ) -> *mut crocksdb_t;
}
extern "C" {
    pub fn crocksdb_backup_engine_open(
        options: *const crocksdb_options_t,
        path: rocksdb_Slice,
        s: *mut rocksdb_Status,
    ) -> *mut crocksdb_backup_engine_t;
}
extern "C" {
    pub fn crocksdb_backup_engine_create_new_backup(
        be: *mut crocksdb_backup_engine_t,
        db: *mut crocksdb_t,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_backup_engine_purge_old_backups(
        be: *mut crocksdb_backup_engine_t,
        num_backups_to_keep: u32,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_restore_options_create() -> *mut crocksdb_restore_options_t;
}
extern "C" {
    pub fn crocksdb_restore_options_destroy(opt: *mut crocksdb_restore_options_t);
}
extern "C" {
    pub fn crocksdb_restore_options_set_keep_log_files(
        opt: *mut crocksdb_restore_options_t,
        v: libc::c_int,
    );
}
extern "C" {
    pub fn crocksdb_backup_engine_restore_db_from_latest_backup(
        be: *mut crocksdb_backup_engine_t,
        db_dir: rocksdb_Slice,
        wal_dir: rocksdb_Slice,
        restore_options: *const crocksdb_restore_options_t,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_backup_engine_get_backup_info(
        be: *mut crocksdb_backup_engine_t,
    ) -> *const crocksdb_backup_engine_info_t;
}
extern "C" {
    pub fn crocksdb_backup_engine_info_count(
        info: *const crocksdb_backup_engine_info_t,
    ) -> libc::c_int;
}
extern "C" {
    pub fn crocksdb_backup_engine_info_timestamp(
        info: *const crocksdb_backup_engine_info_t,
        index: libc::c_int,
    ) -> i64;
}
extern "C" {
    pub fn crocksdb_backup_engine_info_backup_id(
        info: *const crocksdb_backup_engine_info_t,
        index: libc::c_int,
    ) -> u32;
}
extern "C" {
    pub fn crocksdb_backup_engine_info_size(
        info: *const crocksdb_backup_engine_info_t,
        index: libc::c_int,
    ) -> u64;
}
extern "C" {
    pub fn crocksdb_backup_engine_info_number_files(
        info: *const crocksdb_backup_engine_info_t,
        index: libc::c_int,
    ) -> u32;
}
extern "C" {
    pub fn crocksdb_backup_engine_info_destroy(info: *const crocksdb_backup_engine_info_t);
}
extern "C" {
    pub fn crocksdb_backup_engine_close(be: *mut crocksdb_backup_engine_t);
}
extern "C" {
    pub fn crocksdb_open_column_families(
        options: *const crocksdb_options_t,
        name: rocksdb_Slice,
        num_column_families: libc::c_int,
        column_family_names: *mut rocksdb_Slice,
        column_family_options: *mut *const crocksdb_options_t,
        column_family_handles: *mut *mut crocksdb_column_family_handle_t,
        s: *mut rocksdb_Status,
    ) -> *mut crocksdb_t;
}
extern "C" {
    pub fn crocksdb_open_column_families_with_ttl(
        options: *const crocksdb_options_t,
        name: rocksdb_Slice,
        num_column_families: libc::c_int,
        column_family_names: *mut rocksdb_Slice,
        column_family_options: *mut *const crocksdb_options_t,
        ttl_array: *const i32,
        read_only: libc::c_uchar,
        column_family_handles: *mut *mut crocksdb_column_family_handle_t,
        s: *mut rocksdb_Status,
    ) -> *mut crocksdb_t;
}
extern "C" {
    pub fn crocksdb_open_for_read_only_column_families(
        options: *const crocksdb_options_t,
        name: rocksdb_Slice,
        num_column_families: libc::c_int,
        column_family_names: *mut rocksdb_Slice,
        column_family_options: *mut *const crocksdb_options_t,
        column_family_handles: *mut *mut crocksdb_column_family_handle_t,
        error_if_log_file_exist: libc::c_uchar,
        s: *mut rocksdb_Status,
    ) -> *mut crocksdb_t;
}
extern "C" {
    pub fn crocksdb_list_column_families(
        options: *const crocksdb_options_t,
        name: rocksdb_Slice,
        lencf: *mut usize,
        s: *mut rocksdb_Status,
    ) -> *mut *mut libc::c_char;
}
extern "C" {
    pub fn crocksdb_list_column_families_destroy(list: *mut *mut libc::c_char, len: usize);
}
extern "C" {
    pub fn crocksdb_create_column_family(
        db: *mut crocksdb_t,
        column_family_options: *const crocksdb_options_t,
        column_family_name: *const libc::c_char,
        s: *mut rocksdb_Status,
    ) -> *mut crocksdb_column_family_handle_t;
}
extern "C" {
    pub fn crocksdb_drop_column_family(
        db: *mut crocksdb_t,
        handle: *mut crocksdb_column_family_handle_t,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_column_family_handle_id(arg1: *mut crocksdb_column_family_handle_t) -> u32;
}
extern "C" {
    pub fn crocksdb_column_family_handle_destroy(arg1: *mut crocksdb_column_family_handle_t);
}
extern "C" {
    pub fn crocksdb_close(db: *mut crocksdb_t);
}
extern "C" {
    pub fn crocksdb_pause_bg_work(db: *mut crocksdb_t);
}
extern "C" {
    pub fn crocksdb_continue_bg_work(db: *mut crocksdb_t);
}
extern "C" {
    pub fn crocksdb_put(
        db: *mut crocksdb_t,
        options: *const rocksdb_WriteOptions,
        key: *const libc::c_char,
        keylen: usize,
        val: *const libc::c_char,
        vallen: usize,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_put_cf(
        db: *mut crocksdb_t,
        options: *const rocksdb_WriteOptions,
        column_family: *mut crocksdb_column_family_handle_t,
        key: *const libc::c_char,
        keylen: usize,
        val: *const libc::c_char,
        vallen: usize,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_delete(
        db: *mut crocksdb_t,
        options: *const rocksdb_WriteOptions,
        key: *const libc::c_char,
        keylen: usize,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_delete_cf(
        db: *mut crocksdb_t,
        options: *const rocksdb_WriteOptions,
        column_family: *mut crocksdb_column_family_handle_t,
        key: *const libc::c_char,
        keylen: usize,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_single_delete(
        db: *mut crocksdb_t,
        options: *const rocksdb_WriteOptions,
        key: *const libc::c_char,
        keylen: usize,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_single_delete_cf(
        db: *mut crocksdb_t,
        options: *const rocksdb_WriteOptions,
        column_family: *mut crocksdb_column_family_handle_t,
        key: *const libc::c_char,
        keylen: usize,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_delete_range_cf(
        db: *mut crocksdb_t,
        options: *const rocksdb_WriteOptions,
        column_family: *mut crocksdb_column_family_handle_t,
        begin_key: *const libc::c_char,
        begin_keylen: usize,
        end_key: *const libc::c_char,
        end_keylen: usize,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_merge(
        db: *mut crocksdb_t,
        options: *const rocksdb_WriteOptions,
        key: *const libc::c_char,
        keylen: usize,
        val: *const libc::c_char,
        vallen: usize,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_merge_cf(
        db: *mut crocksdb_t,
        options: *const rocksdb_WriteOptions,
        column_family: *mut crocksdb_column_family_handle_t,
        key: *const libc::c_char,
        keylen: usize,
        val: *const libc::c_char,
        vallen: usize,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_write(
        db: *mut crocksdb_t,
        options: *const rocksdb_WriteOptions,
        batch: *mut crocksdb_writebatch_t,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_write_multi_batch(
        db: *mut crocksdb_t,
        options: *const rocksdb_WriteOptions,
        batches: *mut *mut crocksdb_writebatch_t,
        batch_size: usize,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_get(
        db: *mut crocksdb_t,
        options: *const rocksdb_ReadOptions,
        key: *const libc::c_char,
        keylen: usize,
        vallen: *mut usize,
        s: *mut rocksdb_Status,
    ) -> *mut libc::c_char;
}
extern "C" {
    pub fn crocksdb_get_cf(
        db: *mut crocksdb_t,
        options: *const rocksdb_ReadOptions,
        column_family: *mut crocksdb_column_family_handle_t,
        key: *const libc::c_char,
        keylen: usize,
        vallen: *mut usize,
        s: *mut rocksdb_Status,
    ) -> *mut libc::c_char;
}
extern "C" {
    pub fn crocksdb_multi_get(
        db: *mut crocksdb_t,
        options: *const rocksdb_ReadOptions,
        num_keys: usize,
        keys_list: *const *const libc::c_char,
        keys_list_sizes: *const usize,
        values_list: *mut *mut libc::c_char,
        values_list_sizes: *mut usize,
        statuses: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_multi_get_cf(
        db: *mut crocksdb_t,
        options: *const rocksdb_ReadOptions,
        column_families: *const *const crocksdb_column_family_handle_t,
        num_keys: usize,
        keys_list: *const *const libc::c_char,
        keys_list_sizes: *const usize,
        values_list: *mut *mut libc::c_char,
        values_list_sizes: *mut usize,
        statuses: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_create_iterator(
        db: *mut crocksdb_t,
        options: *const rocksdb_ReadOptions,
    ) -> *mut crocksdb_iterator_t;
}
extern "C" {
    pub fn crocksdb_create_iterator_cf(
        db: *mut crocksdb_t,
        options: *const rocksdb_ReadOptions,
        column_family: *mut crocksdb_column_family_handle_t,
    ) -> *mut crocksdb_iterator_t;
}
extern "C" {
    pub fn crocksdb_create_iterators(
        db: *mut crocksdb_t,
        opts: *const rocksdb_ReadOptions,
        column_families: *mut *mut crocksdb_column_family_handle_t,
        iterators: *mut *mut crocksdb_iterator_t,
        size: usize,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_create_snapshot(db: *mut crocksdb_t) -> *const crocksdb_snapshot_t;
}
extern "C" {
    pub fn crocksdb_release_snapshot(db: *mut crocksdb_t, snapshot: *const crocksdb_snapshot_t);
}
extern "C" {
    pub fn crocksdb_get_snapshot_sequence_number(snapshot: *const crocksdb_snapshot_t) -> u64;
}
extern "C" {
    pub fn crocksdb_create_map_property() -> *mut crocksdb_map_property_t;
}
extern "C" {
    pub fn crocksdb_destroy_map_property(info: *mut crocksdb_map_property_t);
}
extern "C" {
    pub fn crocksdb_get_map_property_cf(
        db: *mut crocksdb_t,
        column_family: *mut crocksdb_column_family_handle_t,
        property: *const libc::c_char,
        data: *mut crocksdb_map_property_t,
    ) -> libc::c_uchar;
}
extern "C" {
    pub fn crocksdb_map_property_value(
        info: *mut crocksdb_map_property_t,
        propname: *const libc::c_char,
    ) -> *mut libc::c_char;
}
extern "C" {
    pub fn crocksdb_map_property_int_value(
        info: *mut crocksdb_map_property_t,
        propname: *const libc::c_char,
    ) -> u64;
}
extern "C" {
    pub fn crocksdb_property_value(
        db: *mut crocksdb_t,
        propname: *const libc::c_char,
    ) -> *mut libc::c_char;
}
extern "C" {
    pub fn crocksdb_property_value_cf(
        db: *mut crocksdb_t,
        column_family: *mut crocksdb_column_family_handle_t,
        propname: *const libc::c_char,
    ) -> *mut libc::c_char;
}
extern "C" {
    pub fn crocksdb_approximate_sizes(
        db: *mut crocksdb_t,
        num_ranges: libc::c_int,
        range_start_key: *const *const libc::c_char,
        range_start_key_len: *const usize,
        range_limit_key: *const *const libc::c_char,
        range_limit_key_len: *const usize,
        sizes: *mut u64,
    );
}
extern "C" {
    pub fn crocksdb_approximate_sizes_cf(
        db: *mut crocksdb_t,
        column_family: *mut crocksdb_column_family_handle_t,
        num_ranges: libc::c_int,
        range_start_key: *const *const libc::c_char,
        range_start_key_len: *const usize,
        range_limit_key: *const *const libc::c_char,
        range_limit_key_len: *const usize,
        sizes: *mut u64,
    );
}
extern "C" {
    pub fn crocksdb_approximate_memtable_stats(
        db: *const crocksdb_t,
        range_start_key: *const libc::c_char,
        range_start_key_len: usize,
        range_limit_key: *const libc::c_char,
        range_limit_key_len: usize,
        count: *mut u64,
        size: *mut u64,
    );
}
extern "C" {
    pub fn crocksdb_approximate_memtable_stats_cf(
        db: *const crocksdb_t,
        cf: *const crocksdb_column_family_handle_t,
        range_start_key: *const libc::c_char,
        range_start_key_len: usize,
        range_limit_key: *const libc::c_char,
        range_limit_key_len: usize,
        count: *mut u64,
        size: *mut u64,
    );
}
extern "C" {
    pub fn crocksdb_compact_range(
        db: *mut crocksdb_t,
        start_key: *const libc::c_char,
        start_key_len: usize,
        limit_key: *const libc::c_char,
        limit_key_len: usize,
    );
}
extern "C" {
    pub fn crocksdb_compact_range_cf(
        db: *mut crocksdb_t,
        column_family: *mut crocksdb_column_family_handle_t,
        start_key: *const libc::c_char,
        start_key_len: usize,
        limit_key: *const libc::c_char,
        limit_key_len: usize,
    );
}
extern "C" {
    pub fn crocksdb_compact_range_opt(
        db: *mut crocksdb_t,
        opt: *const rocksdb_CompactRangeOptions,
        start_key: *const libc::c_char,
        start_key_len: usize,
        limit_key: *const libc::c_char,
        limit_key_len: usize,
    );
}
extern "C" {
    pub fn crocksdb_compact_range_cf_opt(
        db: *mut crocksdb_t,
        column_family: *mut crocksdb_column_family_handle_t,
        opt: *const rocksdb_CompactRangeOptions,
        start_key: *const libc::c_char,
        start_key_len: usize,
        limit_key: *const libc::c_char,
        limit_key_len: usize,
    );
}
extern "C" {
    pub fn crocksdb_delete_file(
        db: *mut crocksdb_t,
        name: *const libc::c_char,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_livefiles(db: *mut crocksdb_t) -> *const crocksdb_livefiles_t;
}
extern "C" {
    pub fn crocksdb_flush(
        db: *mut crocksdb_t,
        options: *const rocksdb_FlushOptions,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_flush_cf(
        db: *mut crocksdb_t,
        column_family: *mut crocksdb_column_family_handle_t,
        options: *const rocksdb_FlushOptions,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_flush_cfs(
        db: *mut crocksdb_t,
        column_familys: *mut *const crocksdb_column_family_handle_t,
        num_handles: libc::c_int,
        options: *const rocksdb_FlushOptions,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_flush_wal(db: *mut crocksdb_t, sync: libc::c_uchar, s: *mut rocksdb_Status);
}
extern "C" {
    pub fn crocksdb_sync_wal(db: *mut crocksdb_t, s: *mut rocksdb_Status);
}
extern "C" {
    pub fn crocksdb_get_latest_sequence_number(db: *mut crocksdb_t) -> u64;
}
extern "C" {
    pub fn crocksdb_disable_file_deletions(db: *mut crocksdb_t, s: *mut rocksdb_Status);
}
extern "C" {
    pub fn crocksdb_enable_file_deletions(
        db: *mut crocksdb_t,
        force: libc::c_uchar,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_get_db_options(db: *mut crocksdb_t) -> *mut crocksdb_options_t;
}
extern "C" {
    pub fn crocksdb_set_db_options(
        db: *mut crocksdb_t,
        names: *mut *const libc::c_char,
        values: *mut *const libc::c_char,
        num_options: usize,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_get_options_cf(
        db: *const crocksdb_t,
        column_family: *mut crocksdb_column_family_handle_t,
    ) -> *mut crocksdb_options_t;
}
extern "C" {
    pub fn crocksdb_set_options_cf(
        db: *mut crocksdb_t,
        cf: *mut crocksdb_column_family_handle_t,
        names: *mut *const libc::c_char,
        values: *mut *const libc::c_char,
        num_options: usize,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_destroy_db(
        options: *const crocksdb_options_t,
        name: *const libc::c_char,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_repair_db(
        options: *const crocksdb_options_t,
        name: *const libc::c_char,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_iter_destroy(arg1: *mut crocksdb_iterator_t);
}
extern "C" {
    pub fn crocksdb_iter_valid(arg1: *const crocksdb_iterator_t) -> libc::c_uchar;
}
extern "C" {
    pub fn crocksdb_iter_seek_to_first(arg1: *mut crocksdb_iterator_t);
}
extern "C" {
    pub fn crocksdb_iter_seek_to_last(arg1: *mut crocksdb_iterator_t);
}
extern "C" {
    pub fn crocksdb_iter_seek(arg1: *mut crocksdb_iterator_t, k: *const libc::c_char, klen: usize);
}
extern "C" {
    pub fn crocksdb_iter_seek_for_prev(
        arg1: *mut crocksdb_iterator_t,
        k: *const libc::c_char,
        klen: usize,
    );
}
extern "C" {
    pub fn crocksdb_iter_next(arg1: *mut crocksdb_iterator_t);
}
extern "C" {
    pub fn crocksdb_iter_prev(arg1: *mut crocksdb_iterator_t);
}
extern "C" {
    pub fn crocksdb_iter_key(
        arg1: *const crocksdb_iterator_t,
        klen: *mut usize,
    ) -> *const libc::c_char;
}
extern "C" {
    pub fn crocksdb_iter_value(
        arg1: *const crocksdb_iterator_t,
        vlen: *mut usize,
    ) -> *const libc::c_char;
}
extern "C" {
    pub fn crocksdb_iter_get_error(arg1: *const crocksdb_iterator_t, s: *mut rocksdb_Status);
}
extern "C" {
    pub fn crocksdb_writebatch_create() -> *mut crocksdb_writebatch_t;
}
extern "C" {
    pub fn crocksdb_writebatch_create_with_capacity(
        reserved_bytes: usize,
    ) -> *mut crocksdb_writebatch_t;
}
extern "C" {
    pub fn crocksdb_writebatch_create_from(
        rep: *const libc::c_char,
        size: usize,
    ) -> *mut crocksdb_writebatch_t;
}
extern "C" {
    pub fn crocksdb_writebatch_destroy(arg1: *mut crocksdb_writebatch_t);
}
extern "C" {
    pub fn crocksdb_writebatch_clear(arg1: *mut crocksdb_writebatch_t);
}
extern "C" {
    pub fn crocksdb_writebatch_count(arg1: *mut crocksdb_writebatch_t) -> libc::c_int;
}
extern "C" {
    pub fn crocksdb_writebatch_put(
        arg1: *mut crocksdb_writebatch_t,
        key: *const libc::c_char,
        klen: usize,
        val: *const libc::c_char,
        vlen: usize,
    );
}
extern "C" {
    pub fn crocksdb_writebatch_put_cf(
        arg1: *mut crocksdb_writebatch_t,
        column_family: *mut crocksdb_column_family_handle_t,
        key: *const libc::c_char,
        klen: usize,
        val: *const libc::c_char,
        vlen: usize,
    );
}
extern "C" {
    pub fn crocksdb_writebatch_putv(
        b: *mut crocksdb_writebatch_t,
        num_keys: libc::c_int,
        keys_list: *const *const libc::c_char,
        keys_list_sizes: *const usize,
        num_values: libc::c_int,
        values_list: *const *const libc::c_char,
        values_list_sizes: *const usize,
    );
}
extern "C" {
    pub fn crocksdb_writebatch_putv_cf(
        b: *mut crocksdb_writebatch_t,
        column_family: *mut crocksdb_column_family_handle_t,
        num_keys: libc::c_int,
        keys_list: *const *const libc::c_char,
        keys_list_sizes: *const usize,
        num_values: libc::c_int,
        values_list: *const *const libc::c_char,
        values_list_sizes: *const usize,
    );
}
extern "C" {
    pub fn crocksdb_writebatch_merge(
        arg1: *mut crocksdb_writebatch_t,
        key: *const libc::c_char,
        klen: usize,
        val: *const libc::c_char,
        vlen: usize,
    );
}
extern "C" {
    pub fn crocksdb_writebatch_merge_cf(
        arg1: *mut crocksdb_writebatch_t,
        column_family: *mut crocksdb_column_family_handle_t,
        key: *const libc::c_char,
        klen: usize,
        val: *const libc::c_char,
        vlen: usize,
    );
}
extern "C" {
    pub fn crocksdb_writebatch_mergev(
        b: *mut crocksdb_writebatch_t,
        num_keys: libc::c_int,
        keys_list: *const *const libc::c_char,
        keys_list_sizes: *const usize,
        num_values: libc::c_int,
        values_list: *const *const libc::c_char,
        values_list_sizes: *const usize,
    );
}
extern "C" {
    pub fn crocksdb_writebatch_mergev_cf(
        b: *mut crocksdb_writebatch_t,
        column_family: *mut crocksdb_column_family_handle_t,
        num_keys: libc::c_int,
        keys_list: *const *const libc::c_char,
        keys_list_sizes: *const usize,
        num_values: libc::c_int,
        values_list: *const *const libc::c_char,
        values_list_sizes: *const usize,
    );
}
extern "C" {
    pub fn crocksdb_writebatch_delete(
        arg1: *mut crocksdb_writebatch_t,
        key: *const libc::c_char,
        klen: usize,
    );
}
extern "C" {
    pub fn crocksdb_writebatch_delete_cf(
        arg1: *mut crocksdb_writebatch_t,
        column_family: *mut crocksdb_column_family_handle_t,
        key: *const libc::c_char,
        klen: usize,
    );
}
extern "C" {
    pub fn crocksdb_writebatch_single_delete(
        arg1: *mut crocksdb_writebatch_t,
        key: *const libc::c_char,
        klen: usize,
    );
}
extern "C" {
    pub fn crocksdb_writebatch_single_delete_cf(
        arg1: *mut crocksdb_writebatch_t,
        column_family: *mut crocksdb_column_family_handle_t,
        key: *const libc::c_char,
        klen: usize,
    );
}
extern "C" {
    pub fn crocksdb_writebatch_deletev(
        b: *mut crocksdb_writebatch_t,
        num_keys: libc::c_int,
        keys_list: *const *const libc::c_char,
        keys_list_sizes: *const usize,
    );
}
extern "C" {
    pub fn crocksdb_writebatch_deletev_cf(
        b: *mut crocksdb_writebatch_t,
        column_family: *mut crocksdb_column_family_handle_t,
        num_keys: libc::c_int,
        keys_list: *const *const libc::c_char,
        keys_list_sizes: *const usize,
    );
}
extern "C" {
    pub fn crocksdb_writebatch_delete_range(
        b: *mut crocksdb_writebatch_t,
        start_key: *const libc::c_char,
        start_key_len: usize,
        end_key: *const libc::c_char,
        end_key_len: usize,
    );
}
extern "C" {
    pub fn crocksdb_writebatch_delete_range_cf(
        b: *mut crocksdb_writebatch_t,
        column_family: *mut crocksdb_column_family_handle_t,
        start_key: *const libc::c_char,
        start_key_len: usize,
        end_key: *const libc::c_char,
        end_key_len: usize,
    );
}
extern "C" {
    pub fn crocksdb_writebatch_delete_rangev(
        b: *mut crocksdb_writebatch_t,
        num_keys: libc::c_int,
        start_keys_list: *const *const libc::c_char,
        start_keys_list_sizes: *const usize,
        end_keys_list: *const *const libc::c_char,
        end_keys_list_sizes: *const usize,
    );
}
extern "C" {
    pub fn crocksdb_writebatch_delete_rangev_cf(
        b: *mut crocksdb_writebatch_t,
        column_family: *mut crocksdb_column_family_handle_t,
        num_keys: libc::c_int,
        start_keys_list: *const *const libc::c_char,
        start_keys_list_sizes: *const usize,
        end_keys_list: *const *const libc::c_char,
        end_keys_list_sizes: *const usize,
    );
}
extern "C" {
    pub fn crocksdb_writebatch_put_log_data(
        arg1: *mut crocksdb_writebatch_t,
        blob: *const libc::c_char,
        len: usize,
    );
}
extern "C" {
    pub fn crocksdb_writebatch_iterate(
        arg1: *mut crocksdb_writebatch_t,
        state: *mut libc::c_void,
        put: ::std::option::Option<
            unsafe extern "C" fn(
                arg1: *mut libc::c_void,
                k: *const libc::c_char,
                klen: usize,
                v: *const libc::c_char,
                vlen: usize,
            ),
        >,
        deleted: ::std::option::Option<
            unsafe extern "C" fn(arg1: *mut libc::c_void, k: *const libc::c_char, klen: usize),
        >,
    );
}
extern "C" {
    pub fn crocksdb_writebatch_iterate_cf(
        b: *mut crocksdb_writebatch_t,
        state: *mut libc::c_void,
        put: ::std::option::Option<
            unsafe extern "C" fn(
                arg1: *mut libc::c_void,
                k: *const libc::c_char,
                klen: usize,
                v: *const libc::c_char,
                vlen: usize,
            ),
        >,
        put_cf: ::std::option::Option<
            unsafe extern "C" fn(
                arg1: *mut libc::c_void,
                cf: u32,
                k: *const libc::c_char,
                klen: usize,
                v: *const libc::c_char,
                vlen: usize,
            ),
        >,
        deleted: ::std::option::Option<
            unsafe extern "C" fn(arg1: *mut libc::c_void, k: *const libc::c_char, klen: usize),
        >,
        deleted_cf: ::std::option::Option<
            unsafe extern "C" fn(
                arg1: *mut libc::c_void,
                cf: u32,
                k: *const libc::c_char,
                klen: usize,
            ),
        >,
    );
}
extern "C" {
    pub fn crocksdb_writebatch_data(
        arg1: *mut crocksdb_writebatch_t,
        size: *mut usize,
    ) -> *const libc::c_char;
}
extern "C" {
    pub fn crocksdb_writebatch_set_save_point(arg1: *mut crocksdb_writebatch_t);
}
extern "C" {
    pub fn crocksdb_writebatch_pop_save_point(
        arg1: *mut crocksdb_writebatch_t,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_writebatch_rollback_to_save_point(
        arg1: *mut crocksdb_writebatch_t,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_writebatch_set_content(
        b: *mut crocksdb_writebatch_t,
        data: *const libc::c_char,
        dlen: usize,
    );
}
extern "C" {
    pub fn crocksdb_writebatch_append_content(
        dest: *mut crocksdb_writebatch_t,
        data: *const libc::c_char,
        dlen: usize,
    );
}
extern "C" {
    pub fn crocksdb_writebatch_ref_count(data: *const libc::c_char, dlen: usize) -> libc::c_int;
}
extern "C" {
    pub fn crocksdb_writebatch_ref_iterator_create(
        data: *const libc::c_char,
        dlen: usize,
    ) -> *mut crocksdb_writebatch_iterator_t;
}
extern "C" {
    pub fn crocksdb_writebatch_iterator_create(
        dest: *mut crocksdb_writebatch_t,
    ) -> *mut crocksdb_writebatch_iterator_t;
}
extern "C" {
    pub fn crocksdb_writebatch_iterator_destroy(it: *mut crocksdb_writebatch_iterator_t);
}
extern "C" {
    pub fn crocksdb_writebatch_iterator_valid(
        it: *mut crocksdb_writebatch_iterator_t,
    ) -> libc::c_uchar;
}
extern "C" {
    pub fn crocksdb_writebatch_iterator_next(it: *mut crocksdb_writebatch_iterator_t);
}
extern "C" {
    pub fn crocksdb_writebatch_iterator_key(
        it: *mut crocksdb_writebatch_iterator_t,
        klen: *mut usize,
    ) -> *const libc::c_char;
}
extern "C" {
    pub fn crocksdb_writebatch_iterator_value(
        it: *mut crocksdb_writebatch_iterator_t,
        klen: *mut usize,
    ) -> *const libc::c_char;
}
extern "C" {
    pub fn crocksdb_writebatch_iterator_value_type(
        it: *mut crocksdb_writebatch_iterator_t,
    ) -> libc::c_int;
}
extern "C" {
    pub fn crocksdb_writebatch_iterator_column_family_id(
        it: *mut crocksdb_writebatch_iterator_t,
    ) -> u32;
}
extern "C" {
    pub fn crocksdb_block_based_options_create() -> *mut crocksdb_block_based_table_options_t;
}
extern "C" {
    pub fn crocksdb_block_based_options_destroy(options: *mut crocksdb_block_based_table_options_t);
}
extern "C" {
    pub fn crocksdb_block_based_options_set_metadata_block_size(
        options: *mut crocksdb_block_based_table_options_t,
        block_size: usize,
    );
}
extern "C" {
    pub fn crocksdb_block_based_options_set_block_size(
        options: *mut crocksdb_block_based_table_options_t,
        block_size: usize,
    );
}
extern "C" {
    pub fn crocksdb_block_based_options_set_block_size_deviation(
        options: *mut crocksdb_block_based_table_options_t,
        block_size_deviation: libc::c_int,
    );
}
extern "C" {
    pub fn crocksdb_block_based_options_set_block_restart_interval(
        options: *mut crocksdb_block_based_table_options_t,
        block_restart_interval: libc::c_int,
    );
}
extern "C" {
    pub fn crocksdb_block_based_options_set_filter_policy(
        options: *mut crocksdb_block_based_table_options_t,
        filter_policy: *mut crocksdb_filterpolicy_t,
    );
}
extern "C" {
    pub fn crocksdb_block_based_options_set_no_block_cache(
        options: *mut crocksdb_block_based_table_options_t,
        no_block_cache: libc::c_uchar,
    );
}
extern "C" {
    pub fn crocksdb_block_based_options_set_block_cache(
        options: *mut crocksdb_block_based_table_options_t,
        block_cache: *mut crocksdb_cache_t,
    );
}
extern "C" {
    pub fn crocksdb_block_based_options_set_block_cache_compressed(
        options: *mut crocksdb_block_based_table_options_t,
        block_cache_compressed: *mut crocksdb_cache_t,
    );
}
extern "C" {
    pub fn crocksdb_block_based_options_set_whole_key_filtering(
        arg1: *mut crocksdb_block_based_table_options_t,
        arg2: libc::c_uchar,
    );
}
extern "C" {
    pub fn crocksdb_block_based_options_set_format_version(
        arg1: *mut crocksdb_block_based_table_options_t,
        arg2: libc::c_int,
    );
}
extern "C" {
    pub fn crocksdb_block_based_options_set_index_type(
        arg1: *mut crocksdb_block_based_table_options_t,
        arg2: rocksdb_BlockBasedTableOptions_IndexType,
    );
}
extern "C" {
    pub fn crocksdb_block_based_options_set_hash_index_allow_collision(
        arg1: *mut crocksdb_block_based_table_options_t,
        arg2: libc::c_uchar,
    );
}
extern "C" {
    pub fn crocksdb_block_based_options_set_partition_filters(
        arg1: *mut crocksdb_block_based_table_options_t,
        arg2: libc::c_uchar,
    );
}
extern "C" {
    pub fn crocksdb_block_based_options_set_cache_index_and_filter_blocks(
        arg1: *mut crocksdb_block_based_table_options_t,
        arg2: libc::c_uchar,
    );
}
extern "C" {
    pub fn crocksdb_block_based_options_set_pin_top_level_index_and_filter(
        arg1: *mut crocksdb_block_based_table_options_t,
        arg2: libc::c_uchar,
    );
}
extern "C" {
    pub fn crocksdb_block_based_options_set_cache_index_and_filter_blocks_with_high_priority(
        arg1: *mut crocksdb_block_based_table_options_t,
        arg2: libc::c_uchar,
    );
}
extern "C" {
    pub fn crocksdb_block_based_options_set_pin_l0_filter_and_index_blocks_in_cache(
        arg1: *mut crocksdb_block_based_table_options_t,
        arg2: libc::c_uchar,
    );
}
extern "C" {
    pub fn crocksdb_block_based_options_set_read_amp_bytes_per_bit(
        arg1: *mut crocksdb_block_based_table_options_t,
        arg2: libc::c_int,
    );
}
extern "C" {
    pub fn crocksdb_options_set_block_based_table_factory(
        opt: *mut crocksdb_options_t,
        table_options: *mut crocksdb_block_based_table_options_t,
    );
}
extern "C" {
    pub fn crocksdb_options_get_block_cache_usage(opt: *mut crocksdb_options_t) -> usize;
}
extern "C" {
    pub fn crocksdb_options_set_block_cache_capacity(
        opt: *mut crocksdb_options_t,
        capacity: usize,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_options_get_block_cache_capacity(opt: *mut crocksdb_options_t) -> usize;
}
extern "C" {
    pub fn crocksdb_flushjobinfo_cf_name(
        arg1: *const crocksdb_flushjobinfo_t,
        arg2: *mut usize,
    ) -> *const libc::c_char;
}
extern "C" {
    pub fn crocksdb_flushjobinfo_file_path(
        arg1: *const crocksdb_flushjobinfo_t,
        arg2: *mut usize,
    ) -> *const libc::c_char;
}
extern "C" {
    pub fn crocksdb_flushjobinfo_table_properties(
        arg1: *const crocksdb_flushjobinfo_t,
    ) -> *const crocksdb_table_properties_t;
}
extern "C" {
    pub fn crocksdb_flushjobinfo_triggered_writes_slowdown(
        arg1: *const crocksdb_flushjobinfo_t,
    ) -> libc::c_uchar;
}
extern "C" {
    pub fn crocksdb_flushjobinfo_triggered_writes_stop(
        arg1: *const crocksdb_flushjobinfo_t,
    ) -> libc::c_uchar;
}
extern "C" {
    pub fn crocksdb_compactionjobinfo_status(
        info: *const crocksdb_compactionjobinfo_t,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_compactionjobinfo_cf_name(
        arg1: *const crocksdb_compactionjobinfo_t,
        arg2: *mut usize,
    ) -> *const libc::c_char;
}
extern "C" {
    pub fn crocksdb_compactionjobinfo_input_files_count(
        arg1: *const crocksdb_compactionjobinfo_t,
    ) -> usize;
}
extern "C" {
    pub fn crocksdb_compactionjobinfo_input_file_at(
        arg1: *const crocksdb_compactionjobinfo_t,
        pos: usize,
        arg2: *mut usize,
    ) -> *const libc::c_char;
}
extern "C" {
    pub fn crocksdb_compactionjobinfo_output_files_count(
        arg1: *const crocksdb_compactionjobinfo_t,
    ) -> usize;
}
extern "C" {
    pub fn crocksdb_compactionjobinfo_output_file_at(
        arg1: *const crocksdb_compactionjobinfo_t,
        pos: usize,
        arg2: *mut usize,
    ) -> *const libc::c_char;
}
extern "C" {
    pub fn crocksdb_compactionjobinfo_table_properties(
        arg1: *const crocksdb_compactionjobinfo_t,
    ) -> *const crocksdb_table_properties_collection_t;
}
extern "C" {
    pub fn crocksdb_compactionjobinfo_elapsed_micros(
        arg1: *const crocksdb_compactionjobinfo_t,
    ) -> u64;
}
extern "C" {
    pub fn crocksdb_compactionjobinfo_num_corrupt_keys(
        arg1: *const crocksdb_compactionjobinfo_t,
    ) -> u64;
}
extern "C" {
    pub fn crocksdb_compactionjobinfo_base_input_level(
        arg1: *const crocksdb_compactionjobinfo_t,
    ) -> libc::c_int;
}
extern "C" {
    pub fn crocksdb_compactionjobinfo_output_level(
        arg1: *const crocksdb_compactionjobinfo_t,
    ) -> libc::c_int;
}
extern "C" {
    pub fn crocksdb_compactionjobinfo_input_records(
        arg1: *const crocksdb_compactionjobinfo_t,
    ) -> u64;
}
extern "C" {
    pub fn crocksdb_compactionjobinfo_output_records(
        arg1: *const crocksdb_compactionjobinfo_t,
    ) -> u64;
}
extern "C" {
    pub fn crocksdb_compactionjobinfo_total_input_bytes(
        arg1: *const crocksdb_compactionjobinfo_t,
    ) -> u64;
}
extern "C" {
    pub fn crocksdb_compactionjobinfo_total_output_bytes(
        arg1: *const crocksdb_compactionjobinfo_t,
    ) -> u64;
}
extern "C" {
    pub fn crocksdb_compactionjobinfo_num_input_files(
        info: *const crocksdb_compactionjobinfo_t,
    ) -> usize;
}
extern "C" {
    pub fn crocksdb_compactionjobinfo_num_input_files_at_output_level(
        info: *const crocksdb_compactionjobinfo_t,
    ) -> usize;
}
extern "C" {
    pub fn crocksdb_subcompactionjobinfo_status(
        arg1: *const crocksdb_subcompactionjobinfo_t,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_subcompactionjobinfo_cf_name(
        arg1: *const crocksdb_subcompactionjobinfo_t,
        arg2: *mut usize,
    ) -> *const libc::c_char;
}
extern "C" {
    pub fn crocksdb_subcompactionjobinfo_thread_id(
        arg1: *const crocksdb_subcompactionjobinfo_t,
    ) -> u64;
}
extern "C" {
    pub fn crocksdb_subcompactionjobinfo_base_input_level(
        arg1: *const crocksdb_subcompactionjobinfo_t,
    ) -> libc::c_int;
}
extern "C" {
    pub fn crocksdb_subcompactionjobinfo_output_level(
        arg1: *const crocksdb_subcompactionjobinfo_t,
    ) -> libc::c_int;
}
extern "C" {
    pub fn crocksdb_externalfileingestioninfo_cf_name(
        arg1: *const crocksdb_externalfileingestioninfo_t,
        arg2: *mut usize,
    ) -> *const libc::c_char;
}
extern "C" {
    pub fn crocksdb_externalfileingestioninfo_internal_file_path(
        arg1: *const crocksdb_externalfileingestioninfo_t,
        arg2: *mut usize,
    ) -> *const libc::c_char;
}
extern "C" {
    pub fn crocksdb_externalfileingestioninfo_table_properties(
        arg1: *const crocksdb_externalfileingestioninfo_t,
    ) -> *const crocksdb_table_properties_t;
}
extern "C" {
    pub fn crocksdb_externalfileingestioninfo_picked_level(
        arg1: *const crocksdb_externalfileingestioninfo_t,
    ) -> libc::c_int;
}
extern "C" {
    pub fn crocksdb_writestallinfo_cf_name(
        arg1: *const crocksdb_writestallinfo_t,
        arg2: *mut usize,
    ) -> *const libc::c_char;
}
extern "C" {
    pub fn crocksdb_writestallinfo_cur(
        arg1: *const crocksdb_writestallinfo_t,
    ) -> *const crocksdb_writestallcondition_t;
}
extern "C" {
    pub fn crocksdb_writestallinfo_prev(
        arg1: *const crocksdb_writestallinfo_t,
    ) -> *const crocksdb_writestallcondition_t;
}
pub type on_flush_begin_cb = ::std::option::Option<
    unsafe extern "C" fn(
        arg1: *mut libc::c_void,
        arg2: *mut crocksdb_t,
        arg3: *const crocksdb_flushjobinfo_t,
    ),
>;
pub type on_flush_completed_cb = ::std::option::Option<
    unsafe extern "C" fn(
        arg1: *mut libc::c_void,
        arg2: *mut crocksdb_t,
        arg3: *const crocksdb_flushjobinfo_t,
    ),
>;
pub type on_compaction_begin_cb = ::std::option::Option<
    unsafe extern "C" fn(
        arg1: *mut libc::c_void,
        arg2: *mut crocksdb_t,
        arg3: *const crocksdb_compactionjobinfo_t,
    ),
>;
pub type on_compaction_completed_cb = ::std::option::Option<
    unsafe extern "C" fn(
        arg1: *mut libc::c_void,
        arg2: *mut crocksdb_t,
        arg3: *const crocksdb_compactionjobinfo_t,
    ),
>;
pub type on_subcompaction_begin_cb = ::std::option::Option<
    unsafe extern "C" fn(arg1: *mut libc::c_void, arg2: *const crocksdb_subcompactionjobinfo_t),
>;
pub type on_subcompaction_completed_cb = ::std::option::Option<
    unsafe extern "C" fn(arg1: *mut libc::c_void, arg2: *const crocksdb_subcompactionjobinfo_t),
>;
pub type on_external_file_ingested_cb = ::std::option::Option<
    unsafe extern "C" fn(
        arg1: *mut libc::c_void,
        arg2: *mut crocksdb_t,
        arg3: *const crocksdb_externalfileingestioninfo_t,
    ),
>;
pub type on_background_error_cb = ::std::option::Option<
    unsafe extern "C" fn(
        arg1: *mut libc::c_void,
        arg2: rocksdb_BackgroundErrorReason,
        s: *mut rocksdb_Status,
    ),
>;
pub type on_stall_conditions_changed_cb = ::std::option::Option<
    unsafe extern "C" fn(arg1: *mut libc::c_void, arg2: *const crocksdb_writestallinfo_t),
>;
pub type crocksdb_logger_logv_cb = ::std::option::Option<
    unsafe extern "C" fn(
        arg1: *mut libc::c_void,
        log_level: rocksdb_InfoLogLevel,
        msg: rocksdb_Slice,
    ),
>;
extern "C" {
    pub fn crocksdb_eventlistener_create(
        state_: *mut libc::c_void,
        destructor_: ::std::option::Option<unsafe extern "C" fn(arg1: *mut libc::c_void)>,
        on_flush_begin: on_flush_begin_cb,
        on_flush_completed: on_flush_completed_cb,
        on_compaction_begin: on_compaction_begin_cb,
        on_compaction_completed: on_compaction_completed_cb,
        on_subcompaction_begin: on_subcompaction_begin_cb,
        on_subcompaction_completed: on_subcompaction_completed_cb,
        on_external_file_ingested: on_external_file_ingested_cb,
        on_background_error: on_background_error_cb,
        on_stall_conditions_changed: on_stall_conditions_changed_cb,
    ) -> *mut crocksdb_eventlistener_t;
}
extern "C" {
    pub fn crocksdb_eventlistener_destroy(arg1: *mut crocksdb_eventlistener_t);
}
extern "C" {
    pub fn crocksdb_options_add_eventlistener(
        arg1: *mut crocksdb_options_t,
        arg2: *mut crocksdb_eventlistener_t,
    );
}
extern "C" {
    pub fn crocksdb_cuckoo_options_create() -> *mut crocksdb_cuckoo_table_options_t;
}
extern "C" {
    pub fn crocksdb_cuckoo_options_destroy(options: *mut crocksdb_cuckoo_table_options_t);
}
extern "C" {
    pub fn crocksdb_cuckoo_options_set_hash_ratio(
        options: *mut crocksdb_cuckoo_table_options_t,
        v: f64,
    );
}
extern "C" {
    pub fn crocksdb_cuckoo_options_set_max_search_depth(
        options: *mut crocksdb_cuckoo_table_options_t,
        v: u32,
    );
}
extern "C" {
    pub fn crocksdb_cuckoo_options_set_cuckoo_block_size(
        options: *mut crocksdb_cuckoo_table_options_t,
        v: u32,
    );
}
extern "C" {
    pub fn crocksdb_cuckoo_options_set_identity_as_first_hash(
        options: *mut crocksdb_cuckoo_table_options_t,
        v: libc::c_uchar,
    );
}
extern "C" {
    pub fn crocksdb_cuckoo_options_set_use_module_hash(
        options: *mut crocksdb_cuckoo_table_options_t,
        v: libc::c_uchar,
    );
}
extern "C" {
    pub fn crocksdb_options_set_cuckoo_table_factory(
        opt: *mut crocksdb_options_t,
        table_options: *mut crocksdb_cuckoo_table_options_t,
    );
}
extern "C" {
    pub fn crocksdb_options_create() -> *mut crocksdb_options_t;
}
extern "C" {
    pub fn crocksdb_options_copy(arg1: *const crocksdb_options_t) -> *mut crocksdb_options_t;
}
extern "C" {
    pub fn crocksdb_options_destroy(arg1: *mut crocksdb_options_t);
}
extern "C" {
    pub fn crocksdb_column_family_descriptor_destroy(
        cf_desc: *mut crocksdb_column_family_descriptor,
    );
}
extern "C" {
    pub fn crocksdb_name_from_column_family_descriptor(
        cf_desc: *const crocksdb_column_family_descriptor,
    ) -> *const libc::c_char;
}
extern "C" {
    pub fn crocksdb_options_from_column_family_descriptor(
        cf_desc: *const crocksdb_column_family_descriptor,
    ) -> *mut crocksdb_options_t;
}
extern "C" {
    pub fn crocksdb_options_increase_parallelism(
        opt: *mut crocksdb_options_t,
        total_threads: libc::c_int,
    );
}
extern "C" {
    pub fn crocksdb_options_optimize_for_point_lookup(
        opt: *mut crocksdb_options_t,
        block_cache_size_mb: u64,
    );
}
extern "C" {
    pub fn crocksdb_options_optimize_level_style_compaction(
        opt: *mut crocksdb_options_t,
        memtable_memory_budget: u64,
    );
}
extern "C" {
    pub fn crocksdb_options_optimize_universal_style_compaction(
        opt: *mut crocksdb_options_t,
        memtable_memory_budget: u64,
    );
}
extern "C" {
    pub fn crocksdb_options_set_compaction_filter(
        arg1: *mut crocksdb_options_t,
        arg2: *mut crocksdb_compactionfilter_t,
    );
}
extern "C" {
    pub fn crocksdb_options_set_compaction_filter_factory(
        arg1: *mut crocksdb_options_t,
        arg2: *mut crocksdb_compactionfilterfactory_t,
    );
}
extern "C" {
    pub fn crocksdb_options_compaction_readahead_size(arg1: *mut crocksdb_options_t, arg2: usize);
}
extern "C" {
    pub fn crocksdb_options_set_comparator(
        arg1: *mut crocksdb_options_t,
        arg2: *mut crocksdb_comparator_t,
    );
}
extern "C" {
    pub fn crocksdb_options_set_merge_operator(
        arg1: *mut crocksdb_options_t,
        arg2: *mut crocksdb_mergeoperator_t,
    );
}
extern "C" {
    pub fn crocksdb_options_set_compression_per_level(
        opt: *mut crocksdb_options_t,
        level_values: *mut rocksdb_CompressionType,
        num_levels: usize,
    );
}
extern "C" {
    pub fn crocksdb_options_get_compression_level_number(opt: *mut crocksdb_options_t) -> usize;
}
extern "C" {
    pub fn crocksdb_options_get_compression_per_level(
        opt: *mut crocksdb_options_t,
        level_values: *mut rocksdb_CompressionType,
    );
}
extern "C" {
    pub fn crocksdb_set_bottommost_compression(
        opt: *mut crocksdb_options_t,
        c: rocksdb_CompressionType,
    );
}
extern "C" {
    pub fn crocksdb_options_set_create_if_missing(
        arg1: *mut crocksdb_options_t,
        arg2: libc::c_uchar,
    );
}
extern "C" {
    pub fn crocksdb_options_set_create_missing_column_families(
        arg1: *mut crocksdb_options_t,
        arg2: libc::c_uchar,
    );
}
extern "C" {
    pub fn crocksdb_options_set_error_if_exists(arg1: *mut crocksdb_options_t, arg2: libc::c_uchar);
}
extern "C" {
    pub fn crocksdb_options_set_paranoid_checks(arg1: *mut crocksdb_options_t, arg2: libc::c_uchar);
}
extern "C" {
    pub fn crocksdb_options_set_env(arg1: *mut crocksdb_options_t, arg2: *mut rocksdb_Env);
}
extern "C" {
    pub fn crocksdb_logger_create(
        rep: *mut libc::c_void,
        destructor_: ::std::option::Option<unsafe extern "C" fn(arg1: *mut libc::c_void)>,
        logv: crocksdb_logger_logv_cb,
    ) -> *mut crocksdb_logger_t;
}
extern "C" {
    pub fn crocksdb_options_set_info_log(
        arg1: *mut crocksdb_options_t,
        arg2: *mut crocksdb_logger_t,
    );
}
extern "C" {
    pub fn crocksdb_options_set_info_log_level(
        arg1: *mut crocksdb_options_t,
        arg2: rocksdb_InfoLogLevel,
    );
}
extern "C" {
    pub fn crocksdb_options_set_write_buffer_size(arg1: *mut crocksdb_options_t, arg2: usize);
}
extern "C" {
    pub fn crocksdb_options_get_write_buffer_size(arg1: *mut crocksdb_options_t) -> usize;
}
extern "C" {
    pub fn crocksdb_options_set_db_write_buffer_size(arg1: *mut crocksdb_options_t, arg2: usize);
}
extern "C" {
    pub fn crocksdb_options_set_max_open_files(arg1: *mut crocksdb_options_t, arg2: libc::c_int);
}
extern "C" {
    pub fn crocksdb_options_set_max_total_wal_size(opt: *mut crocksdb_options_t, n: u64);
}
extern "C" {
    pub fn crocksdb_options_set_bottommost_compression_options(
        arg1: *mut crocksdb_options_t,
        arg2: libc::c_int,
        arg3: libc::c_int,
        arg4: libc::c_int,
        arg5: libc::c_int,
        arg6: libc::c_int,
    );
}
extern "C" {
    pub fn crocksdb_options_set_compression_options(
        arg1: *mut crocksdb_options_t,
        arg2: libc::c_int,
        arg3: libc::c_int,
        arg4: libc::c_int,
        arg5: libc::c_int,
        arg6: libc::c_int,
    );
}
extern "C" {
    pub fn crocksdb_options_set_use_direct_reads(opt: *mut crocksdb_options_t, v: libc::c_uchar);
}
extern "C" {
    pub fn crocksdb_options_set_use_direct_io_for_flush_and_compaction(
        opt: *mut crocksdb_options_t,
        v: libc::c_uchar,
    );
}
extern "C" {
    pub fn crocksdb_options_set_prefix_extractor(
        arg1: *mut crocksdb_options_t,
        arg2: *mut crocksdb_slicetransform_t,
    );
}
extern "C" {
    pub fn crocksdb_options_set_memtable_insert_with_hint_prefix_extractor(
        arg1: *mut crocksdb_options_t,
        arg2: *mut crocksdb_slicetransform_t,
    );
}
extern "C" {
    pub fn crocksdb_options_set_num_levels(arg1: *mut crocksdb_options_t, arg2: libc::c_int);
}
extern "C" {
    pub fn crocksdb_options_get_num_levels(arg1: *mut crocksdb_options_t) -> libc::c_int;
}
extern "C" {
    pub fn crocksdb_options_set_level0_file_num_compaction_trigger(
        arg1: *mut crocksdb_options_t,
        arg2: libc::c_int,
    );
}
extern "C" {
    pub fn crocksdb_options_get_level0_file_num_compaction_trigger(
        arg1: *mut crocksdb_options_t,
    ) -> libc::c_int;
}
extern "C" {
    pub fn crocksdb_options_set_level0_slowdown_writes_trigger(
        arg1: *mut crocksdb_options_t,
        arg2: libc::c_int,
    );
}
extern "C" {
    pub fn crocksdb_options_get_level0_slowdown_writes_trigger(
        arg1: *mut crocksdb_options_t,
    ) -> libc::c_int;
}
extern "C" {
    pub fn crocksdb_options_set_level0_stop_writes_trigger(
        arg1: *mut crocksdb_options_t,
        arg2: libc::c_int,
    );
}
extern "C" {
    pub fn crocksdb_options_get_level0_stop_writes_trigger(
        arg1: *mut crocksdb_options_t,
    ) -> libc::c_int;
}
extern "C" {
    pub fn crocksdb_options_set_target_file_size_base(arg1: *mut crocksdb_options_t, arg2: u64);
}
extern "C" {
    pub fn crocksdb_options_get_target_file_size_base(arg1: *const crocksdb_options_t) -> u64;
}
extern "C" {
    pub fn crocksdb_options_set_target_file_size_multiplier(
        arg1: *mut crocksdb_options_t,
        arg2: libc::c_int,
    );
}
extern "C" {
    pub fn crocksdb_options_set_max_bytes_for_level_base(arg1: *mut crocksdb_options_t, arg2: u64);
}
extern "C" {
    pub fn crocksdb_options_get_max_bytes_for_level_base(arg1: *mut crocksdb_options_t) -> u64;
}
extern "C" {
    pub fn crocksdb_options_set_optimize_filters_for_hits(
        arg1: *mut crocksdb_options_t,
        arg2: libc::c_uchar,
    );
}
extern "C" {
    pub fn crocksdb_options_set_level_compaction_dynamic_level_bytes(
        arg1: *mut crocksdb_options_t,
        arg2: libc::c_uchar,
    );
}
extern "C" {
    pub fn crocksdb_options_get_level_compaction_dynamic_level_bytes(
        options: *const crocksdb_options_t,
    ) -> libc::c_uchar;
}
extern "C" {
    pub fn crocksdb_options_set_max_bytes_for_level_multiplier(
        arg1: *mut crocksdb_options_t,
        arg2: f64,
    );
}
extern "C" {
    pub fn crocksdb_options_get_max_bytes_for_level_multiplier(
        arg1: *mut crocksdb_options_t,
    ) -> f64;
}
extern "C" {
    pub fn crocksdb_options_set_max_bytes_for_level_multiplier_additional(
        arg1: *mut crocksdb_options_t,
        level_values: *mut libc::c_int,
        num_levels: usize,
    );
}
extern "C" {
    pub fn crocksdb_options_get_sst_partitioner_factory(
        arg1: *mut crocksdb_options_t,
    ) -> *mut crocksdb_sst_partitioner_factory_t;
}
extern "C" {
    pub fn crocksdb_options_set_sst_partitioner_factory(
        arg1: *mut crocksdb_options_t,
        arg2: *mut crocksdb_sst_partitioner_factory_t,
    );
}
extern "C" {
    pub fn crocksdb_options_enable_statistics(arg1: *mut crocksdb_options_t, arg2: libc::c_uchar);
}
extern "C" {
    pub fn crocksdb_options_reset_statistics(arg1: *mut crocksdb_options_t);
}
extern "C" {
    pub fn crocksdb_load_latest_options(
        dbpath: *const libc::c_char,
        env: *mut rocksdb_Env,
        db_options: *mut crocksdb_options_t,
        cf_descs: *mut *mut *mut crocksdb_column_family_descriptor,
        cf_descs_len: *mut usize,
        ignore_unknown_options: libc::c_uchar,
        s: *mut rocksdb_Status,
    ) -> libc::c_uchar;
}
extern "C" {
    pub fn crocksdb_options_statistics_get_string(
        opt: *mut crocksdb_options_t,
    ) -> *mut libc::c_char;
}
extern "C" {
    pub fn crocksdb_options_statistics_get_ticker_count(
        opt: *mut crocksdb_options_t,
        ticker_type: u32,
    ) -> u64;
}
extern "C" {
    pub fn crocksdb_options_statistics_get_and_reset_ticker_count(
        opt: *mut crocksdb_options_t,
        ticker_type: u32,
    ) -> u64;
}
extern "C" {
    pub fn crocksdb_options_statistics_get_histogram_string(
        opt: *mut crocksdb_options_t,
        type_: u32,
    ) -> *mut libc::c_char;
}
extern "C" {
    pub fn crocksdb_options_statistics_get_histogram(
        opt: *mut crocksdb_options_t,
        type_: u32,
        median: *mut f64,
        percentile95: *mut f64,
        percentile99: *mut f64,
        average: *mut f64,
        standard_deviation: *mut f64,
        max: *mut f64,
    ) -> libc::c_uchar;
}
extern "C" {
    pub fn crocksdb_options_set_max_write_buffer_number(
        arg1: *mut crocksdb_options_t,
        arg2: libc::c_int,
    );
}
extern "C" {
    pub fn crocksdb_options_get_max_write_buffer_number(
        arg1: *mut crocksdb_options_t,
    ) -> libc::c_int;
}
extern "C" {
    pub fn crocksdb_options_set_min_write_buffer_number_to_merge(
        arg1: *mut crocksdb_options_t,
        arg2: libc::c_int,
    );
}
extern "C" {
    pub fn crocksdb_options_get_min_write_buffer_number_to_merge(
        arg1: *mut crocksdb_options_t,
    ) -> libc::c_int;
}
extern "C" {
    pub fn crocksdb_options_set_max_write_buffer_number_to_maintain(
        arg1: *mut crocksdb_options_t,
        arg2: libc::c_int,
    );
}
extern "C" {
    pub fn crocksdb_options_set_max_background_jobs(
        arg1: *mut crocksdb_options_t,
        arg2: libc::c_int,
    );
}
extern "C" {
    pub fn crocksdb_options_get_max_background_jobs(arg1: *const crocksdb_options_t)
        -> libc::c_int;
}
extern "C" {
    pub fn crocksdb_options_set_max_background_compactions(
        arg1: *mut crocksdb_options_t,
        arg2: libc::c_int,
    );
}
extern "C" {
    pub fn crocksdb_options_get_max_background_compactions(
        arg1: *const crocksdb_options_t,
    ) -> libc::c_int;
}
extern "C" {
    pub fn crocksdb_options_set_base_background_compactions(
        arg1: *mut crocksdb_options_t,
        arg2: libc::c_int,
    );
}
extern "C" {
    pub fn crocksdb_options_get_base_background_compactions(
        arg1: *const crocksdb_options_t,
    ) -> libc::c_int;
}
extern "C" {
    pub fn crocksdb_options_set_max_background_flushes(
        arg1: *mut crocksdb_options_t,
        arg2: libc::c_int,
    );
}
extern "C" {
    pub fn crocksdb_options_get_max_background_flushes(
        arg1: *const crocksdb_options_t,
    ) -> libc::c_int;
}
extern "C" {
    pub fn crocksdb_options_set_max_log_file_size(arg1: *mut crocksdb_options_t, arg2: usize);
}
extern "C" {
    pub fn crocksdb_options_set_log_file_time_to_roll(arg1: *mut crocksdb_options_t, arg2: usize);
}
extern "C" {
    pub fn crocksdb_options_set_keep_log_file_num(arg1: *mut crocksdb_options_t, arg2: usize);
}
extern "C" {
    pub fn crocksdb_options_set_recycle_log_file_num(arg1: *mut crocksdb_options_t, arg2: usize);
}
extern "C" {
    pub fn crocksdb_options_set_soft_rate_limit(arg1: *mut crocksdb_options_t, arg2: f64);
}
extern "C" {
    pub fn crocksdb_options_set_hard_rate_limit(arg1: *mut crocksdb_options_t, arg2: f64);
}
extern "C" {
    pub fn crocksdb_options_set_soft_pending_compaction_bytes_limit(
        opt: *mut crocksdb_options_t,
        v: usize,
    );
}
extern "C" {
    pub fn crocksdb_options_get_soft_pending_compaction_bytes_limit(
        opt: *mut crocksdb_options_t,
    ) -> usize;
}
extern "C" {
    pub fn crocksdb_options_set_hard_pending_compaction_bytes_limit(
        opt: *mut crocksdb_options_t,
        v: usize,
    );
}
extern "C" {
    pub fn crocksdb_options_get_hard_pending_compaction_bytes_limit(
        opt: *mut crocksdb_options_t,
    ) -> usize;
}
extern "C" {
    pub fn crocksdb_options_set_rate_limit_delay_max_milliseconds(
        arg1: *mut crocksdb_options_t,
        arg2: libc::c_uint,
    );
}
extern "C" {
    pub fn crocksdb_options_set_max_manifest_file_size(arg1: *mut crocksdb_options_t, arg2: usize);
}
extern "C" {
    pub fn crocksdb_options_set_table_cache_numshardbits(
        arg1: *mut crocksdb_options_t,
        arg2: libc::c_int,
    );
}
extern "C" {
    pub fn crocksdb_options_set_writable_file_max_buffer_size(
        arg1: *mut crocksdb_options_t,
        arg2: libc::c_int,
    );
}
extern "C" {
    pub fn crocksdb_options_set_arena_block_size(arg1: *mut crocksdb_options_t, arg2: usize);
}
extern "C" {
    pub fn crocksdb_options_set_use_fsync(arg1: *mut crocksdb_options_t, arg2: libc::c_int);
}
extern "C" {
    pub fn crocksdb_options_set_db_paths(
        arg1: *mut crocksdb_options_t,
        arg2: *const *const libc::c_char,
        arg3: *const usize,
        arg4: *const u64,
        arg5: libc::c_int,
    );
}
extern "C" {
    pub fn crocksdb_options_get_db_paths_num(arg1: *mut crocksdb_options_t) -> usize;
}
extern "C" {
    pub fn crocksdb_options_get_db_path(
        arg1: *mut crocksdb_options_t,
        index: usize,
    ) -> *const libc::c_char;
}
extern "C" {
    pub fn crocksdb_options_get_path_target_size(
        arg1: *mut crocksdb_options_t,
        index: usize,
    ) -> u64;
}
extern "C" {
    pub fn crocksdb_options_set_db_log_dir(
        arg1: *mut crocksdb_options_t,
        arg2: *const libc::c_char,
    );
}
extern "C" {
    pub fn crocksdb_options_set_wal_dir(arg1: *mut crocksdb_options_t, arg2: *const libc::c_char);
}
extern "C" {
    pub fn crocksdb_options_set_wal_ttl_seconds(arg1: *mut crocksdb_options_t, arg2: u64);
}
extern "C" {
    pub fn crocksdb_options_set_wal_size_limit_mb(arg1: *mut crocksdb_options_t, arg2: u64);
}
extern "C" {
    pub fn crocksdb_options_set_manifest_preallocation_size(
        arg1: *mut crocksdb_options_t,
        arg2: usize,
    );
}
extern "C" {
    pub fn crocksdb_options_set_allow_mmap_reads(
        arg1: *mut crocksdb_options_t,
        arg2: libc::c_uchar,
    );
}
extern "C" {
    pub fn crocksdb_options_set_allow_mmap_writes(
        arg1: *mut crocksdb_options_t,
        arg2: libc::c_uchar,
    );
}
extern "C" {
    pub fn crocksdb_options_set_is_fd_close_on_exec(
        arg1: *mut crocksdb_options_t,
        arg2: libc::c_uchar,
    );
}
extern "C" {
    pub fn crocksdb_options_set_skip_log_error_on_recovery(
        arg1: *mut crocksdb_options_t,
        arg2: libc::c_uchar,
    );
}
extern "C" {
    pub fn crocksdb_options_set_stats_dump_period_sec(
        arg1: *mut crocksdb_options_t,
        arg2: libc::c_uint,
    );
}
extern "C" {
    pub fn crocksdb_options_set_advise_random_on_open(
        arg1: *mut crocksdb_options_t,
        arg2: libc::c_uchar,
    );
}
extern "C" {
    pub fn crocksdb_options_set_access_hint_on_compaction_start(
        arg1: *mut crocksdb_options_t,
        arg2: libc::c_int,
    );
}
extern "C" {
    pub fn crocksdb_options_set_use_adaptive_mutex(
        arg1: *mut crocksdb_options_t,
        arg2: libc::c_uchar,
    );
}
extern "C" {
    pub fn crocksdb_options_set_bytes_per_sync(arg1: *mut crocksdb_options_t, arg2: u64);
}
extern "C" {
    pub fn crocksdb_options_set_enable_pipelined_write(
        arg1: *mut crocksdb_options_t,
        arg2: libc::c_uchar,
    );
}
extern "C" {
    pub fn crocksdb_options_set_enable_multi_batch_write(
        opt: *mut crocksdb_options_t,
        v: libc::c_uchar,
    );
}
extern "C" {
    pub fn crocksdb_options_is_enable_multi_batch_write(
        opt: *mut crocksdb_options_t,
    ) -> libc::c_uchar;
}
extern "C" {
    pub fn crocksdb_options_set_unordered_write(arg1: *mut crocksdb_options_t, arg2: libc::c_uchar);
}
extern "C" {
    pub fn crocksdb_options_set_allow_concurrent_memtable_write(
        arg1: *mut crocksdb_options_t,
        arg2: libc::c_uchar,
    );
}
extern "C" {
    pub fn crocksdb_options_set_manual_wal_flush(
        arg1: *mut crocksdb_options_t,
        arg2: libc::c_uchar,
    );
}
extern "C" {
    pub fn crocksdb_options_set_enable_write_thread_adaptive_yield(
        arg1: *mut crocksdb_options_t,
        arg2: libc::c_uchar,
    );
}
extern "C" {
    pub fn crocksdb_options_set_max_sequential_skip_in_iterations(
        arg1: *mut crocksdb_options_t,
        arg2: u64,
    );
}
extern "C" {
    pub fn crocksdb_options_set_disable_auto_compactions(
        arg1: *mut crocksdb_options_t,
        arg2: libc::c_int,
    );
}
extern "C" {
    pub fn crocksdb_options_get_disable_auto_compactions(
        arg1: *const crocksdb_options_t,
    ) -> libc::c_int;
}
extern "C" {
    pub fn crocksdb_options_set_disable_write_stall(
        arg1: *mut crocksdb_options_t,
        arg2: libc::c_uchar,
    );
}
extern "C" {
    pub fn crocksdb_options_get_disable_write_stall(
        arg1: *const crocksdb_options_t,
    ) -> libc::c_uchar;
}
extern "C" {
    pub fn crocksdb_options_set_delete_obsolete_files_period_micros(
        arg1: *mut crocksdb_options_t,
        arg2: u64,
    );
}
extern "C" {
    pub fn crocksdb_options_prepare_for_bulk_load(arg1: *mut crocksdb_options_t);
}
extern "C" {
    pub fn crocksdb_options_get_memtable_factory_name(
        opt: *mut crocksdb_options_t,
    ) -> *const libc::c_char;
}
extern "C" {
    pub fn crocksdb_options_set_memtable_vector_rep(arg1: *mut crocksdb_options_t);
}
extern "C" {
    pub fn crocksdb_options_set_memtable_prefix_bloom_size_ratio(
        arg1: *mut crocksdb_options_t,
        arg2: f64,
    );
}
extern "C" {
    pub fn crocksdb_options_set_max_compaction_bytes(arg1: *mut crocksdb_options_t, arg2: u64);
}
extern "C" {
    pub fn crocksdb_options_get_max_compaction_bytes(arg1: *mut crocksdb_options_t) -> u64;
}
extern "C" {
    pub fn crocksdb_options_set_hash_skip_list_rep(
        arg1: *mut crocksdb_options_t,
        arg2: usize,
        arg3: i32,
        arg4: i32,
    );
}
extern "C" {
    pub fn crocksdb_options_set_hash_link_list_rep(arg1: *mut crocksdb_options_t, arg2: usize);
}
extern "C" {
    pub fn crocksdb_options_set_doubly_skip_list_rep(opt: *mut crocksdb_options_t);
}
extern "C" {
    pub fn crocksdb_options_set_plain_table_factory(
        arg1: *mut crocksdb_options_t,
        arg2: u32,
        arg3: libc::c_int,
        arg4: f64,
        arg5: usize,
    );
}
extern "C" {
    pub fn crocksdb_options_set_min_level_to_compress(
        opt: *mut crocksdb_options_t,
        level: libc::c_int,
    );
}
extern "C" {
    pub fn crocksdb_options_set_memtable_huge_page_size(arg1: *mut crocksdb_options_t, arg2: usize);
}
extern "C" {
    pub fn crocksdb_options_set_max_successive_merges(arg1: *mut crocksdb_options_t, arg2: usize);
}
extern "C" {
    pub fn crocksdb_options_set_bloom_locality(arg1: *mut crocksdb_options_t, arg2: u32);
}
extern "C" {
    pub fn crocksdb_options_set_inplace_update_support(
        arg1: *mut crocksdb_options_t,
        arg2: libc::c_uchar,
    );
}
extern "C" {
    pub fn crocksdb_options_set_inplace_update_num_locks(
        arg1: *mut crocksdb_options_t,
        arg2: usize,
    );
}
extern "C" {
    pub fn crocksdb_options_set_report_bg_io_stats(
        arg1: *mut crocksdb_options_t,
        arg2: libc::c_int,
    );
}
extern "C" {
    pub fn crocksdb_options_set_compaction_readahead_size(
        arg1: *mut crocksdb_options_t,
        arg2: usize,
    );
}
extern "C" {
    pub fn crocksdb_options_set_max_subcompactions(arg1: *mut crocksdb_options_t, arg2: u32);
}
extern "C" {
    pub fn crocksdb_options_set_wal_bytes_per_sync(arg1: *mut crocksdb_options_t, arg2: u64);
}
extern "C" {
    pub fn crocksdb_options_set_wal_recovery_mode(
        arg1: *mut crocksdb_options_t,
        arg2: rocksdb_WALRecoveryMode,
    );
}
extern "C" {
    pub fn crocksdb_options_set_compression(
        arg1: *mut crocksdb_options_t,
        arg2: rocksdb_CompressionType,
    );
}
extern "C" {
    pub fn crocksdb_options_get_compression(
        arg1: *mut crocksdb_options_t,
    ) -> rocksdb_CompressionType;
}
extern "C" {
    pub fn crocksdb_options_set_compaction_style(
        arg1: *mut crocksdb_options_t,
        arg2: rocksdb_CompactionStyle,
    );
}
extern "C" {
    pub fn crocksdb_options_set_universal_compaction_options(
        arg1: *mut crocksdb_options_t,
        arg2: *mut crocksdb_universal_compaction_options_t,
    );
}
extern "C" {
    pub fn crocksdb_options_set_fifo_compaction_options(
        opt: *mut crocksdb_options_t,
        fifo: *mut crocksdb_fifo_compaction_options_t,
    );
}
extern "C" {
    pub fn crocksdb_options_set_ratelimiter(
        opt: *mut crocksdb_options_t,
        limiter: *mut crocksdb_ratelimiter_t,
    );
}
extern "C" {
    pub fn crocksdb_options_get_ratelimiter(
        opt: *mut crocksdb_options_t,
    ) -> *mut crocksdb_ratelimiter_t;
}
extern "C" {
    pub fn crocksdb_options_set_vector_memtable_factory(
        opt: *mut crocksdb_options_t,
        reserved_bytes: u64,
    );
}
extern "C" {
    pub fn crocksdb_options_set_atomic_flush(opt: *mut crocksdb_options_t, enable: libc::c_uchar);
}
extern "C" {
    pub fn crocksdb_options_set_compaction_priority(
        arg1: *mut crocksdb_options_t,
        arg2: rocksdb_CompactionPri,
    );
}
extern "C" {
    pub fn crocksdb_options_set_delayed_write_rate(arg1: *mut crocksdb_options_t, arg2: u64);
}
extern "C" {
    pub fn crocksdb_options_set_force_consistency_checks(
        arg1: *mut crocksdb_options_t,
        arg2: libc::c_uchar,
    );
}
extern "C" {
    pub fn crocksdb_options_get_force_consistency_checks(
        arg1: *mut crocksdb_options_t,
    ) -> libc::c_uchar;
}
extern "C" {
    pub fn crocksdb_ratelimiter_create(
        rate_bytes_per_sec: i64,
        refill_period_us: i64,
        fairness: i32,
    ) -> *mut crocksdb_ratelimiter_t;
}
extern "C" {
    pub fn crocksdb_ratelimiter_create_with_auto_tuned(
        rate_bytes_per_sec: i64,
        refill_period_us: i64,
        fairness: i32,
        mode: rocksdb_RateLimiter_Mode,
        auto_tuned: libc::c_uchar,
    ) -> *mut crocksdb_ratelimiter_t;
}
extern "C" {
    pub fn crocksdb_writeampbasedratelimiter_create_with_auto_tuned(
        rate_bytes_per_sec: i64,
        refill_period_us: i64,
        fairness: i32,
        mode: rocksdb_RateLimiter_Mode,
        auto_tuned: libc::c_uchar,
    ) -> *mut crocksdb_ratelimiter_t;
}
extern "C" {
    pub fn crocksdb_ratelimiter_destroy(arg1: *mut crocksdb_ratelimiter_t);
}
extern "C" {
    pub fn crocksdb_ratelimiter_set_bytes_per_second(
        limiter: *mut crocksdb_ratelimiter_t,
        rate_bytes_per_sec: i64,
    );
}
extern "C" {
    pub fn crocksdb_ratelimiter_set_auto_tuned(
        limiter: *mut crocksdb_ratelimiter_t,
        auto_tuned: libc::c_uchar,
    );
}
extern "C" {
    pub fn crocksdb_ratelimiter_get_singleburst_bytes(limiter: *mut crocksdb_ratelimiter_t) -> i64;
}
extern "C" {
    pub fn crocksdb_ratelimiter_request(
        limiter: *mut crocksdb_ratelimiter_t,
        bytes: i64,
        pri: rocksdb_Env_IOPriority,
        op_ty: rocksdb_RateLimiter_OpType,
    );
}
extern "C" {
    pub fn crocksdb_ratelimiter_get_total_bytes_through(
        limiter: *mut crocksdb_ratelimiter_t,
        pri: rocksdb_Env_IOPriority,
    ) -> i64;
}
extern "C" {
    pub fn crocksdb_ratelimiter_get_bytes_per_second(limiter: *mut crocksdb_ratelimiter_t) -> i64;
}
extern "C" {
    pub fn crocksdb_ratelimiter_get_auto_tuned(
        limiter: *mut crocksdb_ratelimiter_t,
    ) -> libc::c_uchar;
}
extern "C" {
    pub fn crocksdb_ratelimiter_get_total_requests(
        limiter: *mut crocksdb_ratelimiter_t,
        pri: rocksdb_Env_IOPriority,
    ) -> i64;
}
extern "C" {
    pub fn crocksdb_compactionfiltercontext_is_full_compaction(
        context: *mut crocksdb_compactionfiltercontext_t,
    ) -> libc::c_uchar;
}
extern "C" {
    pub fn crocksdb_compactionfiltercontext_is_manual_compaction(
        context: *mut crocksdb_compactionfiltercontext_t,
    ) -> libc::c_uchar;
}
extern "C" {
    pub fn crocksdb_compactionfiltercontext_is_bottommost_level(
        context: *mut crocksdb_compactionfiltercontext_t,
    ) -> libc::c_uchar;
}
extern "C" {
    pub fn crocksdb_compactionfiltercontext_file_numbers(
        context: *mut crocksdb_compactionfiltercontext_t,
        buffer: *mut *const u64,
        len: *mut usize,
    );
}
extern "C" {
    pub fn crocksdb_compactionfiltercontext_table_properties(
        context: *mut crocksdb_compactionfiltercontext_t,
        offset: usize,
    ) -> *mut crocksdb_table_properties_t;
}
extern "C" {
    pub fn crocksdb_compactionfilterfactory_create(
        state: *mut libc::c_void,
        destructor: ::std::option::Option<unsafe extern "C" fn(arg1: *mut libc::c_void)>,
        create_compaction_filter: ::std::option::Option<
            unsafe extern "C" fn(
                arg1: *mut libc::c_void,
                context: *mut crocksdb_compactionfiltercontext_t,
            ) -> *mut crocksdb_compactionfilter_t,
        >,
        should_filter_table_file_creation: ::std::option::Option<
            unsafe extern "C" fn(
                arg1: *mut libc::c_void,
                reason: rocksdb_TableFileCreationReason,
            ) -> libc::c_uchar,
        >,
        name: ::std::option::Option<
            unsafe extern "C" fn(arg1: *mut libc::c_void) -> *const libc::c_char,
        >,
    ) -> *mut crocksdb_compactionfilterfactory_t;
}
extern "C" {
    pub fn crocksdb_compactionfilterfactory_destroy(arg1: *mut crocksdb_compactionfilterfactory_t);
}
extern "C" {
    pub fn crocksdb_comparator_create(
        state: *mut libc::c_void,
        destructor: ::std::option::Option<unsafe extern "C" fn(arg1: *mut libc::c_void)>,
        compare: ::std::option::Option<
            unsafe extern "C" fn(
                arg1: *mut libc::c_void,
                a: *const libc::c_char,
                alen: usize,
                b: *const libc::c_char,
                blen: usize,
            ) -> libc::c_int,
        >,
        name: ::std::option::Option<
            unsafe extern "C" fn(arg1: *mut libc::c_void) -> *const libc::c_char,
        >,
    ) -> *mut crocksdb_comparator_t;
}
extern "C" {
    pub fn crocksdb_comparator_destroy(arg1: *mut crocksdb_comparator_t);
}
extern "C" {
    pub fn crocksdb_filterpolicy_create(
        state: *mut libc::c_void,
        destructor: ::std::option::Option<unsafe extern "C" fn(arg1: *mut libc::c_void)>,
        create_filter: ::std::option::Option<
            unsafe extern "C" fn(
                arg1: *mut libc::c_void,
                key_array: *const *const libc::c_char,
                key_length_array: *const usize,
                num_keys: libc::c_int,
                filter_length: *mut usize,
            ) -> *mut libc::c_char,
        >,
        key_may_match: ::std::option::Option<
            unsafe extern "C" fn(
                arg1: *mut libc::c_void,
                key: *const libc::c_char,
                length: usize,
                filter: *const libc::c_char,
                filter_length: usize,
            ) -> libc::c_uchar,
        >,
        delete_filter: ::std::option::Option<
            unsafe extern "C" fn(
                arg1: *mut libc::c_void,
                filter: *const libc::c_char,
                filter_length: usize,
            ),
        >,
        name: ::std::option::Option<
            unsafe extern "C" fn(arg1: *mut libc::c_void) -> *const libc::c_char,
        >,
    ) -> *mut crocksdb_filterpolicy_t;
}
extern "C" {
    pub fn crocksdb_filterpolicy_destroy(arg1: *mut crocksdb_filterpolicy_t);
}
extern "C" {
    pub fn crocksdb_filterpolicy_create_bloom(
        bits_per_key: libc::c_int,
    ) -> *mut crocksdb_filterpolicy_t;
}
extern "C" {
    pub fn crocksdb_filterpolicy_create_bloom_full(
        bits_per_key: libc::c_int,
    ) -> *mut crocksdb_filterpolicy_t;
}
extern "C" {
    pub fn crocksdb_mergeoperator_create(
        state: *mut libc::c_void,
        destructor: ::std::option::Option<unsafe extern "C" fn(arg1: *mut libc::c_void)>,
        full_merge: ::std::option::Option<
            unsafe extern "C" fn(
                arg1: *mut libc::c_void,
                key: *const libc::c_char,
                key_length: usize,
                existing_value: *const libc::c_char,
                existing_value_length: usize,
                operands_list: *const *const libc::c_char,
                operands_list_length: *const usize,
                num_operands: libc::c_int,
                success: *mut libc::c_uchar,
                new_value_length: *mut usize,
            ) -> *mut libc::c_char,
        >,
        partial_merge: ::std::option::Option<
            unsafe extern "C" fn(
                arg1: *mut libc::c_void,
                key: *const libc::c_char,
                key_length: usize,
                operands_list: *const *const libc::c_char,
                operands_list_length: *const usize,
                num_operands: libc::c_int,
                success: *mut libc::c_uchar,
                new_value_length: *mut usize,
            ) -> *mut libc::c_char,
        >,
        delete_value: ::std::option::Option<
            unsafe extern "C" fn(
                arg1: *mut libc::c_void,
                value: *const libc::c_char,
                value_length: usize,
            ),
        >,
        name: ::std::option::Option<
            unsafe extern "C" fn(arg1: *mut libc::c_void) -> *const libc::c_char,
        >,
    ) -> *mut crocksdb_mergeoperator_t;
}
extern "C" {
    pub fn crocksdb_mergeoperator_destroy(arg1: *mut crocksdb_mergeoperator_t);
}
extern "C" {
    pub fn crocksdb_readoptions_set_table_filter(
        arg1: *mut rocksdb_ReadOptions,
        arg2: *mut libc::c_void,
        table_filter: ::std::option::Option<
            unsafe extern "C" fn(
                arg1: *mut libc::c_void,
                arg2: *const crocksdb_table_properties_t,
            ) -> libc::c_uchar,
        >,
        destory: ::std::option::Option<unsafe extern "C" fn(arg1: *mut libc::c_void)>,
    );
}
extern "C" {
    pub fn crocksdb_writeoptions_init(arg1: *mut rocksdb_WriteOptions);
}
extern "C" {
    pub fn crocksdb_compactrangeoptions_init(arg1: *mut rocksdb_CompactRangeOptions);
}
extern "C" {
    pub fn crocksdb_flushoptions_init(arg1: *mut rocksdb_FlushOptions);
}
extern "C" {
    pub fn crocksdb_jemalloc_nodump_allocator_create(
        s: *mut rocksdb_Status,
    ) -> *mut crocksdb_memory_allocator_t;
}
extern "C" {
    pub fn crocksdb_memory_allocator_destroy(arg1: *mut crocksdb_memory_allocator_t);
}
extern "C" {
    pub fn crocksdb_lru_cache_options_create() -> *mut crocksdb_lru_cache_options_t;
}
extern "C" {
    pub fn crocksdb_lru_cache_options_destroy(arg1: *mut crocksdb_lru_cache_options_t);
}
extern "C" {
    pub fn crocksdb_lru_cache_options_set_capacity(
        arg1: *mut crocksdb_lru_cache_options_t,
        arg2: usize,
    );
}
extern "C" {
    pub fn crocksdb_lru_cache_options_set_num_shard_bits(
        arg1: *mut crocksdb_lru_cache_options_t,
        arg2: libc::c_int,
    );
}
extern "C" {
    pub fn crocksdb_lru_cache_options_set_strict_capacity_limit(
        arg1: *mut crocksdb_lru_cache_options_t,
        arg2: libc::c_uchar,
    );
}
extern "C" {
    pub fn crocksdb_lru_cache_options_set_high_pri_pool_ratio(
        arg1: *mut crocksdb_lru_cache_options_t,
        arg2: f64,
    );
}
extern "C" {
    pub fn crocksdb_lru_cache_options_set_memory_allocator(
        arg1: *mut crocksdb_lru_cache_options_t,
        arg2: *mut crocksdb_memory_allocator_t,
    );
}
extern "C" {
    pub fn crocksdb_cache_create_lru(
        arg1: *mut crocksdb_lru_cache_options_t,
    ) -> *mut crocksdb_cache_t;
}
extern "C" {
    pub fn crocksdb_cache_destroy(cache: *mut crocksdb_cache_t);
}
extern "C" {
    pub fn crocksdb_cache_set_capacity(cache: *mut crocksdb_cache_t, capacity: usize);
}
extern "C" {
    pub fn crocksdb_default_env_create() -> *mut rocksdb_Env;
}
extern "C" {
    pub fn crocksdb_mem_env_create(arg1: *mut rocksdb_Env) -> *mut rocksdb_Env;
}
extern "C" {
    pub fn crocksdb_ctr_encrypted_env_create(
        base_env: *mut rocksdb_Env,
        ciphertext: *const libc::c_char,
        ciphertext_len: usize,
    ) -> *mut rocksdb_Env;
}
extern "C" {
    pub fn crocksdb_env_set_background_threads(
        env: *mut rocksdb_Env,
        n: libc::c_int,
        pri: rocksdb_Env_Priority,
    );
}
extern "C" {
    pub fn crocksdb_env_join_all_threads(env: *mut rocksdb_Env);
}
extern "C" {
    pub fn crocksdb_env_file_exists(
        env: *mut rocksdb_Env,
        path: rocksdb_Slice,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_env_delete_file(
        env: *mut rocksdb_Env,
        path: rocksdb_Slice,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_env_destroy(arg1: *mut rocksdb_Env);
}
extern "C" {
    pub fn crocksdb_envoptions_create() -> *mut crocksdb_envoptions_t;
}
extern "C" {
    pub fn crocksdb_envoptions_destroy(opt: *mut crocksdb_envoptions_t);
}
extern "C" {
    pub fn crocksdb_sequential_file_create(
        env: *mut rocksdb_Env,
        path: rocksdb_Slice,
        opts: *const crocksdb_envoptions_t,
        s: *mut rocksdb_Status,
    ) -> *mut crocksdb_sequential_file_t;
}
extern "C" {
    pub fn crocksdb_sequential_file_read(
        arg1: *mut crocksdb_sequential_file_t,
        n: usize,
        buf: *mut libc::c_char,
        s: *mut rocksdb_Status,
    ) -> usize;
}
extern "C" {
    pub fn crocksdb_sequential_file_skip(
        arg1: *mut crocksdb_sequential_file_t,
        n: usize,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_sequential_file_destroy(arg1: *mut crocksdb_sequential_file_t);
}
pub type crocksdb_encryption_key_manager_get_file_cb = ::std::option::Option<
    unsafe extern "C" fn(
        state: *mut libc::c_void,
        fname: rocksdb_Slice,
        file_info: *mut crocksdb_file_encryption_info_t,
        arg1: *mut rocksdb_Status,
    ),
>;
pub type crocksdb_encryption_key_manager_new_file_cb = ::std::option::Option<
    unsafe extern "C" fn(
        state: *mut libc::c_void,
        fname: rocksdb_Slice,
        file_info: *mut crocksdb_file_encryption_info_t,
        arg1: *mut rocksdb_Status,
    ),
>;
pub type crocksdb_encryption_key_manager_delete_file_cb = ::std::option::Option<
    unsafe extern "C" fn(state: *mut libc::c_void, fname: rocksdb_Slice, arg1: *mut rocksdb_Status),
>;
pub type crocksdb_encryption_key_manager_link_file_cb = ::std::option::Option<
    unsafe extern "C" fn(
        state: *mut libc::c_void,
        src_fname: rocksdb_Slice,
        dst_fname: rocksdb_Slice,
        arg1: *mut rocksdb_Status,
    ),
>;
extern "C" {
    pub fn crocksdb_encryption_key_manager_create(
        state: *mut libc::c_void,
        destructor: ::std::option::Option<unsafe extern "C" fn(arg1: *mut libc::c_void)>,
        get_file: crocksdb_encryption_key_manager_get_file_cb,
        new_file: crocksdb_encryption_key_manager_new_file_cb,
        delete_file: crocksdb_encryption_key_manager_delete_file_cb,
        link_file: crocksdb_encryption_key_manager_link_file_cb,
    ) -> *mut rocksdb_encryption_KeyManager;
}
extern "C" {
    pub fn crocksdb_encryption_key_manager_destroy(arg1: *mut rocksdb_encryption_KeyManager);
}
extern "C" {
    pub fn crocksdb_key_managed_encrypted_env_create(
        arg1: *mut rocksdb_Env,
        arg2: *mut rocksdb_encryption_KeyManager,
    ) -> *mut rocksdb_Env;
}
pub type crocksdb_file_system_inspector_read_cb = ::std::option::Option<
    unsafe extern "C" fn(state: *mut libc::c_void, len: usize, s: *mut rocksdb_Status) -> usize,
>;
pub type crocksdb_file_system_inspector_write_cb = ::std::option::Option<
    unsafe extern "C" fn(state: *mut libc::c_void, len: usize, s: *mut rocksdb_Status) -> usize,
>;
extern "C" {
    pub fn crocksdb_file_system_inspector_create(
        state: *mut libc::c_void,
        destructor: ::std::option::Option<unsafe extern "C" fn(arg1: *mut libc::c_void)>,
        read: crocksdb_file_system_inspector_read_cb,
        write: crocksdb_file_system_inspector_write_cb,
    ) -> *mut crocksdb_file_system_inspector_t;
}
extern "C" {
    pub fn crocksdb_file_system_inspector_destroy(arg1: *mut crocksdb_file_system_inspector_t);
}
extern "C" {
    pub fn crocksdb_file_system_inspector_read(
        inspector: *mut crocksdb_file_system_inspector_t,
        len: usize,
        s: *mut rocksdb_Status,
    ) -> usize;
}
extern "C" {
    pub fn crocksdb_file_system_inspector_write(
        inspector: *mut crocksdb_file_system_inspector_t,
        len: usize,
        s: *mut rocksdb_Status,
    ) -> usize;
}
extern "C" {
    pub fn crocksdb_file_system_inspected_env_create(
        arg1: *mut rocksdb_Env,
        arg2: *mut crocksdb_file_system_inspector_t,
    ) -> *mut rocksdb_Env;
}
extern "C" {
    pub fn crocksdb_sstfilereader_create(
        io_options: *const crocksdb_options_t,
    ) -> *mut crocksdb_sstfilereader_t;
}
extern "C" {
    pub fn crocksdb_sstfilereader_open(
        reader: *mut crocksdb_sstfilereader_t,
        name: *const libc::c_char,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_sstfilereader_new_iterator(
        reader: *mut crocksdb_sstfilereader_t,
        options: *const rocksdb_ReadOptions,
    ) -> *mut crocksdb_iterator_t;
}
extern "C" {
    pub fn crocksdb_sstfilereader_read_table_properties(
        reader: *const crocksdb_sstfilereader_t,
        ctx: *mut libc::c_void,
        cb: ::std::option::Option<
            unsafe extern "C" fn(arg1: *mut libc::c_void, arg2: *const crocksdb_table_properties_t),
        >,
    );
}
extern "C" {
    pub fn crocksdb_sstfilereader_verify_checksum(
        reader: *mut crocksdb_sstfilereader_t,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_sstfilereader_destroy(reader: *mut crocksdb_sstfilereader_t);
}
extern "C" {
    pub fn crocksdb_sstfilewriter_create(
        env: *const crocksdb_envoptions_t,
        io_options: *const crocksdb_options_t,
    ) -> *mut crocksdb_sstfilewriter_t;
}
extern "C" {
    pub fn crocksdb_sstfilewriter_create_cf(
        env: *const crocksdb_envoptions_t,
        io_options: *const crocksdb_options_t,
        column_family: *mut crocksdb_column_family_handle_t,
    ) -> *mut crocksdb_sstfilewriter_t;
}
extern "C" {
    pub fn crocksdb_sstfilewriter_open(
        writer: *mut crocksdb_sstfilewriter_t,
        name: *const libc::c_char,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_sstfilewriter_put(
        writer: *mut crocksdb_sstfilewriter_t,
        key: *const libc::c_char,
        keylen: usize,
        val: *const libc::c_char,
        vallen: usize,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_sstfilewriter_merge(
        writer: *mut crocksdb_sstfilewriter_t,
        key: *const libc::c_char,
        keylen: usize,
        val: *const libc::c_char,
        vallen: usize,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_sstfilewriter_delete(
        writer: *mut crocksdb_sstfilewriter_t,
        key: *const libc::c_char,
        keylen: usize,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_sstfilewriter_delete_range(
        writer: *mut crocksdb_sstfilewriter_t,
        begin_key: *const libc::c_char,
        begin_keylen: usize,
        end_key: *const libc::c_char,
        end_keylen: usize,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_sstfilewriter_finish(
        writer: *mut crocksdb_sstfilewriter_t,
        info: *mut crocksdb_externalsstfileinfo_t,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_sstfilewriter_file_size(writer: *mut crocksdb_sstfilewriter_t) -> u64;
}
extern "C" {
    pub fn crocksdb_sstfilewriter_destroy(writer: *mut crocksdb_sstfilewriter_t);
}
extern "C" {
    pub fn crocksdb_externalsstfileinfo_create() -> *mut crocksdb_externalsstfileinfo_t;
}
extern "C" {
    pub fn crocksdb_externalsstfileinfo_destroy(arg1: *mut crocksdb_externalsstfileinfo_t);
}
extern "C" {
    pub fn crocksdb_externalsstfileinfo_file_path(
        arg1: *mut crocksdb_externalsstfileinfo_t,
        arg2: *mut usize,
    ) -> *const libc::c_char;
}
extern "C" {
    pub fn crocksdb_externalsstfileinfo_smallest_key(
        arg1: *mut crocksdb_externalsstfileinfo_t,
        arg2: *mut usize,
    ) -> *const libc::c_char;
}
extern "C" {
    pub fn crocksdb_externalsstfileinfo_largest_key(
        arg1: *mut crocksdb_externalsstfileinfo_t,
        arg2: *mut usize,
    ) -> *const libc::c_char;
}
extern "C" {
    pub fn crocksdb_externalsstfileinfo_sequence_number(
        arg1: *mut crocksdb_externalsstfileinfo_t,
    ) -> u64;
}
extern "C" {
    pub fn crocksdb_externalsstfileinfo_file_size(arg1: *mut crocksdb_externalsstfileinfo_t)
        -> u64;
}
extern "C" {
    pub fn crocksdb_externalsstfileinfo_num_entries(
        arg1: *mut crocksdb_externalsstfileinfo_t,
    ) -> u64;
}
extern "C" {
    pub fn crocksdb_ingestexternalfileoptions_init(arg1: *mut rocksdb_IngestExternalFileOptions);
}
extern "C" {
    pub fn crocksdb_ingest_external_file(
        db: *mut crocksdb_t,
        file_list: *const *const libc::c_char,
        list_len: usize,
        opt: *const rocksdb_IngestExternalFileOptions,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_ingest_external_file_cf(
        db: *mut crocksdb_t,
        handle: *mut crocksdb_column_family_handle_t,
        file_list: *const *const libc::c_char,
        list_len: usize,
        opt: *const rocksdb_IngestExternalFileOptions,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_ingest_external_file_optimized(
        db: *mut crocksdb_t,
        handle: *mut crocksdb_column_family_handle_t,
        file_list: *const *const libc::c_char,
        list_len: usize,
        opt: *const rocksdb_IngestExternalFileOptions,
        s: *mut rocksdb_Status,
    ) -> libc::c_uchar;
}
extern "C" {
    pub fn crocksdb_slicetransform_create(
        state: *mut libc::c_void,
        destructor: ::std::option::Option<unsafe extern "C" fn(arg1: *mut libc::c_void)>,
        transform: ::std::option::Option<
            unsafe extern "C" fn(
                arg1: *mut libc::c_void,
                key: *const libc::c_char,
                length: usize,
                dst_length: *mut usize,
            ) -> *mut libc::c_char,
        >,
        in_domain: ::std::option::Option<
            unsafe extern "C" fn(
                arg1: *mut libc::c_void,
                key: *const libc::c_char,
                length: usize,
            ) -> libc::c_uchar,
        >,
        in_range: ::std::option::Option<
            unsafe extern "C" fn(
                arg1: *mut libc::c_void,
                key: *const libc::c_char,
                length: usize,
            ) -> libc::c_uchar,
        >,
        name: ::std::option::Option<
            unsafe extern "C" fn(arg1: *mut libc::c_void) -> *const libc::c_char,
        >,
    ) -> *mut crocksdb_slicetransform_t;
}
extern "C" {
    pub fn crocksdb_slicetransform_create_fixed_prefix(
        arg1: usize,
    ) -> *mut crocksdb_slicetransform_t;
}
extern "C" {
    pub fn crocksdb_slicetransform_create_noop() -> *mut crocksdb_slicetransform_t;
}
extern "C" {
    pub fn crocksdb_slicetransform_destroy(arg1: *mut crocksdb_slicetransform_t);
}
extern "C" {
    pub fn crocksdb_universal_compaction_options_create(
    ) -> *mut crocksdb_universal_compaction_options_t;
}
extern "C" {
    pub fn crocksdb_universal_compaction_options_set_size_ratio(
        arg1: *mut crocksdb_universal_compaction_options_t,
        arg2: libc::c_int,
    );
}
extern "C" {
    pub fn crocksdb_universal_compaction_options_set_min_merge_width(
        arg1: *mut crocksdb_universal_compaction_options_t,
        arg2: libc::c_int,
    );
}
extern "C" {
    pub fn crocksdb_universal_compaction_options_set_max_merge_width(
        arg1: *mut crocksdb_universal_compaction_options_t,
        arg2: libc::c_int,
    );
}
extern "C" {
    pub fn crocksdb_universal_compaction_options_set_max_size_amplification_percent(
        arg1: *mut crocksdb_universal_compaction_options_t,
        arg2: libc::c_int,
    );
}
extern "C" {
    pub fn crocksdb_universal_compaction_options_set_compression_size_percent(
        arg1: *mut crocksdb_universal_compaction_options_t,
        arg2: libc::c_int,
    );
}
extern "C" {
    pub fn crocksdb_universal_compaction_options_set_stop_style(
        arg1: *mut crocksdb_universal_compaction_options_t,
        arg2: rocksdb_CompactionStopStyle,
    );
}
extern "C" {
    pub fn crocksdb_universal_compaction_options_destroy(
        arg1: *mut crocksdb_universal_compaction_options_t,
    );
}
extern "C" {
    pub fn crocksdb_fifo_compaction_options_create() -> *mut crocksdb_fifo_compaction_options_t;
}
extern "C" {
    pub fn crocksdb_fifo_compaction_options_set_max_table_files_size(
        fifo_opts: *mut crocksdb_fifo_compaction_options_t,
        size: u64,
    );
}
extern "C" {
    pub fn crocksdb_fifo_compaction_options_set_allow_compaction(
        fifo_opts: *mut crocksdb_fifo_compaction_options_t,
        allow_compaction: libc::c_uchar,
    );
}
extern "C" {
    pub fn crocksdb_fifo_compaction_options_destroy(
        fifo_opts: *mut crocksdb_fifo_compaction_options_t,
    );
}
extern "C" {
    pub fn crocksdb_livefiles_count(arg1: *const crocksdb_livefiles_t) -> usize;
}
extern "C" {
    pub fn crocksdb_livefiles_name(
        arg1: *const crocksdb_livefiles_t,
        index: libc::c_int,
    ) -> *const libc::c_char;
}
extern "C" {
    pub fn crocksdb_livefiles_level(
        arg1: *const crocksdb_livefiles_t,
        index: libc::c_int,
    ) -> libc::c_int;
}
extern "C" {
    pub fn crocksdb_livefiles_size(arg1: *const crocksdb_livefiles_t, index: libc::c_int) -> usize;
}
extern "C" {
    pub fn crocksdb_livefiles_smallestkey(
        arg1: *const crocksdb_livefiles_t,
        index: libc::c_int,
        size: *mut usize,
    ) -> *const libc::c_char;
}
extern "C" {
    pub fn crocksdb_livefiles_largestkey(
        arg1: *const crocksdb_livefiles_t,
        index: libc::c_int,
        size: *mut usize,
    ) -> *const libc::c_char;
}
extern "C" {
    pub fn crocksdb_livefiles_destroy(arg1: *const crocksdb_livefiles_t);
}
extern "C" {
    pub fn crocksdb_get_options_from_string(
        base_options: *const crocksdb_options_t,
        opts_str: *const libc::c_char,
        new_options: *mut crocksdb_options_t,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_delete_files_in_range(
        db: *mut crocksdb_t,
        start_key: *const libc::c_char,
        start_key_len: usize,
        limit_key: *const libc::c_char,
        limit_key_len: usize,
        include_end: libc::c_uchar,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_delete_files_in_range_cf(
        db: *mut crocksdb_t,
        column_family: *mut crocksdb_column_family_handle_t,
        start_key: *const libc::c_char,
        start_key_len: usize,
        limit_key: *const libc::c_char,
        limit_key_len: usize,
        include_end: libc::c_uchar,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_delete_files_in_ranges_cf(
        db: *mut crocksdb_t,
        cf: *mut crocksdb_column_family_handle_t,
        start_keys: *const *const libc::c_char,
        start_keys_lens: *const usize,
        limit_keys: *const *const libc::c_char,
        limit_keys_lens: *const usize,
        num_ranges: usize,
        include_end: libc::c_uchar,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_free(ptr: *mut libc::c_void);
}
extern "C" {
    pub fn crocksdb_create_env_logger(
        fname: *const libc::c_char,
        env: *mut rocksdb_Env,
    ) -> *mut crocksdb_logger_t;
}
extern "C" {
    pub fn crocksdb_create_log_from_options(
        path: *const libc::c_char,
        opts: *mut crocksdb_options_t,
        s: *mut rocksdb_Status,
    ) -> *mut crocksdb_logger_t;
}
extern "C" {
    pub fn crocksdb_log_destroy(arg1: *mut crocksdb_logger_t);
}
extern "C" {
    pub fn crocksdb_get_pinned(
        db: *mut crocksdb_t,
        options: *const rocksdb_ReadOptions,
        key: *const libc::c_char,
        keylen: usize,
        s: *mut rocksdb_Status,
    ) -> *mut crocksdb_pinnableslice_t;
}
extern "C" {
    pub fn crocksdb_get_pinned_cf(
        db: *mut crocksdb_t,
        options: *const rocksdb_ReadOptions,
        column_family: *mut crocksdb_column_family_handle_t,
        key: *const libc::c_char,
        keylen: usize,
        s: *mut rocksdb_Status,
    ) -> *mut crocksdb_pinnableslice_t;
}
extern "C" {
    pub fn crocksdb_pinnableslice_destroy(v: *mut crocksdb_pinnableslice_t);
}
extern "C" {
    pub fn crocksdb_pinnableslice_value(
        t: *const crocksdb_pinnableslice_t,
        vlen: *mut usize,
    ) -> *const libc::c_char;
}
extern "C" {
    pub fn crocksdb_get_supported_compression_number() -> usize;
}
extern "C" {
    pub fn crocksdb_get_supported_compression(arg1: *mut rocksdb_CompressionType, arg2: usize);
}
extern "C" {
    pub fn crocksdb_table_properties_get_u64(
        arg1: *const crocksdb_table_properties_t,
        prop: crocksdb_table_property_t,
    ) -> u64;
}
extern "C" {
    pub fn crocksdb_table_properties_get_str(
        arg1: *const crocksdb_table_properties_t,
        prop: crocksdb_table_property_t,
        slen: *mut usize,
    ) -> *const libc::c_char;
}
extern "C" {
    pub fn crocksdb_table_properties_get_user_properties(
        arg1: *const crocksdb_table_properties_t,
    ) -> *const crocksdb_user_collected_properties_t;
}
extern "C" {
    pub fn crocksdb_user_collected_properties_get(
        props: *const crocksdb_user_collected_properties_t,
        key: *const libc::c_char,
        klen: usize,
        vlen: *mut usize,
    ) -> *const libc::c_char;
}
extern "C" {
    pub fn crocksdb_user_collected_properties_len(
        arg1: *const crocksdb_user_collected_properties_t,
    ) -> usize;
}
extern "C" {
    pub fn crocksdb_user_collected_properties_add(
        arg1: *mut crocksdb_user_collected_properties_t,
        key: *const libc::c_char,
        key_len: usize,
        value: *const libc::c_char,
        value_len: usize,
    );
}
extern "C" {
    pub fn crocksdb_user_collected_properties_iter_create(
        arg1: *const crocksdb_user_collected_properties_t,
    ) -> *mut crocksdb_user_collected_properties_iterator_t;
}
extern "C" {
    pub fn crocksdb_user_collected_properties_iter_destroy(
        arg1: *mut crocksdb_user_collected_properties_iterator_t,
    );
}
extern "C" {
    pub fn crocksdb_user_collected_properties_iter_valid(
        arg1: *const crocksdb_user_collected_properties_iterator_t,
    ) -> libc::c_uchar;
}
extern "C" {
    pub fn crocksdb_user_collected_properties_iter_next(
        arg1: *mut crocksdb_user_collected_properties_iterator_t,
    );
}
extern "C" {
    pub fn crocksdb_user_collected_properties_iter_key(
        arg1: *const crocksdb_user_collected_properties_iterator_t,
        klen: *mut usize,
    ) -> *const libc::c_char;
}
extern "C" {
    pub fn crocksdb_user_collected_properties_iter_value(
        arg1: *const crocksdb_user_collected_properties_iterator_t,
        vlen: *mut usize,
    ) -> *const libc::c_char;
}
extern "C" {
    pub fn crocksdb_table_properties_collection_len(
        arg1: *const crocksdb_table_properties_collection_t,
    ) -> usize;
}
extern "C" {
    pub fn crocksdb_table_properties_collection_destroy(
        arg1: *mut crocksdb_table_properties_collection_t,
    );
}
extern "C" {
    pub fn crocksdb_table_properties_collection_iter_create(
        arg1: *const crocksdb_table_properties_collection_t,
    ) -> *mut crocksdb_table_properties_collection_iterator_t;
}
extern "C" {
    pub fn crocksdb_table_properties_collection_iter_destroy(
        arg1: *mut crocksdb_table_properties_collection_iterator_t,
    );
}
extern "C" {
    pub fn crocksdb_table_properties_collection_iter_valid(
        arg1: *const crocksdb_table_properties_collection_iterator_t,
    ) -> libc::c_uchar;
}
extern "C" {
    pub fn crocksdb_table_properties_collection_iter_next(
        arg1: *mut crocksdb_table_properties_collection_iterator_t,
    );
}
extern "C" {
    pub fn crocksdb_table_properties_collection_iter_key(
        arg1: *const crocksdb_table_properties_collection_iterator_t,
        klen: *mut usize,
    ) -> *const libc::c_char;
}
extern "C" {
    pub fn crocksdb_table_properties_collection_iter_value(
        arg1: *const crocksdb_table_properties_collection_iterator_t,
    ) -> *const crocksdb_table_properties_t;
}
extern "C" {
    pub fn crocksdb_table_properties_collector_create(
        state: *mut libc::c_void,
        name: ::std::option::Option<
            unsafe extern "C" fn(arg1: *mut libc::c_void) -> *const libc::c_char,
        >,
        destruct: ::std::option::Option<unsafe extern "C" fn(arg1: *mut libc::c_void)>,
        add: ::std::option::Option<
            unsafe extern "C" fn(
                arg1: *mut libc::c_void,
                key: *const libc::c_char,
                key_len: usize,
                value: *const libc::c_char,
                value_len: usize,
                entry_type: libc::c_int,
                seq: u64,
                file_size: u64,
            ),
        >,
        finish: ::std::option::Option<
            unsafe extern "C" fn(
                arg1: *mut libc::c_void,
                props: *mut crocksdb_user_collected_properties_t,
            ),
        >,
    ) -> *mut crocksdb_table_properties_collector_t;
}
extern "C" {
    pub fn crocksdb_table_properties_collector_destroy(
        arg1: *mut crocksdb_table_properties_collector_t,
    );
}
extern "C" {
    pub fn crocksdb_table_properties_collector_factory_create(
        state: *mut libc::c_void,
        name: ::std::option::Option<
            unsafe extern "C" fn(arg1: *mut libc::c_void) -> *const libc::c_char,
        >,
        destruct: ::std::option::Option<unsafe extern "C" fn(arg1: *mut libc::c_void)>,
        create_table_properties_collector: ::std::option::Option<
            unsafe extern "C" fn(
                arg1: *mut libc::c_void,
                cf: u32,
            ) -> *mut crocksdb_table_properties_collector_t,
        >,
    ) -> *mut crocksdb_table_properties_collector_factory_t;
}
extern "C" {
    pub fn crocksdb_table_properties_collector_factory_destroy(
        arg1: *mut crocksdb_table_properties_collector_factory_t,
    );
}
extern "C" {
    pub fn crocksdb_options_add_table_properties_collector_factory(
        opt: *mut crocksdb_options_t,
        f: *mut crocksdb_table_properties_collector_factory_t,
    );
}
extern "C" {
    pub fn crocksdb_options_set_compact_on_deletion(
        opt: *mut crocksdb_options_t,
        sliding_window_size: usize,
        deletion_trigger: usize,
    );
}
extern "C" {
    pub fn crocksdb_get_properties_of_all_tables(
        db: *mut crocksdb_t,
        s: *mut rocksdb_Status,
    ) -> *mut crocksdb_table_properties_collection_t;
}
extern "C" {
    pub fn crocksdb_get_properties_of_all_tables_cf(
        db: *mut crocksdb_t,
        cf: *mut crocksdb_column_family_handle_t,
        s: *mut rocksdb_Status,
    ) -> *mut crocksdb_table_properties_collection_t;
}
extern "C" {
    pub fn crocksdb_get_properties_of_tables_in_range(
        db: *mut crocksdb_t,
        cf: *mut crocksdb_column_family_handle_t,
        num_ranges: libc::c_int,
        start_keys: *const *const libc::c_char,
        start_keys_lens: *const usize,
        limit_keys: *const *const libc::c_char,
        limit_keys_lens: *const usize,
        s: *mut rocksdb_Status,
    ) -> *mut crocksdb_table_properties_collection_t;
}
extern "C" {
    pub fn crocksdb_keyversions_destroy(kvs: *mut crocksdb_keyversions_t);
}
extern "C" {
    pub fn crocksdb_get_all_key_versions(
        db: *mut crocksdb_t,
        begin_key: *const libc::c_char,
        begin_keylen: usize,
        end_key: *const libc::c_char,
        end_keylen: usize,
        s: *mut rocksdb_Status,
    ) -> *mut crocksdb_keyversions_t;
}
extern "C" {
    pub fn crocksdb_keyversions_count(kvs: *const crocksdb_keyversions_t) -> usize;
}
extern "C" {
    pub fn crocksdb_keyversions_key(
        kvs: *const crocksdb_keyversions_t,
        index: libc::c_int,
    ) -> *const libc::c_char;
}
extern "C" {
    pub fn crocksdb_keyversions_value(
        kvs: *const crocksdb_keyversions_t,
        index: libc::c_int,
    ) -> *const libc::c_char;
}
extern "C" {
    pub fn crocksdb_keyversions_seq(kvs: *const crocksdb_keyversions_t, index: libc::c_int) -> u64;
}
extern "C" {
    pub fn crocksdb_keyversions_type(
        kvs: *const crocksdb_keyversions_t,
        index: libc::c_int,
    ) -> libc::c_int;
}
extern "C" {
    pub fn crocksdb_set_external_sst_file_global_seq_no(
        db: *mut crocksdb_t,
        column_family: *mut crocksdb_column_family_handle_t,
        file: *const libc::c_char,
        seq_no: u64,
        s: *mut rocksdb_Status,
    ) -> u64;
}
extern "C" {
    pub fn crocksdb_get_column_family_meta_data(
        db: *mut crocksdb_t,
        cf: *mut crocksdb_column_family_handle_t,
        arg1: *mut crocksdb_column_family_meta_data_t,
    );
}
extern "C" {
    pub fn crocksdb_column_family_meta_data_create() -> *mut crocksdb_column_family_meta_data_t;
}
extern "C" {
    pub fn crocksdb_column_family_meta_data_destroy(arg1: *mut crocksdb_column_family_meta_data_t);
}
extern "C" {
    pub fn crocksdb_column_family_meta_data_level_count(
        arg1: *const crocksdb_column_family_meta_data_t,
    ) -> usize;
}
extern "C" {
    pub fn crocksdb_column_family_meta_data_level_data(
        arg1: *const crocksdb_column_family_meta_data_t,
        n: usize,
    ) -> *const crocksdb_level_meta_data_t;
}
extern "C" {
    pub fn crocksdb_level_meta_data_file_count(arg1: *const crocksdb_level_meta_data_t) -> usize;
}
extern "C" {
    pub fn crocksdb_level_meta_data_file_data(
        arg1: *const crocksdb_level_meta_data_t,
        n: usize,
    ) -> *const crocksdb_sst_file_meta_data_t;
}
extern "C" {
    pub fn crocksdb_sst_file_meta_data_size(arg1: *const crocksdb_sst_file_meta_data_t) -> usize;
}
extern "C" {
    pub fn crocksdb_sst_file_meta_data_name(
        arg1: *const crocksdb_sst_file_meta_data_t,
    ) -> *const libc::c_char;
}
extern "C" {
    pub fn crocksdb_sst_file_meta_data_smallestkey(
        arg1: *const crocksdb_sst_file_meta_data_t,
        arg2: *mut usize,
    ) -> *const libc::c_char;
}
extern "C" {
    pub fn crocksdb_sst_file_meta_data_largestkey(
        arg1: *const crocksdb_sst_file_meta_data_t,
        arg2: *mut usize,
    ) -> *const libc::c_char;
}
extern "C" {
    pub fn crocksdb_compaction_options_init(arg1: *mut rocksdb_CompactionOptions);
}
extern "C" {
    pub fn crocksdb_compact_files_cf(
        arg1: *mut crocksdb_t,
        arg2: *mut crocksdb_column_family_handle_t,
        arg3: *const rocksdb_CompactionOptions,
        input_file_names: *mut *const libc::c_char,
        input_file_count: usize,
        output_level: libc::c_int,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_get_perf_level() -> rocksdb_PerfLevel;
}
extern "C" {
    pub fn crocksdb_set_perf_level(level: rocksdb_PerfLevel);
}
extern "C" {
    pub fn crocksdb_get_perf_context() -> *mut crocksdb_perf_context_t;
}
extern "C" {
    pub fn crocksdb_perf_context_reset(arg1: *mut crocksdb_perf_context_t);
}
extern "C" {
    pub fn crocksdb_perf_context_user_key_comparison_count(
        arg1: *mut crocksdb_perf_context_t,
    ) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_block_cache_hit_count(arg1: *mut crocksdb_perf_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_block_read_count(arg1: *mut crocksdb_perf_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_block_read_byte(arg1: *mut crocksdb_perf_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_block_read_time(arg1: *mut crocksdb_perf_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_block_cache_index_hit_count(
        arg1: *mut crocksdb_perf_context_t,
    ) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_index_block_read_count(arg1: *mut crocksdb_perf_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_block_cache_filter_hit_count(
        arg1: *mut crocksdb_perf_context_t,
    ) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_filter_block_read_count(arg1: *mut crocksdb_perf_context_t)
        -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_block_checksum_time(arg1: *mut crocksdb_perf_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_block_decompress_time(arg1: *mut crocksdb_perf_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_get_read_bytes(arg1: *mut crocksdb_perf_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_multiget_read_bytes(arg1: *mut crocksdb_perf_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_iter_read_bytes(arg1: *mut crocksdb_perf_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_internal_key_skipped_count(
        arg1: *mut crocksdb_perf_context_t,
    ) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_internal_delete_skipped_count(
        arg1: *mut crocksdb_perf_context_t,
    ) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_internal_recent_skipped_count(
        arg1: *mut crocksdb_perf_context_t,
    ) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_internal_merge_count(arg1: *mut crocksdb_perf_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_get_snapshot_time(arg1: *mut crocksdb_perf_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_get_from_memtable_time(arg1: *mut crocksdb_perf_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_get_from_memtable_count(arg1: *mut crocksdb_perf_context_t)
        -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_get_post_process_time(arg1: *mut crocksdb_perf_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_get_from_output_files_time(
        arg1: *mut crocksdb_perf_context_t,
    ) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_seek_on_memtable_time(arg1: *mut crocksdb_perf_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_seek_on_memtable_count(arg1: *mut crocksdb_perf_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_next_on_memtable_count(arg1: *mut crocksdb_perf_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_prev_on_memtable_count(arg1: *mut crocksdb_perf_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_seek_child_seek_time(arg1: *mut crocksdb_perf_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_seek_child_seek_count(arg1: *mut crocksdb_perf_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_seek_min_heap_time(arg1: *mut crocksdb_perf_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_seek_max_heap_time(arg1: *mut crocksdb_perf_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_seek_internal_seek_time(arg1: *mut crocksdb_perf_context_t)
        -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_find_next_user_entry_time(
        arg1: *mut crocksdb_perf_context_t,
    ) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_write_wal_time(arg1: *mut crocksdb_perf_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_write_memtable_time(arg1: *mut crocksdb_perf_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_write_delay_time(arg1: *mut crocksdb_perf_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_write_pre_and_post_process_time(
        arg1: *mut crocksdb_perf_context_t,
    ) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_db_mutex_lock_nanos(arg1: *mut crocksdb_perf_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_write_thread_wait_nanos(arg1: *mut crocksdb_perf_context_t)
        -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_write_scheduling_flushes_compactions_time(
        arg1: *mut crocksdb_perf_context_t,
    ) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_db_condition_wait_nanos(arg1: *mut crocksdb_perf_context_t)
        -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_merge_operator_time_nanos(
        arg1: *mut crocksdb_perf_context_t,
    ) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_read_index_block_nanos(arg1: *mut crocksdb_perf_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_read_filter_block_nanos(arg1: *mut crocksdb_perf_context_t)
        -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_new_table_block_iter_nanos(
        arg1: *mut crocksdb_perf_context_t,
    ) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_new_table_iterator_nanos(
        arg1: *mut crocksdb_perf_context_t,
    ) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_block_seek_nanos(arg1: *mut crocksdb_perf_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_find_table_nanos(arg1: *mut crocksdb_perf_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_bloom_memtable_hit_count(
        arg1: *mut crocksdb_perf_context_t,
    ) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_bloom_memtable_miss_count(
        arg1: *mut crocksdb_perf_context_t,
    ) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_bloom_sst_hit_count(arg1: *mut crocksdb_perf_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_bloom_sst_miss_count(arg1: *mut crocksdb_perf_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_env_new_sequential_file_nanos(
        arg1: *mut crocksdb_perf_context_t,
    ) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_env_new_random_access_file_nanos(
        arg1: *mut crocksdb_perf_context_t,
    ) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_env_new_writable_file_nanos(
        arg1: *mut crocksdb_perf_context_t,
    ) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_env_reuse_writable_file_nanos(
        arg1: *mut crocksdb_perf_context_t,
    ) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_env_new_random_rw_file_nanos(
        arg1: *mut crocksdb_perf_context_t,
    ) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_env_new_directory_nanos(arg1: *mut crocksdb_perf_context_t)
        -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_env_file_exists_nanos(arg1: *mut crocksdb_perf_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_env_get_children_nanos(arg1: *mut crocksdb_perf_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_env_get_children_file_attributes_nanos(
        arg1: *mut crocksdb_perf_context_t,
    ) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_env_delete_file_nanos(arg1: *mut crocksdb_perf_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_env_create_dir_nanos(arg1: *mut crocksdb_perf_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_env_create_dir_if_missing_nanos(
        arg1: *mut crocksdb_perf_context_t,
    ) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_env_delete_dir_nanos(arg1: *mut crocksdb_perf_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_env_get_file_size_nanos(arg1: *mut crocksdb_perf_context_t)
        -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_env_get_file_modification_time_nanos(
        arg1: *mut crocksdb_perf_context_t,
    ) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_env_rename_file_nanos(arg1: *mut crocksdb_perf_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_env_link_file_nanos(arg1: *mut crocksdb_perf_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_env_lock_file_nanos(arg1: *mut crocksdb_perf_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_env_unlock_file_nanos(arg1: *mut crocksdb_perf_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_env_new_logger_nanos(arg1: *mut crocksdb_perf_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_get_cpu_nanos(arg1: *mut crocksdb_perf_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_iter_next_cpu_nanos(arg1: *mut crocksdb_perf_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_iter_prev_cpu_nanos(arg1: *mut crocksdb_perf_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_iter_seek_cpu_nanos(arg1: *mut crocksdb_perf_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_encrypt_data_nanos(arg1: *mut crocksdb_perf_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_perf_context_decrypt_data_nanos(arg1: *mut crocksdb_perf_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_get_iostats_context() -> *mut crocksdb_iostats_context_t;
}
extern "C" {
    pub fn crocksdb_iostats_context_reset(arg1: *mut crocksdb_iostats_context_t);
}
extern "C" {
    pub fn crocksdb_iostats_context_bytes_written(arg1: *mut crocksdb_iostats_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_iostats_context_bytes_read(arg1: *mut crocksdb_iostats_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_iostats_context_open_nanos(arg1: *mut crocksdb_iostats_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_iostats_context_allocate_nanos(arg1: *mut crocksdb_iostats_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_iostats_context_write_nanos(arg1: *mut crocksdb_iostats_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_iostats_context_read_nanos(arg1: *mut crocksdb_iostats_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_iostats_context_range_sync_nanos(arg1: *mut crocksdb_iostats_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_iostats_context_fsync_nanos(arg1: *mut crocksdb_iostats_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_iostats_context_prepare_write_nanos(
        arg1: *mut crocksdb_iostats_context_t,
    ) -> u64;
}
extern "C" {
    pub fn crocksdb_iostats_context_logger_nanos(arg1: *mut crocksdb_iostats_context_t) -> u64;
}
extern "C" {
    pub fn crocksdb_sst_partitioner_request_create() -> *mut crocksdb_sst_partitioner_request_t;
}
extern "C" {
    pub fn crocksdb_sst_partitioner_request_destroy(req: *mut crocksdb_sst_partitioner_request_t);
}
extern "C" {
    pub fn crocksdb_sst_partitioner_request_prev_user_key(
        req: *mut crocksdb_sst_partitioner_request_t,
        len: *mut usize,
    ) -> *const libc::c_char;
}
extern "C" {
    pub fn crocksdb_sst_partitioner_request_current_user_key(
        req: *mut crocksdb_sst_partitioner_request_t,
        len: *mut usize,
    ) -> *const libc::c_char;
}
extern "C" {
    pub fn crocksdb_sst_partitioner_request_current_output_file_size(
        req: *mut crocksdb_sst_partitioner_request_t,
    ) -> u64;
}
extern "C" {
    pub fn crocksdb_sst_partitioner_req_set_prev_user_key(
        req: *mut crocksdb_sst_partitioner_request_t,
        key: *const libc::c_char,
        len: usize,
    );
}
extern "C" {
    pub fn crocksdb_sst_partitioner_req_set_current_user_key(
        req: *mut crocksdb_sst_partitioner_request_t,
        key: *const libc::c_char,
        len: usize,
    );
}
extern "C" {
    pub fn crocksdb_sst_partitioner_request_set_current_output_file_size(
        req: *mut crocksdb_sst_partitioner_request_t,
        current_output_file_size: u64,
    );
}
pub type crocksdb_sst_partitioner_should_partition_cb = ::std::option::Option<
    unsafe extern "C" fn(
        underlying: *mut libc::c_void,
        req: *mut crocksdb_sst_partitioner_request_t,
    ) -> rocksdb_PartitionerResult,
>;
pub type crocksdb_sst_partitioner_can_do_trivial_move_cb = ::std::option::Option<
    unsafe extern "C" fn(
        underlying: *mut libc::c_void,
        smallest_user_key: *const libc::c_char,
        smallest_user_key_len: usize,
        largest_user_key: *const libc::c_char,
        largest_user_key_len: usize,
    ) -> libc::c_uchar,
>;
extern "C" {
    pub fn crocksdb_sst_partitioner_create(
        underlying: *mut libc::c_void,
        destructor: ::std::option::Option<unsafe extern "C" fn(arg1: *mut libc::c_void)>,
        should_partition_cb: crocksdb_sst_partitioner_should_partition_cb,
        can_do_trivial_move_cb: crocksdb_sst_partitioner_can_do_trivial_move_cb,
    ) -> *mut crocksdb_sst_partitioner_t;
}
extern "C" {
    pub fn crocksdb_sst_partitioner_destroy(partitioner: *mut crocksdb_sst_partitioner_t);
}
extern "C" {
    pub fn crocksdb_sst_partitioner_should_partition(
        partitioner: *mut crocksdb_sst_partitioner_t,
        req: *mut crocksdb_sst_partitioner_request_t,
    ) -> rocksdb_PartitionerResult;
}
extern "C" {
    pub fn crocksdb_sst_partitioner_can_do_trivial_move(
        partitioner: *mut crocksdb_sst_partitioner_t,
        smallest_user_key: *const libc::c_char,
        smallest_user_key_len: usize,
        largest_user_key: *const libc::c_char,
        largest_user_key_len: usize,
    ) -> libc::c_uchar;
}
extern "C" {
    pub fn crocksdb_sst_partitioner_context_create() -> *mut crocksdb_sst_partitioner_context_t;
}
extern "C" {
    pub fn crocksdb_sst_partitioner_context_destroy(
        context: *mut crocksdb_sst_partitioner_context_t,
    );
}
extern "C" {
    pub fn crocksdb_sst_partitioner_context_is_full_compaction(
        context: *mut crocksdb_sst_partitioner_context_t,
    ) -> libc::c_uchar;
}
extern "C" {
    pub fn crocksdb_sst_partitioner_context_is_manual_compaction(
        context: *mut crocksdb_sst_partitioner_context_t,
    ) -> libc::c_uchar;
}
extern "C" {
    pub fn crocksdb_sst_partitioner_context_output_level(
        context: *mut crocksdb_sst_partitioner_context_t,
    ) -> libc::c_int;
}
extern "C" {
    pub fn crocksdb_sst_partitioner_context_smallest_key(
        context: *mut crocksdb_sst_partitioner_context_t,
        key_len: *mut usize,
    ) -> *const libc::c_char;
}
extern "C" {
    pub fn crocksdb_sst_partitioner_context_largest_key(
        context: *mut crocksdb_sst_partitioner_context_t,
        key_len: *mut usize,
    ) -> *const libc::c_char;
}
extern "C" {
    pub fn crocksdb_sst_partitioner_context_set_is_full_compaction(
        context: *mut crocksdb_sst_partitioner_context_t,
        is_full_compaction: libc::c_uchar,
    );
}
extern "C" {
    pub fn crocksdb_sst_partitioner_context_set_is_manual_compaction(
        context: *mut crocksdb_sst_partitioner_context_t,
        is_manual_compaction: libc::c_uchar,
    );
}
extern "C" {
    pub fn crocksdb_sst_partitioner_context_set_output_level(
        context: *mut crocksdb_sst_partitioner_context_t,
        output_level: libc::c_int,
    );
}
extern "C" {
    pub fn crocksdb_sst_partitioner_context_set_smallest_key(
        context: *mut crocksdb_sst_partitioner_context_t,
        smallest_key: *const libc::c_char,
        key_len: usize,
    );
}
extern "C" {
    pub fn crocksdb_sst_partitioner_context_set_largest_key(
        context: *mut crocksdb_sst_partitioner_context_t,
        largest_key: *const libc::c_char,
        key_len: usize,
    );
}
pub type crocksdb_sst_partitioner_factory_name_cb = ::std::option::Option<
    unsafe extern "C" fn(underlying: *mut libc::c_void) -> *const libc::c_char,
>;
pub type crocksdb_sst_partitioner_factory_create_partitioner_cb = ::std::option::Option<
    unsafe extern "C" fn(
        underlying: *mut libc::c_void,
        context: *mut crocksdb_sst_partitioner_context_t,
    ) -> *mut crocksdb_sst_partitioner_t,
>;
extern "C" {
    pub fn crocksdb_sst_partitioner_factory_create(
        underlying: *mut libc::c_void,
        destructor: ::std::option::Option<unsafe extern "C" fn(arg1: *mut libc::c_void)>,
        name_cb: crocksdb_sst_partitioner_factory_name_cb,
        create_partitioner_cb: crocksdb_sst_partitioner_factory_create_partitioner_cb,
    ) -> *mut crocksdb_sst_partitioner_factory_t;
}
extern "C" {
    pub fn crocksdb_sst_partitioner_factory_destroy(
        factory: *mut crocksdb_sst_partitioner_factory_t,
    );
}
extern "C" {
    pub fn crocksdb_sst_partitioner_factory_name(
        factory: *mut crocksdb_sst_partitioner_factory_t,
    ) -> *const libc::c_char;
}
extern "C" {
    pub fn crocksdb_sst_partitioner_factory_create_partitioner(
        factory: *mut crocksdb_sst_partitioner_factory_t,
        context: *mut crocksdb_sst_partitioner_context_t,
    ) -> *mut crocksdb_sst_partitioner_t;
}
extern "C" {
    pub fn crocksdb_run_ldb_tool(
        argc: libc::c_int,
        argv: *mut *mut libc::c_char,
        opts: *const crocksdb_options_t,
    );
}
extern "C" {
    pub fn crocksdb_run_sst_dump_tool(
        argc: libc::c_int,
        argv: *mut *mut libc::c_char,
        opts: *const crocksdb_options_t,
    );
}
#[repr(C)]
#[derive(Debug)]
pub struct ctitandb_blob_index_t {
    pub file_number: u64,
    pub blob_offset: u64,
    pub blob_size: u64,
}
#[repr(C)]
#[derive(Debug)]
pub struct ctitandb_options_t {
    _unused: [u8; 0],
}
extern "C" {
    pub fn ctitandb_open_column_families(
        name: *const libc::c_char,
        tdb_options: *const ctitandb_options_t,
        num_column_families: libc::c_int,
        column_family_names: *mut *const libc::c_char,
        titan_column_family_options: *mut *const ctitandb_options_t,
        column_family_handles: *mut *mut crocksdb_column_family_handle_t,
        s: *mut rocksdb_Status,
    ) -> *mut crocksdb_t;
}
extern "C" {
    pub fn ctitandb_create_column_family(
        db: *mut crocksdb_t,
        titan_column_family_options: *const ctitandb_options_t,
        column_family_name: *const libc::c_char,
        s: *mut rocksdb_Status,
    ) -> *mut crocksdb_column_family_handle_t;
}
extern "C" {
    pub fn ctitandb_options_create() -> *mut ctitandb_options_t;
}
extern "C" {
    pub fn ctitandb_options_destroy(arg1: *mut ctitandb_options_t);
}
extern "C" {
    pub fn ctitandb_options_copy(arg1: *mut ctitandb_options_t) -> *mut ctitandb_options_t;
}
extern "C" {
    pub fn ctitandb_options_set_rocksdb_options(
        opts: *mut ctitandb_options_t,
        rocksdb_opts: *const crocksdb_options_t,
    );
}
extern "C" {
    pub fn ctitandb_get_titan_options_cf(
        db: *const crocksdb_t,
        column_family: *mut crocksdb_column_family_handle_t,
    ) -> *mut ctitandb_options_t;
}
extern "C" {
    pub fn ctitandb_get_titan_db_options(db: *mut crocksdb_t) -> *mut ctitandb_options_t;
}
extern "C" {
    pub fn ctitandb_options_dirname(arg1: *mut ctitandb_options_t) -> *const libc::c_char;
}
extern "C" {
    pub fn ctitandb_options_set_dirname(arg1: *mut ctitandb_options_t, name: *const libc::c_char);
}
extern "C" {
    pub fn ctitandb_options_min_blob_size(arg1: *mut ctitandb_options_t) -> u64;
}
extern "C" {
    pub fn ctitandb_options_set_min_blob_size(arg1: *mut ctitandb_options_t, size: u64);
}
extern "C" {
    pub fn ctitandb_options_blob_file_compression(arg1: *mut ctitandb_options_t) -> libc::c_int;
}
extern "C" {
    pub fn ctitandb_options_set_gc_merge_rewrite(
        arg1: *mut ctitandb_options_t,
        arg2: libc::c_uchar,
    );
}
extern "C" {
    pub fn ctitandb_options_set_blob_file_compression(
        arg1: *mut ctitandb_options_t,
        type_: rocksdb_CompressionType,
    );
}
extern "C" {
    pub fn ctitandb_options_set_compression_options(
        opt: *mut ctitandb_options_t,
        arg1: libc::c_int,
        arg2: libc::c_int,
        arg3: libc::c_int,
        arg4: libc::c_int,
        arg5: libc::c_int,
    );
}
extern "C" {
    pub fn ctitandb_decode_blob_index(
        value: *const libc::c_char,
        value_size: usize,
        index: *mut ctitandb_blob_index_t,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn ctitandb_encode_blob_index(
        index: *const ctitandb_blob_index_t,
        value: *mut *mut libc::c_char,
        value_size: *mut usize,
    );
}
extern "C" {
    pub fn ctitandb_options_set_disable_background_gc(
        options: *mut ctitandb_options_t,
        disable: libc::c_uchar,
    );
}
extern "C" {
    pub fn ctitandb_options_set_level_merge(
        options: *mut ctitandb_options_t,
        enable: libc::c_uchar,
    );
}
extern "C" {
    pub fn ctitandb_options_set_range_merge(
        options: *mut ctitandb_options_t,
        enable: libc::c_uchar,
    );
}
extern "C" {
    pub fn ctitandb_options_set_max_sorted_runs(
        options: *mut ctitandb_options_t,
        size: libc::c_int,
    );
}
extern "C" {
    pub fn ctitandb_options_set_max_gc_batch_size(options: *mut ctitandb_options_t, size: u64);
}
extern "C" {
    pub fn ctitandb_options_set_min_gc_batch_size(options: *mut ctitandb_options_t, size: u64);
}
extern "C" {
    pub fn ctitandb_options_set_blob_file_discardable_ratio(
        options: *mut ctitandb_options_t,
        ratio: f64,
    );
}
extern "C" {
    pub fn ctitandb_options_set_sample_file_size_ratio(
        options: *mut ctitandb_options_t,
        ratio: f64,
    );
}
extern "C" {
    pub fn ctitandb_options_set_merge_small_file_threshold(
        options: *mut ctitandb_options_t,
        size: u64,
    );
}
extern "C" {
    pub fn ctitandb_options_set_max_background_gc(options: *mut ctitandb_options_t, size: i32);
}
extern "C" {
    pub fn ctitandb_options_set_purge_obsolete_files_period_sec(
        options: *mut ctitandb_options_t,
        period: libc::c_uint,
    );
}
extern "C" {
    pub fn ctitandb_options_set_blob_cache(
        options: *mut ctitandb_options_t,
        cache: *mut crocksdb_cache_t,
    );
}
extern "C" {
    pub fn ctitandb_options_get_blob_cache_usage(opt: *mut ctitandb_options_t) -> usize;
}
extern "C" {
    pub fn ctitandb_options_set_blob_cache_capacity(
        opt: *mut ctitandb_options_t,
        capacity: usize,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn ctitandb_options_get_blob_cache_capacity(opt: *mut ctitandb_options_t) -> usize;
}
extern "C" {
    pub fn ctitandb_options_set_discardable_ratio(options: *mut ctitandb_options_t, ratio: f64);
}
extern "C" {
    pub fn ctitandb_options_set_sample_ratio(options: *mut ctitandb_options_t, ratio: f64);
}
extern "C" {
    pub fn ctitandb_options_set_blob_run_mode(
        options: *mut ctitandb_options_t,
        mode: rocksdb_titandb_TitanBlobRunMode,
    );
}
extern "C" {
    pub fn ctitandb_readoptions_init(arg1: *mut rocksdb_titandb_TitanReadOptions);
}
extern "C" {
    pub fn ctitandb_create_iterator(
        db: *mut crocksdb_t,
        titan_options: *const rocksdb_titandb_TitanReadOptions,
    ) -> *mut crocksdb_iterator_t;
}
extern "C" {
    pub fn ctitandb_create_iterator_cf(
        db: *mut crocksdb_t,
        titan_options: *const rocksdb_titandb_TitanReadOptions,
        column_family: *mut crocksdb_column_family_handle_t,
    ) -> *mut crocksdb_iterator_t;
}
extern "C" {
    pub fn ctitandb_create_iterators(
        db: *mut crocksdb_t,
        titan_options: *const rocksdb_titandb_TitanReadOptions,
        column_families: *mut *mut crocksdb_column_family_handle_t,
        iterators: *mut *mut crocksdb_iterator_t,
        size: usize,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn ctitandb_delete_files_in_range(
        db: *mut crocksdb_t,
        start_key: *const libc::c_char,
        start_key_len: usize,
        limit_key: *const libc::c_char,
        limit_key_len: usize,
        include_end: libc::c_uchar,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn ctitandb_delete_files_in_range_cf(
        db: *mut crocksdb_t,
        column_family: *mut crocksdb_column_family_handle_t,
        start_key: *const libc::c_char,
        start_key_len: usize,
        limit_key: *const libc::c_char,
        limit_key_len: usize,
        include_end: libc::c_uchar,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn ctitandb_delete_files_in_ranges_cf(
        db: *mut crocksdb_t,
        cf: *mut crocksdb_column_family_handle_t,
        start_keys: *const *const libc::c_char,
        start_keys_lens: *const usize,
        limit_keys: *const *const libc::c_char,
        limit_keys_lens: *const usize,
        num_ranges: usize,
        include_end: libc::c_uchar,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn ctitandb_delete_blob_files_in_range(
        db: *mut crocksdb_t,
        start_key: *const libc::c_char,
        start_key_len: usize,
        limit_key: *const libc::c_char,
        limit_key_len: usize,
        include_end: libc::c_uchar,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn ctitandb_delete_blob_files_in_range_cf(
        db: *mut crocksdb_t,
        column_family: *mut crocksdb_column_family_handle_t,
        start_key: *const libc::c_char,
        start_key_len: usize,
        limit_key: *const libc::c_char,
        limit_key_len: usize,
        include_end: libc::c_uchar,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn ctitandb_delete_blob_files_in_ranges_cf(
        db: *mut crocksdb_t,
        cf: *mut crocksdb_column_family_handle_t,
        start_keys: *const *const libc::c_char,
        start_keys_lens: *const usize,
        limit_keys: *const *const libc::c_char,
        limit_keys_lens: *const usize,
        num_ranges: usize,
        include_end: libc::c_uchar,
        s: *mut rocksdb_Status,
    );
}
extern "C" {
    pub fn crocksdb_free_cplus_array(arr: *const libc::c_char);
}
extern "C" {
    pub fn crocksdb_to_cplus_array(s: rocksdb_Slice) -> *const libc::c_char;
}
