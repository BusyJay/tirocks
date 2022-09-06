// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::ffi::CString;

use tirocks_sys::r;

use crate::{CfOptions, Options};

#[derive(Default)]
pub struct Ldb {
    opt: Options,
    cfs: Vec<(String, CfOptions)>,
}

impl Ldb {
    #[inline]
    pub fn new(opt: Options) -> Self {
        Self {
            opt,
            cfs: Vec::new(),
        }
    }

    #[inline]
    pub fn add_cf(&mut self, name: impl Into<String>, cf: CfOptions) -> &mut Self {
        self.cfs.push((name.into(), cf));
        self
    }

    #[inline]
    pub fn options_mut(&mut self) -> &mut Options {
        &mut self.opt
    }

    #[inline]
    pub fn run(&self, args: &[impl AsRef<str>]) {
        unsafe {
            let c_args: Vec<_> = args
                .iter()
                .map(|a| CString::new(a.as_ref()).unwrap())
                .collect();
            let c_arg_ptrs: Vec<_> = c_args.iter().map(|a| a.as_ptr()).collect();
            let names: Vec<_> = self
                .cfs
                .iter()
                .map(|(name, _)| r(name.as_bytes()))
                .collect();
            let opts: Vec<_> = self.cfs.iter().map(|(_, opt)| opt.as_ptr()).collect();

            tirocks_sys::crocksdb_run_ldb_tool(
                c_arg_ptrs.len(),
                c_arg_ptrs.as_ptr(),
                self.opt.as_ptr(),
                self.cfs.len(),
                names.as_ptr(),
                opts.as_ptr(),
            );
        }
    }
}

#[derive(Default)]
pub struct SstDump {
    opt: Options,
}

impl SstDump {
    #[inline]
    pub fn new(opt: Options) -> Self {
        Self { opt }
    }

    #[inline]
    pub fn options_mut(&mut self) -> &mut Options {
        &mut self.opt
    }

    #[inline]
    pub fn run(&self, args: &[impl AsRef<str>]) {
        unsafe {
            let c_args: Vec<_> = args
                .iter()
                .map(|a| CString::new(a.as_ref()).unwrap())
                .collect();
            let c_arg_ptrs: Vec<_> = c_args.iter().map(|a| a.as_ptr()).collect();

            tirocks_sys::crocksdb_run_sst_dump_tool(
                c_arg_ptrs.len(),
                c_arg_ptrs.as_ptr(),
                self.opt.as_ptr(),
            );
        }
    }
}
