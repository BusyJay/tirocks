// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::{
    fmt::{self, Debug, Display, Formatter},
    marker::PhantomData,
    ops::{Deref, Index},
    ptr,
    str::{self, Utf8Error},
};

use tirocks_sys::{
    crocksdb_table_properties_collection_iterator_t, r, rocksdb_TablePropertiesCollection, s,
};

use crate::util;

#[repr(transparent)]
pub struct TableProperties(tirocks_sys::rocksdb_TableProperties);

impl TableProperties {
    #[inline]
    pub(crate) unsafe fn from_ptr<'a>(
        ptr: *const tirocks_sys::rocksdb_TableProperties,
    ) -> &'a TableProperties {
        &*(ptr as *const TableProperties)
    }

    /// the total size of all data blocks.
    #[inline]
    pub fn data_size(&self) -> u64 {
        unsafe { tirocks_sys::crocksdb_table_properties_get_data_size(self as *const _ as _) }
    }

    /// the size of index block.
    #[inline]
    pub fn index_size(&self) -> u64 {
        unsafe { tirocks_sys::crocksdb_table_properties_get_index_size(self as *const _ as _) }
    }

    /// Total number of index partitions if kTwoLevelIndexSearch is used
    #[inline]
    pub fn index_partitions(&self) -> u64 {
        unsafe {
            tirocks_sys::crocksdb_table_properties_get_index_partitions(self as *const _ as _)
        }
    }

    /// Size of the top-level index if kTwoLevelIndexSearch is used
    #[inline]
    pub fn top_level_index_size(&self) -> u64 {
        unsafe {
            tirocks_sys::crocksdb_table_properties_get_top_level_index_size(self as *const _ as _)
        }
    }

    /// Whether the index key is user key. Otherwise it includes 8 byte of sequence
    /// number added by internal key format.
    #[inline]
    pub fn index_key_is_user_key(&self) -> u64 {
        unsafe {
            tirocks_sys::crocksdb_table_properties_get_index_key_is_user_key(self as *const _ as _)
        }
    }

    /// Whether delta encoding is used to encode the index values.
    #[inline]
    pub fn index_value_is_delta_encoded(&self) -> u64 {
        unsafe {
            tirocks_sys::crocksdb_table_properties_get_index_value_is_delta_encoded(
                self as *const _ as _,
            )
        }
    }

    /// the size of filter block.
    #[inline]
    pub fn filter_size(&self) -> u64 {
        unsafe { tirocks_sys::crocksdb_table_properties_get_filter_size(self as *const _ as _) }
    }

    /// total raw key size
    #[inline]
    pub fn raw_key_size(&self) -> u64 {
        unsafe { tirocks_sys::crocksdb_table_properties_get_raw_key_size(self as *const _ as _) }
    }

    /// total raw value size
    #[inline]
    pub fn raw_value_size(&self) -> u64 {
        unsafe { tirocks_sys::crocksdb_table_properties_get_raw_value_size(self as *const _ as _) }
    }

    /// the number of blocks in this table
    #[inline]
    pub fn num_data_blocks(&self) -> u64 {
        unsafe { tirocks_sys::crocksdb_table_properties_get_num_data_blocks(self as *const _ as _) }
    }

    /// the number of entries in this table
    #[inline]
    pub fn num_entries(&self) -> u64 {
        unsafe { tirocks_sys::crocksdb_table_properties_get_num_entries(self as *const _ as _) }
    }

    /// the number of deletions in the table
    #[inline]
    pub fn num_deletions(&self) -> u64 {
        unsafe { tirocks_sys::crocksdb_table_properties_get_num_deletions(self as *const _ as _) }
    }

    /// the number of merge operands in the table
    #[inline]
    pub fn num_merge_operands(&self) -> u64 {
        unsafe {
            tirocks_sys::crocksdb_table_properties_get_num_merge_operands(self as *const _ as _)
        }
    }

    /// the number of range deletions in this table
    #[inline]
    pub fn num_range_deletions(&self) -> u64 {
        unsafe {
            tirocks_sys::crocksdb_table_properties_get_num_range_deletions(self as *const _ as _)
        }
    }

    /// format version, reserved for backward compatibility
    #[inline]
    pub fn format_version(&self) -> u64 {
        unsafe { tirocks_sys::crocksdb_table_properties_get_format_version(self as *const _ as _) }
    }

    /// If 0, key is variable length. Otherwise number of bytes for each key.
    #[inline]
    pub fn fixed_key_len(&self) -> u64 {
        unsafe { tirocks_sys::crocksdb_table_properties_get_fixed_key_len(self as *const _ as _) }
    }

    /// ID of column family for this SST file, corresponding to the CF identified
    /// by column_family_name.
    #[inline]
    pub fn column_family_id(&self) -> u64 {
        unsafe {
            tirocks_sys::crocksdb_table_properties_get_column_family_id(self as *const _ as _)
        }
    }

    /// Timestamp of the latest key. 0 means unknown.
    #[inline]
    pub fn creation_time(&self) -> u64 {
        unsafe { tirocks_sys::crocksdb_table_properties_get_creation_time(self as *const _ as _) }
    }

    /// Timestamp of the earliest key. 0 means unknown.
    #[inline]
    pub fn oldest_key_time(&self) -> u64 {
        unsafe { tirocks_sys::crocksdb_table_properties_get_oldest_key_time(self as *const _ as _) }
    }

    /// Actual SST file creation time. 0 means unknown.
    #[inline]
    pub fn file_creation_time(&self) -> u64 {
        unsafe {
            tirocks_sys::crocksdb_table_properties_get_file_creation_time(self as *const _ as _)
        }
    }

    /// Name of the column family with which this SST file is associated.
    /// If column family is unknown, `column_family_name` will be an empty string.
    #[inline]
    pub fn column_family_name(&self) -> std::result::Result<&str, Utf8Error> {
        unsafe {
            let mut buf = r(&[]);
            tirocks_sys::crocksdb_table_properties_get_column_family_name(
                self as *const _ as _,
                &mut buf,
            );
            str::from_utf8(s(buf))
        }
    }

    /// The name of the filter policy used in this table.
    /// If no filter policy is used, `filter_policy_name` will be an empty string.
    #[inline]
    pub fn filter_policy_name(&self) -> std::result::Result<&str, Utf8Error> {
        unsafe {
            let mut buf = r(&[]);
            tirocks_sys::crocksdb_table_properties_get_filter_policy_name(
                self as *const _ as _,
                &mut buf,
            );
            str::from_utf8(s(buf))
        }
    }

    /// The name of the comparator used in this table.
    #[inline]
    pub fn comparator_name(&self) -> std::result::Result<&str, Utf8Error> {
        unsafe {
            let mut buf = r(&[]);
            tirocks_sys::crocksdb_table_properties_get_comparator_name(
                self as *const _ as _,
                &mut buf,
            );
            str::from_utf8(s(buf))
        }
    }

    /// The name of the merge operator used in this table.
    /// If no merge operator is used, `merge_operator_name` will be "nullptr".
    #[inline]
    pub fn merge_operator_name(&self) -> std::result::Result<&str, Utf8Error> {
        unsafe {
            let mut buf = r(&[]);
            tirocks_sys::crocksdb_table_properties_get_merge_operator_name(
                self as *const _ as _,
                &mut buf,
            );
            str::from_utf8(s(buf))
        }
    }

    /// The name of the prefix extractor used in this table
    /// If no prefix extractor is used, `prefix_extractor_name` will be "nullptr".
    #[inline]
    pub fn prefix_extractor_name(&self) -> std::result::Result<&str, Utf8Error> {
        unsafe {
            let mut buf = r(&[]);
            tirocks_sys::crocksdb_table_properties_get_prefix_extractor_name(
                self as *const _ as _,
                &mut buf,
            );
            str::from_utf8(s(buf))
        }
    }

    /// The names of the property collectors factories used in this table
    /// separated by commas
    /// {collector_name[1]},{collector_name[2]},{collector_name[3]} ..
    #[inline]
    pub fn property_collectors_names(&self) -> std::result::Result<&str, Utf8Error> {
        unsafe {
            let mut buf = r(&[]);
            tirocks_sys::crocksdb_table_properties_get_property_collectors_names(
                self as *const _ as _,
                &mut buf,
            );
            str::from_utf8(s(buf))
        }
    }

    /// The compression algo used to compress the SST files.
    #[inline]
    pub fn compression_name(&self) -> std::result::Result<&str, Utf8Error> {
        unsafe {
            let mut buf = r(&[]);
            tirocks_sys::crocksdb_table_properties_get_compression_name(
                self as *const _ as _,
                &mut buf,
            );
            str::from_utf8(s(buf))
        }
    }

    /// Compression options used to compress the SST files.
    #[inline]
    pub fn compression_options(&self) -> std::result::Result<&str, Utf8Error> {
        unsafe {
            let mut buf = r(&[]);
            tirocks_sys::crocksdb_table_properties_get_compression_options(
                self as *const _ as _,
                &mut buf,
            );
            str::from_utf8(s(buf))
        }
    }
}

