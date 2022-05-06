// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use libc::c_void;
use tirocks_sys::{crocksdb_map_property_t, r, s};

use crate::{
    table_properties::builtin::{OwnedTablePropertiesCollection, TablePropertiesCollection},
    util::{check_status, range_to_rocks},
    RawDb, Result, Status,
};

use super::cf::RawColumnFamilyHandle;
use std::{ffi::CStr, io::Write, ops::Index};

pub trait Property {
    fn key(&self) -> &[u8];
}

pub trait IntProperty: Property {}

pub trait MapProperty: Property {}

macro_rules! define {
    ($(#[$outer:meta])* $name:ident, $value:ident) => {
        $(#[$outer])*
        pub struct $name;

        impl Property for $name {
            #[inline]
            fn key(&self) -> &[u8] {
                unsafe { s(tirocks_sys::$value) }
            }
        }
    };
    (int $(#[$outer:meta])* $name:ident, $value:ident) => {
        define!($(#[$outer])* $name, $value);
        impl IntProperty for $name {}
    };
    (map $(#[$outer:meta])* $name:ident, $value:ident) => {
        define!($(#[$outer])* $name, $value);
        impl MapProperty for $name {}
    }
}

/// The number of files at specified level.
pub struct PropNumFilesAtLevel {
    key: Vec<u8>,
}

impl PropNumFilesAtLevel {
    pub fn new(level: usize) -> Self {
        let prefix = unsafe { s(tirocks_sys::crocksdb_property_name_num_files_at_level_prefix) };
        let mut key = Vec::with_capacity(prefix.len() + 1);
        key.extend_from_slice(prefix);
        write!(key, "{}", level).unwrap();
        Self { key }
    }
}

impl Property for PropNumFilesAtLevel {
    #[inline]
    fn key(&self) -> &[u8] {
        &self.key
    }
}

impl IntProperty for PropNumFilesAtLevel {}

/// The compression ratio of data at specified level. Here, compression ratio is defined
/// as uncompressed data size / compressed file size. Returns "-1.0" if no open files.
pub struct PropCompressionRatioAtLevel {
    key: Vec<u8>,
}

impl PropCompressionRatioAtLevel {
    pub fn new(level: usize) -> Self {
        let prefix =
            unsafe { s(tirocks_sys::crocksdb_property_name_compression_ratio_at_level_prefix) };
        let mut key = Vec::with_capacity(prefix.len() + 1);
        key.extend_from_slice(prefix);
        write!(key, "{}", level).unwrap();
        Self { key }
    }
}

impl Property for PropCompressionRatioAtLevel {
    #[inline]
    fn key(&self) -> &[u8] {
        &self.key
    }
}

define! {
    /// A multi-line string containing the data described by kCFStats followed by the data
    /// described by kDBStats.
    PropStats, crocksdb_property_name_stats
}

define! {
    /// A multi-line string summarizing current SST files.
    PropSsTables, crocksdb_property_name_ss_tables
}

define! {
    /// Combined with [`PropCfStatsNoFileHistogram`] and [`PropCfFileHistogram`].
    PropCfStats, crocksdb_property_name_cf_stats
}

define! {
    map
    /// A multi-line string with general columm family stats per-level over db's lifetime
    /// ("L<n>"), aggregated over db's lifetime ("Sum"), and aggregated over the interval
    /// since the last retrieval ("Int").
    ///
    /// It could also be used to return the stats in the format of the map. In this case
    /// there will a pair of string to array of double for each level as well as for "Sum".
    /// "Int" stats will not be affected when this form of stats are retrieved.
    PropCfStatsNoFileHistogram, crocksdb_property_name_cf_stats_no_file_histogram
}

define! {
    /// How many file reads to every level, as well as the histogram of latency of single
    /// requests.
    PropCfFileHistogram, crocksdb_property_name_cf_file_histogram
}

define! {
    /// A multi-line string with general database stats, both cumulative (over the db's lifetime)
    /// and interval (since the last retrieval of kDBStats).
    PropDbStats, crocksdb_property_name_db_stats
}

define! {
    /// Multi-line string containing the number of files per level and total size of each level
    /// (MB).
    PropLevelStats, crocksdb_property_name_level_stats
}

define! {
    int
    /// Number of immutable memtables that have not yet been flushed.
    PropNumImmutableMemTable, crocksdb_property_name_num_immutable_mem_table
}

define! {
    int
    /// Number of immutable memtables that have already been flushed.
    PropNumImmutableMemTableFlushed, crocksdb_property_name_num_immutable_mem_table_flushed
}

define! {
    int
    /// 1 if a memtable flush is pending; 0 otherwise.
    PropMemTableFlushPending, crocksdb_property_name_mem_table_flush_pending
}

define! {
    int
    /// the number of currently running flushes.
    PropNumRunningFlushes, crocksdb_property_name_num_running_flushes
}

define! {
    int
    /// 1 if at least one compaction is pending; 0 otherwise.
    PropCompactionPending, crocksdb_property_name_compaction_pending
}

define! {
    int
    /// The number of currently running compactions.
    PropNumRunningCompactions, crocksdb_property_name_num_running_compactions
}

define! {
    int
    /// Accumulated number of background errors.
    PropBackgroundErrors, crocksdb_property_name_background_errors
}

define! {
    int
    /// Approximate size of active memtable (bytes).
    PropCurSizeActiveMemTable, crocksdb_property_name_cur_size_active_mem_table
}

define! {
    int
    /// Approximate size of active and unflushed immutable memtables (bytes).
    PropCurSizeAllMemTables, crocksdb_property_name_cur_size_all_mem_tables
}

define! {
    int
    /// Approximate size of active, unflushed immutable, and pinned immutable memtables (bytes).
    PropSizeAllMemTables, crocksdb_property_name_size_all_mem_tables
}

define! {
    int
    /// Total number of entries in the active memtable.
    PropNumEntriesActiveMemTable, crocksdb_property_name_num_entries_active_mem_table
}

define! {
    int
    /// Total number of entries in the unflushed immutable memtables.
    PropNumEntriesImmMemTables, crocksdb_property_name_num_entries_imm_mem_tables
}

define! {
    int
    /// Total number of delete entries in the active memtable.
    PropNumDeletesActiveMemTable, crocksdb_property_name_num_deletes_active_mem_table
}

define! {
    int
    /// Total number of delete entries in the unflushed immutable memtables.
    PropNumDeletesImmMemTables, crocksdb_property_name_num_deletes_imm_mem_tables
}

define! {
    int
    /// Estimated number of total keys in the active and unflushed immutable memtables and
    /// storage.
    PropEstimateNumKeys, crocksdb_property_name_estimate_num_keys
}

define! {
    int
    /// Estimated memory used for reading SST tables, excluding memory used in block cache
    /// (e.g., filter and index blocks).
    PropEstimateTableReadersMem, crocksdb_property_name_estimate_table_readers_mem
}

define! {
    int
    /// 0 if deletion of obsolete files is enabled; otherwise, returns a non-zero number.
    PropIsFileDeletionsEnabled, crocksdb_property_name_is_file_deletions_enabled
}

define! {
    int
    /// Number of unreleased snapshots of the database.
    PropNumSnapshots, crocksdb_property_name_num_snapshots
}

define! {
    int
    /// Number representing unix timestamp of oldest unreleased snapshot.
    PropOldestSnapshotTime, crocksdb_property_name_oldest_snapshot_time
}

define! {
    int
    /// Number representing sequence number of oldest unreleased snapshot.
    PropOldestSnapshotSequence, crocksdb_property_name_oldest_snapshot_sequence
}

define! {
    int
    /// Number of live versions. `Version` is a rocksdb internal data structure. More
    /// live versions often mean more SST files are held from being deleted, by iterators
    /// or unfinished compactions.
    PropNumLiveVersions, crocksdb_property_name_num_live_versions
}

define! {
    int
    /// Number of current LSM version. It is a uint64_t integer number, incremented after
    /// there is any change to the LSM tree. The number is not preserved after restarting
    /// the DB. After DB restart, it will start from 0 again.
    PropCurrentSuperVersionNumber, crocksdb_property_name_current_super_version_number
}

define! {
    int
    /// An estimate of the amount of live data in bytes.
    PropEstimateLiveDataSize, crocksdb_property_name_estimate_live_data_size
}

define! {
    int
    /// The minimum log number of the log files that should be kept.
    PropMinLogNumberToKeep, crocksdb_property_name_min_log_number_to_keep
}

define! {
    int
    /// The minimum file number for an obsolete SST to be kept. The max value of `u64`
    /// will be returned if all obsolete files can be deleted.
    PropMinObsoleteSstNumberToKeep, crocksdb_property_name_min_obsolete_sst_number_to_keep
}

define! {
    int
    /// Total size (bytes) of all SST files.
    ///
    /// WARNING: may slow down online queries if there are too many files.
    PropTotalSstFilesSize, crocksdb_property_name_total_sst_files_size
}

define! {
    int
    /// Total size (bytes) of all SST files belong to the latest LSM tree.
    PropLiveSstFilesSize, crocksdb_property_name_live_sst_files_size
}

define! {
    int
    /// Number of level to which L0 data will be compacted.
    PropBaseLevel, crocksdb_property_name_base_level
}

define! {
    int
    /// Estimated total number of bytes compaction needs to rewrite to get all levels down
    /// to under target size. Not valid for other compactions than level-based.
    PropEstimatePendingCompactionBytes, crocksdb_property_name_estimate_pending_compaction_bytes
}

define! {
    /// A string representation of the aggregated table properties of the target column family.
    PropAggregatedTableProperties, crocksdb_property_name_aggregated_table_properties
}

/// Same as the [`PropAggregatedTableProperties`] but only returns the aggregated table
/// properties of the pecified level at the target column family.
pub struct PropAggregatedTablePropertiesAtLevel {
    key: Vec<u8>,
}

impl PropAggregatedTablePropertiesAtLevel {
    pub fn new(level: usize) -> Self {
        let prefix =
            unsafe { s(tirocks_sys::crocksdb_property_name_aggregated_table_properties_at_level) };
        let mut key = Vec::with_capacity(prefix.len() + 1);
        key.extend_from_slice(prefix);
        write!(key, "{}", level).unwrap();
        Self { key }
    }
}

impl Property for PropAggregatedTablePropertiesAtLevel {
    #[inline]
    fn key(&self) -> &[u8] {
        &self.key
    }
}

define! {
    int
    /// The current actual delayed write rate. 0 means no delay.
    PropActualDelayedWriteRate, crocksdb_property_name_actual_delayed_write_rate
}

define! {
    int
    /// 1 if write has been stopped.
    PropIsWriteStopped, crocksdb_property_name_is_write_stopped
}

define! {
    int
    /// 1 if write has been stalled.
    PropIsWriteStalled, crocksdb_property_name_is_write_stalled
}

define! {
    int
    /// An estimation of oldest key timestamp in the DB. Currently only available for
    /// FIFO compaction with compaction_options_fifo.allow_compaction = false.
    PropEstimateOldestKeyTime, crocksdb_property_name_estimate_oldest_key_time
}

define! {
    int
    /// Block cache capacity.
    PropBlockCacheCapacity, crocksdb_property_name_block_cache_capacity
}

define! {
    int
    /// The memory size for the entries residing in block cache.
    PropBlockCacheUsage, crocksdb_property_name_block_cache_usage
}

define! {
    int
    /// The memory size for the entries being pinned.
    PropBlockCachePinnedUsage, crocksdb_property_name_block_cache_pinned_usage
}

define! {
    /// Multi-line string of options.statistics.
    PropOptionsStatistics, crocksdb_property_name_options_statistics
}

pub struct PropertyMap {
    ptr: *mut crocksdb_map_property_t,
}

impl Drop for PropertyMap {
    #[inline]
    fn drop(&mut self) {
        unsafe { tirocks_sys::crocksdb_destroy_map_property(self.ptr) }
    }
}

impl Default for PropertyMap {
    #[inline]
    fn default() -> Self {
        let ptr = unsafe { tirocks_sys::crocksdb_create_map_property() };
        PropertyMap { ptr }
    }
}

impl PropertyMap {
    #[inline]
    pub fn get(&self, index: impl AsRef<[u8]>) -> Option<&[u8]> {
        let key = index.as_ref();
        unsafe {
            let mut buf = r(&[]);
            let found = tirocks_sys::crocksdb_map_property_value(self.ptr, r(key), &mut buf);
            if found {
                Some(s(buf))
            } else {
                None
            }
        }
    }

    #[inline]
    pub fn get_u64(&self, index: impl AsRef<[u8]>) -> Option<u64> {
        let key = index.as_ref();
        unsafe {
            let mut res = 0;
            let found = tirocks_sys::crocksdb_map_property_int_value(self.ptr, r(key), &mut res);
            if found {
                Some(res)
            } else {
                None
            }
        }
    }
}

impl<Q: AsRef<[u8]>> Index<Q> for PropertyMap {
    type Output = [u8];

    #[inline]
    fn index(&self, index: Q) -> &[u8] {
        let key = index.as_ref();
        self.get(key)
            .unwrap_or_else(|| panic!("no entry found for key {:?}", key))
    }
}

impl RawDb {
    /// DB implementations can export properties about their state via this method. If "prop"
    /// is a valid property understood by this DB implementation (see struct inherits `Property`
    /// trait for valid options), returns true. Otherwise, returns false.
    #[inline]
    pub fn property(&self, cf: &RawColumnFamilyHandle, prop: &impl Property) -> Option<Vec<u8>> {
        let key = prop.key();
        unsafe {
            let ptr =
                tirocks_sys::crocksdb_property_value_cf(self.as_ptr(), cf.as_mut_ptr(), r(key));
            if !ptr.is_null() {
                let res = Some(CStr::from_ptr(ptr).to_bytes().to_vec());
                libc::free(ptr as *mut c_void);
                res
            } else {
                None
            }
        }
    }

    /// Similar to [`property`], but only works for a subset of properties whose return value
    /// is an integer.
    #[inline]
    pub fn property_u64(&self, cf: &RawColumnFamilyHandle, prop: &impl IntProperty) -> Option<u64> {
        let key = prop.key();
        unsafe {
            let mut value = 0;
            let f = tirocks_sys::crocksdb_property_int_value_cf(
                self.as_ptr(),
                cf.as_mut_ptr(),
                r(key),
                &mut value,
            );
            if f {
                Some(value)
            } else {
                None
            }
        }
    }

    /// Same as [`property_u64`], but this one returns the aggregated u64 property from all column
    /// families.
    #[inline]
    pub fn property_aggregated_u64(&self, prop: &impl IntProperty) -> Option<u64> {
        let key = prop.key();
        unsafe {
            let mut value = 0;
            let f = tirocks_sys::crocksdb_property_aggregated_int_value(
                self.as_ptr(),
                r(key),
                &mut value,
            );
            if f {
                Some(value)
            } else {
                None
            }
        }
    }

    /// Similar to [`property`], but only works for a subset of properties whose return value
    /// is a map.
    #[inline]
    pub fn property_map(
        &self,
        cf: &RawColumnFamilyHandle,
        prop: &impl MapProperty,
    ) -> Option<PropertyMap> {
        let mut value = PropertyMap::default();
        if self.property_map_to(cf, prop, &mut value) {
            Some(value)
        } else {
            None
        }
    }

    /// Similar to [`property_map`], but allow reusing existing map.
    #[inline]
    pub fn property_map_to(
        &self,
        cf: &RawColumnFamilyHandle,
        prop: &impl MapProperty,
        value: &mut PropertyMap,
    ) -> bool {
        let key = prop.key();
        unsafe {
            tirocks_sys::crocksdb_get_map_property_cf(
                self.as_ptr(),
                cf.as_mut_ptr(),
                r(key),
                value.ptr,
            )
        }
    }

    #[inline]
    pub fn properties_of_all_tables(
        &self,
        cf: &RawColumnFamilyHandle,
        c: &mut OwnedTablePropertiesCollection,
    ) -> Result<()> {
        unsafe {
            let mut s = Status::default();
            tirocks_sys::crocksdb_get_properties_of_all_tables_cf(
                self.as_ptr(),
                cf.as_mut_ptr(),
                c.get(),
                s.as_mut_ptr(),
            );
            check_status!(s)
        }
    }

    #[inline]
    pub fn properties_of_tables_in_range(
        &self,
        cf: &RawColumnFamilyHandle,
        ranges: &[(impl AsRef<[u8]>, impl AsRef<[u8]>)],
        c: &mut OwnedTablePropertiesCollection,
    ) -> Result<()> {
        unsafe {
            let mut s = Status::default();
            let ranges: Vec<_> = ranges
                .into_iter()
                .map(|(s, e)| range_to_rocks(s, e))
                .collect();
            tirocks_sys::crocksdb_get_properties_of_tables_in_range(
                self.as_ptr(),
                cf.as_mut_ptr(),
                c.get(),
                ranges.len() as i32,
                ranges.as_ptr(),
                s.as_mut_ptr(),
            );
            check_status!(s)
        }
    }
}
