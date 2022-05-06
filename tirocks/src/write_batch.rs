// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::{marker::PhantomData, mem};

use tirocks_sys::{r, rocksdb_WriteBatch, rocksdb_WriteBatch_Iterator, s};

use crate::{db::RawColumnFamilyHandle, util::check_status, Result, Status};

/// WriteBatch holds a collection of updates to apply atomically to a DB.
///
/// The updates are applied in the order in which they are added
/// to the WriteBatch.  For example, the value of "key" will be "v3"
/// after the following batch is written:
///
/// ```ignore
/// batch.put(&cf, "key", "v1");
/// batch.delete(&cf, "key");
/// batch.put(&cf, "key", "v2");
/// batch.put(&cf, "key", "v3");
/// ```
///
/// Multiple threads can invoke const methods on a WriteBatch without
/// external synchronization, but if any of the threads may call a
/// non-const method, all threads accessing the same WriteBatch must use
/// external synchronization.
#[derive(Debug)]
pub struct WriteBatch {
    ptr: *mut rocksdb_WriteBatch,
}

impl Default for WriteBatch {
    #[inline]
    fn default() -> Self {
        unsafe {
            let ptr = tirocks_sys::crocksdb_writebatch_create();
            Self { ptr }
        }
    }
}

impl Drop for WriteBatch {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            tirocks_sys::crocksdb_writebatch_destroy(self.ptr);
        }
    }
}

impl WriteBatch {
    #[inline]
    pub fn with_capacity(reserved_bytes: usize) -> Self {
        unsafe {
            let ptr = tirocks_sys::crocksdb_writebatch_create_with_capacity(reserved_bytes);
            WriteBatch { ptr }
        }
    }

    #[inline]
    pub fn from_bytes(buf: &[u8]) -> Self {
        unsafe {
            let ptr = tirocks_sys::crocksdb_writebatch_create_from(r(buf));
            WriteBatch { ptr }
        }
    }

    /// Store the mapping "key->value" in the database.
    #[inline]
    pub fn put(&mut self, cf: &RawColumnFamilyHandle, key: &[u8], value: &[u8]) -> Result<()> {
        unsafe {
            let mut s = Status::default();
            tirocks_sys::crocksdb_writebatch_put_cf(
                self.ptr,
                cf.as_mut_ptr(),
                r(key),
                r(value),
                s.as_mut_ptr(),
            );
            check_status!(s)
        }
    }

    /// Same as [`put`] but put to default cf and is slightly more efficient.
    #[inline]
    pub fn put_default(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        unsafe {
            let mut s = Status::default();
            tirocks_sys::crocksdb_writebatch_put(self.ptr, r(key), r(value), s.as_mut_ptr());
            check_status!(s)
        }
    }

    /// Variant of Put() that gathers output like writev(2).  The key and value
    /// that will be written to the database are concatenations of arrays of
    /// slices.
    #[inline]
    pub fn put_vectored(
        &mut self,
        cf: &RawColumnFamilyHandle,
        keys: &[&[u8]],
        values: &[&[u8]],
    ) -> Result<()> {
        unsafe {
            let mut s = Status::default();
            if tirocks_sys::rocks_slice_same_as_rust() {
                let keys: &[_] = mem::transmute(keys);
                let values: &[_] = mem::transmute(values);
                tirocks_sys::crocksdb_writebatch_putv_cf(
                    self.ptr,
                    cf.as_mut_ptr(),
                    keys.len() as i32,
                    keys.as_ptr(),
                    values.len() as i32,
                    values.as_ptr(),
                    s.as_mut_ptr(),
                )
            } else {
                let keys: Vec<_> = keys.into_iter().map(|k| r(k)).collect();
                let values: Vec<_> = values.into_iter().map(|v| r(v)).collect();
                tirocks_sys::crocksdb_writebatch_putv_cf(
                    self.ptr,
                    cf.as_mut_ptr(),
                    keys.len() as i32,
                    keys.as_ptr(),
                    values.len() as i32,
                    values.as_ptr(),
                    s.as_mut_ptr(),
                )
            }
            check_status!(s)
        }
    }

