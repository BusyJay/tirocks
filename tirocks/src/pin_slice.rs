// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::ops::Deref;

use tirocks_sys::{r, rocksdb_PinnableSlice, s};

/// A Slice that can be pinned with some cleanup tasks, which will be run upon
/// `reset()` or object destruction, whichever is invoked first. This can be used
/// to avoid memcpy by having the PinSlice object referring to the data
/// that is locked in the memory and release them after the data is consumed.
pub struct PinSlice {
    ptr: *mut rocksdb_PinnableSlice,
}

impl Default for PinSlice {
    #[inline]
    fn default() -> Self {
        let ptr = unsafe { tirocks_sys::crocksdb_pinnableslice_create() };
        Self { ptr }
    }
}

impl Deref for PinSlice {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe {
            let mut buf = r(&[]);
            tirocks_sys::crocksdb_pinnableslice_value(self.ptr, &mut buf);
            s(buf)
        }
    }
}

impl PinSlice {
    pub(crate) fn get_ptr(&self) -> *mut rocksdb_PinnableSlice {
        self.ptr
    }

    #[inline]
    pub fn reset(&mut self) {
        unsafe { tirocks_sys::crocksdb_pinnableslice_reset(self.ptr) }
    }
}

impl Drop for PinSlice {
    #[inline]
    fn drop(&mut self) {
        unsafe { tirocks_sys::crocksdb_pinnableslice_destroy(self.ptr) }
    }
}
