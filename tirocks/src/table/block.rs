// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use tirocks_sys::rocksdb_BlockBasedTableOptions;

pub struct BlockBasedTableOptions {
    ptr: *mut rocksdb_BlockBasedTableOptions,
}
