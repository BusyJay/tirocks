// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

mod inspected;
pub mod logger;
mod sequential_file;

use self::inspected::DBFileSystemInspector;
use crate::{Code, Result, Status};
use libc::c_char;
use tirocks_sys::{ffi_call, r};

pub use self::inspected::FileSystemInspector;
pub use self::sequential_file::SequentialFile;

pub type IoPriority = tirocks_sys::rocksdb_Env_IOPriority;
pub type Priority = tirocks_sys::rocksdb_Env_Priority;

/// Options while opening a file to read/write
// TODO: perhaps use the C struct directly
pub struct EnvOptions {
    ptr: *mut tirocks_sys::crocksdb_envoptions_t,
}

impl Default for EnvOptions {
    #[inline]
    fn default() -> EnvOptions {
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
    ptr: *mut tirocks_sys::rocksdb_Env,
    // Some env is based on other instance. And base should outlive `ptr`.
    _base: Option<Box<Env>>,
}

unsafe impl Send for Env {}

unsafe impl Sync for Env {}

impl Default for Env {
    /// Return a default environment suitable for the current operating
    /// system.  Sophisticated users may wish to provide their own Env
    /// implementation instead of relying on this default environment.
    #[inline]
    fn default() -> Env {
        unsafe { Env::new(None, tirocks_sys::crocksdb_default_env_create()) }
    }
}

impl Env {
    fn new(base: Option<Env>, ptr: *mut tirocks_sys::rocksdb_Env) -> Env {
        Env {
            ptr,
            _base: base.map(Box::new),
        }
    }

    /// Returns a new environment that stores its data in memory and delegates
    /// all non-file-storage tasks to base_env. base_env must remain live
    /// while the result is in use.
    #[inline]
    pub fn with_mem(base: Env) -> Env {
        unsafe {
            let mem = tirocks_sys::crocksdb_mem_env_create(base.ptr);
            Env::new(Some(base), mem)
        }
    }

    /// Create a ctr encrypted env with a given base env and a given ciper text.
    /// The length of ciper text must be 2^n, and must be less or equal to 2048.
    /// The recommanded block size are 1024, 512 and 256.
    pub fn with_ctr_encrypted(base_env: Env, ciphertext: &[u8]) -> Result<Env> {
        let len = ciphertext.len();
        if len > 2048 || !len.is_power_of_two() {
            return Err(Status::with_error(
                Code::kInvalidArgument,
                "ciphertext length must be less or equal to 2048, and must be power of 2",
            ));
        }
        let env = unsafe {
            tirocks_sys::crocksdb_ctr_encrypted_env_create(
                base_env.ptr,
                ciphertext.as_ptr() as *const c_char,
                len,
            )
        };
        Ok(Env::new(Some(base_env), env))
    }

    /// Create an encrypted env that accepts an external key manager.
    #[cfg(feature = "encryption")]
    pub fn with_key_managed_encrypted<T: EncryptionKeyManager>(
        base_env: Env,
        key_manager: T,
    ) -> Result<Env, String> {
        let db_key_manager = DBEncryptionKeyManager::new(key_manager);
        let env = unsafe {
            crocksdb_ffi::crocksdb_key_managed_encrypted_env_create(
                base_env.inner,
                db_key_manager.inner,
            )
        };
        Ok(Env::new(Some(base_env), env))
    }

    pub fn with_file_system_inspected<T: FileSystemInspector>(
        base_env: Env,
        file_system_inspector: T,
    ) -> Result<Env> {
        let db_file_system_inspector = DBFileSystemInspector::new(file_system_inspector);
        let env = unsafe {
            tirocks_sys::crocksdb_file_system_inspected_env_create(
                base_env.ptr,
                db_file_system_inspector.ptr,
            )
        };
        Ok(Env::new(Some(base_env), env))
    }

    /// Create a brand new sequentially-readable file with the specified name.
    /// On failure returns non-OK.  If the file does not exist, returns a non-OK status.
    #[inline]
    pub fn new_sequential_file(&self, path: &str, opts: EnvOptions) -> Result<SequentialFile> {
        unsafe {
            let file_path = r(path.as_bytes());
            let file = ffi_call!(crocksdb_sequential_file_create(
                self.ptr, file_path, opts.ptr
            ))?;
            Ok(SequentialFile::from_ptr(file))
        }
    }

    /// Returns OK if the named file exists.
    ///         NotFound if the named file does not exist,
    ///                  the calling process does not have permission to determine
    ///                  whether this file exists, or if the path is invalid.
    ///         IOError if an IO Error was encountered
    #[inline]
    pub fn file_exists(&self, path: &str) -> Result<()> {
        unsafe {
            let file_path = r(path.as_bytes());
            ffi_call!(crocksdb_env_file_exists(self.ptr, file_path))?;
            Ok(())
        }
    }

    /// Delete the named file.
    #[inline]
    pub fn delete_file(&self, path: &str) -> Result<()> {
        unsafe {
            let file_path = r(path.as_bytes());
            ffi_call!(crocksdb_env_delete_file(self.ptr, file_path))?;
            Ok(())
        }
    }

    /// The number of background worker threads of a specific thread pool
    /// for this environment. 'LOW' is the default pool.
    #[inline]
    pub fn set_background_threads(&self, pri: Priority, number: usize) {
        unsafe {
            tirocks_sys::crocksdb_env_set_background_threads(self.ptr, number as i32, pri);
        }
    }

    /// Wait for all threads to terminate.
    #[inline]
    pub fn wait_for_join(&self) {
        unsafe {
            tirocks_sys::crocksdb_env_join_all_threads(self.ptr);
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

#[cfg(test)]
mod tests;
