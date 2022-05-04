// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use libc::c_void;
use std::ffi::CStr;
use std::ops::Deref;
use std::path::Path;
use std::ptr::NonNull;
use std::slice;
use std::sync::{Arc, Mutex};
use tirocks_sys::{r, rocksdb_DB, rocksdb_Iterator};

use crate::option::{CfOptions, DbOptions, PathToSlice, ReadOptions, TitanCfOptions, WriteOptions};
use crate::util::check_status;
use crate::{comparator::SysComparator, env::Env};
use crate::{Code, Result, Status};

use crate::db::cf::RawColumnFamilyHandle;

use super::cf::DEFAULT_CF_NAME;
use super::iter::{Iterable, RawIterator};
use super::pin_slice::PinSlice;
use super::snap::RawSnapshot;

pub trait RawDbRef {
    fn visit<T>(&self, f: impl FnOnce(&RawDb) -> T) -> T;
}

impl<'a> RawDbRef for &'a RawDb {
    #[inline]
    fn visit<T>(&self, f: impl FnOnce(&RawDb) -> T) -> T {
        f(self)
    }
}

pub trait DbRef {
    fn visit<T>(&self, f: impl FnOnce(&Db) -> T) -> T;
}

impl<'a> DbRef for &'a Db {
    #[inline]
    fn visit<T>(&self, f: impl FnOnce(&Db) -> T) -> T {
        f(self)
    }
}
impl DbRef for Db {
    #[inline]
    fn visit<T>(&self, f: impl FnOnce(&Db) -> T) -> T {
        f(self)
    }
}
impl DbRef for Arc<Db> {
    #[inline]
    fn visit<T>(&self, f: impl FnOnce(&Db) -> T) -> T {
        f(self)
    }
}
impl DbRef for Arc<Mutex<Db>> {
    fn visit<T>(&self, f: impl FnOnce(&Db) -> T) -> T {
        f(&self.lock().unwrap())
    }
}

impl<R> RawDbRef for R
where
    R: DbRef,
{
    #[inline]
    fn visit<T>(&self, f: impl FnOnce(&RawDb) -> T) -> T {
        DbRef::visit(self, |db| unsafe { f(RawDb::from_ptr(db.as_ptr())) })
    }
}

#[repr(transparent)]
pub struct RawDb(rocksdb_DB);

impl RawDb {
    #[inline]
    pub(crate) fn as_ptr(&self) -> *mut rocksdb_DB {
        self as *const RawDb as *mut rocksdb_DB
    }

    pub(crate) unsafe fn from_ptr<'a>(ptr: *const rocksdb_DB) -> &'a RawDb {
        &*(ptr as *const RawDb)
    }

    pub fn put(&self, opt: &WriteOptions, key: &[u8], val: &[u8]) -> Result<()> {
        let mut s = Status::default();
        unsafe {
            tirocks_sys::crocksdb_put(self.as_ptr(), opt.get(), r(key), r(val), s.as_mut_ptr());
        }
        check_status!(s)
    }

    pub fn put_cf(
        &self,
        opt: &WriteOptions,
        cf: &RawColumnFamilyHandle,
        key: &[u8],
        val: &[u8],
    ) -> Result<()> {
        let mut s = Status::default();
        unsafe {
            tirocks_sys::crocksdb_put_cf(
                self.as_ptr(),
                opt.get(),
                cf.as_mut_ptr(),
                r(key),
                r(val),
                s.as_mut_ptr(),
            );
        }
        check_status!(s)
    }

    pub fn delete(&self, opt: &WriteOptions, key: &[u8]) -> Result<()> {
        let mut s = Status::default();
        unsafe {
            tirocks_sys::crocksdb_delete(self.as_ptr(), opt.get(), r(key), s.as_mut_ptr());
        }
        check_status!(s)
    }

    pub fn delete_cf(
        &self,
        opt: &WriteOptions,
        cf: &RawColumnFamilyHandle,
        key: &[u8],
    ) -> Result<()> {
        let mut s = Status::default();
        unsafe {
            tirocks_sys::crocksdb_delete_cf(
                self.as_ptr(),
                opt.get(),
                cf.as_mut_ptr(),
                r(key),
                s.as_mut_ptr(),
            );
        }
        check_status!(s)
    }

    pub fn single_delete(&self, opt: &WriteOptions, key: &[u8]) -> Result<()> {
        let mut s = Status::default();
        unsafe {
            tirocks_sys::crocksdb_single_delete(self.as_ptr(), opt.get(), r(key), s.as_mut_ptr());
        }
        check_status!(s)
    }

    pub fn single_delete_cf(
        &self,
        opt: &WriteOptions,
        cf: &RawColumnFamilyHandle,
        key: &[u8],
    ) -> Result<()> {
        let mut s = Status::default();
        unsafe {
            tirocks_sys::crocksdb_single_delete_cf(
                self.as_ptr(),
                opt.get(),
                cf.as_mut_ptr(),
                r(key),
                s.as_mut_ptr(),
            );
        }
        check_status!(s)
    }

