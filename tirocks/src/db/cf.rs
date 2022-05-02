// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use tirocks_sys::rocksdb_ColumnFamilyHandle;

#[derive(Debug)]
pub struct ColumnFamilyHandle {
    ptr: *mut rocksdb_ColumnFamilyHandle,
}

impl ColumnFamilyHandle {
    #[inline]
    pub(crate) fn from_ptr(ptr: *mut rocksdb_ColumnFamilyHandle) -> ColumnFamilyHandle {
        ColumnFamilyHandle { ptr }
    }

    #[inline]
    pub fn id(&self) -> u32 {
        unsafe { tirocks_sys::crocksdb_column_family_handle_id(self.ptr) }
    }
}

impl Drop for ColumnFamilyHandle {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            tirocks_sys::crocksdb_column_family_handle_destroy(self.ptr);
        }
    }
}
