// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

// For now the macro is only used by encryption.
#![cfg_attr(not(feature = "encryption"), allow(unused))]

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
