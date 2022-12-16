// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use libc::c_void;
use std::mem::MaybeUninit;
use std::ops::Deref;
use std::path::Path;
use std::str;
use std::sync::{Arc, Mutex};
use tirocks_sys::{r, rocksdb_DB, SimplePostWriteCallback};

use crate::metadata::{CfMetaData, SizeApproximationOptions};
use crate::option::{
    CfOptions, DbOptions, OwnedRawDbOptions, OwnedRawTitanDbOptions, RawCfOptions, RawDbOptions,
    RawOptions, RawTitanOptions, ReadOptions, TitanCfOptions, WriteOptions,
};
use crate::properties::table::user::SequenceNumber;
use crate::util::{self, ffi_call, range_to_rocks, split_pairs, PathToSlice, RustRange};
use crate::{comparator::SysComparator, env::Env};
use crate::{Code, PinSlice, RawIterator, Result, Status, WriteBatch};

use crate::db::cf::RawCfHandle;

use super::cf::{RefCountedCfHandle, DEFAULT_CF_NAME};

/// A helper trait that makes it possible to safely access `RawDb` reference.
pub trait RawDbRef {
    fn with<T>(&self, f: impl FnOnce(&RawDb) -> T) -> T;
}

impl<'a> RawDbRef for &'a RawDb {
    #[inline]
    fn with<T>(&self, f: impl FnOnce(&RawDb) -> T) -> T {
        f(self)
    }
}

/// A helper trait that makes it possible to safely access `Db` reference.
pub trait DbRef {
    fn with<T>(&self, f: impl FnOnce(&Db) -> T) -> T;
}

impl<'a> DbRef for &'a Db {
    #[inline]
    fn with<T>(&self, f: impl FnOnce(&Db) -> T) -> T {
        f(self)
    }
}
impl DbRef for Db {
    #[inline]
    fn with<T>(&self, f: impl FnOnce(&Db) -> T) -> T {
        f(self)
    }
}
impl DbRef for Arc<Db> {
    #[inline]
    fn with<T>(&self, f: impl FnOnce(&Db) -> T) -> T {
        f(self)
    }
}
impl DbRef for Arc<Mutex<Db>> {
    fn with<T>(&self, f: impl FnOnce(&Db) -> T) -> T {
        f(&self.lock().unwrap())
    }
}

impl<R> RawDbRef for R
where
    R: DbRef,
{
    #[inline]
    fn with<T>(&self, f: impl FnOnce(&RawDb) -> T) -> T {
        DbRef::with(self, |db| unsafe { f(RawDb::from_ptr(db.get_ptr())) })
    }
}

struct PostWriteCallback<'a, F: FnMut()> {
    raw: SimplePostWriteCallback,
    _callback: &'a mut F,
}

extern "C" fn on_post_write_callback<F: FnMut()>(ctx: *mut c_void) {
    unsafe {
        let ctx = &mut *(ctx as *mut F);
        ctx();
    }
}

impl<'a, F: FnMut()> PostWriteCallback<'a, F> {
    #[inline]
    fn new(callback: &'a mut F) -> Self {
        let mut raw = MaybeUninit::uninit();
        unsafe {
            tirocks_sys::crocksdb_simple_post_write_callback_init(
                raw.as_mut_ptr(),
                callback as *mut F as *mut c_void,
                Some(on_post_write_callback::<F>),
            );
            Self {
                raw: raw.assume_init(),
                _callback: callback,
            }
        }
    }

    #[inline]
    fn as_raw_callback(&mut self) -> *mut SimplePostWriteCallback {
        &mut self.raw as _
    }
}

/// A struct representing rocksdb::DB.
#[repr(transparent)]
pub struct RawDb(rocksdb_DB);

impl RawDb {
    #[inline]
    pub(crate) fn get_ptr(&self) -> *mut rocksdb_DB {
        self as *const RawDb as *mut rocksdb_DB
    }

