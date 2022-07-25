// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::{borrow::Cow, ffi::CStr, marker::PhantomData, os::raw::c_char, ptr, slice};

use libc::c_void;
use tirocks_sys::{
    crocksdb_mergeoperator_t, r, rocksdb_MergeOperator_MergeOperationInput,
    rocksdb_MergeOperator_MergeOperationOutput, rocksdb_Slice, s,
};

use crate::util;

#[repr(transparent)]
pub struct MergeOperationInput(rocksdb_MergeOperator_MergeOperationInput);

impl MergeOperationInput {
    fn as_ptr(&self) -> *const rocksdb_MergeOperator_MergeOperationInput {
        self as *const _ as _
    }

    /// The key associated with the merge operation.
    #[inline]
    pub fn key(&self) -> &[u8] {
        unsafe {
            let mut buf = r(&[]);
            tirocks_sys::crocksdb_mergeoperationinput_key(self.as_ptr(), &mut buf);
            s(buf)
        }
    }

    /// The existing value of the current key, None means that the value doesn't exist.
    #[inline]
    pub fn existing_value(&self) -> Option<&[u8]> {
        unsafe {
            let mut v = ptr::null();
            tirocks_sys::crocksdb_mergeoperationinput_existing_value(self.as_ptr(), &mut v);
            if !v.is_null() {
                Some(s(*v))
            } else {
                None
            }
        }
    }

    /// A list of operands to apply.
    #[inline]
    pub fn operand_list(&self) -> Cow<[&[u8]]> {
        unsafe {
            let mut p = ptr::null();
            let mut size = 0;
            tirocks_sys::crocksdb_mergeoperationinput_operand_list(
                self.as_ptr(),
                &mut p,
                &mut size,
            );
            let arr = slice::from_raw_parts(p, size);
            util::rocks_slice_to_array(arr)
        }
    }
}

#[repr(transparent)]
pub struct MergeOperationOutput(rocksdb_MergeOperator_MergeOperationOutput);

impl MergeOperationOutput {
    fn as_mut_ptr(&mut self) -> *mut rocksdb_MergeOperator_MergeOperationOutput {
        self as *mut _ as _
    }

    /// Client is responsible for filling the merge result here.
    #[inline]
    pub fn set_new_value(&mut self, value: &[u8]) -> &mut Self {
        unsafe {
            tirocks_sys::crocksdb_mergeoperationoutput_set_new_value(self.as_mut_ptr(), r(value));
        }
        self
    }

    #[inline]
    pub fn append_new_value(&mut self, value: &[u8]) -> &mut Self {
        unsafe {
            tirocks_sys::crocksdb_mergeoperationoutput_append_new_value(
                self.as_mut_ptr(),
                r(value),
            );
        }
        self
    }

    /// If the merge result is one of the existing operands client can set this field to
    /// the operand instead of using new_value.
    #[inline]
    pub fn set_existing_operand(&mut self, input: &MergeOperationInput, pos: usize) -> &mut Self {
        unsafe {
            tirocks_sys::crocksdb_mergeoperationoutput_set_existing_operand(
                self.as_mut_ptr(),
                input.as_ptr(),
                pos,
            );
        }
        self
    }

    /// If the merge result is existing_value, client can set this field to existing_value instead
    /// of using new_value.
    #[inline]
    pub fn set_existing_operand_to_value(&mut self, input: &MergeOperationInput) -> &mut Self {
        unsafe {
            tirocks_sys::crocksdb_mergeoperationoutput_set_existing_operand_to_value(
                self.as_mut_ptr(),
                input.as_ptr(),
            )
        }
        self
    }
}

type AppendCb = Option<unsafe extern "C" fn(*mut c_void, rocksdb_Slice)>;

unsafe extern "C" fn append_to_output(ctx: *mut c_void, v: rocksdb_Slice) {
    let c = &mut *(ctx as *mut MergeOperationOutput);
    c.append_new_value(s(v));
}

/// A struct to collect slice.
pub struct NewValue<'a> {
    ctx: *mut c_void,
    func: AppendCb,
    lifetime: PhantomData<&'a ()>,
}

impl<'a> NewValue<'a> {
    /// Append the slice.
    #[inline]
    pub fn append(&mut self, v: &[u8]) {
        unsafe { self.func.unwrap()(self.ctx, r(v)) }
    }

    /// Contruct a struct to collect slice.
    #[inline]
    pub fn with_output(output: &mut MergeOperationOutput) -> NewValue {
        NewValue {
            ctx: output as *mut _ as _,
            func: Some(append_to_output),
            lifetime: PhantomData,
        }
    }
}

