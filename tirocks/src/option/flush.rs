// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::mem::MaybeUninit;

use tirocks_sys::{
    rocksdb_BottommostLevelCompaction, rocksdb_CompactRangeOptions, rocksdb_CompactionOptions,
    rocksdb_FlushOptions, rocksdb_IngestExternalFileOptions,
};

use super::CompressionType;

/// Options that control flush operations
#[derive(Debug)]
#[repr(transparent)]
pub struct FlushOptions {
    raw: rocksdb_FlushOptions,
}

impl Default for FlushOptions {
    #[inline]
    fn default() -> Self {
        let mut opt = MaybeUninit::uninit();
        unsafe {
            tirocks_sys::crocksdb_flushoptions_init(opt.as_mut_ptr());
            Self {
                raw: opt.assume_init(),
            }
        }
    }
}

impl FlushOptions {
    /// If true, the flush will wait until the flush is done.
    /// Default: true
    #[inline]
    pub fn set_wait(&mut self, wait: bool) -> &mut Self {
        self.raw.wait = wait;
        self
    }

    /// If true, the flush would proceed immediately even it means writes will
    /// stall for the duration of the flush; if false the operation will wait
    /// until it's possible to do flush w/o causing stall or until required flush
    /// is performed by someone else (foreground call or background thread).
    /// Default: false
    #[inline]
    pub fn set_allow_write_stall(&mut self, allow: bool) -> &mut Self {
        self.raw.allow_write_stall = allow;
        self
    }

    #[inline]
    pub(crate) fn as_ptr(&self) -> *const rocksdb_FlushOptions {
        &self.raw
    }
}

// CompactionOptions are used in CompactFiles() call.
#[derive(Debug)]
#[repr(transparent)]
pub struct CompactionOptions {
    raw: rocksdb_CompactionOptions,
}

impl Default for CompactionOptions {
    #[inline]
    fn default() -> Self {
        let mut opt = MaybeUninit::uninit();
        unsafe {
            tirocks_sys::crocksdb_compaction_options_init(opt.as_mut_ptr());
            Self {
                raw: opt.assume_init(),
            }
        }
    }
}

impl CompactionOptions {
    /// Compaction output compression type
    /// Default: snappy
    /// If set to `kDisableCompressionOption`, RocksDB will choose compression type
    /// according to the `ColumnFamilyOptions`, taking into account the output
    /// level if `compression_per_level` is specified.
    #[inline]
    pub fn set_compression(&mut self, compression: CompressionType) -> &mut Self {
        self.raw.compression = compression;
        self
    }

    /// Compaction will create files of size `output_file_size_limit`.
    /// Default: MAX, which means that compaction will create a single file
    #[inline]
    pub fn set_output_file_size_limit(&mut self, size_limit: u64) -> &mut Self {
        self.raw.output_file_size_limit = size_limit;
        self
    }

    /// If > 0, it will replace the option in the DBOptions for this compaction.
    #[inline]
    pub fn set_max_subcompactions(&mut self, subcompaction: u32) -> &mut Self {
        self.raw.max_subcompactions = subcompaction;
        self
    }

    #[inline]
    pub(crate) fn as_ptr(&self) -> *const rocksdb_CompactionOptions {
        &self.raw
    }
}

pub type BottommostLevelCompaction = rocksdb_BottommostLevelCompaction;

// CompactRangeOptions is used by CompactRange() call.
#[derive(Debug)]
#[repr(transparent)]
pub struct CompactRangeOptions {
    raw: rocksdb_CompactRangeOptions,
}

impl Default for CompactRangeOptions {
    #[inline]
    fn default() -> Self {
        let mut opt = MaybeUninit::uninit();
        unsafe {
            tirocks_sys::crocksdb_compactrangeoptions_init(opt.as_mut_ptr());
            Self {
                raw: opt.assume_init(),
            }
        }
    }
}

impl CompactRangeOptions {
    /// If true, no other compaction will run at the same time as this
    /// manual compaction
    #[inline]
    pub fn set_exclusive_manual_compaction(&mut self, exclusive: bool) -> &mut Self {
        self.raw.exclusive_manual_compaction = exclusive;
        self
    }

    /// If true, compacted files will be moved to the minimum level capable
    /// of holding the data or given level (specified non-negative target_level).
    #[inline]
    pub fn set_change_level(&mut self, change_level: bool) -> &mut Self {
        self.raw.change_level = change_level;
        self
    }

    /// If change_level is true and target_level have non-negative value, compacted
    /// files will be moved to target_level.
    #[inline]
    pub fn set_target_level(&mut self, target_level: i32) -> &mut Self {
        self.raw.target_level = target_level;
        self
    }

    /// Compaction outputs will be placed in options.db_paths[target_path_id].
    /// Behavior is undefined if target_path_id is out of range.
    #[inline]
    pub fn set_target_path_id(&mut self, path_id: u32) -> &mut Self {
        self.raw.target_path_id = path_id;
        self
    }

