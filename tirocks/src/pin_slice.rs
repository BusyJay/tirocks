// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use tirocks_sys::{r, rocksdb_PinnableSlice, s};

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

impl AsRef<[u8]> for PinSlice {
    #[inline]
    fn as_ref(&self) -> &[u8] {
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
