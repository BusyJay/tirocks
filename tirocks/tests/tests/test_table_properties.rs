// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::{collections::HashMap, ffi::CStr, fmt};

use tirocks::{
    db::DefaultCfOnlyBuilder,
    option::{FlushOptions, ReadOptions, WriteOptions},
    properties::table::{
        builtin::{OwnedTablePropertiesCollection, TablePropertiesCollection},
        user::{
            Context, EntryType, SysTablePropertiesCollectorFactory, TablePropertiesCollector,
            TablePropertiesCollectorFactory, UserCollectedProperties,
        },
    },
    table_filter::TableFilter,
    OpenOptions, Result,
};

use super::tempdir_with_prefix;

enum Props {
    NumKeys = 0,
    NumPuts,
    NumMerges,
    NumDeletes,
}

fn encode_u32(x: u32) -> Vec<u8> {
    x.to_le_bytes().to_vec()
}

fn decode_u32(x: &[u8]) -> u32 {
    let mut dst = [0u8; 4];
    dst.copy_from_slice(&x[..4]);
    u32::from_le_bytes(dst)
}

struct ExampleCollector {
    num_keys: u32,
    num_puts: u32,
    num_merges: u32,
    num_deletes: u32,
    last_key: Vec<u8>,
}

impl ExampleCollector {
    fn new() -> ExampleCollector {
        ExampleCollector {
            num_keys: 0,
            num_puts: 0,
            num_merges: 0,
            num_deletes: 0,
            last_key: Vec::new(),
        }
    }

    fn add(&mut self, other: &ExampleCollector) {
        self.num_keys += other.num_keys;
        self.num_puts += other.num_puts;
        self.num_merges += other.num_merges;
        self.num_deletes += other.num_deletes;
    }

    fn encode(&self, properties: &mut UserCollectedProperties) {
        properties.add(&[Props::NumKeys as u8], &encode_u32(self.num_keys));
        properties.add(&[Props::NumPuts as u8], &encode_u32(self.num_puts));
        properties.add(&[Props::NumMerges as u8], &encode_u32(self.num_merges));
        properties.add(&[Props::NumDeletes as u8], &encode_u32(self.num_deletes));
    }

    fn decode(props: &UserCollectedProperties) -> ExampleCollector {
        assert!(!props.is_empty());
        let mut c = ExampleCollector::new();
        c.num_keys = decode_u32(&props[&[Props::NumKeys as u8]]);
        c.num_puts = decode_u32(&props[&[Props::NumPuts as u8]]);
        c.num_merges = decode_u32(&props[&[Props::NumMerges as u8]]);
        c.num_deletes = decode_u32(&props[&[Props::NumDeletes as u8]]);

        for (k, v) in props {
            assert_eq!(v, props.get(k).unwrap());
        }
        assert!(props
            .get(&[Props::NumKeys as u8, Props::NumPuts as u8])
            .is_none());
        assert!(props.len() >= 4);

        c
    }
}

impl fmt::Display for ExampleCollector {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "keys={}, puts={}, merges={}, deletes={}",
            self.num_keys, self.num_puts, self.num_merges, self.num_deletes
        )
    }
}

impl TablePropertiesCollector for ExampleCollector {
    fn add(&mut self, key: &[u8], _: &[u8], entry_type: EntryType, _: u64, _: u64) -> Result<()> {
        if key != self.last_key.as_slice() {
            self.num_keys += 1;
            self.last_key.clear();
            self.last_key.extend_from_slice(key);
        }
        match entry_type {
            EntryType::kEntryPut => self.num_puts += 1,
            EntryType::kEntryMerge => self.num_merges += 1,
            EntryType::kEntryDelete | EntryType::kEntrySingleDelete => self.num_deletes += 1,
            _ => {}
        }
        Ok(())
    }

    fn finish(&mut self, properties: &mut UserCollectedProperties) -> Result<()> {
        self.encode(properties);
        Ok(())
    }

    fn name(&self) -> &CStr {
        CStr::from_bytes_with_nul(b"ExampleCollector\0").unwrap()
    }
}

struct ExampleFactory {}

impl ExampleFactory {
    fn new() -> ExampleFactory {
        ExampleFactory {}
    }
}

impl TablePropertiesCollectorFactory for ExampleFactory {
    type Collector = ExampleCollector;
    fn create_table_properties_collector(&self, _: Context) -> ExampleCollector {
        ExampleCollector::new()
    }

    fn name(&self) -> &CStr {
        CStr::from_bytes_with_nul(b"example-collector\0").unwrap()
    }
}

fn check_collection(
    collection: &TablePropertiesCollection,
    num_files: usize,
    num_keys: u32,
    num_puts: u32,
    num_merges: u32,
    num_deletes: u32,
) {
    let mut res = ExampleCollector::new();
    assert!(!collection.is_empty());
    let props: HashMap<_, _> = collection.into_iter().collect();
    assert_eq!(props.len(), collection.len());
    for (k, v) in &props {
        assert!(k.ends_with(b".sst"));
        assert_eq!(v.property_collectors_names(), Ok("[example-collector]"));
        res.add(&ExampleCollector::decode(v.user_collected_properties()));
    }
    assert_eq!(props.len(), num_files);
    assert_eq!(res.num_keys, num_keys);
    assert_eq!(res.num_puts, num_puts);
    assert_eq!(res.num_merges, num_merges);
    assert_eq!(res.num_deletes, num_deletes);
}

