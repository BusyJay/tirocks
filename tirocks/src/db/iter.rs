// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::{
    marker::PhantomData,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
};

use tirocks_sys::{r, rocksdb_Iterator, s};

use crate::{option::ReadOptions, table_properties::user::SequenceNumber, RawDb, Status};

use super::cf::RawColumnFamilyHandle;

pub unsafe trait Iterable {
    fn raw_iter(&self, opt: &mut ReadOptions) -> *mut rocksdb_Iterator;
    fn raw_iter_cf(
        &self,
        opt: &mut ReadOptions,
        cf: &RawColumnFamilyHandle,
    ) -> *mut rocksdb_Iterator;
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
    unsafe fn from_ptr(ptr: *mut rocksdb_Iterator) -> Self {
        Self {
            ptr,
            _life: PhantomData,
        }
    }
    pub fn new<I: Iterable>(i: &'a I, opt: &'a mut ReadOptions) -> Self {
        unsafe { Self::from_ptr(i.raw_iter(opt)) }
    }

    pub fn with_cf<I: Iterable>(
        i: &'a I,
        opt: &'a mut ReadOptions,
        cf: &RawColumnFamilyHandle,
    ) -> Self {
        unsafe { Self::from_ptr(i.raw_iter_cf(opt, cf)) }
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
    pub fn status(&self) -> Status {
        unsafe {
            let mut s = Status::default();
            tirocks_sys::crocksdb_iter_get_error(self.ptr, s.as_mut_ptr());
            s
        }
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

pub struct Iterator<'a, I: Iterable + 'a> {
    iter: ManuallyDrop<RawIterator<'a>>,
    _i: I,
    _read: ReadOptions,
}

impl<'a, I: Iterable + 'a> Iterator<'a, I> {
    #[inline]
    pub fn new(i: I, mut read: ReadOptions) -> Self {
        unsafe {
            let ptr = i.raw_iter(&mut read);
            Iterator {
                iter: ManuallyDrop::new(RawIterator::from_ptr(ptr)),
                _i: i,
                _read: read,
            }
        }
    }

    #[inline]
    pub fn with_cf(i: I, mut read: ReadOptions, cf: &RawColumnFamilyHandle) -> Self {
        unsafe {
            let ptr = i.raw_iter_cf(&mut read, cf);
            Iterator {
                iter: ManuallyDrop::new(RawIterator::from_ptr(ptr)),
                _i: i,
                _read: read,
            }
        }
    }
}

impl<'a, I: Iterable + 'a> Deref for Iterator<'a, I> {
    type Target = RawIterator<'a>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.iter
    }
}

impl<'a, I: Iterable + 'a> DerefMut for Iterator<'a, I> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.iter
    }
}

impl<'a, I: Iterable + 'a> Drop for Iterator<'a, I> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.iter);
        }
    }
}

unsafe impl Iterable for RawDb {
    fn raw_iter(&self, opt: &mut ReadOptions) -> *mut rocksdb_Iterator {
        unsafe { tirocks_sys::crocksdb_create_iterator(self.as_ptr(), opt.get() as _) }
    }

    fn raw_iter_cf(
        &self,
        opt: &mut ReadOptions,
        cf: &RawColumnFamilyHandle,
    ) -> *mut rocksdb_Iterator {
        unsafe {
            tirocks_sys::crocksdb_create_iterator_cf(self.as_ptr(), opt.get() as _, cf.as_mut_ptr())
        }
    }
}

impl RawDb {
    pub fn iter<'a>(&'a self, read: &'a mut ReadOptions) -> RawIterator<'a> {
        RawIterator::new(self, read)
    }

    pub fn iter_cf<'a>(
        &'a self,
        read: &'a mut ReadOptions,
        cf: &RawColumnFamilyHandle,
    ) -> RawIterator<'a> {
        RawIterator::with_cf(self, read, cf)
    }
}
