// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::fmt::{self, Debug, Formatter};
use std::{mem, ptr};

use crate::util::utf8_name;
use crate::{Code, Result, Status};
use libc::c_void;
use tirocks_sys::{
    crocksdb_file_encryption_info_t, rocksdb_Slice, rocksdb_Status,
    rocksdb_encryption_EncryptionMethod, rocksdb_encryption_KeyManager,
};

pub type EncryptionMethod = rocksdb_encryption_EncryptionMethod;

#[derive(Clone, PartialEq, Eq)]
pub struct FileEncryptionInfo {
    method: EncryptionMethod,
    key: Vec<u8>,
    iv: Vec<u8>,
}

impl FileEncryptionInfo {
    #[inline]
    pub fn new(method: EncryptionMethod, key: Vec<u8>, iv: Vec<u8>) -> FileEncryptionInfo {
        FileEncryptionInfo { method, key, iv }
    }

    #[inline]
    pub fn method(&self) -> EncryptionMethod {
        self.method
    }

    #[inline]
    pub fn key(&self) -> &[u8] {
        &self.key
    }

    #[inline]
    pub fn iv(&self) -> &[u8] {
        &self.iv
    }

    unsafe fn copy_to(&self, file_info: *mut crocksdb_file_encryption_info_t) {
        let info = &mut *file_info;
        info.method = self.method;
        info.key = self.key.as_ptr() as _;
        info.key_len = self.key.len();
        info.iv = self.iv.as_ptr() as _;
        info.iv_len = self.iv.len();
    }
}

impl Debug for FileEncryptionInfo {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "FileEncryptionInfo [method={:?}, key=...<{} bytes>, iv=...<{} bytes>]",
            self.method,
            self.key.len(),
            self.iv.len()
        )
    }
}

/// Manage encryption keys for files. `Env` will query KeyManager for the
/// key being used for each file, and update KeyManager when it creates a
/// new file or moving files around.
pub trait KeyManager: Sync + Send {
    fn get_file(&self, file_name: &str) -> Result<FileEncryptionInfo>;
    fn new_file(&self, file_name: &str) -> Result<FileEncryptionInfo>;
    fn delete_file(&self, file_name: &str) -> Result<()>;
    fn link_file(&self, src_file_name: &str, dst_file_name: &str) -> Result<()>;
}

extern "C" fn key_manager_destructor<T: KeyManager>(ctx: *mut c_void) {
    unsafe {
        // Recover from raw pointer and implicitly drop.
        drop(Box::from_raw(ctx as *mut T));
    }
}

extern "C" fn key_manager_get_file<T: KeyManager>(
    ctx: *mut c_void,
    file_name: rocksdb_Slice,
    file_info: *mut crocksdb_file_encryption_info_t,
    status: *mut rocksdb_Status,
) {
    unsafe {
        let key_manager = &*(ctx as *mut T);
        let name = utf8_name!(file_name, "Encryption file name", status);

        match key_manager.get_file(name) {
            Ok(ret) => {
                ret.copy_to(file_info);
                *status = Status::with_code(Code::kOk).into_raw();
            }
            Err(err) => *status = err.into_raw(),
        }
    }
}

extern "C" fn key_manager_new_file<T: KeyManager>(
    ctx: *mut c_void,
    file_name: rocksdb_Slice,
    file_info: *mut crocksdb_file_encryption_info_t,
    status: *mut rocksdb_Status,
) {
    unsafe {
        let key_manager = &*(ctx as *mut T);
        let name = utf8_name!(file_name, "Encryption file name", status);

        match key_manager.new_file(name) {
            Ok(ret) => {
                ret.copy_to(file_info);
                *status = Status::with_code(Code::kOk).into_raw();
            }
            Err(err) => *status = err.into_raw(),
        }
    }
}

extern "C" fn key_manager_delete_file<T: KeyManager>(
    ctx: *mut c_void,
    file_name: rocksdb_Slice,
    status: *mut rocksdb_Status,
) {
    unsafe {
        let key_manager = &*(ctx as *mut T);
        let name = utf8_name!(file_name, "Encryption file name", status);
        match key_manager.delete_file(name) {
            Ok(()) => ptr::write(status, Status::with_code(Code::kOk).into_raw()),
            Err(err) => ptr::write(status, err.into_raw()),
        }
    }
}

extern "C" fn key_manager_link_file<T: KeyManager>(
    ctx: *mut c_void,
    src: rocksdb_Slice,
    dst: rocksdb_Slice,
    status: *mut rocksdb_Status,
) {
    unsafe {
        let key_manager = &*(ctx as *mut T);
        let src_name = utf8_name!(src, "Encryption file name", status);
        let dst_name = utf8_name!(dst, "Encryption file name", status);
        match key_manager.link_file(src_name, dst_name) {
            Ok(()) => ptr::write(status, Status::with_code(Code::kOk).into_raw()),
            Err(err) => ptr::write(status, err.into_raw()),
        }
    }
}

