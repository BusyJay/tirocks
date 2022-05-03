// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::{str::Utf8Error, ops::Deref, ptr::NonNull, mem};

use crate::{Result, Status, util::{utf8_name, check_status}, db::Db};
use tirocks_sys::{r, rocksdb_ColumnFamilyHandle, s};

use super::db::DbRef;

pub const DEFAULT_CF_NAME: &str = "default";

#[derive(Debug)]
#[repr(transparent)]
pub struct RawColumnFamilyHandle(rocksdb_ColumnFamilyHandle);

impl RawColumnFamilyHandle {
    #[inline]
    pub fn id(&self) -> u32 {
        unsafe { tirocks_sys::crocksdb_column_family_handle_id(self as *const _ as _) }
    }

    #[inline]
    pub fn name(&self) -> std::result::Result<&str, Utf8Error> {
        unsafe {
            let mut name = r(&[]);
            tirocks_sys::crocksdb_column_family_handle_name(self as *const _ as _, &mut name);
            std::str::from_utf8(s(name))
        }
    }

    #[inline]
    pub unsafe fn drop(&mut self, db: &Db) -> Result<()> {
        let mut s = Status::default();
        tirocks_sys::crocksdb_column_family_handle_destroy(
            db.get(),
            self as *mut _ as _,
            s.as_mut_ptr(),
        );
        check_status!(s)
    }

    pub fn is_default_cf(&self) -> bool {
        self.name().map_or(false, |n| n == DEFAULT_CF_NAME)
    }

    pub(crate) unsafe fn destroy(&mut self, db: &Db) -> Result<()> {
        let mut s = Status::default();
        unsafe {
            tirocks_sys::crocksdb_drop_column_family(db.get(), self as *mut _  as _, s.as_mut_ptr());
        }
        check_status!(s)
    }
}

pub struct ColumnFamilyHandle<D: DbRef> {
    ptr: NonNull<rocksdb_ColumnFamilyHandle>,
    // Make sure handle won't outlive db.
    _db: D,
}

impl<D: DbRef> ColumnFamilyHandle<D> {
    pub fn from_db(db: D, name: &str) -> Option<Self> {
        unsafe {
            let ptr = db.visit(|d| d.get_cf_raw(name));
            Some(Self {
                ptr: mem::transmute(*ptr?),
                _db: db,
            })
        }
    }
}

impl<D: DbRef> Deref for ColumnFamilyHandle<D> {
    type Target = RawColumnFamilyHandle;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.ptr.as_ptr() as *mut RawColumnFamilyHandle) }
    }
}
