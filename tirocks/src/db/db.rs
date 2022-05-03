// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use libc::c_void;
use std::ffi::{CStr, CString};
use std::path::Path;
use std::ptr::NonNull;
use std::sync::{Arc, Mutex};
use tirocks_sys::{r, rocksdb_DB, rocksdb_ColumnFamilyHandle};

use crate::option::{CfOptions, DbOptions, PathToSlice, TitanCfOptions};
use crate::util::check_status;
use crate::{comparator::SysComparator, env::Env};
use crate::{Code, Result, Status};

use crate::db::cf::RawColumnFamilyHandle;

use super::cf::DEFAULT_CF_NAME;

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

#[repr(transparent)]
pub struct RawDb(rocksdb_DB);

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
            for h in self.handles {
                if h.as_ref().is_default_cf() {
                    continue;
                }
                // TODO: may should log.
                let _ = h.as_mut().destroy(self);
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
        self.handles.push(NonNull::new(ptr as *mut RawColumnFamilyHandle).unwrap());
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
        self.handles.push(NonNull::new(ptr as *mut RawColumnFamilyHandle).unwrap());
        Ok(())
    }

    pub fn drop_column_family(&mut self, name: &str) -> Result<()> {
        if name == DEFAULT_CF_NAME {
            return Err(Status:with_invalid_argument("default cf can't be dropped."));
        }
        let pos = self.handles.iter().position(|h| {
            unsafe { h.as_ref().name().map_or(false, |n| n == name) }
        });
        let pos = match pos {
            Some(p) => p,
            None => return Err(Status::with_code(Code::kNotFound)),
        };
        let h = self.handles.swap_remove(pos);
        unsafe {
            let destroy_res = h.as_mut().destroy(self);
            let drop_res = h.as_mut().drop(self);
            destroy_res.and(drop_res)
        }
    }

    pub fn get_cf(&self, name: &str) -> Option<&RawColumnFamilyHandle> {
        unsafe {
            self.get_cf_raw(name).map(|c| c.as_ref())
        }
    }

    pub(crate) unsafe fn get_cf_raw(&self, name: &str) -> Option<&NonNull<RawColumnFamilyHandle>> {
        for h in &self.handles {
            if h.as_ref().name().map_or(false, |n| n == name) {
                return Some(h);
            }
        }
        None
    }

    pub(crate) fn get(&self) -> *mut rocksdb_DB {
        self.ptr
    }
}