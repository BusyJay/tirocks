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

/// A helper macros to generate getters/setters that delegates calls to C functions.
/// ```skip
/// impl A {
///     simple_access! {
///         prefix
///         /// Docs about set
///         name1: Type [ .get() ]
///         /// Docs about get
///         (<get) name2: Type
///     }
/// }
/// ```
/// `simple_access` will generate setter `set_name1` with the default implementation call
/// to `prefix_set_name1(self.as_mut_ptr(), val.get())`. The expression inside `[]` will be
/// appended to value directly.
/// `(<)` means getter, `get` means the delegate C function is `prefix_get_name2`. If `get`
/// is omitted, then the function becomes `prefix_name2`.
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

/// A helper micros for handling FFI calls that need error handling.
///
/// It's simply translate the call
/// ```ignored
/// ffi_call!(func(...))
/// ```
/// to
/// ```ignored
/// let res = tirocks_sys::func(..., &mut status);
/// if status.ok() {
///     Ok(res)
/// } else {
///     Err(status)
/// }
/// ```
macro_rules! ffi_call {
    ($func:ident($($arg:expr,)+)) => ({
        let mut status = $crate::Status::default();
        let res = tirocks_sys::$func($($arg),+, status.as_mut_ptr());
        if status.ok() {
            Ok(res)
        } else {
            Err(status)
        }
    });
    ($func:ident($arg:expr, $($extra:expr),*)) => {
        $crate::util::ffi_call!($func($arg, $($extra,)*))
    };
    ($func:ident($arg:expr)) => {
        $crate::util::ffi_call!($func($arg,))
    };
    ($func:ident()) => ({
        let mut status = $crate::Status::default();
        let res = tirocks_sys::$func(status.as_mut_ptr());
        if status.ok() {
            Ok(res)
        } else {
            Err(status)
        }
    })
}

pub(crate) use ffi_call;

use crate::{db::RawCfHandle, RawDb};

pub unsafe fn split_pairs(
    pairs: &[(impl AsRef<[u8]>, impl AsRef<[u8]>)],
) -> (Vec<rocksdb_Slice>, Vec<rocksdb_Slice>) {
    let mut keys = Vec::with_capacity(pairs.len());
    let mut values = Vec::with_capacity(pairs.len());
    for (k, v) in pairs {
        keys.push(r(k.as_ref()));
        values.push(r(v.as_ref()));
    }
    (keys, values)
}

#[inline]
pub unsafe fn range_to_rocks(start: &impl AsRef<[u8]>, end: &impl AsRef<[u8]>) -> rocksdb_Range {
    rocksdb_Range {
        start: r(start.as_ref()),
        limit: r(end.as_ref()),
    }
}

// `FnOnce` is a more accurate type, but it will require Unpin.
unsafe extern "C" fn bytes_receiver<T: FnMut(&[u8])>(ptr: *mut c_void, buf: rocksdb_Slice) {
    (*(ptr as *mut T))(s(buf))
}

pub type BytesReceiver = unsafe extern "C" fn(*mut c_void, rocksdb_Slice);

pub fn wrap_string_receiver<T: FnMut(&[u8])>(receiver: &mut T) -> (*mut c_void, BytesReceiver) {
    (receiver as *mut T as *mut c_void, bytes_receiver::<T>)
}

/// A common factory for all types that creates types using `Default` trait.
pub struct DefaultFactory<F> {
    item: PhantomData<F>,
}

impl<F> DefaultFactory<F> {
    #[inline]
    pub(crate) fn c_name(&self) -> &CStr {
        CStr::from_bytes_with_nul(b"DefaultFactory\0").unwrap()
    }
}

impl<F> Default for DefaultFactory<F> {
    #[inline]
    fn default() -> Self {
        Self { item: PhantomData }
    }
}

unsafe impl<F> Send for DefaultFactory<F> {}
unsafe impl<F> Sync for DefaultFactory<F> {}

/// A common factory for items that can be cloned.
pub struct CloneFactory<F> {
    item: F,
}

impl<F> CloneFactory<F> {
    #[inline]
    pub fn new(item: F) -> Self {
        CloneFactory { item }
    }

    #[inline]
    pub(crate) fn c_name(&self) -> &CStr {
        CStr::from_bytes_with_nul(b"CloneFactory\0").unwrap()
    }

    #[inline]
    pub(crate) fn name(&self) -> &str {
        "CloneFactory"
    }
}

impl<F: Clone> CloneFactory<F> {
    pub(crate) fn clone(&self) -> F {
        self.item.clone()
    }
}

#[inline]
pub unsafe fn rocks_slice_to_array(arr: &[rocksdb_Slice]) -> Cow<[&[u8]]> {
    if tirocks_sys::rocks_slice_same_as_rust() {
        Cow::Borrowed(mem::transmute(arr))
    } else {
        let arr = arr.iter().map(|v| unsafe { s(*v) }).collect();
        Cow::Owned(arr)
    }
}

#[inline]
pub unsafe fn array_to_rocks_slice<'a>(arr: &'a [&[u8]]) -> Cow<'a, [rocksdb_Slice]> {
    if tirocks_sys::rocks_slice_same_as_rust() {
        Cow::Borrowed(mem::transmute(arr))
    } else {
        let arr = arr.iter().map(|v| unsafe { r(*v) }).collect();
        Cow::Owned(arr)
    }
}

pub type RustRange<'a> = (Option<&'a [u8]>, Option<&'a [u8]>);
pub type RocksRange = (Option<rocksdb_Slice>, Option<rocksdb_Slice>);

pub unsafe fn range_to_range_ptr(ranges: &[RustRange]) -> (Vec<RocksRange>, Vec<rocksdb_RangePtr>) {
    let rocks_ranges: Vec<_> = ranges
        .iter()
        .map(|pair| (pair.0.map(|k| r(k)), pair.1.map(|k| r(k))))
        .collect();
    // It may be optimized by `rocks_slice_same_as_rust`, but it seems too risky.
    let range_ptrs: Vec<_> = rocks_ranges
        .iter()
        .map(|(begin, end)| rocksdb_RangePtr {
            start: begin.as_ref().map_or_else(ptr::null, |k| k),
            limit: end.as_ref().map_or_else(ptr::null, |k| k),
        })
        .collect();
    (rocks_ranges, range_ptrs)
}
