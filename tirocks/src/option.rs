// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

mod cf;
mod db;
mod flush;
mod read;
mod write;

use std::{
    ops::{Deref, DerefMut},
    os::unix::prelude::OsStrExt,
    path::Path,
    ptr,
    sync::Arc,
};

pub use cf::{
    CfOptions, CompactionPriority, CompactionStyle, FifoCompactionOptions, TitanBlobRunMode,
    TitanCfOptions, UniversalCompactionOptions,
};
pub use db::{DbOptions, TitanDbOptions};
pub use flush::{
    BottommostLevelCompaction, CompactRangeOptions, CompactionOptions, FlushOptions,
    IngestExternalFileOptions,
};
pub use read::{ReadOptions, ReadTier};
use tirocks_sys::{
    r, rocksdb_CompressionType, rocksdb_Options, rocksdb_Slice, rocksdb_titandb_TitanOptions,
};
pub use write::WriteOptions;

use crate::{comparator::SysComparator, env::Env};

use self::{
    cf::{RawCfOptions, RawTitanCfOptions},
    db::{RawDbOptions, RawTitanDbOptions},
};

pub type CompressionType = rocksdb_CompressionType;

/// An owned slice that can be used with the weird rocksdb Options
/// API, which requires a pointer to slice to outlive the options.
///
/// The safety requires the slice is pinned. Not using pin here because
/// we may use an array to manage multiple slices.
struct OwnedSlice {
    data: Vec<u8>,
    slice: rocksdb_Slice,
}

impl OwnedSlice {
    #[inline]
    fn set_data(&mut self, data: Option<Vec<u8>>) -> *mut rocksdb_Slice {
        match data {
            Some(data) => {
                self.data = data;
                self.slice = rocksdb_Slice {
                    data_: self.data.as_ptr() as _,
                    size_: self.data.len(),
                };
                &mut self.slice
            }
            None => {
                *self = Default::default();
                ptr::null_mut()
            }
        }
    }
}

impl Default for OwnedSlice {
    #[inline]
    fn default() -> Self {
        Self {
            data: Default::default(),
            slice: rocksdb_Slice {
                data_: ptr::null(),
                size_: 0,
            },
        }
    }
}

pub(crate) trait PathToSlice {
    unsafe fn path_to_slice(&self) -> rocksdb_Slice;
}

impl<T: AsRef<Path>> PathToSlice for T {
    #[inline]
    unsafe fn path_to_slice(&self) -> rocksdb_Slice {
        let p = self.as_ref().as_os_str().as_bytes();
        r(p)
    }
}

/// Options to control the behavior of a database (passed to DB::Open)
#[repr(transparent)]
pub struct RawOptions(rocksdb_Options);

impl RawOptions {
    /// All data will be in level 0 without any automatic compaction.
    /// It's recommended to manually call CompactRange(NULL, NULL) before reading
    /// from the database, because otherwise the read can be very slow.
    #[inline]
    pub fn prepare_for_bulk_load(&mut self) -> &mut Self {
        unsafe {
            tirocks_sys::crocksdb_options_prepare_for_bulk_load(self as *mut _ as _);
        }
        self
    }

    #[inline]
    pub fn as_db_options(&self) -> &RawDbOptions {
        unsafe {
            RawDbOptions::from_ptr(tirocks_sys::crocksdb_options_get_dboptions(
                self as *const _ as _,
            ))
        }
    }

    #[inline]
    pub fn as_db_options_mut(&mut self) -> &mut RawDbOptions {
        unsafe {
            RawDbOptions::from_ptr_mut(tirocks_sys::crocksdb_options_get_dboptions(
                self as *const _ as _,
            ))
        }
    }

    #[inline]
    pub fn as_cf_options(&self) -> &RawCfOptions {
        unsafe {
            RawCfOptions::from_ptr(tirocks_sys::crocksdb_options_get_cfoptions(
                self as *const _ as _,
            ))
        }
    }

    #[inline]
    pub fn as_cf_options_mut(&mut self) -> &mut RawCfOptions {
        unsafe {
            RawCfOptions::from_ptr_mut(tirocks_sys::crocksdb_options_get_cfoptions(
                self as *const _ as _,
            ))
        }
    }

