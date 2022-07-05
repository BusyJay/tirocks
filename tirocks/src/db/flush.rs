// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::{mem::ManuallyDrop, path::Path, ptr};

use tirocks_sys::{r, rocksdb_Slice};

use crate::{
    option::{CompactRangeOptions, CompactionOptions, FlushOptions, IngestExternalFileOptions},
    util::{ffi_call, PathToSlice},
    RawDb, Result, Status,
};

use super::cf::RawCfHandle;

pub struct IngestExternalFileArg<'a, T: AsRef<Path>> {
    pub cf: &'a RawCfHandle,
    pub file_list: &'a [T],
    pub opt: &'a IngestExternalFileOptions,
}

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
        cf: &RawCfHandle,
        begin: Option<&[u8]>,
        end: Option<&[u8]>,
    ) -> Result<()> {
        unsafe {
            let begin = begin.map(|k| r(k));
            let end = end.map(|k| r(k));
            ffi_call!(crocksdb_compact_range_cf_opt(
                self.get_ptr(),
                opt.as_ptr(),
                cf.get_ptr(),
                begin.as_ref().map_or_else(ptr::null, |k| k),
                end.as_ref().map_or_else(ptr::null, |k| k),
            ))
        }
    }

    /// Inputs a list of files specified by file numbers and compacts them to the specified
    /// level. Note that the behavior is different from [`compact_ranage`] in that
    /// `compact_files` performs the compaction job using the CURRENT thread.
    #[inline]
    pub fn compact_files(
        &self,
        opt: &CompactionOptions,
        cf: &RawCfHandle,
        input_file_names: &[impl AsRef<Path>],
        output_level: i32,
    ) -> Result<()> {
        unsafe {
            let input_file_names: Vec<_> =
                input_file_names.iter().map(|n| n.path_to_slice()).collect();
            ffi_call!(crocksdb_compact_files_cf(
                self.get_ptr(),
                opt.as_ptr(),
                cf.get_ptr(),
                input_file_names.as_ptr(),
                input_file_names.len(),
                output_level,
            ))
        }
    }

    /// This function will wait until all currently running background processes
    /// finish. After it returns, no background process will be run until
    /// [`continue_background_work`] is called
    #[inline]
    pub fn pause_background_work(&self) -> Result<()> {
        unsafe { ffi_call!(crocksdb_pause_bg_work(self.get_ptr())) }
    }

    #[inline]
    pub fn continue_background_work(&self) -> Result<()> {
        unsafe { ffi_call!(crocksdb_continue_bg_work(self.get_ptr())) }
    }

    /// Flush a single column family, even when atomic flush is enabled. To flush
    /// multiple column families, use [`flush_multi`].
    #[inline]
    pub fn flush(&self, option: &FlushOptions, cf: &RawCfHandle) -> Result<()> {
        unsafe {
            ffi_call!(crocksdb_flush_cf(
                self.get_ptr(),
                option.as_ptr(),
                cf.get_ptr(),
            ))
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
    pub fn flush_multi(&self, option: &FlushOptions, cfs: &[&RawCfHandle]) -> Result<()> {
        unsafe {
            let cfs: Vec<_> = cfs.iter().map(|c| c.get_ptr()).collect();
            ffi_call!(crocksdb_flush_cfs(
                self.get_ptr(),
                option.as_ptr(),
                cfs.as_ptr(),
                cfs.len() as i32,
            ))
        }
    }

    /// Flush the WAL memory buffer to the file. If sync is true, it calls [`sync_wal`]
    /// afterwards.
    #[inline]
    pub fn flush_wal(&self, sync: bool) -> Result<()> {
        unsafe { ffi_call!(crocksdb_flush_wal(self.get_ptr(), sync)) }
    }

    /// Sync the wal. Note that [`write`] followed by [`sync_wal`] is not exactly the
    /// same as [`write`] with sync=true: in the latter case the changes won't be
    /// visible until the sync is done.
    /// Currently only works if allow_mmap_writes = false in Options.
    #[inline]
    pub fn sync_wal(&self) -> Result<()> {
        unsafe { ffi_call!(crocksdb_sync_wal(self.get_ptr())) }
    }

    /// Load a list of external SST files (1) into the DB
    /// Two primary modes are supported:
    /// - Duplicate keys in the new files will overwrite exiting keys (default)
    /// - Duplicate keys will be skipped (set ingest_behind=true)
    /// In the first mode we will try to find the lowest possible level that
    /// the file can fit in, and ingest the file into this level (2). A file that
    /// have a key range that overlap with the memtable key range will require us
    /// to Flush the memtable first before ingesting the file.
    /// In the second mode we will always ingest in the bottom most level (see
    /// docs to IngestExternalFileOptions::ingest_behind).
    ///
    /// (1) External SST files can be created using SstFileWriter
    /// (2) We will try to ingest the files to the lowest possible level
    ///     even if the file compression doesn't match the level compression
    /// (3) If IngestExternalFileOptions->ingest_behind is set to true,
    ///     we always ingest at the bottommost level, which should be reserved
    ///     for this purpose (see DBOPtions::allow_ingest_behind flag).
    #[inline]
    pub fn ingest_external_file(
        &self,
        opt: &IngestExternalFileOptions,
        cf: &RawCfHandle,
        files: &[impl AsRef<Path>],
    ) -> Result<()> {
        unsafe {
            let files: Vec<_> = files.iter().map(|p| p.path_to_slice()).collect();
            ffi_call!(crocksdb_ingest_external_file_cf(
                self.get_ptr(),
                cf.get_ptr(),
                files.as_ptr(),
                files.len(),
                opt.as_ptr(),
            ))
        }
    }

    /// Same as [`ingest_external_file`] but avoid blocking writes with best effort.
    #[inline]
    pub fn ingest_external_file_optimized(
        &self,
        opt: &IngestExternalFileOptions,
        cf: &RawCfHandle,
        files: &[impl AsRef<Path>],
    ) -> Result<bool> {
        unsafe {
            let files: Vec<_> = files.iter().map(|p| p.path_to_slice()).collect();
            ffi_call!(crocksdb_ingest_external_file_optimized(
                self.get_ptr(),
                cf.get_ptr(),
                files.as_ptr(),
                files.len(),
                opt.as_ptr(),
            ))
        }
    }

    /// Ingest files for multiple column families, and record the result atomically to the
    /// MANIFEST.
    /// If this function returns OK, all column families' ingestion must succeed.
    /// If this function returns NOK, or the process crashes, then non-of the
    /// files will be ingested into the database after recovery.
    /// Note that it is possible for application to observe a mixed state during
    /// the execution of this function. If the user performs range scan over the
    /// column families with iterators, iterator on one column family may return
    /// ingested data, while iterator on other column family returns old data.
    /// Users can use snapshot for a consistent view of data.
    /// If your db ingests multiple SST files using this API, i.e. args.len()
    /// > 1, then RocksDB 5.15 and earlier will not be able to open it.
    ///
    /// REQUIRES: each arg corresponds to a different column family: namely, for
    /// 0 <= i < j < len(args), args[i].column_family != args[j].column_family.
    #[inline]
    pub fn ingest_external_file_multi(
        &self,
        args: &[IngestExternalFileArg<impl AsRef<Path>>],
    ) -> Result<()> {
        unsafe {
            let mut cfs = Vec::with_capacity(args.len());
            let mut file_list = Vec::with_capacity(args.len());
            let mut file_count_list = Vec::with_capacity(args.len());
            let mut opts = Vec::with_capacity(args.len());
            for arg in args {
                cfs.push(arg.cf.get_ptr());
                let mut list = ManuallyDrop::new(Vec::with_capacity(arg.file_list.len()));
                for f in arg.file_list {
                    list.push(f.path_to_slice());
                }
                file_list.push(list.as_ptr());
                file_count_list.push(list.len());
                opts.push(arg.opt.as_ptr());
            }
            let res = ffi_call!(crocksdb_ingest_external_file_multi_cf(
                self.get_ptr(),
                cfs.as_ptr(),
                file_list.as_ptr(),
                file_count_list.as_ptr(),
                opts.as_ptr(),
                args.len(),
            ));
            for (p, len) in file_list.iter().zip(file_count_list) {
                drop(Vec::from_raw_parts(*p as *mut rocksdb_Slice, len, len));
            }
            res
        }
    }
}
