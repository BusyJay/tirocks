// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::ffi::CStr;

use libc::{c_char, c_void};
use tirocks_sys::{
    crocksdb_sst_partitioner_factory_t, rocksdb_PartitionerRequest, rocksdb_PartitionerResult,
    rocksdb_Slice, rocksdb_SstPartitioner, rocksdb_SstPartitioner_Context, s,
};

pub type PartitionerResult = tirocks_sys::rocksdb_PartitionerResult;

#[repr(transparent)]
pub struct PartitionerRequest(rocksdb_PartitionerRequest);

impl PartitionerRequest {
    #[inline]
    pub fn prev_user_key(&self) -> &[u8] {
        unsafe { s(*self.0.prev_user_key) }
    }

    #[inline]
    pub fn current_user_key(&self) -> &[u8] {
        unsafe { s(*self.0.current_user_key) }
    }

    #[inline]
    pub fn current_output_file_size(&self) -> u64 {
        self.0.current_output_file_size
    }
}

/// A SstPartitioner is a generic pluggable way of defining the partition
/// of SST files. Compaction job will split the SST files on partition boundary
/// to lower the write amplification during SST file promote to higher level.
pub trait SstPartitioner: Send {
    /// It is called for all keys in compaction. When partitioner want to create
    /// new SST file it needs to return true. It means compaction job will finish
    /// current SST file where last key is "prev_user_key" parameter and start new
    /// SST file where first key is "current_user_key". Returns decission if
    /// partition boundary was detected and compaction should create new file.
    fn should_partition(&mut self, req: &PartitionerRequest) -> PartitionerResult;

    /// Called with smallest and largest keys in SST file when compation try to do
    /// trivial move. Returns true is partitioner allows to do trivial move.
    fn can_do_trivial_move(&mut self, smallest_user_key: &[u8], largest_user_key: &[u8]) -> bool;
}

extern "C" fn destructor<P: SstPartitioner>(c: *mut c_void) {
    unsafe {
        drop(Box::from_raw(c as *mut P));
    }
}

extern "C" fn should_partition<P: SstPartitioner>(
    c: *mut c_void,
    request: *const rocksdb_PartitionerRequest,
) -> rocksdb_PartitionerResult {
    unsafe {
        let c = &mut *(c as *mut P);
        c.should_partition(&*(request as *const PartitionerRequest))
    }
}

extern "C" fn can_do_trivial_move<P: SstPartitioner>(
    c: *mut c_void,
    smallest_user_key: rocksdb_Slice,
    largest_user_key: rocksdb_Slice,
) -> bool {
    unsafe {
        let c = &mut *(c as *mut P);
        c.can_do_trivial_move(s(smallest_user_key), s(largest_user_key))
    }
}

#[repr(transparent)]
pub struct Context(rocksdb_SstPartitioner_Context);

impl Context {
    /// Does this compaction run include all data files
    #[inline]
    pub fn is_full_compaction(&self) -> bool {
        self.0.is_full_compaction
    }

    /// Is this compaction requested by the client (true),
    /// or is it occurring as an automatic compaction process
    #[inline]
    pub fn is_manual_compaction(&self) -> bool {
        self.0.is_manual_compaction
    }

    /// Output level for this compaction
    #[inline]
    pub fn output_level(&self) -> i32 {
        self.0.output_level
    }

    /// Smallest key for compaction
    #[inline]
    pub fn smallest_user_key(&self) -> &[u8] {
        unsafe { s(self.0.smallest_user_key) }
    }

    /// Largest key for compaction
    #[inline]
    pub fn largest_user_key(&self) -> &[u8] {
        unsafe { s(self.0.largest_user_key) }
    }
}

pub trait SstPartitionerFactory: Send + Sync {
    type Partitioner: SstPartitioner;

    /// Returns a name that identifies this partitioner factory.
    fn name(&self) -> &CStr;

    fn create_partitioner(&self, context: &Context) -> Self::Partitioner;
}

extern "C" fn factory_name<T: SstPartitionerFactory>(factory: *mut c_void) -> *const c_char {
    unsafe {
        let factory = &mut *(factory as *mut T);
        factory.name().as_ptr()
    }
}

extern "C" fn factory_destruct<T: SstPartitionerFactory>(handle: *mut c_void) {
    unsafe {
        Box::from_raw(handle as *mut T);
    }
}

extern "C" fn create_partitioner<T: SstPartitionerFactory>(
    handle: *mut c_void,
    context: *const rocksdb_SstPartitioner_Context,
) -> *mut rocksdb_SstPartitioner {
    unsafe {
        let handle = &*(handle as *mut T);
        let context = &*(context as *const Context);
        let p = handle.create_partitioner(context);
        tirocks_sys::crocksdb_sst_partitioner_create(
            Box::into_raw(Box::new(p)) as *mut c_void,
            Some(destructor::<T::Partitioner>),
            Some(should_partition::<T::Partitioner>),
            Some(can_do_trivial_move::<T::Partitioner>),
        )
    }
}

/// A wrapped factory that can be shared by multiple ColumnFamilies and DBs.
#[derive(Debug)]
pub struct SysSstParitionerFactory {
    ptr: *mut crocksdb_sst_partitioner_factory_t,
}

impl SysSstParitionerFactory {
    #[inline]
    pub fn new<T: SstPartitionerFactory>(factory: T) -> SysSstParitionerFactory {
        let ptr = unsafe {
            tirocks_sys::crocksdb_sst_partitioner_factory_create(
                Box::into_raw(Box::new(factory)) as *mut c_void,
                Some(factory_destruct::<T>),
                Some(factory_name::<T>),
                Some(create_partitioner::<T>),
            )
        };
        SysSstParitionerFactory { ptr }
    }

    #[inline]
    pub(crate) fn get(&self) -> *mut crocksdb_sst_partitioner_factory_t {
        self.ptr
    }
}

impl Drop for SysSstParitionerFactory {
    #[inline]
    fn drop(&mut self) {
        unsafe { tirocks_sys::crocksdb_sst_partitioner_factory_destroy(self.ptr) }
    }
}