#[test]
fn test_table_properties_collector_factory() {
    let path = tempdir_with_prefix("_rust_rocksdb_collectortest");
    let f = SysTablePropertiesCollectorFactory::new(ExampleFactory::new());
    let mut builder = DefaultCfOnlyBuilder::default();
    builder
        .set_create_if_missing(true)
        .cf_options_mut()
        .add_table_properties_collector_factory(&f);
    let db = builder.open(path.path()).unwrap();

    let samples = vec![
        (b"key1".to_vec(), b"value1".to_vec()),
        (b"key2".to_vec(), b"value2".to_vec()),
        (b"key3".to_vec(), b"value3".to_vec()),
        (b"key4".to_vec(), b"value4".to_vec()),
    ];

    let write_opts = WriteOptions::default();
    let read_opts = ReadOptions::default();
    let cf = db.default_cf();
    // Put 4 keys.
    for &(ref k, ref v) in &samples {
        db.put(&write_opts, cf, k, v).unwrap();
        assert_eq!(Ok(Some(v.clone())), db.get(&read_opts, cf, k));
    }
    db.flush(FlushOptions::default().set_wait(true), cf)
        .unwrap();
    let mut collection = OwnedTablePropertiesCollection::default();
    db.properties_of_all_tables(cf, &mut collection).unwrap();
    check_collection(&collection, 1, 4, 4, 0, 0);

    // Delete 2 keys.
    for &(ref k, _) in &samples[0..2] {
        db.delete(&write_opts, cf, k).unwrap();
    }
    db.flush(FlushOptions::default().set_wait(true), cf)
        .unwrap();
    collection.clear();
    db.properties_of_all_tables(cf, &mut collection).unwrap();
    check_collection(&collection, 2, 6, 4, 0, 2);

    // ["key2", "key3") covers two sst files.
    collection.clear();
    db.properties_of_tables_in_range(cf, &[(b"key2", b"key3")], &mut collection)
        .unwrap();
    check_collection(&collection, 2, 6, 4, 0, 2);

    // ["key3", "key4") covers only the first sst file.
    collection.clear();
    db.properties_of_tables_in_range(cf, &[(b"key3", b"key4")], &mut collection)
        .unwrap();
    check_collection(&collection, 1, 4, 4, 0, 0);
}

struct BigTableFilter {
    max_entries: u64,
}

impl BigTableFilter {
    pub fn new(max_entries: u64) -> BigTableFilter {
        BigTableFilter {
            max_entries: max_entries,
        }
    }
}

impl TableFilter for BigTableFilter {
    fn filter(&self, properties: &tirocks::properties::table::builtin::TableProperties) -> bool {
        properties.num_entries() <= self.max_entries
    }
}

#[test]
fn test_table_properties_with_table_filter() {
    let f = SysTablePropertiesCollectorFactory::new(ExampleFactory::new());
    let mut builder = DefaultCfOnlyBuilder::default();
    builder
        .set_create_if_missing(true)
        .cf_options_mut()
        .add_table_properties_collector_factory(&f);
    let path = tempdir_with_prefix("_rust_rocksdb_collector_with_table_filter");
    let db = builder.open(path.path()).unwrap();
    let write_opts = WriteOptions::default();
    let read_opts = ReadOptions::default();
    let mut flush_opts = FlushOptions::default();
    flush_opts.set_wait(true);
    let cf = db.default_cf();

    // Generate a sst with 4 entries.
    let samples = vec![
        (b"key1".to_vec(), b"value1".to_vec()),
        (b"key2".to_vec(), b"value2".to_vec()),
        (b"key3".to_vec(), b"value3".to_vec()),
        (b"key4".to_vec(), b"value4".to_vec()),
    ];
    for &(ref k, ref v) in &samples {
        db.put(&write_opts, cf, k, v).unwrap();
        assert_eq!(Ok(Some(v.clone())), db.get(&read_opts, cf, k));
    }
    db.flush(&flush_opts, cf).unwrap();

    // Generate a sst with 2 entries
    let samples = vec![
        (b"key5".to_vec(), b"value5".to_vec()),
        (b"key6".to_vec(), b"value6".to_vec()),
    ];
    for &(ref k, ref v) in &samples {
        db.put(&write_opts, cf, k, v).unwrap();
        assert_eq!(Ok(Some(v.clone())), db.get(&read_opts, cf, k));
    }
    db.flush(&flush_opts, cf).unwrap();

    // Scan with table filter
    let f = BigTableFilter::new(2);
    let mut ropts = ReadOptions::default();
    ropts.set_table_filter(f);
    let mut iter = db.iter(&mut ropts, cf);
    let key = b"key";
    let key5 = b"key5";
    iter.seek(key);
    assert!(iter.valid());
    // First sst will be skipped
    assert_eq!(iter.key(), key5.as_ref());
}