impl Display for TableProperties {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut res = Ok(());
        let mut receiver = |buf: &[u8]| {
            res = write!(f, "{}", String::from_utf8_lossy(buf));
        };
        unsafe {
            let (ctx, fp) = util::wrap_string_receiver(&mut receiver);
            tirocks_sys::crocksdb_table_properties_to_string(self as *const _ as _, ctx, Some(fp));
        }
        res
    }
}

#[repr(transparent)]
pub struct TablePropertiesCollection(rocksdb_TablePropertiesCollection);

impl TablePropertiesCollection {
    #[inline]
    pub(crate) unsafe fn from_ptr<'a>(
        ptr: *const tirocks_sys::rocksdb_TablePropertiesCollection,
    ) -> &'a Self {
        &*(ptr as *const TablePropertiesCollection)
    }

    #[inline]
    fn as_ptr(&self) -> *const rocksdb_TablePropertiesCollection {
        self as *const _ as _
    }

    #[inline]
    pub fn get(&self, index: impl AsRef<[u8]>) -> Option<&TableProperties> {
        let bytes = index.as_ref();
        unsafe {
            let ptr =
                tirocks_sys::crocksdb_table_properties_collection_get(self.as_ptr(), r(bytes));
            if !ptr.is_null() {
                Some(TableProperties::from_ptr(ptr))
            } else {
                None
            }
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        unsafe { tirocks_sys::crocksdb_table_properties_collection_len(self.as_ptr()) }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<Q: AsRef<[u8]>> Index<Q> for TablePropertiesCollection {
    type Output = TableProperties;

    #[inline]
    fn index(&self, index: Q) -> &TableProperties {
        let key = index.as_ref();
        self.get(key)
            .unwrap_or_else(|| panic!("no entry found for key {:?}", key))
    }
}

impl<'a> IntoIterator for &'a TablePropertiesCollection {
    type Item = (&'a [u8], &'a TableProperties);
    type IntoIter = TablePropertiesCollectionIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        TablePropertiesCollectionIter::new(self)
    }
}

pub struct TablePropertiesCollectionIter<'a> {
    ptr: *mut crocksdb_table_properties_collection_iterator_t,
    _life: PhantomData<&'a TablePropertiesCollection>,
}

