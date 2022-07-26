// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::{
    fmt::{self, Debug, Formatter},
    mem::MaybeUninit,
};

use libc::c_void;
use tirocks_sys::{rocksdb_ReadOptions, rocksdb_Snapshot, rocksdb_titandb_TitanReadOptions, s};

use crate::{
    table_filter::{self, TableFilter},
    util::simple_access,
};

use super::OwnedSlice;

pub type ReadTier = tirocks_sys::rocksdb_ReadTier;
pub type SequenceNumber = tirocks_sys::rocksdb_SequenceNumber;

#[derive(Default)]
struct ReadOptionsStorage {
    iterate_lower_bound: OwnedSlice,
    iterate_upper_bound: OwnedSlice,
    timestamp: OwnedSlice,
    iterate_start_ts: OwnedSlice,
}

/// Options that control read operations
pub struct ReadOptions {
    raw: rocksdb_titandb_TitanReadOptions,
    slice_store: Option<Box<ReadOptionsStorage>>,
}

impl Default for ReadOptions {
    #[inline]
    fn default() -> ReadOptions {
        let mut opt = MaybeUninit::uninit();
        unsafe {
            tirocks_sys::ctitandb_readoptions_init(opt.as_mut_ptr());
            ReadOptions {
                raw: opt.assume_init(),
                slice_store: None,
            }
        }
    }
}

impl Debug for ReadOptions {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "ReadOptions")
    }
}

impl ReadOptions {
    #[inline]
    fn init_slice_store(&mut self) {
        if self.slice_store.is_none() {
            self.slice_store = Some(Box::new(Default::default()));
        }
    }

    fn as_mut_ptr(&mut self) -> *mut rocksdb_ReadOptions {
        &mut self.raw as *mut _ as _
    }

    fn as_ptr(&self) -> *const rocksdb_ReadOptions {
        &self.raw as *const _ as _
    }

    /// If "snapshot" is set, read as of the supplied snapshot
    /// (which must belong to the DB that is being read and which must
    /// not have been released).  If "snapshot" is nullptr, use an implicit
    /// snapshot of the state at the beginning of this read operation.
    ///
    /// If set, caller should guarantee the snapshot outlives the read operation.
    #[inline]
    pub(crate) unsafe fn set_snapshot(&mut self, snapshot: *const rocksdb_Snapshot) -> &mut Self {
        tirocks_sys::crocksdb_readoptions_set_snapshot(self.as_mut_ptr(), snapshot);
        self
    }

    #[inline]
    pub(crate) fn snapshot(&self) -> *const rocksdb_Snapshot {
        unsafe { tirocks_sys::crocksdb_readoptions_snapshot(self.as_ptr()) }
    }

    /// Sets the smallest key at which the backward iterator can return an
    /// entry. Once the bound is passed, Valid() will be false. It is inclusive
    /// ie the bound value is a valid entry.
    ///
    /// If prefix_extractor is set, the Seek target and `iterate_lower_bound`
    /// need to have the same prefix. This is because ordering is not guaranteed
    /// outside of prefix domain.
    #[inline]
    pub fn set_iterate_lower_bound(
        &mut self,
        lower_bound: impl Into<Option<Vec<u8>>>,
    ) -> &mut Self {
        self.init_slice_store();
        let store = self.slice_store.as_mut().unwrap();
        let ptr = store.iterate_lower_bound.set_data(lower_bound.into());
        unsafe {
            tirocks_sys::crocksdb_readoptions_set_iterate_lower_bound(self.as_mut_ptr(), ptr);
        }
        self
    }

    /// Check [`set_iterate_lower_bound`].
    #[inline]
    pub fn iterate_lower_bound(&self) -> Option<&[u8]> {
        unsafe {
            let ptr = tirocks_sys::crocksdb_readoptions_iterate_lower_bound(self.as_ptr());
            if !ptr.is_null() {
                Some(s(*ptr))
            } else {
                None
            }
        }
    }

