// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use tirocks::{
    db::{DefaultCfOnlyBuilder, MultiCfBuilder},
    mem_table::SysMemTableRepFactory,
    merge_operator::{NewValue, SysMergeOperator},
    option::{ReadOptions, WriteOptions},
    CfOptions, Code, OpenOptions,
};

#[test]
pub fn test_cf() {
    let path = super::tempdir_with_prefix("_rust_rocksdb_cftest");

    let mut builder = DefaultCfOnlyBuilder::default();
    builder
        .set_create_if_missing(true)
        .cf_options_mut()
        .set_merge_operator(&SysMergeOperator::new(test_provided_merge));

    // should be able to create column families
    let mut db = builder.open(path.path()).unwrap();
    db.create_cf("cf1", CfOptions::default()).unwrap();
    let names: Vec<_> = db.cfs().map(|c| c.name().unwrap()).collect();
    assert_eq!(names, vec!["default", "cf1"]);
    db.close().unwrap();

    // should fail to open db without specifying same column families
    let err = builder.open(path.path()).unwrap_err();
    assert_eq!(err.code(), Code::kInvalidArgument);
    let msg = err.message().unwrap().unwrap();
    assert!(msg.starts_with("Column families not opened:"), "{}", msg);

    // should properly open db when specifying all column families
    let mut builder = MultiCfBuilder::default();
    let mut default_cf_opts = CfOptions::default();
    default_cf_opts.set_merge_operator(&SysMergeOperator::new(test_provided_merge));
    let mut db = builder
        .add_cf("default", default_cf_opts)
        .add_cf("cf1", CfOptions::default())
        .open(path.path())
        .unwrap();

    // TODO should be able to write, read, merge, batch, and iterate over a cf
    let write_opt = WriteOptions::default();
    let read_opt = ReadOptions::default();
    let cf1 = db.cf("cf1").unwrap();
    db.put(&write_opt, cf1, b"k1", b"v1").unwrap();
    assert_eq!(db.get(&read_opt, cf1, b"k1"), Ok(Some(b"v1".to_vec())));
    db.put(&write_opt, cf1, b"k1", b"a").unwrap();

    // TODO should be able to use writebatch ops with a cf
    // TODO should be able to iterate over a cf

    // should be able to drop a cf
    db.destroy_cf("cf1").unwrap();
}

pub fn test_provided_merge(
    _: &[u8],
    existing_val: Option<&[u8]>,
    operands: &[&[u8]],
    new_value: &mut NewValue,
) -> bool {
    if let Some(v) = existing_val {
        new_value.append(v);
    } else {
        // clear.
        new_value.append(b"");
    }
    for op in operands {
        new_value.append(op);
    }
    true
}

#[test]
pub fn test_cf_option_use_doubly_skiplist() {
    let mut cf_opts = CfOptions::default();
    let memtable_name = cf_opts.memtable_factory_name();
    assert_eq!(memtable_name, Ok("SkipListFactory"));
    let list = SysMemTableRepFactory::with_doubly_skip_list(0);
    assert_eq!(list.name(), Ok("DoublySkipListFactory"));
    cf_opts.set_memtable_factory(&list);
    let memtable_name = cf_opts.memtable_factory_name();
    assert_eq!(memtable_name, Ok("DoublySkipListFactory"));
}
