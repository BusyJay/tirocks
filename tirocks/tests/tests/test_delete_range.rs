// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::{ffi::CStr, fs, path::Path};

use crc::crc32::{self, Digest, Hasher32};
use tirocks::{
    db::{DefaultCfOnlyBuilder, RawColumnFamilyHandle},
    env::EnvOptions,
    option::{
        CompactRangeOptions, FlushOptions, IngestExternalFileOptions, RawOptions, ReadOptions,
        WriteOptions,
    },
    slice_transform::{SliceTransform, SysSliceTransform},
    table::sst::{ExternalSstFileInfo, SstFileWriter},
    CfOptions, Db, OpenOptions, WriteBatch,
};

use super::tempdir_with_prefix;

fn gen_sst(opt: &RawOptions, cf: Option<&RawColumnFamilyHandle>, path: &Path) {
    let _ = fs::remove_file(path);
    let env_opt = EnvOptions::default();
    let mut writer = SstFileWriter::new(&env_opt, opt, cf);
    writer.open(path).unwrap();
    writer.put(b"key1", b"value1").unwrap();
    writer.put(b"key2", b"value2").unwrap();
    writer.put(b"key3", b"value3").unwrap();
    writer.put(b"key4", b"value4").unwrap();
    writer.finish(None).unwrap();
}

pub fn gen_sst_from_db(opt: &RawOptions, cf: &RawColumnFamilyHandle, path: &Path, db: &Db) {
    let _ = fs::remove_file(path);
    let env_opt = EnvOptions::default();
    let mut writer = SstFileWriter::new(&env_opt, opt, Some(cf));
    writer.open(path).unwrap();
    let mut read_opt = ReadOptions::default();
    let mut iter = db.iter(&mut read_opt, cf);
    iter.seek_to_first();
    while iter.valid() {
        writer.put(iter.key(), iter.value()).unwrap();
        iter.next();
    }
    iter.check().unwrap();
    let mut info = ExternalSstFileInfo::default();
    writer.finish(Some(&mut info)).unwrap();
    iter.seek_to_first();
    assert!(iter.valid());
    assert_eq!(iter.key(), info.smallest_key());
    iter.seek_to_last();
    assert!(iter.valid());
    assert_eq!(iter.key(), info.largest_key());
    assert_eq!(info.sequence_number(), 0);
    assert_eq!(info.file_path().map(Path::new), Ok(path));
    assert_ne!(info.file_size(), 0);
    assert_ne!(info.num_entries(), 0);
}

fn gen_crc32_from_db(db: &Db) -> u32 {
    let mut digest = Digest::new(crc32::IEEE);
    let mut read_opt = ReadOptions::default();
    let cf = db.default_cf();
    let mut iter = db.iter(&mut read_opt, cf);
    iter.seek_to_first();
    while iter.valid() {
        digest.write(iter.key());
        digest.write(iter.value());
        iter.next();
    }
    iter.check().unwrap();
    digest.sum32()
}

fn gen_crc32_from_db_in_range(db: &Db, start_key: &[u8], end_key: &[u8]) -> u32 {
    let mut digest = Digest::new(crc32::IEEE);
    let mut read_opt = ReadOptions::default();
    let cf = db.default_cf();
    let mut iter = db.iter(&mut read_opt, cf);
    iter.seek(start_key);
    while iter.valid() {
        if iter.key() >= end_key {
            break;
        }
        digest.write(iter.key());
        digest.write(iter.value());
        iter.next();
    }
    iter.check().unwrap();
    digest.sum32()
}

#[test]
fn test_delete_range_case_1() {
    let path = tempdir_with_prefix("_rust_rocksdb_test_delete_range_case_1");
    let db = DefaultCfOnlyBuilder::default()
        .set_create_if_missing(true)
        .open(path.path())
        .unwrap();
    let samples_a = vec![
        (b"key1", b"value1"),
        (b"key2", b"value2"),
        (b"key3", b"value3"),
        (b"key4", b"value4"),
    ];
    let write_opt = WriteOptions::default();
    let cf = db.default_cf();
    let read_opt = ReadOptions::default();
    for (k, v) in samples_a {
        db.put(&write_opt, cf, k, v).unwrap();
        assert_eq!(Ok(Some(v.to_vec())), db.get(&read_opt, cf, k));
    }
    let before = gen_crc32_from_db(&db);

    let gen_path = tempdir_with_prefix("_rust_rocksdb_case_1_ingest_sst_gen");
    let test_sstfile = gen_path.path().join("test_sst_file");
    let ingest_opt = IngestExternalFileOptions::default();

    let default_options = db.cf_options(cf);
    gen_sst_from_db(&default_options, cf, &test_sstfile, &db);

    db.delete_range(&write_opt, cf, b"key1", b"key5").unwrap();
    check_kv(
        &db,
        cf,
        &[
            (b"key1", None),
            (b"key2", None),
            (b"key3", None),
            (b"key4", None),
        ],
    );

    db.ingest_external_file(&ingest_opt, cf, &[test_sstfile])
        .unwrap();
    check_kv(
        &db,
        cf,
        &[
            (b"key1", Some(b"value1")),
            (b"key2", Some(b"value2")),
            (b"key3", Some(b"value3")),
            (b"key4", Some(b"value4")),
        ],
    );

    let after = gen_crc32_from_db(&db);
    assert_eq!(before, after);
}