    /// Sets the extent upto which the forward iterator can returns entries.
    /// Once the bound is reached, Valid() will be false. It is exclusive ie
    /// the bound value is not a valid entry.
    ///
    /// If iterator_extractor is not null, the Seek target and
    /// iterate_upper_bound need to have the same prefix. This is because
    /// ordering is not guaranteed outside of prefix domain.
    #[inline]
    pub fn set_iterate_upper_bound(
        &mut self,
        upper_bound: impl Into<Option<Vec<u8>>>,
    ) -> &mut Self {
        self.init_slice_store();
        let store = self.slice_store.as_mut().unwrap();
        let ptr = store.iterate_upper_bound.set_data(upper_bound.into());
        unsafe {
            tirocks_sys::crocksdb_readoptions_set_iterate_upper_bound(self.as_mut_ptr(), ptr);
        }
        self
    }

    /// Check [`set_iterate_upper_bound`].
    #[inline]
    pub fn iterate_upper_bound(&self) -> Option<&[u8]> {
        unsafe {
            let ptr = tirocks_sys::crocksdb_readoptions_iterate_upper_bound(self.as_ptr());
            if !ptr.is_null() {
                Some(s(*ptr))
            } else {
                None
            }
        }
    }

    simple_access! {
        crocksdb_readoptions

        /// RocksDB does auto-readahead for iterators on noticing more than two reads
        /// for a table file. The readahead starts at 8KB and doubles on every
        /// additional read upto 256KB.
        /// This option can help if most of the range scans are large, and if it is
        /// determined that a larger readahead than that enabled by auto-readahead is
        /// needed.
        /// Using a large readahead size (> 2MB) can typically improve the performance
        /// of forward iteration on spinning disks.
        /// Default is 0.
        readahead_size: usize

        /// A threshold for the number of keys that can be skipped before failing an
        /// iterator seek as incomplete. The default value of 0 should be used to
        /// never fail a request as incomplete, even on skipping too many keys.
        /// Default: 0
        max_skippable_internal_keys: u64

        /// Specify if this read request should process data that ALREADY
        /// resides on a particular cache. If the required data is not
        /// found at the specified cache, then Status::Incomplete is returned.
        /// Default: kReadAllTier
        read_tier: ReadTier

        /// If true, all data read from underlying storage will be
        /// verified against corresponding checksums.
        /// Default: true
        verify_checksums: bool

        /// Set whether the "data block"/"index block"" read for this iteration be
        /// placed in block cache.
        ///
        /// Callers may wish to set this field to false for bulk scans. This would
        /// help not to the change eviction order of existing items in the block
        /// cache.
        /// Default: true
        fill_cache: bool

        /// Specify to create a tailing iterator -- a special iterator that has a
        /// view of the complete database (i.e. it can also be used to read newly
        /// added data) and is optimized for sequential reads. It will return records
        /// that were inserted into the database after the creation of the iterator.
        /// Default: false
        tailing: bool

        /// Enable a total order seek regardless of index format (e.g. hash index)
        /// used in the table. Some table format (e.g. plain table) may not support
        /// this option.
        /// If true when calling Get(), we also skip prefix bloom when reading from
        /// block based table. It provides a way to read existing data after
        /// changing implementation of prefix extractor.
        total_order_seek: bool

        /// When true, by default use total_order_seek = true, and RocksDB can
        /// selectively enable prefix seek mode if won't generate a different result
        /// from total_order_seek, based on seek key, and iterator upper bound.
        /// Not supported in ROCKSDB_LITE mode, in the way that even with value true
        /// prefix mode is not used.
        /// Default: false
        auto_prefix_mode: bool

        /// Enforce that the iterator only iterates over the same prefix as the seek.
        /// This option is effective only for prefix seeks, i.e. prefix_extractor is
        /// non-null for the column family and total_order_seek is false.  Unlike
        /// iterate_upper_bound, prefix_same_as_start only works within a prefix
        /// but in both directions.
        /// Default: false
        prefix_same_as_start: bool

        /// Keep the blocks loaded by the iterator pinned in memory as long as the
        /// iterator is not deleted, If used when reading from tables created with
        /// BlockBasedTableOptions::use_delta_encoding = false,
        /// Iterator's property "rocksdb.iterator.is-key-pinned" is guaranteed to
        /// return 1.
        /// Default: false
        pin_data: bool

        /// If true, when PurgeObsoleteFile is called in CleanupIteratorState, we
        /// schedule a background job in the flush job queue and delete obsolete files
        /// in background.
        /// Default: false
        background_purge_on_iterator_cleanup: bool

        /// If true, keys deleted using the DeleteRange() API will be visible to
        /// readers until they are naturally deleted during compaction. This improves
        /// read performance in DBs with many range deletions.
        /// Default: false
        ignore_range_deletions: bool
    }

