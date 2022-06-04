// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::str::FromStr;

use serde::{de::Visitor, Deserialize, Serialize};

type CompressionType = crate::rocksdb_CompressionType;
type TitanBlobRunMode = crate::rocksdb_titandb_TitanBlobRunMode;

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

struct CompressionTypeVisitor;

impl<'de> Visitor<'de> for CompressionTypeVisitor {
    type Value = CompressionType;

    #[inline]
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let c = match v {
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
            _ => return Err(E::custom(format!("unknown compression: {}", v))),
        };
        Ok(c)
    }

    #[inline]
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an one of string { \"no\", \"snappy\", \"zlib\", \"bz2\", \"lz4\", \"lz4hc\", \"xpress\", \"zstd\", \"zstd-not-final\" }")
    }
}

impl<'de> Deserialize<'de> for CompressionType {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(CompressionTypeVisitor)
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
        match s {
            "normal" => Ok(TitanBlobRunMode::kNormal),
            "read-only" => Ok(TitanBlobRunMode::kReadOnly),
            "fallback" => Ok(TitanBlobRunMode::kFallback),
            _ => return Err(format!("unknown run mode: {}", s)),
        }
    }
}

struct TitanBlobRunModeVisitor;

impl<'de> Visitor<'de> for TitanBlobRunModeVisitor {
    type Value = TitanBlobRunMode;

    #[inline]
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        v.parse().map_err(E::custom)
    }

    #[inline]
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an one of string { \"normal\", \"read-only\", \"fallback\" }")
    }
}

impl<'de> Deserialize<'de> for TitanBlobRunMode {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(TitanBlobRunModeVisitor)
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

    use super::{CompressionType, TitanBlobRunMode};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Config {
        compressions: [CompressionType; 10],
        titan_blob_run_mode: [TitanBlobRunMode; 3],
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
compaction_pri = [0, 1, 2, 3]
rate_limiter_mode = [0, 1, 2]
compaction_style = [0, 1, 2, 3]
wal_recover_mode = [0, 1, 2, 3]
"#;
        let c: Config = toml::from_str(cfg_str_num).unwrap();
        assert_eq!(c, cfg);
    }
}
