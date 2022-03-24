// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

mod flush;
mod read;
mod write;

use std::ptr;

pub use flush::{
    BottommostLevelCompaction, CompactRangeOptions, CompactionOptions, FlushOptions,
    IngestExternalFileOptions,
};
pub use read::{ReadOptions, ReadTier};
use tirocks_sys::{rocksdb_CompressionType, rocksdb_Slice};
pub use write::WriteOptions;

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

pub type CompressionType = rocksdb_CompressionType;