    /// Same as [`put_vectored`] but put to default cf and is slightly more efficient.
    #[inline]
    pub fn put_default_vectored(&mut self, keys: &[&[u8]], values: &[&[u8]]) -> Result<()> {
        unsafe {
            let mut s = Status::default();
            if tirocks_sys::rocks_slice_same_as_rust() {
                let keys: &[_] = mem::transmute(keys);
                let values: &[_] = mem::transmute(values);
                tirocks_sys::crocksdb_writebatch_putv(
                    self.ptr,
                    keys.len() as i32,
                    keys.as_ptr(),
                    values.len() as i32,
                    values.as_ptr(),
                    s.as_mut_ptr(),
                )
            } else {
                let keys: Vec<_> = keys.into_iter().map(|k| r(k)).collect();
                let values: Vec<_> = values.into_iter().map(|v| r(v)).collect();
                tirocks_sys::crocksdb_writebatch_putv(
                    self.ptr,
                    keys.len() as i32,
                    keys.as_ptr(),
                    values.len() as i32,
                    values.as_ptr(),
                    s.as_mut_ptr(),
                )
            }
            check_status!(s)
        }
    }

    /// If the database contains a mapping for "key", erase it.  Else do nothing.
    #[inline]
    pub fn delete(&mut self, cf: &RawColumnFamilyHandle, key: &[u8]) -> Result<()> {
        unsafe {
            let mut s = Status::default();
            tirocks_sys::crocksdb_writebatch_delete_cf(
                self.ptr,
                cf.as_mut_ptr(),
                r(key),
                s.as_mut_ptr(),
            );
            check_status!(s)
        }
    }

    /// Same as [`delete`] but delete to default cf and is slightly more efficient.
    #[inline]
    pub fn delete_default(&mut self, key: &[u8]) -> Result<()> {
        unsafe {
            let mut s = Status::default();
            tirocks_sys::crocksdb_writebatch_delete(self.ptr, r(key), s.as_mut_ptr());
            check_status!(s)
        }
    }

    #[inline]
    pub fn delete_vectored(&mut self, cf: &RawColumnFamilyHandle, keys: &[&[u8]]) -> Result<()> {
        unsafe {
            let mut s = Status::default();
            if tirocks_sys::rocks_slice_same_as_rust() {
                let keys: &[_] = mem::transmute(keys);
                tirocks_sys::crocksdb_writebatch_deletev_cf(
                    self.ptr,
                    cf.as_mut_ptr(),
                    keys.len() as i32,
                    keys.as_ptr(),
                    s.as_mut_ptr(),
                )
            } else {
                let keys: Vec<_> = keys.into_iter().map(|k| r(k)).collect();
                tirocks_sys::crocksdb_writebatch_deletev_cf(
                    self.ptr,
                    cf.as_mut_ptr(),
                    keys.len() as i32,
                    keys.as_ptr(),
                    s.as_mut_ptr(),
                )
            }
            check_status!(s)
        }
    }

    #[inline]
    pub fn delete_default_vectored(&mut self, keys: &[&[u8]]) -> Result<()> {
        unsafe {
            let mut s = Status::default();
            if tirocks_sys::rocks_slice_same_as_rust() {
                let keys: &[_] = mem::transmute(keys);
                tirocks_sys::crocksdb_writebatch_deletev(
                    self.ptr,
                    keys.len() as i32,
                    keys.as_ptr(),
                    s.as_mut_ptr(),
                )
            } else {
                let keys: Vec<_> = keys.into_iter().map(|k| r(k)).collect();
                tirocks_sys::crocksdb_writebatch_deletev(
                    self.ptr,
                    keys.len() as i32,
                    keys.as_ptr(),
                    s.as_mut_ptr(),
                )
            }
            check_status!(s)
        }
    }

