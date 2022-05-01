/*  Copyright (c) 2011-present, Facebook, Inc.  All rights reserved.
  This source code is licensed under the BSD-style license found in the
  LICENSE file in the root directory of this source tree. An additional grant
  of patent rights can be found in the PATENTS file in the same directory.
 Copyright (c) 2011 The LevelDB Authors. All rights reserved.
  Use of this source code is governed by a BSD-style license that can be
  found in the LICENSE file. See the AUTHORS file for names of contributors.

  C bindings for rocksdb.  May be useful as a stable ABI that can be
  used by programs that keep rocksdb in a shared library, or for
  a JNI api.

  Does not support:
  . getters for the option types
  . custom comparators that implement key shortening
  . capturing post-write-snapshot
  . custom iter, db, env, cache implementations using just the C bindings

  Some conventions:

  (1) We expose just opaque struct pointers and functions to clients.
  This allows us to change internal representations without having to
  recompile clients.

  (2) For simplicity, there is no equivalent to the Slice type.  Instead,
  the caller has to pass the pointer and length as separate
  arguments.

  (3) All operations that can raise an error are passed
  a "Status* errptr" as the last argument. You must call
 crocksdb_free_cplus_array to free `state_` holded in status.

  (4) Bools have the type unsigned char (0 == false; rest == true)

  (5) All of the pointer arguments must be non-NULL.
*/

#ifndef C_ROCKSDB_INCLUDE_CWRAPPER_H_
#define C_ROCKSDB_INCLUDE_CWRAPPER_H_

#pragma once

#ifdef _WIN32
#ifdef C_ROCKSDB_DLL
#ifdef C_ROCKSDB_LIBRARY_EXPORTS
#define C_ROCKSDB_LIBRARY_API __declspec(dllexport)
#else
#define C_ROCKSDB_LIBRARY_API __declspec(dllimport)
#endif
#else
#define C_ROCKSDB_LIBRARY_API
#endif
#else
#define C_ROCKSDB_LIBRARY_API
#endif

#include "rocksdb/compaction_filter.h"
#include "rocksdb/encryption.h"
#include "rocksdb/env.h"
#include "rocksdb/listener.h"
#include "rocksdb/options.h"
#include "rocksdb/perf_level.h"
#include "rocksdb/rate_limiter.h"
#include "rocksdb/slice.h"
#include "rocksdb/sst_partitioner.h"
#include "rocksdb/status.h"
#include "rocksdb/table.h"
#include "rocksdb/types.h"
#include "titan/options.h"

#ifdef OPENSSL
#include "rocksdb/encryption.h"

using namespace rocksdb::encryption;
#endif

using namespace rocksdb;
using namespace rocksdb::titandb;

