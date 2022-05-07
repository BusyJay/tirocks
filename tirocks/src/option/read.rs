// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::{
    fmt::{self, Debug, Formatter},
    mem::MaybeUninit,
};

use libc::c_void;
use tirocks_sys::{rocksdb_Snapshot, rocksdb_titandb_TitanReadOptions};

use crate::table_filter::{self, TableFilter};

use super::OwnedSlice;

pub type ReadTier = tirocks_sys::rocksdb_ReadTier;
pub type SequenceNumber = tirocks_sys::rocksdb_SequenceNumber;

#[derive(Default)]
struct ReadOptionsStorage {
    iterate_lower_bound: OwnedSlice,
    iterate_upper_bound: OwnedSlice,
    timestamp: OwnedSlice,
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
        write!(f, "{:?}", self.raw)
    }
}

impl ReadOptions {
    #[inline]
    fn init_slice_store(&mut self) {
        if self.slice_store.is_none() {
            self.slice_store = Some(Box::new(Default::default()));
        }
    }

    /// If "snapshot" is set, read as of the supplied snapshot
    /// (which must belong to the DB that is being read and which must
    /// not have been released).  If "snapshot" is nullptr, use an implicit
    /// snapshot of the state at the beginning of this read operation.
    ///
    /// If set, caller should guarantee the snapshot outlives the read operation.
    #[inline]
    pub(crate) unsafe fn set_snapshot(&mut self, snapshot: *const rocksdb_Snapshot) -> &mut Self {
        self.raw._base.snapshot = snapshot;
        self
    }

    #[inline]
    pub(crate) fn snapshot(&self) -> *const rocksdb_Snapshot {
        self.raw._base.snapshot
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
        self.raw._base.iterate_lower_bound = store.iterate_lower_bound.set_data(lower_bound.into());
        self
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
        self.raw._base.iterate_upper_bound = store.iterate_upper_bound.set_data(upper_bound.into());
        self
    }

    /// RocksDB does auto-readahead for iterators on noticing more than two reads
    /// for a table file. The readahead starts at 8KB and doubles on every
    /// additional read upto 256KB.
    /// This option can help if most of the range scans are large, and if it is
    /// determined that a larger readahead than that enabled by auto-readahead is
    /// needed.
    /// Using a large readahead size (> 2MB) can typically improve the performance
    /// of forward iteration on spinning disks.
    /// Default is 0.
    #[inline]
    pub fn set_readahead_size(&mut self, size: usize) -> &mut Self {
        self.raw._base.readahead_size = size;
        self
    }

    /// A threshold for the number of keys that can be skipped before failing an
    /// iterator seek as incomplete. The default value of 0 should be used to
    /// never fail a request as incomplete, even on skipping too many keys.
    /// Default: 0
    #[inline]
    pub fn set_max_skippable_internal_keys(&mut self, keys: u64) -> &mut Self {
        self.raw._base.max_skippable_internal_keys = keys;
        self
    }

    /// Specify if this read request should process data that ALREADY
    /// resides on a particular cache. If the required data is not
    /// found at the specified cache, then Status::Incomplete is returned.
    /// Default: kReadAllTier
    #[inline]
    pub fn set_read_tier(&mut self, read_tier: ReadTier) -> &mut Self {
        self.raw._base.read_tier = read_tier;
        self
    }

    /// If true, all data read from underlying storage will be
    /// verified against corresponding checksums.
    /// Default: true
    #[inline]
    pub fn set_verify_checksums(&mut self, check: bool) -> &mut Self {
        self.raw._base.verify_checksums = check;
        self
    }

    /// Set whether the "data block"/"index block"" read for this iteration be
    /// placed in block cache.
    ///
    /// Callers may wish to set this field to false for bulk scans. This would
    /// help not to the change eviction order of existing items in the block
    /// cache.
    /// Default: true
    #[inline]
    pub fn set_fill_cache(&mut self, fill: bool) -> &mut Self {
        self.raw._base.fill_cache = fill;
        self
    }

