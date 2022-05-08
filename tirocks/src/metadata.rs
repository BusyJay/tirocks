// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::{
    ops::Index,
    str::{self, Utf8Error},
};

use tirocks_sys::{
    crocksdb_livefiles_t, r, rocksdb_ColumnFamilyMetaData, rocksdb_LevelMetaData,
    rocksdb_LiveFileMetaData, rocksdb_SstFileMetaData, s,
};

use crate::RawDb;

pub type SizeApproximationOptions = tirocks_sys::rocksdb_SizeApproximationOptions;

pub struct LiveFileMetaData(rocksdb_LiveFileMetaData);

impl LiveFileMetaData {
    #[inline]
    fn as_ptr(&self) -> *const rocksdb_LiveFileMetaData {
        self as *const _ as _
    }

    #[inline]
    pub fn name(&self) -> std::result::Result<&str, Utf8Error> {
        unsafe {
            let mut buf = r(&[]);
            tirocks_sys::crocksdb_livefiles_name(self.as_ptr(), &mut buf);
            str::from_utf8(s(buf))
        }
    }

    #[inline]
    pub fn level(&self) -> i32 {
        unsafe { tirocks_sys::crocksdb_livefiles_level(self.as_ptr()) }
    }

    #[inline]
    pub fn size(&self) -> usize {
        unsafe { tirocks_sys::crocksdb_livefiles_size(self.as_ptr()) }
    }

    #[inline]
    pub fn smallest_key(&self) -> &[u8] {
        unsafe {
            let mut buf = r(&[]);
            tirocks_sys::crocksdb_livefiles_smallestkey(self.as_ptr(), &mut buf);
            s(buf)
        }
    }

    #[inline]
    pub fn largest_key(&self) -> &[u8] {
        unsafe {
            let mut buf = r(&[]);
            tirocks_sys::crocksdb_livefiles_largestkey(self.as_ptr(), &mut buf);
            s(buf)
        }
    }
}

pub struct LiveFileMetaDataCollection {
    ptr: *mut crocksdb_livefiles_t,
}

impl LiveFileMetaDataCollection {
    #[inline]
    pub fn len(&self) -> usize {
        unsafe { tirocks_sys::crocksdb_livefiles_count(self.ptr) }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn get(&self, index: usize) -> &LiveFileMetaData {
        unsafe {
            let ptr = tirocks_sys::crocksdb_livefiles_get(self.ptr, index);
            &*(ptr as *const LiveFileMetaData)
        }
    }
}

impl Drop for LiveFileMetaDataCollection {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            tirocks_sys::crocksdb_livefiles_destroy(self.ptr);
        }
    }
}

impl Index<usize> for LiveFileMetaDataCollection {
    type Output = LiveFileMetaData;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index)
    }
}

impl RawDb {
    /// Returns a list of all table files with their level, start key and end key
    #[inline]
    pub fn live_files_metadata(&self) -> LiveFileMetaDataCollection {
        let ptr = unsafe { tirocks_sys::crocksdb_livefiles(self.as_ptr()) };
        LiveFileMetaDataCollection { ptr }
    }
}

#[repr(transparent)]
pub struct SstFileMetaData(rocksdb_SstFileMetaData);

impl SstFileMetaData {
    #[inline]
    fn as_ptr(&self) -> *const rocksdb_SstFileMetaData {
        self as *const _ as _
    }

    /// File size in bytes.
    #[inline]
    pub fn size(&self) -> usize {
        unsafe { tirocks_sys::crocksdb_sst_file_meta_data_size(self.as_ptr()) }
    }

    /// The name of the file.
    #[inline]
    pub fn name(&self) -> std::result::Result<&str, Utf8Error> {
        unsafe {
            let mut buf = r(&[]);
            tirocks_sys::crocksdb_sst_file_meta_data_name(self.as_ptr(), &mut buf);
            str::from_utf8(s(buf))
        }
    }

    /// Smallest user defined key in the file.
    #[inline]
    pub fn smallest_key(&self) -> &[u8] {
        unsafe {
            let mut buf = r(&[]);
            tirocks_sys::crocksdb_sst_file_meta_data_smallestkey(self.as_ptr(), &mut buf);
            s(buf)
        }
    }

    /// Largest user defined key in the file.
    #[inline]
    pub fn largest_key(&self) -> &[u8] {
        unsafe {
            let mut buf = r(&[]);
            tirocks_sys::crocksdb_sst_file_meta_data_largestkey(self.as_ptr(), &mut buf);
            s(buf)
        }
    }
}

#[repr(transparent)]
pub struct LevelMetaData(rocksdb_LevelMetaData);

impl LevelMetaData {
    #[inline]
    fn as_ptr(&self) -> *const rocksdb_LevelMetaData {
        self as *const _ as _
    }

    /// The number of all sst files in this level.
    #[inline]
    pub fn file_count(&self) -> usize {
        unsafe { tirocks_sys::crocksdb_level_meta_data_file_count(self.as_ptr()) }
    }

    /// Get metadata of the specified sst file.
    #[inline]
    pub fn file(&self, i: usize) -> &SstFileMetaData {
        unsafe {
            let ptr = tirocks_sys::crocksdb_level_meta_data_file_data(self.as_ptr(), i);
            &*(ptr as *const SstFileMetaData)
        }
    }
}

#[repr(transparent)]
pub struct ColumnFamilyMetaData {
    ptr: *mut rocksdb_ColumnFamilyMetaData,
}

impl Default for ColumnFamilyMetaData {
    #[inline]
    fn default() -> Self {
        unsafe {
            let ptr = tirocks_sys::crocksdb_column_family_meta_data_create();
            Self { ptr }
        }
    }
}

impl Drop for ColumnFamilyMetaData {
    #[inline]
    fn drop(&mut self) {
        unsafe { tirocks_sys::crocksdb_column_family_meta_data_destroy(self.ptr) }
    }
}

impl ColumnFamilyMetaData {
    /// The number of level.
    #[inline]
    pub fn level_count(&self) -> usize {
        unsafe { tirocks_sys::crocksdb_column_family_meta_data_level_count(self.ptr) }
    }

    /// Get metadata of the specified sst file.
    #[inline]
    pub fn level(&self, i: usize) -> &LevelMetaData {
        unsafe {
            let ptr = tirocks_sys::crocksdb_column_family_meta_data_level_data(self.ptr, i);
            &*(ptr as *const LevelMetaData)
        }
    }

    #[inline]
    pub(crate) fn as_mut_ptr(&mut self) -> *mut rocksdb_ColumnFamilyMetaData {
        self.ptr
    }
}