    pub fn delete_range(
        &self,
        opt: &WriteOptions,
        cf: &RawColumnFamilyHandle,
        begin_key: &[u8],
        end_key: &[u8],
    ) -> Result<()> {
        let mut s = Status::default();
        unsafe {
            tirocks_sys::crocksdb_delete_range_cf(
                self.as_ptr(),
                opt.get(),
                cf.as_mut_ptr(),
                r(begin_key),
                r(end_key),
                s.as_mut_ptr(),
            );
        }
        check_status!(s)
    }

    fn check_get(val_ptr: *mut u8, len: usize, s: Status) -> Result<Option<Vec<u8>>> {
        if s.ok() {
            unsafe {
                let res = slice::from_raw_parts(val_ptr, len).to_vec();
                libc::free(val_ptr as *mut c_void);
                Ok(Some(res))
            }
        } else if s.code() == Code::kNotFound {
            Ok(None)
        } else {
            Err(s)
        }
    }

    pub fn get(&self, opt: &ReadOptions, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let mut s = Status::default();
        let mut len = 0;
        let val_ptr = unsafe {
            tirocks_sys::crocksdb_get(
                self.as_ptr(),
                opt.get() as _,
                r(key),
                &mut len,
                s.as_mut_ptr(),
            )
        };
        Self::check_get(val_ptr as _, len, s)
    }

    pub fn get_cf(
        &self,
        opt: &ReadOptions,
        cf: &RawColumnFamilyHandle,
        key: &[u8],
    ) -> Result<Option<Vec<u8>>> {
        let mut s = Status::default();
        let mut len = 0;
        let val_ptr = unsafe {
            tirocks_sys::crocksdb_get_cf(
                self.as_ptr(),
                opt.get() as _,
                cf.as_mut_ptr(),
                r(key),
                &mut len,
                s.as_mut_ptr(),
            )
        };
        Self::check_get(val_ptr as _, len, s)
    }

    fn check_get_to(s: Status) -> Result<bool> {
        if s.ok() {
            Ok(true)
        } else if s.code() == Code::kNotFound {
            Ok(false)
        } else {
            Err(s)
        }
    }

    pub fn get_to(&self, opt: &ReadOptions, key: &[u8], value: &mut PinSlice) -> Result<bool> {
        let mut s = Status::default();
        unsafe {
            tirocks_sys::crocksdb_get_pinned(
                self.as_ptr(),
                opt.get() as _,
                r(key),
                value.get(),
                s.as_mut_ptr(),
            )
        };
        Self::check_get_to(s)
    }

    pub fn get_cf_to(
        &self,
        opt: &ReadOptions,
        cf: &RawColumnFamilyHandle,
        key: &[u8],
        value: &mut PinSlice,
    ) -> Result<bool> {
        let mut s = Status::default();
        unsafe {
            tirocks_sys::crocksdb_get_pinned_cf(
                self.as_ptr(),
                opt.get() as _,
                cf.as_mut_ptr(),
                r(key),
                value.get(),
                s.as_mut_ptr(),
            )
        };
        Self::check_get_to(s)
    }

    pub fn iter<'a>(&'a self, read: &'a mut ReadOptions) -> RawIterator<'a> {
        RawIterator::new(self, read)
    }

    pub fn iter_cf<'a>(
        &'a self,
        read: &'a mut ReadOptions,
        cf: &RawColumnFamilyHandle,
    ) -> RawIterator<'a> {
        RawIterator::with_cf(self, read, cf)
    }

    pub fn snapshot(&self) -> RawSnapshot {
        unsafe {
            let ptr = tirocks_sys::crocksdb_create_snapshot(self.as_ptr());
            RawSnapshot::from_ptr(ptr)
        }
    }

    pub fn release_snapshot(&self, snap: RawSnapshot) {
        unsafe { tirocks_sys::crocksdb_release_snapshot(self.as_ptr(), snap.get()) }
    }
}

unsafe impl Iterable for RawDb {
    fn raw_iter(&self, opt: &mut ReadOptions) -> *mut rocksdb_Iterator {
        unsafe { tirocks_sys::crocksdb_create_iterator(self.as_ptr(), opt.get() as _) }
    }

    fn raw_iter_cf(
        &self,
        opt: &mut ReadOptions,
        cf: &RawColumnFamilyHandle,
    ) -> *mut rocksdb_Iterator {
        unsafe {
            tirocks_sys::crocksdb_create_iterator_cf(self.as_ptr(), opt.get() as _, cf.as_mut_ptr())
        }
    }
}

#[derive(Debug)]
pub struct Db {
    ptr: *mut rocksdb_DB,
    env: Option<Arc<Env>>,
    comparator: Vec<Arc<SysComparator>>,
    handles: Vec<NonNull<RawColumnFamilyHandle>>,
    is_titan: bool,
}

