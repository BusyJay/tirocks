// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::{
    marker::PhantomData,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use tirocks_sys::{r, rocksdb_Iterator, s};

use crate::{
    db::RawCfHandle, option::ReadOptions, util::ffi_call, Db, RawDb, Result, SequenceNumber, Status,
};

/// A trait for types that can create a raw iterator.
///
/// # Safety
///
/// Implementation should make sure the returned pointer can be used forever as long as
/// `self` is alive.
pub unsafe trait Iterable {
    /// Creates a raw iterator.
    ///
    /// A mutable reference to `ReadOptions` is required to make it possible for setting snapshot
    /// at runtime.
    fn raw_iter(&self, opt: &mut ReadOptions, cf: &RawCfHandle) -> *mut rocksdb_Iterator;
}

/// Same as [`CfIterable`], but for simple data source that doesn't have a column family.
pub unsafe trait SimpleIterable {
    /// Creates a raw iterator.
    ///
    /// A mutable reference to `ReadOptions` is required to make it possible for setting snapshot
    /// at runtime.
    fn raw_iter(&self, opt: &mut ReadOptions) -> *mut rocksdb_Iterator;
}

#[repr(transparent)]
pub struct RawIterator<'a> {
    ptr: *mut rocksdb_Iterator,
    _life: PhantomData<&'a ()>,
}

impl<'a> Drop for RawIterator<'a> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            tirocks_sys::crocksdb_iter_destroy(self.ptr);
        }
    }
}

impl<'a> RawIterator<'a> {
    pub(crate) unsafe fn from_ptr(ptr: *mut rocksdb_Iterator) -> Self {
        Self {
            ptr,
            _life: PhantomData,
        }
    }

    pub fn new<I: Iterable>(i: &'a I, opt: &'a mut ReadOptions, cf: &RawCfHandle) -> Self {
        unsafe { Self::from_ptr(i.raw_iter(opt, cf)) }
    }

    pub fn with_simple<I: SimpleIterable>(i: &'a I, opt: &'a mut ReadOptions) -> Self {
        unsafe { Self::from_ptr(i.raw_iter(opt)) }
    }

    /// An iterator is either positioned at a key/value pair, or
    /// not valid.  This method returns true iff the iterator is valid.
    /// Always returns false if !status().ok().
    #[inline]
    pub fn valid(&self) -> bool {
        unsafe { tirocks_sys::crocksdb_iter_valid(self.ptr) }
    }

    /// Position at the first key in the source.  The iterator is Valid()
    /// after this call iff the source is not empty.
    #[inline]
    pub fn seek_to_first(&mut self) {
        unsafe { tirocks_sys::crocksdb_iter_seek_to_first(self.ptr) }
    }

    /// Position at the last key in the source.  The iterator is
    /// Valid() after this call iff the source is not empty.
    #[inline]
    pub fn seek_to_last(&mut self) {
        unsafe { tirocks_sys::crocksdb_iter_seek_to_last(self.ptr) }
    }

    /// Position at the first key in the source that at or past target.
    /// The iterator is Valid() after this call iff the source contains
    /// an entry that comes at or past target.
    /// All Seek*() methods clear any error status() that the iterator had prior to
    /// the call; after the seek, status() indicates only the error (if any) that
    /// happened during the seek, not any past errors.
    #[inline]
    pub fn seek(&mut self, target: &[u8]) {
        unsafe { tirocks_sys::crocksdb_iter_seek(self.ptr, r(target)) }
    }

    /// Position at the last key in the source that at or before target.
    /// The iterator is Valid() after this call iff the source contains
    /// an entry that comes at or before target.
    #[inline]
    pub fn seek_for_prev(&mut self, target: &[u8]) {
        unsafe { tirocks_sys::crocksdb_iter_seek_for_prev(self.ptr, r(target)) }
    }

    /// Moves to the next entry in the source.  After this call, Valid() is
    /// true iff the iterator was not positioned at the last entry in the source.
    /// REQUIRES: Valid()
    #[inline]
    pub fn next(&mut self) {
        unsafe { tirocks_sys::crocksdb_iter_next(self.ptr) }
    }

    /// Moves to the previous entry in the source.  After this call, Valid() is
    /// true iff the iterator was not positioned at the first entry in source.
    /// REQUIRES: Valid()
    #[inline]
    pub fn prev(&mut self) {
        unsafe { tirocks_sys::crocksdb_iter_prev(self.ptr) }
    }

    /// Return the key for the current entry.  The underlying storage for
    /// the returned slice is valid only until the next modification of
    /// the iterator.
    /// REQUIRES: Valid()
    #[inline]
    pub fn key(&self) -> &[u8] {
        unsafe {
            let mut buf = r(&[]);
            tirocks_sys::crocksdb_iter_key(self.ptr, &mut buf);
            s(buf)
        }
    }

