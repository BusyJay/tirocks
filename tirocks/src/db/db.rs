// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::ops::Deref;
use std::path::Path;
use std::str;
use std::sync::{Arc, Mutex};
use tirocks_sys::{r, rocksdb_DB};

use crate::metadata::{ColumnFamilyMetaData, SizeApproximationOptions};
use crate::option::{
    CfOptions, DbOptions, OwnedRawDbOptions, OwnedRawTitanDbOptions, PathToSlice, RawCfOptions,
    RawDbOptions, RawOptions, RawTitanOptions, ReadOptions, TitanCfOptions, WriteOptions,
};
use crate::properties::table::user::SequenceNumber;
use crate::util::{self, check_status, range_to_rocks, split_pairs};
use crate::write_batch::WriteBatch;
use crate::{comparator::SysComparator, env::Env};
use crate::{Code, PinSlice, RawIterator, Result, Status};

use crate::db::cf::RawColumnFamilyHandle;

use super::cf::{RefCountedColumnFamilyHandle, DEFAULT_CF_NAME};

pub trait RawDbRef {
    fn visit<T>(&self, f: impl FnOnce(&RawDb) -> T) -> T;
}

impl<'a> RawDbRef for &'a RawDb {
    #[inline]
    fn visit<T>(&self, f: impl FnOnce(&RawDb) -> T) -> T {
        f(self)
    }
}

pub trait DbRef {
    fn visit<T>(&self, f: impl FnOnce(&Db) -> T) -> T;
}

impl<'a> DbRef for &'a Db {
    #[inline]
    fn visit<T>(&self, f: impl FnOnce(&Db) -> T) -> T {
        f(self)
    }
}
impl DbRef for Db {
    #[inline]
    fn visit<T>(&self, f: impl FnOnce(&Db) -> T) -> T {
        f(self)
    }
}
impl DbRef for Arc<Db> {
    #[inline]
    fn visit<T>(&self, f: impl FnOnce(&Db) -> T) -> T {
        f(self)
    }
}
impl DbRef for Arc<Mutex<Db>> {
    fn visit<T>(&self, f: impl FnOnce(&Db) -> T) -> T {
        f(&self.lock().unwrap())
    }
}

impl<R> RawDbRef for R
where
    R: DbRef,
{
    #[inline]
    fn visit<T>(&self, f: impl FnOnce(&RawDb) -> T) -> T {
        DbRef::visit(self, |db| unsafe { f(RawDb::from_ptr(db.as_ptr())) })
    }
}

#[repr(transparent)]
pub struct RawDb(rocksdb_DB);

impl RawDb {
    #[inline]
    pub(crate) fn as_ptr(&self) -> *mut rocksdb_DB {
        self as *const RawDb as *mut rocksdb_DB
    }

    pub(crate) unsafe fn from_ptr<'a>(ptr: *const rocksdb_DB) -> &'a RawDb {
        &*(ptr as *const RawDb)
    }

    pub fn put(
        &self,
        opt: &WriteOptions,
        cf: &RawColumnFamilyHandle,
        key: &[u8],
        val: &[u8],
    ) -> Result<()> {
        let mut s = Status::default();
        unsafe {
            tirocks_sys::crocksdb_put_cf(
                self.as_ptr(),
                opt.get(),
                cf.get(),
                r(key),
                r(val),
                s.as_mut_ptr(),
            );
        }
        check_status!(s)
    }

    pub fn delete(&self, opt: &WriteOptions, cf: &RawColumnFamilyHandle, key: &[u8]) -> Result<()> {
        let mut s = Status::default();
        unsafe {
            tirocks_sys::crocksdb_delete_cf(
                self.as_ptr(),
                opt.get(),
                cf.get(),
                r(key),
                s.as_mut_ptr(),
            );
        }
        check_status!(s)
    }

    pub fn single_delete(
        &self,
        opt: &WriteOptions,
        cf: &RawColumnFamilyHandle,
        key: &[u8],
    ) -> Result<()> {
        let mut s = Status::default();
        unsafe {
            tirocks_sys::crocksdb_single_delete_cf(
                self.as_ptr(),
                opt.get(),
                cf.get(),
                r(key),
                s.as_mut_ptr(),
            );
        }
        check_status!(s)
    }

