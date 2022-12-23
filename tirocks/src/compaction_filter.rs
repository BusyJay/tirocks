// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::{ffi::CStr, ptr, slice};

use libc::{c_char, c_void};
use tirocks_sys::{
    crocksdb_compactionfilterfactory_t, r, rocksdb_CompactionFilter,
    rocksdb_CompactionFilter_Context, rocksdb_CompactionFilter_Decision, rocksdb_Slice, s,
};

use crate::{
    properties::table::builtin::TableProperties, CloneFactory, DefaultFactory, SequenceNumber,
};

pub type ValueType = tirocks_sys::rocksdb_CompactionFilter_ValueType;
pub type TableFileCreationReason = tirocks_sys::rocksdb_TableFileCreationReason;

/// Possible return values for filter.
pub enum Decision<'a> {
    /// Keep the key-value pair.
    Keep,
    /// Remove the key-value pair or merge operand.
    Remove,
    /// Keep the key and change the value/operand.
    ChangeValue(&'a [u8]),
    /// Remove this key-value pair, and also remove
    /// all key-value pairs with key in [key, *skip_until). This range
    /// of keys will be skipped without reading, potentially saving some
    /// IO operations compared to removing the keys one by one.
    ///
    /// *skip_until <= key is treated the same as Decision::kKeep
    /// (since the range [key, *skip_until) is empty).
    ///
    /// Caveats:
    ///   - The keys are skipped even if there are snapshots containing them,
    ///     i.e. values removed by kRemoveAndSkipUntil can disappear from a
    ///     snapshot - beware if you're using TransactionDB or
    ///     DB::GetSnapshot().
    ///   - If value for a key was overwritten or merged into (multiple Put()s
    ///     or Merge()s), and `CompactionFilter` skips this key with
    ///     kRemoveAndSkipUntil, it's possible that it will remove only
    ///     the new value, exposing the old value that was supposed to be
    ///     overwritten.
    ///   - Doesn't work with PlainTableFactory in prefix mode.
    ///   - If you use kRemoveAndSkipUntil for table files created by
    ///     compaction, consider also reducing compaction_readahead_size
    ///     option.
    RemoveAndSKipUntil(&'a [u8]),
}

/// CompactionFilter allows an application to modify/delete a key-value during
/// table file creation.
pub trait CompactionFilter: Send {
    /// Returns a name that identifies this `CompactionFilter`.
    /// The name will be printed to LOG file on start up for diagnosis.
    fn name(&mut self) -> &CStr;

    /// Allows changing value and skipping ranges of keys.
    /// The default implementation uses Filter() and FilterMergeOperand().
    /// If you're overriding this method, no need to override the other two.
    /// `value_type` indicates whether this key-value corresponds to a normal
    /// value (e.g. written with Put())  or a merge operand (written with Merge()).
    ///
    /// Note: If you are using a TransactionDB, it is not recommended to filter
    /// out or modify merge operands (ValueType::kMergeOperand).
    /// If a merge operation is filtered out, TransactionDB may not realize there
    /// is a write conflict and may allow a Transaction to Commit that should have
    /// failed. Instead, it is better to implement any Merge filtering inside the
    /// MergeOperator.
    ///
    /// Note: for kTypeBlobIndex, `key` is internal key instead of user key.
    fn filter(
        &mut self,
        level: i32,
        key: &[u8],
        seqno: SequenceNumber,
        value_type: ValueType,
        existing_value: &[u8],
    ) -> Decision;
}

extern "C" fn name<F: CompactionFilter>(c: *mut c_void) -> *const c_char {
    unsafe {
        let c = &mut *(c as *mut F);
        c.name().as_ptr()
    }
}

extern "C" fn destructor<F: CompactionFilter>(c: *mut c_void) {
    unsafe {
        drop(Box::from_raw(c as *mut F));
    }
}

extern "C" fn filter<F: CompactionFilter>(
    c: *mut c_void,
    level: i32,
    key: rocksdb_Slice,
    seqno: SequenceNumber,
    value_type: ValueType,
    existing_value: rocksdb_Slice,
    new_value: *mut rocksdb_Slice,
    skip_until: *mut rocksdb_Slice,
) -> rocksdb_CompactionFilter_Decision {
    unsafe {
        // Hack: Tenichally, the lifetime of ChangeValue and RemoveAndSkipUntil only lives
        // in this function scope. However, technically, it's impossible for a function to
        // return a value that only valid during its caller. Instead, it will have to store
        // the value inside `c` or in some global variables, which means it should be valid
        // at least before the next call or `c` is destroyed. Both of them are guaranteed not
        // to happen when accessing the value.
        let c = &mut *(c as *mut F);
        match c.filter(level, s(key), seqno, value_type, s(existing_value)) {
            Decision::Keep => rocksdb_CompactionFilter_Decision::kKeep,
            Decision::Remove => rocksdb_CompactionFilter_Decision::kRemove,
            Decision::ChangeValue(v) => {
                *new_value = r(v);
                rocksdb_CompactionFilter_Decision::kChangeValue
            }
            Decision::RemoveAndSKipUntil(v) => {
                *skip_until = r(v);
                rocksdb_CompactionFilter_Decision::kRemoveAndSkipUntil
            }
        }
    }
}

/// Context information for a table file creation.
#[repr(transparent)]
pub struct Context(rocksdb_CompactionFilter_Context);

impl Context {
    #[inline]
    fn as_ptr(&self) -> *const rocksdb_CompactionFilter_Context {
        self as *const _ as _
    }

    /// Whether this table file is created as part of a compaction including all
    /// table files.
    #[inline]
    pub fn is_full_compaction(&self) -> bool {
        unsafe { tirocks_sys::crocksdb_compactionfiltercontext_is_full_compaction(self.as_ptr()) }
    }

    /// Whether this table file is created as part of a compaction requested by
    /// the client.
    #[inline]
    pub fn is_manual_compaction(&self) -> bool {
        unsafe { tirocks_sys::crocksdb_compactionfiltercontext_is_manual_compaction(self.as_ptr()) }
    }

    /// Whether output files are in bottommost level or not.
    #[inline]
    pub fn is_bottommost_level(&self) -> bool {
        unsafe { tirocks_sys::crocksdb_compactionfiltercontext_is_bottommost_level(self.as_ptr()) }
    }

    /// File numbers of all involved SST files.
    #[inline]
    pub fn file_numbers(&self) -> &[u64] {
        let (mut buffer, mut len): (*const u64, usize) = (ptr::null_mut(), 0);
        unsafe {
            tirocks_sys::crocksdb_compactionfiltercontext_file_numbers(
                self.as_ptr(),
                &mut buffer,
                &mut len,
            );
            slice::from_raw_parts(buffer, len)
        }
    }

    /// Properties of involved SST files.
    #[inline]
    pub fn table_properties(&self, offset: usize) -> &TableProperties {
        unsafe {
            let raw = tirocks_sys::crocksdb_compactionfiltercontext_table_properties(
                self.as_ptr(),
                offset,
            );
            TableProperties::from_ptr(raw)
        }
    }

    /// The range of the compaction.
    #[inline]
    pub fn start_key(&self) -> &[u8] {
        unsafe {
            let mut buf = r(&[]);
            tirocks_sys::crocksdb_compactionfiltercontext_start_key(self.as_ptr(), &mut buf);
            s(buf)
        }
    }

    /// The range of the compaction.
    #[inline]
    pub fn end_key(&self) -> &[u8] {
        unsafe {
            let mut buf = r(&[]);
            tirocks_sys::crocksdb_compactionfiltercontext_end_key(self.as_ptr(), &mut buf);
            s(buf)
        }
    }
}

pub trait CompactionFilterFactory: Send + Sync {
    type Filter: CompactionFilter;

    /// Returns whether a thread creating table files for the specified `reason`
    /// should invoke `CreateCompactionFilter()` and pass KVs through the returned
    /// filter.
    fn should_filter_table_file_creation(&self, reason: TableFileCreationReason) -> bool {
        reason == TableFileCreationReason::kCompaction
    }

    fn create_compaction_filter(&self, context: &Context) -> Self::Filter;

    /// Returns a name that identifies this `CompactionFilter` factory.
    fn name(&self) -> &CStr;
}

impl<F: Default + CompactionFilter> CompactionFilterFactory for DefaultFactory<F> {
    type Filter = F;

    #[inline]
    fn create_compaction_filter(&self, _: &Context) -> Self::Filter {
        F::default()
    }

    #[inline]
    fn name(&self) -> &CStr {
        self.c_name()
    }
}

impl<F: Clone + CompactionFilter + Send + Sync> CompactionFilterFactory for CloneFactory<F> {
    type Filter = F;

    #[inline]
    fn create_compaction_filter(&self, _: &Context) -> Self::Filter {
        self.clone()
    }

    #[inline]
    fn name(&self) -> &CStr {
        self.c_name()
    }
}

extern "C" fn factory_name<T: CompactionFilterFactory>(factory: *mut c_void) -> *const c_char {
    unsafe {
        let factory = &mut *(factory as *mut T);
        factory.name().as_ptr()
    }
}

extern "C" fn factory_destruct<T: CompactionFilterFactory>(handle: *mut c_void) {
    unsafe {
        Box::from_raw(handle as *mut T);
    }
}

extern "C" fn create_compaction_filter<T: CompactionFilterFactory>(
    handle: *mut c_void,
    context: *const rocksdb_CompactionFilter_Context,
) -> *mut rocksdb_CompactionFilter {
    unsafe {
        let handle = &*(handle as *mut T);
        let context = &*(context as *const Context);
        let f = handle.create_compaction_filter(context);
        tirocks_sys::crocksdb_compactionfilter_create(
            Box::into_raw(Box::new(f)) as *mut c_void,
            Some(destructor::<T::Filter>),
            Some(filter::<T::Filter>),
            Some(name::<T::Filter>),
        )
    }
}

extern "C" fn should_filter_table_file_creation<T: CompactionFilterFactory>(
    handle: *mut c_void,
    reason: TableFileCreationReason,
) -> bool {
    unsafe {
        let handle = &*(handle as *mut T);
        handle.should_filter_table_file_creation(reason)
    }
}

/// A wrapped factory that can be shared by multiple ColumnFamilies and DBs.
#[derive(Debug)]
pub struct SysCompactionFilterFactory {
    ptr: *mut crocksdb_compactionfilterfactory_t,
}

impl SysCompactionFilterFactory {
    #[inline]
    pub fn new<T: CompactionFilterFactory>(factory: T) -> SysCompactionFilterFactory {
        let ptr = unsafe {
            tirocks_sys::crocksdb_compactionfilterfactory_create(
                Box::into_raw(Box::new(factory)) as *mut c_void,
                Some(factory_destruct::<T>),
                Some(create_compaction_filter::<T>),
                Some(should_filter_table_file_creation::<T>),
                Some(factory_name::<T>),
            )
        };
        SysCompactionFilterFactory { ptr }
    }

    #[inline]
    pub(crate) fn get_ptr(&self) -> *mut crocksdb_compactionfilterfactory_t {
        self.ptr
    }
}

impl Drop for SysCompactionFilterFactory {
    #[inline]
    fn drop(&mut self) {
        unsafe { tirocks_sys::crocksdb_compactionfilterfactory_destroy(self.ptr) }
    }
}
