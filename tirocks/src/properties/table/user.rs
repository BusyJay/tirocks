// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::{ffi::CStr, marker::PhantomData, ops::Index, ptr};

use libc::{c_char, c_void};
use tirocks_sys::{
    crocksdb_table_properties_collector_factory_t, crocksdb_table_properties_collector_t,
    crocksdb_user_collected_properties_iterator_t, r, rocksdb_Slice, rocksdb_Status,
    rocksdb_UserCollectedProperties, s,
};

use crate::Status;

pub type SequenceNumber = tirocks_sys::rocksdb_SequenceNumber;
pub type EntryType = tirocks_sys::rocksdb_EntryType;

#[derive(Debug)]
pub struct Context {
    column_family_id: u32,
}

impl Context {
    const UNKNOWN_COLUMN_FAMILY_ID: u32 = i32::MAX as u32;

    #[inline]
    pub fn column_family_id(&self) -> u32 {
        self.column_family_id
    }
}

/// Other than basic table properties, each table may also have the user
/// collected properties.
/// The value of the user-collected properties are encoded as raw bytes --
/// users have to interpret these values by themselves.
#[repr(transparent)]
pub struct UserCollectedProperties(tirocks_sys::rocksdb_UserCollectedProperties);

impl UserCollectedProperties {
    #[inline]
    pub(crate) unsafe fn from_ptr<'a>(
        ptr: *const tirocks_sys::rocksdb_UserCollectedProperties,
    ) -> &'a UserCollectedProperties {
        &*(ptr as *const UserCollectedProperties)
    }

    #[inline]
    pub(crate) unsafe fn from_ptr_mut<'a>(
        ptr: *mut tirocks_sys::rocksdb_UserCollectedProperties,
    ) -> &'a mut UserCollectedProperties {
        &mut *(ptr as *mut UserCollectedProperties)
    }

    #[inline]
    pub fn get(&self, index: impl AsRef<[u8]>) -> Option<&[u8]> {
        let bytes = index.as_ref();
        unsafe {
            let mut val = r(&[]);
            let exists = tirocks_sys::crocksdb_user_collected_properties_get(
                self as *const _ as _,
                r(bytes),
                &mut val,
            );
            if exists {
                return Some(s(val));
            }
            None
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        unsafe { tirocks_sys::crocksdb_user_collected_properties_len(self as *const _ as _) }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn add(&mut self, key: &[u8], val: &[u8]) {
        unsafe {
            tirocks_sys::crocksdb_user_collected_properties_add(self as *mut _ as _, r(key), r(val))
        }
    }
}

impl<Q: AsRef<[u8]>> Index<Q> for UserCollectedProperties {
    type Output = [u8];

    #[inline]
    fn index(&self, index: Q) -> &[u8] {
        let key = index.as_ref();
        self.get(key)
            .unwrap_or_else(|| panic!("no entry found for key {:?}", key))
    }
}

impl<'a> IntoIterator for &'a UserCollectedProperties {
    type Item = (&'a [u8], &'a [u8]);
    type IntoIter = UserCollectedPropertiesIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        UserCollectedPropertiesIter::new(self)
    }
}

pub struct UserCollectedPropertiesIter<'a> {
    ptr: *mut crocksdb_user_collected_properties_iterator_t,
    _life: PhantomData<&'a UserCollectedProperties>,
}

impl<'a> Drop for UserCollectedPropertiesIter<'a> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            tirocks_sys::crocksdb_user_collected_properties_iter_destroy(self.ptr);
        }
    }
}

impl<'a> UserCollectedPropertiesIter<'a> {
    #[inline]
    fn new(props: &'a UserCollectedProperties) -> UserCollectedPropertiesIter<'a> {
        unsafe {
            UserCollectedPropertiesIter {
                ptr: tirocks_sys::crocksdb_user_collected_properties_iter_create(
                    props as *const _ as _,
                ),
                _life: PhantomData,
            }
        }
    }
}

impl<'a> Iterator for UserCollectedPropertiesIter<'a> {
    type Item = (&'a [u8], &'a [u8]);

    #[inline]
    fn next(&mut self) -> Option<(&'a [u8], &'a [u8])> {
        unsafe {
            let (mut key, mut val) = (r(&[]), r(&[]));
            if tirocks_sys::crocksdb_user_collected_properties_iter_next(
                self.ptr, &mut key, &mut val,
            ) {
                Some((s(key), s(val)))
            } else {
                None
            }
        }
    }
}

/// `TablePropertiesCollector` provides the mechanism for users to collect
/// their own properties that they are interested in. This class is essentially
/// a collection of callback functions that will be invoked during table
/// building. It is constructed with TablePropertiesCollectorFactory. The methods
/// don't need to be thread-safe, as we will create exactly one
/// TablePropertiesCollector object per table and then call it sequentially
pub trait TablePropertiesCollector {
    /// The name of the properties collector can be used for debugging purpose.
    fn name(&self) -> &CStr;

