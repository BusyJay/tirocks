// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::{path::Path, ptr};

use tirocks_sys::r;

use crate::{
    option::{CompactRangeOptions, CompactionOptions, FlushOptions, PathToSlice},
    util::check_status,
    RawDb, Result, Status,
};

use super::cf::RawColumnFamilyHandle;

impl RawDb {
    /// Compact the underlying storage for the key range [*begin,*end].
    /// The actual compaction interval might be superset of [*begin, *end].
    /// In particular, deleted and overwritten versions are discarded,
    /// and the data is rearranged to reduce the cost of operations
    /// needed to access the data.  This operation should typically only
    /// be invoked by users who understand the underlying implementation.
    ///
    /// begin==None is treated as a key before all keys in the database.
    /// end==None is treated as a key after all keys in the database.
    /// Therefore the following call will compact the entire database:
    ///    `CompactRange(options, None, None)`.
    /// Note that after the entire database is compacted, all data are pushed
    /// down to the last level containing any data. If the total data size after
    /// compaction is reduced, that level might not be appropriate for hosting all
    /// the files. In this case, client could set options.change_level to true, to
    /// move the files back to the minimum level capable of holding the data set
    /// or a given level (specified by non-negative options.target_level).
    #[inline]
    pub fn compact_range(
        &self,
        opt: &CompactRangeOptions,
        cf: &RawColumnFamilyHandle,
        begin: Option<&[u8]>,
        end: Option<&[u8]>,
    ) -> Result<()> {
        unsafe {
            let begin = begin.map(|k| r(k));
            let end = end.map(|k| r(k));
            let mut s = Status::default();
            tirocks_sys::crocksdb_compact_range_cf_opt(
                self.as_ptr(),
                opt.as_ptr(),
                cf.as_mut_ptr(),
                begin.as_ref().map_or_else(ptr::null, |k| k),
                end.as_ref().map_or_else(ptr::null, |k| k),
                s.as_mut_ptr(),
            );
            check_status!(s)
        }
    }

    /// Inputs a list of files specified by file numbers and compacts them to the specified
    /// level. Note that the behavior is different from [`compact_ranage`] in that
    /// `compact_files` performs the compaction job using the CURRENT thread.
    #[inline]
    pub fn compact_files(
        &self,
        opt: &CompactionOptions,
        cf: &RawColumnFamilyHandle,
        input_file_names: &[impl AsRef<Path>],
        output_level: i32,
    ) -> Result<()> {
        unsafe {
            let input_file_names: Vec<_> = input_file_names
                .into_iter()
                .map(|n| n.path_to_slice())
                .collect();
            let mut s = Status::default();
            tirocks_sys::crocksdb_compact_files_cf(
                self.as_ptr(),
                opt.as_ptr(),
                cf.as_mut_ptr(),
                input_file_names.as_ptr(),
                input_file_names.len(),
                output_level,
                s.as_mut_ptr(),
            );
            check_status!(s)
        }
    }

    /// This function will wait until all currently running background processes
    /// finish. After it returns, no background process will be run until
    /// [`continue_background_work`] is called
    #[inline]
    pub fn pause_background_work(&self) -> Result<()> {
        unsafe {
            let mut s = Status::default();
            tirocks_sys::crocksdb_pause_bg_work(self.as_ptr(), s.as_mut_ptr());
            check_status!(s)
        }
    }

    #[inline]
    pub fn continue_background_work(&self) -> Result<()> {
        unsafe {
            let mut s = Status::default();
            tirocks_sys::crocksdb_continue_bg_work(self.as_ptr(), s.as_mut_ptr());
            check_status!(s)
        }
    }

    /// Flush a single column family, even when atomic flush is enabled. To flush
    /// multiple column families, use [`flush_multi`].
    #[inline]
    pub fn flush(&self, option: &FlushOptions, cf: &RawColumnFamilyHandle) -> Result<()> {
        unsafe {
            let mut s = Status::default();
            tirocks_sys::crocksdb_flush_cf(
                self.as_ptr(),
                option.as_ptr(),
                cf.as_mut_ptr(),
                s.as_mut_ptr(),
            );
            check_status!(s)
        }
    }

    /// Flushes multiple column families.
    ///
    /// If atomic flush is not enabled, `flush_multi` is equivalent to calling [`flush`] multiple
    /// times. If atomic flush is enabled, `flush_multi` will flush all column families specified
    /// in 'cfs' up to the latest sequence number at the time when flush is requested.
    /// Note that RocksDB 5.15 and earlier may not be able to open later versions with atomic
    /// flush enabled.
    #[inline]
    pub fn flush_multi(&self, option: &FlushOptions, cfs: &[&RawColumnFamilyHandle]) -> Result<()> {
        unsafe {
            let cfs: Vec<_> = cfs.into_iter().map(|c| c.as_mut_ptr()).collect();
            let mut s = Status::default();
            tirocks_sys::crocksdb_flush_cfs(
                self.as_ptr(),
                option.as_ptr(),
                cfs.as_ptr(),
                cfs.len() as i32,
                s.as_mut_ptr(),
            );
            check_status!(s)
        }
    }

    /// Flush the WAL memory buffer to the file. If sync is true, it calls [`sync_wal`]
    /// afterwards.
    #[inline]
    pub fn flush_wal(&self, sync: bool) -> Result<()> {
        unsafe {
            let mut s = Status::default();
            tirocks_sys::crocksdb_flush_wal(self.as_ptr(), sync, s.as_mut_ptr());
            check_status!(s)
        }
    }

    /// Sync the wal. Note that [`write`] followed by [`sync_wal`] is not exactly the
    /// same as [`write`] with sync=true: in the latter case the changes won't be
    /// visible until the sync is done.
    /// Currently only works if allow_mmap_writes = false in Options.
    #[inline]
    pub fn sync_wal(&self) -> Result<()> {
        unsafe {
            let mut s = Status::default();
            tirocks_sys::crocksdb_sync_wal(self.as_ptr(), s.as_mut_ptr());
            check_status!(s)
        }
    }
}