    pub fn delete_range(
        &self,
        opt: &WriteOptions,
        cf: &RawColumnFamilyHandle,
        begin_key: &[u8],
        end_key: &[u8],
    ) -> Result<()> {
        let mut s = Status::default();
        unsafe {
            tirocks_sys::crocksdb_delete_range_cf(
                self.as_ptr(),
                opt.get(),
                cf.get(),
                r(begin_key),
                r(end_key),
                s.as_mut_ptr(),
            );
        }
        check_status!(s)
    }

    /// Apply the specified updates to the database.
    /// If `updates` contains no update, WAL will still be synced if
    /// options.sync=true.
    /// Returns OK on success, non-OK on failure.
    /// Note: consider setting options.sync = true.
    #[inline]
    pub fn write(&self, opt: &WriteOptions, updates: &mut WriteBatch) -> Result<()> {
        let mut s = Status::default();
        unsafe {
            tirocks_sys::crocksdb_write(
                self.as_ptr(),
                opt.get(),
                updates.as_mut_ptr(),
                s.as_mut_ptr(),
            );
        }
        check_status!(s)
    }

    pub fn get(
        &self,
        opt: &ReadOptions,
        cf: &RawColumnFamilyHandle,
        key: &[u8],
    ) -> Result<Option<Vec<u8>>> {
        let mut s = Status::default();
        let mut res = None;
        unsafe {
            let mut f = |r: &[u8]| res = Some(r.to_vec());
            let (ctx, fp) = util::wrap_string_receiver(&mut f);
            tirocks_sys::crocksdb_get_cf(
                self.as_ptr(),
                opt.get() as _,
                cf.get(),
                r(key),
                ctx,
                Some(fp),
                s.as_mut_ptr(),
            )
        };
        if s.ok() {
            Ok(res)
        } else if s.code() == Code::kNotFound {
            Ok(None)
        } else {
            Err(s)
        }
    }

    pub fn get_to(
        &self,
        opt: &ReadOptions,
        cf: &RawColumnFamilyHandle,
        key: &[u8],
        value: &mut PinSlice,
    ) -> Result<bool> {
        let mut s = Status::default();
        unsafe {
            tirocks_sys::crocksdb_get_pinned_cf(
                self.as_ptr(),
                opt.get() as _,
                cf.get(),
                r(key),
                value.get(),
                s.as_mut_ptr(),
            )
        };
        if s.ok() {
            Ok(true)
        } else if s.code() == Code::kNotFound {
            Ok(false)
        } else {
            Err(s)
        }
    }

