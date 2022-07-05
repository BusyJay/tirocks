// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::{path::Path, sync::Arc};
use tirocks_sys::{r, rocksdb_ColumnFamilyHandle, rocksdb_DB};

use crate::{
    comparator::SysComparator,
    env::Env,
    option::{
        CfOptions, DbOptions, Options, RawCfOptions, RawDbOptions, TitanCfOptions, TitanDbOptions,
    },
    util::{ffi_call, PathToSlice},
    Result, Status,
};

use super::{cf::RefCountedCfHandle, imp::Db};

#[derive(Default, Debug)]
pub struct DefaultCfOnlyBuilder {
    opt: Options,
    ttl: Option<i32>,
    error_if_log_exists: Option<bool>,
}

impl DefaultCfOnlyBuilder {
    #[inline]
    pub fn set_options(&mut self, opt: Options) -> &mut Self {
        self.opt = opt;
        self
    }

    #[inline]
    pub fn set_ttl(&mut self, ttl: i32) -> &mut Self {
        self.ttl = Some(ttl);
        self
    }

    #[inline]
    pub fn set_read_only(&mut self, enable: bool, error_if_log_exists: bool) -> &mut Self {
        self.error_if_log_exists = if enable {
            Some(error_if_log_exists)
        } else {
            None
        };
        self
    }

    #[inline]
    pub fn options_mut(&mut self) -> &mut Options {
        &mut self.opt
    }

    /// A shortcut for `options_mut().db_options_mut()`.
    #[inline]
    pub fn db_options_mut(&mut self) -> &mut RawDbOptions {
        self.opt.db_options_mut()
    }

    /// A shortcut for `options_mut().cf_options_mut()`.
    #[inline]
    pub fn cf_options_mut(&mut self) -> &mut RawCfOptions {
        self.opt.cf_options_mut()
    }

    /// A shortcut for the common used `options_mut().db_options_mut().set_create_if_missing()`.
    #[inline]
    pub fn set_create_if_missing(&mut self, create: bool) -> &mut Self {
        self.options_mut()
            .db_options_mut()
            .set_create_if_missing(create);
        self
    }

    #[inline]
    pub fn options(&self) -> &Options {
        &self.opt
    }
}

#[derive(Debug, Default)]
pub struct MultiCfBuilder {
    db: DbOptions,
    cfs: Vec<(String, CfOptions)>,
    ttl: Vec<i32>,
    error_if_log_exists: Option<bool>,
}

impl MultiCfBuilder {
    pub fn new(db: DbOptions) -> MultiCfBuilder {
        MultiCfBuilder {
            db,
            cfs: vec![],
            ttl: vec![],
            error_if_log_exists: None,
        }
    }

    pub fn add_cf(&mut self, name: impl Into<String>, cf: CfOptions) -> &mut Self {
        self.add_cf_internal(name, cf, None)
    }

    pub fn add_cf_ttl(&mut self, name: impl Into<String>, cf: CfOptions, ttl: i32) -> &mut Self {
        self.add_cf_internal(name, cf, Some(ttl))
    }

    fn add_cf_internal(
        &mut self,
        name: impl Into<String>,
        cf: CfOptions,
        ttl: Option<i32>,
    ) -> &mut Self {
        let name = name.into();
        if let Some(ttl) = ttl {
            if self.cfs.len() != self.ttl.len() {
                panic!("ttl should be specified for all cf.");
            }
            self.ttl.push(ttl);
        } else if !self.ttl.is_empty() {
            panic!("ttl is missing for {}", name);
        }
        self.cfs.push((name, cf));
        self
    }

    #[inline]
    pub fn db_options_mut(&mut self) -> &mut DbOptions {
        &mut self.db
    }

    /// A shortcut for the common used `db_options_mut().set_create_if_missing()`.
    #[inline]
    pub fn set_create_if_missing(&mut self, create: bool) -> &mut Self {
        self.db.set_create_if_missing(create);
        self
    }

    /// A shortcut for `db_options_mut().set_create_missing_column_families()`.
    #[inline]
    pub fn set_create_missing_column_families(&mut self, create: bool) -> &mut Self {
        self.db.set_create_missing_column_families(create);
        self
    }

    pub fn set_read_only(&mut self, enable: bool, error_if_log_exists: bool) -> &mut Self {
        self.error_if_log_exists = if enable {
            Some(error_if_log_exists)
        } else {
            None
        };
        self
    }
}

#[derive(Debug, Default)]
pub struct MultiCfTitanBuilder {
    db: TitanDbOptions,
    cfs: Vec<(String, TitanCfOptions)>,
}

impl MultiCfTitanBuilder {
    pub fn new(db: TitanDbOptions) -> MultiCfTitanBuilder {
        MultiCfTitanBuilder { db, cfs: vec![] }
    }

    pub fn add_cf(&mut self, name: impl Into<String>, cf: TitanCfOptions) -> &mut Self {
        self.cfs.push((name.into(), cf));
        self
    }

    #[inline]
    pub fn db_options_mut(&mut self) -> &mut TitanDbOptions {
        &mut self.db
    }

    /// A shortcut for the common used `db_options_mut().set_create_if_missing()`.
    #[inline]
    pub fn set_create_if_missing(&mut self, create: bool) -> &mut Self {
        self.db.set_create_if_missing(create);
        self
    }

    /// A shortcut for `db_options_mut().set_create_missing_column_families()`.
    #[inline]
    pub fn set_create_missing_column_families(&mut self, create: bool) -> &mut Self {
        self.db.set_create_missing_column_families(create);
        self
    }
}

