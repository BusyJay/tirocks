// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

pub struct Statistics {
    ptr: *mut tirocks_sys::crocksdb_statistics_t,
}

impl Default for Statistics {
    #[inline]
    fn default() -> Self {
        Self {
            ptr: unsafe { tirocks_sys::crocksdb_statistics_create() },
        }
    }
}

impl Statistics {
    #[inline]
    pub(crate) fn as_mut_ptr(&self) -> *mut tirocks_sys::crocksdb_statistics_t {
        self.ptr
    }
}

impl Drop for Statistics {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            tirocks_sys::crocksdb_statistics_destroy(self.ptr);
        }
    }
}
