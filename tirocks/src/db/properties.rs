// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use tirocks_sys::r;

use crate::{
    properties::{
        table::builtin::OwnedTablePropertiesCollection, IntProperty, MapProperty, Property,
        PropertyMap,
    },
    util::{self, check_status, range_to_rocks},
    RawDb, Result, Status,
};

use super::cf::RawColumnFamilyHandle;

impl RawDb {
    /// DB implementations can export properties about their state via this method. If "prop"
    /// is a valid property understood by this DB implementation (see struct inherits `Property`
    /// trait for valid options), returns true. Otherwise, returns false.
    #[inline]
    pub fn property(&self, cf: &RawColumnFamilyHandle, prop: &impl Property) -> Option<Vec<u8>> {
        let key = prop.key();
        unsafe {
            let mut content = None;
            let mut handle = |v: &[u8]| content = Some(v.to_vec());
            let (ctx, fp) = util::wrap_string_receiver(&mut handle);
            tirocks_sys::crocksdb_property_value_cf(self.as_ptr(), cf.get(), r(key), ctx, Some(fp));
            content
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
                cf.get(),
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
                cf.get(),
                r(key),
                value.as_mut_ptr(),
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
                cf.get(),
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
                cf.get(),
                c.get(),
                ranges.len() as i32,
                ranges.as_ptr(),
                s.as_mut_ptr(),
            );
            check_status!(s)
        }
    }
}
