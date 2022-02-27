// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use libc::c_char;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("tirocks::Other(`{0}`)")]
    Other(String),
}

impl Error {
    #[inline]
    pub(crate) fn to_crocksdb_error(&self) -> *mut c_char {
        match self {
            Error::Other(s) => {
                unsafe { libc::strdup(s.as_ptr() as _) }
            }
        }
    }
}

impl From<tirocks_sys::Error> for Error {
    #[inline]
    fn from(e: tirocks_sys::Error) -> Error {
        Error::Other(e.0)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