pub(crate) struct SysKeyManager {
    ptr: *mut rocksdb_encryption_KeyManager,
}

unsafe impl Send for SysKeyManager {}
unsafe impl Sync for SysKeyManager {}

impl SysKeyManager {
    pub fn new<T: KeyManager>(key_manager: T) -> SysKeyManager {
        let ctx = Box::into_raw(Box::new(key_manager)) as *mut c_void;
        let ptr = unsafe {
            tirocks_sys::crocksdb_encryption_key_manager_create(
                ctx,
                Some(key_manager_destructor::<T>),
                Some(key_manager_get_file::<T>),
                Some(key_manager_new_file::<T>),
                Some(key_manager_delete_file::<T>),
                Some(key_manager_link_file::<T>),
            )
        };
        SysKeyManager { ptr }
    }

    #[inline]
    pub(crate) fn into_raw(self) -> *mut rocksdb_encryption_KeyManager {
        let p = self.ptr;
        mem::forget(self);
        p
    }
}

impl Drop for SysKeyManager {
    fn drop(&mut self) {
        unsafe {
            tirocks_sys::crocksdb_encryption_key_manager_destroy(self.ptr);
        }
    }
}

#[cfg(test)]
mod test {
    use std::fs::File;
    use std::io::{Read, Write};
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::Arc;

    use crate::env::{Env, EnvOptions};

    use super::*;

    #[derive(Default)]
    struct AccessRecord {
        drop: AtomicBool,
        invalid_encrypted: AtomicBool,
        get_access: AtomicUsize,
        new_access: AtomicUsize,
        delete_access: AtomicUsize,
        link_access: AtomicUsize,
    }

    #[derive(Default)]
    struct TestKeyManager {
        record: Arc<AccessRecord>,
    }

    impl KeyManager for TestKeyManager {
        fn get_file(&self, _: &str) -> Result<FileEncryptionInfo> {
            self.record.get_access.fetch_add(1, Ordering::SeqCst);
            let method = if self.record.invalid_encrypted.load(Ordering::SeqCst) {
                EncryptionMethod::kUnknown
            } else {
                EncryptionMethod::kPlaintext
            };
            Ok(FileEncryptionInfo::new(method, vec![], vec![]))
        }

        fn new_file(&self, _: &str) -> Result<FileEncryptionInfo> {
            self.record.new_access.fetch_add(1, Ordering::SeqCst);
            let method = if self.record.invalid_encrypted.load(Ordering::SeqCst) {
                EncryptionMethod::kUnknown
            } else {
                EncryptionMethod::kPlaintext
            };
            Ok(FileEncryptionInfo::new(method, vec![], vec![]))
        }

        fn delete_file(&self, _: &str) -> Result<()> {
            self.record.delete_access.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        fn link_file(&self, _: &str, _: &str) -> Result<()> {
            self.record.link_access.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    impl Drop for TestKeyManager {
        fn drop(&mut self) {
            self.record.drop.store(true, Ordering::SeqCst);
        }
    }

    #[test]
    fn test_encrypted_manager() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path()).unwrap();
        let file_path = dir.path().join("test.txt");
        let file_path_str = file_path.to_str().unwrap();
        let data = [0; 16];
        File::create(&file_path).unwrap().write_all(&data).unwrap();

        let key_manager = TestKeyManager::default();
        let record = key_manager.record.clone();
        let env = Env::with_key_manager_encrypted(Env::default(), key_manager).unwrap();
        assert!(!record.drop.load(Ordering::SeqCst));
        assert_eq!(record.get_access.load(Ordering::SeqCst), 0);
        record.invalid_encrypted.store(true, Ordering::SeqCst);
        let status = env
            .new_sequential_file(file_path_str, EnvOptions::default())
            .unwrap_err();
        assert_eq!(status.code(), Code::kInvalidArgument);
        assert_eq!(record.get_access.load(Ordering::SeqCst), 1);

        record.invalid_encrypted.store(false, Ordering::SeqCst);
        let mut f = env
            .new_sequential_file(file_path_str, EnvOptions::default())
            .unwrap();
        let mut buf = [3; 16];
        f.read_exact(&mut buf).unwrap();
        assert_eq!(buf, data);
        assert_eq!(record.get_access.load(Ordering::SeqCst), 2);

        assert_eq!(record.delete_access.load(Ordering::SeqCst), 0);
        env.delete_file(file_path_str).unwrap();
        assert_eq!(record.delete_access.load(Ordering::SeqCst), 1);

        drop(env);
        assert!(record.drop.load(Ordering::SeqCst));
    }
}