    pub(crate) unsafe fn from_ptr<'a>(ptr: *const rocksdb_DB) -> &'a RawDb {
        &*(ptr as *const RawDb)
    }

    /// Set the database entry for "key" to "value". If "key" already exists, it will be
    /// overwritten. Returns OK on success, and a non-OK status on error.
    /// Note: consider setting options.sync = true.
    pub fn put(&self, opt: &WriteOptions, cf: &RawCfHandle, key: &[u8], val: &[u8]) -> Result<()> {
        unsafe {
            ffi_call!(crocksdb_put_cf(
                self.get_ptr(),
                opt.get_ptr(),
                cf.get_ptr(),
                r(key),
                r(val),
            ))
        }
    }

    /// Remove the database entry (if any) for "key".  Returns OK on success, and a non-OK status
    /// on error.  It is not an error if "key" did not exist in the database.
    /// Note: consider setting options.sync = true.
    pub fn delete(&self, opt: &WriteOptions, cf: &RawCfHandle, key: &[u8]) -> Result<()> {
        unsafe {
            ffi_call!(crocksdb_delete_cf(
                self.get_ptr(),
                opt.get_ptr(),
                cf.get_ptr(),
                r(key),
            ))
        }
    }

    /// Remove the database entry for "key". Requires that the key exists and was not overwritten.
    /// Returns OK on success, and a non-OK status on error.  It is not an error if "key" did not
    /// exist in the database.
    ///
    /// If a key is overwritten (by calling Put() multiple times), then the result of calling
    /// SingleDelete() on this key is undefined.  SingleDelete() only behaves correctly if there
    /// has been only one Put() for this key since the previous call to SingleDelete() for this
    /// key.
    ///
    /// This feature is currently an experimental performance optimization for a very specific
    /// workload.  It is up to the caller to ensure that SingleDelete is only used for a key that
    /// is not deleted using Delete() or written using Merge().  Mixing SingleDelete operations
    /// with Deletes and Merges can result in undefined behavior.
    ///
    /// Note: consider setting options.sync = true.
    pub fn single_delete(&self, opt: &WriteOptions, cf: &RawCfHandle, key: &[u8]) -> Result<()> {
        unsafe {
            ffi_call!(crocksdb_single_delete_cf(
                self.get_ptr(),
                opt.get_ptr(),
                cf.get_ptr(),
                r(key),
            ))
        }
    }

    /// Removes the database entries in the range ["begin_key", "end_key"), i.e., including
    /// "begin_key" and excluding "end_key". Returns OK on success, and a non-OK status on
    /// error. It is not an error if the database does not contain any existing data in the range
    /// ["begin_key", "end_key").
    ///
    /// If "end_key" comes before "start_key" according to the user's comparator, a
    /// `Status::InvalidArgument` is returned.
    ///
    /// This feature is now usable in production, with the following caveats:
    /// 1) Accumulating many range tombstones in the memtable will degrade read performance; this
    /// can be avoided by manually flushing occasionally.
    /// 2) Limiting the maximum number of open files in the presence of range tombstones can
    /// degrade read performance. To avoid this problem, set max_open_files to -1 whenever
    /// possible.
    pub fn delete_range(
        &self,
        opt: &WriteOptions,
        cf: &RawCfHandle,
        begin_key: &[u8],
        end_key: &[u8],
    ) -> Result<()> {
        unsafe {
            ffi_call!(crocksdb_delete_range_cf(
                self.get_ptr(),
                opt.get_ptr(),
                cf.get_ptr(),
                r(begin_key),
                r(end_key),
            ))
        }
    }

    /// Apply the specified updates to the database.
    /// If `updates` contains no update, WAL will still be synced if
    /// options.sync=true.
    /// Returns OK on success, non-OK on failure.
    /// Note: consider setting options.sync = true.
    #[inline]
    pub fn write(&self, opt: &WriteOptions, updates: &mut WriteBatch) -> Result<()> {
        unsafe {
            ffi_call!(crocksdb_write(
                self.get_ptr(),
                opt.get_ptr(),
                updates.as_mut_ptr(),
            ))
        }
    }

