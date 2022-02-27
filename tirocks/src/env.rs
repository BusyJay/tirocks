// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

mod inspected;
mod sequential_file;

use std::sync::Arc;
use libc::c_char;
use tirocks_sys::{ffi_try, s};
use crate::rate_limiter::RateLimiter;
use self::inspected::DBFileSystemInspector;

pub use self::inspected::FileSystemInspector;

pub type IoPriority = tirocks_sys::rocksdb_Env_IOPriority;

/// Options while opening a file to read/write
pub struct EnvOptions {
    ptr: *mut tirocks_sys::crocksdb_envoptions_t,
}

impl EnvOptions {
    pub fn new() -> EnvOptions {
        unsafe {
            EnvOptions {
                ptr: tirocks_sys::crocksdb_envoptions_create(),
            }
        }
    }
}

impl Drop for EnvOptions {
    fn drop(&mut self) {
        unsafe {
            tirocks_sys::crocksdb_envoptions_destroy(self.ptr);
        }
    }
}

/// An Env is an interface used by the rocksdb implementation to access
/// operating system functionality like the filesystem etc.  Callers
/// may wish to provide a custom Env object when opening a database to
/// get fine gain control; e.g., to rate limit file system operations.
pub struct Env {
    ptr: *mut tirocks_sys::crocksdb_env_t,
    // Some env is based on other instance. And base should outlive self.
    // So wrap it into `Arc`.
    base: Option<Arc<Env>>,
}

unsafe impl Send for Env {}

unsafe impl Sync for Env {}

impl Default for Env {
    /// Return a default environment suitable for the current operating
    /// system.  Sophisticated users may wish to provide their own Env
    /// implementation instead of relying on this default environment.
    #[inline]
    fn default() -> Env {
        unsafe {
            Env {
                ptr: tirocks_sys::crocksdb_default_env_create(),
                base: None,
            }
        }
    }
}

impl Env {
    /// Returns a new environment that stores its data in memory and delegates
    /// all non-file-storage tasks to base_env. base_env must remain live
    /// while the result is in use.
    #[inline]
    pub fn mem() -> Env {
        unsafe {
            Env {
                ptr: tirocks_sys::crocksdb_mem_env_create(),
                base: None,
            }
        }
    }

    /// Create a ctr encrypted env with a given base env and a given ciper text.
    /// The length of ciper text must be 2^n, and must be less or equal to 2048.
    /// The recommanded block size are 1024, 512 and 256.
    pub fn ctr_encrypted(base_env: Arc<Env>, ciphertext: &[u8]) -> Result<Env, String> {
        let len = ciphertext.len();
        if len > 2048 || !len.is_power_of_two() {
            return Err(
                "ciphertext length must be less or equal to 2048, and must be power of 2"
                    .to_owned(),
            );
        }
        let env = unsafe {
            tirocks_sys::crocksdb_ctr_encrypted_env_create(
                base_env.ptr,
                ciphertext.as_ptr() as *const c_char,
                len,
            )
        };
        Ok(Env {
            ptr: env,
            base: Some(base_env),
        })
    }

    /// Create a ctr encrypted env with the default env
    #[inline]
    pub fn default_ctr_encrypted(ciphertext: &[u8]) -> Result<Env, String> {
        Env::ctr_encrypted(Arc::new(Env::default()), ciphertext)
    }

    /// Create an encrypted env that accepts an external key manager.
    #[cfg(feature = "encryption")]
    pub fn key_managed_encrypted<T: EncryptionKeyManager>(
        base_env: Arc<Env>,
        key_manager: T,
    ) -> Result<Env, String> {
        let db_key_manager = DBEncryptionKeyManager::new(key_manager);
        let env = unsafe {
            crocksdb_ffi::crocksdb_key_managed_encrypted_env_create(
                base_env.inner,
                db_key_manager.inner,
            )
        };
        Ok(Env {
            ptr: env,
            base: Some(base_env),
        })
    }

    pub fn file_system_inspected<T: FileSystemInspector>(
        base_env: Arc<Env>,
        file_system_inspector: T,
    ) -> Result<Env, String> {
        let db_file_system_inspector = DBFileSystemInspector::new(file_system_inspector);
        let env = unsafe {
            tirocks_sys::crocksdb_file_system_inspected_env_create(
                base_env.ptr,
                db_file_system_inspector.ptr,
            )
        };
        Ok(Env {
            ptr: env,
            base: Some(base_env),
        })
    }

    pub fn new_sequential_file(
        &self,
        path: &str,
        opts: EnvOptions,
    ) -> Result<SequentialFile, String> {
        unsafe {
            let file_path = s(path.as_bytes());
            let file = ffi_try!(crocksdb_sequential_file_create(
                self.ptr,
                file_path,
                opts.ptr,
            ));
            Ok(SequentialFile::new(file))
        }
    }

    pub fn file_exists(&self, path: &str) -> Result<(), String> {
        unsafe {
            let file_path = s(path.as_bytes());
            ffi_try!(crocksdb_env_file_exists(self.ptr, file_path));
            Ok(())
        }
    }

    pub fn delete_file(&self, path: &str) -> Result<(), String> {
        unsafe {
            let file_path = s(path.as_bytes());
            ffi_try!(crocksdb_env_delete_file(self.ptr, file_path));
            Ok(())
        }
    }
}

impl Drop for Env {
    fn drop(&mut self) {
        unsafe {
            tirocks_sys::crocksdb_env_destroy(self.ptr);
        }
    }
}
