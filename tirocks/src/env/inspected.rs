// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use crate::{util::ffi_call, Result};
use libc::c_void;
use tirocks_sys::rocksdb_Status;

/// Trait to inspect storage requests. FileSystemInspectedEnv will consult
/// FileSystemInspector before issuing actual disk IO.
pub trait FileSystemInspector: Send + Sync {
    /// Request to read `len` bytes and return the allowed size.
    #[inline]
    fn read(&self, len: usize) -> Result<usize> {
        Ok(len)
    }

    /// Request to write `len` bytes and return the allowed size.
    #[inline]
    fn write(&self, len: usize) -> Result<usize> {
        Ok(len)
    }
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
    status: *mut rocksdb_Status,
) -> usize {
    let file_system_inspector = unsafe { &*(ctx as *mut T) };
    match file_system_inspector.read(len) {
        Ok(ret) => ret,
        Err(e) => {
            unsafe {
                std::ptr::write(status, e.into_raw());
            }
            0
        }
    }
}

extern "C" fn file_system_inspector_write<T: FileSystemInspector>(
    ctx: *mut c_void,
    len: usize,
    status: *mut rocksdb_Status,
) -> usize {
    let file_system_inspector = unsafe { &*(ctx as *mut T) };
    match file_system_inspector.write(len) {
        Ok(ret) => ret,
        Err(e) => {
            unsafe {
                std::ptr::write(status, e.into_raw());
            }
            0
        }
    }
}

pub(crate) struct SysFileSystemInspector {
    pub ptr: *mut tirocks_sys::crocksdb_file_system_inspector_t,
}

unsafe impl Send for SysFileSystemInspector {}
unsafe impl Sync for SysFileSystemInspector {}

impl SysFileSystemInspector {
    #[inline]
    pub fn new<T: FileSystemInspector>(file_system_inspector: T) -> SysFileSystemInspector {
        let ctx = Box::into_raw(Box::new(file_system_inspector)) as *mut c_void;
        let instance = unsafe {
            tirocks_sys::crocksdb_file_system_inspector_create(
                ctx,
                Some(file_system_inspector_destructor::<T>),
                Some(file_system_inspector_read::<T>),
                Some(file_system_inspector_write::<T>),
            )
        };
        SysFileSystemInspector { ptr: instance }
    }
}

impl Drop for SysFileSystemInspector {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            tirocks_sys::crocksdb_file_system_inspector_destroy(self.ptr);
        }
    }
}

#[cfg(test)]
impl FileSystemInspector for SysFileSystemInspector {
    #[inline]
    fn read(&self, len: usize) -> Result<usize> {
        unsafe { ffi_call!(crocksdb_file_system_inspector_read(self.ptr, len)) }
    }

    #[inline]
    fn write(&self, len: usize) -> Result<usize> {
        unsafe { ffi_call!(crocksdb_file_system_inspector_write(self.ptr, len)) }
    }
}