impl<'a> Drop for TablePropertiesCollectionIter<'a> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            tirocks_sys::crocksdb_table_properties_collection_iter_destroy(self.ptr);
        }
    }
}

impl<'a> TablePropertiesCollectionIter<'a> {
    #[inline]
    fn new(props: &'a TablePropertiesCollection) -> TablePropertiesCollectionIter<'a> {
        unsafe {
            TablePropertiesCollectionIter {
                ptr: tirocks_sys::crocksdb_table_properties_collection_iter_create(props.as_ptr()),
                _life: PhantomData,
            }
        }
    }
}

impl<'a> Iterator for TablePropertiesCollectionIter<'a> {
    type Item = (&'a [u8], &'a TableProperties);

    #[inline]
    fn next(&mut self) -> Option<(&'a [u8], &'a TableProperties)> {
        unsafe {
            let (mut key, mut val) = (r(&[]), ptr::null());
            if tirocks_sys::crocksdb_table_properties_collection_iter_next(
                self.ptr, &mut key, &mut val,
            ) {
                Some((s(key), TableProperties::from_ptr(val)))
            } else {
                None
            }
        }
    }
}

#[derive(Debug)]
pub struct OwnedTablePropertiesCollection {
    ptr: *mut rocksdb_TablePropertiesCollection,
}

impl Drop for OwnedTablePropertiesCollection {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            tirocks_sys::crocksdb_table_properties_collection_destroy(self.ptr);
        }
    }
}

impl Deref for OwnedTablePropertiesCollection {
    type Target = TablePropertiesCollection;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { TablePropertiesCollection::from_ptr(self.ptr) }
    }
}

impl Default for OwnedTablePropertiesCollection {
    #[inline]
    fn default() -> Self {
        unsafe {
            let ptr = tirocks_sys::crocksdb_table_properties_collection_create();
            Self::from_ptr(ptr)
        }
    }
}

impl OwnedTablePropertiesCollection {
    #[inline]
    pub fn clear(&mut self) {
        unsafe {
            tirocks_sys::crocksdb_table_properties_collection_clear(self.ptr);
        }
    }

    #[inline]
    pub(crate) fn get_ptr(&self) -> *mut rocksdb_TablePropertiesCollection {
        self.ptr
    }

    #[inline]
    pub(crate) unsafe fn from_ptr(
        ptr: *mut rocksdb_TablePropertiesCollection,
    ) -> OwnedTablePropertiesCollection {
        OwnedTablePropertiesCollection { ptr }
    }
}
