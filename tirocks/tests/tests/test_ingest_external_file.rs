// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::fs;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::Arc;

use tempfile::TempDir;
use tirocks::db::{DefaultCfOnlyBuilder, RawColumnFamilyHandle};
use tirocks::env::{Env, EnvOptions};
use tirocks::merge_operator::SysMergeOperator;
use tirocks::option::{IngestExternalFileOptions, RawOptions, ReadOptions, WriteOptions};
use tirocks::table::sst::{SstFileReader, SstFileWriter};
use tirocks::{
    set_external_sst_file_global_sequence_number, CfOptions, Code, Db, OpenOptions, Options,
};

use crate::test_cf::test_provided_merge;
use crate::test_delete_range::gen_sst_from_db;

use super::tempdir_with_prefix;

pub fn gen_sst(
    opt: &RawOptions,
    cf: Option<&RawColumnFamilyHandle>,
    path: &Path,
    data: &[(&[u8], &[u8])],
) {
    let _ = fs::remove_file(path);
    let env_opt = EnvOptions::default();
    let mut writer = SstFileWriter::new(&env_opt, opt, cf);
    writer.open(path).unwrap();
    for &(k, v) in data {
        writer.put(k, v).unwrap();
    }

    writer.finish(None).unwrap();
}

fn gen_sst_put(opt: &RawOptions, cf: Option<&RawColumnFamilyHandle>, path: &Path) {
    gen_sst(
        opt,
        cf,
        path,
        &[(b"k1", b"a"), (b"k2", b"b"), (b"k3", b"c")],
    );
}

fn gen_sst_merge(opt: &RawOptions, cf: Option<&RawColumnFamilyHandle>, path: &Path) {
    let _ = fs::remove_file(path);
    let env_opt = EnvOptions::default();
    let mut writer = SstFileWriter::new(&env_opt, opt, cf);
    writer.open(path).unwrap();
    writer.merge(b"k3", b"d").unwrap();
    writer.finish(None).unwrap();
}

fn gen_sst_delete(opt: &RawOptions, cf: Option<&RawColumnFamilyHandle>, path: &Path) {
    let _ = fs::remove_file(path);
    let env_opt = EnvOptions::default();
    let mut writer = SstFileWriter::new(&env_opt, opt, cf);
    writer.open(path).unwrap();
    writer.delete(b"k3").unwrap();
    writer.finish(None).unwrap();
}

#[test]
fn test_ingest_external_file() {
    let path = tempdir_with_prefix("_rust_rocksdb_ingest_sst");
    let mut db = create_default_database(&path);
    db.create_cf("cf1", CfOptions::default()).unwrap();
    let cf1 = db.cf("cf1").unwrap();
    let gen_path = tempdir_with_prefix("_rust_rocksdb_ingest_sst_gen");
    let test_sstfile = gen_path.path().join("test_sst_file");
    let default_cf = db.default_cf();
    let default_options = db.cf_options(default_cf);

    gen_sst(
        &default_options,
        Some(default_cf),
        &test_sstfile,
        &[(b"k1", b"v1"), (b"k2", b"v2")],
    );
    let mut ingest_opt = IngestExternalFileOptions::default();
    db.ingest_external_file(&ingest_opt, default_cf, &[&test_sstfile])
        .unwrap();
    assert!(test_sstfile.exists());
    let mut read_opt = ReadOptions::default();
    assert_eq!(
        db.get(&read_opt, default_cf, b"k1"),
        Ok(Some(b"v1".to_vec()))
    );
    assert_eq!(
        db.get(&read_opt, default_cf, b"k2"),
        Ok(Some(b"v2".to_vec()))
    );

    gen_sst(
        &Options::default(),
        None,
        &test_sstfile,
        &[(b"k1", b"v3"), (b"k2", b"v4")],
    );
    db.ingest_external_file(&ingest_opt, cf1, &[&test_sstfile])
        .unwrap();
    assert_eq!(db.get(&read_opt, cf1, b"k1"), Ok(Some(b"v3".to_vec())));
    assert_eq!(db.get(&read_opt, cf1, b"k2"), Ok(Some(b"v4".to_vec())));
    let snap = db.snapshot();

    gen_sst(
        &Options::default(),
        None,
        &test_sstfile,
        &[(b"k2", b"v5"), (b"k3", b"v6")],
    );
    ingest_opt.set_move_files(true);
    db.ingest_external_file(&ingest_opt, cf1, &[test_sstfile])
        .unwrap();

    assert_eq!(db.get(&read_opt, cf1, b"k1"), Ok(Some(b"v3".to_vec())));
    assert_eq!(db.get(&read_opt, cf1, b"k2"), Ok(Some(b"v5".to_vec())));
    assert_eq!(db.get(&read_opt, cf1, b"k3"), Ok(Some(b"v6".to_vec())));
    assert_eq!(
        snap.get(&mut read_opt, cf1, b"k1"),
        Ok(Some(b"v3".to_vec()))
    );
    assert_eq!(
        snap.get(&mut read_opt, cf1, b"k2"),
        Ok(Some(b"v4".to_vec()))
    );
    assert_eq!(snap.get(&mut read_opt, cf1, b"k3"), Ok(None));
}