#[test]
fn test_delete_range_case_2() {
    let path = tempdir_with_prefix("_rust_rocksdb_test_delete_range_case_2_1");
    let mut builder = DefaultCfOnlyBuilder::default();
    let db = builder
        .set_create_if_missing(true)
        .open(path.path())
        .unwrap();
    let samples_a = vec![
        (b"key1", b"value1"),
        (b"key2", b"value2"),
        (b"key3", b"value3"),
        (b"key4", b"value4"),
    ];
    let write_opt = WriteOptions::default();
    let cf = db.default_cf();
    let read_opt = ReadOptions::default();
    for (k, v) in samples_a {
        db.put(&write_opt, cf, k, v).unwrap();
        assert_eq!(db.get(&read_opt, cf, k), Ok(Some(v.to_vec())));
    }
    let before = gen_crc32_from_db(&db);

    let gen_path = tempdir_with_prefix("_rust_rocksdb_case_2_ingest_sst_gen");
    let test_sstfile = gen_path.path().join("test_sst_file");
    let ingest_opt = IngestExternalFileOptions::default();

    let default_options = db.cf_options(cf);
    gen_sst_from_db(&default_options, cf, &test_sstfile, &db);

    db.delete_range(&write_opt, cf, b"key1", b"key5").unwrap();
    check_kv(
        &db,
        cf,
        &[
            (b"key1", None),
            (b"key2", None),
            (b"key3", None),
            (b"key4", None),
        ],
    );

    let path = tempdir_with_prefix("_rust_rocksdb_test_delete_range_case_2_2");
    let db2 = builder.open(path.path()).unwrap();
    let cf = db2.default_cf();
    db2.ingest_external_file(&ingest_opt, cf, &[test_sstfile])
        .unwrap();
    check_kv(
        &db2,
        cf,
        &[
            (b"key1", Some(b"value1")),
            (b"key2", Some(b"value2")),
            (b"key3", Some(b"value3")),
            (b"key4", Some(b"value4")),
        ],
    );

    let after = gen_crc32_from_db(&db2);
    assert_eq!(before, after);
}

#[test]
fn test_delete_range_case_3() {
    let path = tempdir_with_prefix("_rust_rocksdb_test_delete_range_case_3");
    let mut builder = DefaultCfOnlyBuilder::default();
    builder.set_create_if_missing(true);
    let db = builder.open(path.path()).unwrap();
    let samples_a = vec![
        (b"key1", b"value1"),
        (b"key2", b"value2"),
        (b"key3", b"value3"),
        (b"key4", b"value4"),
        (b"key5", b"value5"),
    ];
    let write_opt = WriteOptions::default();
    let cf = db.default_cf();
    let read_opt = ReadOptions::default();
    for (k, v) in samples_a {
        db.put(&write_opt, cf, k, v).unwrap();
        assert_eq!(db.get(&read_opt, cf, k), Ok(Some(v.to_vec())));
    }
    let before = gen_crc32_from_db(&db);

    db.delete_range(&write_opt, cf, b"key2", b"key4").unwrap();

    check_kv(
        &db,
        cf,
        &[
            (b"key1", Some(b"value1")),
            (b"key2", None),
            (b"key3", None),
            (b"key4", Some(b"value4")),
            (b"key5", Some(b"value5")),
        ],
    );

    let path2 = tempdir_with_prefix("_rust_rocksdb_test_delete_range_case_3_2");
    let db2 = builder.open(path2.path()).unwrap();
    let cf2 = db2.default_cf();

    let samples_b = vec![(b"key2", b"value2"), (b"key3", b"value3")];
    for (k, v) in samples_b {
        db2.put(&write_opt, cf2, k, v).unwrap();
        assert_eq!(db2.get(&read_opt, cf2, k), Ok(Some(v.to_vec())));
    }

    let gen_path = tempdir_with_prefix("_rust_rocksdb_case_3_ingest_sst_gen");
    let test_sstfile = gen_path.path().join("test_sst_file");
    let ingest_opt = IngestExternalFileOptions::default();

    let default_options = db2.cf_options(cf2);
    gen_sst_from_db(&default_options, cf2, &test_sstfile, &db2);

    db.ingest_external_file(&ingest_opt, cf, &[test_sstfile])
        .unwrap();
    check_kv(
        &db,
        cf,
        &[
            (b"key1", Some(b"value1")),
            (b"key2", Some(b"value2")),
            (b"key3", Some(b"value3")),
            (b"key4", Some(b"value4")),
            (b"key5", Some(b"value5")),
        ],
    );

    let after = gen_crc32_from_db(&db);
    assert_eq!(before, after);
}

#[test]
fn test_delete_range_case_4() {
    let path = tempdir_with_prefix("_rust_rocksdb_test_delete_range_case_4");
    let mut builder = DefaultCfOnlyBuilder::default();
    let db = builder
        .set_create_if_missing(true)
        .open(path.path())
        .unwrap();
    let samples_a = vec![
        (b"key1", b"value1"),
        (b"key2", b"value2"),
        (b"key3", b"value3"),
        (b"key4", b"value4"),
        (b"key5", b"value5"),
    ];
    let write_opt = WriteOptions::default();
    let cf = db.default_cf();
    let mut read_opt = ReadOptions::default();
    for (k, v) in samples_a {
        db.put(&write_opt, cf, k, v).unwrap();
        assert_eq!(Ok(Some(v.to_vec())), db.get(&mut read_opt, cf, k));
    }
    let before = gen_crc32_from_db(&db);

    db.delete_range(&write_opt, cf, b"key4", b"key6").unwrap();

    check_kv(
        &db,
        cf,
        &[
            (b"key1", Some(b"value1")),
            (b"key2", Some(b"value2")),
            (b"key3", Some(b"value3")),
            (b"key4", None),
            (b"key5", None),
        ],
    );

    let path = tempdir_with_prefix("_rust_rocksdb_test_delete_range_case_4_2");
    let db2 = builder.open(path.path()).unwrap();
    let cf2 = db2.default_cf();

    let samples_b = vec![(b"key4", b"value4"), (b"key5", b"value5")];
    for (k, v) in samples_b {
        db2.put(&write_opt, cf2, k, v).unwrap();
        assert_eq!(Ok(Some(v.to_vec())), db2.get(&read_opt, cf2, k));
    }

    let gen_path = tempdir_with_prefix("_rust_rocksdb_case_4_ingest_sst_gen");
    let test_sstfile = gen_path.path().join("test_sst_file");
    let ingest_opt = IngestExternalFileOptions::default();

    let default_options = db2.cf_options(cf2);
    gen_sst_from_db(&default_options, cf2, &test_sstfile, &db2);

    db.ingest_external_file(&ingest_opt, cf, &[test_sstfile])
        .unwrap();
    check_kv(
        &db,
        cf,
        &[
            (b"key1", Some(b"value1")),
            (b"key2", Some(b"value2")),
            (b"key3", Some(b"value3")),
            (b"key4", Some(b"value4")),
            (b"key5", Some(b"value5")),
        ],
    );

    let after = gen_crc32_from_db(&db);
    assert_eq!(before, after);
}

