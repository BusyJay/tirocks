// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

mod test_cf;
mod test_compact_range;
mod test_compaction_filter;
mod test_delete_files;
mod test_delete_range;
#[cfg(feature = "encryption")]
mod test_encryption;
mod test_event_listener;
mod test_ingest_external_file;
mod test_iterator;
mod test_logger;
mod test_metadata;
mod test_misc;
mod test_multithread;
mod test_options;
mod test_prefix_extractor;
mod test_rate_limiter;
mod test_readonly;
mod test_slice_transform;
mod test_table_properties;
mod test_titan;

fn tempdir_with_prefix(prefix: &str) -> tempfile::TempDir {
    tempfile::Builder::new().prefix(prefix).tempdir().expect("")
}
