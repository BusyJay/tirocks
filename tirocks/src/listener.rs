// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use libc::c_void;
use tirocks_sys::{
    crocksdb_eventlistener_t, rocksdb_BackgroundErrorReason, rocksdb_CompactionJobInfo, rocksdb_DB,
    rocksdb_ExternalFileIngestionInfo, rocksdb_FlushJobInfo, rocksdb_Status,
    rocksdb_SubcompactionJobInfo, rocksdb_WriteStallInfo,
};

use crate::Status;

use self::{
    job_info::{CompactionJobInfo, FlushJobInfo, SubcompactionJobInfo},
    other_info::{ExternalFileIngestionInfo, WriteStallInfo},
};

pub mod job_info;
pub mod other_info;

pub type BackgroundErrorReason = rocksdb_BackgroundErrorReason;

/// EventListener trait contains a set of call-back functions that will
/// be called when specific RocksDB event happens such as flush.  It can
/// be used as a building block for developing custom features such as
/// stats-collector or external compaction algorithm.
///
/// Note that call-back functions should not run for an extended period of
/// time before the function returns, otherwise RocksDB may be blocked.
/// For more information, please see
/// [doc of rocksdb](https://github.com/facebook/rocksdb/blob/master/include/rocksdb/listener.h).
pub trait EventListener: Send + Sync {
    /// A callback function to RocksDB which will be called before a
    /// RocksDB starts to flush memtables.  The default implementation is
    /// no-op.
    ///
    /// Note that the this function must be implemented in a way such that
    /// it should not run for an extended period of time before the function
    /// returns.  Otherwise, RocksDB may be blocked.
    fn on_flush_begin(&self, _: &FlushJobInfo) {}

    /// A callback function to RocksDB which will be called whenever a
    /// registered RocksDB flushes a file.  The default implementation is
    /// no-op.
    ///
    /// Note that the this function must be implemented in a way such that
    /// it should not run for an extended period of time before the function
    /// returns.  Otherwise, RocksDB may be blocked.
    fn on_flush_completed(&self, _: &FlushJobInfo) {}

    /// A callback function to RocksDB which will be called before a
    /// RocksDB starts to compact.  The default implementation is
    /// no-op.
    ///
    /// Note that the this function must be implemented in a way such that
    /// it should not run for an extended period of time before the function
    /// returns.  Otherwise, RocksDB may be blocked.
    fn on_compaction_begin(&self, _: &CompactionJobInfo) {}

    /// A callback function for RocksDB which will be called whenever
    /// a registered RocksDB compacts a file. The default implementation
    /// is a no-op.
    ///
    /// Note that this function must be implemented in a way such that
    /// it should not run for an extended period of time before the function
    /// returns. Otherwise, RocksDB may be blocked.
    ///
    /// @param db a pointer to the rocksdb instance which just compacted
    ///   a file.
    /// @param ci a reference to a CompactionJobInfo struct. 'ci' is released
    ///  after this function is returned, and must be copied if it is needed
    ///  outside of this function.
    fn on_compaction_completed(&self, _: &CompactionJobInfo) {}

    /// A callback function for RocksDB which will be called each time when
    /// a registered RocksDB uses multiple subcompactions to compact a file. The
    /// callback is called by each subcompaction and in the same thread.
    fn on_subcompaction_begin(&self, _: &SubcompactionJobInfo) {}

    /// A callback function for RocksDB which will be called each time when
    /// a registered RocksDB uses multiple subcompactions to compact a file. The
    /// callback is called by each subcompaction and in the same thread.
    fn on_subcompaction_completed(&self, _: &SubcompactionJobInfo) {}

    /// A callback function for RocksDB which will be called after an external
    /// file is ingested using IngestExternalFile.
    ///
    /// Note that the this function will run on the same thread as
    /// IngestExternalFile(), if this function is blocked, IngestExternalFile()
    /// will be blocked from finishing.
    fn on_external_file_ingested(&self, _: &ExternalFileIngestionInfo) {}

    /// A callback function for RocksDB which will be called before setting the
    /// background error status to a non-OK value. The new background error status
    /// is provided in `bg_error` and can be modified by the callback. E.g., a
    /// callback can suppress errors by resetting it to Status::OK(), thus
    /// preventing the database from entering read-only mode. We do not provide any
    /// guarantee when failed flushes/compactions will be rescheduled if the user
    /// suppresses an error.
    ///
    /// Note that this function can run on the same threads as flush, compaction,
    /// and user writes. So, it is extremely important not to perform heavy
    /// computations or blocking calls in this function.
    fn on_background_error(&self, _: BackgroundErrorReason, _: &mut Status) {}

