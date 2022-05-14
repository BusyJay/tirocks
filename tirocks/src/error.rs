// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::fmt::{self, Debug};
use std::io;
use std::str::Utf8Error;

use tirocks_sys::rocksdb_Status;

/// A safe wrapper around rocksdb::Status.
#[derive(PartialEq)]
#[repr(transparent)]
pub struct Status(rocksdb_Status);

pub type Code = tirocks_sys::rocksdb_Status_Code;
pub type Severity = tirocks_sys::rocksdb_Status_Severity;
pub type SubCode = tirocks_sys::rocksdb_Status_SubCode;

impl Status {
    #[inline]
    pub fn with_code(code: Code) -> Status {
        Status(rocksdb_Status::with_code(code))
    }

    #[inline]
    pub fn new(code: Code, sub_code: SubCode, state: impl AsRef<[u8]>) -> Status {
        let mut s = Status::with_error(code, state);
        s.set_sub_code(sub_code);
        s
    }

    /// Build a status with given errors.
    ///
    /// # Panics
    ///
    /// `code` should not be `kOK`.
    #[inline]
    pub fn with_error(code: Code, state: impl AsRef<[u8]>) -> Status {
        Status(rocksdb_Status::with_error(code, state))
    }

    #[inline]
    pub fn with_no_space(msg: impl AsRef<[u8]>) -> Status {
        Self::new(Code::kIOError, SubCode::kNoSpace, msg)
    }

    #[inline]
    pub fn with_invalid_argument(msg: impl AsRef<[u8]>) -> Status {
        Self::new(Code::kInvalidArgument, SubCode::kNone, msg)
    }

    #[inline]
    pub fn with_memory_limmit(msg: impl AsRef<[u8]>) -> Status {
        Self::new(Code::kIOError, SubCode::kMemoryLimit, msg)
    }

    #[inline]
    pub fn with_space_limit(msg: impl AsRef<[u8]>) -> Status {
        Self::new(Code::kIOError, SubCode::kSpaceLimit, msg)
    }

    #[inline]
    pub fn with_path_not_found(msg: impl AsRef<[u8]>) -> Status {
        Self::new(Code::kIOError, SubCode::kPathNotFound, msg)
    }

    #[inline]
    pub fn ok(&self) -> bool {
        self.0.ok()
    }

    #[inline]
    pub fn state(&self) -> Option<&[u8]> {
        self.0.state()
    }

    #[inline]
    pub fn message(&self) -> std::result::Result<Option<&str>, Utf8Error> {
        self.0.message()
    }

    #[inline]
    pub fn code(&self) -> Code {
        self.0.code()
    }

    #[inline]
    pub fn severity(&self) -> Severity {
        self.0.severity()
    }

    #[inline]
    pub fn set_severity(&mut self, severity: Severity) {
        self.0.set_severity(severity);
    }

    #[inline]
    pub fn sub_code(&self) -> SubCode {
        self.0.sub_code()
    }

    #[inline]
    pub fn set_sub_code(&mut self, sub_code: SubCode) {
        self.0.set_sub_code(sub_code)
    }

    #[inline]
    pub(crate) fn into_raw(self) -> rocksdb_Status {
        self.0
    }

    #[inline]
    pub(crate) fn as_mut_ptr(&mut self) -> *mut rocksdb_Status {
        &mut self.0
    }

    #[inline]
    pub fn path_not_exist(&self) -> bool {
        self.code() == Code::kNotFound
            || self.code() == Code::kIOError && self.sub_code() == SubCode::kPathNotFound
    }

    #[inline]
    pub(crate) unsafe fn from_mut_ptr<'a>(ptr: *mut rocksdb_Status) -> &'a mut Status {
        &mut *(ptr as *mut Status)
    }
}

impl Default for Status {
    #[inline]
    fn default() -> Status {
        Status::with_code(Code::kOk)
    }
}

impl From<rocksdb_Status> for Status {
    #[inline]
    fn from(s: rocksdb_Status) -> Status {
        Status(s)
    }
}

impl From<Status> for io::Error {
    #[inline]
    fn from(s: Status) -> Self {
        let kind = match s.code() {
            Code::kIOError => match s.sub_code() {
                #[cfg(feature = "nightly")]
                SubCode::kNoSpace => io::ErrorKind::StorageFull,
                #[cfg(feature = "nightly")]
                SubCode::kSpaceLimit => io::ErrorKind::FilesystemQuotaExceeded,
                SubCode::kMemoryLimit => io::ErrorKind::OutOfMemory,
                SubCode::kPathNotFound => io::ErrorKind::NotFound,
                _ => io::ErrorKind::Other,
            },
            Code::kNotFound => io::ErrorKind::NotFound,
            _ => io::ErrorKind::Other,
        };
        let msg = match s.state() {
            Some(s) => String::from_utf8_lossy(s),
            None => return Self::from(kind),
        };
        Self::new(kind, msg)
    }
}

impl Debug for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.ok() {
            write!(f, "OK")
        } else {
            write!(
                f,
                "Code: {:?}, SubCode: {:?}, Severity: {:?}, msg: {:?}",
                self.code(),
                self.sub_code(),
                self.severity(),
                self.message()
            )
        }
    }
}

pub type Result<T> = std::result::Result<T, Status>;

/// A helper micros for handling FFI calls that need error handling.
///
/// It's simply translate the call
/// ```ignored
/// ffi_call!(func(...))
/// ```
/// to
/// ```ignored
/// let res = tirocks_sys::func(..., &mut status);
/// if status.ok() {
///     Ok(res)
/// } else {
///     Err(status)
/// }
/// ```
macro_rules! ffi_call {
    ($func:ident($($arg:expr),+)) => ({
        let mut status = $crate::Status::default();
        let res = tirocks_sys::$func($($arg),+, status.as_mut_ptr());
        if status.ok() {
            Ok(res)
        } else {
            Err(status)
        }
    });
    ($func:ident()) => ({
        let mut status = $crate::Status::default();
        let res = tirocks_sys::$func(status.as_mut_ptr());
        if status.ok() {
            Ok(res)
        } else {
            Err(status)
        }
    })
}

pub(crate) use ffi_call;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_io_error_transform() {
        let cases = [
            (Status::with_code(Code::kBusy), io::ErrorKind::Other, ""),
            (
                Status::with_error(Code::kAborted, "test"),
                io::ErrorKind::Other,
                "test",
            ),
            #[cfg(feature = "nightly")]
            (
                Status::with_space_limit("limit"),
                io::ErrorKind::FilesystemQuotaExceeded,
                "limit",
            ),
            #[cfg(feature = "nightly")]
            (
                Status::with_no_space("full"),
                io::ErrorKind::StorageFull,
                "full",
            ),
            (
                Status::with_memory_limmit("oom"),
                io::ErrorKind::OutOfMemory,
                "oom",
            ),
            (
                Status::with_path_not_found("path"),
                io::ErrorKind::NotFound,
                "path",
            ),
        ];
        for (status, kind, pattern) in cases {
            let e: io::Error = status.into();
            assert_eq!(e.kind(), kind);
            let msg = format!("{}", e);
            assert!(msg.contains(pattern), "{} in {}", pattern, msg);
        }
    }
}
