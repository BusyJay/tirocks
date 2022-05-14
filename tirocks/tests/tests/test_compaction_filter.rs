// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::{sync::{Arc, atomic::{AtomicBool, Ordering}, Mutex}, ffi::CStr};

use tirocks::{compaction_filter::{CompactionFilter, Decision, SysCompactionFilterFactory}, CloneFactory, db::DefaultCfOnlyBuilder, Options, OpenOptions, option::{WriteOptions, ReadOptions, CompactRangeOptions}};

#[derive(Default, Clone)]
struct Filter {
    drop_called: Arc<AtomicBool>,
    filtered_kvs: Arc<Mutex<Vec<(Vec<u8>, Vec<u8>)>>>,
}

impl CompactionFilter for Filter {
    #[inline]
    fn name(&mut self) -> &CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(b"test compaction filter\0") }
    }

    fn filter(
        &mut self,
        _: i32,
        key: &[u8],
        _: tirocks::properties::table::user::SequenceNumber,
        _: tirocks::compaction_filter::ValueType,
        existing_value: &[u8],
    ) -> Decision {
        self.filtered_kvs
            .lock()
            .unwrap()
            .push((key.to_vec(), existing_value.to_vec()));
        Decision::Remove
    }
}

impl Drop for Filter {
    fn drop(&mut self) {
        self.drop_called.store(true, Ordering::Relaxed);
    }
}


#[test]
fn test_compaction_filter() {
    let path = super::tempdir_with_prefix("compactionfilter");
    let filter = Filter::default();

    // reregister with ignore_snapshots set to true
    let mut builder = DefaultCfOnlyBuilder::default();
    let mut opt = Options::default();
    let factory = SysCompactionFilterFactory::new(CloneFactory::new(filter.clone()));
    opt.cf_options_mut().set_compaction_filter_factory(&factory);
    opt.db_options_mut().set_create_if_missing(true);

    let db = builder.set_options(opt).open(path.path()).unwrap();

    let samples = vec![
        (b"key1".to_vec(), b"value1".to_vec()),
        (b"key2".to_vec(), b"value2".to_vec()),
    ];

    let read_opt = ReadOptions::default();
    let write_opt = WriteOptions::default();
    let default_cf = db.default_cf();
    for (k, v) in &samples {
        db.put(&write_opt, default_cf, k, v).unwrap();
        let gv = db.get(&read_opt, default_cf, k).unwrap().unwrap();
        assert_eq!(*v, gv);
    }

    let compact_opt = CompactRangeOptions::default();
    db.compact_range(&compact_opt, default_cf, Some(b"key1"), Some(b"key3")).unwrap();
    for (k, _) in &samples {
        assert_eq!(db.get(&read_opt, default_cf, k), Ok(None));
    }
    assert_eq!(*filter.filtered_kvs.lock().unwrap(), samples);
    assert!(filter.drop_called.load(Ordering::Relaxed));
}

