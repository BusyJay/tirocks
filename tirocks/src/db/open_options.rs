// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::{path::Path, ptr::NonNull};
use tirocks_sys::r;

use crate::{
    option::{CfOptions, DbOptions, Options, PathToSlice, TitanCfOptions, TitanDbOptions},
    Result, Status,
};

use super::{
    cf::{RawColumnFamilyHandle, RefCountedColumnFamilyHandle},
    db::Db,
};

#[derive(Default, Debug)]
pub struct DefaultCfOnlyBuilder {
    opt: Options,
    ttl: Option<i32>,
    error_if_log_exists: Option<bool>,
}

impl DefaultCfOnlyBuilder {
    pub fn set_option(&mut self, opt: Options) -> &mut Self {
        self.opt = opt;
        self
    }

    pub fn set_ttl(&mut self, ttl: i32) -> &mut Self {
        self.ttl = Some(ttl);
        self
    }

    pub fn set_read_only(&mut self, error_if_log_exists: bool) -> &mut Self {
        self.error_if_log_exists = Some(error_if_log_exists);
        self
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

#[derive(Debug)]
enum OptionSet {
    DefaultCfOnly(DefaultCfOnlyBuilder),
    MultiCf(MultiCfBuilder),
    MultiCfTitan(MultiCfTitanBuilder),
}

impl From<DefaultCfOnlyBuilder> for OpenOptions {
    fn from(builder: DefaultCfOnlyBuilder) -> Self {
        OpenOptions {
            opt_set: OptionSet::DefaultCfOnly(builder),
        }
    }
}

impl From<MultiCfBuilder> for OpenOptions {
    fn from(builder: MultiCfBuilder) -> Self {
        OpenOptions {
            opt_set: OptionSet::MultiCf(builder),
        }
    }
}

impl From<MultiCfTitanBuilder> for OpenOptions {
    fn from(builder: MultiCfTitanBuilder) -> Self {
        OpenOptions {
            opt_set: OptionSet::MultiCfTitan(builder),
        }
    }
}

pub struct OpenOptions {
    opt_set: OptionSet,
}

impl Default for OpenOptions {
    fn default() -> Self {
        OpenOptions {
            opt_set: OptionSet::DefaultCfOnly(DefaultCfOnlyBuilder::default()),
        }
    }
}

impl OpenOptions {
    pub fn open(&self, path: impl AsRef<Path>) -> Result<Db> {
        let mut s = Status::default();
        let mut comparator = vec![];
        let mut env = None;
        let mut handles = Vec::with_capacity(0);
        let mut is_titan = false;
        let ptr = unsafe {
            let p = path.as_ref().path_to_slice();
            match &self.opt_set {
                OptionSet::DefaultCfOnly(builder) => {
                    env = builder.opt.env().cloned();
                    builder.opt.comparator().map(|c| comparator.push(c.clone()));
                    if let Some(ttl) = builder.ttl {
                        tirocks_sys::crocksdb_open_with_ttl(
                            builder.opt.get(),
                            p,
                            ttl,
                            builder.error_if_log_exists.is_some(),
                            s.as_mut_ptr(),
                        )
                    } else if let Some(error_if_log_exists) = builder.error_if_log_exists {
                        tirocks_sys::crocksdb_open_for_read_only(
                            builder.opt.get(),
                            p,
                            error_if_log_exists,
                            s.as_mut_ptr(),
                        )
                    } else {
                        tirocks_sys::crocksdb_open(builder.opt.get(), p, s.as_mut_ptr())
                    }
                }
                OptionSet::MultiCf(builder) => {
                    env = builder.db.env().cloned();
                    let mut names = Vec::with_capacity(builder.cfs.len());
                    let mut opts = Vec::with_capacity(builder.cfs.len());
                    for (name, opt) in &builder.cfs {
                        names.push(r(name.as_bytes()));
                        opts.push(opt.as_ptr());
                        opt.comparator().map(|c| comparator.push(c.clone()));
                    }
                    handles = Vec::with_capacity(builder.cfs.len());
                    if !builder.ttl.is_empty() {
                        tirocks_sys::crocksdb_open_column_families_with_ttl(
                            builder.db.get(),
                            p,
                            builder.cfs.len() as i32,
                            names.as_ptr(),
                            opts.as_ptr(),
                            builder.ttl.as_ptr(),
                            builder.error_if_log_exists.is_some(),
                            handles.as_mut_ptr(),
                            s.as_mut_ptr(),
                        )
                    } else if let Some(error_if_log_file_exist) = builder.error_if_log_exists {
                        tirocks_sys::crocksdb_open_for_read_only_column_families(
                            builder.db.get(),
                            p,
                            builder.cfs.len() as i32,
                            names.as_ptr(),
                            opts.as_ptr(),
                            handles.as_mut_ptr(),
                            error_if_log_file_exist,
                            s.as_mut_ptr(),
                        )
                    } else {
                        tirocks_sys::crocksdb_open_column_families(
                            builder.db.get(),
                            p,
                            builder.cfs.len() as i32,
                            names.as_ptr(),
                            opts.as_ptr(),
                            handles.as_mut_ptr(),
                            s.as_mut_ptr(),
                        )
                    }
                }
                OptionSet::MultiCfTitan(builder) => {
                    env = builder.db.env().cloned();
                    let mut names = Vec::with_capacity(builder.cfs.len());
                    let mut opts = Vec::with_capacity(builder.cfs.len());
                    handles = Vec::with_capacity(builder.cfs.len());
                    for (name, opt) in &builder.cfs {
                        names.push(r(name.as_bytes()));
                        opts.push(opt.as_ptr());
                        opt.comparator().map(|c| comparator.push(c.clone()));
                    }
                    is_titan = true;
                    tirocks_sys::ctitandb_open_column_families(
                        p,
                        builder.db.get(),
                        builder.cfs.len() as i32,
                        names.as_ptr(),
                        opts.as_ptr(),
                        handles.as_mut_ptr(),
                        s.as_mut_ptr(),
                    )
                }
            }
        };
        if s.ok() {
            let handles = handles
                .into_iter()
                .map(|p| unsafe { RefCountedColumnFamilyHandle::from_ptr(p) })
                .collect();
            Ok(Db::new(ptr, env, comparator, handles, is_titan))
        } else {
            Err(s)
        }
    }
}
