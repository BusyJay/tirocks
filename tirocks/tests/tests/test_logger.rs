// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::sync::{atomic::*, Arc};
use std::time::Duration;
use std::thread;

use tirocks::OpenOptions;
use tirocks::db::DefaultCfOnlyBuilder;
use tirocks::env::logger::{Logger, LogLevel, SysInfoLogger};

#[derive(Default, Clone)]
struct TestDrop {
    called: Arc<AtomicUsize>,
}

impl Drop for TestDrop {
    fn drop(&mut self) {
        self.called.fetch_add(1, Ordering::SeqCst);
    }
}

#[derive(Default, Clone)]
struct TestLogger {
    print: Arc<AtomicUsize>,
    drop: TestDrop,
}

impl Logger for TestLogger {
    fn logv(&self, _log_level: LogLevel, log: &[u8]) {
        assert!(!log.is_empty());
        self.print.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn test_logger() {
    let path = super::tempdir_with_prefix("_rust_rocksdb_test_create_info_rust_log_opt");
    let mut builder = DefaultCfOnlyBuilder::default();
    let logger = TestLogger::default();
    let drop_called = logger.drop.called.clone();
    let print = logger.print.clone();
    let log = SysInfoLogger::new(logger);
    builder.options_mut().db_options_mut().set_info_log(&log).set_create_if_missing(true).set_info_log_level(LogLevel::DEBUG_LEVEL);
    let db = builder.open(path.path()).unwrap();
    thread::sleep(Duration::from_secs(2));
    assert_ne!(print.load(Ordering::SeqCst), 0);
    assert_eq!(0, drop_called.load(Ordering::SeqCst));
    drop(log);
    assert_eq!(0, drop_called.load(Ordering::SeqCst));
    drop(builder);
    assert_eq!(0, drop_called.load(Ordering::SeqCst));
    db.close();
    assert_eq!(1, drop_called.load(Ordering::SeqCst));
}