#[test]
fn test_ingest_external_file_new() {
    let path = tempdir_with_prefix("_rust_rocksdb_ingest_sst_new");
    let merge = SysMergeOperator::new(test_provided_merge);
    let mut builder = DefaultCfOnlyBuilder::default();
    builder
        .set_create_if_missing(true)
        .cf_options_mut()
        .set_merge_operator(&merge);
    let db = builder.open(path.path()).unwrap();
    let gen_path = tempdir_with_prefix("_rust_rocksdb_ingest_sst_gen_new");
    let test_sstfile = gen_path.path().join("test_sst_file_new");
    let cf = db.default_cf();
    let default_options = db.cf_options(cf);

    gen_sst_put(&default_options, Some(cf), &test_sstfile);
    let mut ingest_opt = IngestExternalFileOptions::default();
    db.ingest_external_file(&ingest_opt, cf, &[&test_sstfile])
        .unwrap();

    assert!(test_sstfile.exists());
    let mut read_opt = ReadOptions::default();
    assert_eq!(db.get(&read_opt, cf, b"k1"), Ok(Some(b"a".to_vec())));
    assert_eq!(db.get(&read_opt, cf, b"k2"), Ok(Some(b"b".to_vec())));
    assert_eq!(db.get(&read_opt, cf, b"k3"), Ok(Some(b"c".to_vec())));

    let snap = db.snapshot();
    let default_options = db.cf_options(cf);
    gen_sst_merge(&default_options, Some(cf), &test_sstfile);
    db.ingest_external_file(&ingest_opt, cf, &[&test_sstfile])
        .unwrap();

    assert_eq!(db.get(&read_opt, cf, b"k1"), Ok(Some(b"a".to_vec())));
    assert_eq!(db.get(&read_opt, cf, b"k2"), Ok(Some(b"b".to_vec())));
    assert_eq!(db.get(&read_opt, cf, b"k3"), Ok(Some(b"cd".to_vec())));

    let default_options = db.cf_options(cf);
    gen_sst_delete(&default_options, Some(cf), &test_sstfile);
    ingest_opt.set_move_files(true);
    db.ingest_external_file(&ingest_opt, cf, &[test_sstfile])
        .unwrap();

    assert_eq!(db.get(&read_opt, cf, b"k1"), Ok(Some(b"a".to_vec())));
    assert_eq!(db.get(&read_opt, cf, b"k2"), Ok(Some(b"b".to_vec())));
    assert_eq!(db.get(&read_opt, cf, b"k3"), Ok(None));
    assert_eq!(snap.get(&mut read_opt, cf, b"k1"), Ok(Some(b"a".to_vec())));
    assert_eq!(snap.get(&mut read_opt, cf, b"k2"), Ok(Some(b"b".to_vec())));
    assert_eq!(snap.get(&mut read_opt, cf, b"k3"), Ok(Some(b"c".to_vec())));
}

