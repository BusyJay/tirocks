// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::{
    marker::PhantomData,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
};

use tirocks_sys::{rocksdb_Iterator, rocksdb_Snapshot};

use crate::{
    db::{RawCfHandle, RawDbRef},
    option::ReadOptions,
    properties::table::user::SequenceNumber,
    Iterable, PinSlice, RawDb, RawIterator, Result,
};

pub struct RawSnapshot<'a> {
    ptr: *const rocksdb_Snapshot,
    _life: PhantomData<&'a ()>,
}

impl<'a> RawSnapshot<'a> {
    #[inline]
    pub(crate) unsafe fn from_ptr(ptr: *const rocksdb_Snapshot) -> RawSnapshot<'a> {
        RawSnapshot {
            ptr,
            _life: PhantomData,
        }
    }

    #[inline]
    pub(crate) fn as_ptr(&self) -> *const rocksdb_Snapshot {
        self.ptr
    }

    #[inline]
    pub fn sequence_number(&self) -> SequenceNumber {
        let mut n = 0;
        unsafe {
            tirocks_sys::crocksdb_get_snapshot_sequence_number(self.ptr, &mut n);
        }
        n
    }
}

pub struct WithSnapOpt<'a> {
    opt: &'a mut ReadOptions,
    old_snap: *const rocksdb_Snapshot,
}

impl<'a> WithSnapOpt<'a> {
    pub fn new(opt: &'a mut ReadOptions, snap: &RawSnapshot) -> Self {
        let old_snap = opt.snapshot();
        unsafe {
            opt.set_snapshot(snap.as_ptr());
            Self { opt, old_snap }
        }
    }
}

impl<'a> Deref for WithSnapOpt<'a> {
    type Target = ReadOptions;
    fn deref(&self) -> &Self::Target {
        self.opt
    }
}

impl<'a> DerefMut for WithSnapOpt<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.opt
    }
}

impl<'a> Drop for WithSnapOpt<'a> {
    fn drop(&mut self) {
        unsafe {
            self.opt.set_snapshot(self.old_snap);
        }
    }
}

pub struct Snapshot<'a, D: RawDbRef + 'a> {
    snap: ManuallyDrop<RawSnapshot<'a>>,
    db: D,
}

impl<'a, D: RawDbRef + 'a> Snapshot<'a, D> {
    pub fn new(db: D) -> Self {
        unsafe {
            let ptr = db.with(|d| tirocks_sys::crocksdb_create_snapshot(d.get_ptr()));
            Snapshot {
                snap: ManuallyDrop::new(RawSnapshot::from_ptr(ptr)),
                db,
            }
        }
    }

    pub fn get(
        &self,
        opt: &mut ReadOptions,
        cf: &RawCfHandle,
        key: &[u8],
    ) -> Result<Option<Vec<u8>>> {
        let opt = WithSnapOpt::new(opt, &self.snap);
        self.db.with(|d| d.get(&opt, cf, key))
    }

    pub fn get_to(
        &self,
        opt: &mut ReadOptions,
        cf: &RawCfHandle,
        key: &[u8],
        value: &mut PinSlice,
    ) -> Result<bool> {
        let opt = WithSnapOpt::new(opt, &self.snap);
        self.db.with(|d| d.get_to(&opt, cf, key, value))
    }

    pub fn iter<'b>(&'b self, opt: &'b mut ReadOptions, cf: &RawCfHandle) -> RawIterator<'b> {
        RawIterator::new(self, opt, cf)
    }
}

unsafe impl<'a, D: RawDbRef + 'a> Iterable for Snapshot<'a, D> {
    fn raw_iter(&self, opt: &mut ReadOptions, cf: &RawCfHandle) -> *mut rocksdb_Iterator {
        let mut opt = WithSnapOpt::new(opt, &self.snap);
        self.db.with(|d| d.raw_iter(&mut opt, cf))
    }
}

impl<'a, D: RawDbRef + 'a> Deref for Snapshot<'a, D> {
    type Target = RawSnapshot<'a>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.snap
    }
}

impl<'a, D: RawDbRef + 'a> DerefMut for Snapshot<'a, D> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.snap
    }
}

impl<'a, D: RawDbRef + 'a> Drop for Snapshot<'a, D> {
    #[inline]
    fn drop(&mut self) {
        let snap = unsafe { ManuallyDrop::take(&mut self.snap) };
        self.db.with(|d| d.release_raw_snapshot(snap))
    }
}

impl RawDb {
    pub fn raw_snapshot(&self) -> RawSnapshot {
        unsafe {
            let ptr = tirocks_sys::crocksdb_create_snapshot(self.get_ptr());
            RawSnapshot::from_ptr(ptr)
        }
    }

    pub fn release_raw_snapshot(&self, snap: RawSnapshot) {
        unsafe { tirocks_sys::crocksdb_release_snapshot(self.get_ptr(), snap.as_ptr()) }
    }

    pub fn snapshot(&self) -> Snapshot<&RawDb> {
        Snapshot::new(self)
    }
}