/// The Merge Operator
///
/// Essentially, a MergeOperator specifies the SEMANTICS of a merge, which only
/// client knows. It could be numeric addition, list append, string
/// concatenation, edit data structure, ... , anything.
/// The library, on the other hand, is concerned with the exercise of this
/// interface, at the right time (during get, iteration, compaction...)
///
/// To use merge, the client needs to provide an object implementing one of
/// the following interfaces:
///  a) AssociativeMergeOperator - for most simple semantics (always take
///    two values, and merge them into one value, which is then put back
///    into rocksdb); numeric addition and string concatenation are examples;
///
///  b) MergeOperator - the generic class for all the more abstract / complex
///    operations; one method (FullMergeV2) to merge a Put/Delete value with a
///    merge operand; and another method (PartialMerge) that merges multiple
///    operands together. this is especially useful if your key values have
///    complex structures but you would still like to support client-specific
///    incremental updates.
///
/// AssociativeMergeOperator is simpler to implement. MergeOperator is simply
/// more powerful.
///
/// Refer to rocksdb-merge wiki for more details and example implementations.
pub trait MergeOperator: Send + Sync {
    /// The name of the MergeOperator. Used to check for MergeOperator
    /// mismatches (i.e., a DB created with one MergeOperator is
    /// accessed using a different MergeOperator)
    fn name(&self) -> &CStr;

    /// This function applies a stack of merge operands in chrionological order
    /// on top of an existing value. There are two ways in which this method is
    /// being used:
    /// a) During Get() operation, it used to calculate the final value of a key
    /// b) During compaction, in order to collapse some operands with the based
    ///    value.
    ///
    /// Note: The name of the method is somewhat misleading, as both in the cases
    /// of Get() or compaction it may be called on a subset of operands:
    /// ```text
    /// K:    0    +1    +2    +7    +4     +5      2     +1     +2
    ///                              ^
    ///                              |
    ///                          snapshot
    /// ```
    ///
    /// In the example above, Get(K) operation will call FullMerge with a base
    /// value of 2 and operands [+1, +2]. Compaction process might decide to
    /// collapse the beginning of the history up to the snapshot by performing
    /// full Merge with base value of 0 and operands [+1, +2, +7, +3].
    fn full_merge(&self, input: &MergeOperationInput, output: &mut MergeOperationOutput) -> bool;

    /// This function performs merge(left_op, right_op)
    /// when both the operands are themselves merge operation types
    /// that you would have passed to a DB::Merge() call in the same order
    /// (i.e.: DB::Merge(key,left_op), followed by DB::Merge(key,right_op)).
    ///
    /// PartialMerge should combine them into a single merge operation that is
    /// saved into *new_value, and then it should return true.
    /// *new_value should be constructed such that a call to
    /// DB::Merge(key, *new_value) would yield the same result as a call
    /// to DB::Merge(key, left_op) followed by DB::Merge(key, right_op).
    ///
    /// The string that new_value is pointing to will be empty.
    ///
    /// The default implementation of PartialMergeMulti will use this function
    /// as a helper, for backward compatibility.  Any successor class of
    /// MergeOperator should either implement PartialMerge or PartialMergeMulti,
    /// although implementing PartialMergeMulti is suggested as it is in general
    /// more effective to merge multiple operands at a time instead of two
    /// operands at a time.
    ///
    /// If it is impossible or infeasible to combine the two operations,
    /// leave new_value unchanged and return false. The library will
    /// internally keep track of the operations, and apply them in the
    /// correct order once a base-value (a Put/Delete/End-of-Database) is seen.
    fn partial_merge(&self, key: &[u8], lhs: &[u8], rhs: &[u8], new_value: &mut NewValue) -> bool;

    /// This function performs merge when all the operands are themselves merge
    /// operation types that you would have passed to a DB::Merge() call in the
    /// same order (front() first)
    /// (i.e. DB::Merge(key, operand_list[0]), followed by
    ///  DB::Merge(key, operand_list[1]), ...)
    ///
    /// PartialMergeMulti should combine them into a single merge operation that is
    /// saved into *new_value, and then it should return true.  *new_value should
    /// be constructed such that a call to DB::Merge(key, *new_value) would yield
    /// the same result as subquential individual calls to DB::Merge(key, operand)
    /// for each operand in operand_list from front() to back().
    ///
    /// The string that new_value is pointing to will be empty.
    ///
    /// The PartialMergeMulti function will be called when there are at least two
    /// operands.
    ///
    /// In the default implementation, PartialMergeMulti will invoke PartialMerge
    /// multiple times, where each time it only merges two operands.  Developers
    /// should either implement PartialMergeMulti, or implement PartialMerge which
    /// is served as the helper function of the default PartialMergeMulti.
    fn partial_merge_multi(&self, key: &[u8], operands: &[&[u8]], new_value: &mut NewValue)
        -> bool;

    /// Determines whether the PartialMerge can be called with just a single
    /// merge operand.
    /// Override and return true for allowing a single operand. PartialMerge
    /// and PartialMergeMulti should be overridden and implemented
    /// correctly to properly handle a single operand.
    fn allow_single_operand(&self) -> bool;

    /// Allows to control when to invoke a full merge during Get.
    /// This could be used to limit the number of merge operands that are looked at
    /// during a point lookup, thereby helping in limiting the number of levels to
    /// read from.
    /// Doesn't help with iterators.
    ///
    /// Note: the merge operands are passed to this function in the reversed order
    /// relative to how they were merged (passed to FullMerge or FullMergeV2)
    /// for performance reasons, see also:
    /// https://github.com/facebook/rocksdb/issues/3865
    fn should_merge(&self, operands: &[&[u8]]) -> bool;
}