    /// Specify to create a tailing iterator -- a special iterator that has a
    /// view of the complete database (i.e. it can also be used to read newly
    /// added data) and is optimized for sequential reads. It will return records
    /// that were inserted into the database after the creation of the iterator.
    /// Default: false
    #[inline]
    pub fn set_tailing(&mut self, tailing: bool) -> &mut Self {
        self.raw._base.tailing = tailing;
        self
    }

    /// Enable a total order seek regardless of index format (e.g. hash index)
    /// used in the table. Some table format (e.g. plain table) may not support
    /// this option.
    /// If true when calling Get(), we also skip prefix bloom when reading from
    /// block based table. It provides a way to read existing data after
    /// changing implementation of prefix extractor.
    #[inline]
    pub fn set_total_order_seek(&mut self, total_order_seek: bool) -> &mut Self {
        self.raw._base.total_order_seek = total_order_seek;
        self
    }

    /// Enforce that the iterator only iterates over the same prefix as the seek.
    /// This option is effective only for prefix seeks, i.e. prefix_extractor is
    /// non-null for the column family and total_order_seek is false.  Unlike
    /// iterate_upper_bound, prefix_same_as_start only works within a prefix
    /// but in both directions.
    /// Default: false
    #[inline]
    pub fn set_prefix_same_as_start(&mut self, same_as_start: bool) -> &mut Self {
        self.raw._base.prefix_same_as_start = same_as_start;
        self
    }

    /// Keep the blocks loaded by the iterator pinned in memory as long as the
    /// iterator is not deleted, If used when reading from tables created with
    /// BlockBasedTableOptions::use_delta_encoding = false,
    /// Iterator's property "rocksdb.iterator.is-key-pinned" is guaranteed to
    /// return 1.
    /// Default: false
    #[inline]
    pub fn set_pin_data(&mut self, pin_data: bool) -> &mut Self {
        self.raw._base.pin_data = pin_data;
        self
    }

    /// If true, when PurgeObsoleteFile is called in CleanupIteratorState, we
    /// schedule a background job in the flush job queue and delete obsolete files
    /// in background.
    /// Default: false
    #[inline]
    pub fn set_background_purge_on_iterator_cleanup(
        &mut self,
        background_purge: bool,
    ) -> &mut Self {
        self.raw._base.background_purge_on_iterator_cleanup = background_purge;
        self
    }

    /// If true, keys deleted using the DeleteRange() API will be visible to
    /// readers until they are naturally deleted during compaction. This improves
    /// read performance in DBs with many range deletions.
    /// Default: false
    #[inline]
    pub fn set_ignore_range_deletions(&mut self, ignore: bool) -> &mut Self {
        self.raw._base.ignore_range_deletions = ignore;
        self
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
                &mut self.raw._base,
                filter as *mut c_void,
                Some(table_filter::filter::<T>),
                Some(table_filter::destroy::<T>),
            )
        }
        self
    }

    /// Needed to support differential snapshots. Has 2 effects:
    /// 1) Iterator will skip all internal keys with seq < iter_start_seqnum
    /// 2) if this param > 0 iterator will return INTERNAL keys instead of
    ///    user keys; e.g. return tombstones as well.
    /// Default: 0 (don't filter by seq, return user keys)
    #[inline]
    pub fn set_iter_start_sequence_number(&mut self, seq: SequenceNumber) -> &mut Self {
        self.raw._base.iter_start_seqnum = seq;
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
        self.raw._base.timestamp = store.timestamp.set_data(timestamp.into());
        self
    }

    /// If true, it will just return keys without indexing value from blob files.
    /// It is mainly used for the scan-delete operation after DeleteFilesInRange.
    /// Cause DeleteFilesInRange may expose old blob index keys, returning key only
    /// avoids referring to missing blob files. Only used for titan engine.
    ///
    /// Default: false
    #[inline]
    pub fn set_key_only(&mut self, key_only: bool) -> &mut Self {
        self.raw.key_only = key_only;
        self
    }
    // TODO: support table filter.

    #[inline]
    pub(crate) fn get(&self) -> *const rocksdb_titandb_TitanReadOptions {
        &self.raw
    }
}