    pub fn iter<'a>(
        &'a self,
        read: &'a mut ReadOptions,
        cf: &RawColumnFamilyHandle,
    ) -> RawIterator<'a> {
        RawIterator::new(self, read, cf)
    }

    pub fn set_cf_options(
        &self,
        cf: &RawColumnFamilyHandle,
        options: &[(impl AsRef<[u8]>, impl AsRef<[u8]>)],
    ) -> Result<()> {
        unsafe {
            let (key, val) = split_pairs(options);
            let mut s = Status::default();
            tirocks_sys::crocksdb_set_options_cf(
                self.as_ptr(),
                cf.get(),
                key.as_ptr(),
                val.as_ptr(),
                options.len(),
                s.as_mut_ptr(),
            );
            check_status!(s)
        }
    }

    pub fn set_db_options(&self, options: &[(impl AsRef<[u8]>, impl AsRef<[u8]>)]) -> Result<()> {
        unsafe {
            let (key, val) = split_pairs(options);
            let mut s = Status::default();
            tirocks_sys::crocksdb_set_db_options(
                self.as_ptr(),
                key.as_ptr(),
                val.as_ptr(),
                options.len(),
                s.as_mut_ptr(),
            );
            check_status!(s)
        }
    }

    /// Get Options that we use.  During the process of opening the
    /// column family, the options provided when calling DB::Open() or
    /// DB::CreateColumnFamily() will have been "sanitized" and transformed
    /// in an implementation-defined manner.
    #[inline]
    pub fn cf_options(&self, cf: &RawColumnFamilyHandle) -> RawOptions {
        unsafe {
            let ptr = tirocks_sys::crocksdb_get_options_cf(self.as_ptr(), cf.get());
            RawOptions::from_ptr(ptr)
        }
    }

    #[inline]
    pub fn db_options(&self) -> OwnedRawDbOptions {
        unsafe {
            let ptr = tirocks_sys::crocksdb_get_db_options(self.as_ptr());
            OwnedRawDbOptions::from_ptr(ptr)
        }
    }

    /// The sequence number of the most recent transaction.
    #[inline]
    pub fn latest_sequence_number(&self) -> SequenceNumber {
        unsafe {
            let mut n = 0;
            tirocks_sys::crocksdb_get_latest_sequence_number(self.as_ptr(), &mut n);
            n
        }
    }

    /// Prevent file deletions. Compactions will continue to occur,
    /// but no obsolete files will be deleted. Calling this multiple
    /// times have the same effect as calling it once.
    #[inline]
    pub fn disable_file_deletions(&self) -> Result<()> {
        unsafe {
            let mut s = Status::default();
            tirocks_sys::crocksdb_disable_file_deletions(self.as_ptr(), s.as_mut_ptr());
            check_status!(s)
        }
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
        unsafe {
            let mut s = Status::default();
            tirocks_sys::crocksdb_enable_file_deletions(self.as_ptr(), force, s.as_mut_ptr());
            check_status!(s)
        }
    }

    /// Delete the file name from the db directory and update the internal state to
    /// reflect that. Supports deletion of sst and log files only. 'name' must be
    /// path relative to the db directory. eg. 000001.sst, /archive/000003.log
    #[inline]
    pub fn delete_file(&self, name: impl AsRef<Path>) -> Result<()> {
        unsafe {
            let mut s = Status::default();
            tirocks_sys::crocksdb_delete_file(self.as_ptr(), name.path_to_slice(), s.as_mut_ptr());
            check_status!(s)
        }
    }

    /// Obtains the meta data of the specified column family of the DB.
    ///
    /// Existing data will be cleared first.
    #[inline]
    pub fn cf_metadata(&self, cf: &RawColumnFamilyHandle, data: &mut ColumnFamilyMetaData) {
        unsafe {
            tirocks_sys::crocksdb_get_column_family_meta_data(
                self.as_ptr(),
                cf.get(),
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
        cf: &RawColumnFamilyHandle,
        ranges: &[(impl AsRef<[u8]>, impl AsRef<[u8]>)],
    ) -> Result<Vec<u64>> {
        let mut sizes = Vec::with_capacity(ranges.len());
        unsafe {
            let raw_ranges: Vec<_> = ranges
                .into_iter()
                .map(|(s, e)| range_to_rocks(s, e))
                .collect();
            let mut s = Status::default();
            tirocks_sys::crocksdb_approximate_sizes_cf(
                self.as_ptr(),
                opt,
                cf.get(),
                raw_ranges.as_ptr(),
                raw_ranges.len() as i32,
                sizes.as_mut_ptr(),
                s.as_mut_ptr(),
            );
            check_status!(s)?;
            sizes.set_len(raw_ranges.len());
            Ok(sizes)
        }
    }

    /// The method is similar to [`approximate_sizes`], except it returns approximate number
    /// and size of records in memtables.
    pub fn approximate_mem_table_stats(
        &self,
        cf: &RawColumnFamilyHandle,
        start_key: &[u8],
        end_key: &[u8],
    ) -> (u64, u64) {
        unsafe {
            let raw_range = range_to_rocks(&start_key, &end_key);
            let (mut count, mut size) = (0, 0);
            tirocks_sys::crocksdb_approximate_memtable_stats_cf(
                self.as_ptr(),
                cf.get(),
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
            let mut s = Status::default();
            tirocks_sys::crocksdb_destroy_db(
                path.path_to_slice(),
                options.get(),
                cf_names.as_ptr(),
                cf_opts.as_ptr(),
                cfs.len(),
                s.as_mut_ptr(),
            );
            check_status!(s)
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
            let mut s = Status::default();
            tirocks_sys::crocksdb_repair_db(
                path.path_to_slice(),
                options.as_ptr(),
                cf_names.as_ptr(),
                cf_opts.as_ptr(),
                cfs.len(),
                s.as_mut_ptr(),
            );
            check_status!(s)
        }
    }
}

#[derive(Debug)]
pub struct Db {
    ptr: *mut rocksdb_DB,
    _env: Option<Arc<Env>>,
    comparator: Vec<Arc<SysComparator>>,
    handles: Vec<RefCountedColumnFamilyHandle>,
    is_titan: bool,
}

impl Drop for Db {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            for h in &mut self.handles {
                // TODO: may should log.
                let _ = h.maybe_drop(self.ptr);
            }
            tirocks_sys::crocksdb_destroy(self.ptr as _);
        }
    }
}

