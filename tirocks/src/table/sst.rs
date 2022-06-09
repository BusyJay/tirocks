// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::{
    marker::PhantomData,
    path::Path,
    ptr,
    str::{self, Utf8Error},
};

use tirocks_sys::{
    crocksdb_externalsstfileinfo_destroy, r, rocksdb_ExternalSstFileInfo, rocksdb_SstFileReader,
    rocksdb_SstFileWriter, s,
};

use crate::{
    db::RawColumnFamilyHandle,
    env::EnvOptions,
    option::{RawOptions, ReadOptions},
    properties::table::{builtin::TableProperties, user::SequenceNumber},
    snapshot::WithSnapOpt,
    util::{ffi_call, simple_access, PathToSlice},
    Options, RawIterator, RawSnapshot, Result, Status,
};

/// ExternalSstFileInfo include information about sst files created
/// using SstFileWriter.
#[derive(Debug)]
pub struct ExternalSstFileInfo {
    ptr: *mut rocksdb_ExternalSstFileInfo,
}

impl Default for ExternalSstFileInfo {
    #[inline]
    fn default() -> Self {
        let ptr = unsafe { tirocks_sys::crocksdb_externalsstfileinfo_create() };
        Self { ptr }
    }
}

impl Drop for ExternalSstFileInfo {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            crocksdb_externalsstfileinfo_destroy(self.ptr);
        }
    }
}

impl ExternalSstFileInfo {
    #[inline]
    fn as_ptr(&self) -> *const rocksdb_ExternalSstFileInfo {
        self.ptr
    }

    /// external sst file path
    #[inline]
    pub fn file_path(&self) -> std::result::Result<&str, Utf8Error> {
        unsafe {
            let mut buf = r(&[]);
            tirocks_sys::crocksdb_externalsstfileinfo_file_path(self.as_ptr(), &mut buf);
            str::from_utf8(s(buf))
        }
    }

    /// smallest user key in file
    #[inline]
    pub fn smallest_key(&self) -> &[u8] {
        unsafe {
            let mut buf = r(&[]);
            tirocks_sys::crocksdb_externalsstfileinfo_smallest_key(self.as_ptr(), &mut buf);
            s(buf)
        }
    }

    /// largest user key in file
    #[inline]
    pub fn largest_key(&self) -> &[u8] {
        unsafe {
            let mut buf = r(&[]);
            tirocks_sys::crocksdb_externalsstfileinfo_largest_key(self.as_ptr(), &mut buf);
            s(buf)
        }
    }

    simple_access! {
        crocksdb_externalsstfileinfo

        /// sequence number of all keys in file
        (<) sequence_number: SequenceNumber

        /// file size in bytes
        (<) file_size: u64

        /// number of entries in file
        (<) num_entries: u64
    }
}

/// SstFileWriter is used to create sst files that can be added to database later
/// All keys in files generated by SstFileWriter will have sequence number = 0.
#[derive(Debug)]
pub struct SstFileWriter<'a> {
    ptr: *mut rocksdb_SstFileWriter,
    life: PhantomData<&'a ()>,
}

impl<'a> Drop for SstFileWriter<'a> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            tirocks_sys::crocksdb_sstfilewriter_destroy(self.ptr);
        }
    }
}

impl<'a> SstFileWriter<'a> {
    /// Create a writer that writes SST file that will be ingested into specified column family.
    /// `None` means the CF is unknown.
    #[inline]
    pub fn new(
        env_opt: &'a EnvOptions,
        opt: &'a RawOptions,
        cf: Option<&'a RawColumnFamilyHandle>,
    ) -> Self {
        let ptr = unsafe {
            tirocks_sys::crocksdb_sstfilewriter_create_cf(
                env_opt.get_ptr(),
                opt.as_ptr(),
                cf.map_or_else(ptr::null_mut, |c| c.get_ptr()),
            )
        };
        Self {
            ptr,
            life: PhantomData,
        }
    }

    /// Prepare SstFileWriter to write into file located at "file_path".
    #[inline]
    pub fn open(&mut self, path: impl AsRef<Path>) -> Result<()> {
        unsafe { ffi_call!(crocksdb_sstfilewriter_open(self.ptr, path.path_to_slice(),)) }
    }