unsafe extern "C" fn full_merge<T: MergeOperator>(
    ctx: *mut c_void,
    input: *const rocksdb_MergeOperator_MergeOperationInput,
    output: *mut rocksdb_MergeOperator_MergeOperationOutput,
) -> bool {
    let c = &*(ctx as *mut T);
    c.full_merge(
        &*(input as *const MergeOperationInput),
        &mut *(output as *mut MergeOperationOutput),
    )
}

unsafe extern "C" fn partial_merge<T: MergeOperator>(
    ctx: *mut c_void,
    key: rocksdb_Slice,
    lhs: rocksdb_Slice,
    rhs: rocksdb_Slice,
    append_ctx: *mut c_void,
    append_cb: AppendCb,
) -> bool {
    let c = &*(ctx as *mut T);
    c.partial_merge(
        s(key),
        s(lhs),
        s(rhs),
        &mut NewValue {
            ctx: append_ctx,
            func: append_cb,
            lifetime: PhantomData,
        },
    )
}

unsafe extern "C" fn partial_merge_mult<T: MergeOperator>(
    ctx: *mut c_void,
    key: rocksdb_Slice,
    ops: *const rocksdb_Slice,
    count: usize,
    append_ctx: *mut c_void,
    append_cb: AppendCb,
) -> bool {
    let c = &*(ctx as *mut T);
    let ops = slice::from_raw_parts(ops, count);
    let mut new_v = NewValue {
        ctx: append_ctx,
        func: append_cb,
        lifetime: PhantomData,
    };
    let r_ops = util::rocks_slice_to_array(ops);
    c.partial_merge_multi(s(key), &r_ops, &mut new_v)
}

unsafe extern "C" fn name<T: MergeOperator>(ctx: *mut c_void) -> *const c_char {
    let c = &*(ctx as *mut T);
    c.name().as_ptr()
}

unsafe extern "C" fn allow_single_operand<T: MergeOperator>(ctx: *mut c_void) -> bool {
    let c = &*(ctx as *mut T);
    c.allow_single_operand()
}

unsafe extern "C" fn should_merge<T: MergeOperator>(
    ctx: *mut c_void,
    ops: *const rocksdb_Slice,
    count: usize,
) -> bool {
    let c = &*(ctx as *mut T);
    let ops = slice::from_raw_parts(ops, count);
    let r_ops = util::rocks_slice_to_array(ops);
    c.should_merge(&r_ops)
}

unsafe extern "C" fn destroy<T: MergeOperator>(ctx: *mut c_void) {
    drop(Box::from_raw(ctx as *mut T));
}

impl<F> MergeOperator for F
where
    F: Fn(&[u8], Option<&[u8]>, &[&[u8]], &mut NewValue) -> bool + Send + Sync,
{
    fn name(&self) -> &CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(b"anoy merge operator\0") }
    }

    fn full_merge(&self, input: &MergeOperationInput, output: &mut MergeOperationOutput) -> bool {
        unsafe {
            let mut cb = NewValue::with_output(output);
            (*self)(
                input.key(),
                input.existing_value(),
                &input.operand_list(),
                &mut cb,
            )
        }
    }

    fn partial_merge(&self, key: &[u8], lhs: &[u8], rhs: &[u8], new_value: &mut NewValue) -> bool {
        (*self)(key, None, &[lhs, rhs], new_value)
    }

    fn partial_merge_multi(
        &self,
        key: &[u8],
        operand_list: &[&[u8]],
        new_value: &mut NewValue,
    ) -> bool {
        (*self)(key, None, operand_list, new_value)
    }

    fn allow_single_operand(&self) -> bool {
        false
    }

    fn should_merge(&self, _: &[&[u8]]) -> bool {
        false
    }
}

#[derive(Debug)]
pub struct SysMergeOperator {
    ptr: *mut crocksdb_mergeoperator_t,
}

impl Drop for SysMergeOperator {
    #[inline]
    fn drop(&mut self) {
        unsafe { tirocks_sys::crocksdb_mergeoperator_destroy(self.ptr) }
    }
}

impl SysMergeOperator {
    #[inline]
    pub fn new<T: MergeOperator>(t: T) -> Self {
        unsafe {
            let ctx = Box::into_raw(Box::new(t));
            let ptr = tirocks_sys::crocksdb_mergeoperator_create(
                ctx as *mut c_void,
                Some(destroy::<T>),
                Some(full_merge::<T>),
                Some(partial_merge::<T>),
                Some(partial_merge_mult::<T>),
                Some(name::<T>),
                Some(allow_single_operand::<T>),
                Some(should_merge::<T>),
            );
            Self { ptr }
        }
    }

    pub(crate) fn get_ptr(&self) -> *mut crocksdb_mergeoperator_t {
        self.ptr
    }
}
