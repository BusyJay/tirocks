// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::{cmp::Ordering, ffi::CStr};

use libc::{c_char, c_void};
use tirocks_sys::{crocksdb_comparator_t, rocksdb_Slice, s};

pub trait Comparator: Sync + Send {
    fn cmp(&self, lhs: &[u8], rhs: &[u8]) -> Ordering;
    fn name(&self) -> &CStr;
}

extern "C" fn name<C: Comparator>(c: *mut c_void) -> *const c_char {
    unsafe {
        let c = &*(c as *mut C);
        c.name().as_ptr()
    }
}

extern "C" fn destructor<C: Comparator>(c: *mut c_void) {
    unsafe {
        drop(Box::from_raw(c as *mut C));
    }
}

extern "C" fn compare<C: Comparator>(
    c: *mut c_void,
    lhs: rocksdb_Slice,
    rhs: rocksdb_Slice,
) -> i32 {
    unsafe {
        let c = &*(c as *mut C);
        match c.cmp(s(lhs), s(rhs)) {
            Ordering::Less => -1,
            Ordering::Equal => 0,
            Ordering::Greater => 1,
        }
    }
}

/// A wrapped factory that can be shared by multiple ColumnFamilies and DBs.
#[derive(Debug)]
pub struct SysComparator {
    ptr: *mut crocksdb_comparator_t,
}

impl SysComparator {
    #[inline]
    pub fn new<C: Comparator>(c: C) -> SysComparator {
        let ptr = unsafe {
            tirocks_sys::crocksdb_comparator_create(
                Box::into_raw(Box::new(c)) as *mut c_void,
                Some(destructor::<C>),
                Some(compare::<C>),
                Some(name::<C>),
            )
        };
        SysComparator { ptr }
    }

    #[inline]
    pub(crate) fn get_ptr(&self) -> *mut crocksdb_comparator_t {
        self.ptr
    }
}

impl Drop for SysComparator {
    #[inline]
    fn drop(&mut self) {
        unsafe { tirocks_sys::crocksdb_comparator_destroy(self.ptr) }
    }
}
