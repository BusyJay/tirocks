// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::fmt::{self, Debug, Formatter};
use std::mem::MaybeUninit;

use tirocks_sys::rocksdb_WriteOptions;

use super::OwnedSlice;

/// Options that control write operations
pub struct WriteOptions {
    raw: rocksdb_WriteOptions,
    // Storage for iterate_lower_bound, iterate_upper_bound and timestamp.
    slice_store: Option<Box<OwnedSlice>>,
}

impl Default for WriteOptions {
    #[inline]
    fn default() -> Self {
        let mut opt = MaybeUninit::uninit();
        unsafe {
            tirocks_sys::crocksdb_writeoptions_init(opt.as_mut_ptr());
            Self {
                raw: opt.assume_init(),
                slice_store: None,
            }
        }
    }
}

impl Debug for WriteOptions {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.raw)
    }
}

impl WriteOptions {
    /// If true, the write will be flushed from the operating system
    /// buffer cache (by calling WritableFile::Sync()) before the write
    /// is considered complete.  If this flag is true, writes will be
    /// slower.
    ///
    /// If this flag is false, and the machine crashes, some recent
    /// writes may be lost.  Note that if it is just the process that
    /// crashes (i.e., the machine does not reboot), no writes will be
    /// lost even if sync==false.
    ///
    /// In other words, a DB write with sync==false has similar
    /// crash semantics as the "write()" system call.  A DB write
    /// with sync==true has similar crash semantics to a "write()"
    /// system call followed by "fdatasync()".
    ///
    /// Default: false
    #[inline]
    pub fn set_sync(&mut self, sync: bool) -> &mut Self {
        self.raw.sync = sync;
        self
    }

    /// If true, writes will not first go to the write ahead log,
    /// and the write may get lost after a crash. The backup engine
    /// relies on write-ahead logs to back up the memtable, so if
    /// you disable write-ahead logs, you must create backups with
    /// flush_before_backup=true to avoid losing unflushed memtable data.
    /// Default: false
    #[inline]
    pub fn set_disable_wal(&mut self, disable_wal: bool) -> &mut Self {
        self.raw.disableWAL = disable_wal;
        self
    }

    /// If true and if user is trying to write to column families that don't exist
    /// (they were dropped),  ignore the write (don't return an error). If there
    /// are multiple writes in a WriteBatch, other writes will succeed.
    /// Default: false
    #[inline]
    pub fn set_ignore_missing_column_families(&mut self, ignore: bool) -> &mut Self {
        self.raw.ignore_missing_column_families = ignore;
        self
    }

    /// If true and we need to wait or sleep for the write request, fails
    /// immediately with Status::Incomplete().
    /// Default: false
    #[inline]
    pub fn set_no_slowdown(&mut self, no_slowdown: bool) -> &mut Self {
        self.raw.no_slowdown = no_slowdown;
        self
    }

    /// If true, this write request is of lower priority if compaction is
    /// behind. In this case, no_slowdown = true, the request will be cancelled
    /// immediately with Status::Incomplete() returned. Otherwise, it will be
    /// slowed down. The slowdown value is determined by RocksDB to guarantee
    /// it introduces minimum impacts to high priority writes.
    ///
    /// Default: false
    #[inline]
    pub fn set_low_priority(&mut self, low: bool) -> &mut Self {
        self.raw.low_pri = low;
        self
    }

    /// If true, this writebatch will maintain the last insert positions of each
    /// memtable as hints in concurrent write. It can improve write performance
    /// in concurrent writes if keys in one writebatch are sequential. In
    /// non-concurrent writes (when concurrent_memtable_writes is false) this
    /// option will be ignored.
    ///
    /// Default: false
    #[inline]
    pub fn set_memtable_insert_hint_per_batch(&mut self, hint_per_batch: bool) -> &mut Self {
        self.raw.memtable_insert_hint_per_batch = hint_per_batch;
        self
    }

    #[inline]
    pub(crate) fn get_ptr(&self) -> *const rocksdb_WriteOptions {
        &self.raw
    }
}
