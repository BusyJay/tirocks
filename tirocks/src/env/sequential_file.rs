// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::io::{self, Read};

use crate::Result;
use tirocks_sys::{crocksdb_sequential_file_t, ffi_try};

/// A file abstraction for reading sequentially through a file.
pub struct SequentialFile {
    ptr: *mut crocksdb_sequential_file_t,
}

impl SequentialFile {
    #[inline]
    pub(crate) fn from_ptr(ptr: *mut crocksdb_sequential_file_t) -> SequentialFile {
        SequentialFile { ptr }
    }

    /// Skip "n" bytes from the file. This is guaranteed to be no
    /// slower that reading the same data, but may be faster.
    ///
    /// If end of file is reached, skipping will stop at the end of the
    /// file, and Skip will return OK.
    #[inline]
    pub fn skip(&mut self, n: usize) -> Result<()> {
        unsafe {
            ffi_try!(crocksdb_sequential_file_skip(self.ptr, n));
            Ok(())
        }
    }

    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        unsafe {
            let size = ffi_try! {
                crocksdb_sequential_file_read(
                    self.ptr,
                    buf.len(),
                    buf.as_mut_ptr() as *mut libc::c_char
                )
            };
            Ok(size as usize)
        }
    }
}

impl Read for SequentialFile {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        Ok(SequentialFile::read(self, buf)?)
    }
}
