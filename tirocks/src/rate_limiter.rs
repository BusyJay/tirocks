// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use crate::env::IoPriority;
use std::marker::PhantomData;
use std::time::Duration;

pub type RateLimiterMode = tirocks_sys::rocksdb_RateLimiter_Mode;
pub type OpType = tirocks_sys::rocksdb_RateLimiter_OpType;

pub struct RateLimiterBuilder {
    /// It controls the total write rate of compaction and flush in bytes per
    /// second. Currently, RocksDB does not enforce rate limit for anything other
    /// than flush and compaction, e.g. write to WAL.
    pub rate_bytes_per_sec: i64,
    /// It controls how often tokens are refilled. For example, when `rate_bytes_per_sec`
    /// is set to 10MB/s and refill_period is set to 100ms, then 1MB is refilled every
    /// 100ms internally. Larger value can lead to burstier writes while smaller value
    /// introduces more CPU overhead.
    pub refill_period: Duration,
    /// RateLimiter accepts high-pri requests and low-pri requests. A low-pri request is
    /// usually blocked in favor of hi-pri request. Currently, RocksDB assigns low-pri to
    /// request from compaction and high-pri to request from flush. Low-pri requests can
    /// get blocked if flush requests come in continuously. This fairness parameter grants
    /// low-pri requests permission by 1/fairness chance even though high-pri requests
    /// exist to avoid starvation.
    pub fairness: i32,
    /// Mode indicates which types of operations count against the limit.
    pub mode: RateLimiterMode,
    /// Enables dynamic adjustment of rate limit within the range
    /// `[rate_bytes_per_sec / 20, rate_bytes_per_sec]`, according to the recent demand for
    /// background I/O.
    pub auto_tuned: bool,
    // So adding new fields won't be a breaking change in the future.
    _hidden: PhantomData<()>,
}

impl RateLimiterBuilder {
    #[inline]
    pub fn new(rate_bytes_per_sec: i64) -> RateLimiterBuilder {
        RateLimiterBuilder {
            rate_bytes_per_sec,
            refill_period: Duration::from_millis(100),
            fairness: 10,
            mode: RateLimiterMode::kWritesOnly,
            auto_tuned: false,
            _hidden: PhantomData,
        }
    }

    /// Create a RateLimiter object, which can be shared among RocksDB instances to
    /// control write rate of flush and compaction.
    #[inline]
    pub fn build_generic(&self) -> RateLimiter {
        let ptr = unsafe {
            tirocks_sys::crocksdb_ratelimiter_create_with_auto_tuned(
                self.rate_bytes_per_sec,
                self.refill_period.as_micros() as i64,
                self.fairness,
                self.mode,
                self.auto_tuned as u8,
            )
        };
        RateLimiter { ptr }
    }

    /// Similar to generic auto tuned rate limiter, but with following advantages:
    ///
    /// - only one important internal knob, easier to tune
    /// - respond quickly to write flow burst, introduce zero write stall when loading data
    /// - anti-jitter even when pressure is low
    #[inline]
    pub fn build_write_amp_based(&self) -> RateLimiter {
        let ptr = unsafe {
            tirocks_sys::crocksdb_writeampbasedratelimiter_create_with_auto_tuned(
                self.rate_bytes_per_sec,
                self.refill_period.as_micros() as i64,
                self.fairness,
                self.mode,
                self.auto_tuned as u8,
            )
        };
        RateLimiter { ptr }
    }
}

/// A rate limiter can be shared among RocksDB instances to control write
/// rate of flush and compaction.
pub struct RateLimiter {
    ptr: *mut tirocks_sys::crocksdb_ratelimiter_t,
}

unsafe impl Send for RateLimiter {}
unsafe impl Sync for RateLimiter {}

impl RateLimiter {
    /// Dynamically change rate limiter's bytes per second.
    #[inline]
    pub fn set_bytes_per_second(&self, bytes_per_sec: i64) {
        unsafe {
            tirocks_sys::crocksdb_ratelimiter_set_bytes_per_second(self.ptr, bytes_per_sec);
        }
    }

    /// Dynamically change rate limiter's auto_tuned mode.
    #[inline]
    pub fn set_auto_tuned(&self, auto_tuned: bool) {
        unsafe { tirocks_sys::crocksdb_ratelimiter_set_auto_tuned(self.ptr, auto_tuned as u8) }
    }

    /// Requests token to read or write bytes.
    ///
    /// If this request can not be satisfied, the call is blocked. Caller is
    /// responsible to make sure bytes <= SingleBurstBytes().
    #[inline]
    pub fn request(&self, bytes: i64, pri: IoPriority, op_ty: OpType) {
        unsafe {
            tirocks_sys::crocksdb_ratelimiter_request(self.ptr, bytes, pri, op_ty);
        }
    }

    /// Max bytes can be granted in a single burst
    #[inline]
    pub fn single_burst_bytes(&self) -> i64 {
        unsafe { tirocks_sys::crocksdb_ratelimiter_get_singleburst_bytes(self.ptr) }
    }

    /// Total bytes that go through rate limiter.
    pub fn total_bytes_through(&self, pri: IoPriority) -> i64 {
        unsafe { tirocks_sys::crocksdb_ratelimiter_get_total_bytes_through(self.ptr, pri) }
    }

    pub fn bytes_per_second(&self) -> i64 {
        unsafe { tirocks_sys::crocksdb_ratelimiter_get_bytes_per_second(self.ptr) }
    }

    /// Total count of requests that go through rate limiter
    pub fn total_requests(&self, pri: IoPriority) -> i64 {
        unsafe { tirocks_sys::crocksdb_ratelimiter_get_total_requests(self.ptr, pri) }
    }
}

impl Drop for RateLimiter {
    #[inline]
    fn drop(&mut self) {
        unsafe { tirocks_sys::crocksdb_ratelimiter_destroy(self.ptr) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_rate_limiter() {
        let speed = 3000.;
        let limiter = super::RateLimiterBuilder::new(speed as i64).build_generic();
        let total_bytes = 6000;
        let mut consumed = 0;
        let step = 60;
        let timer = Instant::now();
        while consumed < total_bytes {
            limiter.request(step, IoPriority::IO_HIGH, OpType::kWrite);
            consumed += step;
        }
        let elapsed = timer.elapsed();
        let actual_speed = consumed as f64 / elapsed.as_secs_f64();
        assert!(
            actual_speed < speed * 1.5 && actual_speed > speed * 0.5,
            "{}",
            actual_speed
        );
    }
}
