// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::marker::PhantomData;

use tirocks_sys::{r, rocksdb_WriteBatch, rocksdb_WriteBatch_Iterator, s};

use crate::{
    util::{self, ffi_call},
    Result, Status,
};

// TODO: use actual definition from db/cf.rs.
pub struct RawCfHandle;

impl RawCfHandle {
    fn get_ptr(&self) -> *mut tirocks_sys::rocksdb_ColumnFamilyHandle {
        unimplemented!()
    }
}

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
    pub fn put(&mut self, cf: &RawCfHandle, key: &[u8], value: &[u8]) -> Result<()> {
        unsafe {
            ffi_call!(crocksdb_writebatch_put_cf(
                self.ptr,
                cf.get_ptr(),
                r(key),
                r(value),
            ))
        }
    }

    /// Same as [`put`] but put to default cf and is slightly more efficient.
    #[inline]
    pub fn put_default(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        unsafe { ffi_call!(crocksdb_writebatch_put(self.ptr, r(key), r(value))) }
    }

    /// Variant of Put() that gathers output like writev(2).  The key and value
    /// that will be written to the database are concatenations of arrays of
    /// slices.
    #[inline]
    pub fn put_vectored(
        &mut self,
        cf: &RawCfHandle,
        keys: &[&[u8]],
        values: &[&[u8]],
    ) -> Result<()> {
        unsafe {
            let keys = util::array_to_rocks_slice(keys);
            let values = util::array_to_rocks_slice(values);
            ffi_call!(crocksdb_writebatch_putv_cf(
                self.ptr,
                cf.get_ptr(),
                keys.len() as i32,
                keys.as_ptr(),
                values.len() as i32,
                values.as_ptr(),
            ))
        }
    }

    /// Same as [`put_vectored`] but put to default cf and is slightly more efficient.
    #[inline]
    pub fn put_default_vectored(&mut self, keys: &[&[u8]], values: &[&[u8]]) -> Result<()> {
        unsafe {
            let keys = util::array_to_rocks_slice(keys);
            let values = util::array_to_rocks_slice(values);
            ffi_call!(crocksdb_writebatch_putv(
                self.ptr,
                keys.len() as i32,
                keys.as_ptr(),
                values.len() as i32,
                values.as_ptr(),
            ))
        }
    }

    /// If the database contains a mapping for "key", erase it.  Else do nothing.
    #[inline]
    pub fn delete(&mut self, cf: &RawCfHandle, key: &[u8]) -> Result<()> {
        unsafe {
            ffi_call!(crocksdb_writebatch_delete_cf(
                self.ptr,
                cf.get_ptr(),
                r(key),
            ))
        }
    }

    /// Same as [`delete`] but delete to default cf and is slightly more efficient.
    #[inline]
    pub fn delete_default(&mut self, key: &[u8]) -> Result<()> {
        unsafe { ffi_call!(crocksdb_writebatch_delete(self.ptr, r(key))) }
    }

    #[inline]
    pub fn delete_vectored(&mut self, cf: &RawCfHandle, keys: &[&[u8]]) -> Result<()> {
        unsafe {
            let keys = util::array_to_rocks_slice(keys);
            ffi_call!(crocksdb_writebatch_deletev_cf(
                self.ptr,
                cf.get_ptr(),
                keys.len() as i32,
                keys.as_ptr(),
            ))
        }
    }

    #[inline]
    pub fn delete_default_vectored(&mut self, keys: &[&[u8]]) -> Result<()> {
        unsafe {
            let keys = util::array_to_rocks_slice(keys);
            ffi_call!(crocksdb_writebatch_deletev(
                self.ptr,
                keys.len() as i32,
                keys.as_ptr(),
            ))
        }
    }

    // WriteBatch implementation of Db::single_delete().
    #[inline]
    pub fn single_delete(&mut self, cf: &RawCfHandle, key: &[u8]) -> Result<()> {
        unsafe {
            let mut s = Status::default();
            ffi_call!(crocksdb_writebatch_single_delete_cf(
                self.ptr,
                cf.get_ptr(),
                r(key),
            ))
        }
    }

    /// Same as [`single_delete`] but delete to default cf and is slightly more efficient.
    #[inline]
    pub fn single_delete_default(&mut self, key: &[u8]) -> Result<()> {
        unsafe { ffi_call!(crocksdb_writebatch_single_delete(self.ptr, r(key))) }
    }

    #[inline]
    pub fn single_delete_vectored(&mut self, cf: &RawCfHandle, keys: &[&[u8]]) -> Result<()> {
        unsafe {
            let keys = util::array_to_rocks_slice(keys);
            ffi_call!(crocksdb_writebatch_single_deletev_cf(
                self.ptr,
                cf.get_ptr(),
                keys.len() as i32,
                keys.as_ptr(),
            ))
        }
    }

    #[inline]
    pub fn delete_single_default_vectored(&mut self, keys: &[&[u8]]) -> Result<()> {
        unsafe {
            let keys = util::array_to_rocks_slice(keys);
            ffi_call!(crocksdb_writebatch_single_deletev(
                self.ptr,
                keys.len() as i32,
                keys.as_ptr(),
            ))
        }
    }

    /// WriteBatch implementation of Db::delete_range().
    #[inline]
    pub fn delete_range(
        &mut self,
        cf: &RawCfHandle,
        begin_key: &[u8],
        end_key: &[u8],
    ) -> Result<()> {
        unsafe {
            ffi_call!(crocksdb_writebatch_delete_range_cf(
                self.ptr,
                cf.get_ptr(),
                r(begin_key),
                r(end_key),
            ))
        }
    }

    /// Same as [`delete_range`] but delete to default cf and is slightly more efficient.
    #[inline]
    pub fn delete_range_default(&mut self, begin_key: &[u8], end_key: &[u8]) -> Result<()> {
        unsafe {
            ffi_call!(crocksdb_writebatch_delete_range(
                self.ptr,
                r(begin_key),
                r(end_key),
            ))
        }
    }

    #[inline]
    pub fn delete_range_vectored(
        &mut self,
        cf: &RawCfHandle,
        begin_keys: &[&[u8]],
        end_keys: &[&[u8]],
    ) -> Result<()> {
        unsafe {
            let begin_keys = util::array_to_rocks_slice(begin_keys);
            let end_keys = util::array_to_rocks_slice(end_keys);
            ffi_call!(crocksdb_writebatch_delete_rangev_cf(
                self.ptr,
                cf.get_ptr(),
                begin_keys.len() as i32,
                begin_keys.as_ptr(),
                end_keys.len() as i32,
                end_keys.as_ptr(),
            ))
        }
    }

    #[inline]
    pub fn delete_range_default_vectored(
        &mut self,
        begin_keys: &[&[u8]],
        end_keys: &[&[u8]],
    ) -> Result<()> {
        unsafe {
            let begin_keys = util::array_to_rocks_slice(begin_keys);
            let end_keys = util::array_to_rocks_slice(end_keys);
            ffi_call!(crocksdb_writebatch_delete_rangev(
                self.ptr,
                begin_keys.len() as i32,
                begin_keys.as_ptr(),
                end_keys.len() as i32,
                end_keys.as_ptr(),
            ))
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
        unsafe { ffi_call!(crocksdb_writebatch_put_log_data(self.ptr, r(blob))) }
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
        unsafe { ffi_call!(crocksdb_writebatch_rollback_to_save_point(self.ptr)) }
    }

    /// Pop the most recent save point.
    /// If there is no previous call to SetSavePoint(), Status::NotFound()
    /// will be returned.
    /// Otherwise returns Status::OK().
    #[inline]
    pub fn pop_save_point(&mut self) -> Result<()> {
        unsafe { ffi_call!(crocksdb_writebatch_pop_save_point(self.ptr)) }
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

/// Value types encoded as the last component of internal keys.
///
/// If value is not equal to any of these, it means either corruption or version incompatible.
pub mod value_type {
    pub const TYPE_DELETION: u8 = 0x0;
    pub const TYPE_VALUE: u8 = 0x1;
    pub const TYPE_MERGE: u8 = 0x2;
    /// WAL only.
    pub const TYPE_LOG_DATA: u8 = 0x3;
    /// WAL only.
    pub const TYPE_CF_DELETION: u8 = 0x4;
    /// WAL only.
    pub const TYPE_CF_VALUE: u8 = 0x5;
    /// WAL only.
    pub const TYPE_CF_MERGE: u8 = 0x6;
    pub const TYPE_SINGLE_DELETION: u8 = 0x7;
    /// WAL only.
    pub const TYPE_CF_SINGLE_DELETION: u8 = 0x8;
    /// WAL only.
    pub const TYPE_BEGIN_PREPARE_XID: u8 = 0x9;
    /// WAL only.
    pub const TYPE_END_PREPARE_XID: u8 = 0xA;
    /// WAL only.
    pub const TYPE_COMMIT_XID: u8 = 0xB;
    /// WAL only.
    pub const TYPE_ROLLBACK_XID: u8 = 0xC;
    /// WAL only.
    pub const TYPE_NOOP: u8 = 0xD;
    /// WAL only.
    pub const TYPE_CF_RANGE_DELETION: u8 = 0xE;
    /// meta block
    pub const TYPE_RANGE_DELETION: u8 = 0xF;
    /// Blob DB only
    pub const TYPE_CF_BLOB_INDEX: u8 = 0x10;
    /// Blob DB only
    pub const TYPE_BLOB_INDEX: u8 = 0x11;
    // When the prepared record is also persisted in db, we use a different
    // record. This is to ensure that the WAL that is generated by a WritePolicy
    // is not mistakenly read by another, which would result into data
    // inconsistency. WAL only.
    pub const TYPE_BEGIN_PERSISTED_PREPARE_XID: u8 = 0x12;
    // Similar to kTypeBeginPersistedPrepareXID, this is to ensure that WAL
    // generated by WriteUnprepared write policy is not mistakenly read by
    // another. WAL only.
    pub const TYPE_BEGIN_UNPREPARE_XID: u8 = 0x13;
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

    /// Get the value type.
    ///
    /// Check [`crate::write_batch::value_type`] for possible candidates.
    #[inline]
    pub fn value_type(&self) -> u8 {
        unsafe { tirocks_sys::crocksdb_writebatch_iterator_value_type(self.ptr) as u8 }
    }

    #[inline]
    pub fn cf_id(&self) -> u32 {
        unsafe { tirocks_sys::crocksdb_writebatch_iterator_column_family_id(self.ptr) }
    }
}