pub trait OpenOptions {
    /// Open the db and check if it's titan.
    ///
    /// # Safety
    ///
    /// Opened cf will be pushed to `handles`. And they must be used while db pointer
    /// is still alive.
    unsafe fn open_raw(
        &self,
        comparator: &mut Vec<Arc<SysComparator>>,
        env: &mut Option<Arc<Env>>,
        handles: &mut Vec<*mut rocksdb_ColumnFamilyHandle>,
        path: &Path,
    ) -> Result<(*mut rocksdb_DB, bool)>;

    fn open(&self, path: &Path) -> Result<Db> {
        let mut comparator = vec![];
        let mut env = None;
        let mut handles = Vec::with_capacity(0);
        let (ptr, is_titan) =
            unsafe { self.open_raw(&mut comparator, &mut env, &mut handles, path) }?;
        let handles = if handles.is_empty() {
            vec![unsafe {
                let ptr = tirocks_sys::crocksdb_get_default_column_family(ptr);
                RefCountedCfHandle::from_ptr(ptr, false)
            }]
        } else {
            handles
                .into_iter()
                .map(|p| unsafe { RefCountedCfHandle::from_ptr(p, true) })
                .collect()
        };
        Ok(Db::new(ptr, env, comparator, handles, is_titan))
    }
}

impl OpenOptions for DefaultCfOnlyBuilder {
    unsafe fn open_raw(
        &self,
        comparator: &mut Vec<Arc<SysComparator>>,
        env: &mut Option<Arc<Env>>,
        _handles: &mut Vec<*mut rocksdb_ColumnFamilyHandle>,
        path: &Path,
    ) -> Result<(*mut rocksdb_DB, bool)> {
        *env = self.opt.env().cloned();
        let mut s = Status::default();
        let p = path.path_to_slice();
        if let Some(c) = self.opt.comparator() {
            comparator.push(c.clone());
        }
        let ptr = if let Some(ttl) = self.ttl {
            ffi_call!(crocksdb_open_with_ttl(
                self.opt.get_ptr(),
                p,
                ttl,
                self.error_if_log_exists.is_some(),
            ))
        } else if let Some(error_if_log_exists) = self.error_if_log_exists {
            ffi_call!(crocksdb_open_for_read_only(
                self.opt.get_ptr(),
                p,
                error_if_log_exists,
            ))
        } else {
            ffi_call!(crocksdb_open(self.opt.get_ptr(), p))
        }?;
        Ok((ptr, false))
    }
}

impl OpenOptions for MultiCfBuilder {
    unsafe fn open_raw(
        &self,
        comparator: &mut Vec<Arc<SysComparator>>,
        env: &mut Option<Arc<Env>>,
        handles: &mut Vec<*mut rocksdb_ColumnFamilyHandle>,
        path: &Path,
    ) -> Result<(*mut rocksdb_DB, bool)> {
        *env = self.db.env().cloned();
        let mut names = Vec::with_capacity(self.cfs.len());
        let mut opts = Vec::with_capacity(self.cfs.len());
        for (name, opt) in &self.cfs {
            names.push(r(name.as_bytes()));
            opts.push(opt.as_ptr());
            if let Some(c) = opt.comparator() {
                comparator.push(c.clone());
            }
        }
        handles.reserve_exact(self.cfs.len());
        let p = path.path_to_slice();
        let ptr = if !self.ttl.is_empty() {
            ffi_call!(crocksdb_open_column_families_with_ttl(
                self.db.get_ptr(),
                p,
                self.cfs.len() as i32,
                names.as_ptr(),
                opts.as_ptr(),
                self.ttl.as_ptr(),
                self.error_if_log_exists.is_some(),
                handles.as_mut_ptr(),
            ))
        } else if let Some(error_if_log_file_exist) = self.error_if_log_exists {
            ffi_call!(crocksdb_open_for_read_only_column_families(
                self.db.get_ptr(),
                p,
                self.cfs.len() as i32,
                names.as_ptr(),
                opts.as_ptr(),
                handles.as_mut_ptr(),
                error_if_log_file_exist,
            ))
        } else {
            ffi_call!(crocksdb_open_column_families(
                self.db.get_ptr(),
                p,
                self.cfs.len() as i32,
                names.as_ptr(),
                opts.as_ptr(),
                handles.as_mut_ptr(),
            ))
        }?;
        handles.set_len(self.cfs.len());
        Ok((ptr, false))
    }
}

impl OpenOptions for MultiCfTitanBuilder {
    unsafe fn open_raw(
        &self,
        comparator: &mut Vec<Arc<SysComparator>>,
        env: &mut Option<Arc<Env>>,
        handles: &mut Vec<*mut rocksdb_ColumnFamilyHandle>,
        path: &Path,
    ) -> Result<(*mut rocksdb_DB, bool)> {
        *env = self.db.env().cloned();
        let mut names = Vec::with_capacity(self.cfs.len());
        let mut opts = Vec::with_capacity(self.cfs.len());
        handles.reserve_exact(self.cfs.len());
        for (name, opt) in &self.cfs {
            names.push(r(name.as_bytes()));
            opts.push(opt.as_ptr());
            if let Some(c) = opt.comparator() {
                comparator.push(c.clone());
            }
        }
        let p = path.path_to_slice();
        let ptr = ffi_call!(ctitandb_open_column_families(
            p,
            self.db.get_ptr(),
            self.cfs.len() as i32,
            names.as_ptr(),
            opts.as_ptr(),
            handles.as_mut_ptr(),
        ))?;
        handles.set_len(self.cfs.len());
        Ok((ptr, true))
    }
}