    /// A callback function for RocksDB which will be called whenever a change
    /// of superversion triggers a change of the stall conditions.
    ///
    /// Note that the this function must be implemented in a way such that
    /// it should not run for an extended period of time before the function
    /// returns.  Otherwise, RocksDB may be blocked.
    fn on_stall_conditions_changed(&self, _: &WriteStallInfo) {}
}

extern "C" fn destructor<E: EventListener>(ctx: *mut c_void) {
    unsafe {
        drop(Box::from_raw(ctx as *mut E));
    }
}

// Maybe we should reuse db instance?
// TODO: refactor DB implement so that we can convert DBInstance to DB.
extern "C" fn on_flush_begin<E: EventListener>(
    ctx: *mut c_void,
    _: *mut rocksdb_DB,
    info: *const rocksdb_FlushJobInfo,
) {
    unsafe {
        let l = &*(ctx as *mut E);
        l.on_flush_begin(FlushJobInfo::from_ptr(info));
    }
}

extern "C" fn on_flush_completed<E: EventListener>(
    ctx: *mut c_void,
    _: *mut rocksdb_DB,
    info: *const rocksdb_FlushJobInfo,
) {
    unsafe {
        let l = &*(ctx as *mut E);
        l.on_flush_completed(FlushJobInfo::from_ptr(info));
    }
}

extern "C" fn on_compaction_begin<E: EventListener>(
    ctx: *mut c_void,
    _: *mut rocksdb_DB,
    info: *const rocksdb_CompactionJobInfo,
) {
    unsafe {
        let l = &*(ctx as *mut E);
        l.on_compaction_begin(CompactionJobInfo::from_ptr(info));
    }
}

extern "C" fn on_compaction_completed<E: EventListener>(
    ctx: *mut c_void,
    _: *mut rocksdb_DB,
    info: *const rocksdb_CompactionJobInfo,
) {
    unsafe {
        let l = &*(ctx as *mut E);
        l.on_compaction_completed(CompactionJobInfo::from_ptr(info));
    }
}

extern "C" fn on_subcompaction_begin<E: EventListener>(
    ctx: *mut c_void,
    info: *const rocksdb_SubcompactionJobInfo,
) {
    unsafe {
        let l = &*(ctx as *mut E);
        l.on_subcompaction_begin(SubcompactionJobInfo::from_ptr(info));
    }
}

extern "C" fn on_subcompaction_completed<E: EventListener>(
    ctx: *mut c_void,
    info: *const rocksdb_SubcompactionJobInfo,
) {
    unsafe {
        let l = &*(ctx as *mut E);
        l.on_subcompaction_completed(SubcompactionJobInfo::from_ptr(info));
    }
}

extern "C" fn on_external_file_ingested<E: EventListener>(
    ctx: *mut c_void,
    _: *mut rocksdb_DB,
    info: *const rocksdb_ExternalFileIngestionInfo,
) {
    unsafe {
        let l = &*(ctx as *mut E);
        l.on_external_file_ingested(ExternalFileIngestionInfo::from_ptr(info));
    }
}

extern "C" fn on_background_error<E: EventListener>(
    ctx: *mut c_void,
    reason: BackgroundErrorReason,
    status: *mut rocksdb_Status,
) {
    unsafe {
        let l = &*(ctx as *mut E);
        l.on_background_error(reason, Status::from_mut_ptr(status));
    }
}

extern "C" fn on_stall_conditions_changed<E: EventListener>(
    ctx: *mut c_void,
    info: *const rocksdb_WriteStallInfo,
) {
    unsafe {
        let l = &*(ctx as *mut E);
        l.on_stall_conditions_changed(WriteStallInfo::from_ptr(info));
    }
}

/// A wrapped listener that can be shared by multiple DBs.
#[derive(Debug)]
pub struct SysEventListener {
    ptr: *mut crocksdb_eventlistener_t,
}

impl SysEventListener {
    #[inline]
    pub fn new<E: EventListener>(e: E) -> Self {
        let p = Box::new(e);
        let ptr = unsafe {
            tirocks_sys::crocksdb_eventlistener_create(
                Box::into_raw(p) as *mut c_void,
                Some(destructor::<E>),
                Some(on_flush_begin::<E>),
                Some(on_flush_completed::<E>),
                Some(on_compaction_begin::<E>),
                Some(on_compaction_completed::<E>),
                Some(on_subcompaction_begin::<E>),
                Some(on_subcompaction_completed::<E>),
                Some(on_external_file_ingested::<E>),
                Some(on_background_error::<E>),
                Some(on_stall_conditions_changed::<E>),
            )
        };
        SysEventListener { ptr }
    }

    #[inline]
    pub(crate) fn get(&self) -> *mut crocksdb_eventlistener_t {
        self.ptr
    }
}

impl Drop for SysEventListener {
    fn drop(&mut self) {
        unsafe {
            tirocks_sys::crocksdb_eventlistener_destroy(self.ptr);
        }
    }
}