#[test]
fn test_delete_range_case_5() {
    let path = tempdir_with_prefix("_rust_rocksdb_test_delete_range_case_5");
    let mut builder = DefaultCfOnlyBuilder::default();
    let db = builder
        .set_create_if_missing(true)
        .open(path.path())
        .unwrap();
    let samples_a = vec![
        (b"key1", b"value1"),
        (b"key2", b"value2"),
        (b"key3", b"value3"),
        (b"key4", b"value4"),
        (b"key5", b"value5"),
    ];
    let write_opt = WriteOptions::default();
    let read_opt = ReadOptions::default();
    let cf = db.default_cf();
    for (k, v) in samples_a {
        db.put(&write_opt, cf, k, v).unwrap();
        assert_eq!(Ok(Some(v.to_vec())), db.get(&read_opt, cf, k));
    }

    db.delete_range(&write_opt, cf, b"key1", b"key6").unwrap();

    check_kv(
        &db,
        cf,
        &[
            (b"key1", None),
            (b"key2", None),
            (b"key3", None),
            (b"key4", None),
            (b"key5", None),
        ],
    );

    let path2 = tempdir_with_prefix("_rust_rocksdb_test_delete_range_case_5_2");
    let db2 = builder.open(path2.path()).unwrap();
    let cf2 = db2.default_cf();

    let samples_b = vec![(b"key4", b"value4"), (b"key5", b"value5")];
    for (k, v) in samples_b {
        db2.put(&write_opt, cf2, k, v).unwrap();
        assert_eq!(Ok(Some(v.to_vec())), db2.get(&read_opt, cf2, k));
    }
    let before = gen_crc32_from_db(&db2);

    let gen_path = tempdir_with_prefix("_rust_rocksdb_case_5_ingest_sst_gen");
    let test_sstfile = gen_path.path().join("test_sst_file");
    let ingest_opt = IngestExternalFileOptions::default();

    let default_options = db2.cf_options(cf2);
    gen_sst_from_db(&default_options, cf2, &test_sstfile, &db2);

    db.ingest_external_file(&ingest_opt, cf, &[test_sstfile])
        .unwrap();
    check_kv(
        &db,
        cf,
        &[(b"key4", Some(b"value4")), (b"key5", Some(b"value5"))],
    );

    let after = gen_crc32_from_db(&db);
    assert_eq!(before, after);
}

#[test]
fn test_delete_range_case_6() {
    let path = tempdir_with_prefix("_rust_rocksdb_test_delete_range_case_6");
    let mut builder = DefaultCfOnlyBuilder::default();
    let db = builder
        .set_create_if_missing(true)
        .open(path.path())
        .unwrap();
    let write_opt = WriteOptions::default();
    let read_opt = ReadOptions::default();
    let cf = db.default_cf();
    let samples_a = vec![
        (b"key1", b"value1"),
        (b"key2", b"value2"),
        (b"key3", b"value3"),
        (b"key4", b"value4"),
        (b"key5", b"value5"),
    ];
    for (k, v) in samples_a {
        db.put(&write_opt, cf, k, v).unwrap();
        assert_eq!(Ok(Some(v.to_vec())), db.get(&read_opt, cf, k));
    }

    let before = gen_crc32_from_db_in_range(&db, b"key4", b"key6");

    db.delete_range(&write_opt, cf, b"key1", b"key4").unwrap();

    check_kv(
        &db,
        cf,
        &[
            (b"key1", None),
            (b"key2", None),
            (b"key3", None),
            (b"key4", Some(b"value4")),
            (b"key5", Some(b"value5")),
        ],
    );

    let path = tempdir_with_prefix("_rust_rocksdb_test_delete_range_case_5_2");
    let db2 = builder.open(path.path()).unwrap();
    let cf2 = db2.default_cf();

    let samples_b = vec![
        (b"key1", b"value1"),
        (b"key2", b"value2"),
        (b"key3", b"value3"),
    ];
    for (k, v) in samples_b {
        db2.put(&write_opt, cf2, k, v).unwrap();
        assert_eq!(Ok(Some(v.to_vec())), db2.get(&read_opt, cf2, k));
    }

    let gen_path = tempdir_with_prefix("_rust_rocksdb_case_6_ingest_sst_gen");
    let test_sstfile = gen_path.path().join("test_sst_file");
    let ingest_opt = IngestExternalFileOptions::default();

    let default_options = db2.cf_options(cf2);
    gen_sst_from_db(&default_options, cf2, &test_sstfile, &db2);

    db.ingest_external_file(&ingest_opt, cf, &[test_sstfile])
        .unwrap();
    check_kv(
        &db,
        cf,
        &[
            (b"key1", Some(b"value1")),
            (b"key2", Some(b"value2")),
            (b"key3", Some(b"value3")),
            (b"key4", Some(b"value4")),
            (b"key5", Some(b"value5")),
        ],
    );

    let after = gen_crc32_from_db_in_range(&db, b"key4", b"key6");
    assert_eq!(before, after);
}

