// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use tirocks_sys::{r, rocksdb_Range};

use crate::{util::check_status, RawDb, Result, Status};

use super::cf::RawColumnFamilyHandle;

pub type SizeApproximationOptions = tirocks_sys::rocksdb_SizeApproximationOptions;

unsafe fn range_to_rocks(start: &impl AsRef<[u8]>, end: &impl AsRef<[u8]>) -> rocksdb_Range {
    rocksdb_Range {
        start: r(start.as_ref()),
        limit: r(end.as_ref()),
    }
}

impl RawDb {
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
                cf.as_mut_ptr(),
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
                cf.as_mut_ptr(),
                &raw_range,
                &mut count,
                &mut size,
            );
            (count, size)
        }
    }
}
