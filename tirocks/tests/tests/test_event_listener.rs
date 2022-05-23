// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::io::Write;
use std::path::Path;
use std::sync::atomic::*;
use std::sync::Arc;
use std::time::Duration;

use tirocks::db::DefaultCfOnlyBuilder;
use tirocks::db::MultiCfBuilder;
use tirocks::db::DEFAULT_CF_NAME;
use tirocks::listener::job_info::CompactionJobInfo;
use tirocks::listener::job_info::CompactionReason;
use tirocks::listener::job_info::FlushJobInfo;
use tirocks::listener::other_info::ExternalFileIngestionInfo;
use tirocks::listener::other_info::WriteStallCondition;
use tirocks::listener::other_info::WriteStallInfo;
use tirocks::listener::BackgroundErrorReason;
use tirocks::listener::EventListener;
use tirocks::listener::SysEventListener;
use tirocks::option::CompactRangeOptions;
use tirocks::option::FlushOptions;
use tirocks::option::IngestExternalFileOptions;
use tirocks::option::ReadOptions;
use tirocks::option::WriteOptions;
use tirocks::CfOptions;
use tirocks::OpenOptions;
use tirocks::RawDb;
use tirocks::Status;

use crate::test_ingest_external_file::gen_sst;

use super::tempdir_with_prefix;

#[derive(Default, Clone)]
struct EventCounter {
    flush: Arc<AtomicUsize>,
    compaction: Arc<AtomicUsize>,
    ingestion: Arc<AtomicUsize>,
    drop_count: Arc<AtomicUsize>,
    input_records: Arc<AtomicUsize>,
    output_records: Arc<AtomicUsize>,
    input_bytes: Arc<AtomicUsize>,
    output_bytes: Arc<AtomicUsize>,
    manual_compaction: Arc<AtomicUsize>,
}

impl Drop for EventCounter {
    fn drop(&mut self) {
        self.drop_count.fetch_add(1, Ordering::SeqCst);
    }
}

impl EventListener for EventCounter {
    fn on_flush_completed(&self, _db: &RawDb, info: &FlushJobInfo) {
        assert!(!info.cf_name().unwrap().is_empty());
        assert!(Path::new(info.file_path().unwrap()).exists());
        assert_ne!(info.table_properties().data_size(), 0);
        self.flush.fetch_add(1, Ordering::SeqCst);
    }

    fn on_compaction_completed(&self, _db: &RawDb, info: &CompactionJobInfo) {
        assert!(info.status().ok());
        assert!(!info.cf_name().unwrap().is_empty());
        let input_file_count = info.input_file_count();
        assert_ne!(input_file_count, 0);
        for i in 0..input_file_count {
            let path = info.input_file_at(i);
            assert!(Path::new(path.unwrap()).exists());
        }
        assert_eq!(info.num_input_files_at_output_level(), 0);

        let output_file_count = info.output_file_count();
        assert_ne!(output_file_count, 0);
        for i in 0..output_file_count {
            let path = info.output_file_at(i);
            assert!(Path::new(path.unwrap()).exists());
        }

        let props = info.table_properties();
        assert_eq!(props.len(), output_file_count + input_file_count);

        assert_ne!(info.elapsed(), Duration::default());
        assert_eq!(info.num_corrupt_keys(), 0);
        assert!(info.output_level() >= 0);

        self.compaction.fetch_add(1, Ordering::SeqCst);
        self.input_records
            .fetch_add(info.input_records() as usize, Ordering::SeqCst);
        self.output_records
            .fetch_add(info.output_records() as usize, Ordering::SeqCst);
        self.input_bytes
            .fetch_add(info.total_input_bytes() as usize, Ordering::SeqCst);
        self.output_bytes
            .fetch_add(info.total_output_bytes() as usize, Ordering::SeqCst);

        if info.compaction_reason() == CompactionReason::kManualCompaction {
            self.manual_compaction.fetch_add(1, Ordering::SeqCst);
        }
    }

    fn on_external_file_ingested(&self, _db: &RawDb, info: &ExternalFileIngestionInfo) {
        assert!(!info.cf_name().unwrap().is_empty());
        assert!(Path::new(info.internal_file_path().unwrap()).exists());
        assert_ne!(info.table_properties().data_size(), 0);
        assert_ne!(info.picked_level(), 0);
        self.ingestion.fetch_add(1, Ordering::SeqCst);
    }
}

#[derive(Default, Clone)]
struct StallEventCounter {
    flush: Arc<AtomicUsize>,
    stall_conditions_changed_num: Arc<AtomicUsize>,
    triggered_writes_slowdown: Arc<AtomicUsize>,
    triggered_writes_stop: Arc<AtomicUsize>,
    stall_change_from_normal_to_other: Arc<AtomicUsize>,
}

impl EventListener for StallEventCounter {
    fn on_flush_completed(&self, _db: &RawDb, info: &FlushJobInfo) {
        assert!(!info.cf_name().unwrap().is_empty());
        self.flush.fetch_add(1, Ordering::SeqCst);
        self.triggered_writes_slowdown
            .fetch_add(info.triggered_writes_slowdown() as usize, Ordering::SeqCst);
        self.triggered_writes_stop
            .fetch_add(info.triggered_writes_stop() as usize, Ordering::SeqCst);
    }