    // WriteBatch implementation of Db::single_delete().
    #[inline]
    pub fn single_delete(&mut self, cf: &RawColumnFamilyHandle, key: &[u8]) -> Result<()> {
        unsafe {
            let mut s = Status::default();
            tirocks_sys::crocksdb_writebatch_single_delete_cf(
                self.ptr,
                cf.as_mut_ptr(),
                r(key),
                s.as_mut_ptr(),
            );
            check_status!(s)
        }
    }

    /// Same as [`single_delete`] but delete to default cf and is slightly more efficient.
    #[inline]
    pub fn single_delete_default(&mut self, key: &[u8]) -> Result<()> {
        unsafe {
            let mut s = Status::default();
            tirocks_sys::crocksdb_writebatch_single_delete(self.ptr, r(key), s.as_mut_ptr());
            check_status!(s)
        }
    }

    #[inline]
    pub fn single_delete_vectored(
        &mut self,
        cf: &RawColumnFamilyHandle,
        keys: &[&[u8]],
    ) -> Result<()> {
        unsafe {
            let mut s = Status::default();
            if tirocks_sys::rocks_slice_same_as_rust() {
                let keys: &[_] = mem::transmute(keys);
                tirocks_sys::crocksdb_writebatch_single_deletev_cf(
                    self.ptr,
                    cf.as_mut_ptr(),
                    keys.len() as i32,
                    keys.as_ptr(),
                    s.as_mut_ptr(),
                )
            } else {
                let keys: Vec<_> = keys.into_iter().map(|k| r(k)).collect();
                tirocks_sys::crocksdb_writebatch_single_deletev_cf(
                    self.ptr,
                    cf.as_mut_ptr(),
                    keys.len() as i32,
                    keys.as_ptr(),
                    s.as_mut_ptr(),
                )
            }
            check_status!(s)
        }
    }

    #[inline]
    pub fn delete_single_default_vectored(&mut self, keys: &[&[u8]]) -> Result<()> {
        unsafe {
            let mut s = Status::default();
            if tirocks_sys::rocks_slice_same_as_rust() {
                let keys: &[_] = mem::transmute(keys);
                tirocks_sys::crocksdb_writebatch_single_deletev(
                    self.ptr,
                    keys.len() as i32,
                    keys.as_ptr(),
                    s.as_mut_ptr(),
                )
            } else {
                let keys: Vec<_> = keys.into_iter().map(|k| r(k)).collect();
                tirocks_sys::crocksdb_writebatch_single_deletev(
                    self.ptr,
                    keys.len() as i32,
                    keys.as_ptr(),
                    s.as_mut_ptr(),
                )
            }
            check_status!(s)
        }
    }

    /// WriteBatch implementation of Db::delete_range().
    #[inline]
    pub fn delete_range(
        &mut self,
        cf: &RawColumnFamilyHandle,
        begin_key: &[u8],
        end_key: &[u8],
    ) -> Result<()> {
        unsafe {
            let mut s = Status::default();
            tirocks_sys::crocksdb_writebatch_delete_range_cf(
                self.ptr,
                cf.as_mut_ptr(),
                r(begin_key),
                r(end_key),
                s.as_mut_ptr(),
            );
            check_status!(s)
        }
    }

    /// Same as [`delete_range`] but delete to default cf and is slightly more efficient.
    #[inline]
    pub fn delete_range_default(&mut self, begin_key: &[u8], end_key: &[u8]) -> Result<()> {
        unsafe {
            let mut s = Status::default();
            tirocks_sys::crocksdb_writebatch_delete_range(
                self.ptr,
                r(begin_key),
                r(end_key),
                s.as_mut_ptr(),
            );
            check_status!(s)
        }
    }

