// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::marker::PhantomData;
use std::str::FromStr;

use serde::{de::Visitor, Deserialize, Serialize};

type CompressionType = crate::rocksdb_CompressionType;
type TitanBlobRunMode = crate::rocksdb_titandb_TitanBlobRunMode;
type LogLevel = crate::rocksdb_InfoLogLevel;
type PrepopulateBlockCache = crate::rocksdb_BlockBasedTableOptions_PrepopulateBlockCache;
type ChecksumType = crate::rocksdb_ChecksumType;

struct FromStrVisitor<Item> {
    candidate: &'static str,
    _phantom: PhantomData<Item>,
}

impl<Item> FromStrVisitor<Item> {
    #[inline]
    fn new(candidate: &'static str) -> Self {
        Self {
            candidate,
            _phantom: PhantomData,
        }
    }
}

impl<'de, Item: FromStr<Err = String>> Visitor<'de> for FromStrVisitor<Item> {
    type Value = Item;

    #[inline]
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        v.parse().map_err(E::custom)
    }

    #[inline]
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "an one of string {{ {} }}", self.candidate)
    }
}

impl Serialize for CompressionType {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = match self {
            CompressionType::kNoCompression => "no",
            CompressionType::kSnappyCompression => "snappy",
            CompressionType::kZlibCompression => "zlib",
            CompressionType::kBZip2Compression => "bz2",
            CompressionType::kLZ4Compression => "lz4",
            CompressionType::kLZ4HCCompression => "lz4hc",
            CompressionType::kXpressCompression => "xpress",
            CompressionType::kZSTD => "zstd",
            CompressionType::kZSTDNotFinalCompression => "zstd-not-final",
            CompressionType::kDisableCompressionOption => "disable",
        };
        serializer.serialize_str(s)
    }
}

impl FromStr for CompressionType {
    type Err = String;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let c = match s {
            "no" => CompressionType::kNoCompression,
            "snappy" => CompressionType::kSnappyCompression,
            "zlib" => CompressionType::kZlibCompression,
            "bz2" => CompressionType::kBZip2Compression,
            "lz4" => CompressionType::kLZ4Compression,
            "lz4hc" => CompressionType::kLZ4HCCompression,
            "xpress" => CompressionType::kXpressCompression,
            "zstd" => CompressionType::kZSTD,
            "zstd-not-final" => CompressionType::kZSTDNotFinalCompression,
            "disable" => CompressionType::kDisableCompressionOption,
            _ => return Err(format!("unknown compression type: {}", s)),
        };
        Ok(c)
    }
}

impl<'de> Deserialize<'de> for CompressionType {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(FromStrVisitor::<CompressionType>::new("\"no\", \"snappy\", \"zlib\", \"bz2\", \"lz4\", \"lz4hc\", \"xpress\", \"zstd\", \"zstd-not-final\""))
    }
}

impl Serialize for TitanBlobRunMode {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = match self {
            TitanBlobRunMode::kNormal => "normal",
            TitanBlobRunMode::kReadOnly => "read-only",
            TitanBlobRunMode::kFallback => "fallback",
        };
        serializer.serialize_str(s)
    }
}

impl FromStr for TitanBlobRunMode {
    type Err = String;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let m = match s {
            "normal" => TitanBlobRunMode::kNormal,
            "read-only" => TitanBlobRunMode::kReadOnly,
            "fallback" => TitanBlobRunMode::kFallback,
            _ => return Err(format!("unknown run mode: {}", s)),
        };
        Ok(m)
    }
}

impl<'de> Deserialize<'de> for TitanBlobRunMode {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(FromStrVisitor::<TitanBlobRunMode>::new(
            "\"normal\", \"read-only\", \"fallback\"",
        ))
    }
}

impl Serialize for LogLevel {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = match self {
            LogLevel::DEBUG_LEVEL => "debug",
            LogLevel::INFO_LEVEL => "info",
            LogLevel::WARN_LEVEL => "warn",
            LogLevel::ERROR_LEVEL => "error",
            LogLevel::FATAL_LEVEL => "fatal",
            LogLevel::HEADER_LEVEL => "header",
            LogLevel::NUM_INFO_LOG_LEVELS => "num-info-log-levels",
        };
        serializer.serialize_str(s)
    }
}

impl FromStr for LogLevel {
    type Err = String;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let c = match s {
            "debug" => LogLevel::DEBUG_LEVEL,
            "info" => LogLevel::INFO_LEVEL,
            "warn" => LogLevel::WARN_LEVEL,
            "error" => LogLevel::ERROR_LEVEL,
            "fatal" => LogLevel::FATAL_LEVEL,
            _ => return Err(format!("unknown LogLevel: {}", s)),
        };
        Ok(c)
    }
}

impl<'de> Deserialize<'de> for LogLevel {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(FromStrVisitor::<LogLevel>::new(
            "\"debug\", \"info\", \"warn\", \"error\", \"fatal\"",
        ))
    }
}