    #[inline]
    pub fn write_callback<F: FnMut()>(
        &self,
        opt: &WriteOptions,
        updates: &mut WriteBatch,
        mut callback: F,
    ) -> Result<()> {
        let mut callback = PostWriteCallback::new(&mut callback);
        unsafe {
            ffi_call!(crocksdb_write_callback(
                self.get_ptr(),
                opt.get_ptr(),
                updates.as_mut_ptr(),
                callback.as_raw_callback(),
            ))
        }
    }

    #[inline]
    pub fn write_multi(&self, opt: &WriteOptions, updates: &mut [&mut WriteBatch]) -> Result<()> {
        unsafe {
            ffi_call!(crocksdb_write_multi_batch(
                self.get_ptr(),
                opt.get_ptr(),
                // &mut T is the same as *mut T
                updates.as_mut_ptr() as _,
                updates.len(),
            ))
        }
    }

    /// If the database contains an entry for "key" return the corresponding value.
    ///
    /// If there is no entry for "key", returns `Ok(None)`.
    ///
    /// May return some other Status on an error.
    pub fn get(&self, opt: &ReadOptions, cf: &RawCfHandle, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let mut val = None;
        let res = unsafe {
            let mut f = |r: &[u8]| val = Some(r.to_vec());
            let (ctx, fp) = util::wrap_string_receiver(&mut f);
            ffi_call!(crocksdb_get_cf(
                self.get_ptr(),
                opt.get_ptr() as _,
                cf.get_ptr(),
                r(key),
                ctx,
                Some(fp),
            ))
        };
        match res {
            Ok(()) => Ok(val),
            Err(s) => {
                if s.code() == Code::kNotFound {
                    Ok(None)
                } else {
                    Err(s)
                }
            }
        }
    }

    /// Same as [`get`] but avoid allocations and memcpy for most cases.
    ///
    /// If such entry is found, value is updated and `Ok(true)` is returned.
    pub fn get_pinned(
        &self,
        opt: &ReadOptions,
        cf: &RawCfHandle,
        key: &[u8],
        value: &mut PinSlice,
    ) -> Result<bool> {
        let res = unsafe {
            ffi_call!(crocksdb_get_pinned_cf(
                self.get_ptr(),
                opt.get_ptr() as _,
                cf.get_ptr(),
                r(key),
                value.get_ptr(),
            ))
        };
        match res {
            Ok(()) => Ok(true),
            Err(s) => {
                if s.code() == Code::kNotFound {
                    Ok(false)
                } else {
                    Err(s)
                }
            }
        }
    }

    pub fn iter<'a>(&'a self, read: &'a mut ReadOptions, cf: &RawCfHandle) -> RawIterator<'a> {
        RawIterator::new(self, read, cf)
    }

    pub fn set_cf_options(
        &self,
        cf: &RawCfHandle,
        options: &[(impl AsRef<[u8]>, impl AsRef<[u8]>)],
    ) -> Result<()> {
        unsafe {
            let (key, val) = split_pairs(options);
            ffi_call!(crocksdb_set_options_cf(
                self.get_ptr(),
                cf.get_ptr(),
                key.as_ptr(),
                val.as_ptr(),
                options.len(),
            ))
        }
    }

    pub fn set_db_options(&self, options: &[(impl AsRef<[u8]>, impl AsRef<[u8]>)]) -> Result<()> {
        unsafe {
            let (key, val) = split_pairs(options);
            ffi_call!(crocksdb_set_db_options(
                self.get_ptr(),
                key.as_ptr(),
                val.as_ptr(),
                options.len(),
            ))
        }
    }

