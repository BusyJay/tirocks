// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::{path::Path, sync::Arc};
use tirocks_sys::{r, rocksdb_ColumnFamilyHandle, rocksdb_DB};

use crate::{
    comparator::SysComparator,
    env::Env,
    option::{CfOptions, DbOptions, Options, PathToSlice, TitanCfOptions, TitanDbOptions},
    util::check_status,
    Result, Status,
};

use super::{cf::RefCountedColumnFamilyHandle, db::Db};

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
    pub fn set_read_only(&mut self, error_if_log_exists: bool) -> &mut Self {
        self.error_if_log_exists = Some(error_if_log_exists);
        self
    }

    #[inline]
    pub fn options_mut(&mut self) -> &mut Options {
        &mut self.opt
    }

    #[inline]
    pub fn options(&self) -> &Options {
        &self.opt
    }
}

#[derive(Debug)]
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
        } else {
            if !self.ttl.is_empty() {
                panic!("ttl is missing for {}", name);
            }
        }
        self.cfs.push((name, cf));
        self
    }

    pub fn set_read_only(&mut self, error_if_log_exists: bool) -> &mut Self {
        self.error_if_log_exists = Some(error_if_log_exists);
        self
    }
}

#[derive(Debug)]
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
}

pub trait OpenOptions {
    // Open the db and check if it's titan.
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
        let handles = handles
            .into_iter()
            .map(|p| unsafe { RefCountedColumnFamilyHandle::from_ptr(p) })
            .collect();
        Ok(Db::new(ptr, env, comparator, handles, is_titan))
    }
}

impl OpenOptions for DefaultCfOnlyBuilder {
    unsafe fn open_raw(
        &self,
        comparator: &mut Vec<Arc<SysComparator>>,
        env: &mut Option<Arc<Env>>,
        handles: &mut Vec<*mut rocksdb_ColumnFamilyHandle>,
        path: &Path,
    ) -> Result<(*mut rocksdb_DB, bool)> {
        *env = self.opt.env().cloned();
        let mut s = Status::default();
        let p = path.path_to_slice();
        self.opt.comparator().map(|c| comparator.push(c.clone()));
        let ptr = if let Some(ttl) = self.ttl {
            tirocks_sys::crocksdb_open_with_ttl(
                self.opt.get(),
                p,
                ttl,
                self.error_if_log_exists.is_some(),
                s.as_mut_ptr(),
            )
        } else if let Some(error_if_log_exists) = self.error_if_log_exists {
            tirocks_sys::crocksdb_open_for_read_only(
                self.opt.get(),
                p,
                error_if_log_exists,
                s.as_mut_ptr(),
            )
        } else {
            tirocks_sys::crocksdb_open(self.opt.get(), p, s.as_mut_ptr())
        };
        check_status!(s)?;
        handles.push(tirocks_sys::crocksdb_get_default_column_family(ptr));
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
            opt.comparator().map(|c| comparator.push(c.clone()));
        }
        handles.reserve_exact(self.cfs.len());
        let mut s = Status::default();
        let p = path.path_to_slice();
        let ptr = if !self.ttl.is_empty() {
            tirocks_sys::crocksdb_open_column_families_with_ttl(
                self.db.get(),
                p,
                self.cfs.len() as i32,
                names.as_ptr(),
                opts.as_ptr(),
                self.ttl.as_ptr(),
                self.error_if_log_exists.is_some(),
                handles.as_mut_ptr(),
                s.as_mut_ptr(),
            )
        } else if let Some(error_if_log_file_exist) = self.error_if_log_exists {
            tirocks_sys::crocksdb_open_for_read_only_column_families(
                self.db.get(),
                p,
                self.cfs.len() as i32,
                names.as_ptr(),
                opts.as_ptr(),
                handles.as_mut_ptr(),
                error_if_log_file_exist,
                s.as_mut_ptr(),
            )
        } else {
            tirocks_sys::crocksdb_open_column_families(
                self.db.get(),
                p,
                self.cfs.len() as i32,
                names.as_ptr(),
                opts.as_ptr(),
                handles.as_mut_ptr(),
                s.as_mut_ptr(),
            )
        };
        check_status!(s)?;
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
            opt.comparator().map(|c| comparator.push(c.clone()));
        }
        let mut s = Status::default();
        let p = path.path_to_slice();
        let ptr = tirocks_sys::ctitandb_open_column_families(
            p,
            self.db.get(),
            self.cfs.len() as i32,
            names.as_ptr(),
            opts.as_ptr(),
            handles.as_mut_ptr(),
            s.as_mut_ptr(),
        );
        check_status!(s)?;
        Ok((ptr, true))
    }
}
