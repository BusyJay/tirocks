// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::{fmt::Display, mem::MaybeUninit};

use tirocks_sys::{
    rocksdb_HistogramData, rocksdb_Histograms, rocksdb_Tickers, rocksdb_titandb_HistogramType,
    rocksdb_titandb_TickerType,
};

use crate::util;

pub trait TickerType {
    fn r#type(&self) -> u32;
}

pub type Tickers = rocksdb_Tickers;
pub type TitanTickers = rocksdb_titandb_TickerType;

impl TickerType for Tickers {
    #[inline]
    fn r#type(&self) -> u32 {
        *self as u32
    }
}

impl TickerType for TitanTickers {
    #[inline]
    fn r#type(&self) -> u32 {
        *self as u32
    }
}

pub trait HistogramType {
    fn r#type(&self) -> u32;
}

pub type Histograms = rocksdb_Histograms;
pub type TitanHistograms = rocksdb_titandb_HistogramType;

impl HistogramType for Histograms {
    #[inline]
    fn r#type(&self) -> u32 {
        *self as u32
    }
}

impl HistogramType for TitanHistograms {
    #[inline]
    fn r#type(&self) -> u32 {
        *self as u32
    }
}

pub type HistogramData = rocksdb_HistogramData;

pub struct Statistics {
    ptr: *mut tirocks_sys::crocksdb_statistics_t,
}

impl Default for Statistics {
    #[inline]
    fn default() -> Self {
        Self {
            ptr: unsafe { tirocks_sys::crocksdb_statistics_create() },
        }
    }
}

impl Statistics {
    #[inline]
    pub fn take_ticker(&self, ticker_type: impl TickerType) -> u64 {
        unsafe {
            tirocks_sys::crocksdb_statistics_get_and_reset_ticker_count(
                self.ptr,
                ticker_type.r#type(),
            )
        }
    }

    #[inline]
    pub fn ticker(&self, ticker_type: impl TickerType) -> u64 {
        self.ticker_raw(ticker_type.r#type())
    }

    #[inline]
    pub fn ticker_raw(&self, ticker_type: u32) -> u64 {
        unsafe { tirocks_sys::crocksdb_statistics_get_ticker_count(self.ptr, ticker_type) }
    }

    #[inline]
    pub fn histogram(&self, histogram_type: impl HistogramType) -> HistogramData {
        self.histogram_raw(histogram_type.r#type())
    }

    #[inline]
    pub fn histogram_raw(&self, histogram_type: u32) -> HistogramData {
        unsafe {
            let mut data = MaybeUninit::uninit();
            tirocks_sys::crocksdb_statistics_get_histogram(
                self.ptr,
                histogram_type,
                data.as_mut_ptr(),
            );
            data.assume_init()
        }
    }

    /// Resets all ticker and histogram stats
    #[inline]
    pub fn reset(&self) {
        unsafe { tirocks_sys::crocksdb_statistics_reset(self.ptr) }
    }

    #[inline]
    pub(crate) fn as_mut_ptr(&self) -> *mut tirocks_sys::crocksdb_statistics_t {
        self.ptr
    }
}

impl Display for Statistics {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut res = Ok(());
        let mut receiver = |buf: &[u8]| {
            res = write!(f, "{}", String::from_utf8_lossy(buf));
        };
        unsafe {
            let (ctx, fp) = util::wrap_string_receiver(&mut receiver);
            tirocks_sys::crocksdb_statistics_get_string(self.ptr, ctx, Some(fp));
        }
        res
    }
}

impl Drop for Statistics {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            tirocks_sys::crocksdb_statistics_destroy(self.ptr);
        }
    }
}
