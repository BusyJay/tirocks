// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use tirocks_sys::rocksdb_PlainTableOptions;

pub type EncodingType = tirocks_sys::rocksdb_EncodingType;

#[repr(transparent)]
pub struct PlainTableOptions(rocksdb_PlainTableOptions);

impl PlainTableOptions {
    /// plain table has optimization for fix-sized keys, which can
    /// be specified via user_key_len.  Alternatively, you can pass
    /// `kPlainTableVariableLength` if your keys have variable
    /// lengths.
    #[inline]
    pub fn set_user_key_len(&mut self, len: u32) -> &mut Self {
        self.0.user_key_len = len;
        self
    }

    /// the number of bits used for bloom filer per prefix.
    /// You may disable it by passing a zero.
    #[inline]
    pub fn set_bloom_bits_per_key(&mut self, bits: i32) -> &mut Self {
        self.0.bloom_bits_per_key = bits;
        self
    }

    /// the desired utilization of the hash table used for
    /// prefix hashing.
    /// hash_table_ratio = number of prefixes / #buckets in the
    /// hash table
    #[inline]
    pub fn set_hash_table_ratio(&mut self, ratio: f64) -> &mut Self {
        self.0.hash_table_ratio = ratio;
        self
    }

    /// inside each prefix, need to build one index record for
    /// how many keys for binary search inside each hash bucket.
    /// For encoding type kPrefix, the value will be used when
    /// writing to determine an interval to rewrite the full
    /// key. It will also be used as a suggestion and satisfied
    /// when possible.
    #[inline]
    pub fn set_index_sparseness(&mut self, sparseness: usize) -> &mut Self {
        self.0.index_sparseness = sparseness;
        self
    }

    /// if <=0, allocate hash indexes and blooms from malloc.
    /// Otherwise from huge page TLB. The user needs to
    /// reserve huge pages for it to be allocated, like:
    ///     sysctl -w vm.nr_hugepages=20
    /// See linux doc Documentation/vm/hugetlbpage.txt
    #[inline]
    pub fn set_huge_page_table_size(&mut self, size: usize) -> &mut Self {
        self.0.huge_page_tlb_size = size;
        self
    }

    /// how to encode the keys. See enum EncodingType above for
    /// the choices. The value will determine how to encode keys
    /// when writing to a new SST file. This value will be stored
    /// inside the SST file which will be used when reading from
    /// the file, which makes it possible for users to choose
    /// different encoding type when reopening a DB. Files with
    /// different encoding types can co-exist in the same DB and
    /// can be read.
    #[inline]
    pub fn set_encoding_type(&mut self, ty: EncodingType) -> &mut Self {
        self.0.encoding_type = ty;
        self
    }

    /// mode for reading the whole file one record by one without
    /// using the index.
    #[inline]
    pub fn set_full_scan_mode(&mut self, enable: bool) -> &mut Self {
        self.0.full_scan_mode = enable;
        self
    }

    /// compute plain table index and bloom filter during
    /// file building and store it in file. When reading
    /// file, index will be mmaped instead of recomputation.
    #[inline]
    pub fn set_store_index_in_file(&mut self, store: bool) -> &mut Self {
        self.0.store_index_in_file = store;
        self
    }

    #[inline]
    pub(crate) fn get(&self) -> *const rocksdb_PlainTableOptions {
        &self.0
    }
}