    #[inline]
    pub(crate) unsafe fn from_ptr<'a>(ptr: *const rocksdb_Options) -> &'a RawOptions {
        &*(ptr as *const RawOptions)
    }

    #[inline]
    pub(crate) unsafe fn from_ptr_mut<'a>(ptr: *mut rocksdb_Options) -> &'a mut RawOptions {
        &mut *(ptr as *mut RawOptions)
    }
}

#[derive(Debug)]
pub struct Options {
    ptr: *mut RawOptions,
    env: Option<Arc<Env>>,
    comparator: Option<Arc<SysComparator>>,
}

impl Default for Options {
    #[inline]
    fn default() -> Self {
        let ptr = unsafe { tirocks_sys::crocksdb_options_create() };
        Self {
            ptr: ptr as *mut RawOptions,
            env: None,
            comparator: None,
        }
    }
}

impl Deref for Options {
    type Target = RawOptions;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}

impl DerefMut for Options {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.ptr }
    }
}

impl Options {
    /// Same as `DbOptions::set_env`.
    #[inline]
    pub fn set_env(&mut self, env: Arc<Env>) -> &mut Self {
        unsafe {
            self.as_db_options_mut().set_env(&env);
        }
        self.env = Some(env);
        self
    }

    /// Same as `CfOptions::set_comparator`.
    #[inline]
    pub fn set_comparator(&mut self, c: Arc<SysComparator>) -> &mut Self {
        unsafe {
            self.as_cf_options_mut().set_comparator(&c);
        }
        self.comparator = Some(c);
        self
    }
}

impl Drop for Options {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            tirocks_sys::crocksdb_options_destroy(self.ptr as _);
        }
    }
}

#[repr(transparent)]
pub struct RawTitanOptions(rocksdb_titandb_TitanOptions);

impl RawTitanOptions {
    #[inline]
    pub fn as_db_options(&self) -> &RawTitanDbOptions {
        unsafe {
            RawTitanDbOptions::from_ptr(tirocks_sys::ctitandb_options_get_dboptions(
                self as *const _ as _,
            ))
        }
    }

    #[inline]
    pub fn as_db_options_mut(&mut self) -> &mut RawTitanDbOptions {
        unsafe {
            RawTitanDbOptions::from_ptr_mut(tirocks_sys::ctitandb_options_get_dboptions(
                self as *mut _ as _,
            ))
        }
    }

    #[inline]
    pub fn as_cf_options(&self) -> &RawTitanCfOptions {
        unsafe {
            RawTitanCfOptions::from_ptr(tirocks_sys::ctitandb_options_get_cfoptions(
                self as *const _ as _,
            ))
        }
    }

    #[inline]
    pub fn as_cf_options_mut(&mut self) -> &mut RawTitanCfOptions {
        unsafe {
            RawTitanCfOptions::from_ptr_mut(tirocks_sys::ctitandb_options_get_cfoptions(
                self as *mut _ as _,
            ))
        }
    }
}

#[derive(Debug)]
pub struct TitanOptions {
    ptr: *mut RawTitanOptions,
    env: Option<Arc<Env>>,
    comparator: Option<Arc<SysComparator>>,
}

impl Default for TitanOptions {
    #[inline]
    fn default() -> Self {
        let ptr = unsafe { tirocks_sys::ctitandb_options_create() };
        Self {
            ptr: ptr as _,
            env: None,
            comparator: None,
        }
    }
}

impl Deref for TitanOptions {
    type Target = RawTitanOptions;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}

impl DerefMut for TitanOptions {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.ptr }
    }
}

impl TitanOptions {
    /// Same as `DbOptions::set_env`.
    #[inline]
    pub fn set_env(&mut self, env: Arc<Env>) -> &mut Self {
        unsafe {
            self.as_db_options_mut().set_env(&env);
        }
        self.env = Some(env);
        self
    }

    /// Same as `CfOptions::set_comparator`.
    #[inline]
    pub fn set_comparator(&mut self, c: Arc<SysComparator>) -> &mut Self {
        unsafe {
            self.as_cf_options_mut().set_comparator(&c);
        }
        self.comparator = Some(c);
        self
    }
}

impl Drop for TitanOptions {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            tirocks_sys::crocksdb_options_destroy(self.ptr as _);
        }
    }
}
