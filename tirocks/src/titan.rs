// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::{
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
};

use tirocks_sys::{ctitandb_blob_index_t, r};

use crate::{
    util::{self, check_status},
    Result, Status,
};

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
pub struct TitanBlobIndex(ctitandb_blob_index_t);

impl TitanBlobIndex {
    #[inline]
    pub fn new(file_number: u64, blob_size: u64, blob_offset: u64) -> Self {
        Self(ctitandb_blob_index_t {
            file_number,
            blob_offset,
            blob_size,
        })
    }

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
        let mut content = vec![];
        unsafe {
            let mut handle = |res: &[u8]| content.extend_from_slice(res);
            let (ctx, fp) = util::wrap_string_receiver(&mut handle);
            tirocks_sys::ctitandb_encode_blob_index(self as *const _ as _, ctx, Some(fp));
            content
        }
    }
}

impl Deref for TitanBlobIndex {
    type Target = ctitandb_blob_index_t;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TitanBlobIndex {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