    /// A callback to determine whether relevant keys for this scan exist in a
    /// given table based on the table's properties. The callback is passed the
    /// properties of each table during iteration. If the callback returns false,
    /// the table will not be scanned. This option only affects Iterators and has
    /// no impact on point lookups.
    /// Default: empty (every table will be scanned)
    #[inline]
    pub fn set_table_filter<T: TableFilter>(&mut self, f: T) -> &mut Self {
        unsafe {
            let filter = Box::into_raw(Box::new(f));
            tirocks_sys::crocksdb_readoptions_set_table_filter(
                self.as_mut_ptr(),
                filter as *mut c_void,
                Some(table_filter::filter::<T>),
                Some(table_filter::destroy::<T>),
            )
        }
        self
    }

    /// Timestamp of operation. Read should return the latest data visible to the
    /// specified timestamp. All timestamps of the same database must be of the
    /// same length and format. The user is responsible for providing a customized
    /// compare function via Comparator to order <key, timestamp> tuples.
    /// The user-specified timestamp feature is still under active development,
    /// and the API is subject to change.
    #[inline]
    pub fn set_timestamp(&mut self, timestamp: impl Into<Option<Vec<u8>>>) -> &mut Self {
        self.init_slice_store();
        let store = self.slice_store.as_mut().unwrap();
        let ptr = store.timestamp.set_data(timestamp.into());
        unsafe {
            tirocks_sys::crocksdb_readoptions_set_timestamp(self.as_mut_ptr(), ptr);
        }
        self
    }

    /// Set the lower bound (older) and timestamp serves as the upper bound for iterator.
    /// Versions of the same record that fall in the timestamp range will be returned. If
    /// iter_start_ts is None, only the most recent version visible to timestamp is returned.
    /// The user-specified timestamp feature is still under active development,
    /// and the API is subject to change.
    /// Default: None
    #[inline]
    pub fn set_iterate_start_ts(&mut self, timestamp: impl Into<Option<Vec<u8>>>) -> &mut Self {
        self.init_slice_store();
        let store = self.slice_store.as_mut().unwrap();
        let ptr = store.iterate_start_ts.set_data(timestamp.into());
        unsafe {
            tirocks_sys::crocksdb_readoptions_set_iter_start_ts(self.as_mut_ptr(), ptr);
        }
        self
    }

    simple_access! {
        crocksdb_readoptions

        /// It limits the maximum cumulative value size of the keys in batch while
        /// reading through MultiGet. Once the cumulative value size exceeds this
        /// soft limit then all the remaining keys are returned with status Aborted.
        ///
        /// Default: u64::MAX
        value_size_soft_limit: u64

        /// For iterators, RocksDB does auto-readahead on noticing more than two
        /// sequential reads for a table file if user doesn't provide readahead_size.
        /// The readahead starts at 8KB and doubles on every additional read upto
        /// max_auto_readahead_size only when reads are sequential. However at each
        /// level, if iterator moves over next file, readahead_size starts again from
        /// 8KB.
        ///
        /// By enabling this option, RocksDB will do some enhancements for
        /// prefetching the data.
        ///
        /// Default: false
        adaptive_readahead: bool
    }

    /// If true, it will just return keys without indexing value from blob files.
    /// It is mainly used for the scan-delete operation after DeleteFilesInRange.
    /// Cause DeleteFilesInRange may expose old blob index keys, returning key only
    /// avoids referring to missing blob files. Only used for titan engine.
    ///
    /// Default: false
    #[inline]
    pub fn set_key_only(&mut self, key_only: bool) -> &mut Self {
        unsafe {
            tirocks_sys::ctitandb_readoptions_set_key_only(&mut self.raw, key_only);
        }
        self
    }
    // TODO: support table filter.

    #[inline]
    pub(crate) fn get_ptr(&self) -> *const rocksdb_titandb_TitanReadOptions {
        &self.raw
    }
}

impl Drop for ReadOptions {
    fn drop(&mut self) {
        unsafe {
            tirocks_sys::ctitandb_readoptions_inplace_destroy(&mut self.raw);
        }
    }
}

unsafe impl Send for ReadOptions {}
unsafe impl Sync for ReadOptions {}