#[test]
fn test_delete_range_compact() {
    let path = tempdir_with_prefix("_rust_rocksdb_test_delete_range_case_6");
    let mut builder = DefaultCfOnlyBuilder::default();
    let db = builder
        .set_create_if_missing(true)
        .open(path.path())
        .unwrap();
    let write_opt = WriteOptions::default();
    let read_opt = ReadOptions::default();
    let cf = db.default_cf();
    let samples_a = vec![
        (b"key1", b"value1"),
        (b"key2", b"value2"),
        (b"key3", b"value3"),
        (b"key4", b"value4"),
        (b"key5", b"value5"),
    ];
    for (k, v) in samples_a {
        db.put(&write_opt, cf, k, v).unwrap();
        assert_eq!(Ok(Some(v.to_vec())), db.get(&read_opt, cf, k));
    }

    let before = gen_crc32_from_db_in_range(&db, b"key4", b"key6");

    db.delete_range(&write_opt, cf, b"key1", b"key4").unwrap();

    check_kv(
        &db,
        cf,
        &[
            (b"key1", None),
            (b"key2", None),
            (b"key3", None),
            (b"key4", Some(b"value4")),
            (b"key5", Some(b"value5")),
        ],
    );

    let path = tempdir_with_prefix("_rust_rocksdb_test_delete_range_case_5_2");
    let db2 = builder.open(path.path()).unwrap();
    let cf2 = db2.default_cf();

    let samples_b = vec![
        (b"key1", b"value1"),
        (b"key2", b"value2"),
        (b"key3", b"value3"),
    ];
    for (k, v) in samples_b {
        db2.put(&write_opt, cf2, k, v).unwrap();
        assert_eq!(Ok(Some(v.to_vec())), db2.get(&read_opt, cf2, k));
    }

    let gen_path = tempdir_with_prefix("_rust_rocksdb_case_6_ingest_sst_gen");
    let test_sstfile = gen_path.path().join("test_sst_file");
    let ingest_opt = IngestExternalFileOptions::default();

    let default_options = db2.cf_options(cf2);
    gen_sst_from_db(&default_options, cf2, &test_sstfile, &db2);

    db.ingest_external_file(&ingest_opt, cf, &[test_sstfile])
        .unwrap();
    check_kv(
        &db,
        cf,
        &[
            (b"key1", Some(b"value1")),
            (b"key2", Some(b"value2")),
            (b"key3", Some(b"value3")),
            (b"key4", Some(b"value4")),
            (b"key5", Some(b"value5")),
        ],
    );

    let opt = CompactRangeOptions::default();
    db.compact_range(&opt, cf, None, None).unwrap();
    let after = gen_crc32_from_db_in_range(&db, b"key4", b"key6");
    assert_eq!(before, after);
}

pub struct FixedSuffixSliceTransform {
    pub suffix_len: usize,
}

impl FixedSuffixSliceTransform {
    pub fn new(suffix_len: usize) -> FixedSuffixSliceTransform {
        FixedSuffixSliceTransform {
            suffix_len: suffix_len,
        }
    }
}

impl SliceTransform for FixedSuffixSliceTransform {
    fn name(&self) -> &CStr {
        CStr::from_bytes_with_nul(b"FixedSuffix\0").unwrap()
    }

    fn transform<'a>(&self, key: &'a [u8]) -> &'a [u8] {
        let mid = key.len() - self.suffix_len;
        let (left, _) = key.split_at(mid);
        left
    }

    fn in_domain(&self, key: &[u8]) -> bool {
        key.len() >= self.suffix_len
    }
}

fn build_db_with_suffix_transform(path: &Path) -> Db {
    // Prefix extractor(trim the timestamp at tail) for write cf.
    let transform = SysSliceTransform::new(FixedSuffixSliceTransform::new(3));
    let mut builder = DefaultCfOnlyBuilder::default();
    builder
        .set_create_if_missing(true)
        .cf_options_mut()
        .set_prefix_extractor(&transform)
        // Create prefix bloom filter for memtable.
        .set_memtable_prefix_bloom_size_ratio(0.1);
    builder.open(path).unwrap()
}

#[test]
fn test_delete_range_prefix_bloom_case_1() {
    let path = tempdir_with_prefix("_rust_rocksdb_test_delete_range_prefix_bloom_case_1");
    let db = build_db_with_suffix_transform(path.path());
    let write_opt = WriteOptions::default();
    let read_opt = ReadOptions::default();
    let cf = db.default_cf();

    let samples_a = vec![
        (b"keya11111", b"value1"),
        (b"keyb22222", b"value2"),
        (b"keyc33333", b"value3"),
        (b"keyd44444", b"value4"),
    ];
    for (k, v) in samples_a {
        db.put(&write_opt, cf, k, v).unwrap();
        assert_eq!(Ok(Some(v.to_vec())), db.get(&read_opt, cf, k));
    }
    let before = gen_crc32_from_db(&db);

    let gen_path = tempdir_with_prefix("_rust_rocksdb_case_prefix_bloom_1_ingest_sst_gen");
    let test_sstfile = gen_path.path().join("test_sst_file");
    let ingest_opt = IngestExternalFileOptions::default();

    let default_options = db.cf_options(cf);
    gen_sst_from_db(&default_options, cf, &test_sstfile, &db);

    db.delete_range(&write_opt, cf, b"keya11111", b"keye55555")
        .unwrap();
    check_kv(
        &db,
        cf,
        &[
            (b"keya11111", None),
            (b"keyb22222", None),
            (b"keyc33333", None),
            (b"keyd44444", None),
        ],
    );

    db.ingest_external_file(&ingest_opt, cf, &[test_sstfile])
        .unwrap();
    check_kv(
        &db,
        cf,
        &[
            (b"keya11111", Some(b"value1")),
            (b"keyb22222", Some(b"value2")),
            (b"keyc33333", Some(b"value3")),
            (b"keyd44444", Some(b"value4")),
        ],
    );

    let after = gen_crc32_from_db(&db);
    assert_eq!(before, after);
}

