// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

pub struct Db {
    ptr: *mut tirocks_sys::crocksdb_t,
}

#[derive(Default)]
pub struct OpenOptions {}

impl OpenOptions {}
