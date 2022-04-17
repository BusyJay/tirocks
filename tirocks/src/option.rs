// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

mod cf;
mod db;
mod flush;
mod read;
mod write;

use std::{os::unix::prelude::OsStrExt, path::Path, ptr};

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
use tirocks_sys::{r, rocksdb_CompressionType, rocksdb_Slice};
pub use write::WriteOptions;

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

// TODO: simplify the definition
macro_rules! simple_access {
    ($(#[$outer:meta])* imp $prefix:ident <$op:ident> $field:ident / $rename:ident / :$ty:ty$([$($new_tt:tt)*])?) => {
        paste::paste! {
            $(#[$outer])*
            #[inline]
            pub fn [<$op _ $field>](&mut self, val: $ty) -> &mut Self {
                unsafe {
                    tirocks_sys::[<$prefix _ $op _ $rename>](self.ptr, val $($($new_tt)*)?)
                }
                self
            }
        }
    };
    ($(#[$outer:meta])* imp $prefix:ident <$op:ident> $field:ident:$ty:ty$([$($new_tt:tt)*])?) => {
        $crate::option::simple_access!($(#[$outer])* imp $prefix <$op> $field / $field /:$ty $([$($new_tt)*])? );
    };
    ($(#[$outer:meta])* imp $prefix:ident $field:ident / $rename:ident / :$ty:ty$([$($new_tt:tt)*])?) => {
        $crate::option::simple_access!($(#[$outer])* imp $prefix <set> $field / $rename /:$ty $([$($new_tt)*])? );
    };
    ($(#[$outer:meta])* imp $prefix:ident $field:ident:$ty:ty$([$($new_tt:tt)*])?) => {
        $crate::option::simple_access!($(#[$outer])* imp $prefix <set> $field / $field /:$ty $([$($new_tt)*])? );
    };
    ($prefix:ident $($(#[$outer:meta])*$(<$op:ident>)?$field:ident$(/$rename:ident/)?:$ty:ty$([$($new_tt:tt)*])?)+) => {
        $(
            $crate::option::simple_access!(
                $(#[$outer])*
                imp $prefix $(<$op>)? $field$(/$rename/)?:$ty$([$($new_tt)*])?
            );
        )+
    };
    (#[$outer:meta]$($tt:tt)+) => {
        $crate::option::simple_access! {
            crocksdb_options
            #[$outer]$($tt)+
        }
    };
}

pub(crate) use simple_access;

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