    /// Get Options that we use.  During the process of opening the
    /// column family, the options provided when calling DB::Open() or
    /// DB::CreateColumnFamily() will have been "sanitized" and transformed
    /// in an implementation-defined manner.
    #[inline]
    pub fn cf_options(&self, cf: &RawCfHandle) -> RawOptions {
        unsafe {
            let ptr = tirocks_sys::crocksdb_get_options_cf(self.get_ptr(), cf.get_ptr());
            RawOptions::from_ptr(ptr)
        }
    }

    #[inline]
    pub fn db_options(&self) -> OwnedRawDbOptions {
        unsafe {
            let ptr = tirocks_sys::crocksdb_get_db_options(self.get_ptr());
            OwnedRawDbOptions::from_ptr(ptr)
        }
    }

    /// The sequence number of the most recent transaction.
    #[inline]
    pub fn latest_sequence_number(&self) -> SequenceNumber {
        unsafe {
            let mut n = 0;
            tirocks_sys::crocksdb_get_latest_sequence_number(self.get_ptr(), &mut n);
            n
        }
    }

    /// Prevent file deletions. Compactions will continue to occur,
    /// but no obsolete files will be deleted. Calling this multiple
    /// times have the same effect as calling it once.
    #[inline]
    pub fn disable_file_deletions(&self) -> Result<()> {
        unsafe { ffi_call!(crocksdb_disable_file_deletions(self.get_ptr())) }
    }

    /// Allow compactions to delete obsolete files.
    /// If force == true, the call to [`enable_file_deletions`] will guarantee that
    /// file deletions are enabled after the call, even if [`disable_file_deletions`]
    /// was called multiple times before.
    /// If force == false, [`enable_file_deletions`] will only enable file deletion
    /// after it's been called at least as many times as [`disable_file_deletions`]
    /// enabling the two methods to be called by two threads concurrently without
    /// synchronization -- i.e., file deletions will be enabled only after both
    /// threads call [`enable_file_deletions`]
    #[inline]
    pub fn enable_file_deletions(&self, force: bool) -> Result<()> {
        unsafe { ffi_call!(crocksdb_enable_file_deletions(self.get_ptr(), force)) }
    }

    /// Delete the file name from the db directory and update the internal state to
    /// reflect that. Supports deletion of sst and log files only. 'name' must be
    /// path relative to the db directory. eg. 000001.sst, /archive/000003.log
    #[inline]
    pub fn delete_file(&self, name: impl AsRef<Path>) -> Result<()> {
        unsafe { ffi_call!(crocksdb_delete_file(self.get_ptr(), name.path_to_slice())) }
    }

    /// Obtains the meta data of the specified column family of the DB.
    ///
    /// Existing data will be cleared first.
    #[inline]
    pub fn cf_metadata(&self, cf: &RawCfHandle, data: &mut CfMetaData) {
        unsafe {
            tirocks_sys::crocksdb_get_column_family_meta_data(
                self.get_ptr(),
                cf.get_ptr(),
                data.as_mut_ptr(),
            );
        }
    }

    /// Return the approximate file system space used by keys in "[range[i].0 .. range[i].1)".
    ///
    /// Note that the returned sizes measure file system space usage, so if the user data
    /// compresses by a factor of ten, the returned sizes will be one-tenth the size of the
    /// corresponding user data size.
    pub fn approximate_sizes(
        &self,
        opt: &SizeApproximationOptions,
        cf: &RawCfHandle,
        ranges: &[(impl AsRef<[u8]>, impl AsRef<[u8]>)],
    ) -> Result<Vec<u64>> {
        let mut sizes = Vec::with_capacity(ranges.len());
        unsafe {
            let raw_ranges: Vec<_> = ranges.iter().map(|(s, e)| range_to_rocks(s, e)).collect();
            ffi_call!(crocksdb_approximate_sizes_cf(
                self.get_ptr(),
                opt.as_ptr(),
                cf.get_ptr(),
                raw_ranges.as_ptr(),
                raw_ranges.len() as i32,
                sizes.as_mut_ptr(),
            ))?;
            sizes.set_len(raw_ranges.len());
            Ok(sizes)
        }
    }