impl Drop for Db {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            for h in &mut self.handles {
                if h.as_ref().is_default_cf() {
                    continue;
                }
                // TODO: may should log.
                let _ = h.as_mut().destroy(self.ptr);
            }
            tirocks_sys::crocksdb_destroy(self.ptr as _);
        }
    }
}

impl Db {
    pub(crate) fn new(
        ptr: *mut rocksdb_DB,
        env: Option<Arc<Env>>,
        comparator: Vec<Arc<SysComparator>>,
        handles: Vec<NonNull<RawColumnFamilyHandle>>,
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

    pub fn close(self) -> Status {
        let mut s = Status::default();
        unsafe {
            tirocks_sys::crocksdb_close(self.ptr, s.as_mut_ptr());
        }
        s
    }

    pub fn list_column_families(db: DbOptions, path: impl AsRef<Path>) -> Result<Vec<String>> {
        unsafe fn cleanup(ptr: *mut *mut i8, off: usize, total: usize) {
            for i in off..total {
                libc::free(*ptr.add(i) as *mut c_void);
            }
            libc::free(ptr as *mut c_void);
        }
        let mut len: usize = 0;
        let mut s = Status::default();
        let names_ptr = unsafe {
            tirocks_sys::crocksdb_list_column_families(
                db.get(),
                path.as_ref().path_to_slice(),
                &mut len,
                s.as_mut_ptr(),
            )
        };

        check_status!(s)?;

        let mut names = Vec::with_capacity(len);
        unsafe {
            for i in 0..len {
                let n_ptr = *names_ptr.add(i);
                let name = CStr::from_ptr(n_ptr);
                match name.to_str() {
                    Ok(n) => names.push(n.to_string()),
                    Err(e) => {
                        cleanup(names_ptr, i, len);
                        return Err(Status::with_error(Code::kCorruption, format!("{}", e)));
                    }
                }
                libc::free(n_ptr as *mut c_void);
            }
            libc::free(names_ptr as *mut c_void);
        }
        Ok(names)
    }

    pub fn create_column_family(&mut self, name: impl AsRef<str>, opt: CfOptions) -> Result<()> {
        if self.is_titan {
            return self.create_column_family_titan(name, opt.into());
        }
        let mut s = Status::default();
        let ptr = unsafe {
            tirocks_sys::crocksdb_create_column_family(
                self.ptr,
                opt.as_ptr(),
                r(name.as_ref().as_bytes()),
                s.as_mut_ptr(),
            )
        };
        check_status!(s)?;
        opt.comparator().map(|c| self.comparator.push(c.clone()));
        self.handles
            .push(NonNull::new(ptr as *mut RawColumnFamilyHandle).unwrap());
        Ok(())
    }

    pub fn create_column_family_titan(
        &mut self,
        name: impl AsRef<str>,
        mut opt: TitanCfOptions,
    ) -> Result<()> {
        if !self.is_titan {
            return self.create_column_family(name, opt.into());
        }
        let mut s = Status::default();
        let ptr = unsafe {
            tirocks_sys::ctitandb_create_column_family(
                self.ptr,
                opt.as_mut_ptr(),
                r(name.as_ref().as_bytes()),
                s.as_mut_ptr(),
            )
        };
        check_status!(s)?;
        opt.comparator().map(|c| self.comparator.push(c.clone()));
        self.handles
            .push(NonNull::new(ptr as *mut RawColumnFamilyHandle).unwrap());
        Ok(())
    }

    pub fn drop_column_family(&mut self, name: &str) -> Result<()> {
        if name == DEFAULT_CF_NAME {
            return Err(Status::with_invalid_argument(
                "default cf can't be dropped.",
            ));
        }
        let pos = self
            .handles
            .iter()
            .position(|h| unsafe { h.as_ref().name().map_or(false, |n| n == name) });
        let pos = match pos {
            Some(p) => p,
            None => return Err(Status::with_code(Code::kNotFound)),
        };
        let mut h = self.handles.swap_remove(pos);
        unsafe {
            let destroy_res = h.as_mut().destroy(self.ptr);
            let drop_res = h.as_mut().drop(self);
            destroy_res.and(drop_res)
        }
    }

    pub fn cf(&self, name: &str) -> Option<&RawColumnFamilyHandle> {
        unsafe { self.cf_raw(name).map(|c| c.as_ref()) }
    }

    // TODO: what if it's dropped?
    pub(crate) unsafe fn cf_raw(&self, name: &str) -> Option<NonNull<RawColumnFamilyHandle>> {
        for h in &self.handles {
            if h.as_ref().name().map_or(false, |n| n == name) {
                return Some(*h);
            }
        }
        None
    }

    pub(crate) fn get(&self) -> *mut rocksdb_DB {
        self.ptr
    }
}

impl Deref for Db {
    type Target = RawDb;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.ptr as *mut RawDb) }
    }
}
