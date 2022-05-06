// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

pub struct WriteBatch {
    ptr: *mut rocksdb_WriteBatch,
}
