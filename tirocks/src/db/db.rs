// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::sync::Arc;
use tirocks_sys::rocksdb_DB;

use crate::{comparator::SysComparator, env::Env};

use crate::db::cf::ColumnFamilyHandle;

#[repr(transparent)]
pub struct RawDb(rocksdb_DB);

#[derive(Debug)]
pub struct Db {
    ptr: *mut RawDb,
    env: Option<Arc<Env>>,
    comparator: Vec<Arc<SysComparator>>,
    handles: Vec<ColumnFamilyHandle>,
    is_titan: bool,
}

impl Drop for Db {
    #[inline]
    fn drop(&mut self) {
        self.handles.clear();
        unsafe {
            tirocks_sys::crocksdb_destroy(self.ptr as _);
        }
    }
}

impl Db {
    pub(crate) fn new(
        ptr: *mut RawDb,
        env: Option<Arc<Env>>,
        comparator: Vec<Arc<SysComparator>>,
        handles: Vec<ColumnFamilyHandle>,
        is_titan: bool,
    ) -> Self {
        Self {
            ptr,
            env,
            comparator,
            handles,
            is_titan,
        }
    }
}