impl Serialize for PrepopulateBlockCache {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = match self {
            PrepopulateBlockCache::kDisable => "disable",
            PrepopulateBlockCache::kFlushOnly => "flush-only",
        };
        serializer.serialize_str(s)
    }
}

impl FromStr for PrepopulateBlockCache {
    type Err = String;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let c = match s {
            "disabled" | "disable" => PrepopulateBlockCache::kDisable,
            "flush-only" => PrepopulateBlockCache::kFlushOnly,
            _ => return Err(format!("unknown PrepopulateBlockCache: {}", s)),
        };
        Ok(c)
    }
}

impl<'de> Deserialize<'de> for PrepopulateBlockCache {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(FromStrVisitor::<PrepopulateBlockCache>::new(
            "\"disabled\", \"flush-only\"",
        ))
    }
}

impl Serialize for ChecksumType {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = match self {
            ChecksumType::kNoChecksum => "no",
            ChecksumType::kCRC32c => "crc32c",
            ChecksumType::kxxHash => "xxhash",
            ChecksumType::kxxHash64 => "xxhash64",
            ChecksumType::kXXH3 => "xxh3",
        };
        serializer.serialize_str(s)
    }
}

impl FromStr for ChecksumType {
    type Err = String;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let c = match s {
            "no" => ChecksumType::kNoChecksum,
            "crc32c" => ChecksumType::kCRC32c,
            "xxhash" => ChecksumType::kxxHash,
            "xxhash64" => ChecksumType::kxxHash64,
            "xxh3" => ChecksumType::kXXH3,
            _ => return Err(format!("unknown checksumtype: {}", s)),
        };
        Ok(c)
    }
}

impl<'de> Deserialize<'de> for ChecksumType {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(FromStrVisitor::<ChecksumType>::new(
            "\"no\", \"crc32c\", \"xxhash\", \"xxhash64\", \"xxh3\"",
        ))
    }
}

macro_rules! numberic_enum {
    ($($vis:ident $enum:ident { $($variant:ident = ($value:expr, $repr:expr), )* })*) => {
        $(
            use crate::$enum;

            struct $vis;

            impl<'de> Visitor<'de> for $vis {
                type Value = $enum;

                #[inline]
                fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                where
                        E: serde::de::Error, {
                    match v {
                        $(
                            $repr => Ok($enum::$variant),
                        )*
                        _ => Err(E::custom(format!("unknown value {}", v))),
                    }
                }

                #[inline]
                fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
                where
                E: serde::de::Error, {
                    match v {
                        $(
                            $value => Ok($enum::$variant),
                        )*
                        _ => Err(E::custom(format!("unknown value {}", v))),
                    }
                }

                #[inline]
                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    formatter.write_str("failed to parse the value")
                }
            }

            impl Serialize for $enum {
                #[inline]
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer {
                    let v = match self {
                        $(
                            $enum::$variant => $repr,
                        )*
                    };
                    serializer.serialize_str(v)
                }
            }

            impl<'de> Deserialize<'de> for $enum {
                #[inline]
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                        D: serde::Deserializer<'de> {
                    deserializer.deserialize_str($vis)
                }
            }
        )*
    };
}

