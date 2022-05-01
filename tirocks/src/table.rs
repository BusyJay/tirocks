// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

pub mod block;
pub mod plain;

use tirocks_sys::crocksdb_tablefactory_t;

use self::{block::BlockBasedTableOptions, plain::PlainTableOptions};

#[derive(Debug)]
pub struct SysTableFactory {
    ptr: *mut crocksdb_tablefactory_t,
}

impl SysTableFactory {
    #[inline]
    pub fn new_block_based(opt: &BlockBasedTableOptions) -> Self {
        let ptr = unsafe { tirocks_sys::crocksdb_tablefactory_create_block_based(opt.get()) };
        Self { ptr }
    }

    #[inline]
    pub fn new_plain(opt: &PlainTableOptions) -> Self {
        let ptr = unsafe { tirocks_sys::crocksdb_tablefactory_create_plain(opt.get()) };
        Self { ptr }
    }

    #[inline]
    pub(crate) fn get(&self) -> *mut crocksdb_tablefactory_t {
        self.ptr
    }
}

impl Drop for SysTableFactory {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            tirocks_sys::crocksdb_tablefactory_destroy(self.ptr);
        }
    }
}
