// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::sync::Arc;

pub struct DBOptions {
    inner: *mut tirocks_sys::crocksdb_options_t,
    env: Option<Arc<Env>>,
    titan_inner: *mut tirocks_sys::ctitandb_options_t,
}