#[test]
fn test_delete_range_prefix_bloom_case_2() {
    let path = tempdir_with_prefix("_rust_rocksdb_test_delete_range_prefix_bloom_case_2");
    let db = build_db_with_suffix_transform(path.path());
    let write_opt = WriteOptions::default();
    let read_opt = ReadOptions::default();
    let cf = db.default_cf();

    let samples_a = vec![
        (b"keya11111", b"value1"),
        (b"keyb22222", b"value2"),
        (b"keyc33333", b"value3"),
        (b"keyd44444", b"value4"),
    ];
    for (k, v) in samples_a {
        db.put(&write_opt, cf, k, v).unwrap();
        assert_eq!(Ok(Some(v.to_vec())), db.get(&read_opt, cf, k));
    }
    let before = gen_crc32_from_db(&db);

    let gen_path = tempdir_with_prefix("_rust_rocksdb_case_prefix_bloom_1_ingest_sst_gen");
    let test_sstfile = gen_path.path().join("test_sst_file");
    let ingest_opt = IngestExternalFileOptions::default();

    let default_options = db.cf_options(cf);
    gen_sst_from_db(&default_options, cf, &test_sstfile, &db);

    db.delete_range(&write_opt, cf, b"keya11111", b"keye55555")
        .unwrap();
    check_kv(
        &db,
        cf,
        &[
            (b"keya11111", None),
            (b"keyb22222", None),
            (b"keyc33333", None),
            (b"keyd44444", None),
        ],
    );

    let path2 = tempdir_with_prefix("_rust_rocksdb_test_delete_range_prefix_bloom_case_2_2");
    let db2 = build_db_with_suffix_transform(path2.path());
    let cf2 = db2.default_cf();

    db2.ingest_external_file(&ingest_opt, cf2, &[test_sstfile])
        .unwrap();
    check_kv(
        &db2,
        cf2,
        &[
            (b"keya11111", Some(b"value1")),
            (b"keyb22222", Some(b"value2")),
            (b"keyc33333", Some(b"value3")),
            (b"keyd44444", Some(b"value4")),
        ],
    );

    let after = gen_crc32_from_db(&db2);
    assert_eq!(before, after);
}

#[test]
fn test_delete_range_prefix_bloom_case_3() {
    let path = tempdir_with_prefix("_rust_rocksdb_test_delete_range_prefix_bloom_case_3");
    let db = build_db_with_suffix_transform(path.path());
    let write_opt = WriteOptions::default();
    let read_opt = ReadOptions::default();
    let cf = db.default_cf();
    let samples_a = vec![
        (b"keya11111", b"value1"),
        (b"keyb22222", b"value2"),
        (b"keyc33333", b"value3"),
        (b"keyd44444", b"value4"),
        (b"keye55555", b"value5"),
    ];
    for (k, v) in samples_a {
        db.put(&write_opt, cf, k, v).unwrap();
        assert_eq!(Ok(Some(v.to_vec())), db.get(&read_opt, cf, k));
    }
    let before = gen_crc32_from_db(&db);

    db.delete_range(&write_opt, cf, b"keyb22222", b"keyd44444")
        .unwrap();

    check_kv(
        &db,
        cf,
        &[
            (b"keya11111", Some(b"value1")),
            (b"keyb22222", None),
            (b"keyc33333", None),
            (b"keyd44444", Some(b"value4")),
            (b"keye55555", Some(b"value5")),
        ],
    );

    let path2 = tempdir_with_prefix("_rust_rocksdb_test_delete_range_prefix_bloom_case_3_2");
    let db2 = build_db_with_suffix_transform(path2.path());
    let cf2 = db2.default_cf();
    let samples_b = vec![(b"keyb22222", b"value2"), (b"keyc33333", b"value3")];
    for (k, v) in samples_b {
        db2.put(&write_opt, cf2, k, v).unwrap();
        assert_eq!(Ok(Some(v.to_vec())), db2.get(&read_opt, cf2, k));
    }

    let gen_path = tempdir_with_prefix("_rust_rocksdb_prefix_bloom_case_3_ingest_sst_gen");
    let test_sstfile = gen_path.path().join("test_sst_file");
    let ingest_opt = IngestExternalFileOptions::default();

    let default_options = db2.cf_options(cf2);
    gen_sst_from_db(&default_options, cf2, &test_sstfile, &db2);

    db.ingest_external_file(&ingest_opt, cf, &[test_sstfile])
        .unwrap();
    check_kv(
        &db,
        cf,
        &[
            (b"keya11111", Some(b"value1")),
            (b"keyb22222", Some(b"value2")),
            (b"keyc33333", Some(b"value3")),
            (b"keyd44444", Some(b"value4")),
            (b"keye55555", Some(b"value5")),
        ],
    );

    let after = gen_crc32_from_db(&db);
    assert_eq!(before, after);
}