    /// The method is similar to [`approximate_sizes`], except it returns approximate number
    /// and size of records in memtables.
    pub fn approximate_mem_table_stats(
        &self,
        cf: &RawCfHandle,
        start_key: &[u8],
        end_key: &[u8],
    ) -> (u64, u64) {
        unsafe {
            let raw_range = range_to_rocks(&start_key, &end_key);
            let (mut count, mut size) = (0, 0);
            tirocks_sys::crocksdb_approximate_memtable_stats_cf(
                self.get_ptr(),
                cf.get_ptr(),
                &raw_range,
                &mut count,
                &mut size,
            );
            (count, size)
        }
    }

    /// Destroy the contents of the specified database. Be very careful using this method.
    #[inline]
    pub fn destroy(
        path: impl AsRef<Path>,
        options: &RawOptions,
        cfs: &[(impl AsRef<str>, impl AsRef<RawCfOptions>)],
    ) -> Result<()> {
        unsafe {
            let mut cf_names = Vec::with_capacity(cfs.len());
            let mut cf_opts = Vec::with_capacity(cfs.len());
            for (name, opt) in cfs {
                cf_names.push(r(name.as_ref().as_bytes()));
                cf_opts.push(opt.as_ref().as_ptr());
            }
            ffi_call!(crocksdb_destroy_db(
                path.path_to_slice(),
                options.as_ptr(),
                cf_names.as_ptr(),
                cf_opts.as_ptr(),
                cfs.len(),
            ))
        }
    }

    #[inline]
    pub fn repair(
        path: impl AsRef<Path>,
        options: &RawDbOptions,
        cfs: &[(impl AsRef<str>, impl AsRef<RawCfOptions>)],
    ) -> Result<()> {
        unsafe {
            let mut cf_names = Vec::with_capacity(cfs.len());
            let mut cf_opts = Vec::with_capacity(cfs.len());
            for (name, opt) in cfs {
                cf_names.push(r(name.as_ref().as_bytes()));
                cf_opts.push(opt.as_ref().as_ptr());
            }
            ffi_call!(crocksdb_repair_db(
                path.path_to_slice(),
                options.as_ptr(),
                cf_names.as_ptr(),
                cf_opts.as_ptr(),
                cfs.len(),
            ))
        }
    }
}

unsafe impl Sync for RawDb {}
unsafe impl Send for RawDb {}

/// The safe wrapper of `rocksdb::DB` that manages the required lifetime of resources.
#[derive(Debug)]
pub struct Db {
    ptr: *mut rocksdb_DB,
    _env: Option<Arc<Env>>,
    comparator: Vec<Arc<SysComparator>>,
    handles: Vec<RefCountedCfHandle>,
    is_titan: bool,
}

impl Drop for Db {
    #[inline]
    fn drop(&mut self) {
        let _ = self.clear_handles();
        unsafe {
            tirocks_sys::crocksdb_destroy(self.ptr as _);
        }
    }
}

impl Db {
    pub(crate) fn new(
        ptr: *mut rocksdb_DB,
        env: Option<Arc<Env>>,
        comparator: Vec<Arc<SysComparator>>,
        handles: Vec<RefCountedCfHandle>,
        is_titan: bool,
    ) -> Self {
        Self {
            ptr,
            _env: env,
            comparator,
            handles,
            is_titan,
        }
    }

    fn clear_handles(&mut self) -> Result<()> {
        for mut h in self.handles.drain(..) {
            unsafe {
                h.maybe_drop(self.ptr)?;
            }
        }
        Ok(())
    }

    /// Close the DB by releasing resources, closing files etc. This will not fsync the WAL files.
    /// If syncing is required, the caller must first call SyncWAL(), or Write() using an empty
    /// write batch with WriteOptions.sync=true.
    ///
    /// Unlike C++ API, if the close is aborted due to unreleased snapshot, the DB will be dropped
    /// any way as part of its resource has been freed already.
    pub fn close(mut self) -> Result<()> {
        unsafe {
            self.clear_handles()?;
            ffi_call!(crocksdb_close(self.ptr))
        }
    }

