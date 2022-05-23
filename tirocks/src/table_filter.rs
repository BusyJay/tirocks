// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use libc::c_void;
use tirocks_sys::rocksdb_TableProperties;

use crate::properties::table::builtin::TableProperties;

pub trait TableFilter: Send + Sync {
    fn filter(&self, properties: &TableProperties) -> bool;
}

impl<F> TableFilter for F
where
    F: Fn(&TableProperties) -> bool + Send + Sync,
{
    fn filter(&self, properties: &TableProperties) -> bool {
        self(properties)
    }
}

pub(crate) extern "C" fn filter<T: TableFilter>(
    ctx: *mut c_void,
    props: *const rocksdb_TableProperties,
) -> bool {
    unsafe {
        let filter = &*(ctx as *mut T);
        filter.filter(TableProperties::from_ptr(props))
    }
}

pub(crate) extern "C" fn destroy<T: TableFilter>(filter: *mut c_void) {
    unsafe {
        Box::from_raw(filter as *mut T);
    }
}