#[test]
fn test_delete_range_prefix_bloom_case_4() {
    let path = tempdir_with_prefix("_rust_rocksdb_test_delete_range_prefix_bloom_case_4");
    let db = build_db_with_suffix_transform(path.path());
    let write_opt = WriteOptions::default();
    let read_opt = ReadOptions::default();
    let cf = db.default_cf();
    let samples_a = vec![
        (b"keya11111", b"value1"),
        (b"keyb22222", b"value2"),
        (b"keyc33333", b"value3"),
        (b"keyd44444", b"value4"),
        (b"keye55555", b"value5"),
    ];
    for (k, v) in samples_a {
        db.put(&write_opt, cf, k, v).unwrap();
        assert_eq!(Ok(Some(v.to_vec())), db.get(&read_opt, cf, k));
    }
    let before = gen_crc32_from_db(&db);

    db.delete_range(&write_opt, cf, b"keyd44444", b"keyf66666")
        .unwrap();

    check_kv(
        &db,
        cf,
        &[
            (b"keya11111", Some(b"value1")),
            (b"keyb22222", Some(b"value2")),
            (b"keyc33333", Some(b"value3")),
            (b"keyd44444", None),
            (b"keye55555", None),
        ],
    );

    let path2 = tempdir_with_prefix("_rust_rocksdb_test_delete_range_prefix_bloom_case_4_2");
    let db2 = build_db_with_suffix_transform(path2.path());
    let cf2 = db2.default_cf();

    let samples_b = vec![(b"keyd44444", b"value4"), (b"keye55555", b"value5")];
    for (k, v) in samples_b {
        db2.put(&write_opt, cf2, k, v).unwrap();
        assert_eq!(Ok(Some(v.to_vec())), db2.get(&read_opt, cf2, k));
    }

    let gen_path = tempdir_with_prefix("_rust_rocksdb_case_prefix_bloom_4_ingest_sst_gen");
    let test_sstfile = gen_path.path().join("test_sst_file");
    let ingest_opt = IngestExternalFileOptions::default();

    let default_options = db2.cf_options(cf2);
    gen_sst_from_db(&default_options, cf2, &test_sstfile, &db2);

    db.ingest_external_file(&ingest_opt, cf, &[test_sstfile])
        .unwrap();
    check_kv(
        &db,
        cf,
        &[
            (b"keya11111", Some(b"value1")),
            (b"keyb22222", Some(b"value2")),
            (b"keyc33333", Some(b"value3")),
            (b"keyd44444", Some(b"value4")),
            (b"keye55555", Some(b"value5")),
        ],
    );

    let after = gen_crc32_from_db(&db);
    assert_eq!(before, after);
}

#[test]
fn test_delete_range_prefix_bloom_case_5() {
    let path = tempdir_with_prefix("_rust_rocksdb_test_delete_range_prefix_bloom_case_5");
    let db = build_db_with_suffix_transform(path.path());
    let write_opt = WriteOptions::default();
    let read_opt = ReadOptions::default();
    let cf = db.default_cf();

    let samples_a = vec![
        (b"keya11111", b"value1"),
        (b"keyb22222", b"value2"),
        (b"keyc33333", b"value3"),
        (b"keyd44444", b"value4"),
        (b"keye55555", b"value5"),
    ];
    for (k, v) in samples_a {
        db.put(&write_opt, cf, k, v).unwrap();
        assert_eq!(Ok(Some(v.to_vec())), db.get(&read_opt, cf, k));
    }

    db.delete_range(&write_opt, cf, b"keya11111", b"keyf66666")
        .unwrap();

    check_kv(
        &db,
        cf,
        &[
            (b"keya11111", None),
            (b"keyb22222", None),
            (b"keyc33333", None),
            (b"keyd44444", None),
            (b"keye55555", None),
        ],
    );

    let path2 = tempdir_with_prefix("_rust_rocksdb_test_delete_range_prefix_bloom_case_5_2");
    let db2 = build_db_with_suffix_transform(path2.path());
    let cf2 = db2.default_cf();

    let samples_b = vec![(b"keyd44444", b"value4"), (b"keye55555", b"value5")];
    for (k, v) in samples_b {
        db2.put(&write_opt, cf2, k, v).unwrap();
        assert_eq!(Ok(Some(v.to_vec())), db2.get(&read_opt, cf2, k));
    }
    let before = gen_crc32_from_db(&db2);

    let gen_path = tempdir_with_prefix("_rust_rocksdb_case_prefix_bloom_5_ingest_sst_gen");
    let test_sstfile = gen_path.path().join("test_sst_file");
    let ingest_opt = IngestExternalFileOptions::default();

    let default_options = db2.cf_options(cf2);
    gen_sst_from_db(&default_options, cf2, &test_sstfile, &db2);

    db.ingest_external_file(&ingest_opt, cf, &[test_sstfile])
        .unwrap();
    check_kv(
        &db,
        cf,
        &[
            (b"keyd44444", Some(b"value4")),
            (b"keye55555", Some(b"value5")),
        ],
    );

    let after = gen_crc32_from_db(&db);
    assert_eq!(before, after);
}

