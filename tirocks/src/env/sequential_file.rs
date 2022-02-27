// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::{io::{Read, self}, ptr};

use crate::Result;
use tirocks_sys::crocksdb_sequential_file_t;

/// A file abstraction for reading sequentially through a file.
pub struct SequentialFile {
    ptr: *mut crocksdb_sequential_file_t,
}

impl SequentialFile {
    #[inline]
    pub(crate) fn from_ptr(ptr: *mut crocksdb_sequential_file_t) -> SequentialFile {
        SequentialFile {
            ptr,
        }
    }

    /// Skip "n" bytes from the file. This is guaranteed to be no
    /// slower that reading the same data, but may be faster.
    ///
    /// If end of file is reached, skipping will stop at the end of the
    /// file, and Skip will return OK.
    #[inline]
    pub fn skip(&mut self, n: u64) -> Result<()> {

    }
}

impl Read for SequentialFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        unsafe {
            let mut err = ptr::null_mut();
            let size = tirocks_sys::crocksdb_sequential_file_read(
                self.ptr,
                buf.len(),
                buf.as_mut_ptr(),
                &mut err,
            );
            if !err.is_null() {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    crocksdb_ffi::error_message(err),
                ));
            }
            Ok(size as usize)
        }
    }
    /// Read up to "n" bytes from the file.  "scratch[0..n-1]" may be
    /// written by this routine.  Sets "*result" to the data that was
    /// read (including if fewer than "n" bytes were successfully read).
    /// May set "*result" to point at data in "scratch[0..n-1]", so
    /// "scratch[0..n-1]" must be live when "*result" is used.
    /// If an error was encountered, returns a non-OK status.
    pub fn read(&mut self, n: usize, result: &mut [u8], char* scratch) = 0;
   
     // Indicates the upper layers if the current SequentialFile implementation
     // uses direct IO.
     virtual bool use_direct_io() const { return false; }
   
     // Use the returned alignment value to allocate
     // aligned buffer for Direct I/O
     virtual size_t GetRequiredBufferAlignment() const { return kDefaultPageSize; }
   
     // Remove any kind of caching of data from the offset to offset+length
     // of this file. If the length is 0, then it refers to the end of file.
     // If the system is not caching the file contents, then this is a noop.
     virtual Status InvalidateCache(size_t /*offset*/, size_t /*length*/) {
       return Status::NotSupported("InvalidateCache not supported.");
     }
   
     // Positioned Read for direct I/O
     // If Direct I/O enabled, offset, n, and scratch should be properly aligned
     virtual Status PositionedRead(uint64_t /*offset*/, size_t /*n*/,
                                   Slice* /*result*/, char* /*scratch*/) {
       return Status::NotSupported();
     }
   
     // If you're adding methods here, remember to add them to
     // SequentialFileWrapper too.
   };