#[test]
fn test_ingest_external_file_new_cf() {
    let path = tempdir_with_prefix("_rust_rocksdb_ingest_sst_new_cf");
    let mut db = create_default_database(&path);
    let gen_path = tempdir_with_prefix("_rust_rocksdb_ingest_sst_gen_new_cf");
    let test_sstfile = gen_path.path().join("test_sst_file_new_cf");
    let mut cf_opt = CfOptions::default();
    let sys_merge = SysMergeOperator::new(test_provided_merge);
    cf_opt.set_merge_operator(&sys_merge);
    db.create_cf("cf1", cf_opt).unwrap();
    let cf1 = db.cf("cf1").unwrap();

    let mut ingest_opt = IngestExternalFileOptions::default();
    gen_sst_put(&Options::default(), None, &test_sstfile);

    db.ingest_external_file(&ingest_opt, cf1, &[&test_sstfile])
        .unwrap();
    assert!(test_sstfile.exists());
    let mut read_opt = ReadOptions::default();
    assert_eq!(db.get(&read_opt, cf1, b"k1"), Ok(Some(b"a".to_vec())));
    assert_eq!(db.get(&read_opt, cf1, b"k2"), Ok(Some(b"b".to_vec())));
    assert_eq!(db.get(&read_opt, cf1, b"k3"), Ok(Some(b"c".to_vec())));

    let snap = db.snapshot();
    ingest_opt.set_move_files(true);
    gen_sst_merge(&Options::default(), None, &test_sstfile);
    db.ingest_external_file(&ingest_opt, cf1, &[&test_sstfile])
        .unwrap();
    assert_eq!(db.get(&read_opt, cf1, b"k1"), Ok(Some(b"a".to_vec())));
    assert_eq!(db.get(&read_opt, cf1, b"k2"), Ok(Some(b"b".to_vec())));
    assert_eq!(db.get(&read_opt, cf1, b"k3"), Ok(Some(b"cd".to_vec())));

    gen_sst_delete(&Options::default(), None, &test_sstfile);
    db.ingest_external_file(&ingest_opt, cf1, &[&test_sstfile])
        .unwrap();

    assert_eq!(db.get(&read_opt, cf1, b"k1"), Ok(Some(b"a".to_vec())));
    assert_eq!(db.get(&read_opt, cf1, b"k2"), Ok(Some(b"b".to_vec())));
    assert_eq!(db.get(&read_opt, cf1, b"k3"), Ok(None));
    assert_eq!(snap.get(&mut read_opt, cf1, b"k1"), Ok(Some(b"a".to_vec())));
    assert_eq!(snap.get(&mut read_opt, cf1, b"k2"), Ok(Some(b"b".to_vec())));
    assert_eq!(snap.get(&mut read_opt, cf1, b"k3"), Ok(Some(b"c".to_vec())));
    assert_eq!(db.get(&read_opt, cf1, b"k3"), Ok(None));
}

fn check_kv(db: &Db, cf: &RawColumnFamilyHandle, data: &[(&[u8], Option<&[u8]>)]) {
    for (k, v) in data {
        assert_eq!(
            db.get(&ReadOptions::default(), cf, k),
            Ok(v.map(|v| v.to_vec()))
        );
    }
}

fn put_delete_and_generate_sst_cf(
    opt: &RawOptions,
    db: &Db,
    cf: &RawColumnFamilyHandle,
    path: &Path,
) {
    let write_opt = WriteOptions::default();
    db.put(&write_opt, cf, b"k1", b"v1").unwrap();
    db.put(&write_opt, cf, b"k2", b"v2").unwrap();
    db.put(&write_opt, cf, b"k3", b"v3").unwrap();
    db.put(&write_opt, cf, b"k4", b"v4").unwrap();
    db.delete(&write_opt, cf, b"k1").unwrap();
    db.delete(&write_opt, cf, b"k3").unwrap();
    gen_sst_from_db(opt, cf, path, db);
}

fn create_default_database(path: &TempDir) -> Db {
    DefaultCfOnlyBuilder::default()
        .set_create_if_missing(true)
        .open(path.path())
        .unwrap()
}

fn create_cfs(db: &mut Db, cfs: &[&str]) {
    for cf in cfs {
        if *cf != "default" {
            db.create_cf(*cf, CfOptions::default()).unwrap();
        }
    }
}

