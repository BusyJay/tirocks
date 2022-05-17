// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::{
    fmt::{self, Debug, Formatter},
    str::{self, Utf8Error},
    time::Duration,
};

use tirocks_sys::{
    r, rocksdb_CompactionJobInfo, rocksdb_FlushJobInfo, rocksdb_SubcompactionJobInfo, s,
};

use crate::{
    table_properties::builtin::{TableProperties, TablePropertiesCollection},
    Status,
};

pub type CompactionReason = tirocks_sys::rocksdb_CompactionReason;

#[repr(transparent)]
pub struct FlushJobInfo(tirocks_sys::rocksdb_FlushJobInfo);

impl FlushJobInfo {
    #[inline]
    pub(crate) unsafe fn from_ptr<'a>(ptr: *const rocksdb_FlushJobInfo) -> &'a FlushJobInfo {
        &*(ptr as *const FlushJobInfo)
    }

    #[inline]
    fn as_ptr(&self) -> *const rocksdb_FlushJobInfo {
        self as *const FlushJobInfo as _
    }

    /// the job id, which is unique in the same thread.
    #[inline]
    pub fn id(&self) -> i32 {
        unsafe { tirocks_sys::crocksdb_flushjobinfo_job_id(self.as_ptr()) }
    }

    /// the name of the column family
    #[inline]
    pub fn cf_name(&self) -> std::result::Result<&str, Utf8Error> {
        unsafe {
            let mut buf = r(&[]);
            tirocks_sys::crocksdb_flushjobinfo_cf_name(self.as_ptr(), &mut buf);
            str::from_utf8(s(buf))
        }
    }

    /// the path to the newly created file
    #[inline]
    pub fn file_path(&self) -> std::result::Result<&str, Utf8Error> {
        unsafe {
            let mut buf = r(&[]);
            tirocks_sys::crocksdb_flushjobinfo_file_path(self.as_ptr(), &mut buf);
            str::from_utf8(s(buf))
        }
    }

    /// If true, then rocksdb is currently slowing-down all writes to prevent
    /// creating too many Level 0 files as compaction seems not able to
    /// catch up the write request speed.  This indicates that there are
    /// too many files in Level 0.
    #[inline]
    pub fn triggered_writes_slowdown(&self) -> bool {
        unsafe { tirocks_sys::crocksdb_flushjobinfo_triggered_writes_slowdown(self.as_ptr()) }
    }

    /// If true, then rocksdb is currently blocking any writes to prevent
    /// creating more L0 files.  This indicates that there are too many
    /// files in level 0.  Compactions should try to compact L0 files down
    /// to lower levels as soon as possible.
    #[inline]
    pub fn triggered_writes_stop(&self) -> bool {
        unsafe { tirocks_sys::crocksdb_flushjobinfo_triggered_writes_stop(self.as_ptr()) }
    }

    /// Table properties of the table being flushed
    #[inline]
    pub fn table_properties(&self) -> &TableProperties {
        unsafe {
            let p = tirocks_sys::crocksdb_flushjobinfo_table_properties(self.as_ptr());
            TableProperties::from_ptr(p)
        }
    }
}

impl Debug for FlushJobInfo {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "FlushJobInfo [{}]", self.id())
    }
}

#[repr(transparent)]
pub struct CompactionJobInfo(rocksdb_CompactionJobInfo);

