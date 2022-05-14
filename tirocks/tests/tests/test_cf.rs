// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use tirocks::{mem_table::SysMemTableRepFactory, CfOptions};

#[test]
pub fn test_column_family_option_use_doubly_skiplist() {
    let mut cf_opts = CfOptions::default();
    let memtable_name = cf_opts.memtable_factory_name();
    assert_eq!(memtable_name, Ok("SkipListFactory"));
    let list = SysMemTableRepFactory::with_doubly_skip_list(0);
    assert_eq!(list.name(), Ok("DoublySkipListFactory"));
    cf_opts.set_memtable_factory(&list);
    let memtable_name = cf_opts.memtable_factory_name();
    assert_eq!(memtable_name, Ok("DoublySkipListFactory"));
}