    /// By default level based compaction will only compact the bottommost level
    /// if there is a compaction filter
    #[inline]
    pub fn set_bottommost_level_compaction(
        &mut self,
        compaction: BottommostLevelCompaction,
    ) -> &mut Self {
        self.raw.bottommost_level_compaction = compaction;
        self
    }

    /// If true, will execute immediately even if doing so would cause the DB to
    /// enter write stall mode. Otherwise, it'll sleep until load is low enough.
    #[inline]
    pub fn set_allow_write_stall(&mut self, allow: bool) -> &mut Self {
        self.raw.allow_write_stall = allow;
        self
    }

    /// If > 0, it will replace the option in the DBOptions for this compaction.
    #[inline]
    pub fn set_max_subcompactions(&mut self, subcompaction: u32) -> &mut Self {
        self.raw.max_subcompactions = subcompaction;
        self
    }

    #[inline]
    pub(crate) fn as_ptr(&self) -> *const rocksdb_CompactRangeOptions {
        &self.raw
    }
}

// IngestExternalFileOptions is used by IngestExternalFile()
#[derive(Debug)]
#[repr(transparent)]
pub struct IngestExternalFileOptions {
    raw: rocksdb_IngestExternalFileOptions,
}

impl Default for IngestExternalFileOptions {
    #[inline]
    fn default() -> Self {
        let mut opt = MaybeUninit::uninit();
        unsafe {
            tirocks_sys::crocksdb_ingestexternalfileoptions_init(opt.as_mut_ptr());
            Self {
                raw: opt.assume_init(),
            }
        }
    }
}

impl IngestExternalFileOptions {
    /// Can be set to true to move the files instead of copying them.
    #[inline]
    pub fn set_move_files(&mut self, m: bool) -> &mut Self {
        self.raw.move_files = m;
        self
    }

    /// If set to true, ingestion falls back to copy when move fails.
    #[inline]
    pub fn set_failed_move_fall_back_to_copy(&mut self, fallback: bool) -> &mut Self {
        self.raw.failed_move_fall_back_to_copy = fallback;
        self
    }

    /// If set to false, an ingested file keys could appear in existing snapshots
    /// that where created before the file was ingested.
    #[inline]
    pub fn set_snapshot_consistency(&mut self, consistency: bool) -> &mut Self {
        self.raw.snapshot_consistency = consistency;
        self
    }

    /// If set to false, IngestExternalFile() will fail if the file key range
    /// overlaps with existing keys or tombstones in the DB.
    #[inline]
    pub fn set_allow_global_sequence_number(&mut self, allow: bool) -> &mut Self {
        self.raw.allow_global_seqno = allow;
        self
    }

    // If set to false and the file key range overlaps with the memtable key range
    // (memtable flush required), IngestExternalFile will fail.
    #[inline]
    pub fn set_allow_blocking_flush(&mut self, allow: bool) -> &mut Self {
        self.raw.allow_blocking_flush = allow;
        self
    }

    /// Set to true if you would like duplicate keys in the file being ingested
    /// to be skipped rather than overwriting existing data under that key.
    /// Usecase: back-fill of some historical data in the database without
    /// over-writing existing newer version of data.
    /// This option could only be used if the DB has been running
    /// with allow_ingest_behind=true since the dawn of time.
    /// All files will be ingested at the bottommost level with seqno=0.
    #[inline]
    pub fn set_ingest_behind(&mut self, behind: bool) -> &mut Self {
        self.raw.ingest_behind = behind;
        self
    }

    /// Set to true if you would like to write global_seqno to a given offset in
    /// the external SST file for backward compatibility. Older versions of
    /// RocksDB writes a global_seqno to a given offset within ingested SST files,
    /// and new versions of RocksDB do not. If you ingest an external SST using
    /// new version of RocksDB and would like to be able to downgrade to an
    /// older version of RocksDB, you should set 'write_global_seqno' to true. If
    /// your service is just starting to use the new RocksDB, we recommend that
    /// you set this option to false, which brings two benefits:
    /// 1. No extra random write for global_seqno during ingestion.
    /// 2. Without writing external SST file, it's possible to do checksum.
    /// We have a plan to set this option to false by default in the future.
    #[inline]
    pub fn set_write_global_sequence_number(&mut self, write: bool) -> &mut Self {
        self.raw.write_global_seqno = write;
        self
    }

    /// Set to true if you would like to verify the checksums of each block of the
    /// external SST file before ingestion.
    /// Warning: setting this to true causes slowdown in file ingestion because
    /// the external SST file has to be read.
    #[inline]
    pub fn set_verify_checksums_before_ingest(&mut self, verify: bool) -> &mut Self {
        self.raw.verify_checksums_before_ingest = verify;
        self
    }

    #[inline]
    pub(crate) fn as_ptr(&self) -> *const rocksdb_IngestExternalFileOptions {
        &self.raw
    }
}
