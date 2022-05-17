// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

// For now the macro is only used by encryption.
#![cfg_attr(not(feature = "encryption"), allow(unused))]

use libc::c_void;
use tirocks_sys::{rocksdb_Slice, s};

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

// `FnOnce` is a more accurate type, but it will require Unpin.
unsafe extern "C" fn bytes_receiver<T: FnMut(&[u8])>(ptr: *mut c_void, buf: rocksdb_Slice) {
    (*(ptr as *mut T))(s(buf))
}

pub type BytesReceiver = unsafe extern "C" fn(*mut c_void, rocksdb_Slice);

pub fn wrap_string_receiver<T: FnMut(&[u8])>(receiver: &mut T) -> (*mut c_void, BytesReceiver) {
    (receiver as *mut T as *mut c_void, bytes_receiver::<T>)
}
