// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::{
    ops::Deref,
    ptr::NonNull,
    str::Utf8Error,
    sync::atomic::{self, AtomicUsize, Ordering},
};

use crate::{util::ffi_call, RawDb, Result, Status};
use tirocks_sys::{r, rocksdb_ColumnFamilyHandle, rocksdb_DB, s};

use super::db::DbRef;

pub const DEFAULT_CF_NAME: &str = "default";

#[derive(Debug)]
#[repr(transparent)]
pub struct RawCfHandle(rocksdb_ColumnFamilyHandle);

impl RawCfHandle {
    /// The ID of cf.
    #[inline]
    pub fn id(&self) -> u32 {
        unsafe { tirocks_sys::crocksdb_column_family_handle_id(self.get_ptr()) }
    }

    /// The name of cf.
    #[inline]
    pub fn name(&self) -> std::result::Result<&str, Utf8Error> {
        unsafe {
            let mut name = r(&[]);
            tirocks_sys::crocksdb_column_family_handle_name(self.get_ptr(), &mut name);
            std::str::from_utf8(s(name))
        }
    }

    /// Check if the current cf is the default cf.
    pub fn is_default_cf(&self) -> bool {
        self.name().map_or(false, |n| n == DEFAULT_CF_NAME)
    }

    pub(crate) fn get_ptr(&self) -> *mut rocksdb_ColumnFamilyHandle {
        self as *const _ as *mut _
    }
}

#[derive(Debug)]
struct Inner {
    ptr: *mut rocksdb_ColumnFamilyHandle,
    managed: bool,
    ref_count: AtomicUsize,
}

#[derive(Debug)]
pub struct RefCountedCfHandle {
    inner: NonNull<Inner>,
}

impl RefCountedCfHandle {
    #[inline]
    pub unsafe fn maybe_drop(&mut self, db: *mut rocksdb_DB) -> Result<bool> {
        let cnt = self
            .inner
            .as_ref()
            .ref_count
            .fetch_sub(1, Ordering::Release);
        if cnt > 1 {
            return Ok(false);
        }
        // TODO: thread sanitizer doesn't support fence.
        atomic::fence(Ordering::Acquire);
        let res = if self.inner.as_ref().managed {
            ffi_call!(crocksdb_column_family_handle_destroy(db, self.get_ptr()))
        } else {
            Ok(())
        };
        drop(Box::from(self.inner.as_ptr()));
        res?;
        Ok(true)
    }

    pub(crate) unsafe fn destroy(&mut self, db: *mut rocksdb_DB) -> Result<()> {
        ffi_call!(crocksdb_drop_column_family(db, self.get_ptr()))
    }

    pub(crate) fn get_ptr(&self) -> *mut rocksdb_ColumnFamilyHandle {
        unsafe { self.inner.as_ref().ptr }
    }

    #[inline]
    pub(crate) unsafe fn from_ptr(
        ptr: *mut rocksdb_ColumnFamilyHandle,
        managed: bool,
    ) -> RefCountedCfHandle {
        let inner = Box::into_raw(Box::new(Inner {
            ptr,
            managed,
            ref_count: AtomicUsize::new(1),
        }));
        RefCountedCfHandle {
            inner: NonNull::new_unchecked(inner),
        }
    }
}

impl Deref for RefCountedCfHandle {
    type Target = RawCfHandle;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.inner.as_ref().ptr as *mut RawCfHandle) }
    }
}

impl Clone for RefCountedCfHandle {
    #[inline]
    fn clone(&self) -> Self {
        unsafe {
            self.inner
                .as_ref()
                .ref_count
                .fetch_add(1, Ordering::Relaxed);
            Self { inner: self.inner }
        }
    }
}

pub struct CfHandle<D: DbRef> {
    handle: RefCountedCfHandle,
    // Make sure handle won't outlive db.
    db: D,
}

impl<D: DbRef> CfHandle<D> {
    /// Get the handle of given cf.
    pub fn new(db: D, name: &str) -> Option<Self> {
        unsafe {
            let handle = db.visit(|d| d.cf_raw(name).cloned())?;
            Some(Self { handle, db })
        }
    }

    /// Create the handle of the default cf.
    pub fn default(db: D) -> Self {
        unsafe {
            // Search instead of using raw pointer to avoid allocation.
            let handle = db.visit(|d| d.cf_raw(DEFAULT_CF_NAME).cloned()).unwrap();
            Self { handle, db }
        }
    }
}

impl<D: DbRef> Deref for CfHandle<D> {
    type Target = RawCfHandle;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl<D: DbRef> Drop for CfHandle<D> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            let handle = &mut self.handle;
            self.db.visit(|d| {
                let _ = handle.maybe_drop(d.get_ptr());
            });
        }
    }
}

impl RawDb {
    /// Get the raw handle of the default cf.
    pub fn default_cf(&self) -> &RawCfHandle {
        unsafe { &*(self.default_cf_raw() as *mut RawCfHandle) }
    }

    /// Get the raw handle pointer of the default cf.
    pub fn default_cf_raw(&self) -> *mut rocksdb_ColumnFamilyHandle {
        unsafe { tirocks_sys::crocksdb_get_default_column_family(self.get_ptr()) }
    }
}