    fn on_stall_conditions_changed(&self, info: &WriteStallInfo) {
        assert_eq!(info.cf_name(), Ok("test_cf"));
        self.stall_conditions_changed_num
            .fetch_add(1, Ordering::SeqCst);
        if info.prev_state() == WriteStallCondition::kNormal
            && info.cur_state() != WriteStallCondition::kNormal
        {
            self.stall_change_from_normal_to_other
                .fetch_add(1, Ordering::SeqCst);
        }
    }
}

#[derive(Default, Clone)]
struct BackgroundErrorCounter {
    background_error: Arc<AtomicUsize>,
}

impl EventListener for BackgroundErrorCounter {
    fn on_background_error(&self, _: BackgroundErrorReason, _: &mut Status) {
        self.background_error.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn test_event_listener_stall_conditions_changed() {
    let path = tempdir_with_prefix("_rust_rocksdb_event_listener_stall_conditions");

    let counter = StallEventCounter::default();
    let sys_counter = SysEventListener::new(counter.clone());
    let mut builder = MultiCfBuilder::default();
    builder
        .set_create_if_missing(true)
        .set_create_missing_column_families(true)
        .db_options_mut()
        .add_event_listener(&sys_counter);
    builder.add_cf(DEFAULT_CF_NAME, CfOptions::default());
    let mut cf_opt = CfOptions::default();
    cf_opt
        .set_level0_slowdown_writes_trigger(1)
        .set_level0_stop_writes_trigger(1)
        .set_level0_file_num_compaction_trigger(1);
    builder.add_cf("test_cf", cf_opt);
    let db = builder.open(path.path()).unwrap();
    drop(sys_counter);

    let test_cf = db.cf("test_cf").unwrap();
    let write_opt = WriteOptions::default();
    for i in 1..5 {
        db.put(
            &write_opt,
            test_cf,
            format!("{:04}", i).as_bytes(),
            format!("{:04}", i).as_bytes(),
        )
        .unwrap();
        db.flush(FlushOptions::default().set_wait(true), test_cf)
            .unwrap();
    }
    let flush_cnt = counter.flush.load(Ordering::SeqCst);
    assert_ne!(flush_cnt, 0);
    let stall_conditions_changed_num = counter.stall_conditions_changed_num.load(Ordering::SeqCst);
    let triggered_writes_slowdown = counter.triggered_writes_slowdown.load(Ordering::SeqCst);
    let triggered_writes_stop = counter.triggered_writes_stop.load(Ordering::SeqCst);
    let stall_change_from_normal_to_other = counter
        .stall_change_from_normal_to_other
        .load(Ordering::SeqCst);
    assert_ne!(stall_conditions_changed_num, 0);
    assert_ne!(triggered_writes_slowdown, 0);
    assert_ne!(triggered_writes_stop, 0);
    assert_ne!(stall_change_from_normal_to_other, 0);
}

#[test]
fn test_event_listener_basic() {
    let path = tempdir_with_prefix("_rust_rocksdb_event_listener_flush");

    let counter = EventCounter::default();
    let sys_counter = SysEventListener::new(counter.clone());
    let mut builder = DefaultCfOnlyBuilder::default();
    builder
        .set_create_if_missing(true)
        .db_options_mut()
        .add_event_listener(&sys_counter);
    let db = builder.open(path.path()).unwrap();
    let write_opt = WriteOptions::default();
    let cf = db.default_cf();
    for i in 1..8000 {
        db.put(
            &write_opt,
            cf,
            format!("{:04}", i).as_bytes(),
            format!("{:04}", i).as_bytes(),
        )
        .unwrap();
    }
    db.flush(FlushOptions::default().set_wait(true), cf)
        .unwrap();
    assert_ne!(counter.flush.load(Ordering::SeqCst), 0);

    for i in 1..8000 {
        db.put(
            &write_opt,
            cf,
            format!("{:04}", i).as_bytes(),
            format!("{:04}", i).as_bytes(),
        )
        .unwrap();
    }
    db.flush(FlushOptions::default().set_wait(true), cf)
        .unwrap();
    let flush_cnt = counter.flush.load(Ordering::SeqCst);
    assert_ne!(flush_cnt, 0);
    assert_eq!(counter.compaction.load(Ordering::SeqCst), 0);
    db.compact_range(&CompactRangeOptions::default(), cf, None, None)
        .unwrap();
    assert_eq!(counter.flush.load(Ordering::SeqCst), flush_cnt);
    assert_ne!(counter.compaction.load(Ordering::SeqCst), 0);
    drop(sys_counter);
    drop(builder);
    assert_eq!(counter.drop_count.load(Ordering::SeqCst), 0);
    db.close().unwrap();
    assert_eq!(counter.drop_count.load(Ordering::SeqCst), 1);
    assert!(
        counter.input_records.load(Ordering::SeqCst)
            > counter.output_records.load(Ordering::SeqCst)
    );
    assert!(
        counter.input_bytes.load(Ordering::SeqCst) > counter.output_bytes.load(Ordering::SeqCst)
    );
    assert_eq!(counter.manual_compaction.load(Ordering::SeqCst), 1);
}

#[test]
fn test_event_listener_ingestion() {
    let path = tempdir_with_prefix("_rust_rocksdb_event_listener_ingestion");

    let counter = EventCounter::default();
    let sys_counter = SysEventListener::new(counter.clone());
    let mut builder = DefaultCfOnlyBuilder::default();
    builder
        .set_create_if_missing(true)
        .db_options_mut()
        .add_event_listener(&sys_counter);
    let db = builder.open(path.path()).unwrap();
    let cf = db.default_cf();

    let gen_path = tempdir_with_prefix("_rust_rocksdb_ingest_sst_gen");
    let test_sstfile = gen_path.path().join("test_sst_file");

    let default_options = db.cf_options(cf);
    gen_sst(
        &default_options,
        Some(cf),
        &test_sstfile,
        &[(b"k1", b"v1"), (b"k2", b"v2")],
    );

    let ingest_opt = IngestExternalFileOptions::default();
    db.ingest_external_file(&ingest_opt, cf, &[&test_sstfile])
        .unwrap();
    assert!(test_sstfile.exists());
    let read_opt = ReadOptions::default();
    assert_eq!(db.get(&read_opt, cf, b"k1"), Ok(Some(b"v1".to_vec())));
    assert_eq!(db.get(&read_opt, cf, b"k2"), Ok(Some(b"v2".to_vec())));
    assert_ne!(counter.ingestion.load(Ordering::SeqCst), 0);
    let files = db.live_files_metadata();
    assert!(test_sstfile.exists());
    let f = &files[0];
    db.delete_file(f.name().unwrap()).unwrap();

    let read_opt = ReadOptions::default();
    assert_eq!(db.get(&read_opt, cf, b"k1"), Ok(None));
    assert_eq!(db.get(&read_opt, cf, b"k2"), Ok(None));
}

#[test]
fn test_event_listener_background_error() {
    // TODO(yiwu): should create a test Env object which inject some IO error, to
    // actually trigger background error.
    let path = tempdir_with_prefix("_rust_rocksdb_event_listener_ingestion");

    let counter = BackgroundErrorCounter::default();
    let mut builder = DefaultCfOnlyBuilder::default();
    builder
        .db_options_mut()
        .add_event_listener(&SysEventListener::new(counter.clone()));
    let db = builder
        .set_create_if_missing(true)
        .open(path.path())
        .unwrap();

    let write_opt = WriteOptions::default();
    let flush_opt = FlushOptions::default();
    let cf = db.default_cf();
    for i in 1..10 {
        db.put(&write_opt, cf, format!("{:04}", i).as_bytes(), b"value")
            .unwrap();
        db.flush(&flush_opt, cf).unwrap();
    }
    assert_eq!(counter.background_error.load(Ordering::SeqCst), 0);
}

#[derive(Default, Clone)]
struct BackgroundErrorCleaner(Arc<AtomicUsize>);

impl EventListener for BackgroundErrorCleaner {
    fn on_background_error(&self, _: BackgroundErrorReason, s: &mut Status) {
        *s = Status::default();
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn test_event_listener_status_reset() {
    let path = tempdir_with_prefix("_test_event_listener_status_reset");

    let cleaner = BackgroundErrorCleaner::default();
    let counter = cleaner.0.clone();
    let mut builder = DefaultCfOnlyBuilder::default();
    builder
        .db_options_mut()
        .add_event_listener(&SysEventListener::new(cleaner.clone()));
    let db = builder
        .set_create_if_missing(true)
        .open(path.path())
        .unwrap();

    let write_opt = WriteOptions::default();
    let cf = db.default_cf();
    let mut flush_opt = FlushOptions::default();
    flush_opt.set_wait(true);
    for i in 1..5 {
        db.put(&write_opt, cf, format!("{:04}", i).as_bytes(), b"value")
            .unwrap();
    }
    db.flush(&flush_opt, cf).unwrap();

    disturb_sst_file(&db, path.path());

    for i in 1..5 {
        db.put(&write_opt, cf, format!("{:04}", i).as_bytes(), b"value")
            .unwrap();
    }
    compact_files_to_bottom(&db);
    db.flush(&flush_opt, cf).unwrap();
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

fn disturb_sst_file(db: &RawDb, path: &Path) {
    let files = db.live_files_metadata();
    let f = &files[0];
    let file_name = f.name().unwrap();

    let sst_path = path.to_path_buf().join(&file_name[1..]);

    let mut file = std::fs::File::create(sst_path).unwrap();
    file.write(b"surprise").unwrap();
    file.sync_all().unwrap();
}

fn compact_files_to_bottom(db: &RawDb) {
    let mut opt = CompactRangeOptions::default();
    opt.set_max_subcompactions(1);
    // output_level should be from 0 to 6.
    let cf = db.default_cf();
    let _ = db.compact_range(&opt, cf, None, None);
}