    /// Return the value for the current entry.  The underlying storage for
    /// the returned slice is valid only until the next modification of
    /// the iterator.
    /// REQUIRES: Valid()
    #[inline]
    pub fn value(&self) -> &[u8] {
        unsafe {
            let mut buf = r(&[]);
            tirocks_sys::crocksdb_iter_value(self.ptr, &mut buf);
            s(buf)
        }
    }

    /// Return the sequence number for the current entry if it's available.
    /// Return false if it's not available.
    /// REQUIRES: Valid()
    #[inline]
    pub fn sequence_number(&self) -> Option<SequenceNumber> {
        unsafe {
            let mut seqno = 0;
            let b = tirocks_sys::crocksdb_iter_seqno(self.ptr, &mut seqno);
            if b {
                Some(seqno)
            } else {
                None
            }
        }
    }

    /// If an error has occurred, return it.  Else return an ok status.
    /// If non-blocking IO is requested and this operation cannot be
    /// satisfied without doing some IO, then this returns Status::Incomplete().
    #[inline]
    pub fn check(&self) -> Result<()> {
        unsafe { ffi_call!(crocksdb_iter_get_error(self.ptr)) }
    }

    /// If supported, renew the iterator to represent the latest state. The
    /// iterator will be invalidated after the call. Not supported if
    /// ReadOptions.snapshot is given when creating the iterator.
    #[inline]
    pub fn refresh(&mut self) -> Status {
        unsafe {
            let mut s = Status::default();
            tirocks_sys::crocksdb_iter_refresh(self.ptr, s.as_mut_ptr());
            s
        }
    }
}

impl<'a> std::iter::Iterator for RawIterator<'a> {
    type Item = (Vec<u8>, Vec<u8>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.valid() {
            let kv = Some((self.key().to_vec(), self.value().to_vec()));
            RawIterator::next(self);
            return kv;
        }
        None
    }
}

unsafe impl<'a> Send for RawIterator<'a> {}
unsafe impl<'a> Sync for RawIterator<'a> {}

pub struct Iterator<'a, I: 'a> {
    iter: ManuallyDrop<RawIterator<'a>>,
    _i: I,
    _read: ReadOptions,
}

impl<'a, I: Iterable + 'a> Iterator<'a, I> {
    #[inline]
    pub fn new(i: I, mut read: ReadOptions, cf: &RawCfHandle) -> Self {
        unsafe {
            let ptr = i.raw_iter(&mut read, cf);
            Iterator {
                iter: ManuallyDrop::new(RawIterator::from_ptr(ptr)),
                _i: i,
                _read: read,
            }
        }
    }
}

impl<'a, I: SimpleIterable + 'a> Iterator<'a, I> {
    #[inline]
    pub fn with_simple(i: I, mut read: ReadOptions) -> Self {
        unsafe {
            let ptr = i.raw_iter(&mut read);
            Iterator {
                iter: ManuallyDrop::new(RawIterator::from_ptr(ptr)),
                _i: i,
                _read: read,
            }
        }
    }
}

impl<'a, I: 'a> Deref for Iterator<'a, I> {
    type Target = RawIterator<'a>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.iter
    }
}

impl<'a, I: 'a> DerefMut for Iterator<'a, I> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.iter
    }
}

impl<'a, I: 'a> Drop for Iterator<'a, I> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.iter);
        }
    }
}

unsafe impl Iterable for RawDb {
    fn raw_iter(&self, opt: &mut ReadOptions, cf: &RawCfHandle) -> *mut rocksdb_Iterator {
        unsafe {
            tirocks_sys::crocksdb_create_iterator_cf(
                self.get_ptr(),
                opt.get_ptr() as _,
                cf.get_ptr(),
            )
        }
    }
}

unsafe impl Iterable for Db {
    #[inline]
    fn raw_iter(&self, opt: &mut ReadOptions, cf: &RawCfHandle) -> *mut rocksdb_Iterator {
        unsafe {
            if !self.is_titan() {
                tirocks_sys::crocksdb_create_iterator_cf(
                    self.get_ptr(),
                    opt.get_ptr() as _,
                    cf.get_ptr(),
                )
            } else {
                tirocks_sys::ctitandb_create_iterator_cf(
                    self.get_ptr(),
                    opt.get_ptr(),
                    cf.get_ptr(),
                )
            }
        }
    }
}

unsafe impl Iterable for Arc<Db> {
    #[inline]
    fn raw_iter(&self, opt: &mut ReadOptions, cf: &RawCfHandle) -> *mut rocksdb_Iterator {
        (**self).raw_iter(opt, cf)
    }
}
