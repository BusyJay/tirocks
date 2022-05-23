// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

// For now the macro is only used by encryption.
#![cfg_attr(not(feature = "encryption"), allow(unused))]

use std::{
    borrow::Cow, ffi::CStr, marker::PhantomData, mem, os::unix::prelude::OsStrExt, path::Path, ptr,
};

use crate::{Result, Status};
use libc::c_void;
use tirocks_sys::{r, rocksdb_Range, rocksdb_RangePtr, rocksdb_Slice, s};

macro_rules! utf8_name {
    ($slice:expr, $ctx:expr, $status:expr) => {
        match std::str::from_utf8(tirocks_sys::s($slice)) {
            Ok(n) => n,
            Err(e) => {
                std::ptr::write(
                    $status,
                    Status::with_invalid_argument(format!("{}: {}", $ctx, e)).into_raw(),
                );
                return;
            }
        }
    };
}

pub(crate) use utf8_name;

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

macro_rules! expand_one_row {
    (setter $(#[$outer:meta])* $prefix:ident <$op:ident> $field:ident / $rename:ident / :$ty:ty$([$($new_tt:tt)*])?) => {
        paste::paste! {
            $(#[$outer])*
            #[inline]
            pub fn [<$op _ $field>](&mut self, val: $ty) -> &mut Self {
                unsafe {
                    tirocks_sys::[<$prefix _ $op _ $rename>](self.as_mut_ptr(), val $($($new_tt)*)?)
                }
                self
            }
        }
    };
    (getter $(#[$outer:meta])* $prefix:ident $(<$op:ident>)? $field:ident / $rename:ident / :$ty:ty$([$($new_tt:tt)*])?) => {
        paste::paste! {
            $(#[$outer])*
            #[inline]
            pub fn $field(&self) -> $ty {
                unsafe {
                    $($($new_tt)*)? tirocks_sys::[<$prefix _ $($op _)? $rename>](self.as_ptr())
                }
            }
        }
    };
}

pub(crate) use expand_one_row;

/// A helper macros to generate getters/setters.
/// ```skip
/// impl A {
///     simple_access! {
///         prefix
///         /// Docs
///         name: Type
///     }
/// }
/// ```
/// `simple_access` will generate setter `set_name` with the default implementation call
/// to `prefix_set_name(self.as_mut_ptr(), val)`.
// TODO: simplify the definition
macro_rules! simple_access {
    ($(#[$outer:meta])* <$method:ident> $prefix:ident $(<$op:ident>)? $field:ident / $rename:ident / :$ty:ty$([$($new_tt:tt)*])?) => {
        $crate::util::expand_one_row!($method $(#[$outer])* $prefix $(<$op>)? $field / $rename / :$ty $([$($new_tt)*])? );
    };
    ($(#[$outer:meta])* <$method:ident> $prefix:ident $(<$op:ident>)? $field:ident:$ty:ty$([$($new_tt:tt)*])?) => {
        $crate::util::expand_one_row!($method $(#[$outer])* $prefix $(<$op>)? $field / $field /:$ty $([$($new_tt)*])? );
    };
    ($(#[$outer:meta])* imp $prefix:ident $field:ident$(/$rename:ident/)?:$ty:ty$([$($new_tt:tt)*])?) => {
        $crate::util::simple_access!($(#[$outer])* <setter> $prefix <set> $field$(/$rename/)?:$ty$([$($new_tt)*])?);
    };
    ($(#[$outer:meta])* imp $prefix:ident ($op:ident)$field:ident$(/$rename:ident/)?:$ty:ty$([$($new_tt:tt)*])?) => {
        $crate::util::simple_access!($(#[$outer])* <setter> $prefix <$op> $field$(/$rename/)?:$ty$([$($new_tt)*])?);
    };
    ($(#[$outer:meta])* imp $prefix:ident (<$($op:ident)?) $field:ident$(/$rename:ident/)?:$ty:ty$([$($new_tt:tt)*])?) => {
        $crate::util::simple_access!($(#[$outer])* <getter> $prefix $(<$op>)? $field$(/$rename/)?:$ty$([$($new_tt)*])?);
    };
    ($prefix:ident $($(#[$outer:meta])*$(($($t:tt)*))?$field:ident$(/$rename:ident/)?:$ty:ty$([$($new_tt:tt)*])?)+) => {
        $(
            $crate::util::simple_access!(
                $(#[$outer])*
                imp $prefix $(($($t)*))? $field$(/$rename/)?:$ty$([$($new_tt)*])?
            );
        )+
    };
    (#[$outer:meta]$($tt:tt)+) => {
        $crate::util::simple_access! {
            crocksdb_options
            #[$outer]$($tt)+
        }
    };
}

pub(crate) use simple_access;

// `FnOnce` is a more accurate type, but it will require Unpin.
unsafe extern "C" fn bytes_receiver<T: FnMut(&[u8])>(ptr: *mut c_void, buf: rocksdb_Slice) {
    (*(ptr as *mut T))(s(buf))
}

pub type BytesReceiver = unsafe extern "C" fn(*mut c_void, rocksdb_Slice);

pub fn wrap_string_receiver<T: FnMut(&[u8])>(receiver: &mut T) -> (*mut c_void, BytesReceiver) {
    (receiver as *mut T as *mut c_void, bytes_receiver::<T>)
}
