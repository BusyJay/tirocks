// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use tirocks_sys::{r, rocksdb_Range, rocksdb_Slice};

macro_rules! utf8_name {
    ($slice:expr, $ctx:expr, $status:expr) => {
        match std::str::from_utf8(tirocks_sys::s($slice)) {
            Ok(n) => n,
            Err(e) => {
                std::ptr::write(
                    $status,
                    Status::with_invalid_argument(format!("{}: {}", $ctx, e)).into_raw(),
                );
                return;
            }
        }
    };
}

pub(crate) use utf8_name;

// TODO: simplify the definition
macro_rules! simple_access {
    ($(#[$outer:meta])* imp $prefix:ident <$op:ident> $field:ident / $rename:ident / :$ty:ty$([$($new_tt:tt)*])?) => {
        paste::paste! {
            $(#[$outer])*
            #[inline]
            pub fn [<$op _ $field>](&mut self, val: $ty) -> &mut Self {
                unsafe {
                    tirocks_sys::[<$prefix _ $op _ $rename>](self.as_mut_ptr(), val $($($new_tt)*)?)
                }
                self
            }
        }
    };
    ($(#[$outer:meta])* imp $prefix:ident <$op:ident> $field:ident:$ty:ty$([$($new_tt:tt)*])?) => {
        $crate::util::simple_access!($(#[$outer])* imp $prefix <$op> $field / $field /:$ty $([$($new_tt)*])? );
    };
    ($(#[$outer:meta])* imp $prefix:ident $field:ident / $rename:ident / :$ty:ty$([$($new_tt:tt)*])?) => {
        $crate::util::simple_access!($(#[$outer])* imp $prefix <set> $field / $rename /:$ty $([$($new_tt)*])? );
    };
    ($(#[$outer:meta])* imp $prefix:ident $field:ident:$ty:ty$([$($new_tt:tt)*])?) => {
        $crate::util::simple_access!($(#[$outer])* imp $prefix <set> $field / $field /:$ty $([$($new_tt)*])? );
    };
    ($prefix:ident $($(#[$outer:meta])*$(<$op:ident>)?$field:ident$(/$rename:ident/)?:$ty:ty$([$($new_tt:tt)*])?)+) => {
        $(
            $crate::util::simple_access!(
                $(#[$outer])*
                imp $prefix $(<$op>)? $field$(/$rename/)?:$ty$([$($new_tt)*])?
            );
        )+
    };
    (#[$outer:meta]$($tt:tt)+) => {
        $crate::util::simple_access! {
            crocksdb_options
            #[$outer]$($tt)+
        }
    };
}

pub(crate) use simple_access;

macro_rules! check_status {
    ($status:ident) => {
        if $status.ok() {
            Ok(())
        } else {
            Err($status)
        }
    };
}

pub(crate) use check_status;

pub unsafe fn split_pairs(
    pairs: &[(impl AsRef<[u8]>, impl AsRef<[u8]>)],
) -> (Vec<rocksdb_Slice>, Vec<rocksdb_Slice>) {
    let mut keys = Vec::with_capacity(pairs.len());
    let mut values = Vec::with_capacity(pairs.len());
    for (k, v) in pairs {
        keys.push(r(k.as_ref()));
        values.push(r(v.as_ref()));
    }
    (keys, values)
}

#[inline]
pub unsafe fn range_to_rocks(start: &impl AsRef<[u8]>, end: &impl AsRef<[u8]>) -> rocksdb_Range {
    rocksdb_Range {
        start: r(start.as_ref()),
        limit: r(end.as_ref()),
    }
}