#[test]
fn test_delete_range_prefix_bloom_case_6() {
    let path = tempdir_with_prefix("_rust_rocksdb_test_delete_range_prefix_bloom_case_6");
    let db = build_db_with_suffix_transform(path.path());
    let write_opt = WriteOptions::default();
    let read_opt = ReadOptions::default();
    let cf = db.default_cf();
    let samples_a = vec![
        (b"keya11111", b"value1"),
        (b"keyb22222", b"value2"),
        (b"keyc33333", b"value3"),
        (b"keyd44444", b"value4"),
        (b"keye55555", b"value5"),
    ];
    for (k, v) in samples_a {
        db.put(&write_opt, cf, k, v).unwrap();
        assert_eq!(Ok(Some(v.to_vec())), db.get(&read_opt, cf, k));
    }

    let before = gen_crc32_from_db_in_range(&db, b"key4", b"key6");

    db.delete_range(&write_opt, cf, b"keya11111", b"keyd44444")
        .unwrap();

    check_kv(
        &db,
        cf,
        &[
            (b"keya11111", None),
            (b"keyb22222", None),
            (b"keyc33333", None),
            (b"keyd44444", Some(b"value4")),
            (b"keye55555", Some(b"value5")),
        ],
    );

    let path2 = tempdir_with_prefix("_rust_rocksdb_test_delete_range_prefix_bloom_case_6_2");
    let db2 = build_db_with_suffix_transform(path2.path());
    let cf2 = db2.default_cf();

    let samples_b = vec![
        (b"keya11111", b"value1"),
        (b"keyb22222", b"value2"),
        (b"keyc33333", b"value3"),
    ];
    for (k, v) in samples_b {
        db2.put(&write_opt, cf2, k, v).unwrap();
        assert_eq!(Ok(Some(v.to_vec())), db2.get(&read_opt, cf2, k));
    }

    let gen_path = tempdir_with_prefix("_rust_rocksdb_case_6_ingest_sst_gen");
    let test_sstfile = gen_path.path().join("test_sst_file");
    let ingest_opt = IngestExternalFileOptions::default();

    let default_options = db2.cf_options(cf2);
    gen_sst_from_db(&default_options, cf2, &test_sstfile, &db2);

    db.ingest_external_file(&ingest_opt, cf, &[test_sstfile])
        .unwrap();
    check_kv(
        &db,
        cf,
        &[
            (b"keya11111", Some(b"value1")),
            (b"keyb22222", Some(b"value2")),
            (b"keyc33333", Some(b"value3")),
            (b"keyd44444", Some(b"value4")),
            (b"keye55555", Some(b"value5")),
        ],
    );

    let after = gen_crc32_from_db_in_range(&db, b"key4", b"key6");
    assert_eq!(before, after);
}

fn check_kv(db: &Db, cf: &RawColumnFamilyHandle, data: &[(&[u8], Option<&[u8]>)]) {
    let read_opt = ReadOptions::default();
    for &(k, v) in data {
        assert_eq!(db.get(&read_opt, cf, k), Ok(v.map(|s| s.to_vec())));
    }
}

#[test]
fn test_delete_range_prefix_bloom_compact_case() {
    let path = tempdir_with_prefix("_rust_rocksdb_test_delete_range_prefix_bloom_case_6");
    let db = build_db_with_suffix_transform(path.path());
    let write_opt = WriteOptions::default();
    let read_opt = ReadOptions::default();
    let cf = db.default_cf();
    let samples_a = vec![
        (b"keya11111", b"value1"),
        (b"keyb22222", b"value2"),
        (b"keyc33333", b"value3"),
        (b"keyd44444", b"value4"),
        (b"keye55555", b"value5"),
    ];
    for (k, v) in samples_a {
        db.put(&write_opt, cf, k, v).unwrap();
        assert_eq!(Ok(Some(v.to_vec())), db.get(&read_opt, cf, k));
    }

    let before = gen_crc32_from_db_in_range(&db, b"keyd44444", b"keyf66666");

    db.delete_range(&write_opt, cf, b"keya11111", b"keyd44444")
        .unwrap();

    check_kv(
        &db,
        cf,
        &[
            (b"keya11111", None),
            (b"keyb22222", None),
            (b"keyc33333", None),
            (b"keyd44444", Some(b"value4")),
            (b"keye55555", Some(b"value5")),
        ],
    );

    let path2 = tempdir_with_prefix("_rust_rocksdb_test_delete_range_prefix_bloom_case_6_2");
    let db2 = build_db_with_suffix_transform(path2.path());
    let cf2 = db2.default_cf();

    let samples_b = vec![
        (b"keya11111", b"value1"),
        (b"keyb22222", b"value2"),
        (b"keyc33333", b"value3"),
    ];
    for (k, v) in samples_b {
        db2.put(&write_opt, cf2, k, v).unwrap();
        assert_eq!(Ok(Some(v.to_vec())), db2.get(&read_opt, cf2, k));
    }

    let gen_path = tempdir_with_prefix("_rust_rocksdb_case_6_ingest_sst_gen");
    let test_sstfile = gen_path.path().join("test_sst_file");
    let ingest_opt = IngestExternalFileOptions::default();

    let default_options = db2.cf_options(cf2);
    gen_sst_from_db(&default_options, cf2, &test_sstfile, &db2);

    db.ingest_external_file(&ingest_opt, cf, &[test_sstfile])
        .unwrap();
    db.compact_range(&CompactRangeOptions::default(), cf, None, None)
        .unwrap();
    check_kv(
        &db,
        &cf,
        &[
            (b"keya11111", Some(b"value1")),
            (b"keyb22222", Some(b"value2")),
            (b"keyc33333", Some(b"value3")),
            (b"keyd44444", Some(b"value4")),
            (b"keye55555", Some(b"value5")),
        ],
    );

    let after = gen_crc32_from_db_in_range(&db, b"keyd44444", b"keyf66666");
    assert_eq!(before, after);
}

#[test]
fn test_delete_range() {
    // Test `DB::delete_range()`
    let path = tempdir_with_prefix("_rust_rocksdb_test_delete_range");
    let db = DefaultCfOnlyBuilder::default()
        .set_create_if_missing(true)
        .open(path.path())
        .unwrap();
    let write_opt = WriteOptions::default();
    let read_opt = ReadOptions::default();
    let cf = db.default_cf();

    // Prepare some data.
    let samples_a = vec![(b"a", b"v1"), (b"b", b"v2"), (b"c", b"v3")];
    let prepare_data = || {
        for (k, v) in &samples_a {
            db.put(&write_opt, cf, *k, *v).unwrap();
            assert_eq!(Ok(Some(v.to_vec())), db.get(&read_opt, cf, *k));
        }
    };

    prepare_data();
    // Ensure delete range interface works to delete the specified range `[b"a", b"c")`.
    db.delete_range(&write_opt, cf, b"a", b"c").unwrap();

    let check_data = || {
        check_kv(&db, cf, &[(b"a", None), (b"b", None), (b"c", Some(b"v3"))]);
    };
    check_data();

    // Test `WriteBatch::delete_range_default()`
    prepare_data();
    let mut batch = WriteBatch::default();
    batch.delete_range_default(b"a", b"c").unwrap();
    assert!(db.write(&write_opt, &mut batch).is_ok());
    check_data();

    // Test `WriteBatch::delete_range_cf()`
    prepare_data();
    let mut batch = WriteBatch::default();
    batch.delete_range(cf, b"a", b"c").unwrap();
    assert!(db.write(&write_opt, &mut batch).is_ok());
    check_data();
}