    /// Will be called when a new key/value pair is inserted into the table.
    fn add(
        &mut self,
        key: &[u8],
        value: &[u8],
        entry_type: EntryType,
        seq: SequenceNumber,
        file_size: u64,
    ) -> Status;

    /// Will be called when a table has already been built and is ready for
    /// writing the properties block.
    fn finish(&mut self, properties: &mut UserCollectedProperties) -> Status;
}

extern "C" fn collector_name<T: TablePropertiesCollector>(handle: *mut c_void) -> *const c_char {
    unsafe {
        let handle = &mut *(handle as *mut T);
        handle.name().as_ptr()
    }
}

extern "C" fn collector_destruct<T: TablePropertiesCollector>(handle: *mut c_void) {
    unsafe {
        drop(Box::from_raw(handle as *mut T));
    }
}

extern "C" fn collector_add<T: TablePropertiesCollector>(
    handle: *mut c_void,
    key: rocksdb_Slice,
    value: rocksdb_Slice,
    entry_type: EntryType,
    seq: SequenceNumber,
    file_size: u64,
    status: *mut rocksdb_Status,
) {
    unsafe {
        let handle = &mut *(handle as *mut T);
        let s = handle.add(s(key), s(value), entry_type, seq, file_size);
        ptr::write(status, s.into_raw());
    }
}

extern "C" fn collector_finish<T: TablePropertiesCollector>(
    handle: *mut c_void,
    props: *mut rocksdb_UserCollectedProperties,
    status: *mut rocksdb_Status,
) {
    unsafe {
        let handle = &mut *(handle as *mut T);
        let s = handle.finish(&mut *(props as *mut UserCollectedProperties));
        ptr::write(status, s.into_raw());
    }
}

/// Constructs `TablePropertiesCollector`.
/// Internals create a new `TablePropertiesCollector` for each new table.
pub trait TablePropertiesCollectorFactory: Send + Sync {
    type Collector: TablePropertiesCollector;

    /// The name of the properties collector can be used for debugging purpose.
    fn name(&self) -> &CStr;

    /// Has to be thread-safe.
    fn create_table_properties_collector(&self, ctx: Context) -> Self::Collector;
}

extern "C" fn factory_name<T: TablePropertiesCollectorFactory>(
    factory: *mut c_void,
) -> *const c_char {
    unsafe {
        let factory = &mut *(factory as *mut T);
        factory.name().as_ptr()
    }
}

extern "C" fn factory_destruct<T: TablePropertiesCollectorFactory>(handle: *mut c_void) {
    unsafe {
        Box::from_raw(handle as *mut T);
    }
}

extern "C" fn create_table_properties_collector<T: TablePropertiesCollectorFactory>(
    handle: *mut c_void,
    column_family_id: u32,
) -> *mut crocksdb_table_properties_collector_t {
    unsafe {
        let handle = &mut *(handle as *mut T);
        let collector = handle.create_table_properties_collector(Context { column_family_id });
        tirocks_sys::crocksdb_table_properties_collector_create(
            Box::into_raw(Box::new(collector)) as *mut c_void,
            Some(collector_name::<T::Collector>),
            Some(collector_destruct::<T::Collector>),
            Some(collector_add::<T::Collector>),
            Some(collector_finish::<T::Collector>),
        )
    }
}

/// A wrapped factory that can be shared by multiple ColumnFamilies and DBs.
#[derive(Debug)]
pub struct SysTablePropertiesCollectorFactory {
    ptr: *mut crocksdb_table_properties_collector_factory_t,
}

impl SysTablePropertiesCollectorFactory {
    #[inline]
    pub fn new<T: TablePropertiesCollectorFactory>(
        factory: T,
    ) -> SysTablePropertiesCollectorFactory {
        let ptr = unsafe {
            tirocks_sys::crocksdb_table_properties_collector_factory_create(
                Box::into_raw(Box::new(factory)) as *mut c_void,
                Some(factory_name::<T>),
                Some(factory_destruct::<T>),
                Some(create_table_properties_collector::<T>),
            )
        };
        SysTablePropertiesCollectorFactory { ptr }
    }

    #[inline]
    pub(crate) fn get(&self) -> *mut crocksdb_table_properties_collector_factory_t {
        self.ptr
    }

    pub fn with_compact_on_deletion(sliding_window_size: usize, deletion_trigger: usize) -> Self {
        unsafe {
            let ptr =
                tirocks_sys::crocksdb_table_properties_collector_factory_create_compact_on_deletion(
                    sliding_window_size,
                    deletion_trigger,
                );
            SysTablePropertiesCollectorFactory { ptr }
        }
    }
}

impl Drop for SysTablePropertiesCollectorFactory {
    #[inline]
    fn drop(&mut self) {
        unsafe { tirocks_sys::crocksdb_table_properties_collector_factory_destroy(self.ptr) }
    }
}