impl CompactionJobInfo {
    #[inline]
    pub(crate) unsafe fn from_ptr<'a>(
        ptr: *const rocksdb_CompactionJobInfo,
    ) -> &'a CompactionJobInfo {
        &*(ptr as *const CompactionJobInfo)
    }

    #[inline]
    fn as_ptr(&self) -> *const rocksdb_CompactionJobInfo {
        self as *const CompactionJobInfo as _
    }

    /// The status indicating whether the compaction was successful or not.
    #[inline]
    pub fn status(&self) -> Status {
        let mut s = Status::default();
        unsafe { tirocks_sys::crocksdb_compactionjobinfo_status(self.as_ptr(), s.as_mut_ptr()) }
        s
    }

    /// The name of the column family where the compaction happened.
    #[inline]
    pub fn cf_name(&self) -> std::result::Result<&str, Utf8Error> {
        unsafe {
            let mut buf = r(&[]);
            tirocks_sys::crocksdb_compactionjobinfo_cf_name(self.as_ptr(), &mut buf);
            str::from_utf8(s(buf))
        }
    }

    // The count of the compaction input files.
    #[inline]
    pub fn input_file_count(&self) -> usize {
        unsafe { tirocks_sys::crocksdb_compactionjobinfo_input_files_count(self.as_ptr()) }
    }

    /// The name of the compaction input files at given offset.
    #[inline]
    pub fn input_file_at(&self, off: usize) -> std::result::Result<&str, Utf8Error> {
        unsafe {
            let mut buf = r(&[]);
            tirocks_sys::crocksdb_compactionjobinfo_input_file_at(self.as_ptr(), off, &mut buf);
            str::from_utf8(s(buf))
        }
    }

    /// The count of the compaction output files.
    #[inline]
    pub fn output_file_count(&self) -> usize {
        unsafe { tirocks_sys::crocksdb_compactionjobinfo_output_files_count(self.as_ptr()) }
    }

    /// The name of the compaction output files at given offset.
    #[inline]
    pub fn output_file_at(&self, off: usize) -> std::result::Result<&str, Utf8Error> {
        unsafe {
            let mut buf = r(&[]);
            tirocks_sys::crocksdb_compactionjobinfo_output_file_at(self.as_ptr(), off, &mut buf);
            str::from_utf8(s(buf))
        }
    }

    /// Table properties for input and output tables.
    /// The map is keyed by values from input_files and output_files.
    #[inline]
    pub fn table_properties(&self) -> TablePropertiesCollection {
        unsafe {
            let prop = tirocks_sys::crocksdb_compactionjobinfo_table_properties(self.as_ptr());
            TablePropertiesCollection::from_borrowed(prop)
        }
    }

    /// The elapsed time of this compaction.
    #[inline]
    pub fn elapsed(&self) -> Duration {
        unsafe {
            let micros = tirocks_sys::crocksdb_compactionjobinfo_elapsed_micros(self.as_ptr());
            Duration::from_micros(micros)
        }
    }

    /// Number of corrupt keys (ParseInternalKey returned false when applied to
    /// the key) encountered and written out.
    #[inline]
    pub fn num_corrupt_keys(&self) -> u64 {
        unsafe { tirocks_sys::crocksdb_compactionjobinfo_num_corrupt_keys(self.as_ptr()) }
    }

    /// The smallest input level of the compaction.
    #[inline]
    pub fn base_input_level(&self) -> i32 {
        unsafe { tirocks_sys::crocksdb_compactionjobinfo_base_input_level(self.as_ptr()) }
    }

    /// The output level of the compaction.
    #[inline]
    pub fn output_level(&self) -> i32 {
        unsafe { tirocks_sys::crocksdb_compactionjobinfo_output_level(self.as_ptr()) }
    }

    /// The number of compaction input records.
    #[inline]
    pub fn input_records(&self) -> u64 {
        unsafe { tirocks_sys::crocksdb_compactionjobinfo_input_records(self.as_ptr()) }
    }

    /// The number of compaction output records.
    #[inline]
    pub fn output_records(&self) -> u64 {
        unsafe { tirocks_sys::crocksdb_compactionjobinfo_output_records(self.as_ptr()) }
    }

    /// The size of the compaction input in bytes.
    #[inline]
    pub fn total_input_bytes(&self) -> u64 {
        unsafe { tirocks_sys::crocksdb_compactionjobinfo_total_input_bytes(self.as_ptr()) }
    }

    /// The size of the compaction output in bytes.
    #[inline]
    pub fn total_output_bytes(&self) -> u64 {
        unsafe { tirocks_sys::crocksdb_compactionjobinfo_total_output_bytes(self.as_ptr()) }
    }

    /// The number of compaction input files at the output level.
    #[inline]
    pub fn num_input_files_at_output_level(&self) -> usize {
        unsafe {
            tirocks_sys::crocksdb_compactionjobinfo_num_input_files_at_output_level(self.as_ptr())
        }
    }

    /// Reason to run the compaction
    #[inline]
    pub fn compaction_reason(&self) -> CompactionReason {
        unsafe { tirocks_sys::crocksdb_compactionjobinfo_compaction_reason(self.as_ptr()) }
    }
}

#[repr(transparent)]
pub struct SubcompactionJobInfo(rocksdb_SubcompactionJobInfo);

impl SubcompactionJobInfo {
    #[inline]
    pub(crate) unsafe fn from_ptr<'a>(
        ptr: *const rocksdb_SubcompactionJobInfo,
    ) -> &'a SubcompactionJobInfo {
        &*(ptr as *const SubcompactionJobInfo)
    }

    #[inline]
    fn as_ptr(&self) -> *const rocksdb_SubcompactionJobInfo {
        self as *const SubcompactionJobInfo as _
    }

    /// The status indicating whether the compaction was successful or not.
    #[inline]
    pub fn status(&self) -> Status {
        let mut s = Status::default();
        unsafe { tirocks_sys::crocksdb_subcompactionjobinfo_status(self.as_ptr(), s.as_mut_ptr()) }
        s
    }

    /// The name of the column family where the compaction happened.
    #[inline]
    pub fn cf_name(&self) -> std::result::Result<&str, Utf8Error> {
        unsafe {
            let mut buf = r(&[]);
            tirocks_sys::crocksdb_subcompactionjobinfo_cf_name(self.as_ptr(), &mut buf);
            str::from_utf8(s(buf))
        }
    }

    /// The id of the thread that completed this compaction job.
    #[inline]
    pub fn thread_id(&self) -> u64 {
        unsafe { tirocks_sys::crocksdb_subcompactionjobinfo_thread_id(self.as_ptr()) }
    }

    /// The smallest input level of the compaction.
    #[inline]
    pub fn base_input_level(&self) -> i32 {
        unsafe { tirocks_sys::crocksdb_subcompactionjobinfo_base_input_level(self.as_ptr()) }
    }

    /// The output level of the compaction.
    #[inline]
    pub fn output_level(&self) -> i32 {
        unsafe { tirocks_sys::crocksdb_subcompactionjobinfo_output_level(self.as_ptr()) }
    }
}
