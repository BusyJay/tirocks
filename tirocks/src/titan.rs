// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::{
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    ptr, slice,
};

use tirocks_sys::{ctitandb_blob_index_t, r};

use crate::{util::check_status, Result, Status};

/// Format of blob index (not fixed size):
///
///    +------+-------------+------------------------------------+
///    | type | file number |            blob handle             |
///    +------+-------------+------------------------------------+
///    | char |  Varint64   | Varint64(offsest) + Varint64(size) |
///    +------+-------------+------------------------------------+
///
/// It is stored in LSM-Tree as the value of key, then Titan can use this blob
/// index to locate actual value from blob file.
#[derive(Debug)]
#[repr(transparent)]
pub struct BlobIndex(ctitandb_blob_index_t);

impl BlobIndex {
    pub fn decode(value: &[u8]) -> Result<Self> {
        let mut blob = MaybeUninit::uninit();
        let mut s = Status::default();
        unsafe {
            tirocks_sys::ctitandb_decode_blob_index(
                r(value),
                blob.as_mut_ptr() as *mut ctitandb_blob_index_t,
                s.as_mut_ptr(),
            );
            check_status!(s)?;
            Ok(blob.assume_init())
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut value = ptr::null_mut();
        let mut value_size = 0;
        unsafe {
            tirocks_sys::ctitandb_encode_blob_index(
                &self as *const _ as _,
                &mut value,
                &mut value_size,
            );
            let slice = slice::from_raw_parts(value as *mut u8, value_size);
            let vec = slice.to_vec();
            libc::free(value as *mut libc::c_void);
            vec
        }
    }
}

impl Deref for BlobIndex {
    type Target = ctitandb_blob_index_t;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BlobIndex {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
