// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::str::{self, Utf8Error};

use tirocks_sys::{
    r, rocksdb_ExternalFileIngestionInfo, rocksdb_WriteStallCondition, rocksdb_WriteStallInfo, s,
};

use crate::properties::table::builtin::TableProperties;

pub type WriteStallCondition = rocksdb_WriteStallCondition;

#[repr(transparent)]
pub struct ExternalFileIngestionInfo(rocksdb_ExternalFileIngestionInfo);

impl ExternalFileIngestionInfo {
    #[inline]
    pub(crate) unsafe fn from_ptr<'a>(
        ptr: *const rocksdb_ExternalFileIngestionInfo,
    ) -> &'a ExternalFileIngestionInfo {
        &*(ptr as *const ExternalFileIngestionInfo)
    }

    #[inline]
    fn as_ptr(&self) -> *const rocksdb_ExternalFileIngestionInfo {
        self as *const ExternalFileIngestionInfo as _
    }

    /// The name of the column family.
    #[inline]
    pub fn cf_name(&self) -> std::result::Result<&str, Utf8Error> {
        unsafe {
            let mut buf = r(&[]);
            tirocks_sys::crocksdb_externalfileingestioninfo_cf_name(self.as_ptr(), &mut buf);
            str::from_utf8(s(buf))
        }
    }

    /// Path of the file inside the DB
    #[inline]
    pub fn internal_file_path(&self) -> std::result::Result<&str, Utf8Error> {
        unsafe {
            let mut buf = r(&[]);
            tirocks_sys::crocksdb_externalfileingestioninfo_internal_file_path(
                self.as_ptr(),
                &mut buf,
            );
            str::from_utf8(s(buf))
        }
    }

    /// Table properties of the table being flushed
    #[inline]
    pub fn table_properties(&self) -> &TableProperties {
        unsafe {
            let prop =
                tirocks_sys::crocksdb_externalfileingestioninfo_table_properties(self.as_ptr());
            TableProperties::from_ptr(prop)
        }
    }

    /// Level inside the DB we picked for the external file.
    pub fn picked_level(&self) -> i32 {
        unsafe { tirocks_sys::crocksdb_externalfileingestioninfo_picked_level(self.as_ptr()) }
    }
}

#[repr(transparent)]
pub struct WriteStallInfo(rocksdb_WriteStallInfo);

impl WriteStallInfo {
    #[inline]
    pub(crate) unsafe fn from_ptr<'a>(ptr: *const rocksdb_WriteStallInfo) -> &'a WriteStallInfo {
        &*(ptr as *const WriteStallInfo)
    }

    #[inline]
    fn as_ptr(&self) -> *const rocksdb_WriteStallInfo {
        self as *const WriteStallInfo as _
    }

    /// The name of the column family.
    #[inline]
    pub fn cf_name(&self) -> std::result::Result<&str, Utf8Error> {
        unsafe {
            let mut buf = r(&[]);
            tirocks_sys::crocksdb_writestallinfo_cf_name(self.as_ptr(), &mut buf);
            str::from_utf8(s(buf))
        }
    }

    /// Current state of the write controller
    #[inline]
    pub fn cur_state(&self) -> WriteStallCondition {
        unsafe { tirocks_sys::crocksdb_writestallinfo_cur(self.as_ptr()) }
    }

    /// Previous state of the write controller
    #[inline]
    pub fn prev_state(&self) -> WriteStallCondition {
        unsafe { tirocks_sys::crocksdb_writestallinfo_prev(self.as_ptr()) }
    }
}