#[test]
fn test_ingest_simulate_real_world() {
    const ALL_CFS: [&str; 3] = ["lock", "write", "default"];
    let path = tempdir_with_prefix("_rust_rocksdb_ingest_real_world_1");
    let mut db = create_default_database(&path);
    let gen_path = tempdir_with_prefix("_rust_rocksdb_ingest_real_world_new_cf");
    create_cfs(&mut db, &ALL_CFS);
    for cf_name in &ALL_CFS {
        eprintln!("generating {}", cf_name);
        let cf = db.cf(cf_name).unwrap();
        let cf_opts = Options::default();
        put_delete_and_generate_sst_cf(&cf_opts, &db, cf, &gen_path.path().join(cf_name));
    }

    let path2 = tempdir_with_prefix("_rust_rocksdb_ingest_real_world_2");
    let mut db2 = create_default_database(&path2);
    create_cfs(&mut db2, &ALL_CFS);
    for cf_name in &ALL_CFS {
        let cf2 = db2.cf(cf_name).unwrap();
        let mut ingest_opt = IngestExternalFileOptions::default();
        ingest_opt.set_move_files(true);
        db2.ingest_external_file(&ingest_opt, cf2, &[gen_path.path().join(cf_name)])
            .unwrap();
        check_kv(
            &db,
            db.cf(cf_name).unwrap(),
            &[
                (b"k1", None),
                (b"k2", Some(b"v2")),
                (b"k3", None),
                (b"k4", Some(b"v4")),
            ],
        );
        let opts = Options::default();
        gen_sst_from_db(&opts, cf2, &gen_path.path().join(cf_name), &db2);
    }

    for cf_name in &ALL_CFS {
        let cf = db.cf(cf_name).unwrap();
        let ingest_opt = IngestExternalFileOptions::default();
        db.ingest_external_file(&&ingest_opt, cf, &[gen_path.path().join(cf_name)])
            .unwrap();
        check_kv(
            &db,
            cf,
            &[
                (b"k1", None),
                (b"k2", Some(b"v2")),
                (b"k3", None),
                (b"k4", Some(b"v4")),
            ],
        );
    }
}

#[test]
fn test_mem_sst_file_writer() {
    let path = tempdir_with_prefix("_rust_mem_sst_file_writer");
    let db = create_default_database(&path);

    let env = Arc::new(Env::with_mem(Env::default()));
    let cf = db.default_cf();
    let mut opts = Options::default();
    opts.set_env(env.clone());

    let mem_sst_path = path.path().join("mem_sst");
    gen_sst(
        &opts,
        None,
        &mem_sst_path,
        &[(b"k1", b"v1"), (b"k2", b"v2"), (b"k3", b"v3")],
    );
    // Check that the file is not on disk.
    assert!(!mem_sst_path.exists());

    let mut buf = Vec::new();
    let mut sst = env
        .new_sequential_file(&mem_sst_path, EnvOptions::default())
        .unwrap();
    sst.read_to_end(&mut buf).unwrap();

    // Write the data to a temp file.
    let sst_path = path.path().join("temp_sst_path");
    fs::File::create(&sst_path)
        .unwrap()
        .write_all(&buf)
        .unwrap();
    // Ingest the temp file to check the test kvs.
    let ingest_opts = IngestExternalFileOptions::default();
    db.ingest_external_file(&ingest_opts, cf, &[sst_path.to_str().unwrap()])
        .unwrap();
    check_kv(
        &db,
        cf,
        &[
            (b"k1", Some(b"v1")),
            (b"k2", Some(b"v2")),
            (b"k3", Some(b"v3")),
        ],
    );

    assert!(env.file_exists(&mem_sst_path).is_ok());
    assert!(env.delete_file(&mem_sst_path).is_ok());
    assert!(env.file_exists(&mem_sst_path).is_err());
}

#[test]
fn test_set_external_sst_file_global_seq_no() {
    let db_path = tempdir_with_prefix("_rust_rocksdb_set_external_sst_file_global_seq_no_db");
    let db = create_default_database(&db_path);
    let path = tempdir_with_prefix("_rust_rocksdb_set_external_sst_file_global_seq_no");
    let sstfile = path.path().join("sst_file");
    let cf = db.default_cf();
    gen_sst(
        &Options::default(),
        Some(cf),
        &sstfile,
        &[(b"k1", b"v1"), (b"k2", b"v2")],
    );

    let seq_no = 1;
    // varify change seq_no
    let r1 = set_external_sst_file_global_sequence_number(&db, &cf, &sstfile, seq_no);
    assert_ne!(r1, Ok(seq_no));
    // varify that seq_no are equal
    let r2 = set_external_sst_file_global_sequence_number(&db, &cf, &sstfile, seq_no);
    assert_eq!(r2, Ok(seq_no));

    // change seq_no back to 0 so that it can be ingested
    set_external_sst_file_global_sequence_number(&db, cf, &sstfile, 0).unwrap();

    db.ingest_external_file(&IngestExternalFileOptions::default(), cf, &[sstfile])
        .unwrap();
    check_kv(&db, cf, &[(b"k1", Some(b"v1")), (b"k2", Some(b"v2"))]);
}