    /// Add a Put key with value to currently opened file
    /// REQUIRES: key is after any previously added key according to comparator.
    #[inline]
    pub fn put(&mut self, key: &[u8], val: &[u8]) -> Result<()> {
        unsafe { ffi_call!(crocksdb_sstfilewriter_put(self.ptr, r(key), r(val))) }
    }

    /// Add a Merge key with value to currently opened file
    /// REQUIRES: key is after any previously added key according to comparator.
    #[inline]
    pub fn merge(&mut self, key: &[u8], val: &[u8]) -> Result<()> {
        unsafe { ffi_call!(crocksdb_sstfilewriter_merge(self.ptr, r(key), r(val))) }
    }

    /// Add a deletion key to currently opened file
    /// REQUIRES: key is after any previously added key according to comparator.
    #[inline]
    pub fn delete(&mut self, key: &[u8]) -> Result<()> {
        unsafe { ffi_call!(crocksdb_sstfilewriter_delete(self.ptr, r(key))) }
    }

    /// Add a range deletion tombstone to currently opened file
    #[inline]
    pub fn delete_range(&mut self, begin: &[u8], end: &[u8]) -> Result<()> {
        unsafe {
            ffi_call!(crocksdb_sstfilewriter_delete_range(
                self.ptr,
                r(begin),
                r(end),
            ))
        }
    }

    /// Finalize writing to sst file and close file.
    ///
    /// An optional ExternalSstFileInfo pointer can be passed to the function
    /// which will be populated with information about the created sst file.
    #[inline]
    pub fn finish(&mut self, info: Option<&mut ExternalSstFileInfo>) -> Result<()> {
        unsafe {
            let info_ptr = info.map_or_else(ptr::null_mut, |i| i.ptr);
            ffi_call!(crocksdb_sstfilewriter_finish(self.ptr, info_ptr))
        }
    }

    /// Return the current file size.
    #[inline]
    pub fn file_size(&self) -> u64 {
        unsafe { tirocks_sys::crocksdb_sstfilewriter_file_size(self.ptr) }
    }
}

/// SstFileReader is used to read sst files that are generated by DB or
/// SstFileWriter.
pub struct SstFileReader<'a> {
    ptr: *mut rocksdb_SstFileReader,
    life: PhantomData<&'a ()>,
}

impl<'a> Drop for SstFileReader<'a> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            tirocks_sys::crocksdb_sstfilereader_destroy(self.ptr);
        }
    }
}

impl<'a> SstFileReader<'a> {
    #[inline]
    pub fn new(options: &Options) -> Self {
        let ptr = unsafe { tirocks_sys::crocksdb_sstfilereader_create(options.get_ptr()) };
        Self {
            ptr,
            life: PhantomData,
        }
    }

    /// Prepares to read from the file located at "path".
    #[inline]
    pub fn open(&mut self, path: impl AsRef<Path>) -> Result<()> {
        unsafe { ffi_call!(crocksdb_sstfilereader_open(self.ptr, path.path_to_slice(),)) }
    }

    #[inline]
    pub fn table_properties(&self) -> &TableProperties {
        unsafe {
            let raw = tirocks_sys::crocksdb_sstfilereader_get_table_properties(self.ptr);
            TableProperties::from_ptr(raw)
        }
    }

    #[inline]
    pub fn iter<'b>(
        &'b self,
        snap: Option<&'b RawSnapshot>,
        opt: &'b mut ReadOptions,
    ) -> RawIterator<'b> {
        let mut _with_snap_opt = None;
        let opt = match snap {
            None => opt.get_ptr(),
            Some(s) => {
                _with_snap_opt = Some(WithSnapOpt::new(opt, s));
                _with_snap_opt.as_ref().unwrap().get_ptr()
            }
        };
        unsafe {
            let ptr = tirocks_sys::crocksdb_sstfilereader_new_iterator(self.ptr, opt as _);
            RawIterator::from_ptr(ptr)
        }
    }

    /// Verifies whether there is corruption in this table.
    #[inline]
    pub fn verify_checksum(&mut self) -> Result<()> {
        unsafe { ffi_call!(crocksdb_sstfilereader_verify_checksum(self.ptr)) }
    }
}