impl Db {
    pub(crate) fn new(
        ptr: *mut rocksdb_DB,
        env: Option<Arc<Env>>,
        comparator: Vec<Arc<SysComparator>>,
        handles: Vec<RefCountedColumnFamilyHandle>,
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

    pub fn close(self) -> Status {
        let mut s = Status::default();
        unsafe {
            tirocks_sys::crocksdb_close(self.ptr, s.as_mut_ptr());
        }
        s
    }

    pub fn list_column_families(db: DbOptions, path: impl AsRef<Path>) -> Result<Vec<String>> {
        let mut convert_s = Status::default();
        let mut s = Status::default();
        let mut names = Vec::new();
        unsafe {
            let mut handle = |name: &[u8]| match str::from_utf8(name) {
                Ok(n) => names.push(n.to_string()),
                Err(e) => {
                    convert_s = Status::with_error(Code::kCorruption, format!("{}", e));
                }
            };
            let (ctx, fp) = util::wrap_string_receiver(&mut handle);
            tirocks_sys::crocksdb_list_column_families(
                db.get(),
                path.as_ref().path_to_slice(),
                ctx,
                Some(fp),
                s.as_mut_ptr(),
            )
        }

        check_status!(s)?;
        check_status!(convert_s)?;
        Ok(names)
    }

    pub fn create_column_family(&mut self, name: impl AsRef<str>, opt: CfOptions) -> Result<()> {
        if self.is_titan {
            return self.create_column_family_titan(name, opt.into());
        }
        let mut s = Status::default();
        let ptr = unsafe {
            tirocks_sys::crocksdb_create_column_family(
                self.ptr,
                opt.as_ptr(),
                r(name.as_ref().as_bytes()),
                s.as_mut_ptr(),
            )
        };
        check_status!(s)?;
        opt.comparator().map(|c| self.comparator.push(c.clone()));
        self.handles
            .push(unsafe { RefCountedColumnFamilyHandle::from_ptr(ptr) });
        Ok(())
    }

    pub fn create_column_family_titan(
        &mut self,
        name: impl AsRef<str>,
        mut opt: TitanCfOptions,
    ) -> Result<()> {
        if !self.is_titan {
            return self.create_column_family(name, opt.into());
        }
        let mut s = Status::default();
        let ptr = unsafe {
            tirocks_sys::ctitandb_create_column_family(
                self.ptr,
                opt.as_mut_ptr(),
                r(name.as_ref().as_bytes()),
                s.as_mut_ptr(),
            )
        };
        check_status!(s)?;
        opt.comparator().map(|c| self.comparator.push(c.clone()));
        self.handles
            .push(unsafe { RefCountedColumnFamilyHandle::from_ptr(ptr) });
        Ok(())
    }

    pub fn destroy_column_family(&mut self, name: &str) -> Result<bool> {
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

    pub fn cf(&self, name: &str) -> Option<&RawColumnFamilyHandle> {
        unsafe { self.cf_raw(name).map(|c| &**c) }
    }

    pub(crate) unsafe fn cf_raw(&self, name: &str) -> Option<&RefCountedColumnFamilyHandle> {
        for h in &self.handles {
            if h.name().map_or(false, |n| n == name) {
                return Some(h);
            }
        }
        None
    }

    // TitanOptions doesn't inherit Options, so we can mix them.
    #[inline]
    pub fn cf_options_titan(&self, cf: &RawColumnFamilyHandle) -> Option<RawTitanOptions> {
        unsafe {
            if self.is_titan {
                let ptr = tirocks_sys::ctitandb_get_titan_options_cf(self.as_ptr(), cf.get());
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
                let ptr = tirocks_sys::ctitandb_get_titan_db_options(self.as_ptr());
                Some(OwnedRawTitanDbOptions::from_ptr(ptr))
            } else {
                None
            }
        }
    }

    pub(crate) fn get(&self) -> *mut rocksdb_DB {
        self.ptr
    }

    #[inline]
    pub fn is_titan(&self) -> bool {
        self.is_titan
    }

    #[inline]
    pub fn iter<'a>(
        &'a self,
        read: &'a mut ReadOptions,
        cf: &'a RawColumnFamilyHandle,
    ) -> RawIterator<'a> {
        RawIterator::new(self, read, cf)
    }
}

impl Deref for Db {
    type Target = RawDb;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.ptr as *mut RawDb) }
    }
}