#[test]
fn test_ingest_external_file_optimized() {
    let path = tempdir_with_prefix("_rust_rocksdb_ingest_sst_optimized");
    let db = create_default_database(&path);
    let gen_path = tempdir_with_prefix("_rust_rocksdb_ingest_sst_gen_new_cf");
    let test_sstfile = gen_path.path().join("test_sst_file_optimized");
    let cf = db.default_cf();

    let ingest_opt = IngestExternalFileOptions::default();
    gen_sst_put(&Options::default(), None, &test_sstfile);

    let write_opt = WriteOptions::default();
    db.put(&write_opt, cf, b"k0", b"k0").unwrap();

    // No overlap with the memtable.
    let has_flush = db
        .ingest_external_file_optimized(&ingest_opt, cf, &[&test_sstfile])
        .unwrap();
    assert!(!has_flush);
    assert!(test_sstfile.exists());
    let read_opt = ReadOptions::default();
    assert_eq!(db.get(&read_opt, cf, b"k1"), Ok(Some(b"a".to_vec())));
    assert_eq!(db.get(&read_opt, cf, b"k2"), Ok(Some(b"b".to_vec())));
    assert_eq!(db.get(&read_opt, cf, b"k3"), Ok(Some(b"c".to_vec())));

    db.put(&write_opt, cf, b"k1", b"k1").unwrap();

    // Overlap with the memtable.
    let has_flush = db
        .ingest_external_file_optimized(&ingest_opt, cf, &[&test_sstfile])
        .unwrap();
    assert!(has_flush);
    assert!(test_sstfile.exists());
    assert_eq!(db.get(&read_opt, cf, b"k1"), Ok(Some(b"a".to_vec())));
    assert_eq!(db.get(&read_opt, cf, b"k2"), Ok(Some(b"b".to_vec())));
    assert_eq!(db.get(&read_opt, cf, b"k3"), Ok(Some(b"c".to_vec())));
}

#[test]
fn test_read_sst() {
    let dir = tempdir_with_prefix("_rust_rocksdb_test_read_sst");
    let sst_path = dir.path().join("sst");
    gen_sst_put(&Options::default(), None, &sst_path);

    let mut reader = SstFileReader::new(&Options::default());
    reader.open(&sst_path).unwrap();
    reader.verify_checksum().unwrap();
    let props = reader.table_properties();
    assert_eq!(props.num_entries(), 3);
    let mut read_opt = ReadOptions::default();
    let mut it = reader.iter(None, &mut read_opt);
    it.seek_to_first();
    assert!(it.valid());
    assert_eq!(
        it.collect::<Vec<_>>(),
        vec![
            (b"k1".to_vec(), b"a".to_vec()),
            (b"k2".to_vec(), b"b".to_vec()),
            (b"k3".to_vec(), b"c".to_vec()),
        ]
    );
}

#[test]
fn test_read_invalid_sst() {
    let dir = tempdir_with_prefix("_rust_rocksdb_test_read_invalid_sst");
    let sst_path = dir.path().join("sst");
    gen_sst_put(&Options::default(), None, &sst_path);

    // corrupt one byte.
    let mut f = fs::OpenOptions::new().write(true).open(&sst_path).unwrap();
    f.seek(SeekFrom::Start(9)).unwrap();
    f.write(b"!").unwrap();

    let mut reader = SstFileReader::new(&Options::default());
    reader.open(&sst_path).unwrap();
    let err = reader.verify_checksum().unwrap_err();
    assert_eq!(err.code(), Code::kCorruption);
    assert!(err
        .message()
        .unwrap()
        .unwrap()
        .contains("checksum mismatch"));
}

#[test]
fn test_ingest_external_file_options() {
    let mut ingest_opt = IngestExternalFileOptions::default();
    ingest_opt.set_write_global_sequence_number(false);
    assert_eq!(false, ingest_opt.write_global_sequence_number());
    ingest_opt.set_write_global_sequence_number(true);
    assert_eq!(true, ingest_opt.write_global_sequence_number());
}
