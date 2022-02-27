// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use libc::{c_void, c_char};
use crate::Result;

/// Trait to inspect storage requests. FileSystemInspectedEnv will consult
/// FileSystemInspector before issuing actual disk IO.
pub trait FileSystemInspector {
    /// Request to read `len` bytes and return the allowed size.
    fn read(&self, len: usize) -> Result<usize>;
    /// Request to write `len` bytes and return the allowed size.
    fn write(&self, len: usize) -> Result<usize>;
}

extern "C" fn file_system_inspector_destructor<T: FileSystemInspector>(ctx: *mut c_void) {
    unsafe {
        // Recover from raw pointer and implicitly drop.
        Box::from_raw(ctx as *mut T);
    }
}

extern "C" fn file_system_inspector_read<T: FileSystemInspector>(
    ctx: *mut c_void,
    len: usize,
    errptr: *mut *mut c_char,
) -> usize {
    let file_system_inspector = unsafe { &*(ctx as *mut T) };
    match file_system_inspector.read(len) {
        Ok(ret) => ret,
        Err(e) => {
            unsafe { *errptr = e.to_crocksdb_error() };
            0
        }
    }
}

extern "C" fn file_system_inspector_write<T: FileSystemInspector>(
    ctx: *mut c_void,
    len: usize,
    errptr: *mut *mut c_char,
) -> usize {
    let file_system_inspector = unsafe { &*(ctx as *mut T) };
    match file_system_inspector.write(len) {
        Ok(ret) => ret,
        Err(e) => {
            unsafe { *errptr = e.to_crocksdb_error(); }
            0
        }
    }
}

pub struct DBFileSystemInspector {
    pub ptr: *mut tirocks_sys::crocksdb_file_system_inspector_t,
}

unsafe impl Send for DBFileSystemInspector {}
unsafe impl Sync for DBFileSystemInspector {}

impl DBFileSystemInspector {
    #[inline]
    pub fn new<T: FileSystemInspector>(file_system_inspector: T) -> DBFileSystemInspector {
        let ctx = Box::into_raw(Box::new(file_system_inspector)) as *mut c_void;
        let instance = unsafe {
            tirocks_sys::crocksdb_file_system_inspector_create(
                ctx,
                Some(file_system_inspector_destructor::<T>),
                Some(file_system_inspector_read::<T>),
                Some(file_system_inspector_write::<T>),
            )
        };
        DBFileSystemInspector { ptr: instance }
    }
}

impl Drop for DBFileSystemInspector {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            tirocks_sys::crocksdb_file_system_inspector_destroy(self.ptr);
        }
    }
}

#[cfg(test)]
impl FileSystemInspector for DBFileSystemInspector {
    #[inline]
    fn read(&self, len: usize) -> Result<usize> {
        let ret = unsafe { tirocks_sys::ffi_try!(crocksdb_file_system_inspector_read(self.ptr, len)) };
        Ok(ret)
    }

    #[inline]
    fn write(&self, len: usize) -> Result<usize> {
        let ret = unsafe { tirocks_sys::ffi_try!(crocksdb_file_system_inspector_write(self.ptr, len)) };
        Ok(ret)
    }
}