    #[inline]
    pub fn delete_range_vectored(
        &mut self,
        cf: &RawColumnFamilyHandle,
        begin_keys: &[&[u8]],
        end_keys: &[&[u8]],
    ) -> Result<()> {
        unsafe {
            let mut s = Status::default();
            if tirocks_sys::rocks_slice_same_as_rust() {
                let begin_keys: &[_] = mem::transmute(begin_keys);
                let end_keys: &[_] = mem::transmute(end_keys);
                tirocks_sys::crocksdb_writebatch_delete_rangev_cf(
                    self.ptr,
                    cf.as_mut_ptr(),
                    begin_keys.len() as i32,
                    begin_keys.as_ptr(),
                    end_keys.len() as i32,
                    end_keys.as_ptr(),
                    s.as_mut_ptr(),
                )
            } else {
                let begin_keys: Vec<_> = begin_keys.into_iter().map(|k| r(k)).collect();
                let end_keys: Vec<_> = end_keys.into_iter().map(|k| r(k)).collect();
                tirocks_sys::crocksdb_writebatch_delete_rangev_cf(
                    self.ptr,
                    cf.as_mut_ptr(),
                    begin_keys.len() as i32,
                    begin_keys.as_ptr(),
                    end_keys.len() as i32,
                    end_keys.as_ptr(),
                    s.as_mut_ptr(),
                )
            }
            check_status!(s)
        }
    }

    #[inline]
    pub fn delete_range_default_vectored(
        &mut self,
        begin_keys: &[&[u8]],
        end_keys: &[&[u8]],
    ) -> Result<()> {
        unsafe {
            let mut s = Status::default();
            if tirocks_sys::rocks_slice_same_as_rust() {
                let begin_keys: &[_] = mem::transmute(begin_keys);
                let end_keys: &[_] = mem::transmute(end_keys);
                tirocks_sys::crocksdb_writebatch_delete_rangev(
                    self.ptr,
                    begin_keys.len() as i32,
                    begin_keys.as_ptr(),
                    end_keys.len() as i32,
                    end_keys.as_ptr(),
                    s.as_mut_ptr(),
                )
            } else {
                let begin_keys: Vec<_> = begin_keys.into_iter().map(|k| r(k)).collect();
                let end_keys: Vec<_> = end_keys.into_iter().map(|k| r(k)).collect();
                tirocks_sys::crocksdb_writebatch_delete_rangev(
                    self.ptr,
                    begin_keys.len() as i32,
                    begin_keys.as_ptr(),
                    end_keys.len() as i32,
                    end_keys.as_ptr(),
                    s.as_mut_ptr(),
                )
            }
            check_status!(s)
        }
    }

    /// Append a blob of arbitrary size to the records in this batch. The blob will
    /// be stored in the transaction log but not in any other file. In particular,
    /// it will not be persisted to the SST files. When iterating over this
    /// WriteBatch, WriteBatch::Handler::LogData will be called with the contents
    /// of the blob as it is encountered. Blobs, puts, deletes, and merges will be
    /// encountered in the same order in which they were inserted. The blob will
    /// NOT consume sequence number(s) and will NOT increase the count of the batch
    ///
    /// Example application: add timestamps to the transaction log for use in
    /// replication.
    #[inline]
    pub fn put_log_data(&mut self, blob: &[u8]) -> Result<()> {
        unsafe {
            let mut s = Status::default();
            tirocks_sys::crocksdb_writebatch_put_log_data(self.ptr, r(blob), s.as_mut_ptr());
            check_status!(s)
        }
    }

    /// Clear all updates buffered in this batch.
    #[inline]
    pub fn clear(&mut self) {
        unsafe {
            tirocks_sys::crocksdb_writebatch_clear(self.ptr);
        }
    }

    /// Records the state of the batch for future calls to RollbackToSavePoint().
    /// May be called multiple times to set multiple save points.
    #[inline]
    pub fn set_save_point(&mut self) {
        unsafe {
            tirocks_sys::crocksdb_writebatch_set_save_point(self.ptr);
        }
    }