    /// ListColumnFamilies will open the DB specified by argument name and return the list of all
    /// column families in that DB. The ordering of returned column families is unspecified.
    pub fn list_cfs(db: DbOptions, path: impl AsRef<Path>) -> Result<Vec<String>> {
        let mut convert_s = Status::default();
        let mut names = Vec::new();
        unsafe {
            let mut handle = |name: &[u8]| match str::from_utf8(name) {
                Ok(n) => names.push(n.to_string()),
                Err(e) => {
                    convert_s = Status::with_error(Code::kCorruption, format!("{}", e));
                }
            };
            let (ctx, fp) = util::wrap_string_receiver(&mut handle);
            ffi_call!(crocksdb_list_column_families(
                db.get_ptr(),
                path.as_ref().path_to_slice(),
                ctx,
                Some(fp),
            ))?;
        }
        if !convert_s.ok() {
            return Err(convert_s);
        }
        Ok(names)
    }

    /// Create a column_family.
    ///
    /// If it's a titan db, it's the same as calling `create_cf_titan` with the titan specific
    /// options set to default.
    pub fn create_cf(&mut self, name: impl AsRef<str>, opt: CfOptions) -> Result<()> {
        if self.is_titan {
            return self.create_cf_titan(name, opt.into());
        }
        let ptr = unsafe {
            ffi_call!(crocksdb_create_column_family(
                self.ptr,
                opt.as_ptr(),
                r(name.as_ref().as_bytes()),
            ))
        }?;
        if let Some(c) = opt.comparator() {
            self.comparator.push(c.clone());
        }
        self.handles
            .push(unsafe { RefCountedCfHandle::from_ptr(ptr, true) });
        Ok(())
    }

    /// Create a column_family.
    ///
    /// If it's not titan, it's the same as calling `create_cf`. Titan specific options will be
    /// ignored.
    pub fn create_cf_titan(
        &mut self,
        name: impl AsRef<str>,
        mut opt: TitanCfOptions,
    ) -> Result<()> {
        if !self.is_titan {
            return self.create_cf(name, opt.into());
        }
        let ptr = unsafe {
            ffi_call!(ctitandb_create_column_family(
                self.ptr,
                opt.as_mut_ptr(),
                r(name.as_ref().as_bytes()),
            ))
        }?;
        if let Some(c) = opt.comparator() {
            self.comparator.push(c.clone());
        }
        self.handles
            .push(unsafe { RefCountedCfHandle::from_ptr(ptr, true) });
        Ok(())
    }

    /// Destroy a column family specified by name. This call records a drop record in the
    /// manifest and prevents the column family from flushing and compacting.
    ///
    /// Destroying the default cf is a no-op. After calling this method on other cf, no matter
    /// what value is returned, you will not be able to access the cf by `cf(name)` or similar
    /// method anymore. If a handle to the cf is holded when calling `destroy_cf`, you can still
    /// access the column family data via the handle. Such handle can be created by
    /// `CfHandle::new(db, name)`.
    pub fn destroy_cf(&mut self, name: &str) -> Result<bool> {
        if name == DEFAULT_CF_NAME {
            return Err(Status::with_invalid_argument(
                "default cf can't be dropped.",
            ));
        }
        let pos = self
            .handles
            .iter()
            .position(|h| h.name().map_or(false, |n| n == name));
        let pos = match pos {
            Some(p) => p,
            None => return Err(Status::with_code(Code::kNotFound)),
        };
        let mut h = self.handles.swap_remove(pos);
        unsafe {
            let destroy_res = h.destroy(self.ptr);
            let drop_res = h.maybe_drop(self.ptr);
            destroy_res.and(drop_res)
        }
    }

