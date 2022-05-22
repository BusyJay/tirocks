// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

mod test_cf;
mod test_compact_range;
mod test_compaction_filter;
mod test_delete_files;
mod test_delete_range;
#[cfg(feature = "encryption")]
mod test_encryption;
mod test_iterator;
mod test_logger;
mod test_misc;

fn tempdir_with_prefix(prefix: &str) -> tempfile::TempDir {
    tempfile::Builder::new().prefix(prefix).tempdir().expect("")
}