    /// Remove all entries in this batch (Put, Merge, Delete, PutLogData) since the
    /// most recent call to SetSavePoint() and removes the most recent save point.
    /// If there is no previous call to SetSavePoint(), Status::NotFound()
    /// will be returned.
    /// Otherwise returns Status::OK().
    #[inline]
    pub fn rollback_to_save_point(&mut self) -> Result<()> {
        unsafe {
            let mut s = Status::default();
            tirocks_sys::crocksdb_writebatch_rollback_to_save_point(self.ptr, s.as_mut_ptr());
            check_status!(s)
        }
    }

    /// Pop the most recent save point.
    /// If there is no previous call to SetSavePoint(), Status::NotFound()
    /// will be returned.
    /// Otherwise returns Status::OK().
    #[inline]
    pub fn pop_save_point(&mut self) -> Result<()> {
        unsafe {
            let mut s = Status::default();
            tirocks_sys::crocksdb_writebatch_pop_save_point(self.ptr, s.as_mut_ptr());
            check_status!(s)
        }
    }

    /// Retrieve the serialized version of this batch.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            let mut buf = r(&[]);
            tirocks_sys::crocksdb_writebatch_data(self.ptr, &mut buf);
            s(buf)
        }
    }

    /// Returns the number of updates in the batch
    #[inline]
    pub fn count(&self) -> usize {
        unsafe { tirocks_sys::crocksdb_writebatch_count(self.ptr) as usize }
    }

    //// Parse the blob as write batch and returns the number of updates.
    #[inline]
    pub fn count_blob(bytes: &[u8]) -> usize {
        unsafe { tirocks_sys::crocksdb_writebatch_ref_count(r(bytes)) as usize }
    }

    /// Iterate over current writebatch.
    #[inline]
    pub fn iter(&self) -> WriteBatchIter {
        unsafe {
            let ptr = tirocks_sys::crocksdb_writebatch_iterator_create(self.ptr);
            WriteBatchIter::from_ptr(ptr)
        }
    }

    /// Parse the blob as write batch and iterate over it.
    #[inline]
    pub fn iter_blob(bytes: &[u8]) -> WriteBatchIter {
        unsafe {
            let ptr = tirocks_sys::crocksdb_writebatch_ref_iterator_create(r(bytes));
            WriteBatchIter::from_ptr(ptr)
        }
    }

    #[inline]
    pub(crate) fn as_mut_ptr(&mut self) -> *mut rocksdb_WriteBatch {
        self.ptr
    }
}

pub struct WriteBatchIter<'a> {
    ptr: *mut rocksdb_WriteBatch_Iterator,
    _life: PhantomData<&'a ()>,
}

impl<'a> Drop for WriteBatchIter<'a> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            tirocks_sys::crocksdb_writebatch_iterator_destroy(self.ptr);
        }
    }
}

impl<'a> WriteBatchIter<'a> {
    fn from_ptr(ptr: *mut rocksdb_WriteBatch_Iterator) -> Self {
        WriteBatchIter {
            ptr,
            _life: PhantomData,
        }
    }

    #[inline]
    pub fn valid(&self) -> bool {
        unsafe { tirocks_sys::crocksdb_writebatch_iterator_valid(self.ptr) }
    }

    #[inline]
    pub fn next(&mut self) {
        unsafe {
            tirocks_sys::crocksdb_writebatch_iterator_next(self.ptr);
        }
    }

    #[inline]
    pub fn key(&self) -> &[u8] {
        unsafe {
            let mut buf = r(&[]);
            tirocks_sys::crocksdb_writebatch_iterator_key(self.ptr, &mut buf);
            s(buf)
        }
    }

    #[inline]
    pub fn value(&self) -> &[u8] {
        unsafe {
            let mut buf = r(&[]);
            tirocks_sys::crocksdb_writebatch_iterator_value(self.ptr, &mut buf);
            s(buf)
        }
    }

    #[inline]
    pub fn value_type(&self) -> libc::c_char {
        unsafe { tirocks_sys::crocksdb_writebatch_iterator_value_type(self.ptr) }
    }

    #[inline]
    pub fn column_family_id(&self) -> u32 {
        unsafe { tirocks_sys::crocksdb_writebatch_iterator_column_family_id(self.ptr) }
    }
}