    pub fn cfs(&self) -> impl Iterator<Item = &RawCfHandle> {
        self.handles.iter().map(|c| &**c)
    }

    pub fn cf(&self, name: &str) -> Option<&RawCfHandle> {
        unsafe { self.cf_raw(name).map(|c| &**c) }
    }

    pub(crate) unsafe fn cf_raw(&self, name: &str) -> Option<&RefCountedCfHandle> {
        self.handles
            .iter()
            .find(|h| h.name().map_or(false, |n| n == name))
    }

    // TitanOptions doesn't inherit Options, so we can mix them.
    #[inline]
    pub fn cf_options_titan(&self, cf: &RawCfHandle) -> Option<RawTitanOptions> {
        unsafe {
            if self.is_titan {
                let ptr = tirocks_sys::ctitandb_get_titan_options_cf(self.get_ptr(), cf.get_ptr());
                Some(RawTitanOptions::from_ptr(ptr))
            } else {
                None
            }
        }
    }

    #[inline]
    pub fn db_options_titan(&self) -> Option<OwnedRawTitanDbOptions> {
        unsafe {
            if self.is_titan {
                let ptr = tirocks_sys::ctitandb_get_titan_db_options(self.get_ptr());
                Some(OwnedRawTitanDbOptions::from_ptr(ptr))
            } else {
                None
            }
        }
    }

    #[inline]
    pub fn is_titan(&self) -> bool {
        self.is_titan
    }

    #[inline]
    pub fn iter<'a>(&'a self, read: &'a mut ReadOptions, cf: &'a RawCfHandle) -> RawIterator<'a> {
        RawIterator::new(self, read, cf)
    }

    /// Delete files which are entirely in the given range
    /// Could leave some keys in the range which are in files which are not
    /// entirely in the range. Also leaves L0 files regardless of whether they're
    /// in the range.
    /// Snapshots before the delete might not see the data in the given range.
    pub fn delete_files_in_range(
        &self,
        cf: &RawCfHandle,
        begin: Option<&[u8]>,
        end: Option<&[u8]>,
        include_end: bool,
    ) -> Result<()> {
        self.delete_files_in_ranges(cf, &[(begin, end)], include_end)
    }

    /// Delete files in multiple ranges at once
    /// Delete files in a lot of ranges one at a time can be slow, use this API for
    /// better performance in that case.
    pub fn delete_files_in_ranges(
        &self,
        cf: &RawCfHandle,
        ranges: &[RustRange],
        include_end: bool,
    ) -> Result<()> {
        unsafe {
            let (_rocks_ranges, range_ptrs) = util::range_to_range_ptr(ranges);
            let mut s = Status::default();
            let f = if !self.is_titan() {
                tirocks_sys::crocksdb_delete_files_in_ranges_cf
            } else {
                tirocks_sys::ctitandb_delete_files_in_ranges_cf
            };
            f(
                self.get_ptr(),
                cf.get_ptr(),
                range_ptrs.as_ptr(),
                range_ptrs.len(),
                include_end,
                s.as_mut_ptr(),
            );
            if s.ok() {
                Ok(())
            } else {
                Err(s)
            }
        }
    }

    pub fn delete_blob_files_in_ranges(
        &self,
        cf: &RawCfHandle,
        ranges: &[RustRange],
        include_end: bool,
    ) -> Result<()> {
        if !self.is_titan() {
            return Ok(());
        }
        unsafe {
            let (_rocks_ranges, range_ptrs) = util::range_to_range_ptr(ranges);
            ffi_call!(ctitandb_delete_blob_files_in_ranges_cf(
                self.get_ptr(),
                cf.get_ptr(),
                range_ptrs.as_ptr(),
                range_ptrs.len(),
                include_end,
            ))
        }
    }
}

impl Deref for Db {
    type Target = RawDb;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.ptr as *mut RawDb) }
    }
}

unsafe impl Sync for Db {}
unsafe impl Send for Db {}