numberic_enum! {
    CompactionPriVisitor rocksdb_CompactionPri {
        kByCompensatedSize = (0, "by-compensated-size"),
        kOldestLargestSeqFirst = (1, "oldest-largest-seq-first"),
        kOldestSmallestSeqFirst = (2, "oldest-smallest-seq-first"),
        kMinOverlappingRatio = (3, "min-overlapping-ratio"),
    }

    RateLimiterModeVisitor rocksdb_RateLimiter_Mode {
        kReadsOnly = (0, "read-only"),
        kWritesOnly = (1, "write-only"),
        kAllIo = (2, "all-io"),
    }

    CompactionStyleVisitor rocksdb_CompactionStyle {
        kCompactionStyleLevel = (0, "level"),
        kCompactionStyleUniversal = (1, "universal"),
        kCompactionStyleFIFO = (2, "fifo"),
        kCompactionStyleNone = (3, "none"),
    }

    WalRecoveryModeVisitor rocksdb_WALRecoveryMode {
        kTolerateCorruptedTailRecords = (0, "tolerate-corrupted-tail-records"),
        kAbsoluteConsistency = (1, "absolute-consistency"),
        kPointInTimeRecovery = (2, "point-in-time"),
        kSkipAnyCorruptedRecords = (3, "skip-any-corrupted-records"),
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use crate::{
        rocksdb_CompactionPri, rocksdb_CompactionStyle, rocksdb_RateLimiter_Mode,
        rocksdb_WALRecoveryMode,
    };

    use super::*;

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Config {
        compressions: [CompressionType; 10],
        titan_blob_run_mode: [TitanBlobRunMode; 3],
        log_level: [LogLevel; 5],
        prepopulate_block_cache: [PrepopulateBlockCache; 2],
        checksum_type: [ChecksumType; 5],
        compaction_pri: [rocksdb_CompactionPri; 4],
        rate_limiter_mode: [rocksdb_RateLimiter_Mode; 3],
        compaction_style: [rocksdb_CompactionStyle; 4],
        wal_recover_mode: [rocksdb_WALRecoveryMode; 4],
    }

    #[test]
    fn test_serde() {
        let cfg = Config {
            compressions: [
                CompressionType::kNoCompression,
                CompressionType::kSnappyCompression,
                CompressionType::kZlibCompression,
                CompressionType::kBZip2Compression,
                CompressionType::kLZ4Compression,
                CompressionType::kLZ4HCCompression,
                CompressionType::kXpressCompression,
                CompressionType::kZSTD,
                CompressionType::kZSTDNotFinalCompression,
                CompressionType::kDisableCompressionOption,
            ],
            titan_blob_run_mode: [
                TitanBlobRunMode::kNormal,
                TitanBlobRunMode::kReadOnly,
                TitanBlobRunMode::kFallback,
            ],
            log_level: [
                LogLevel::ERROR_LEVEL,
                LogLevel::INFO_LEVEL,
                LogLevel::WARN_LEVEL,
                LogLevel::ERROR_LEVEL,
                LogLevel::FATAL_LEVEL,
            ],
            prepopulate_block_cache: [
                PrepopulateBlockCache::kDisable,
                PrepopulateBlockCache::kFlushOnly,
            ],
            checksum_type: [
                ChecksumType::kNoChecksum,
                ChecksumType::kCRC32c,
                ChecksumType::kxxHash,
                ChecksumType::kxxHash64,
                ChecksumType::kXXH3,
            ],
            compaction_pri: [
                rocksdb_CompactionPri::kByCompensatedSize,
                rocksdb_CompactionPri::kOldestLargestSeqFirst,
                rocksdb_CompactionPri::kOldestSmallestSeqFirst,
                rocksdb_CompactionPri::kMinOverlappingRatio,
            ],
            rate_limiter_mode: [
                rocksdb_RateLimiter_Mode::kReadsOnly,
                rocksdb_RateLimiter_Mode::kWritesOnly,
                rocksdb_RateLimiter_Mode::kAllIo,
            ],
            compaction_style: [
                rocksdb_CompactionStyle::kCompactionStyleLevel,
                rocksdb_CompactionStyle::kCompactionStyleUniversal,
                rocksdb_CompactionStyle::kCompactionStyleFIFO,
                rocksdb_CompactionStyle::kCompactionStyleNone,
            ],
            wal_recover_mode: [
                rocksdb_WALRecoveryMode::kTolerateCorruptedTailRecords,
                rocksdb_WALRecoveryMode::kAbsoluteConsistency,
                rocksdb_WALRecoveryMode::kPointInTimeRecovery,
                rocksdb_WALRecoveryMode::kSkipAnyCorruptedRecords,
            ],
        };
        let cfg_str = r#"compressions = ["no", "snappy", "zlib", "bz2", "lz4", "lz4hc", "xpress", "zstd", "zstd-not-final", "disable"]
titan_blob_run_mode = ["normal", "read-only", "fallback"]
log_level = ["error", "info", "warn", "error", "fatal"]
prepopulate_block_cache = ["disable", "flush-only"]
checksum_type = ["no", "crc32c", "xxhash", "xxhash64", "xxh3"]
compaction_pri = ["by-compensated-size", "oldest-largest-seq-first", "oldest-smallest-seq-first", "min-overlapping-ratio"]
rate_limiter_mode = ["read-only", "write-only", "all-io"]
compaction_style = ["level", "universal", "fifo", "none"]
wal_recover_mode = ["tolerate-corrupted-tail-records", "absolute-consistency", "point-in-time", "skip-any-corrupted-records"]
"#;
        let c: Config = toml::from_str(cfg_str).unwrap();
        assert_eq!(c, cfg);
        let s = toml::to_string(&cfg).unwrap();
        assert_eq!(s, cfg_str);
        let cfg_str_num = r#"
compressions = ["no", "snappy", "zlib", "bz2", "lz4", "lz4hc", "xpress", "zstd", "zstd-not-final", "disable"]
titan_blob_run_mode = ["normal", "read-only", "fallback"]
log_level = ["error", "info", "warn", "error", "fatal"]
prepopulate_block_cache = ["disable", "flush-only"]
checksum_type = ["no", "crc32c", "xxhash", "xxhash64", "xxh3"]
compaction_pri = [0, 1, 2, 3]
rate_limiter_mode = [0, 1, 2]
compaction_style = [0, 1, 2, 3]
wal_recover_mode = [0, 1, 2, 3]
"#;
        let c: Config = toml::from_str(cfg_str_num).unwrap();
        assert_eq!(c, cfg);
    }
}
