// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::ffi::CStr;

use libc::{c_char, c_void};
use tirocks_sys::{crocksdb_slicetransform_t, r, rocksdb_Slice, s};

/// A SliceTransform is a generic pluggable way of transforming one string
/// to another. Its primary use-case is in configuring rocksdb
/// to store prefix blooms by setting prefix_extractor in
/// ColumnFamilyOptions.
pub trait SliceTransform: Send + Sync {
    /// Return the name of this transformation.
    fn name(&self) -> &CStr;

    /// Extract a prefix from a specified key. This method is called when
    /// a key is inserted into the db, and the returned slice is used to
    /// create a bloom filter.
    fn transform(&self, key: &[u8]) -> &[u8];

    /// Determine whether the specified key is compatible with the logic
    /// specified in the Transform method. This method is invoked for every
    /// key that is inserted into the db. If this method returns true,
    /// then Transform is called to translate the key to its prefix and
    /// that returned prefix is inserted into the bloom filter. If this
    /// method returns false, then the call to Transform is skipped and
    /// no prefix is inserted into the bloom filters.
    ///
    /// For example, if the Transform method operates on a fixed length
    /// prefix of size 4, then an invocation to InDomain("abc") returns
    /// false because the specified key length(3) is shorter than the
    /// prefix size of 4.
    ///
    /// Wiki documentation here:
    /// https://github.com/facebook/rocksdb/wiki/Prefix-Seek-API-Changes
    fn in_domain(&self, key: &[u8]) -> bool;

    /// Some SliceTransform will have a full length which can be used to
    /// determine if two keys are consecuitive. Can be disabled by always
    /// returning 0
    #[inline]
    fn full_length_enabled(&self, _len: &mut usize) -> bool {
        false
    }

    /// Transform(s)=Transform(`prefix`) for any s with `prefix` as a prefix.
    ///
    /// This function is not used by RocksDB, but for users. If users pass
    /// Options by string to RocksDB, they might not know what prefix extractor
    /// they are using. This function is to help users can determine:
    ///   if they want to iterate all keys prefixing `prefix`, whether it is
    ///   safe to use prefix bloom filter and seek to key `prefix`.
    /// If this function returns true, this means a user can Seek() to a prefix
    /// using the bloom filter. Otherwise, user needs to skip the bloom filter
    /// by setting ReadOptions.total_order_seek = true.
    ///
    /// Here is an example: Suppose we implement a slice transform that returns
    /// the first part of the string after splitting it using delimiter ",":
    /// 1. SameResultWhenAppended("abc,") should return true. If applying prefix
    ///    bloom filter using it, all slices matching "abc:.*" will be extracted
    ///    to "abc,", so any SST file or memtable containing any of those key
    ///    will not be filtered out.
    /// 2. SameResultWhenAppended("abc") should return false. A user will not
    ///    guaranteed to see all the keys matching "abc.*" if a user seek to "abc"
    ///    against a DB with the same setting. If one SST file only contains
    ///    "abcd,e", the file can be filtered out and the key will be invisible.
    ///
    /// i.e., an implementation always returning false is safe.
    #[inline]
    fn same_result_when_appended(&self, _prefix: &[u8]) -> bool {
        false
    }
}

extern "C" fn name<T: SliceTransform>(transform: *mut c_void) -> *const c_char {
    unsafe {
        let trans = &*(transform as *mut T);
        trans.name().as_ptr()
    }
}

extern "C" fn destructor<T: SliceTransform>(transform: *mut c_void) {
    unsafe {
        drop(Box::from_raw(transform as *mut T));
    }
}

extern "C" fn transform<T: SliceTransform>(
    transform: *mut c_void,
    key: rocksdb_Slice,
    dest: *mut rocksdb_Slice,
) {
    unsafe {
        let trans = &*(transform as *mut T);
        *dest = r(trans.transform(s(key)));
    }
}

extern "C" fn in_domain<T: SliceTransform>(transform: *mut c_void, key: rocksdb_Slice) -> bool {
    unsafe {
        let trans = &*(transform as *mut T);
        trans.in_domain(s(key))
    }
}

extern "C" fn full_length_enabled<T: SliceTransform>(
    transform: *mut c_void,
    len: *mut usize,
) -> bool {
    unsafe {
        let trans = &*(transform as *mut T);
        trans.full_length_enabled(&mut *len)
    }
}

extern "C" fn same_result_when_appended<T: SliceTransform>(
    transform: *mut c_void,
    key: rocksdb_Slice,
) -> bool {
    unsafe {
        let trans = &*(transform as *mut T);
        trans.same_result_when_appended(s(key))
    }
}

/// A wrapped factory that can be shared by multiple ColumnFamilies and DBs.
#[derive(Debug)]
pub struct SysSliceTransform {
    ptr: *mut crocksdb_slicetransform_t,
}

impl SysSliceTransform {
    #[inline]
    pub fn new<T: SliceTransform>(t: T) -> SysSliceTransform {
        let ptr = unsafe {
            tirocks_sys::crocksdb_slicetransform_create(
                Box::into_raw(Box::new(t)) as *mut c_void,
                Some(destructor::<T>),
                Some(transform::<T>),
                Some(in_domain::<T>),
                Some(full_length_enabled::<T>),
                Some(same_result_when_appended::<T>),
                Some(name::<T>),
            )
        };
        SysSliceTransform { ptr }
    }

    #[inline]
    pub(crate) fn get(&self) -> *mut crocksdb_slicetransform_t {
        self.ptr
    }
}

impl Drop for SysSliceTransform {
    #[inline]
    fn drop(&mut self) {
        unsafe { tirocks_sys::crocksdb_slicetransform_destroy(self.ptr) }
    }
}