extern "C" {

#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>

/* Exported types */

typedef struct crocksdb_cloud_envoptions_t crocksdb_cloud_envoptions_t;
typedef struct crocksdb_t crocksdb_t;
typedef struct crocksdb_backup_engine_t crocksdb_backup_engine_t;
typedef struct crocksdb_backup_engine_info_t crocksdb_backup_engine_info_t;
typedef struct crocksdb_restore_options_t crocksdb_restore_options_t;
typedef struct crocksdb_cache_t crocksdb_cache_t;
typedef struct crocksdb_statistics_t crocksdb_statistics_t;
typedef struct crocksdb_compactionfilterfactory_t
    crocksdb_compactionfilterfactory_t;
typedef struct crocksdb_comparator_t crocksdb_comparator_t;
typedef struct crocksdb_filelock_t crocksdb_filelock_t;
typedef struct crocksdb_filterpolicy_t crocksdb_filterpolicy_t;
typedef struct crocksdb_tablefactory_t crocksdb_tablefactory_t;
typedef struct crocksdb_memtablerepfactory_t crocksdb_memtablerepfactory_t;
typedef struct crocksdb_iterator_t crocksdb_iterator_t;
typedef struct crocksdb_logger_t crocksdb_logger_t;
typedef struct crocksdb_logger_impl_t crocksdb_logger_impl_t;
typedef struct crocksdb_mergeoperator_t crocksdb_mergeoperator_t;
typedef struct crocksdb_column_family_descriptor
    crocksdb_column_family_descriptor;
typedef struct crocksdb_cuckoo_table_options_t crocksdb_cuckoo_table_options_t;
typedef struct crocksdb_randomfile_t crocksdb_randomfile_t;
typedef struct crocksdb_seqfile_t crocksdb_seqfile_t;
typedef struct crocksdb_slicetransform_t crocksdb_slicetransform_t;
typedef struct crocksdb_snapshot_t crocksdb_snapshot_t;
typedef struct crocksdb_writablefile_t crocksdb_writablefile_t;
typedef struct crocksdb_writebatch_t crocksdb_writebatch_t;
typedef struct crocksdb_livefiles_t crocksdb_livefiles_t;
typedef struct crocksdb_column_family_handle_t crocksdb_column_family_handle_t;
typedef struct crocksdb_envoptions_t crocksdb_envoptions_t;
typedef struct crocksdb_sequential_file_t crocksdb_sequential_file_t;
typedef struct crocksdb_sstfilereader_t crocksdb_sstfilereader_t;
typedef struct crocksdb_sstfilewriter_t crocksdb_sstfilewriter_t;
typedef struct crocksdb_externalsstfileinfo_t crocksdb_externalsstfileinfo_t;
typedef struct crocksdb_ratelimiter_t crocksdb_ratelimiter_t;
typedef struct crocksdb_pinnableslice_t crocksdb_pinnableslice_t;
typedef struct crocksdb_user_collected_properties_iterator_t
    crocksdb_user_collected_properties_iterator_t;
typedef struct crocksdb_table_properties_collection_iterator_t
    crocksdb_table_properties_collection_iterator_t;
typedef struct crocksdb_table_properties_collector_t
    crocksdb_table_properties_collector_t;
typedef struct crocksdb_table_properties_collector_factory_t
    crocksdb_table_properties_collector_factory_t;
typedef struct crocksdb_eventlistener_t crocksdb_eventlistener_t;
typedef struct crocksdb_keyversions_t crocksdb_keyversions_t;
typedef struct crocksdb_column_family_meta_data_t
    crocksdb_column_family_meta_data_t;
typedef struct crocksdb_level_meta_data_t crocksdb_level_meta_data_t;
typedef struct crocksdb_sst_file_meta_data_t crocksdb_sst_file_meta_data_t;
typedef struct crocksdb_perf_context_t crocksdb_perf_context_t;
typedef struct crocksdb_iostats_context_t crocksdb_iostats_context_t;
typedef struct crocksdb_writestallcondition_t crocksdb_writestallcondition_t;
// Following names are defined as static const std::string in DB::Properties.
// bindgen can parse std string directly, but it can introduce a lot of
// complexity. So defining them as c string instead.
extern Slice const crocksdb_property_name_num_files_at_level_prefix;
extern Slice const crocksdb_property_name_compression_ratio_at_level_prefix;
extern Slice const crocksdb_property_name_stats;
extern Slice const crocksdb_property_name_ss_tables;
extern Slice const crocksdb_property_name_cf_stats;
extern Slice const crocksdb_property_name_cf_stats_no_file_histogram;
extern Slice const crocksdb_property_name_cf_file_histogram;
extern Slice const crocksdb_property_name_db_stats;
extern Slice const crocksdb_property_name_level_stats;
extern Slice const crocksdb_property_name_num_immutable_mem_table;
extern Slice const crocksdb_property_name_num_immutable_mem_table_flushed;
extern Slice const crocksdb_property_name_mem_table_flush_pending;
extern Slice const crocksdb_property_name_num_running_flushes;
extern Slice const crocksdb_property_name_compaction_pending;
extern Slice const crocksdb_property_name_num_running_compactions;
extern Slice const crocksdb_property_name_background_errors;
extern Slice const crocksdb_property_name_cur_size_active_mem_table;
extern Slice const crocksdb_property_name_cur_size_all_mem_tables;
extern Slice const crocksdb_property_name_size_all_mem_tables;
extern Slice const crocksdb_property_name_num_entries_active_mem_table;
extern Slice const crocksdb_property_name_num_entries_imm_mem_tables;
extern Slice const crocksdb_property_name_num_deletes_active_mem_table;
extern Slice const crocksdb_property_name_num_deletes_imm_mem_tables;
extern Slice const crocksdb_property_name_estimate_num_keys;
extern Slice const crocksdb_property_name_estimate_table_readers_mem;
extern Slice const crocksdb_property_name_is_file_deletions_enabled;
extern Slice const crocksdb_property_name_num_snapshots;
extern Slice const crocksdb_property_name_oldest_snapshot_time;
extern Slice const crocksdb_property_name_oldest_snapshot_sequence;
extern Slice const crocksdb_property_name_num_live_versions;
extern Slice const crocksdb_property_name_current_super_version_number;
extern Slice const crocksdb_property_name_estimate_live_data_size;
extern Slice const crocksdb_property_name_min_log_number_to_keep;
extern Slice const crocksdb_property_name_min_obsolete_sst_number_to_keep;
extern Slice const crocksdb_property_name_total_sst_files_size;
extern Slice const crocksdb_property_name_live_sst_files_size;
extern Slice const crocksdb_property_name_base_level;
extern Slice const crocksdb_property_name_estimate_pending_compaction_bytes;
extern Slice const crocksdb_property_name_aggregated_table_properties;
extern Slice const crocksdb_property_name_aggregated_table_properties_at_level;
extern Slice const crocksdb_property_name_actual_delayed_write_rate;
extern Slice const crocksdb_property_name_is_write_stopped;
extern Slice const crocksdb_property_name_is_write_stalled;
extern Slice const crocksdb_property_name_estimate_oldest_key_time;
extern Slice const crocksdb_property_name_block_cache_capacity;
extern Slice const crocksdb_property_name_block_cache_usage;
extern Slice const crocksdb_property_name_block_cache_pinned_usage;
extern Slice const crocksdb_property_name_options_statistics;
typedef struct crocksdb_map_property_t crocksdb_map_property_t;
typedef struct crocksdb_writebatch_iterator_t crocksdb_writebatch_iterator_t;
typedef struct crocksdb_sst_partitioner_t crocksdb_sst_partitioner_t;
typedef struct crocksdb_sst_partitioner_request_t
    crocksdb_sst_partitioner_request_t;
typedef struct crocksdb_sst_partitioner_context_t
    crocksdb_sst_partitioner_context_t;
typedef struct crocksdb_sst_partitioner_factory_t
    crocksdb_sst_partitioner_factory_t;

typedef struct crocksdb_file_system_inspector_t
    crocksdb_file_system_inspector_t;

/* DB operations */

extern C_ROCKSDB_LIBRARY_API crocksdb_t* crocksdb_open(const Options* options,
                                                       Slice name, Status* s);

extern C_ROCKSDB_LIBRARY_API crocksdb_t* crocksdb_open_with_ttl(
    const Options* options, Slice name, int ttl, Status* s);

extern C_ROCKSDB_LIBRARY_API crocksdb_t* crocksdb_open_for_read_only(
    const Options* options, Slice name, unsigned char error_if_log_file_exist,
    Status* s);

extern C_ROCKSDB_LIBRARY_API void rocksdb_resume(crocksdb_t* db, Status* s);

extern C_ROCKSDB_LIBRARY_API crocksdb_backup_engine_t*
crocksdb_backup_engine_open(const Options* options, Slice path, Status* s);

extern C_ROCKSDB_LIBRARY_API void crocksdb_backup_engine_create_new_backup(
    crocksdb_backup_engine_t* be, crocksdb_t* db, Status* s);

extern C_ROCKSDB_LIBRARY_API void crocksdb_backup_engine_purge_old_backups(
    crocksdb_backup_engine_t* be, uint32_t num_backups_to_keep, Status* s);

extern C_ROCKSDB_LIBRARY_API crocksdb_restore_options_t*
crocksdb_restore_options_create();
extern C_ROCKSDB_LIBRARY_API void crocksdb_restore_options_destroy(
    crocksdb_restore_options_t* opt);
extern C_ROCKSDB_LIBRARY_API void crocksdb_restore_options_set_keep_log_files(
    crocksdb_restore_options_t* opt, int v);

extern C_ROCKSDB_LIBRARY_API void
crocksdb_backup_engine_restore_db_from_latest_backup(
    crocksdb_backup_engine_t* be, Slice db_dir, Slice wal_dir,
    const crocksdb_restore_options_t* restore_options, Status* s);

extern C_ROCKSDB_LIBRARY_API const crocksdb_backup_engine_info_t*
crocksdb_backup_engine_get_backup_info(crocksdb_backup_engine_t* be);

extern C_ROCKSDB_LIBRARY_API int crocksdb_backup_engine_info_count(
    const crocksdb_backup_engine_info_t* info);

extern C_ROCKSDB_LIBRARY_API int64_t crocksdb_backup_engine_info_timestamp(
    const crocksdb_backup_engine_info_t* info, int index);

extern C_ROCKSDB_LIBRARY_API uint32_t crocksdb_backup_engine_info_backup_id(
    const crocksdb_backup_engine_info_t* info, int index);

extern C_ROCKSDB_LIBRARY_API uint64_t crocksdb_backup_engine_info_size(
    const crocksdb_backup_engine_info_t* info, int index);

extern C_ROCKSDB_LIBRARY_API uint32_t crocksdb_backup_engine_info_number_files(
    const crocksdb_backup_engine_info_t* info, int index);

extern C_ROCKSDB_LIBRARY_API void crocksdb_backup_engine_info_destroy(
    const crocksdb_backup_engine_info_t* info);

extern C_ROCKSDB_LIBRARY_API void crocksdb_backup_engine_close(
    crocksdb_backup_engine_t* be);

extern C_ROCKSDB_LIBRARY_API crocksdb_t* crocksdb_open_column_families(
    const DBOptions* options, Slice name, int num_column_families,
    Slice* column_family_names,
    const ColumnFamilyOptions** column_family_options,
    crocksdb_column_family_handle_t** column_family_handles, Status* s);

extern C_ROCKSDB_LIBRARY_API crocksdb_t* crocksdb_open_column_families_with_ttl(
    const DBOptions* options, Slice name, int num_column_families,
    Slice* column_family_names,
    const ColumnFamilyOptions** column_family_options, const int32_t* ttl_array,
    unsigned char read_only,
    crocksdb_column_family_handle_t** column_family_handles, Status* s);

extern C_ROCKSDB_LIBRARY_API crocksdb_t*
crocksdb_open_for_read_only_column_families(
    const DBOptions* options, Slice name, int num_column_families,
    Slice* column_family_names,
    const ColumnFamilyOptions** column_family_options,
    crocksdb_column_family_handle_t** column_family_handles,
    unsigned char error_if_log_file_exist, Status* s);

extern C_ROCKSDB_LIBRARY_API char** crocksdb_list_column_families(
    const DBOptions* options, Slice name, size_t* lencf, Status* s);

extern C_ROCKSDB_LIBRARY_API void crocksdb_list_column_families_destroy(
    char** list, size_t len);

extern C_ROCKSDB_LIBRARY_API crocksdb_column_family_handle_t*
crocksdb_create_column_family(crocksdb_t* db,
                              const ColumnFamilyOptions* column_family_options,
                              const char* column_family_name, Status* s);

extern C_ROCKSDB_LIBRARY_API void crocksdb_drop_column_family(
    crocksdb_t* db, crocksdb_column_family_handle_t* handle, Status* s);

extern C_ROCKSDB_LIBRARY_API uint32_t
crocksdb_column_family_handle_id(crocksdb_column_family_handle_t*);

extern C_ROCKSDB_LIBRARY_API void crocksdb_column_family_handle_destroy(
    crocksdb_column_family_handle_t*);

extern C_ROCKSDB_LIBRARY_API void crocksdb_close(crocksdb_t* db);

// This function will wait until all currently running background processes
// finish. After it returns, no background process will be run until
// crocksdb_continue_bg_work is called
extern C_ROCKSDB_LIBRARY_API void crocksdb_pause_bg_work(crocksdb_t* db);
extern C_ROCKSDB_LIBRARY_API void crocksdb_continue_bg_work(crocksdb_t* db);

extern C_ROCKSDB_LIBRARY_API void crocksdb_put(crocksdb_t* db,
                                               const WriteOptions* options,
                                               const char* key, size_t keylen,
                                               const char* val, size_t vallen,
                                               Status* s);

extern C_ROCKSDB_LIBRARY_API void crocksdb_put_cf(
    crocksdb_t* db, const WriteOptions* options,
    crocksdb_column_family_handle_t* column_family, const char* key,
    size_t keylen, const char* val, size_t vallen, Status* s);

extern C_ROCKSDB_LIBRARY_API void crocksdb_delete(crocksdb_t* db,
                                                  const WriteOptions* options,
                                                  const char* key,
                                                  size_t keylen, Status* s);

extern C_ROCKSDB_LIBRARY_API void crocksdb_delete_cf(
    crocksdb_t* db, const WriteOptions* options,
    crocksdb_column_family_handle_t* column_family, const char* key,
    size_t keylen, Status* s);

extern C_ROCKSDB_LIBRARY_API void crocksdb_single_delete(
    crocksdb_t* db, const WriteOptions* options, const char* key, size_t keylen,
    Status* s);

extern C_ROCKSDB_LIBRARY_API void crocksdb_single_delete_cf(
    crocksdb_t* db, const WriteOptions* options,
    crocksdb_column_family_handle_t* column_family, const char* key,
    size_t keylen, Status* s);

extern C_ROCKSDB_LIBRARY_API void crocksdb_delete_range_cf(
    crocksdb_t* db, const WriteOptions* options,
    crocksdb_column_family_handle_t* column_family, const char* begin_key,
    size_t begin_keylen, const char* end_key, size_t end_keylen, Status* s);

extern C_ROCKSDB_LIBRARY_API void crocksdb_merge(crocksdb_t* db,
                                                 const WriteOptions* options,
                                                 const char* key, size_t keylen,
                                                 const char* val, size_t vallen,
                                                 Status* s);

extern C_ROCKSDB_LIBRARY_API void crocksdb_merge_cf(
    crocksdb_t* db, const WriteOptions* options,
    crocksdb_column_family_handle_t* column_family, const char* key,
    size_t keylen, const char* val, size_t vallen, Status* s);

extern C_ROCKSDB_LIBRARY_API void crocksdb_write(crocksdb_t* db,
                                                 const WriteOptions* options,
                                                 crocksdb_writebatch_t* batch,
                                                 Status* s);

/* Returns NULL if not found.  A malloc()ed array otherwise.
   Stores the length of the array in *vallen. */
extern C_ROCKSDB_LIBRARY_API char* crocksdb_get(crocksdb_t* db,
                                                const ReadOptions* options,
                                                const char* key, size_t keylen,
                                                size_t* vallen, Status* s);

extern C_ROCKSDB_LIBRARY_API char* crocksdb_get_cf(
    crocksdb_t* db, const ReadOptions* options,
    crocksdb_column_family_handle_t* column_family, const char* key,
    size_t keylen, size_t* vallen, Status* s);

// if values_list[i] == NULL and errs[i] == NULL,
// then we got status.IsNotFound(), which we will not return.
// all errors except status status.ok() and status.IsNotFound() are returned.
//
// errs, values_list and values_list_sizes must be num_keys in length,
// allocated by the caller.
// errs is a list of strings as opposed to the conventional one error,
// where errs[i] is the status for retrieval of keys_list[i].
// each non-NULL errs entry is a malloc()ed, null terminated string.
// each non-NULL values_list entry is a malloc()ed array, with
// the length for each stored in values_list_sizes[i].
extern C_ROCKSDB_LIBRARY_API void crocksdb_multi_get(
    crocksdb_t* db, const ReadOptions* options, size_t num_keys,
    const char* const* keys_list, const size_t* keys_list_sizes,
    char** values_list, size_t* values_list_sizes, Status* statuses);

extern C_ROCKSDB_LIBRARY_API void crocksdb_multi_get_cf(
    crocksdb_t* db, const ReadOptions* options,
    const crocksdb_column_family_handle_t* const* column_families,
    size_t num_keys, const char* const* keys_list,
    const size_t* keys_list_sizes, char** values_list,
    size_t* values_list_sizes, Status* statuses);

extern C_ROCKSDB_LIBRARY_API crocksdb_iterator_t* crocksdb_create_iterator(
    crocksdb_t* db, const ReadOptions* options);

extern C_ROCKSDB_LIBRARY_API crocksdb_iterator_t* crocksdb_create_iterator_cf(
    crocksdb_t* db, const ReadOptions* options,
    crocksdb_column_family_handle_t* column_family);

extern C_ROCKSDB_LIBRARY_API void crocksdb_create_iterators(
    crocksdb_t* db, const ReadOptions* opts,
    crocksdb_column_family_handle_t** column_families,
    crocksdb_iterator_t** iterators, size_t size, Status* s);

extern C_ROCKSDB_LIBRARY_API const crocksdb_snapshot_t*
crocksdb_create_snapshot(crocksdb_t* db);

extern C_ROCKSDB_LIBRARY_API void crocksdb_release_snapshot(
    crocksdb_t* db, const crocksdb_snapshot_t* snapshot);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_get_snapshot_sequence_number(const crocksdb_snapshot_t* snapshot);

/* Returns NULL if property name is unknown.
   Else returns a pointer to a malloc()-ed null-terminated value. */
extern C_ROCKSDB_LIBRARY_API crocksdb_map_property_t*
crocksdb_create_map_property();

extern C_ROCKSDB_LIBRARY_API void crocksdb_destroy_map_property(
    crocksdb_map_property_t* info);

extern C_ROCKSDB_LIBRARY_API unsigned char crocksdb_get_map_property_cf(
    crocksdb_t* db, crocksdb_column_family_handle_t* column_family,
    const char* property, crocksdb_map_property_t* data);

extern C_ROCKSDB_LIBRARY_API char* crocksdb_map_property_value(
    crocksdb_map_property_t* info, const char* propname);

extern C_ROCKSDB_LIBRARY_API uint64_t crocksdb_map_property_int_value(
    crocksdb_map_property_t* info, const char* propname);

extern C_ROCKSDB_LIBRARY_API char* crocksdb_property_value(
    crocksdb_t* db, const char* propname);

extern C_ROCKSDB_LIBRARY_API char* crocksdb_property_value_cf(
    crocksdb_t* db, crocksdb_column_family_handle_t* column_family,
    const char* propname);

extern C_ROCKSDB_LIBRARY_API void crocksdb_approximate_sizes(
    crocksdb_t* db, int num_ranges, const char* const* range_start_key,
    const size_t* range_start_key_len, const char* const* range_limit_key,
    const size_t* range_limit_key_len, uint64_t* sizes);

extern C_ROCKSDB_LIBRARY_API void crocksdb_approximate_sizes_cf(
    crocksdb_t* db, crocksdb_column_family_handle_t* column_family,
    int num_ranges, const char* const* range_start_key,
    const size_t* range_start_key_len, const char* const* range_limit_key,
    const size_t* range_limit_key_len, uint64_t* sizes);

extern C_ROCKSDB_LIBRARY_API void crocksdb_approximate_memtable_stats(
    const crocksdb_t* db, const char* range_start_key,
    size_t range_start_key_len, const char* range_limit_key,
    size_t range_limit_key_len, uint64_t* count, uint64_t* size);

extern C_ROCKSDB_LIBRARY_API void crocksdb_approximate_memtable_stats_cf(
    const crocksdb_t* db, const crocksdb_column_family_handle_t* cf,
    const char* range_start_key, size_t range_start_key_len,
    const char* range_limit_key, size_t range_limit_key_len, uint64_t* count,
    uint64_t* size);

extern C_ROCKSDB_LIBRARY_API void crocksdb_compact_range(crocksdb_t* db,
                                                         const char* start_key,
                                                         size_t start_key_len,
                                                         const char* limit_key,
                                                         size_t limit_key_len);

extern C_ROCKSDB_LIBRARY_API void crocksdb_compact_range_cf(
    crocksdb_t* db, crocksdb_column_family_handle_t* column_family,
    const char* start_key, size_t start_key_len, const char* limit_key,
    size_t limit_key_len);

extern C_ROCKSDB_LIBRARY_API void crocksdb_compact_range_opt(
    crocksdb_t* db, const CompactRangeOptions* opt, const char* start_key,
    size_t start_key_len, const char* limit_key, size_t limit_key_len);

extern C_ROCKSDB_LIBRARY_API void crocksdb_compact_range_cf_opt(
    crocksdb_t* db, crocksdb_column_family_handle_t* column_family,
    const CompactRangeOptions* opt, const char* start_key, size_t start_key_len,
    const char* limit_key, size_t limit_key_len);

extern C_ROCKSDB_LIBRARY_API void crocksdb_delete_file(crocksdb_t* db,
                                                       const char* name,
                                                       Status* s);

extern C_ROCKSDB_LIBRARY_API const crocksdb_livefiles_t* crocksdb_livefiles(
    crocksdb_t* db);

extern C_ROCKSDB_LIBRARY_API void crocksdb_flush(crocksdb_t* db,
                                                 const FlushOptions* options,
                                                 Status* s);

extern C_ROCKSDB_LIBRARY_API void crocksdb_flush_cf(
    crocksdb_t* db, crocksdb_column_family_handle_t* column_family,
    const FlushOptions* options, Status* s);

extern C_ROCKSDB_LIBRARY_API void crocksdb_flush_cfs(
    crocksdb_t* db, const crocksdb_column_family_handle_t** column_familys,
    int num_handles, const FlushOptions* options, Status* s);

extern C_ROCKSDB_LIBRARY_API void crocksdb_flush_wal(crocksdb_t* db,
                                                     unsigned char sync,
                                                     Status* s);

extern C_ROCKSDB_LIBRARY_API void crocksdb_sync_wal(crocksdb_t* db, Status* s);

extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_get_latest_sequence_number(crocksdb_t* db);

extern C_ROCKSDB_LIBRARY_API void crocksdb_disable_file_deletions(
    crocksdb_t* db, Status* s);

extern C_ROCKSDB_LIBRARY_API void crocksdb_enable_file_deletions(
    crocksdb_t* db, unsigned char force, Status* s);

extern C_ROCKSDB_LIBRARY_API DBOptions* crocksdb_get_db_options(crocksdb_t* db);

extern C_ROCKSDB_LIBRARY_API void crocksdb_set_db_options(crocksdb_t* db,
                                                          const char** names,
                                                          const char** values,
                                                          size_t num_options,
                                                          Status* s);

extern C_ROCKSDB_LIBRARY_API Options* crocksdb_get_options_cf(
    const crocksdb_t* db, crocksdb_column_family_handle_t* column_family);

extern C_ROCKSDB_LIBRARY_API void crocksdb_set_options_cf(
    crocksdb_t* db, crocksdb_column_family_handle_t* cf, const char** names,
    const char** values, size_t num_options, Status* s);

/* Management operations */

extern C_ROCKSDB_LIBRARY_API void crocksdb_destroy_db(const Options* options,
                                                      const char* name,
                                                      Status* s);

extern C_ROCKSDB_LIBRARY_API void crocksdb_repair_db(const Options* options,
                                                     const char* name,
                                                     Status* s);

/* Iterator */

extern C_ROCKSDB_LIBRARY_API void crocksdb_iter_destroy(crocksdb_iterator_t*);
extern C_ROCKSDB_LIBRARY_API unsigned char crocksdb_iter_valid(
    const crocksdb_iterator_t*);
extern C_ROCKSDB_LIBRARY_API void crocksdb_iter_seek_to_first(
    crocksdb_iterator_t*);
extern C_ROCKSDB_LIBRARY_API void crocksdb_iter_seek_to_last(
    crocksdb_iterator_t*);
extern C_ROCKSDB_LIBRARY_API void crocksdb_iter_seek(crocksdb_iterator_t*,
                                                     const char* k,
                                                     size_t klen);
extern C_ROCKSDB_LIBRARY_API void crocksdb_iter_seek_for_prev(
    crocksdb_iterator_t*, const char* k, size_t klen);
extern C_ROCKSDB_LIBRARY_API void crocksdb_iter_next(crocksdb_iterator_t*);
extern C_ROCKSDB_LIBRARY_API void crocksdb_iter_prev(crocksdb_iterator_t*);
extern C_ROCKSDB_LIBRARY_API const char* crocksdb_iter_key(
    const crocksdb_iterator_t*, size_t* klen);
extern C_ROCKSDB_LIBRARY_API const char* crocksdb_iter_value(
    const crocksdb_iterator_t*, size_t* vlen);
extern C_ROCKSDB_LIBRARY_API void crocksdb_iter_get_error(
    const crocksdb_iterator_t*, Status* s);

/* Write batch */

extern C_ROCKSDB_LIBRARY_API crocksdb_writebatch_t*
crocksdb_writebatch_create();
extern C_ROCKSDB_LIBRARY_API crocksdb_writebatch_t*
crocksdb_writebatch_create_with_capacity(size_t reserved_bytes);
extern C_ROCKSDB_LIBRARY_API crocksdb_writebatch_t*
crocksdb_writebatch_create_from(const char* rep, size_t size);
extern C_ROCKSDB_LIBRARY_API void crocksdb_writebatch_destroy(
    crocksdb_writebatch_t*);
extern C_ROCKSDB_LIBRARY_API void crocksdb_writebatch_clear(
    crocksdb_writebatch_t*);
extern C_ROCKSDB_LIBRARY_API int crocksdb_writebatch_count(
    crocksdb_writebatch_t*);
extern C_ROCKSDB_LIBRARY_API void crocksdb_writebatch_put(
    crocksdb_writebatch_t*, const char* key, size_t klen, const char* val,
    size_t vlen);
extern C_ROCKSDB_LIBRARY_API void crocksdb_writebatch_put_cf(
    crocksdb_writebatch_t*, crocksdb_column_family_handle_t* column_family,
    const char* key, size_t klen, const char* val, size_t vlen);
extern C_ROCKSDB_LIBRARY_API void crocksdb_writebatch_putv(
    crocksdb_writebatch_t* b, int num_keys, const char* const* keys_list,
    const size_t* keys_list_sizes, int num_values,
    const char* const* values_list, const size_t* values_list_sizes);
extern C_ROCKSDB_LIBRARY_API void crocksdb_writebatch_putv_cf(
    crocksdb_writebatch_t* b, crocksdb_column_family_handle_t* column_family,
    int num_keys, const char* const* keys_list, const size_t* keys_list_sizes,
    int num_values, const char* const* values_list,
    const size_t* values_list_sizes);
extern C_ROCKSDB_LIBRARY_API void crocksdb_writebatch_merge(
    crocksdb_writebatch_t*, const char* key, size_t klen, const char* val,
    size_t vlen);
extern C_ROCKSDB_LIBRARY_API void crocksdb_writebatch_merge_cf(
    crocksdb_writebatch_t*, crocksdb_column_family_handle_t* column_family,
    const char* key, size_t klen, const char* val, size_t vlen);
extern C_ROCKSDB_LIBRARY_API void crocksdb_writebatch_mergev(
    crocksdb_writebatch_t* b, int num_keys, const char* const* keys_list,
    const size_t* keys_list_sizes, int num_values,
    const char* const* values_list, const size_t* values_list_sizes);
extern C_ROCKSDB_LIBRARY_API void crocksdb_writebatch_mergev_cf(
    crocksdb_writebatch_t* b, crocksdb_column_family_handle_t* column_family,
    int num_keys, const char* const* keys_list, const size_t* keys_list_sizes,
    int num_values, const char* const* values_list,
    const size_t* values_list_sizes);
extern C_ROCKSDB_LIBRARY_API void crocksdb_writebatch_delete(
    crocksdb_writebatch_t*, const char* key, size_t klen);
extern C_ROCKSDB_LIBRARY_API void crocksdb_writebatch_delete_cf(
    crocksdb_writebatch_t*, crocksdb_column_family_handle_t* column_family,
    const char* key, size_t klen);
extern C_ROCKSDB_LIBRARY_API void crocksdb_writebatch_single_delete(
    crocksdb_writebatch_t*, const char* key, size_t klen);
extern C_ROCKSDB_LIBRARY_API void crocksdb_writebatch_single_delete_cf(
    crocksdb_writebatch_t*, crocksdb_column_family_handle_t* column_family,
    const char* key, size_t klen);
extern C_ROCKSDB_LIBRARY_API void crocksdb_writebatch_deletev(
    crocksdb_writebatch_t* b, int num_keys, const char* const* keys_list,
    const size_t* keys_list_sizes);
extern C_ROCKSDB_LIBRARY_API void crocksdb_writebatch_deletev_cf(
    crocksdb_writebatch_t* b, crocksdb_column_family_handle_t* column_family,
    int num_keys, const char* const* keys_list, const size_t* keys_list_sizes);
extern C_ROCKSDB_LIBRARY_API void crocksdb_writebatch_delete_range(
    crocksdb_writebatch_t* b, const char* start_key, size_t start_key_len,
    const char* end_key, size_t end_key_len);
extern C_ROCKSDB_LIBRARY_API void crocksdb_writebatch_delete_range_cf(
    crocksdb_writebatch_t* b, crocksdb_column_family_handle_t* column_family,
    const char* start_key, size_t start_key_len, const char* end_key,
    size_t end_key_len);
extern C_ROCKSDB_LIBRARY_API void crocksdb_writebatch_delete_rangev(
    crocksdb_writebatch_t* b, int num_keys, const char* const* start_keys_list,
    const size_t* start_keys_list_sizes, const char* const* end_keys_list,
    const size_t* end_keys_list_sizes);
extern C_ROCKSDB_LIBRARY_API void crocksdb_writebatch_delete_rangev_cf(
    crocksdb_writebatch_t* b, crocksdb_column_family_handle_t* column_family,
    int num_keys, const char* const* start_keys_list,
    const size_t* start_keys_list_sizes, const char* const* end_keys_list,
    const size_t* end_keys_list_sizes);
extern C_ROCKSDB_LIBRARY_API void crocksdb_writebatch_put_log_data(
    crocksdb_writebatch_t*, const char* blob, size_t len);
extern C_ROCKSDB_LIBRARY_API void crocksdb_writebatch_iterate(
    crocksdb_writebatch_t*, void* state,
    void (*put)(void*, const char* k, size_t klen, const char* v, size_t vlen),
    void (*deleted)(void*, const char* k, size_t klen));

extern C_ROCKSDB_LIBRARY_API void crocksdb_writebatch_iterate_cf(
    crocksdb_writebatch_t* b, void* state,
    void (*put)(void*, const char* k, size_t klen, const char* v, size_t vlen),
    void (*put_cf)(void*, uint32_t cf, const char* k, size_t klen,
                   const char* v, size_t vlen),
    void (*deleted)(void*, const char* k, size_t klen),
    void (*deleted_cf)(void*, uint32_t cf, const char* k, size_t klen));

extern C_ROCKSDB_LIBRARY_API const char* crocksdb_writebatch_data(
    crocksdb_writebatch_t*, size_t* size);
extern C_ROCKSDB_LIBRARY_API void crocksdb_writebatch_set_save_point(
    crocksdb_writebatch_t*);
extern C_ROCKSDB_LIBRARY_API void crocksdb_writebatch_pop_save_point(
    crocksdb_writebatch_t*, Status* s);
extern C_ROCKSDB_LIBRARY_API void crocksdb_writebatch_rollback_to_save_point(
    crocksdb_writebatch_t*, Status* s);
extern C_ROCKSDB_LIBRARY_API void crocksdb_writebatch_rollback_to_save_point(
    crocksdb_writebatch_t*, Status* s);
extern C_ROCKSDB_LIBRARY_API void crocksdb_writebatch_set_content(
    crocksdb_writebatch_t* b, const char* data, size_t dlen);
extern C_ROCKSDB_LIBRARY_API void crocksdb_writebatch_append_content(
    crocksdb_writebatch_t* dest, const char* data, size_t dlen);
extern C_ROCKSDB_LIBRARY_API int crocksdb_writebatch_ref_count(const char* data,
                                                               size_t dlen);
extern C_ROCKSDB_LIBRARY_API crocksdb_writebatch_iterator_t*
crocksdb_writebatch_ref_iterator_create(const char* data, size_t dlen);
extern C_ROCKSDB_LIBRARY_API crocksdb_writebatch_iterator_t*
crocksdb_writebatch_iterator_create(crocksdb_writebatch_t* dest);
extern C_ROCKSDB_LIBRARY_API void crocksdb_writebatch_iterator_destroy(
    crocksdb_writebatch_iterator_t* it);
extern C_ROCKSDB_LIBRARY_API unsigned char crocksdb_writebatch_iterator_valid(
    crocksdb_writebatch_iterator_t* it);
extern C_ROCKSDB_LIBRARY_API void crocksdb_writebatch_iterator_next(
    crocksdb_writebatch_iterator_t* it);
extern C_ROCKSDB_LIBRARY_API const char* crocksdb_writebatch_iterator_key(
    crocksdb_writebatch_iterator_t* it, size_t* klen);
extern C_ROCKSDB_LIBRARY_API const char* crocksdb_writebatch_iterator_value(
    crocksdb_writebatch_iterator_t* it, size_t* klen);
extern C_ROCKSDB_LIBRARY_API int crocksdb_writebatch_iterator_value_type(
    crocksdb_writebatch_iterator_t* it);
extern C_ROCKSDB_LIBRARY_API uint32_t
crocksdb_writebatch_iterator_column_family_id(
    crocksdb_writebatch_iterator_t* it);

/* Block based table options */

extern C_ROCKSDB_LIBRARY_API BlockBasedTableOptions*
crocksdb_block_based_options_create();
extern C_ROCKSDB_LIBRARY_API void crocksdb_block_based_options_destroy(
    BlockBasedTableOptions* options);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_block_based_options_set_metadata_block_size(
    BlockBasedTableOptions* options, uint64_t block_size);
extern C_ROCKSDB_LIBRARY_API void crocksdb_block_based_options_set_block_size(
    BlockBasedTableOptions* options, size_t block_size);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_block_based_options_set_block_size_deviation(
    BlockBasedTableOptions* options, int block_size_deviation);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_block_based_options_set_block_restart_interval(
    BlockBasedTableOptions* options, int block_restart_interval);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_block_based_options_set_filter_policy(
    BlockBasedTableOptions* options, crocksdb_filterpolicy_t* filter_policy);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_block_based_options_set_no_block_cache(BlockBasedTableOptions* options,
                                                bool no_block_cache);
extern C_ROCKSDB_LIBRARY_API void crocksdb_block_based_options_set_block_cache(
    BlockBasedTableOptions* options, crocksdb_cache_t* block_cache);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_block_based_options_set_block_cache_compressed(
    BlockBasedTableOptions* options, crocksdb_cache_t* block_cache_compressed);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_block_based_options_set_whole_key_filtering(BlockBasedTableOptions*,
                                                     bool);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_block_based_options_set_format_version(BlockBasedTableOptions*,
                                                uint32_t);
extern C_ROCKSDB_LIBRARY_API void crocksdb_block_based_options_set_index_type(
    BlockBasedTableOptions*, BlockBasedTableOptions::IndexType);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_block_based_options_set_hash_index_allow_collision(
    BlockBasedTableOptions*, bool);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_block_based_options_set_partition_filters(BlockBasedTableOptions*,
                                                   bool);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_block_based_options_set_cache_index_and_filter_blocks(
    BlockBasedTableOptions*, bool);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_block_based_options_set_pin_top_level_index_and_filter(
    BlockBasedTableOptions*, bool);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_block_based_options_set_cache_index_and_filter_blocks_with_high_priority(
    BlockBasedTableOptions*, bool);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_block_based_options_set_pin_l0_filter_and_index_blocks_in_cache(
    BlockBasedTableOptions*, bool);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_block_based_options_set_read_amp_bytes_per_bit(BlockBasedTableOptions*,
                                                        uint32_t);
extern C_ROCKSDB_LIBRARY_API crocksdb_tablefactory_t*
crocksdb_tablefactory_create_block_based(const BlockBasedTableOptions*);
extern C_ROCKSDB_LIBRARY_API crocksdb_tablefactory_t*
crocksdb_tablefactory_create_plain(const PlainTableOptions*);
extern C_ROCKSDB_LIBRARY_API void crocksdb_tablefactory_destroy(
    crocksdb_tablefactory_t*);

extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_table_factory(
    ColumnFamilyOptions* opt, const crocksdb_tablefactory_t* table);

extern C_ROCKSDB_LIBRARY_API size_t
crocksdb_options_get_block_cache_usage(const ColumnFamilyOptions* opt);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_block_cache_capacity(
    ColumnFamilyOptions* opt, size_t capacity, Status* s);
extern C_ROCKSDB_LIBRARY_API size_t
crocksdb_options_get_block_cache_capacity(const ColumnFamilyOptions* opt);

/* Flush job info */

extern C_ROCKSDB_LIBRARY_API int crocksdb_flushjobinfo_job_id(
    const FlushJobInfo*);
extern C_ROCKSDB_LIBRARY_API void crocksdb_flushjobinfo_cf_name(
    const FlushJobInfo*, Slice*);
extern C_ROCKSDB_LIBRARY_API void crocksdb_flushjobinfo_file_path(
    const FlushJobInfo*, Slice*);
extern C_ROCKSDB_LIBRARY_API const TableProperties*
crocksdb_flushjobinfo_table_properties(const FlushJobInfo*);
extern C_ROCKSDB_LIBRARY_API bool
crocksdb_flushjobinfo_triggered_writes_slowdown(const FlushJobInfo*);
extern C_ROCKSDB_LIBRARY_API bool crocksdb_flushjobinfo_triggered_writes_stop(
    const FlushJobInfo*);

/* Compaction job info */
extern C_ROCKSDB_LIBRARY_API void crocksdb_compactionjobinfo_status(
    const CompactionJobInfo* info, Status* s);
extern C_ROCKSDB_LIBRARY_API void crocksdb_compactionjobinfo_cf_name(
    const CompactionJobInfo*, Slice*);
extern C_ROCKSDB_LIBRARY_API size_t
crocksdb_compactionjobinfo_input_files_count(const CompactionJobInfo*);
extern C_ROCKSDB_LIBRARY_API void crocksdb_compactionjobinfo_input_file_at(
    const CompactionJobInfo*, size_t pos, Slice*);
extern C_ROCKSDB_LIBRARY_API size_t
crocksdb_compactionjobinfo_output_files_count(const CompactionJobInfo*);
extern C_ROCKSDB_LIBRARY_API void crocksdb_compactionjobinfo_output_file_at(
    const CompactionJobInfo*, size_t pos, Slice*);
extern C_ROCKSDB_LIBRARY_API const TablePropertiesCollection*
crocksdb_compactionjobinfo_table_properties(const CompactionJobInfo*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_compactionjobinfo_elapsed_micros(const CompactionJobInfo*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_compactionjobinfo_num_corrupt_keys(const CompactionJobInfo*);
extern C_ROCKSDB_LIBRARY_API int crocksdb_compactionjobinfo_base_input_level(
    const CompactionJobInfo*);
extern C_ROCKSDB_LIBRARY_API int crocksdb_compactionjobinfo_output_level(
    const CompactionJobInfo*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_compactionjobinfo_input_records(const CompactionJobInfo*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_compactionjobinfo_output_records(const CompactionJobInfo*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_compactionjobinfo_total_input_bytes(const CompactionJobInfo*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_compactionjobinfo_total_output_bytes(const CompactionJobInfo*);
extern C_ROCKSDB_LIBRARY_API size_t
crocksdb_compactionjobinfo_num_input_files(const CompactionJobInfo* info);
extern C_ROCKSDB_LIBRARY_API size_t
crocksdb_compactionjobinfo_num_input_files_at_output_level(
    const CompactionJobInfo* info);
extern C_ROCKSDB_LIBRARY_API CompactionReason
crocksdb_compactionjobinfo_compaction_reason(const CompactionJobInfo* info);

/* Subcompaction job info */
extern C_ROCKSDB_LIBRARY_API void crocksdb_subcompactionjobinfo_status(
    const SubcompactionJobInfo*, Status* s);
extern C_ROCKSDB_LIBRARY_API void crocksdb_subcompactionjobinfo_cf_name(
    const SubcompactionJobInfo*, Slice*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_subcompactionjobinfo_thread_id(const SubcompactionJobInfo*);
extern C_ROCKSDB_LIBRARY_API int crocksdb_subcompactionjobinfo_base_input_level(
    const SubcompactionJobInfo*);
extern C_ROCKSDB_LIBRARY_API int crocksdb_subcompactionjobinfo_output_level(
    const SubcompactionJobInfo*);

/* External file ingestion info */
extern C_ROCKSDB_LIBRARY_API void crocksdb_externalfileingestioninfo_cf_name(
    const ExternalFileIngestionInfo*, Slice*);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_externalfileingestioninfo_internal_file_path(
    const ExternalFileIngestionInfo*, Slice*);
extern C_ROCKSDB_LIBRARY_API const TableProperties*
crocksdb_externalfileingestioninfo_table_properties(
    const ExternalFileIngestionInfo*);
extern C_ROCKSDB_LIBRARY_API int
crocksdb_externalfileingestioninfo_picked_level(
    const ExternalFileIngestionInfo*);

/* External write stall info */
extern C_ROCKSDB_LIBRARY_API void crocksdb_writestallinfo_cf_name(
    const WriteStallInfo*, Slice*);
extern C_ROCKSDB_LIBRARY_API WriteStallCondition
crocksdb_writestallinfo_cur(const WriteStallInfo*);
extern C_ROCKSDB_LIBRARY_API WriteStallCondition
crocksdb_writestallinfo_prev(const WriteStallInfo*);

/* Event listener */

typedef void (*on_flush_begin_cb)(void*, crocksdb_t*, const FlushJobInfo*);
typedef void (*on_flush_completed_cb)(void*, crocksdb_t*, const FlushJobInfo*);
typedef void (*on_compaction_begin_cb)(void*, crocksdb_t*,
                                       const CompactionJobInfo*);
typedef void (*on_compaction_completed_cb)(void*, crocksdb_t*,
                                           const CompactionJobInfo*);
typedef void (*on_subcompaction_begin_cb)(void*, const SubcompactionJobInfo*);
typedef void (*on_subcompaction_completed_cb)(void*,
                                              const SubcompactionJobInfo*);
typedef void (*on_external_file_ingested_cb)(void*, crocksdb_t*,
                                             const ExternalFileIngestionInfo*);
typedef void (*on_background_error_cb)(void*, BackgroundErrorReason, Status* s);
typedef void (*on_stall_conditions_changed_cb)(void*, const WriteStallInfo*);
typedef void (*crocksdb_logger_logv_cb)(void*, InfoLogLevel log_level,
                                        Slice msg);

extern C_ROCKSDB_LIBRARY_API crocksdb_eventlistener_t*
crocksdb_eventlistener_create(
    void* state_, void (*destructor_)(void*), on_flush_begin_cb on_flush_begin,
    on_flush_completed_cb on_flush_completed,
    on_compaction_begin_cb on_compaction_begin,
    on_compaction_completed_cb on_compaction_completed,
    on_subcompaction_begin_cb on_subcompaction_begin,
    on_subcompaction_completed_cb on_subcompaction_completed,
    on_external_file_ingested_cb on_external_file_ingested,
    on_background_error_cb on_background_error,
    on_stall_conditions_changed_cb on_stall_conditions_changed);
extern C_ROCKSDB_LIBRARY_API void crocksdb_eventlistener_destroy(
    crocksdb_eventlistener_t*);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_add_eventlistener(
    DBOptions*, crocksdb_eventlistener_t*);

/* Cuckoo table options */

extern C_ROCKSDB_LIBRARY_API crocksdb_cuckoo_table_options_t*
crocksdb_cuckoo_options_create();
extern C_ROCKSDB_LIBRARY_API void crocksdb_cuckoo_options_destroy(
    crocksdb_cuckoo_table_options_t* options);
extern C_ROCKSDB_LIBRARY_API void crocksdb_cuckoo_options_set_hash_ratio(
    crocksdb_cuckoo_table_options_t* options, double v);
extern C_ROCKSDB_LIBRARY_API void crocksdb_cuckoo_options_set_max_search_depth(
    crocksdb_cuckoo_table_options_t* options, uint32_t v);
extern C_ROCKSDB_LIBRARY_API void crocksdb_cuckoo_options_set_cuckoo_block_size(
    crocksdb_cuckoo_table_options_t* options, uint32_t v);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_cuckoo_options_set_identity_as_first_hash(
    crocksdb_cuckoo_table_options_t* options, unsigned char v);
extern C_ROCKSDB_LIBRARY_API void crocksdb_cuckoo_options_set_use_module_hash(
    crocksdb_cuckoo_table_options_t* options, unsigned char v);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_cuckoo_table_factory(
    ColumnFamilyOptions* opt, crocksdb_cuckoo_table_options_t* table_options);

/* Options */

extern C_ROCKSDB_LIBRARY_API Options* crocksdb_options_create();
extern C_ROCKSDB_LIBRARY_API DBOptions* crocksdb_options_get_dboptions(
    Options*);
extern C_ROCKSDB_LIBRARY_API DBOptions* crocksdb_dboptions_create();
extern C_ROCKSDB_LIBRARY_API ColumnFamilyOptions*
crocksdb_options_get_cfoptions(Options*);
extern C_ROCKSDB_LIBRARY_API ColumnFamilyOptions* crocksdb_cfoptions_create();
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_destroy(Options*);
extern C_ROCKSDB_LIBRARY_API void crocksdb_dboptions_destroy(DBOptions*);
extern C_ROCKSDB_LIBRARY_API void crocksdb_cfoptions_destroy(
    ColumnFamilyOptions*);
extern C_ROCKSDB_LIBRARY_API void crocksdb_column_family_descriptor_destroy(
    crocksdb_column_family_descriptor* cf_desc);
extern C_ROCKSDB_LIBRARY_API const char*
crocksdb_name_from_column_family_descriptor(
    const crocksdb_column_family_descriptor* cf_desc);
extern C_ROCKSDB_LIBRARY_API const ColumnFamilyOptions*
crocksdb_options_from_column_family_descriptor(
    const crocksdb_column_family_descriptor* cf_desc);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_increase_parallelism(
    DBOptions* opt, int total_threads);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_optimize_for_point_lookup(
    ColumnFamilyOptions* opt, uint64_t block_cache_size_mb);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_options_optimize_level_style_compaction(
    ColumnFamilyOptions* opt, uint64_t memtable_memory_budget);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_options_optimize_universal_style_compaction(
    ColumnFamilyOptions* opt, uint64_t memtable_memory_budget);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_options_set_compaction_filter_factory(
    ColumnFamilyOptions*, crocksdb_compactionfilterfactory_t*);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_comparator(
    ColumnFamilyOptions*, crocksdb_comparator_t*);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_merge_operator(
    ColumnFamilyOptions*, crocksdb_mergeoperator_t*);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_compression_per_level(
    ColumnFamilyOptions* opt, const CompressionType* level_values,
    size_t num_levels);
extern C_ROCKSDB_LIBRARY_API size_t
crocksdb_options_get_compression_level_number(const ColumnFamilyOptions* opt);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_get_compression_per_level(
    const ColumnFamilyOptions* opt, CompressionType* level_values);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_bottommost_compression(
    ColumnFamilyOptions* opt, CompressionType c);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_create_if_missing(
    DBOptions*, bool);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_options_set_create_missing_column_families(DBOptions*, bool);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_error_if_exists(
    DBOptions*, bool);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_paranoid_checks(
    DBOptions*, bool);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_env(DBOptions*, Env*);
extern C_ROCKSDB_LIBRARY_API crocksdb_logger_t* crocksdb_logger_create(
    void* rep, void (*destructor_)(void*), crocksdb_logger_logv_cb logv);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_info_log(
    DBOptions*, crocksdb_logger_t*);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_info_log_level(
    DBOptions*, InfoLogLevel);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_write_buffer_size(
    ColumnFamilyOptions*, size_t);
extern C_ROCKSDB_LIBRARY_API size_t
crocksdb_options_get_write_buffer_size(const ColumnFamilyOptions*);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_db_write_buffer_size(
    DBOptions*, size_t);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_max_open_files(
    DBOptions*, int);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_max_total_wal_size(
    DBOptions* opt, uint64_t n);
extern C_ROCKSDB_LIBRARY_API CompressionOptions*
crocksdb_options_get_bottommost_compression_options(ColumnFamilyOptions*);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_compression_options(
    ColumnFamilyOptions*, const CompressionOptions*);
extern C_ROCKSDB_LIBRARY_API CompressionOptions*
crocksdb_options_get_compression_options(ColumnFamilyOptions*);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_use_direct_reads(
    DBOptions* opt, bool v);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_options_set_use_direct_io_for_flush_and_compaction(DBOptions* opt,
                                                            bool v);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_prefix_extractor(
    ColumnFamilyOptions*, crocksdb_slicetransform_t*);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_options_set_memtable_insert_with_hint_prefix_extractor(
    ColumnFamilyOptions*, crocksdb_slicetransform_t*);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_num_levels(
    ColumnFamilyOptions*, int);
extern C_ROCKSDB_LIBRARY_API int crocksdb_options_get_num_levels(
    const ColumnFamilyOptions*);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_options_set_level0_file_num_compaction_trigger(ColumnFamilyOptions*,
                                                        int);
extern C_ROCKSDB_LIBRARY_API int
crocksdb_options_get_level0_file_num_compaction_trigger(
    const ColumnFamilyOptions*);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_options_set_level0_slowdown_writes_trigger(ColumnFamilyOptions*, int);
extern C_ROCKSDB_LIBRARY_API int
crocksdb_options_get_level0_slowdown_writes_trigger(const ColumnFamilyOptions*);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_options_set_level0_stop_writes_trigger(ColumnFamilyOptions*, int);
extern C_ROCKSDB_LIBRARY_API int
crocksdb_options_get_level0_stop_writes_trigger(const ColumnFamilyOptions*);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_target_file_size_base(
    ColumnFamilyOptions*, uint64_t);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_options_get_target_file_size_base(const ColumnFamilyOptions*);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_options_set_target_file_size_multiplier(ColumnFamilyOptions*, int);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_max_bytes_for_level_base(
    ColumnFamilyOptions*, uint64_t);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_options_get_max_bytes_for_level_base(ColumnFamilyOptions*);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_options_set_optimize_filters_for_hits(ColumnFamilyOptions*, bool);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_options_set_level_compaction_dynamic_level_bytes(ColumnFamilyOptions*,
                                                          bool);
extern C_ROCKSDB_LIBRARY_API bool
crocksdb_options_get_level_compaction_dynamic_level_bytes(
    const ColumnFamilyOptions* const options);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_options_set_max_bytes_for_level_multiplier(ColumnFamilyOptions*,
                                                    double);
extern C_ROCKSDB_LIBRARY_API double
crocksdb_options_get_max_bytes_for_level_multiplier(const ColumnFamilyOptions*);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_options_set_max_bytes_for_level_multiplier_additional(
    ColumnFamilyOptions*, const int* level_values, size_t num_levels);
extern C_ROCKSDB_LIBRARY_API crocksdb_sst_partitioner_factory_t*
crocksdb_options_get_sst_partitioner_factory(ColumnFamilyOptions*);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_sst_partitioner_factory(
    ColumnFamilyOptions*, crocksdb_sst_partitioner_factory_t*);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_statistics(
    DBOptions*, crocksdb_statistics_t*);
extern C_ROCKSDB_LIBRARY_API unsigned char crocksdb_load_latest_options(
    const char* dbpath, Env* env, DBOptions* db_options,
    crocksdb_column_family_descriptor*** cf_descs, size_t* cf_descs_len,
    unsigned char ignore_unknown_options, Status* s);

extern C_ROCKSDB_LIBRARY_API crocksdb_statistics_t*
crocksdb_statistics_create();
extern C_ROCKSDB_LIBRARY_API void crocksdb_statistics_destroy(
    crocksdb_statistics_t*);
/* returns a pointer to a malloc()-ed, null terminated string */
extern C_ROCKSDB_LIBRARY_API char* crocksdb_statistics_get_string(
    crocksdb_statistics_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t crocksdb_statistics_get_ticker_count(
    crocksdb_statistics_t*, uint32_t ticker_type);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_statistics_get_and_reset_ticker_count(crocksdb_statistics_t*,
                                               uint32_t ticker_type);
extern C_ROCKSDB_LIBRARY_API char* crocksdb_statistics_get_histogram_string(
    crocksdb_statistics_t*, uint32_t type);
extern C_ROCKSDB_LIBRARY_API void crocksdb_statistics_get_histogram(
    crocksdb_statistics_t*, uint32_t type, double* median, double* percentile95,
    double* percentile99, double* average, double* standard_deviation,
    double* max);

extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_max_write_buffer_number(
    ColumnFamilyOptions*, int);
extern C_ROCKSDB_LIBRARY_API int crocksdb_options_get_max_write_buffer_number(
    const ColumnFamilyOptions*);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_options_set_min_write_buffer_number_to_merge(ColumnFamilyOptions*,
                                                      int);
extern C_ROCKSDB_LIBRARY_API int
crocksdb_options_get_min_write_buffer_number_to_merge(
    const ColumnFamilyOptions*);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_options_set_max_write_buffer_number_to_maintain(ColumnFamilyOptions*,
                                                         int);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_max_background_jobs(
    DBOptions*, int);
extern C_ROCKSDB_LIBRARY_API int crocksdb_options_get_max_background_jobs(
    const DBOptions*);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_options_set_max_background_compactions(DBOptions*, int);
extern C_ROCKSDB_LIBRARY_API int
crocksdb_options_get_max_background_compactions(const DBOptions*);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_max_background_flushes(
    DBOptions*, int);
extern C_ROCKSDB_LIBRARY_API int crocksdb_options_get_max_background_flushes(
    const DBOptions*);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_max_log_file_size(
    DBOptions*, size_t);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_log_file_time_to_roll(
    DBOptions*, size_t);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_keep_log_file_num(
    DBOptions*, size_t);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_recycle_log_file_num(
    DBOptions*, size_t);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_options_set_soft_pending_compaction_bytes_limit(
    ColumnFamilyOptions* opt, uint64_t v);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_options_get_soft_pending_compaction_bytes_limit(
    const ColumnFamilyOptions* opt);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_options_set_hard_pending_compaction_bytes_limit(
    ColumnFamilyOptions* opt, uint64_t v);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_options_get_hard_pending_compaction_bytes_limit(
    const ColumnFamilyOptions* opt);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_max_manifest_file_size(
    DBOptions*, uint64_t);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_table_cache_numshardbits(
    DBOptions*, int);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_options_set_writable_file_max_buffer_size(DBOptions*, size_t);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_arena_block_size(
    ColumnFamilyOptions*, size_t);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_use_fsync(DBOptions*,
                                                                 bool);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_add_db_paths(DBOptions*,
                                                                Slice,
                                                                uint64_t);
extern C_ROCKSDB_LIBRARY_API size_t
crocksdb_options_get_db_paths_num(const DBOptions*);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_get_db_path(const DBOptions*,
                                                               size_t idx,
                                                               Slice* path);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_options_get_path_target_size(const DBOptions*, size_t index);

extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_db_log_dir(DBOptions*,
                                                                  Slice);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_wal_dir(DBOptions*,
                                                               Slice);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_wal_ttl_seconds(
    DBOptions*, uint64_t);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_wal_size_limit_mb(
    DBOptions*, uint64_t);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_options_set_manifest_preallocation_size(DBOptions*, size_t);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_allow_mmap_reads(
    DBOptions*, bool);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_allow_mmap_writes(
    DBOptions*, bool);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_is_fd_close_on_exec(
    DBOptions*, bool);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_stats_dump_period_sec(
    DBOptions*, unsigned int);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_advise_random_on_open(
    DBOptions*, bool);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_options_set_access_hint_on_compaction_start(DBOptions*,
                                                     DBOptions::AccessHint);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_use_adaptive_mutex(
    DBOptions*, bool);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_bytes_per_sync(
    DBOptions*, uint64_t);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_enable_pipelined_write(
    DBOptions*, bool);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_enable_pipelined_commit(
    DBOptions* opt, bool v);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_unordered_write(
    DBOptions*, bool);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_options_set_allow_concurrent_memtable_write(DBOptions*, bool);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_manual_wal_flush(
    DBOptions*, bool);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_options_set_enable_write_thread_adaptive_yield(DBOptions*, bool);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_options_set_max_sequential_skip_in_iterations(ColumnFamilyOptions*,
                                                       uint64_t);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_disable_auto_compactions(
    ColumnFamilyOptions*, bool);
extern C_ROCKSDB_LIBRARY_API bool crocksdb_options_get_disable_auto_compactions(
    const ColumnFamilyOptions*);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_disable_write_stall(
    ColumnFamilyOptions*, bool);
extern C_ROCKSDB_LIBRARY_API bool crocksdb_options_get_disable_write_stall(
    const ColumnFamilyOptions*);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_options_set_delete_obsolete_files_period_micros(DBOptions*, uint64_t);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_prepare_for_bulk_load(
    Options*);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_get_memtable_factory_name(
    ColumnFamilyOptions* opt, Slice* name);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_options_set_memtable_prefix_bloom_size_ratio(ColumnFamilyOptions*,
                                                      double);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_max_compaction_bytes(
    ColumnFamilyOptions*, uint64_t);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_options_get_max_compaction_bytes(const ColumnFamilyOptions*);

extern C_ROCKSDB_LIBRARY_API crocksdb_memtablerepfactory_t*
    crocksdb_memtablerepfactory_create_hash_skip_list(size_t, int32_t, int32_t);
extern C_ROCKSDB_LIBRARY_API crocksdb_memtablerepfactory_t*
    crocksdb_memtablerepfactory_create_hash_link_list(size_t);
extern C_ROCKSDB_LIBRARY_API crocksdb_memtablerepfactory_t*
    crocksdb_memtablerepfactory_create_doubly_skip_list(size_t);
extern C_ROCKSDB_LIBRARY_API crocksdb_memtablerepfactory_t*
crocksdb_memtablerepfactory_create_vector(uint64_t reserved_bytes);
extern C_ROCKSDB_LIBRARY_API void crocksdb_memtablerepfactory_destroy(
    crocksdb_memtablerepfactory_t*);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_memtable_factory(
    ColumnFamilyOptions* opt, const crocksdb_memtablerepfactory_t*);

extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_min_level_to_compress(
    ColumnFamilyOptions* opt, int level);

extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_memtable_huge_page_size(
    ColumnFamilyOptions*, size_t);

extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_max_successive_merges(
    ColumnFamilyOptions*, size_t);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_bloom_locality(
    ColumnFamilyOptions*, uint32_t);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_inplace_update_support(
    ColumnFamilyOptions*, bool);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_inplace_update_num_locks(
    ColumnFamilyOptions*, size_t);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_report_bg_io_stats(
    ColumnFamilyOptions*, bool);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_options_set_compaction_readahead_size(DBOptions*, size_t);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_max_subcompactions(
    DBOptions*, uint32_t);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_wal_bytes_per_sync(
    DBOptions*, uint64_t);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_wal_recovery_mode(
    DBOptions*, WALRecoveryMode);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_compression(
    ColumnFamilyOptions*, CompressionType);
extern C_ROCKSDB_LIBRARY_API CompressionType
crocksdb_options_get_compression(const ColumnFamilyOptions*);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_compaction_style(
    ColumnFamilyOptions*, CompactionStyle);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_options_set_universal_compaction_options(
    ColumnFamilyOptions*, const CompactionOptionsUniversal*);
extern C_ROCKSDB_LIBRARY_API CompactionOptionsUniversal*
crocksdb_options_get_universal_compaction_options(ColumnFamilyOptions*);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_fifo_compaction_options(
    ColumnFamilyOptions* opt, const CompactionOptionsFIFO* fifo);
extern C_ROCKSDB_LIBRARY_API CompactionOptionsFIFO*
crocksdb_options_get_fifo_compaction_options(ColumnFamilyOptions*);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_ratelimiter(
    DBOptions* opt, crocksdb_ratelimiter_t* limiter);
extern C_ROCKSDB_LIBRARY_API crocksdb_ratelimiter_t*
crocksdb_options_get_ratelimiter(const DBOptions* opt);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_atomic_flush(
    DBOptions* opt, bool enable);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_compaction_priority(
    ColumnFamilyOptions*, CompactionPri);

extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_delayed_write_rate(
    DBOptions*, uint64_t);
extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_force_consistency_checks(
    ColumnFamilyOptions*, bool);
extern C_ROCKSDB_LIBRARY_API bool crocksdb_options_get_force_consistency_checks(
    const ColumnFamilyOptions*);

/* RateLimiter */
extern C_ROCKSDB_LIBRARY_API crocksdb_ratelimiter_t*
crocksdb_ratelimiter_create(int64_t rate_bytes_per_sec,
                            int64_t refill_period_us, int32_t fairness);
extern C_ROCKSDB_LIBRARY_API crocksdb_ratelimiter_t*
crocksdb_ratelimiter_create_with_auto_tuned(int64_t rate_bytes_per_sec,
                                            int64_t refill_period_us,
                                            int32_t fairness,
                                            RateLimiter::Mode mode,
                                            unsigned char auto_tuned);
extern C_ROCKSDB_LIBRARY_API crocksdb_ratelimiter_t*
crocksdb_writeampbasedratelimiter_create_with_auto_tuned(
    int64_t rate_bytes_per_sec, int64_t refill_period_us, int32_t fairness,
    RateLimiter::Mode mode, unsigned char auto_tuned);
extern C_ROCKSDB_LIBRARY_API void crocksdb_ratelimiter_destroy(
    crocksdb_ratelimiter_t*);
extern C_ROCKSDB_LIBRARY_API void crocksdb_ratelimiter_set_bytes_per_second(
    crocksdb_ratelimiter_t* limiter, int64_t rate_bytes_per_sec);
extern C_ROCKSDB_LIBRARY_API void crocksdb_ratelimiter_set_auto_tuned(
    crocksdb_ratelimiter_t* limiter, unsigned char auto_tuned);
extern C_ROCKSDB_LIBRARY_API int64_t
crocksdb_ratelimiter_get_singleburst_bytes(crocksdb_ratelimiter_t* limiter);
extern C_ROCKSDB_LIBRARY_API void crocksdb_ratelimiter_request(
    crocksdb_ratelimiter_t* limiter, int64_t bytes, Env::IOPriority pri,
    RateLimiter::OpType op_ty);
extern C_ROCKSDB_LIBRARY_API int64_t
crocksdb_ratelimiter_get_total_bytes_through(crocksdb_ratelimiter_t* limiter,
                                             Env::IOPriority pri);
extern C_ROCKSDB_LIBRARY_API int64_t
crocksdb_ratelimiter_get_bytes_per_second(crocksdb_ratelimiter_t* limiter);
extern C_ROCKSDB_LIBRARY_API unsigned char crocksdb_ratelimiter_get_auto_tuned(
    crocksdb_ratelimiter_t* limiter);
extern C_ROCKSDB_LIBRARY_API int64_t crocksdb_ratelimiter_get_total_requests(
    crocksdb_ratelimiter_t* limiter, Env::IOPriority pri);

/* Compaction Filter Context */

extern C_ROCKSDB_LIBRARY_API bool
crocksdb_compactionfiltercontext_is_full_compaction(
    const CompactionFilter::Context* context);

extern C_ROCKSDB_LIBRARY_API bool
crocksdb_compactionfiltercontext_is_manual_compaction(
    const CompactionFilter::Context* context);

extern C_ROCKSDB_LIBRARY_API bool
crocksdb_compactionfiltercontext_is_bottommost_level(
    const CompactionFilter::Context* context);

extern C_ROCKSDB_LIBRARY_API void crocksdb_compactionfiltercontext_file_numbers(
    const CompactionFilter::Context* context, const uint64_t** buffer,
    size_t* len);

extern C_ROCKSDB_LIBRARY_API const TableProperties*
crocksdb_compactionfiltercontext_table_properties(
    const CompactionFilter::Context* context, size_t offset);

extern C_ROCKSDB_LIBRARY_API void crocksdb_compactionfiltercontext_start_key(
    const CompactionFilter::Context* context, Slice* start_key);

extern C_ROCKSDB_LIBRARY_API void crocksdb_compactionfiltercontext_end_key(
    const CompactionFilter::Context* context, Slice* end_key);

/* Compaction Filter Factory */

extern C_ROCKSDB_LIBRARY_API CompactionFilter* crocksdb_compactionfilter_create(
    void* state, void (*destructor)(void*),
    CompactionFilter::Decision (*filter)(void*, int level, Slice key,
                                         SequenceNumber seqno,
                                         CompactionFilter::ValueType value_type,
                                         Slice existing_value, Slice* new_value,
                                         Slice* skip_until),
    const char* (*name)(void*));

extern C_ROCKSDB_LIBRARY_API crocksdb_compactionfilterfactory_t*
crocksdb_compactionfilterfactory_create(
    void* state, void (*destructor)(void*),
    CompactionFilter* (*create_compaction_filter)(
        void*, const CompactionFilter::Context* context),
    bool (*should_filter_table_file_creation)(void*,
                                              TableFileCreationReason reason),
    const char* (*name)(void*));
extern C_ROCKSDB_LIBRARY_API void crocksdb_compactionfilterfactory_destroy(
    crocksdb_compactionfilterfactory_t*);

/* Comparator */

extern C_ROCKSDB_LIBRARY_API crocksdb_comparator_t* crocksdb_comparator_create(
    void* state, void (*destructor)(void*),
    int (*compare)(void*, Slice lhs, Slice rhs), const char* (*name)(void*));
extern C_ROCKSDB_LIBRARY_API void crocksdb_comparator_destroy(
    crocksdb_comparator_t*);

/* Filter policy */

extern C_ROCKSDB_LIBRARY_API crocksdb_filterpolicy_t*
crocksdb_filterpolicy_create(
    void* state, void (*destructor)(void*),
    char* (*create_filter)(void*, const char* const* key_array,
                           const size_t* key_length_array, int num_keys,
                           size_t* filter_length),
    unsigned char (*key_may_match)(void*, const char* key, size_t length,
                                   const char* filter, size_t filter_length),
    void (*delete_filter)(void*, const char* filter, size_t filter_length),
    const char* (*name)(void*));
extern C_ROCKSDB_LIBRARY_API void crocksdb_filterpolicy_destroy(
    crocksdb_filterpolicy_t*);

extern C_ROCKSDB_LIBRARY_API crocksdb_filterpolicy_t*
crocksdb_filterpolicy_create_bloom_format(int bits_per_key,
                                          bool use_block_based_builder);

/* Merge Operator */

extern C_ROCKSDB_LIBRARY_API crocksdb_mergeoperator_t*
crocksdb_mergeoperator_create(
    void* state, void (*destructor)(void*),
    char* (*full_merge)(void*, const char* key, size_t key_length,
                        const char* existing_value,
                        size_t existing_value_length,
                        const char* const* operands_list,
                        const size_t* operands_list_length, int num_operands,
                        unsigned char* success, size_t* new_value_length),
    char* (*partial_merge)(void*, const char* key, size_t key_length,
                           const char* const* operands_list,
                           const size_t* operands_list_length, int num_operands,
                           unsigned char* success, size_t* new_value_length),
    void (*delete_value)(void*, const char* value, size_t value_length),
    const char* (*name)(void*));
extern C_ROCKSDB_LIBRARY_API void crocksdb_mergeoperator_destroy(
    crocksdb_mergeoperator_t*);

/* Read options */
extern C_ROCKSDB_LIBRARY_API void crocksdb_readoptions_set_table_filter(
    ReadOptions*, void*,
    unsigned char (*table_filter)(void*, const TableProperties*),
    void (*destory)(void*));

/* Write options */

extern C_ROCKSDB_LIBRARY_API void crocksdb_writeoptions_init(WriteOptions*);

/* Compact range options */

extern C_ROCKSDB_LIBRARY_API void crocksdb_compactrangeoptions_init(
    CompactRangeOptions*);

/* Flush options */

extern C_ROCKSDB_LIBRARY_API void crocksdb_flushoptions_init(FlushOptions*);

/* Cache */

extern C_ROCKSDB_LIBRARY_API LRUCacheOptions*
crocksdb_lru_cache_options_create();
extern C_ROCKSDB_LIBRARY_API void crocksdb_lru_cache_options_destroy(
    LRUCacheOptions*);
extern C_ROCKSDB_LIBRARY_API void crocksdb_lru_cache_options_set_capacity(
    LRUCacheOptions*, size_t);
extern C_ROCKSDB_LIBRARY_API void crocksdb_lru_cache_options_set_num_shard_bits(
    LRUCacheOptions*, int);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_lru_cache_options_set_strict_capacity_limit(LRUCacheOptions*, bool);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_lru_cache_options_set_high_pri_pool_ratio(LRUCacheOptions*, double);
extern C_ROCKSDB_LIBRARY_API void crocksdb_lru_cache_options_set_use_jemalloc(
    LRUCacheOptions*, JemallocAllocatorOptions*, Status*);
extern C_ROCKSDB_LIBRARY_API crocksdb_cache_t* crocksdb_cache_create_lru(
    LRUCacheOptions*);
extern C_ROCKSDB_LIBRARY_API void crocksdb_cache_destroy(
    crocksdb_cache_t* cache);
extern C_ROCKSDB_LIBRARY_API void crocksdb_cache_set_capacity(
    crocksdb_cache_t* cache, size_t capacity);

/* Env */

extern C_ROCKSDB_LIBRARY_API Env* crocksdb_default_env_create();
extern C_ROCKSDB_LIBRARY_API Env* crocksdb_mem_env_create(Env*);
extern C_ROCKSDB_LIBRARY_API Env* crocksdb_ctr_encrypted_env_create(
    Env* base_env, const char* ciphertext, size_t ciphertext_len);
extern C_ROCKSDB_LIBRARY_API void crocksdb_env_set_background_threads(
    Env* env, int n, Env::Priority pri);
extern C_ROCKSDB_LIBRARY_API void crocksdb_env_join_all_threads(Env* env);
extern C_ROCKSDB_LIBRARY_API void crocksdb_env_file_exists(Env* env, Slice path,
                                                           Status* s);
extern C_ROCKSDB_LIBRARY_API void crocksdb_env_delete_file(Env* env, Slice path,
                                                           Status* s);
extern C_ROCKSDB_LIBRARY_API void crocksdb_env_destroy(Env*);

extern C_ROCKSDB_LIBRARY_API crocksdb_envoptions_t*
crocksdb_envoptions_create();
extern C_ROCKSDB_LIBRARY_API void crocksdb_envoptions_destroy(
    crocksdb_envoptions_t* opt);

extern C_ROCKSDB_LIBRARY_API crocksdb_sequential_file_t*
crocksdb_sequential_file_create(Env* env, Slice path,
                                const crocksdb_envoptions_t* opts, Status* s);
extern C_ROCKSDB_LIBRARY_API size_t crocksdb_sequential_file_read(
    crocksdb_sequential_file_t*, size_t n, char* buf, Status* s);
extern C_ROCKSDB_LIBRARY_API void crocksdb_sequential_file_skip(
    crocksdb_sequential_file_t*, size_t n, Status* s);
extern C_ROCKSDB_LIBRARY_API void crocksdb_sequential_file_destroy(
    crocksdb_sequential_file_t*);

/* KeyManagedEncryptedEnv */

#ifdef OPENSSL
extern C_ROCKSDB_LIBRARY_API void crocksdb_file_encryption_info_init(
    FileEncryptionInfo* info, EncryptionMethod method, Slice key, Slice iv);
typedef void (*crocksdb_encryption_key_manager_get_file_cb)(
    void* state, Slice fname, FileEncryptionInfo* file_info, Status*);
typedef void (*crocksdb_encryption_key_manager_new_file_cb)(
    void* state, Slice fname, FileEncryptionInfo* file_info, Status*);
typedef void (*crocksdb_encryption_key_manager_delete_file_cb)(void* state,
                                                               Slice fname,
                                                               Status*);
typedef void (*crocksdb_encryption_key_manager_link_file_cb)(void* state,
                                                             Slice src_fname,
                                                             Slice dst_fname,
                                                             Status*);

extern C_ROCKSDB_LIBRARY_API KeyManager* crocksdb_encryption_key_manager_create(
    void* state, void (*destructor)(void*),
    crocksdb_encryption_key_manager_get_file_cb get_file,
    crocksdb_encryption_key_manager_new_file_cb new_file,
    crocksdb_encryption_key_manager_delete_file_cb delete_file,
    crocksdb_encryption_key_manager_link_file_cb link_file);
extern C_ROCKSDB_LIBRARY_API void crocksdb_encryption_key_manager_destroy(
    KeyManager*);

extern C_ROCKSDB_LIBRARY_API Env* crocksdb_key_managed_encrypted_env_create(
    Env*, KeyManager*);
#endif

/* FileSystemInspectedEnv */

typedef size_t (*crocksdb_file_system_inspector_read_cb)(void* state,
                                                         size_t len, Status* s);
typedef size_t (*crocksdb_file_system_inspector_write_cb)(void* state,
                                                          size_t len,
                                                          Status* s);

extern C_ROCKSDB_LIBRARY_API crocksdb_file_system_inspector_t*
crocksdb_file_system_inspector_create(
    void* state, void (*destructor)(void*),
    crocksdb_file_system_inspector_read_cb read,
    crocksdb_file_system_inspector_write_cb write);
extern C_ROCKSDB_LIBRARY_API void crocksdb_file_system_inspector_destroy(
    crocksdb_file_system_inspector_t*);
extern C_ROCKSDB_LIBRARY_API size_t crocksdb_file_system_inspector_read(
    crocksdb_file_system_inspector_t* inspector, size_t len, Status* s);
extern C_ROCKSDB_LIBRARY_API size_t crocksdb_file_system_inspector_write(
    crocksdb_file_system_inspector_t* inspector, size_t len, Status* s);

extern C_ROCKSDB_LIBRARY_API Env* crocksdb_file_system_inspected_env_create(
    Env*, crocksdb_file_system_inspector_t*);

/* SstFile */

extern C_ROCKSDB_LIBRARY_API crocksdb_sstfilereader_t*
crocksdb_sstfilereader_create(const Options* io_options);

extern C_ROCKSDB_LIBRARY_API void crocksdb_sstfilereader_open(
    crocksdb_sstfilereader_t* reader, const char* name, Status* s);

extern C_ROCKSDB_LIBRARY_API crocksdb_iterator_t*
crocksdb_sstfilereader_new_iterator(crocksdb_sstfilereader_t* reader,
                                    const ReadOptions* options);

extern C_ROCKSDB_LIBRARY_API const TableProperties*
crocksdb_sstfilereader_get_table_properties(
    const crocksdb_sstfilereader_t* reader);

extern C_ROCKSDB_LIBRARY_API void crocksdb_sstfilereader_verify_checksum(
    crocksdb_sstfilereader_t* reader, Status* s);

extern C_ROCKSDB_LIBRARY_API void crocksdb_sstfilereader_destroy(
    crocksdb_sstfilereader_t* reader);

extern C_ROCKSDB_LIBRARY_API crocksdb_sstfilewriter_t*
crocksdb_sstfilewriter_create(const crocksdb_envoptions_t* env,
                              const Options* io_options);
extern C_ROCKSDB_LIBRARY_API crocksdb_sstfilewriter_t*
crocksdb_sstfilewriter_create_cf(
    const crocksdb_envoptions_t* env, const Options* io_options,
    crocksdb_column_family_handle_t* column_family);
extern C_ROCKSDB_LIBRARY_API void crocksdb_sstfilewriter_open(
    crocksdb_sstfilewriter_t* writer, const char* name, Status* s);
extern C_ROCKSDB_LIBRARY_API void crocksdb_sstfilewriter_put(
    crocksdb_sstfilewriter_t* writer, const char* key, size_t keylen,
    const char* val, size_t vallen, Status* s);
extern C_ROCKSDB_LIBRARY_API void crocksdb_sstfilewriter_merge(
    crocksdb_sstfilewriter_t* writer, const char* key, size_t keylen,
    const char* val, size_t vallen, Status* s);
extern C_ROCKSDB_LIBRARY_API void crocksdb_sstfilewriter_delete(
    crocksdb_sstfilewriter_t* writer, const char* key, size_t keylen,
    Status* s);
extern C_ROCKSDB_LIBRARY_API void crocksdb_sstfilewriter_delete_range(
    crocksdb_sstfilewriter_t* writer, const char* begin_key,
    size_t begin_keylen, const char* end_key, size_t end_keylen, Status* s);
extern C_ROCKSDB_LIBRARY_API void crocksdb_sstfilewriter_finish(
    crocksdb_sstfilewriter_t* writer, crocksdb_externalsstfileinfo_t* info,
    Status* s);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_sstfilewriter_file_size(crocksdb_sstfilewriter_t* writer);
extern C_ROCKSDB_LIBRARY_API void crocksdb_sstfilewriter_destroy(
    crocksdb_sstfilewriter_t* writer);

/* ExternalSstFileInfo */

extern C_ROCKSDB_LIBRARY_API crocksdb_externalsstfileinfo_t*
crocksdb_externalsstfileinfo_create();
extern C_ROCKSDB_LIBRARY_API void crocksdb_externalsstfileinfo_destroy(
    crocksdb_externalsstfileinfo_t*);
extern C_ROCKSDB_LIBRARY_API const char* crocksdb_externalsstfileinfo_file_path(
    crocksdb_externalsstfileinfo_t*, size_t*);
extern C_ROCKSDB_LIBRARY_API const char*
crocksdb_externalsstfileinfo_smallest_key(crocksdb_externalsstfileinfo_t*,
                                          size_t*);
extern C_ROCKSDB_LIBRARY_API const char*
crocksdb_externalsstfileinfo_largest_key(crocksdb_externalsstfileinfo_t*,
                                         size_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_externalsstfileinfo_sequence_number(crocksdb_externalsstfileinfo_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_externalsstfileinfo_file_size(crocksdb_externalsstfileinfo_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_externalsstfileinfo_num_entries(crocksdb_externalsstfileinfo_t*);

extern C_ROCKSDB_LIBRARY_API void crocksdb_ingestexternalfileoptions_init(
    IngestExternalFileOptions*);
extern C_ROCKSDB_LIBRARY_API void crocksdb_ingest_external_file(
    crocksdb_t* db, const char* const* file_list, const size_t list_len,
    const IngestExternalFileOptions* opt, Status* s);
extern C_ROCKSDB_LIBRARY_API void crocksdb_ingest_external_file_cf(
    crocksdb_t* db, crocksdb_column_family_handle_t* handle,
    const char* const* file_list, const size_t list_len,
    const IngestExternalFileOptions* opt, Status* s);
extern C_ROCKSDB_LIBRARY_API unsigned char
crocksdb_ingest_external_file_optimized(crocksdb_t* db,
                                        crocksdb_column_family_handle_t* handle,
                                        const char* const* file_list,
                                        const size_t list_len,
                                        const IngestExternalFileOptions* opt,
                                        Status* s);

/* SliceTransform */

extern C_ROCKSDB_LIBRARY_API crocksdb_slicetransform_t*
crocksdb_slicetransform_create(void* state, void (*destructor)(void*),
                               void (*transform)(void*, Slice, Slice* dst),
                               bool (*in_domain)(void*, Slice),
                               bool (*full_length_enabled)(void*, size_t*),
                               bool (*same_result_when_appended)(void*, Slice),
                               const char* (*name)(void*));
extern C_ROCKSDB_LIBRARY_API crocksdb_slicetransform_t*
    crocksdb_slicetransform_create_fixed_prefix(size_t);
extern C_ROCKSDB_LIBRARY_API crocksdb_slicetransform_t*
crocksdb_slicetransform_create_noop();
extern C_ROCKSDB_LIBRARY_API void crocksdb_slicetransform_destroy(
    crocksdb_slicetransform_t*);

/* Universal Compaction options */

extern C_ROCKSDB_LIBRARY_API void crocksdb_universal_compaction_options_init(
    CompactionOptionsUniversal*);

extern C_ROCKSDB_LIBRARY_API void crocksdb_fifo_compaction_options_init(
    CompactionOptionsFIFO*);

extern C_ROCKSDB_LIBRARY_API size_t
crocksdb_livefiles_count(const crocksdb_livefiles_t*);
extern C_ROCKSDB_LIBRARY_API const char* crocksdb_livefiles_name(
    const crocksdb_livefiles_t*, int index);
extern C_ROCKSDB_LIBRARY_API int crocksdb_livefiles_level(
    const crocksdb_livefiles_t*, int index);
extern C_ROCKSDB_LIBRARY_API size_t
crocksdb_livefiles_size(const crocksdb_livefiles_t*, int index);
extern C_ROCKSDB_LIBRARY_API const char* crocksdb_livefiles_smallestkey(
    const crocksdb_livefiles_t*, int index, size_t* size);
extern C_ROCKSDB_LIBRARY_API const char* crocksdb_livefiles_largestkey(
    const crocksdb_livefiles_t*, int index, size_t* size);
extern C_ROCKSDB_LIBRARY_API void crocksdb_livefiles_destroy(
    const crocksdb_livefiles_t*);

/* Utility Helpers */

extern C_ROCKSDB_LIBRARY_API void crocksdb_get_options_from_string(
    const Options* base_options, const char* opts_str, Options* new_options,
    Status* s);

extern C_ROCKSDB_LIBRARY_API void crocksdb_delete_files_in_range(
    crocksdb_t* db, const char* start_key, size_t start_key_len,
    const char* limit_key, size_t limit_key_len, unsigned char include_end,
    Status* s);

extern C_ROCKSDB_LIBRARY_API void crocksdb_delete_files_in_range_cf(
    crocksdb_t* db, crocksdb_column_family_handle_t* column_family,
    const char* start_key, size_t start_key_len, const char* limit_key,
    size_t limit_key_len, unsigned char include_end, Status* s);

extern C_ROCKSDB_LIBRARY_API void crocksdb_delete_files_in_ranges_cf(
    crocksdb_t* db, crocksdb_column_family_handle_t* cf,
    const char* const* start_keys, const size_t* start_keys_lens,
    const char* const* limit_keys, const size_t* limit_keys_lens,
    size_t num_ranges, unsigned char include_end, Status* s);

// referring to convention (3), this should be used by client
// to free memory that was malloc()ed
extern C_ROCKSDB_LIBRARY_API void crocksdb_free(void* ptr);

extern C_ROCKSDB_LIBRARY_API crocksdb_logger_t* crocksdb_create_env_logger(
    const char* fname, Env* env);
extern C_ROCKSDB_LIBRARY_API crocksdb_logger_t*
crocksdb_create_log_from_options(const char* path, const DBOptions* opts,
                                 Status* s);
extern C_ROCKSDB_LIBRARY_API void crocksdb_log_destroy(crocksdb_logger_t*);

extern C_ROCKSDB_LIBRARY_API crocksdb_pinnableslice_t* crocksdb_get_pinned(
    crocksdb_t* db, const ReadOptions* options, const char* key, size_t keylen,
    Status* s);
extern C_ROCKSDB_LIBRARY_API crocksdb_pinnableslice_t* crocksdb_get_pinned_cf(
    crocksdb_t* db, const ReadOptions* options,
    crocksdb_column_family_handle_t* column_family, const char* key,
    size_t keylen, Status* s);
extern C_ROCKSDB_LIBRARY_API void crocksdb_pinnableslice_destroy(
    crocksdb_pinnableslice_t* v);
extern C_ROCKSDB_LIBRARY_API const char* crocksdb_pinnableslice_value(
    const crocksdb_pinnableslice_t* t, size_t* vlen);

extern C_ROCKSDB_LIBRARY_API size_t crocksdb_get_supported_compression_number();
extern C_ROCKSDB_LIBRARY_API void crocksdb_get_supported_compression(
    CompressionType*, size_t);

/* Table Properties */

extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_table_properties_get_data_size(const TableProperties*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_table_properties_get_index_size(const TableProperties*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_table_properties_get_index_partitions(const TableProperties*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_table_properties_get_top_level_index_size(const TableProperties*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_table_properties_get_index_key_is_user_key(const TableProperties*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_table_properties_get_index_value_is_delta_encoded(
    const TableProperties*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_table_properties_get_filter_size(const TableProperties*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_table_properties_get_raw_key_size(const TableProperties*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_table_properties_get_raw_value_size(const TableProperties*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_table_properties_get_num_data_blocks(const TableProperties*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_table_properties_get_num_entries(const TableProperties*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_table_properties_get_num_deletions(const TableProperties*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_table_properties_get_num_merge_operands(const TableProperties*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_table_properties_get_num_range_deletions(const TableProperties*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_table_properties_get_format_version(const TableProperties*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_table_properties_get_fixed_key_len(const TableProperties*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_table_properties_get_column_family_id(const TableProperties*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_table_properties_get_creation_time(const TableProperties*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_table_properties_get_oldest_key_time(const TableProperties*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_table_properties_get_file_creation_time(const TableProperties*);

extern C_ROCKSDB_LIBRARY_API void
crocksdb_table_properties_get_column_family_name(const TableProperties*,
                                                 Slice* val);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_table_properties_get_filter_policy_name(const TableProperties*,
                                                 Slice* val);
extern C_ROCKSDB_LIBRARY_API void crocksdb_table_properties_get_comparator_name(
    const TableProperties*, Slice* val);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_table_properties_get_merge_operator_name(const TableProperties*,
                                                  Slice* val);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_table_properties_get_prefix_extractor_name(const TableProperties*,
                                                    Slice* val);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_table_properties_get_property_collectors_names(const TableProperties*,
                                                        Slice* val);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_table_properties_get_compression_name(const TableProperties*,
                                               Slice* val);
extern C_ROCKSDB_LIBRARY_API void
crocksdb_table_properties_get_compression_options(const TableProperties*,
                                                  Slice* val);

extern C_ROCKSDB_LIBRARY_API char* crocksdb_table_properties_to_string(
    const TableProperties*, size_t* len);

extern C_ROCKSDB_LIBRARY_API const UserCollectedProperties*
crocksdb_table_properties_get_user_properties(const TableProperties*);

extern C_ROCKSDB_LIBRARY_API bool crocksdb_user_collected_properties_get(
    const UserCollectedProperties* props, Slice zkey, Slice* value);

extern C_ROCKSDB_LIBRARY_API size_t
crocksdb_user_collected_properties_len(const UserCollectedProperties*);

extern C_ROCKSDB_LIBRARY_API void crocksdb_user_collected_properties_add(
    UserCollectedProperties*, Slice key, Slice value);

extern C_ROCKSDB_LIBRARY_API crocksdb_user_collected_properties_iterator_t*
crocksdb_user_collected_properties_iter_create(const UserCollectedProperties*);

extern C_ROCKSDB_LIBRARY_API void
crocksdb_user_collected_properties_iter_destroy(
    crocksdb_user_collected_properties_iterator_t*);

extern C_ROCKSDB_LIBRARY_API bool crocksdb_user_collected_properties_iter_next(
    crocksdb_user_collected_properties_iterator_t*, Slice* key, Slice* val);

/* Table Properties Collection */

extern C_ROCKSDB_LIBRARY_API const TableProperties*
crocksdb_table_properties_collection_get(const TablePropertiesCollection*,
                                         Slice);

extern C_ROCKSDB_LIBRARY_API size_t
crocksdb_table_properties_collection_len(const TablePropertiesCollection*);

extern C_ROCKSDB_LIBRARY_API void crocksdb_table_properties_collection_destroy(
    TablePropertiesCollection*);

extern C_ROCKSDB_LIBRARY_API crocksdb_table_properties_collection_iterator_t*
crocksdb_table_properties_collection_iter_create(
    const TablePropertiesCollection*);

extern C_ROCKSDB_LIBRARY_API void
crocksdb_table_properties_collection_iter_destroy(
    crocksdb_table_properties_collection_iterator_t*);

extern C_ROCKSDB_LIBRARY_API bool
crocksdb_table_properties_collection_iter_next(
    crocksdb_table_properties_collection_iterator_t*, Slice* key,
    const TableProperties** val);

/* Table Properties Collector */

extern C_ROCKSDB_LIBRARY_API crocksdb_table_properties_collector_t*
crocksdb_table_properties_collector_create(
    void* state, const char* (*name)(void*), void (*destruct)(void*),
    void (*add)(void*, Slice key, Slice value, EntryType entry_type,
                SequenceNumber seq, uint64_t file_size, Status*),
    void (*finish)(void*, UserCollectedProperties*, Status*));

extern C_ROCKSDB_LIBRARY_API void crocksdb_table_properties_collector_destroy(
    crocksdb_table_properties_collector_t*);

/* Table Properties Collector Factory */

extern C_ROCKSDB_LIBRARY_API crocksdb_table_properties_collector_factory_t*
crocksdb_table_properties_collector_factory_create(
    void* state, const char* (*name)(void*), void (*destruct)(void*),
    crocksdb_table_properties_collector_t* (*create_table_properties_collector)(
        void*, uint32_t column_family_id));

extern C_ROCKSDB_LIBRARY_API void
crocksdb_table_properties_collector_factory_destroy(
    crocksdb_table_properties_collector_factory_t*);

extern C_ROCKSDB_LIBRARY_API void
crocksdb_options_add_table_properties_collector_factory(
    ColumnFamilyOptions* opt, crocksdb_table_properties_collector_factory_t* f);

extern C_ROCKSDB_LIBRARY_API void crocksdb_options_set_compact_on_deletion(
    ColumnFamilyOptions* opt, size_t sliding_window_size,
    size_t deletion_trigger);

/* Get Table Properties */
extern C_ROCKSDB_LIBRARY_API TablePropertiesCollection*
crocksdb_get_properties_of_all_tables(crocksdb_t* db, Status* s);

extern C_ROCKSDB_LIBRARY_API TablePropertiesCollection*
crocksdb_get_properties_of_all_tables_cf(crocksdb_t* db,
                                         crocksdb_column_family_handle_t* cf,
                                         Status* s);

extern C_ROCKSDB_LIBRARY_API TablePropertiesCollection*
crocksdb_get_properties_of_tables_in_range(
    crocksdb_t* db, crocksdb_column_family_handle_t* cf, int num_ranges,
    const char* const* start_keys, const size_t* start_keys_lens,
    const char* const* limit_keys, const size_t* limit_keys_lens, Status* s);

/* Get All Key Versions */

extern C_ROCKSDB_LIBRARY_API void crocksdb_keyversions_destroy(
    crocksdb_keyversions_t* kvs);

extern C_ROCKSDB_LIBRARY_API crocksdb_keyversions_t*
crocksdb_get_all_key_versions(crocksdb_t* db, const char* begin_key,
                              size_t begin_keylen, const char* end_key,
                              size_t end_keylen, Status* s);

extern C_ROCKSDB_LIBRARY_API size_t
crocksdb_keyversions_count(const crocksdb_keyversions_t* kvs);

extern C_ROCKSDB_LIBRARY_API const char* crocksdb_keyversions_key(
    const crocksdb_keyversions_t* kvs, int index);

extern C_ROCKSDB_LIBRARY_API const char* crocksdb_keyversions_value(
    const crocksdb_keyversions_t* kvs, int index);

extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_keyversions_seq(const crocksdb_keyversions_t* kvs, int index);

extern C_ROCKSDB_LIBRARY_API int crocksdb_keyversions_type(
    const crocksdb_keyversions_t* kvs, int index);

/* Modify Sst File Seq No */
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_set_external_sst_file_global_seq_no(
    crocksdb_t* db, crocksdb_column_family_handle_t* column_family,
    const char* file, uint64_t seq_no, Status* s);

/* ColumnFamilyMetaData */
extern C_ROCKSDB_LIBRARY_API void crocksdb_get_column_family_meta_data(
    crocksdb_t* db, crocksdb_column_family_handle_t* cf,
    crocksdb_column_family_meta_data_t*);
extern C_ROCKSDB_LIBRARY_API crocksdb_column_family_meta_data_t*
crocksdb_column_family_meta_data_create();
extern C_ROCKSDB_LIBRARY_API void crocksdb_column_family_meta_data_destroy(
    crocksdb_column_family_meta_data_t*);
extern C_ROCKSDB_LIBRARY_API size_t
crocksdb_column_family_meta_data_level_count(
    const crocksdb_column_family_meta_data_t*);
extern C_ROCKSDB_LIBRARY_API const crocksdb_level_meta_data_t*
crocksdb_column_family_meta_data_level_data(
    const crocksdb_column_family_meta_data_t*, size_t n);
extern C_ROCKSDB_LIBRARY_API size_t
crocksdb_level_meta_data_file_count(const crocksdb_level_meta_data_t*);
extern C_ROCKSDB_LIBRARY_API const crocksdb_sst_file_meta_data_t*
crocksdb_level_meta_data_file_data(const crocksdb_level_meta_data_t*, size_t n);
extern C_ROCKSDB_LIBRARY_API size_t
crocksdb_sst_file_meta_data_size(const crocksdb_sst_file_meta_data_t*);
extern C_ROCKSDB_LIBRARY_API const char* crocksdb_sst_file_meta_data_name(
    const crocksdb_sst_file_meta_data_t*);
extern C_ROCKSDB_LIBRARY_API const char*
crocksdb_sst_file_meta_data_smallestkey(const crocksdb_sst_file_meta_data_t*,
                                        size_t*);
extern C_ROCKSDB_LIBRARY_API const char* crocksdb_sst_file_meta_data_largestkey(
    const crocksdb_sst_file_meta_data_t*, size_t*);

/* CompactFiles */
extern C_ROCKSDB_LIBRARY_API void crocksdb_compaction_options_init(
    CompactionOptions*);

extern C_ROCKSDB_LIBRARY_API void crocksdb_compact_files_cf(
    crocksdb_t*, crocksdb_column_family_handle_t*, const CompactionOptions*,
    const char** input_file_names, size_t input_file_count, int output_level,
    Status* s);

/* PerfContext */
extern C_ROCKSDB_LIBRARY_API PerfLevel crocksdb_get_perf_level(void);
extern C_ROCKSDB_LIBRARY_API void crocksdb_set_perf_level(PerfLevel level);
extern C_ROCKSDB_LIBRARY_API crocksdb_perf_context_t* crocksdb_get_perf_context(
    void);
extern C_ROCKSDB_LIBRARY_API void crocksdb_perf_context_reset(
    crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_user_key_comparison_count(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_block_cache_hit_count(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_block_read_count(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_block_read_byte(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_block_read_time(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_block_cache_index_hit_count(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_index_block_read_count(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_block_cache_filter_hit_count(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_filter_block_read_count(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_block_checksum_time(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_block_decompress_time(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_get_read_bytes(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_multiget_read_bytes(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_iter_read_bytes(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_internal_key_skipped_count(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_internal_delete_skipped_count(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_internal_recent_skipped_count(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_internal_merge_count(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_get_snapshot_time(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_get_from_memtable_time(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_get_from_memtable_count(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_get_post_process_time(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_get_from_output_files_time(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_seek_on_memtable_time(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_seek_on_memtable_count(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_next_on_memtable_count(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_prev_on_memtable_count(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_seek_child_seek_time(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_seek_child_seek_count(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_seek_min_heap_time(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_seek_max_heap_time(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_seek_internal_seek_time(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_find_next_user_entry_time(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_write_wal_time(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_write_memtable_time(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_write_delay_time(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_write_pre_and_post_process_time(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_db_mutex_lock_nanos(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_write_thread_wait_nanos(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_write_scheduling_flushes_compactions_time(
    crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_db_condition_wait_nanos(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_merge_operator_time_nanos(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_read_index_block_nanos(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_read_filter_block_nanos(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_new_table_block_iter_nanos(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_new_table_iterator_nanos(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_block_seek_nanos(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_find_table_nanos(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_bloom_memtable_hit_count(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_bloom_memtable_miss_count(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_bloom_sst_hit_count(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_bloom_sst_miss_count(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_env_new_sequential_file_nanos(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_env_new_random_access_file_nanos(
    crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_env_new_writable_file_nanos(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_env_reuse_writable_file_nanos(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_env_new_random_rw_file_nanos(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_env_new_directory_nanos(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_env_file_exists_nanos(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_env_get_children_nanos(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_env_get_children_file_attributes_nanos(
    crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_env_delete_file_nanos(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_env_create_dir_nanos(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_env_create_dir_if_missing_nanos(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_env_delete_dir_nanos(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_env_get_file_size_nanos(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_env_get_file_modification_time_nanos(
    crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_env_rename_file_nanos(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_env_link_file_nanos(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_env_lock_file_nanos(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_env_unlock_file_nanos(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_env_new_logger_nanos(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_get_cpu_nanos(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_iter_next_cpu_nanos(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_iter_prev_cpu_nanos(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_iter_seek_cpu_nanos(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_encrypt_data_nanos(crocksdb_perf_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_perf_context_decrypt_data_nanos(crocksdb_perf_context_t*);

// IOStatsContext
extern C_ROCKSDB_LIBRARY_API crocksdb_iostats_context_t*
crocksdb_get_iostats_context(void);
extern C_ROCKSDB_LIBRARY_API void crocksdb_iostats_context_reset(
    crocksdb_iostats_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_iostats_context_bytes_written(crocksdb_iostats_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_iostats_context_bytes_read(crocksdb_iostats_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_iostats_context_open_nanos(crocksdb_iostats_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_iostats_context_allocate_nanos(crocksdb_iostats_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_iostats_context_write_nanos(crocksdb_iostats_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_iostats_context_read_nanos(crocksdb_iostats_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_iostats_context_range_sync_nanos(crocksdb_iostats_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_iostats_context_fsync_nanos(crocksdb_iostats_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_iostats_context_prepare_write_nanos(crocksdb_iostats_context_t*);
extern C_ROCKSDB_LIBRARY_API uint64_t
crocksdb_iostats_context_logger_nanos(crocksdb_iostats_context_t*);

/* SstPartitioner */

typedef PartitionerResult (*crocksdb_sst_partitioner_should_partition_cb)(
    void* underlying, const PartitionerRequest* req);
typedef bool (*crocksdb_sst_partitioner_can_do_trivial_move_cb)(
    void* underlying, Slice smallest_user_key, Slice largest_user_key);

extern C_ROCKSDB_LIBRARY_API SstPartitioner* crocksdb_sst_partitioner_create(
    void* underlying, void (*destructor)(void*),
    crocksdb_sst_partitioner_should_partition_cb should_partition_cb,
    crocksdb_sst_partitioner_can_do_trivial_move_cb can_do_trivial_move_cb);

typedef const char* (*crocksdb_sst_partitioner_factory_name_cb)(
    void* underlying);
typedef SstPartitioner* (
    *crocksdb_sst_partitioner_factory_create_partitioner_cb)(
    void* underlying, const SstPartitioner::Context* context);

extern C_ROCKSDB_LIBRARY_API crocksdb_sst_partitioner_factory_t*
crocksdb_sst_partitioner_factory_create(
    void* underlying, void (*destructor)(void*),
    crocksdb_sst_partitioner_factory_name_cb name_cb,
    crocksdb_sst_partitioner_factory_create_partitioner_cb
        create_partitioner_cb);
extern C_ROCKSDB_LIBRARY_API void crocksdb_sst_partitioner_factory_destroy(
    crocksdb_sst_partitioner_factory_t* factory);

extern C_ROCKSDB_LIBRARY_API void crocksdb_run_ldb_tool(int argc, char** argv,
                                                        const Options* opts);
extern C_ROCKSDB_LIBRARY_API void crocksdb_run_sst_dump_tool(
    int argc, char** argv, const Options* opts);

/* Titan */
struct ctitandb_blob_index_t {
  uint64_t file_number;
  uint64_t blob_offset;
  uint64_t blob_size;
};

typedef struct ctitandb_blob_index_t ctitandb_blob_index_t;

extern C_ROCKSDB_LIBRARY_API crocksdb_t* ctitandb_open_column_families(
    const char* name, const TitanDBOptions* tdb_options,
    int num_column_families, const char** column_family_names,
    const TitanCFOptions** titan_column_family_options,
    crocksdb_column_family_handle_t** column_family_handles, Status* s);

extern C_ROCKSDB_LIBRARY_API crocksdb_column_family_handle_t*
ctitandb_create_column_family(crocksdb_t* db,
                              const TitanCFOptions* titan_column_family_options,
                              const char* column_family_name, Status* s);

/* TitanDBOptions */

extern C_ROCKSDB_LIBRARY_API TitanDBOptions* ctitandb_dboptions_create();
extern C_ROCKSDB_LIBRARY_API void ctitandb_dboptions_destroy(TitanDBOptions*);
extern C_ROCKSDB_LIBRARY_API TitanCFOptions* ctitandb_cfoptions_create();
extern C_ROCKSDB_LIBRARY_API void ctitandb_cfoptions_destroy(TitanCFOptions*);

extern C_ROCKSDB_LIBRARY_API TitanOptions* ctitandb_get_titan_options_cf(
    const crocksdb_t* db, crocksdb_column_family_handle_t* column_family);

extern C_ROCKSDB_LIBRARY_API TitanDBOptions* ctitandb_get_titan_db_options(
    crocksdb_t* db);

extern C_ROCKSDB_LIBRARY_API void ctitandb_options_get_dirname(
    const TitanDBOptions*, Slice* name);

extern C_ROCKSDB_LIBRARY_API void ctitandb_options_set_dirname(TitanDBOptions*,
                                                               Slice name);

extern C_ROCKSDB_LIBRARY_API uint64_t
ctitandb_options_get_min_blob_size(const TitanCFOptions*);

extern C_ROCKSDB_LIBRARY_API void ctitandb_options_set_min_blob_size(
    TitanCFOptions*, uint64_t size);

extern C_ROCKSDB_LIBRARY_API CompressionType
ctitandb_options_get_blob_file_compression(const TitanCFOptions*);

extern C_ROCKSDB_LIBRARY_API void ctitandb_options_set_gc_merge_rewrite(
    TitanCFOptions*, bool);

extern C_ROCKSDB_LIBRARY_API void ctitandb_options_set_blob_file_compression(
    TitanCFOptions*, CompressionType type);

extern C_ROCKSDB_LIBRARY_API CompressionOptions*
ctitandb_options_get_blob_file_compression_options(TitanCFOptions* opt);

extern C_ROCKSDB_LIBRARY_API void ctitandb_decode_blob_index(
    const char* value, size_t value_size, ctitandb_blob_index_t* index,
    Status* s);

extern C_ROCKSDB_LIBRARY_API void ctitandb_encode_blob_index(
    const ctitandb_blob_index_t* index, char** value, size_t* value_size);

extern C_ROCKSDB_LIBRARY_API void ctitandb_options_set_disable_background_gc(
    TitanDBOptions* options, bool disable);

extern C_ROCKSDB_LIBRARY_API void ctitandb_options_set_level_merge(
    TitanCFOptions* options, bool enable);

extern C_ROCKSDB_LIBRARY_API void ctitandb_options_set_range_merge(
    TitanCFOptions* options, bool enable);

extern C_ROCKSDB_LIBRARY_API void ctitandb_options_set_max_sorted_runs(
    TitanCFOptions* options, int size);

extern C_ROCKSDB_LIBRARY_API void ctitandb_options_set_max_gc_batch_size(
    TitanCFOptions* options, uint64_t size);

extern C_ROCKSDB_LIBRARY_API void ctitandb_options_set_min_gc_batch_size(
    TitanCFOptions* options, uint64_t size);

extern C_ROCKSDB_LIBRARY_API void
ctitandb_options_set_blob_file_discardable_ratio(TitanCFOptions* options,
                                                 double ratio);

extern C_ROCKSDB_LIBRARY_API void ctitandb_options_set_sample_file_size_ratio(
    TitanCFOptions* options, double ratio);

extern C_ROCKSDB_LIBRARY_API void
ctitandb_options_set_merge_small_file_threshold(TitanCFOptions* options,
                                                uint64_t size);

extern C_ROCKSDB_LIBRARY_API void ctitandb_options_set_max_background_gc(
    TitanDBOptions* options, int32_t size);

extern C_ROCKSDB_LIBRARY_API void
ctitandb_options_set_purge_obsolete_files_period_sec(TitanDBOptions* options,
                                                     uint32_t period);

extern C_ROCKSDB_LIBRARY_API void ctitandb_options_set_blob_cache(
    TitanCFOptions* options, crocksdb_cache_t* cache);

extern C_ROCKSDB_LIBRARY_API size_t
ctitandb_options_get_blob_cache_usage(const TitanCFOptions* opt);

extern C_ROCKSDB_LIBRARY_API void ctitandb_options_set_blob_cache_capacity(
    TitanCFOptions* opt, size_t capacity, Status* s);

extern C_ROCKSDB_LIBRARY_API size_t
ctitandb_options_get_blob_cache_capacity(const TitanCFOptions* opt);

extern C_ROCKSDB_LIBRARY_API void ctitandb_options_set_discardable_ratio(
    TitanCFOptions* options, double ratio);

extern void C_ROCKSDB_LIBRARY_API
ctitandb_options_set_sample_ratio(TitanCFOptions* options, double ratio);

extern void C_ROCKSDB_LIBRARY_API ctitandb_options_set_blob_run_mode(
    TitanCFOptions* options, TitanBlobRunMode mode);

/* TitanReadOptions */

extern C_ROCKSDB_LIBRARY_API void ctitandb_readoptions_init(TitanReadOptions*);

/* Titan Iterator */

extern C_ROCKSDB_LIBRARY_API crocksdb_iterator_t* ctitandb_create_iterator(
    crocksdb_t* db, const TitanReadOptions* titan_options);

extern C_ROCKSDB_LIBRARY_API crocksdb_iterator_t* ctitandb_create_iterator_cf(
    crocksdb_t* db, const TitanReadOptions* titan_options,
    crocksdb_column_family_handle_t* column_family);

extern C_ROCKSDB_LIBRARY_API void ctitandb_create_iterators(
    crocksdb_t* db, const TitanReadOptions* titan_options,
    crocksdb_column_family_handle_t** column_families,
    crocksdb_iterator_t** iterators, size_t size, Status* s);

extern C_ROCKSDB_LIBRARY_API void ctitandb_delete_files_in_range(
    crocksdb_t* db, const char* start_key, size_t start_key_len,
    const char* limit_key, size_t limit_key_len, unsigned char include_end,
    Status* s);

extern C_ROCKSDB_LIBRARY_API void ctitandb_delete_files_in_range_cf(
    crocksdb_t* db, crocksdb_column_family_handle_t* column_family,
    const char* start_key, size_t start_key_len, const char* limit_key,
    size_t limit_key_len, unsigned char include_end, Status* s);

extern C_ROCKSDB_LIBRARY_API void ctitandb_delete_files_in_ranges_cf(
    crocksdb_t* db, crocksdb_column_family_handle_t* cf,
    const char* const* start_keys, const size_t* start_keys_lens,
    const char* const* limit_keys, const size_t* limit_keys_lens,
    size_t num_ranges, unsigned char include_end, Status* s);

extern C_ROCKSDB_LIBRARY_API void ctitandb_delete_blob_files_in_range(
    crocksdb_t* db, const char* start_key, size_t start_key_len,
    const char* limit_key, size_t limit_key_len, unsigned char include_end,
    Status* s);

extern C_ROCKSDB_LIBRARY_API void ctitandb_delete_blob_files_in_range_cf(
    crocksdb_t* db, crocksdb_column_family_handle_t* column_family,
    const char* start_key, size_t start_key_len, const char* limit_key,
    size_t limit_key_len, unsigned char include_end, Status* s);

extern C_ROCKSDB_LIBRARY_API void ctitandb_delete_blob_files_in_ranges_cf(
    crocksdb_t* db, crocksdb_column_family_handle_t* cf,
    const char* const* start_keys, const size_t* start_keys_lens,
    const char* const* limit_keys, const size_t* limit_keys_lens,
    size_t num_ranges, unsigned char include_end, Status* s);

extern C_ROCKSDB_LIBRARY_API void crocksdb_free_cplus_array(const char* arr);
extern C_ROCKSDB_LIBRARY_API const char* crocksdb_to_cplus_array(Slice s);

} /* end extern "C" */

#endif /* C_ROCKSDB_INCLUDE_CWRAPPER_H_ */
