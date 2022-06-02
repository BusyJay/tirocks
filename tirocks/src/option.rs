// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

mod db;
mod flush;
mod read;
mod write;

use std::{
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    ptr,
    sync::Arc,
};

pub use db::{
    DbOptions, OwnedRawDbOptions, OwnedRawTitanDbOptions, RawDbOptions, RawTitanDbOptions,
    TitanDbOptions,
};
pub use flush::{
    BottommostLevelCompaction, CompactRangeOptions, CompactionOptions, FlushOptions,
    IngestExternalFileOptions,
};
pub use read::{ReadOptions, ReadTier};
use tirocks_sys::{
    rocksdb_CompressionType, rocksdb_Options, rocksdb_Slice, rocksdb_titandb_TitanOptions,
};
pub use write::WriteOptions;

use crate::{comparator::SysComparator, env::Env};

pub type CompressionType = rocksdb_CompressionType;

/// Get all supported comressions.
pub fn supported_compressions() -> Vec<CompressionType> {
    unsafe {
        let n = tirocks_sys::crocksdb_get_supported_compression_number();
        let mut v = Vec::with_capacity(n);
        tirocks_sys::crocksdb_get_supported_compression(v.as_mut_ptr(), n);
        v.set_len(n);
        v
    }
}

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