#[test]
fn test_delete_range_sst_files() {
    let path = tempdir_with_prefix("_rust_rocksdb_test_delete_range_sst_files");
    let db = DefaultCfOnlyBuilder::default()
        .set_create_if_missing(true)
        .open(path.path())
        .unwrap();
    let write_opt = WriteOptions::default();
    let read_opt = ReadOptions::default();
    let cf = db.default_cf();

    let samples_a = vec![
        (b"key1", b"value1"),
        (b"key2", b"value2"),
        (b"key3", b"value3"),
        (b"key4", b"value4"),
    ];
    for (k, v) in samples_a {
        db.put(&write_opt, cf, k, v).unwrap();
        assert_eq!(Ok(Some(v.to_vec())), db.get(&read_opt, cf, k));
    }
    db.flush(FlushOptions::default().set_wait(true), cf)
        .unwrap();

    let samples_b = vec![
        (b"key3", b"value5"),
        (b"key6", b"value6"),
        (b"key7", b"value7"),
        (b"key8", b"value8"),
    ];
    for (k, v) in samples_b {
        db.put(&write_opt, cf, k, v).unwrap();
        assert_eq!(Ok(Some(v.to_vec())), db.get(&read_opt, cf, k));
    }
    db.flush(FlushOptions::default().set_wait(true), cf)
        .unwrap();
    assert_eq!(Ok(Some(b"value5".to_vec())), db.get(&read_opt, cf, b"key3"));

    db.delete_range(&write_opt, cf, b"key1", b"key1").unwrap();
    db.delete_range(&write_opt, cf, b"key2", b"key7").unwrap();
    check_kv(
        &db,
        cf,
        &[
            (b"key1", Some(b"value1")),
            (b"key2", None),
            (b"key3", None),
            (b"key4", None),
            (b"key5", None),
            (b"key6", None),
            (b"key7", Some(b"value7")),
            (b"key8", Some(b"value8")),
        ],
    );

    db.delete_range(&write_opt, cf, b"key1", b"key8").unwrap();
    check_kv(
        &db,
        cf,
        &[
            (b"key1", None),
            (b"key2", None),
            (b"key3", None),
            (b"key4", None),
            (b"key5", None),
            (b"key6", None),
            (b"key7", None),
            (b"key8", Some(b"value8")),
        ],
    );
}

#[test]
fn test_delete_range_ingest_file() {
    let path = tempdir_with_prefix("_rust_rocksdb_test_delete_range_ingest_file");
    let mut db = DefaultCfOnlyBuilder::default()
        .set_create_if_missing(true)
        .open(path.path())
        .unwrap();
    let cf = db.default_cf();
    let gen_path = tempdir_with_prefix("_rust_rocksdb_ingest_sst_gen");
    let test_sstfile = gen_path.path().join("test_sst_file");
    let ingest_opt = IngestExternalFileOptions::default();

    let default_options = db.cf_options(cf);
    gen_sst(&default_options, Some(cf), &test_sstfile);

    db.ingest_external_file(&ingest_opt, cf, &[&test_sstfile])
        .unwrap();
    assert!(test_sstfile.exists());
    check_kv(
        &db,
        cf,
        &[
            (b"key1", Some(b"value1")),
            (b"key2", Some(b"value2")),
            (b"key3", Some(b"value3")),
            (b"key4", Some(b"value4")),
        ],
    );

    let write_opt = WriteOptions::default();
    db.delete_range(&write_opt, cf, b"key1", b"key4").unwrap();
    check_kv(
        &db,
        cf,
        &[
            (b"key1", None),
            (b"key2", None),
            (b"key3", None),
            (b"key4", Some(b"value4")),
        ],
    );

    db.create_cf("cf1", CfOptions::default()).unwrap();
    let new_cf = db.cf("cf1").unwrap();
    let new_opt = db.cf_options(new_cf);
    gen_sst(&new_opt, None, &test_sstfile);

    db.ingest_external_file(&ingest_opt, new_cf, &[&test_sstfile])
        .unwrap();
    assert!(test_sstfile.exists());
    check_kv(
        &db,
        new_cf,
        &[
            (b"key1", Some(b"value1")),
            (b"key2", Some(b"value2")),
            (b"key3", Some(b"value3")),
            (b"key4", Some(b"value4")),
        ],
    );

    let snap = db.snapshot();

    db.delete_range(&write_opt, new_cf, b"key1", b"key3")
        .unwrap();
    check_kv(
        &db,
        new_cf,
        &[
            (b"key1", None),
            (b"key2", None),
            (b"key3", Some(b"value3")),
            (b"key4", Some(b"value4")),
        ],
    );
    let mut read_opt = ReadOptions::default();
    assert_eq!(
        snap.get(&mut read_opt, new_cf, b"key1").unwrap().unwrap(),
        b"value1"
    );
    assert_eq!(
        snap.get(&mut read_opt, new_cf, b"key4").unwrap().unwrap(),
        b"value4"
    );
}
