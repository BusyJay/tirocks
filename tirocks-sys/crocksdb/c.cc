//  Copyright (c) 2011-present, Facebook, Inc.  All rights reserved.
//  This source code is licensed under the BSD-style license found in the
//  LICENSE file in the root directory of this source tree. An additional grant
//  of patent rights can be found in the PATENTS file in the same directory.
//
// Copyright (c) 2011 The LevelDB Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file. See the AUTHORS file for names of contributors.

#include "crocksdb/c.h"

#include <stdlib.h>

#include <limits>

#include "db/column_family.h"
#include "rocksdb/cache.h"
#include "rocksdb/compaction_filter.h"
#include "rocksdb/comparator.h"
#include "rocksdb/convenience.h"
#include "rocksdb/db.h"
#include "rocksdb/encryption.h"
#include "rocksdb/env.h"
#include "rocksdb/env_encryption.h"
#include "rocksdb/env_inspected.h"
#include "rocksdb/filter_policy.h"
#include "rocksdb/iostats_context.h"
#include "rocksdb/iterator.h"
#include "rocksdb/ldb_tool.h"
#include "rocksdb/listener.h"
#include "rocksdb/memtablerep.h"
#include "rocksdb/merge_operator.h"
#include "rocksdb/options.h"
#include "rocksdb/perf_context.h"
#include "rocksdb/rate_limiter.h"
#include "rocksdb/slice_transform.h"
#include "rocksdb/sst_dump_tool.h"
#include "rocksdb/sst_file_reader.h"
#include "rocksdb/sst_partitioner.h"
#include "rocksdb/statistics.h"
#include "rocksdb/status.h"
#include "rocksdb/table.h"
#include "rocksdb/table_properties.h"
#include "rocksdb/types.h"
#include "rocksdb/universal_compaction.h"
#include "rocksdb/utilities/backupable_db.h"
#include "rocksdb/utilities/db_ttl.h"
#include "rocksdb/utilities/debug.h"
#include "rocksdb/utilities/options_util.h"
#include "rocksdb/utilities/table_properties_collectors.h"
#include "rocksdb/write_batch.h"
#include "src/blob_format.h"
#include "table/block_based/block_based_table_factory.h"
#include "table/sst_file_writer_collectors.h"
#include "table/table_reader.h"
#include "titan/db.h"
#include "titan/options.h"
#include "util/coding.h"
#include "util/file_reader_writer.h"

#if !defined(ROCKSDB_MAJOR) || !defined(ROCKSDB_MINOR) || \
    !defined(ROCKSDB_PATCH)
#error Only rocksdb 5.7.3+ is supported.
#endif

#if ROCKSDB_MAJOR * 10000 + ROCKSDB_MINOR * 100 + ROCKSDB_PATCH < 50703
#error Only rocksdb 5.7.3+ is supported.
#endif

using rocksdb::BackgroundErrorReason;
using rocksdb::BackupableDBOptions;
using rocksdb::BackupEngine;
using rocksdb::BackupInfo;
using rocksdb::BlockBasedTableOptions;
using rocksdb::BlockCipher;
using rocksdb::Cache;
using rocksdb::ColumnFamilyDescriptor;
using rocksdb::ColumnFamilyHandle;
using rocksdb::ColumnFamilyOptions;
using rocksdb::CompactionFilter;
using rocksdb::CompactionFilterFactory;
using rocksdb::CompactionJobInfo;
using rocksdb::CompactionOptionsFIFO;
using rocksdb::CompactionPri;
using rocksdb::CompactionStyle;
using rocksdb::CompactRangeOptions;
using rocksdb::Comparator;
using rocksdb::CompressionType;
using rocksdb::CTREncryptionProvider;
using rocksdb::CuckooTableOptions;
using rocksdb::DB;
using rocksdb::DBOptions;
using rocksdb::DbPath;
using rocksdb::DBWithTTL;
using rocksdb::EncryptionProvider;
using rocksdb::EntryType;
using rocksdb::Env;
using rocksdb::EnvOptions;
using rocksdb::EventListener;
using rocksdb::ExternalFileIngestionInfo;
using rocksdb::ExternalSstFileInfo;
using rocksdb::FileLock;
using rocksdb::FilterBitsBuilder;
using rocksdb::FilterBitsReader;
using rocksdb::FilterPolicy;
using rocksdb::FlushJobInfo;
using rocksdb::FlushOptions;
using rocksdb::HistogramData;
using rocksdb::InfoLogLevel;
using rocksdb::IngestExternalFileOptions;
using rocksdb::Iterator;
using rocksdb::KeyVersion;
using rocksdb::LiveFileMetaData;
using rocksdb::Logger;
using rocksdb::LRUCacheOptions;
using rocksdb::MergeOperator;
using rocksdb::NewBloomFilterPolicy;
using rocksdb::NewEncryptedEnv;
using rocksdb::NewGenericRateLimiter;
using rocksdb::NewLRUCache;
using rocksdb::Options;
using rocksdb::PartitionerRequest;
using rocksdb::PartitionerResult;
using rocksdb::PinnableSlice;
using rocksdb::RandomAccessFile;
using rocksdb::Range;
using rocksdb::RangePtr;
using rocksdb::RateLimiter;
using rocksdb::ReadOptions;
using rocksdb::ReadTier;
using rocksdb::RestoreOptions;
using rocksdb::SequenceNumber;
using rocksdb::SequentialFile;
using rocksdb::Slice;
using rocksdb::SliceParts;
using rocksdb::SliceTransform;
using rocksdb::Snapshot;
using rocksdb::SstFileReader;
using rocksdb::SstFileWriter;
using rocksdb::SstPartitioner;
using rocksdb::SstPartitionerFactory;
using rocksdb::Status;
using rocksdb::SubcompactionJobInfo;
using rocksdb::TableFileCreationReason;
using rocksdb::TableProperties;
using rocksdb::TablePropertiesCollection;
using rocksdb::TablePropertiesCollector;
using rocksdb::TablePropertiesCollectorFactory;
using rocksdb::UserCollectedProperties;
using rocksdb::WALRecoveryMode;
using rocksdb::WritableFile;
using rocksdb::WriteBatch;
using rocksdb::WriteOptions;
using rocksdb::WriteStallCondition;
using rocksdb::WriteStallInfo;

using rocksdb::BlockBasedTableFactory;
using rocksdb::BottommostLevelCompaction;
using rocksdb::ColumnFamilyData;
using rocksdb::ColumnFamilyHandleImpl;
using rocksdb::ColumnFamilyMetaData;
using rocksdb::CompactionOptions;
using rocksdb::CompactionReason;
using rocksdb::DecodeFixed32;
using rocksdb::DecodeFixed64;
using rocksdb::ExternalSstFilePropertyNames;
using rocksdb::IOStatsContext;
using rocksdb::LDBTool;
using rocksdb::LevelMetaData;
using rocksdb::PerfContext;
using rocksdb::PerfLevel;
using rocksdb::PutFixed64;
using rocksdb::RandomAccessFile;
using rocksdb::RandomAccessFileReader;
using rocksdb::RandomRWFile;
using rocksdb::SSTDumpTool;
using rocksdb::SstFileMetaData;
using rocksdb::TableReader;
using rocksdb::TableReaderOptions;
using rocksdb::VectorRepFactory;

using rocksdb::kMaxSequenceNumber;

using rocksdb::titandb::BlobIndex;
using rocksdb::titandb::TitanBlobRunMode;
using rocksdb::titandb::TitanCFDescriptor;
using rocksdb::titandb::TitanCFOptions;
using rocksdb::titandb::TitanDB;
using rocksdb::titandb::TitanDBOptions;
using rocksdb::titandb::TitanOptions;
using rocksdb::titandb::TitanReadOptions;

using rocksdb::MemoryAllocator;

#ifdef OPENSSL
using rocksdb::encryption::EncryptionMethod;
using rocksdb::encryption::FileEncryptionInfo;
using rocksdb::encryption::KeyManager;
using rocksdb::encryption::NewKeyManagedEncryptedEnv;
#endif

using rocksdb::FileSystemInspector;
using rocksdb::NewFileSystemInspectedEnv;
using std::shared_ptr;

extern "C" {

const char* block_base_table_str = "BlockBasedTable";

struct crocksdb_t {
  DB* rep;
};
struct crocksdb_backup_engine_t {
  BackupEngine* rep;
};
struct crocksdb_backup_engine_info_t {
  std::vector<BackupInfo> rep;
};
struct crocksdb_restore_options_t {
  RestoreOptions rep;
};
struct crocksdb_iterator_t {
  Iterator* rep;
};
struct crocksdb_writebatch_t {
  WriteBatch rep;
};
struct crocksdb_snapshot_t {
  const Snapshot* rep;
};
struct crocksdb_fifo_compaction_options_t {
  CompactionOptionsFIFO rep;
};
struct crocksdb_options_t {
  Options rep;
};
struct crocksdb_column_family_descriptor {
  ColumnFamilyDescriptor rep;
};
struct crocksdb_block_based_table_options_t {
  BlockBasedTableOptions rep;
};
struct crocksdb_cuckoo_table_options_t {
  CuckooTableOptions rep;
};
struct crocksdb_seqfile_t {
  SequentialFile* rep;
};
struct crocksdb_randomfile_t {
  RandomAccessFile* rep;
};
struct crocksdb_writablefile_t {
  WritableFile* rep;
};
struct crocksdb_filelock_t {
  FileLock* rep;
};
struct crocksdb_logger_t {
  shared_ptr<Logger> rep;
};
struct crocksdb_logger_impl_t : public Logger {
  void* rep;

  void (*destructor_)(void*);
  void (*logv_internal_)(void* logger, InfoLogLevel log_level, Slice msg);

  char* buffer_ = new char[500];

  void log_help_(void* logger, InfoLogLevel log_level, const char* format,
                 va_list ap) {
    // TODO: check level before constructing strings.
    va_list ap_copy;
    va_copy(ap_copy, ap);
    int num = vsnprintf(buffer_, 500, format, ap_copy);
    va_end(ap_copy);
    if (num < 500) {
      logv_internal_(rep, log_level, Slice(buffer_, num));
    } else {
      char* large_buffer = new char[num + 1];
      vsnprintf(large_buffer, static_cast<size_t>(num) + 1, format, ap);
      logv_internal_(rep, log_level, Slice(large_buffer, num));
      delete[] large_buffer;
    }
  }

  void Logv(const char* format, va_list ap) override {
    log_help_(rep, InfoLogLevel::HEADER_LEVEL, format, ap);
  }

  void Logv(const InfoLogLevel log_level, const char* format,
            va_list ap) override {
    log_help_(rep, log_level, format, ap);
  }

  virtual ~crocksdb_logger_impl_t() {
    (*destructor_)(rep);
    delete[] buffer_;
  }
};
struct crocksdb_lru_cache_options_t {
  LRUCacheOptions rep;
};
struct crocksdb_cache_t {
  shared_ptr<Cache> rep;
};
struct crocksdb_statistics_t {
  std::shared_ptr<Statistics> statistics;
};
struct crocksdb_memory_allocator_t {
  shared_ptr<MemoryAllocator> rep;
};
struct crocksdb_livefiles_t {
  std::vector<LiveFileMetaData> rep;
};
struct crocksdb_column_family_handle_t {
  ColumnFamilyHandle* rep;
};
struct crocksdb_envoptions_t {
  EnvOptions rep;
};
struct crocksdb_sequential_file_t {
  SequentialFile* rep;
};
struct crocksdb_sstfilereader_t {
  SstFileReader* rep;
};
struct crocksdb_sstfilewriter_t {
  SstFileWriter* rep;
};
struct crocksdb_externalsstfileinfo_t {
  ExternalSstFileInfo rep;
};
struct crocksdb_ratelimiter_t {
  std::shared_ptr<RateLimiter> rep;
};
struct crocksdb_histogramdata_t {
  HistogramData rep;
};
struct crocksdb_pinnableslice_t {
  PinnableSlice rep;
};

struct crocksdb_keyversions_t {
  std::vector<KeyVersion> rep;
};

struct crocksdb_compactionfiltercontext_t {
  CompactionFilter::Context rep;
};

struct crocksdb_column_family_meta_data_t {
  ColumnFamilyMetaData rep;
};
struct crocksdb_level_meta_data_t {
  LevelMetaData rep;
};
struct crocksdb_sst_file_meta_data_t {
  SstFileMetaData rep;
};

const Slice crocksdb_property_name_num_files_at_level_prefix =
    DB::Properties::kNumFilesAtLevelPrefix;
const Slice crocksdb_property_name_compression_ratio_at_level_prefix =
    DB::Properties::kCompressionRatioAtLevelPrefix;
const Slice crocksdb_property_name_stats = DB::Properties::kStats;
const Slice crocksdb_property_name_ss_tables = DB::Properties::kSSTables;
const Slice crocksdb_property_name_cf_stats = DB::Properties::kCFStats;
const Slice crocksdb_property_name_cf_stats_no_file_histogram =
    DB::Properties::kCFStatsNoFileHistogram;
const Slice crocksdb_property_name_cf_file_histogram =
    DB::Properties::kCFFileHistogram;
const Slice crocksdb_property_name_db_stats = DB::Properties::kDBStats;
const Slice crocksdb_property_name_level_stats = DB::Properties::kLevelStats;
const Slice crocksdb_property_name_num_immutable_mem_table =
    DB::Properties::kNumImmutableMemTable;
const Slice crocksdb_property_name_num_immutable_mem_table_flushed =
    DB::Properties::kNumImmutableMemTableFlushed;
const Slice crocksdb_property_name_mem_table_flush_pending =
    DB::Properties::kMemTableFlushPending;
const Slice crocksdb_property_name_num_running_flushes =
    DB::Properties::kNumRunningFlushes;
const Slice crocksdb_property_name_compaction_pending =
    DB::Properties::kCompactionPending;
const Slice crocksdb_property_name_num_running_compactions =
    DB::Properties::kNumRunningCompactions;
const Slice crocksdb_property_name_background_errors =
    DB::Properties::kBackgroundErrors;
const Slice crocksdb_property_name_cur_size_active_mem_table =
    DB::Properties::kCurSizeActiveMemTable;
const Slice crocksdb_property_name_cur_size_all_mem_tables =
    DB::Properties::kCurSizeAllMemTables;
const Slice crocksdb_property_name_size_all_mem_tables =
    DB::Properties::kSizeAllMemTables;
const Slice crocksdb_property_name_num_entries_active_mem_table =
    DB::Properties::kNumEntriesActiveMemTable;
const Slice crocksdb_property_name_num_entries_imm_mem_tables =
    DB::Properties::kNumEntriesImmMemTables;
const Slice crocksdb_property_name_num_deletes_active_mem_table =
    DB::Properties::kNumDeletesActiveMemTable;
const Slice crocksdb_property_name_num_deletes_imm_mem_tables =
    DB::Properties::kNumDeletesImmMemTables;
const Slice crocksdb_property_name_estimate_num_keys =
    DB::Properties::kEstimateNumKeys;
const Slice crocksdb_property_name_estimate_table_readers_mem =
    DB::Properties::kEstimateTableReadersMem;
const Slice crocksdb_property_name_is_file_deletions_enabled =
    DB::Properties::kIsFileDeletionsEnabled;
const Slice crocksdb_property_name_num_snapshots =
    DB::Properties::kNumSnapshots;
const Slice crocksdb_property_name_oldest_snapshot_time =
    DB::Properties::kOldestSnapshotTime;
const Slice crocksdb_property_name_oldest_snapshot_sequence =
    DB::Properties::kOldestSnapshotSequence;
const Slice crocksdb_property_name_num_live_versions =
    DB::Properties::kNumLiveVersions;
const Slice crocksdb_property_name_current_super_version_number =
    DB::Properties::kCurrentSuperVersionNumber;
const Slice crocksdb_property_name_estimate_live_data_size =
    DB::Properties::kEstimateLiveDataSize;
const Slice crocksdb_property_name_min_log_number_to_keep =
    DB::Properties::kMinLogNumberToKeep;
const Slice crocksdb_property_name_min_obsolete_sst_number_to_keep =
    DB::Properties::kMinObsoleteSstNumberToKeep;
const Slice crocksdb_property_name_total_sst_files_size =
    DB::Properties::kTotalSstFilesSize;
const Slice crocksdb_property_name_live_sst_files_size =
    DB::Properties::kLiveSstFilesSize;
const Slice crocksdb_property_name_base_level = DB::Properties::kBaseLevel;
const Slice crocksdb_property_name_estimate_pending_compaction_bytes =
    DB::Properties::kEstimatePendingCompactionBytes;
const Slice crocksdb_property_name_aggregated_table_properties =
    DB::Properties::kAggregatedTableProperties;
const Slice crocksdb_property_name_aggregated_table_properties_at_level =
    DB::Properties::kAggregatedTablePropertiesAtLevel;
const Slice crocksdb_property_name_actual_delayed_write_rate =
    DB::Properties::kActualDelayedWriteRate;
const Slice crocksdb_property_name_is_write_stopped =
    DB::Properties::kIsWriteStopped;
const Slice crocksdb_property_name_is_write_stalled =
    DB::Properties::kIsWriteStalled;
const Slice crocksdb_property_name_estimate_oldest_key_time =
    DB::Properties::kEstimateOldestKeyTime;
const Slice crocksdb_property_name_block_cache_capacity =
    DB::Properties::kBlockCacheCapacity;
const Slice crocksdb_property_name_block_cache_usage =
    DB::Properties::kBlockCacheUsage;
const Slice crocksdb_property_name_block_cache_pinned_usage =
    DB::Properties::kBlockCachePinnedUsage;
const Slice crocksdb_property_name_options_statistics =
    DB::Properties::kOptionsStatistics;

struct crocksdb_map_property_t {
  std::map<std::string, std::string> rep;
};

struct crocksdb_compactionfilter_t : public CompactionFilter {
  void* state_;
  void (*destructor_)(void*);
  Decision (*filter_)(void*, int level, const char* key, size_t key_length,
                      uint64_t seqno, ValueType value_type,
                      const char* existing_value, size_t value_length,
                      char** new_value, size_t* new_value_length,
                      char** skip_until, size_t* skip_until_length);

  const char* (*name_)(void*);

  virtual ~crocksdb_compactionfilter_t() { (*destructor_)(state_); }

  virtual Decision FilterV3(int level, const Slice& key, uint64_t seqno,
                            ValueType value_type, const Slice& existing_value,
                            std::string* new_value,
                            std::string* skip_until) const override {
    char* c_new_value = nullptr;
    char* c_skip_until = nullptr;
    size_t new_value_length, skip_until_length = 0;

    Decision result =
        (*filter_)(state_, level, key.data(), key.size(), seqno, value_type,
                   existing_value.data(), existing_value.size(), &c_new_value,
                   &new_value_length, &c_skip_until, &skip_until_length);
    if (result == Decision::kChangeValue) {
      new_value->assign(c_new_value, new_value_length);
      free(c_new_value);
    } else if (result == Decision::kRemoveAndSkipUntil) {
      skip_until->assign(c_skip_until, skip_until_length);
      free(c_skip_until);
    }
    return result;
  }

  virtual const char* Name() const override { return (*name_)(state_); }
};

struct crocksdb_compactionfilterfactory_t : public CompactionFilterFactory {
  void* state_;
  void (*destructor_)(void*);
  crocksdb_compactionfilter_t* (*create_compaction_filter_)(
      void*, crocksdb_compactionfiltercontext_t* context);
  unsigned char (*should_filter_table_file_creation_)(
      void*, TableFileCreationReason reason);
  const char* (*name_)(void*);

  virtual ~crocksdb_compactionfilterfactory_t() { (*destructor_)(state_); }

  virtual std::unique_ptr<CompactionFilter> CreateCompactionFilter(
      const CompactionFilter::Context& context) override {
    crocksdb_compactionfiltercontext_t ccontext;
    ccontext.rep = context;
    CompactionFilter* cf = (*create_compaction_filter_)(state_, &ccontext);
    return std::unique_ptr<CompactionFilter>(cf);
  }

  virtual bool ShouldFilterTableFileCreation(
      TableFileCreationReason reason) const override {
    return (*should_filter_table_file_creation_)(state_, reason);
  }

  virtual const char* Name() const override { return (*name_)(state_); }
};

struct crocksdb_comparator_t : public Comparator {
  void* state_;
  void (*destructor_)(void*);
  int (*compare_)(void*, const char* a, size_t alen, const char* b,
                  size_t blen);
  const char* (*name_)(void*);

  virtual ~crocksdb_comparator_t() { (*destructor_)(state_); }

  virtual int Compare(const Slice& a, const Slice& b) const override {
    return (*compare_)(state_, a.data(), a.size(), b.data(), b.size());
  }

  virtual const char* Name() const override { return (*name_)(state_); }

  // No-ops since the C binding does not support key shortening methods.
  virtual void FindShortestSeparator(std::string*,
                                     const Slice&) const override {}
  virtual void FindShortSuccessor(std::string*) const override {}
};

struct crocksdb_filterpolicy_t : public FilterPolicy {
  void* state_;
  void (*destructor_)(void*);
  const char* (*name_)(void*);
  char* (*create_)(void*, const char* const* key_array,
                   const size_t* key_length_array, int num_keys,
                   size_t* filter_length);
  unsigned char (*key_match_)(void*, const char* key, size_t length,
                              const char* filter, size_t filter_length);
  void (*delete_filter_)(void*, const char* filter, size_t filter_length);

  virtual ~crocksdb_filterpolicy_t() { (*destructor_)(state_); }

  virtual const char* Name() const override { return (*name_)(state_); }

  virtual void CreateFilter(const Slice* keys, int n,
                            std::string* dst) const override {
    std::vector<const char*> key_pointers(n);
    std::vector<size_t> key_sizes(n);
    for (int i = 0; i < n; i++) {
      key_pointers[i] = keys[i].data();
      key_sizes[i] = keys[i].size();
    }
    size_t len;
    char* filter = (*create_)(state_, &key_pointers[0], &key_sizes[0], n, &len);
    dst->append(filter, len);

    if (delete_filter_ != nullptr) {
      (*delete_filter_)(state_, filter, len);
    } else {
      free(filter);
    }
  }

  virtual bool KeyMayMatch(const Slice& key,
                           const Slice& filter) const override {
    return (*key_match_)(state_, key.data(), key.size(), filter.data(),
                         filter.size());
  }
};

struct crocksdb_mergeoperator_t : public MergeOperator {
  void* state_;
  void (*destructor_)(void*);
  const char* (*name_)(void*);
  char* (*full_merge_)(void*, const char* key, size_t key_length,
                       const char* existing_value, size_t existing_value_length,
                       const char* const* operands_list,
                       const size_t* operands_list_length, int num_operands,
                       unsigned char* success, size_t* new_value_length);
  char* (*partial_merge_)(void*, const char* key, size_t key_length,
                          const char* const* operands_list,
                          const size_t* operands_list_length, int num_operands,
                          unsigned char* success, size_t* new_value_length);
  void (*delete_value_)(void*, const char* value, size_t value_length);

  virtual ~crocksdb_mergeoperator_t() { (*destructor_)(state_); }

  virtual const char* Name() const override { return (*name_)(state_); }

  virtual bool FullMergeV2(const MergeOperationInput& merge_in,
                           MergeOperationOutput* merge_out) const override {
    size_t n = merge_in.operand_list.size();
    std::vector<const char*> operand_pointers(n);
    std::vector<size_t> operand_sizes(n);
    for (size_t i = 0; i < n; i++) {
      Slice operand(merge_in.operand_list[i]);
      operand_pointers[i] = operand.data();
      operand_sizes[i] = operand.size();
    }

    const char* existing_value_data = nullptr;
    size_t existing_value_len = 0;
    if (merge_in.existing_value != nullptr) {
      existing_value_data = merge_in.existing_value->data();
      existing_value_len = merge_in.existing_value->size();
    }

    unsigned char success;
    size_t new_value_len;
    char* tmp_new_value = (*full_merge_)(
        state_, merge_in.key.data(), merge_in.key.size(), existing_value_data,
        existing_value_len, &operand_pointers[0], &operand_sizes[0],
        static_cast<int>(n), &success, &new_value_len);
    merge_out->new_value.assign(tmp_new_value, new_value_len);

    if (delete_value_ != nullptr) {
      (*delete_value_)(state_, tmp_new_value, new_value_len);
    } else {
      free(tmp_new_value);
    }

    return success;
  }

  virtual bool PartialMergeMulti(const Slice& key,
                                 const std::deque<Slice>& operand_list,
                                 std::string* new_value,
                                 Logger*) const override {
    size_t operand_count = operand_list.size();
    std::vector<const char*> operand_pointers(operand_count);
    std::vector<size_t> operand_sizes(operand_count);
    for (size_t i = 0; i < operand_count; ++i) {
      Slice operand(operand_list[i]);
      operand_pointers[i] = operand.data();
      operand_sizes[i] = operand.size();
    }

    unsigned char success;
    size_t new_value_len;
    char* tmp_new_value = (*partial_merge_)(
        state_, key.data(), key.size(), &operand_pointers[0], &operand_sizes[0],
        static_cast<int>(operand_count), &success, &new_value_len);
    new_value->assign(tmp_new_value, new_value_len);

    if (delete_value_ != nullptr) {
      (*delete_value_)(state_, tmp_new_value, new_value_len);
    } else {
      free(tmp_new_value);
    }

    return success;
  }
};

/// An Env that free resources when being deleted.
struct EncryptedEnvWrapper : EnvWrapper {
  EncryptedEnvWrapper(Env* env, EncryptionProvider* provider,
                      BlockCipher* block_cipher)
      : EnvWrapper(env),
        encryption_provider_(provider),
        block_cipher_(block_cipher) {}

  ~EncryptedEnvWrapper() {
    delete encryption_provider_;
    delete block_cipher_;
  }

  EncryptionProvider* encryption_provider_;
  BlockCipher* block_cipher_;
};

struct crocksdb_slicetransform_t : public SliceTransform {
  void* state_;
  void (*destructor_)(void*);
  const char* (*name_)(void*);
  char* (*transform_)(void*, const char* key, size_t length,
                      size_t* dst_length);
  unsigned char (*in_domain_)(void*, const char* key, size_t length);
  unsigned char (*in_range_)(void*, const char* key, size_t length);

  virtual ~crocksdb_slicetransform_t() { (*destructor_)(state_); }

  virtual const char* Name() const override { return (*name_)(state_); }

  virtual Slice Transform(const Slice& src) const override {
    size_t len;
    char* dst = (*transform_)(state_, src.data(), src.size(), &len);
    return Slice(dst, len);
  }

  virtual bool InDomain(const Slice& src) const override {
    return (*in_domain_)(state_, src.data(), src.size());
  }

  virtual bool InRange(const Slice& src) const override {
    return (*in_range_)(state_, src.data(), src.size());
  }
};

struct crocksdb_universal_compaction_options_t {
  rocksdb::CompactionOptionsUniversal* rep;
};

struct crocksdb_writebatch_iterator_t {
  rocksdb::WriteBatch::Iterator* rep;
};

#ifdef OPENSSL
struct crocksdb_file_encryption_info_t {
  FileEncryptionInfo* rep;
};

struct crocksdb_encryption_key_manager_t {
  std::shared_ptr<KeyManager> rep;
};
#endif

struct crocksdb_sst_partitioner_t {
  std::unique_ptr<SstPartitioner> rep;
};

struct crocksdb_sst_partitioner_request_t {
  PartitionerRequest* rep;
  Slice prev_user_key;
  Slice current_user_key;
};

struct crocksdb_sst_partitioner_context_t {
  SstPartitioner::Context* rep;
};

struct crocksdb_sst_partitioner_factory_t {
  std::shared_ptr<SstPartitionerFactory> rep;
};

struct crocksdb_file_system_inspector_t {
  std::shared_ptr<FileSystemInspector> rep;
};

static char* CopyString(const std::string& str) {
  char* result = reinterpret_cast<char*>(malloc(sizeof(char) * str.size()));
  memcpy(result, str.data(), sizeof(char) * str.size());
  return result;
}

crocksdb_t* crocksdb_open(const crocksdb_options_t* options, Slice name,
                          Status* s) {
  DB* db;
  *s = DB::Open(options->rep, name.ToString(), &db);
  if (!s->ok()) {
    return nullptr;
  }
  crocksdb_t* result = new crocksdb_t;
  result->rep = db;
  return result;
}

crocksdb_t* crocksdb_open_with_ttl(const crocksdb_options_t* options,
                                   Slice name, int ttl, Status* s) {
  DBWithTTL* db;
  *s = DBWithTTL::Open(options->rep, name.ToString(), &db, ttl);
  if (!s->ok()) {
    return nullptr;
  }
  crocksdb_t* result = new crocksdb_t;
  result->rep = db;
  return result;
}

crocksdb_t* crocksdb_open_for_read_only(const crocksdb_options_t* options,
                                        Slice name,
                                        unsigned char error_if_log_file_exist,
                                        Status* s) {
  DB* db;
  *s = DB::OpenForReadOnly(options->rep, name.ToString(), &db,
                           error_if_log_file_exist);
  if (!s->ok()) {
    return nullptr;
  }
  crocksdb_t* result = new crocksdb_t;
  result->rep = db;
  return result;
}

void crocksdb_resume(crocksdb_t* db, Status* s) { *s = db->rep->Resume(); }

crocksdb_backup_engine_t* crocksdb_backup_engine_open(
    const crocksdb_options_t* options, Slice path, Status* s) {
  BackupEngine* be;
  *s = BackupEngine::Open(options->rep.env,
                          BackupableDBOptions(path.ToString()), &be);
  if (!s->ok()) {
    return nullptr;
  }
  crocksdb_backup_engine_t* result = new crocksdb_backup_engine_t;
  result->rep = be;
  return result;
}

void crocksdb_backup_engine_create_new_backup(crocksdb_backup_engine_t* be,
                                              crocksdb_t* db, Status* s) {
  *s = be->rep->CreateNewBackup(db->rep);
}

void crocksdb_backup_engine_purge_old_backups(crocksdb_backup_engine_t* be,
                                              uint32_t num_backups_to_keep,
                                              Status* s) {
  *s = be->rep->PurgeOldBackups(num_backups_to_keep);
}

crocksdb_restore_options_t* crocksdb_restore_options_create() {
  return new crocksdb_restore_options_t;
}

void crocksdb_restore_options_destroy(crocksdb_restore_options_t* opt) {
  delete opt;
}

void crocksdb_restore_options_set_keep_log_files(
    crocksdb_restore_options_t* opt, int v) {
  opt->rep.keep_log_files = v;
}

void crocksdb_backup_engine_restore_db_from_latest_backup(
    crocksdb_backup_engine_t* be, Slice db_dir, Slice wal_dir,
    const crocksdb_restore_options_t* restore_options, Status* s) {
  *s = be->rep->RestoreDBFromLatestBackup(db_dir.ToString(), wal_dir.ToString(),
                                          restore_options->rep);
}

const crocksdb_backup_engine_info_t* crocksdb_backup_engine_get_backup_info(
    crocksdb_backup_engine_t* be) {
  crocksdb_backup_engine_info_t* result = new crocksdb_backup_engine_info_t;
  be->rep->GetBackupInfo(&result->rep);
  return result;
}

int crocksdb_backup_engine_info_count(
    const crocksdb_backup_engine_info_t* info) {
  return static_cast<int>(info->rep.size());
}

int64_t crocksdb_backup_engine_info_timestamp(
    const crocksdb_backup_engine_info_t* info, int index) {
  return info->rep[index].timestamp;
}

uint32_t crocksdb_backup_engine_info_backup_id(
    const crocksdb_backup_engine_info_t* info, int index) {
  return info->rep[index].backup_id;
}

uint64_t crocksdb_backup_engine_info_size(
    const crocksdb_backup_engine_info_t* info, int index) {
  return info->rep[index].size;
}

uint32_t crocksdb_backup_engine_info_number_files(
    const crocksdb_backup_engine_info_t* info, int index) {
  return info->rep[index].number_files;
}

void crocksdb_backup_engine_info_destroy(
    const crocksdb_backup_engine_info_t* info) {
  delete info;
}

void crocksdb_backup_engine_close(crocksdb_backup_engine_t* be) {
  delete be->rep;
  delete be;
}

void crocksdb_close(crocksdb_t* db) {
  delete db->rep;
  delete db;
}

void crocksdb_pause_bg_work(crocksdb_t* db) { db->rep->PauseBackgroundWork(); }

void crocksdb_continue_bg_work(crocksdb_t* db) {
  db->rep->ContinueBackgroundWork();
}

crocksdb_t* crocksdb_open_column_families(
    const TitanDBOptions* db_options, Slice name, int num_column_families,
    Slice* column_family_names,
    const crocksdb_options_t** column_family_options,
    crocksdb_column_family_handle_t** column_family_handles, Status* s) {
  std::vector<ColumnFamilyDescriptor> column_families;
  for (int i = 0; i < num_column_families; i++) {
    column_families.push_back(ColumnFamilyDescriptor(
        column_family_names[i].ToString(),
        ColumnFamilyOptions(column_family_options[i]->rep)));
  }

  DB* db;
  std::vector<ColumnFamilyHandle*> handles;
  *s = DB::Open(*db_options, name.ToString(), column_families, &handles, &db);
  if (!s->ok()) {
    return nullptr;
  }

  for (size_t i = 0; i < handles.size(); i++) {
    crocksdb_column_family_handle_t* c_handle =
        new crocksdb_column_family_handle_t;
    c_handle->rep = handles[i];
    column_family_handles[i] = c_handle;
  }
  crocksdb_t* result = new crocksdb_t;
  result->rep = db;
  return result;
}

crocksdb_t* crocksdb_open_column_families_with_ttl(
    const TitanDBOptions* db_options, Slice name, int num_column_families,
    Slice* column_family_names,
    const crocksdb_options_t** column_family_options, const int32_t* ttl_array,
    unsigned char read_only,
    crocksdb_column_family_handle_t** column_family_handles, Status* s) {
  std::vector<ColumnFamilyDescriptor> column_families;
  std::vector<int32_t> ttls;
  for (int i = 0; i < num_column_families; i++) {
    column_families.push_back(ColumnFamilyDescriptor(
        column_family_names[i].ToString(),
        ColumnFamilyOptions(column_family_options[i]->rep)));
    ttls.push_back(ttl_array[i]);
  }

  DBWithTTL* db;
  std::vector<ColumnFamilyHandle*> handles;
  *s = DBWithTTL::Open(*db_options, name.ToString(), column_families, &handles,
                       &db, ttls, read_only);
  if (!s->ok()) {
    return nullptr;
  }

  for (size_t i = 0; i < handles.size(); i++) {
    crocksdb_column_family_handle_t* c_handle =
        new crocksdb_column_family_handle_t;
    c_handle->rep = handles[i];
    column_family_handles[i] = c_handle;
  }
  crocksdb_t* result = new crocksdb_t;
  result->rep = db;
  return result;
}

crocksdb_t* crocksdb_open_for_read_only_column_families(
    const TitanDBOptions* db_options, Slice name, int num_column_families,
    Slice* column_family_names,
    const crocksdb_options_t** column_family_options,
    crocksdb_column_family_handle_t** column_family_handles,
    unsigned char error_if_log_file_exist, Status* s) {
  std::vector<ColumnFamilyDescriptor> column_families;
  for (int i = 0; i < num_column_families; i++) {
    column_families.push_back(ColumnFamilyDescriptor(
        column_family_names[i].ToString(),
        ColumnFamilyOptions(column_family_options[i]->rep)));
  }

  DB* db;
  std::vector<ColumnFamilyHandle*> handles;
  *s = DB::OpenForReadOnly(*db_options, name.ToString(), column_families,
                           &handles, &db, error_if_log_file_exist);
  if (!s->ok()) {
    return nullptr;
  }

  for (size_t i = 0; i < handles.size(); i++) {
    crocksdb_column_family_handle_t* c_handle =
        new crocksdb_column_family_handle_t;
    c_handle->rep = handles[i];
    column_family_handles[i] = c_handle;
  }
  crocksdb_t* result = new crocksdb_t;
  result->rep = db;
  return result;
}

char** crocksdb_list_column_families(const crocksdb_options_t* options,
                                     Slice name, size_t* lencfs, Status* s) {
  std::vector<std::string> fams;
  *s = DB::ListColumnFamilies(DBOptions(options->rep), name.ToString(), &fams);
  if (!s->ok()) {
    return nullptr;
  }

  *lencfs = fams.size();
  char** column_families =
      static_cast<char**>(malloc(sizeof(char*) * fams.size()));
  for (size_t i = 0; i < fams.size(); i++) {
    column_families[i] = strdup(fams[i].c_str());
  }
  return column_families;
}

void crocksdb_list_column_families_destroy(char** list, size_t len) {
  for (size_t i = 0; i < len; ++i) {
    free(list[i]);
  }
  free(list);
}

crocksdb_column_family_handle_t* crocksdb_create_column_family(
    crocksdb_t* db, const crocksdb_options_t* column_family_options,
    const char* column_family_name, Status* s) {
  ColumnFamilyHandle* handle;
  *s = db->rep->CreateColumnFamily(
      ColumnFamilyOptions(column_family_options->rep),
      std::string(column_family_name), &handle);
  if (!s->ok()) {
    return nullptr;
  }
  auto result = new crocksdb_column_family_handle_t;
  result->rep = handle;
  return result;
}

void crocksdb_drop_column_family(crocksdb_t* db,
                                 crocksdb_column_family_handle_t* handle,
                                 Status* s) {
  *s = db->rep->DropColumnFamily(handle->rep);
}

uint32_t crocksdb_column_family_handle_id(
    crocksdb_column_family_handle_t* handle) {
  return handle->rep->GetID();
}

void crocksdb_column_family_handle_destroy(
    crocksdb_column_family_handle_t* handle) {
  delete handle->rep;
  delete handle;
}

void crocksdb_put(crocksdb_t* db, const WriteOptions* options, const char* key,
                  size_t keylen, const char* val, size_t vallen, Status* s) {
  *s = db->rep->Put(*options, Slice(key, keylen), Slice(val, vallen));
}

void crocksdb_put_cf(crocksdb_t* db, const WriteOptions* options,
                     crocksdb_column_family_handle_t* column_family,
                     const char* key, size_t keylen, const char* val,
                     size_t vallen, Status* s) {
  *s = db->rep->Put(*options, column_family->rep, Slice(key, keylen),
                    Slice(val, vallen));
}

void crocksdb_delete(crocksdb_t* db, const WriteOptions* options,
                     const char* key, size_t keylen, Status* s) {
  *s = db->rep->Delete(*options, Slice(key, keylen));
}

void crocksdb_delete_cf(crocksdb_t* db, const WriteOptions* options,
                        crocksdb_column_family_handle_t* column_family,
                        const char* key, size_t keylen, Status* s) {
  *s = db->rep->Delete(*options, column_family->rep, Slice(key, keylen));
}

void crocksdb_single_delete(crocksdb_t* db, const WriteOptions* options,
                            const char* key, size_t keylen, Status* s) {
  *s = db->rep->SingleDelete(*options, Slice(key, keylen));
}

void crocksdb_single_delete_cf(crocksdb_t* db, const WriteOptions* options,
                               crocksdb_column_family_handle_t* column_family,
                               const char* key, size_t keylen, Status* s) {
  *s = db->rep->SingleDelete(*options, column_family->rep, Slice(key, keylen));
}

void crocksdb_delete_range_cf(crocksdb_t* db, const WriteOptions* options,
                              crocksdb_column_family_handle_t* column_family,
                              const char* begin_key, size_t begin_keylen,
                              const char* end_key, size_t end_keylen,
                              Status* s) {
  *s = db->rep->DeleteRange(*options, column_family->rep,
                            Slice(begin_key, begin_keylen),
                            Slice(end_key, end_keylen));
}

void crocksdb_merge(crocksdb_t* db, const WriteOptions* options,
                    const char* key, size_t keylen, const char* val,
                    size_t vallen, Status* s) {
  *s = db->rep->Merge(*options, Slice(key, keylen), Slice(val, vallen));
}

void crocksdb_merge_cf(crocksdb_t* db, const WriteOptions* options,
                       crocksdb_column_family_handle_t* column_family,
                       const char* key, size_t keylen, const char* val,
                       size_t vallen, Status* s) {
  *s = db->rep->Merge(*options, column_family->rep, Slice(key, keylen),
                      Slice(val, vallen));
}

void crocksdb_write(crocksdb_t* db, const WriteOptions* options,
                    crocksdb_writebatch_t* batch, Status* s) {
  *s = db->rep->Write(*options, &batch->rep);
}

char* crocksdb_get(crocksdb_t* db, const ReadOptions* options, const char* key,
                   size_t keylen, size_t* vallen, Status* s) {
  std::string tmp;
  *s = db->rep->Get(*options, Slice(key, keylen), &tmp);
  if (s->ok()) {
    *vallen = tmp.size();
    return CopyString(tmp);
  }
  return nullptr;
}

char* crocksdb_get_cf(crocksdb_t* db, const ReadOptions* options,
                      crocksdb_column_family_handle_t* column_family,
                      const char* key, size_t keylen, size_t* vallen,
                      Status* s) {
  std::string tmp;
  *s = db->rep->Get(*options, column_family->rep, Slice(key, keylen), &tmp);
  if (s->ok()) {
    *vallen = tmp.size();
    return CopyString(tmp);
  }
  return nullptr;
}

void crocksdb_multi_get(crocksdb_t* db, const ReadOptions* options,
                        size_t num_keys, const char* const* keys_list,
                        const size_t* keys_list_sizes, char** values_list,
                        size_t* values_list_sizes, Status* status_list) {
  std::vector<Slice> keys(num_keys);
  for (size_t i = 0; i < num_keys; i++) {
    keys[i] = Slice(keys_list[i], keys_list_sizes[i]);
  }
  std::vector<std::string> values(num_keys);
  std::vector<Status> statuses = db->rep->MultiGet(*options, keys, &values);
  for (size_t i = 0; i < num_keys; i++) {
    status_list[i] = statuses[i];
    if (status_list[i].ok()) {
      values_list[i] = CopyString(values[i]);
      values_list_sizes[i] = values[i].size();
    }
  }
}

void crocksdb_multi_get_cf(
    crocksdb_t* db, const ReadOptions* options,
    const crocksdb_column_family_handle_t* const* column_families,
    size_t num_keys, const char* const* keys_list,
    const size_t* keys_list_sizes, char** values_list,
    size_t* values_list_sizes, Status* status_list) {
  std::vector<Slice> keys(num_keys);
  std::vector<ColumnFamilyHandle*> cfs(num_keys);
  for (size_t i = 0; i < num_keys; i++) {
    keys[i] = Slice(keys_list[i], keys_list_sizes[i]);
    cfs[i] = column_families[i]->rep;
  }
  std::vector<std::string> values(num_keys);
  std::vector<Status> statuses =
      db->rep->MultiGet(*options, cfs, keys, &values);
  for (size_t i = 0; i < num_keys; i++) {
    status_list[i] = statuses[i];
    if (status_list[i].ok()) {
      values_list[i] = CopyString(values[i]);
      values_list_sizes[i] = values[i].size();
    }
  }
}

crocksdb_iterator_t* crocksdb_create_iterator(crocksdb_t* db,
                                              const ReadOptions* options) {
  crocksdb_iterator_t* result = new crocksdb_iterator_t;
  result->rep = db->rep->NewIterator(*options);
  return result;
}

crocksdb_iterator_t* crocksdb_create_iterator_cf(
    crocksdb_t* db, const ReadOptions* options,
    crocksdb_column_family_handle_t* column_family) {
  crocksdb_iterator_t* result = new crocksdb_iterator_t;
  result->rep = db->rep->NewIterator(*options, column_family->rep);
  return result;
}

void crocksdb_create_iterators(
    crocksdb_t* db, const ReadOptions* opts,
    crocksdb_column_family_handle_t** column_families,
    crocksdb_iterator_t** iterators, size_t size, Status* s) {
  std::vector<ColumnFamilyHandle*> column_families_vec(size);
  for (size_t i = 0; i < size; i++) {
    column_families_vec.push_back(column_families[i]->rep);
  }

  std::vector<Iterator*> res;
  *s = db->rep->NewIterators(*opts, column_families_vec, &res);
  if (!s->ok()) {
    for (size_t i = 0; i < res.size(); i++) {
      delete res[i];
    }
    return;
  }
  assert(res.size() == size);

  for (size_t i = 0; i < size; i++) {
    iterators[i] = new crocksdb_iterator_t;
    iterators[i]->rep = res[i];
  }
}

const crocksdb_snapshot_t* crocksdb_create_snapshot(crocksdb_t* db) {
  crocksdb_snapshot_t* result = new crocksdb_snapshot_t;
  result->rep = db->rep->GetSnapshot();
  return result;
}

void crocksdb_release_snapshot(crocksdb_t* db,
                               const crocksdb_snapshot_t* snapshot) {
  db->rep->ReleaseSnapshot(snapshot->rep);
  delete snapshot;
}

uint64_t crocksdb_get_snapshot_sequence_number(
    const crocksdb_snapshot_t* snapshot) {
  return snapshot->rep->GetSequenceNumber();
}

crocksdb_map_property_t* crocksdb_create_map_property() {
  return new crocksdb_map_property_t;
}

void crocksdb_destroy_map_property(crocksdb_map_property_t* info) {
  delete info;
}

unsigned char crocksdb_get_map_property_cf(
    crocksdb_t* db, crocksdb_column_family_handle_t* column_family,
    const char* property, crocksdb_map_property_t* info) {
  return db->rep->GetMapProperty(column_family->rep, property, &info->rep);
}

char* crocksdb_map_property_value(crocksdb_map_property_t* info,
                                  const char* propname) {
  auto iter = info->rep.find(std::string(propname));
  if (iter != info->rep.end()) {
    return strdup(iter->second.c_str());
  } else {
    return nullptr;
  }
}

uint64_t crocksdb_map_property_int_value(crocksdb_map_property_t* info,
                                         const char* propname) {
  auto iter = info->rep.find(std::string(propname));
  if (iter != info->rep.end()) {
    return (uint64_t)stoll(iter->second, nullptr);
  } else {
    return 0;
  }
}

char* crocksdb_property_value(crocksdb_t* db, const char* propname) {
  std::string tmp;
  if (db->rep->GetProperty(Slice(propname), &tmp)) {
    // We use strdup() since we expect human readable output.
    return strdup(tmp.c_str());
  } else {
    return nullptr;
  }
}

char* crocksdb_property_value_cf(crocksdb_t* db,
                                 crocksdb_column_family_handle_t* column_family,
                                 const char* propname) {
  std::string tmp;
  if (db->rep->GetProperty(column_family->rep, Slice(propname), &tmp)) {
    // We use strdup() since we expect human readable output.
    return strdup(tmp.c_str());
  } else {
    return nullptr;
  }
}

void crocksdb_approximate_sizes(crocksdb_t* db, int num_ranges,
                                const char* const* range_start_key,
                                const size_t* range_start_key_len,
                                const char* const* range_limit_key,
                                const size_t* range_limit_key_len,
                                uint64_t* sizes) {
  Range* ranges = new Range[num_ranges];
  for (int i = 0; i < num_ranges; i++) {
    ranges[i].start = Slice(range_start_key[i], range_start_key_len[i]);
    ranges[i].limit = Slice(range_limit_key[i], range_limit_key_len[i]);
  }
  db->rep->GetApproximateSizes(ranges, num_ranges, sizes);
  delete[] ranges;
}

void crocksdb_approximate_sizes_cf(
    crocksdb_t* db, crocksdb_column_family_handle_t* column_family,
    int num_ranges, const char* const* range_start_key,
    const size_t* range_start_key_len, const char* const* range_limit_key,
    const size_t* range_limit_key_len, uint64_t* sizes) {
  Range* ranges = new Range[num_ranges];
  for (int i = 0; i < num_ranges; i++) {
    ranges[i].start = Slice(range_start_key[i], range_start_key_len[i]);
    ranges[i].limit = Slice(range_limit_key[i], range_limit_key_len[i]);
  }
  db->rep->GetApproximateSizes(column_family->rep, ranges, num_ranges, sizes);
  delete[] ranges;
}

void crocksdb_approximate_memtable_stats(const crocksdb_t* db,
                                         const char* range_start_key,
                                         size_t range_start_key_len,
                                         const char* range_limit_key,
                                         size_t range_limit_key_len,
                                         uint64_t* count, uint64_t* size) {
  auto start = Slice(range_start_key, range_start_key_len);
  auto limit = Slice(range_limit_key, range_limit_key_len);
  Range range(start, limit);
  db->rep->GetApproximateMemTableStats(range, count, size);
}

void crocksdb_approximate_memtable_stats_cf(
    const crocksdb_t* db, const crocksdb_column_family_handle_t* cf,
    const char* range_start_key, size_t range_start_key_len,
    const char* range_limit_key, size_t range_limit_key_len, uint64_t* count,
    uint64_t* size) {
  auto start = Slice(range_start_key, range_start_key_len);
  auto limit = Slice(range_limit_key, range_limit_key_len);
  Range range(start, limit);
  db->rep->GetApproximateMemTableStats(cf->rep, range, count, size);
}

void crocksdb_delete_file(crocksdb_t* db, const char* name, Status* s) {
  *s = db->rep->DeleteFile(name);
}

const crocksdb_livefiles_t* crocksdb_livefiles(crocksdb_t* db) {
  crocksdb_livefiles_t* result = new crocksdb_livefiles_t;
  db->rep->GetLiveFilesMetaData(&result->rep);
  return result;
}

void crocksdb_compact_range(crocksdb_t* db, const char* start_key,
                            size_t start_key_len, const char* limit_key,
                            size_t limit_key_len) {
  Slice a, b;
  db->rep->CompactRange(
      CompactRangeOptions(),
      // Pass nullptr Slice if corresponding "const char*" is nullptr
      (start_key ? (a = Slice(start_key, start_key_len), &a) : nullptr),
      (limit_key ? (b = Slice(limit_key, limit_key_len), &b) : nullptr));
}

void crocksdb_compact_range_cf(crocksdb_t* db,
                               crocksdb_column_family_handle_t* column_family,
                               const char* start_key, size_t start_key_len,
                               const char* limit_key, size_t limit_key_len) {
  Slice a, b;
  db->rep->CompactRange(
      CompactRangeOptions(), column_family->rep,
      // Pass nullptr Slice if corresponding "const char*" is nullptr
      (start_key ? (a = Slice(start_key, start_key_len), &a) : nullptr),
      (limit_key ? (b = Slice(limit_key, limit_key_len), &b) : nullptr));
}

void crocksdb_compact_range_opt(crocksdb_t* db, const CompactRangeOptions* opt,
                                const char* start_key, size_t start_key_len,
                                const char* limit_key, size_t limit_key_len) {
  Slice a, b;
  db->rep->CompactRange(
      *opt,
      // Pass nullptr Slice if corresponding "const char*" is nullptr
      (start_key ? (a = Slice(start_key, start_key_len), &a) : nullptr),
      (limit_key ? (b = Slice(limit_key, limit_key_len), &b) : nullptr));
}

void crocksdb_compact_range_cf_opt(
    crocksdb_t* db, crocksdb_column_family_handle_t* column_family,
    const CompactRangeOptions* opt, const char* start_key, size_t start_key_len,
    const char* limit_key, size_t limit_key_len) {
  Slice a, b;
  db->rep->CompactRange(
      *opt, column_family->rep,
      // Pass nullptr Slice if corresponding "const char*" is nullptr
      (start_key ? (a = Slice(start_key, start_key_len), &a) : nullptr),
      (limit_key ? (b = Slice(limit_key, limit_key_len), &b) : nullptr));
}

void crocksdb_flush(crocksdb_t* db, const FlushOptions* options, Status* s) {
  *s = db->rep->Flush(*options);
}

void crocksdb_flush_cf(crocksdb_t* db,
                       crocksdb_column_family_handle_t* column_family,
                       const FlushOptions* options, Status* s) {
  *s = db->rep->Flush(*options, column_family->rep);
}

void crocksdb_flush_cfs(crocksdb_t* db,
                        const crocksdb_column_family_handle_t** column_familys,
                        int num_handles, const FlushOptions* options,
                        Status* s) {
  std::vector<rocksdb::ColumnFamilyHandle*> handles(num_handles);
  for (int i = 0; i < num_handles; i++) {
    handles[i] = column_familys[i]->rep;
  }
  *s = db->rep->Flush(*options, handles);
}

void crocksdb_flush_wal(crocksdb_t* db, unsigned char sync, Status* s) {
  *s = db->rep->FlushWAL(sync);
}

void crocksdb_sync_wal(crocksdb_t* db, Status* s) { *s = db->rep->SyncWAL(); }

uint64_t crocksdb_get_latest_sequence_number(crocksdb_t* db) {
  return db->rep->GetLatestSequenceNumber();
}

void crocksdb_disable_file_deletions(crocksdb_t* db, Status* s) {
  *s = db->rep->DisableFileDeletions();
}

void crocksdb_enable_file_deletions(crocksdb_t* db, unsigned char force,
                                    Status* s) {
  *s = db->rep->EnableFileDeletions(force);
}

crocksdb_options_t* crocksdb_get_db_options(crocksdb_t* db) {
  auto opts = new crocksdb_options_t;
  opts->rep = Options(db->rep->GetDBOptions(), ColumnFamilyOptions());
  return opts;
}

void crocksdb_set_db_options(crocksdb_t* db, const char** names,
                             const char** values, size_t num_options,
                             Status* s) {
  std::unordered_map<std::string, std::string> options;
  for (size_t i = 0; i < num_options; i++) {
    options.emplace(names[i], values[i]);
  }
  *s = db->rep->SetDBOptions(options);
}

crocksdb_options_t* crocksdb_get_options_cf(
    const crocksdb_t* db, crocksdb_column_family_handle_t* column_family) {
  crocksdb_options_t* options = new crocksdb_options_t;
  options->rep = db->rep->GetOptions(column_family->rep);
  return options;
}

void crocksdb_set_options_cf(crocksdb_t* db,
                             crocksdb_column_family_handle_t* cf,
                             const char** names, const char** values,
                             size_t num_options, Status* s) {
  std::unordered_map<std::string, std::string> options;
  for (size_t i = 0; i < num_options; i++) {
    options.emplace(names[i], values[i]);
  }
  *s = db->rep->SetOptions(cf->rep, options);
}

void crocksdb_destroy_db(const crocksdb_options_t* options, const char* name,
                         Status* s) {
  *s = DestroyDB(name, options->rep);
}

void crocksdb_repair_db(const crocksdb_options_t* options, const char* name,
                        Status* s) {
  *s = RepairDB(name, options->rep);
}

void crocksdb_iter_destroy(crocksdb_iterator_t* iter) {
  delete iter->rep;
  delete iter;
}

unsigned char crocksdb_iter_valid(const crocksdb_iterator_t* iter) {
  return iter->rep->Valid();
}

void crocksdb_iter_seek_to_first(crocksdb_iterator_t* iter) {
  iter->rep->SeekToFirst();
}

void crocksdb_iter_seek_to_last(crocksdb_iterator_t* iter) {
  iter->rep->SeekToLast();
}

void crocksdb_iter_seek(crocksdb_iterator_t* iter, const char* k, size_t klen) {
  iter->rep->Seek(Slice(k, klen));
}

void crocksdb_iter_seek_for_prev(crocksdb_iterator_t* iter, const char* k,
                                 size_t klen) {
  iter->rep->SeekForPrev(Slice(k, klen));
}

void crocksdb_iter_next(crocksdb_iterator_t* iter) { iter->rep->Next(); }

void crocksdb_iter_prev(crocksdb_iterator_t* iter) { iter->rep->Prev(); }

const char* crocksdb_iter_key(const crocksdb_iterator_t* iter, size_t* klen) {
  Slice s = iter->rep->key();
  *klen = s.size();
  return s.data();
}

const char* crocksdb_iter_value(const crocksdb_iterator_t* iter, size_t* vlen) {
  Slice s = iter->rep->value();
  *vlen = s.size();
  return s.data();
}

bool crocksdb_iter_seqno(const crocksdb_iterator_t* iter, SequenceNumber* no) {
  return iter->rep->seqno(no);
}

void crocksdb_iter_get_error(const crocksdb_iterator_t* iter, Status* s) {
  *s = iter->rep->status();
}

crocksdb_writebatch_t* crocksdb_writebatch_create() {
  return new crocksdb_writebatch_t;
}

crocksdb_writebatch_t* crocksdb_writebatch_create_with_capacity(
    size_t reserved_bytes) {
  crocksdb_writebatch_t* b = new crocksdb_writebatch_t;
  b->rep = WriteBatch(reserved_bytes);
  return b;
}

crocksdb_writebatch_t* crocksdb_writebatch_create_from(const char* rep,
                                                       size_t size) {
  crocksdb_writebatch_t* b = new crocksdb_writebatch_t;
  b->rep = WriteBatch(std::string(rep, size));
  return b;
}

void crocksdb_writebatch_destroy(crocksdb_writebatch_t* b) { delete b; }

void crocksdb_writebatch_clear(crocksdb_writebatch_t* b) { b->rep.Clear(); }

int crocksdb_writebatch_count(crocksdb_writebatch_t* b) {
  return b->rep.Count();
}

void crocksdb_writebatch_put(crocksdb_writebatch_t* b, const char* key,
                             size_t klen, const char* val, size_t vlen) {
  b->rep.Put(Slice(key, klen), Slice(val, vlen));
}

void crocksdb_writebatch_put_cf(crocksdb_writebatch_t* b,
                                crocksdb_column_family_handle_t* column_family,
                                const char* key, size_t klen, const char* val,
                                size_t vlen) {
  b->rep.Put(column_family->rep, Slice(key, klen), Slice(val, vlen));
}

void crocksdb_writebatch_putv(crocksdb_writebatch_t* b, int num_keys,
                              const char* const* keys_list,
                              const size_t* keys_list_sizes, int num_values,
                              const char* const* values_list,
                              const size_t* values_list_sizes) {
  std::vector<Slice> key_slices(num_keys);
  for (int i = 0; i < num_keys; i++) {
    key_slices[i] = Slice(keys_list[i], keys_list_sizes[i]);
  }
  std::vector<Slice> value_slices(num_values);
  for (int i = 0; i < num_values; i++) {
    value_slices[i] = Slice(values_list[i], values_list_sizes[i]);
  }
  b->rep.Put(SliceParts(key_slices.data(), num_keys),
             SliceParts(value_slices.data(), num_values));
}

void crocksdb_writebatch_putv_cf(crocksdb_writebatch_t* b,
                                 crocksdb_column_family_handle_t* column_family,
                                 int num_keys, const char* const* keys_list,
                                 const size_t* keys_list_sizes, int num_values,
                                 const char* const* values_list,
                                 const size_t* values_list_sizes) {
  std::vector<Slice> key_slices(num_keys);
  for (int i = 0; i < num_keys; i++) {
    key_slices[i] = Slice(keys_list[i], keys_list_sizes[i]);
  }
  std::vector<Slice> value_slices(num_values);
  for (int i = 0; i < num_values; i++) {
    value_slices[i] = Slice(values_list[i], values_list_sizes[i]);
  }
  b->rep.Put(column_family->rep, SliceParts(key_slices.data(), num_keys),
             SliceParts(value_slices.data(), num_values));
}

void crocksdb_writebatch_merge(crocksdb_writebatch_t* b, const char* key,
                               size_t klen, const char* val, size_t vlen) {
  b->rep.Merge(Slice(key, klen), Slice(val, vlen));
}

void crocksdb_writebatch_merge_cf(
    crocksdb_writebatch_t* b, crocksdb_column_family_handle_t* column_family,
    const char* key, size_t klen, const char* val, size_t vlen) {
  b->rep.Merge(column_family->rep, Slice(key, klen), Slice(val, vlen));
}

void crocksdb_writebatch_mergev(crocksdb_writebatch_t* b, int num_keys,
                                const char* const* keys_list,
                                const size_t* keys_list_sizes, int num_values,
                                const char* const* values_list,
                                const size_t* values_list_sizes) {
  std::vector<Slice> key_slices(num_keys);
  for (int i = 0; i < num_keys; i++) {
    key_slices[i] = Slice(keys_list[i], keys_list_sizes[i]);
  }
  std::vector<Slice> value_slices(num_values);
  for (int i = 0; i < num_values; i++) {
    value_slices[i] = Slice(values_list[i], values_list_sizes[i]);
  }
  b->rep.Merge(SliceParts(key_slices.data(), num_keys),
               SliceParts(value_slices.data(), num_values));
}

void crocksdb_writebatch_mergev_cf(
    crocksdb_writebatch_t* b, crocksdb_column_family_handle_t* column_family,
    int num_keys, const char* const* keys_list, const size_t* keys_list_sizes,
    int num_values, const char* const* values_list,
    const size_t* values_list_sizes) {
  std::vector<Slice> key_slices(num_keys);
  for (int i = 0; i < num_keys; i++) {
    key_slices[i] = Slice(keys_list[i], keys_list_sizes[i]);
  }
  std::vector<Slice> value_slices(num_values);
  for (int i = 0; i < num_values; i++) {
    value_slices[i] = Slice(values_list[i], values_list_sizes[i]);
  }
  b->rep.Merge(column_family->rep, SliceParts(key_slices.data(), num_keys),
               SliceParts(value_slices.data(), num_values));
}

void crocksdb_writebatch_delete(crocksdb_writebatch_t* b, const char* key,
                                size_t klen) {
  b->rep.Delete(Slice(key, klen));
}

void crocksdb_writebatch_delete_cf(
    crocksdb_writebatch_t* b, crocksdb_column_family_handle_t* column_family,
    const char* key, size_t klen) {
  b->rep.Delete(column_family->rep, Slice(key, klen));
}

void crocksdb_writebatch_single_delete(crocksdb_writebatch_t* b,
                                       const char* key, size_t klen) {
  b->rep.SingleDelete(Slice(key, klen));
}

void crocksdb_writebatch_single_delete_cf(
    crocksdb_writebatch_t* b, crocksdb_column_family_handle_t* column_family,
    const char* key, size_t klen) {
  b->rep.SingleDelete(column_family->rep, Slice(key, klen));
}

void crocksdb_writebatch_deletev(crocksdb_writebatch_t* b, int num_keys,
                                 const char* const* keys_list,
                                 const size_t* keys_list_sizes) {
  std::vector<Slice> key_slices(num_keys);
  for (int i = 0; i < num_keys; i++) {
    key_slices[i] = Slice(keys_list[i], keys_list_sizes[i]);
  }
  b->rep.Delete(SliceParts(key_slices.data(), num_keys));
}

void crocksdb_writebatch_deletev_cf(
    crocksdb_writebatch_t* b, crocksdb_column_family_handle_t* column_family,
    int num_keys, const char* const* keys_list, const size_t* keys_list_sizes) {
  std::vector<Slice> key_slices(num_keys);
  for (int i = 0; i < num_keys; i++) {
    key_slices[i] = Slice(keys_list[i], keys_list_sizes[i]);
  }
  b->rep.Delete(column_family->rep, SliceParts(key_slices.data(), num_keys));
}

void crocksdb_writebatch_delete_range(crocksdb_writebatch_t* b,
                                      const char* start_key,
                                      size_t start_key_len, const char* end_key,
                                      size_t end_key_len) {
  b->rep.DeleteRange(Slice(start_key, start_key_len),
                     Slice(end_key, end_key_len));
}

void crocksdb_writebatch_delete_range_cf(
    crocksdb_writebatch_t* b, crocksdb_column_family_handle_t* column_family,
    const char* start_key, size_t start_key_len, const char* end_key,
    size_t end_key_len) {
  b->rep.DeleteRange(column_family->rep, Slice(start_key, start_key_len),
                     Slice(end_key, end_key_len));
}

void crocksdb_writebatch_delete_rangev(crocksdb_writebatch_t* b, int num_keys,
                                       const char* const* start_keys_list,
                                       const size_t* start_keys_list_sizes,
                                       const char* const* end_keys_list,
                                       const size_t* end_keys_list_sizes) {
  std::vector<Slice> start_key_slices(num_keys);
  std::vector<Slice> end_key_slices(num_keys);
  for (int i = 0; i < num_keys; i++) {
    start_key_slices[i] = Slice(start_keys_list[i], start_keys_list_sizes[i]);
    end_key_slices[i] = Slice(end_keys_list[i], end_keys_list_sizes[i]);
  }
  b->rep.DeleteRange(SliceParts(start_key_slices.data(), num_keys),
                     SliceParts(end_key_slices.data(), num_keys));
}

void crocksdb_writebatch_delete_rangev_cf(
    crocksdb_writebatch_t* b, crocksdb_column_family_handle_t* column_family,
    int num_keys, const char* const* start_keys_list,
    const size_t* start_keys_list_sizes, const char* const* end_keys_list,
    const size_t* end_keys_list_sizes) {
  std::vector<Slice> start_key_slices(num_keys);
  std::vector<Slice> end_key_slices(num_keys);
  for (int i = 0; i < num_keys; i++) {
    start_key_slices[i] = Slice(start_keys_list[i], start_keys_list_sizes[i]);
    end_key_slices[i] = Slice(end_keys_list[i], end_keys_list_sizes[i]);
  }
  b->rep.DeleteRange(column_family->rep,
                     SliceParts(start_key_slices.data(), num_keys),
                     SliceParts(end_key_slices.data(), num_keys));
}

void crocksdb_writebatch_put_log_data(crocksdb_writebatch_t* b,
                                      const char* blob, size_t len) {
  b->rep.PutLogData(Slice(blob, len));
}

void crocksdb_writebatch_iterate(crocksdb_writebatch_t* b, void* state,
                                 void (*put)(void*, const char* k, size_t klen,
                                             const char* v, size_t vlen),
                                 void (*deleted)(void*, const char* k,
                                                 size_t klen)) {
  class HandlerWrapper : public WriteBatch::Handler {
   public:
    void* state_;
    void (*put_)(void*, const char* k, size_t klen, const char* v, size_t vlen);
    void (*deleted_)(void*, const char* k, size_t klen);
    void Put(const Slice& key, const Slice& value) override {
      (*put_)(state_, key.data(), key.size(), value.data(), value.size());
    }
    void Delete(const Slice& key) override {
      (*deleted_)(state_, key.data(), key.size());
    }
  };
  HandlerWrapper handler;
  handler.state_ = state;
  handler.put_ = put;
  handler.deleted_ = deleted;
  b->rep.Iterate(&handler);
}

void crocksdb_writebatch_iterate_cf(
    crocksdb_writebatch_t* b, void* state,
    void (*put)(void*, const char* k, size_t klen, const char* v, size_t vlen),
    void (*put_cf)(void*, uint32_t cf, const char* k, size_t klen,
                   const char* v, size_t vlen),
    void (*deleted)(void*, const char* k, size_t klen),
    void (*deleted_cf)(void*, uint32_t cf, const char* k, size_t klen)) {
  class HandlerWrapper : public WriteBatch::Handler {
   public:
    void* state_;
    void (*put_)(void*, const char* k, size_t klen, const char* v, size_t vlen);
    void (*put_cf_)(void*, uint32_t cf, const char* k, size_t klen,
                    const char* v, size_t vlen);
    void (*deleted_)(void*, const char* k, size_t klen);
    void (*deleted_cf_)(void*, uint32_t cf, const char* k, size_t klen);

    void Put(const Slice& key, const Slice& value) override {
      (*put_)(state_, key.data(), key.size(), value.data(), value.size());
    }

    Status PutCF(uint32_t column_family_id, const Slice& key,
                 const Slice& value) override {
      (*put_cf_)(state_, column_family_id, key.data(), key.size(), value.data(),
                 value.size());
      return Status::OK();
    }

    void Delete(const Slice& key) override {
      (*deleted_)(state_, key.data(), key.size());
    }

    Status DeleteCF(uint32_t column_family_id, const Slice& key) override {
      (*deleted_cf_)(state_, column_family_id, key.data(), key.size());
      return Status::OK();
    }
  };
  HandlerWrapper handler;
  handler.state_ = state;
  handler.put_ = put;
  handler.put_cf_ = put_cf;
  handler.deleted_ = deleted;
  handler.deleted_cf_ = deleted_cf;
  b->rep.Iterate(&handler);
}

const char* crocksdb_writebatch_data(crocksdb_writebatch_t* b, size_t* size) {
  *size = b->rep.GetDataSize();
  return b->rep.Data().c_str();
}

void crocksdb_writebatch_set_save_point(crocksdb_writebatch_t* b) {
  b->rep.SetSavePoint();
}

void crocksdb_writebatch_pop_save_point(crocksdb_writebatch_t* b, Status* s) {
  *s = b->rep.PopSavePoint();
}

void crocksdb_writebatch_rollback_to_save_point(crocksdb_writebatch_t* b,
                                                Status* s) {
  *s = b->rep.RollbackToSavePoint();
}

void crocksdb_writebatch_set_content(crocksdb_writebatch_t* b, const char* data,
                                     size_t dlen) {
  rocksdb::WriteBatchInternal::SetContents(&b->rep, Slice(data, dlen));
}

void crocksdb_writebatch_append_content(crocksdb_writebatch_t* dest,
                                        const char* data, size_t dlen) {
  rocksdb::WriteBatchInternal::AppendContents(&dest->rep, Slice(data, dlen));
}

int crocksdb_writebatch_ref_count(const char* data, size_t dlen) {
  Slice s(data, dlen);
  rocksdb::WriteBatch::WriteBatchRef ref(s);
  return ref.Count();
}

crocksdb_writebatch_iterator_t* crocksdb_writebatch_ref_iterator_create(
    const char* data, size_t dlen) {
  Slice input(data, dlen);
  rocksdb::WriteBatch::WriteBatchRef ref(input);
  auto it = new crocksdb_writebatch_iterator_t;
  it->rep = ref.NewIterator();
  it->rep->SeekToFirst();
  return it;
}

crocksdb_writebatch_iterator_t* crocksdb_writebatch_iterator_create(
    crocksdb_writebatch_t* dest) {
  auto it = new crocksdb_writebatch_iterator_t;
  it->rep = dest->rep.NewIterator();
  it->rep->SeekToFirst();
  return it;
}

void crocksdb_writebatch_iterator_destroy(crocksdb_writebatch_iterator_t* it) {
  delete it->rep;
  delete it;
}

unsigned char crocksdb_writebatch_iterator_valid(
    crocksdb_writebatch_iterator_t* it) {
  return it->rep->Valid();
}

void crocksdb_writebatch_iterator_next(crocksdb_writebatch_iterator_t* it) {
  it->rep->Next();
}

const char* crocksdb_writebatch_iterator_key(crocksdb_writebatch_iterator_t* it,
                                             size_t* klen) {
  *klen = it->rep->Key().size();
  return it->rep->Key().data();
}

const char* crocksdb_writebatch_iterator_value(
    crocksdb_writebatch_iterator_t* it, size_t* klen) {
  *klen = it->rep->Value().size();
  return it->rep->Value().data();
}

int crocksdb_writebatch_iterator_value_type(
    crocksdb_writebatch_iterator_t* it) {
  return static_cast<int>(it->rep->GetValueType());
}

uint32_t crocksdb_writebatch_iterator_column_family_id(
    crocksdb_writebatch_iterator_t* it) {
  return it->rep->GetColumnFamilyId();
}

crocksdb_block_based_table_options_t* crocksdb_block_based_options_create() {
  return new crocksdb_block_based_table_options_t;
}

void crocksdb_block_based_options_destroy(
    crocksdb_block_based_table_options_t* options) {
  delete options;
}

void crocksdb_block_based_options_set_metadata_block_size(
    crocksdb_block_based_table_options_t* options, size_t block_size) {
  options->rep.metadata_block_size = block_size;
}

void crocksdb_block_based_options_set_block_size(
    crocksdb_block_based_table_options_t* options, size_t block_size) {
  options->rep.block_size = block_size;
}

void crocksdb_block_based_options_set_block_size_deviation(
    crocksdb_block_based_table_options_t* options, int block_size_deviation) {
  options->rep.block_size_deviation = block_size_deviation;
}

void crocksdb_block_based_options_set_block_restart_interval(
    crocksdb_block_based_table_options_t* options, int block_restart_interval) {
  options->rep.block_restart_interval = block_restart_interval;
}

void crocksdb_block_based_options_set_filter_policy(
    crocksdb_block_based_table_options_t* options,
    crocksdb_filterpolicy_t* filter_policy) {
  options->rep.filter_policy.reset(filter_policy);
}

void crocksdb_block_based_options_set_no_block_cache(
    crocksdb_block_based_table_options_t* options,
    unsigned char no_block_cache) {
  options->rep.no_block_cache = no_block_cache;
}

void crocksdb_block_based_options_set_block_cache(
    crocksdb_block_based_table_options_t* options,
    crocksdb_cache_t* block_cache) {
  if (block_cache) {
    options->rep.block_cache = block_cache->rep;
  }
}

void crocksdb_block_based_options_set_block_cache_compressed(
    crocksdb_block_based_table_options_t* options,
    crocksdb_cache_t* block_cache_compressed) {
  if (block_cache_compressed) {
    options->rep.block_cache_compressed = block_cache_compressed->rep;
  }
}

void crocksdb_block_based_options_set_whole_key_filtering(
    crocksdb_block_based_table_options_t* options, unsigned char v) {
  options->rep.whole_key_filtering = v;
}

void crocksdb_block_based_options_set_format_version(
    crocksdb_block_based_table_options_t* options, int v) {
  options->rep.format_version = v;
}

void crocksdb_block_based_options_set_index_type(
    crocksdb_block_based_table_options_t* options,
    BlockBasedTableOptions::IndexType v) {
  options->rep.index_type = v;
}

void crocksdb_block_based_options_set_hash_index_allow_collision(
    crocksdb_block_based_table_options_t* options, unsigned char v) {
  options->rep.hash_index_allow_collision = v;
}

void crocksdb_block_based_options_set_partition_filters(
    crocksdb_block_based_table_options_t* options, unsigned char v) {
  options->rep.partition_filters = v;
}

void crocksdb_block_based_options_set_cache_index_and_filter_blocks(
    crocksdb_block_based_table_options_t* options, unsigned char v) {
  options->rep.cache_index_and_filter_blocks = v;
}

void crocksdb_block_based_options_set_pin_top_level_index_and_filter(
    crocksdb_block_based_table_options_t* options, unsigned char v) {
  options->rep.pin_top_level_index_and_filter = v;
}

void crocksdb_block_based_options_set_cache_index_and_filter_blocks_with_high_priority(
    crocksdb_block_based_table_options_t* options, unsigned char v) {
  options->rep.cache_index_and_filter_blocks_with_high_priority = v;
}

void crocksdb_block_based_options_set_pin_l0_filter_and_index_blocks_in_cache(
    crocksdb_block_based_table_options_t* options, unsigned char v) {
  options->rep.pin_l0_filter_and_index_blocks_in_cache = v;
}

void crocksdb_block_based_options_set_read_amp_bytes_per_bit(
    crocksdb_block_based_table_options_t* options, int v) {
  options->rep.read_amp_bytes_per_bit = v;
}

void crocksdb_options_set_block_based_table_factory(
    crocksdb_options_t* opt,
    crocksdb_block_based_table_options_t* table_options) {
  if (table_options) {
    opt->rep.table_factory.reset(
        rocksdb::NewBlockBasedTableFactory(table_options->rep));
  }
}

void crocksdb_options_set_max_subcompactions(TitanDBOptions* opt, uint32_t v) {
  opt->max_subcompactions = v;
}

void crocksdb_options_set_wal_bytes_per_sync(TitanDBOptions* opt, uint64_t v) {
  opt->wal_bytes_per_sync = v;
}

static BlockBasedTableOptions* get_block_based_table_options(
    crocksdb_options_t* opt) {
  if (opt && opt->rep.table_factory != nullptr) {
    void* table_opt = opt->rep.table_factory->GetOptions();
    if (table_opt &&
        strcmp(opt->rep.table_factory->Name(), block_base_table_str) == 0) {
      return static_cast<BlockBasedTableOptions*>(table_opt);
    }
  }
  return nullptr;
}

size_t crocksdb_options_get_block_cache_usage(crocksdb_options_t* opt) {
  auto opts = get_block_based_table_options(opt);
  if (opts && opts->block_cache) {
    return opts->block_cache->GetUsage();
  }
  return 0;
}

void crocksdb_options_set_block_cache_capacity(crocksdb_options_t* opt,
                                               size_t capacity, Status* s) {
  auto opts = get_block_based_table_options(opt);
  if (opts && opts->block_cache) {
    opts->block_cache->SetCapacity(capacity);
    *s = Status::OK();
  } else {
    *s = Status::InvalidArgument("failed to get block based table options");
  }
}

size_t crocksdb_options_get_block_cache_capacity(crocksdb_options_t* opt) {
  auto opts = get_block_based_table_options(opt);
  if (opts && opts->block_cache) {
    return opts->block_cache->GetCapacity();
  }
  return 0;
}

/* FlushJobInfo */

int crocksdb_flushjobinfo_job_id(const FlushJobInfo* info) {
  return info->job_id;
}

void crocksdb_flushjobinfo_cf_name(const FlushJobInfo* info, Slice* cf_name) {
  *cf_name = info->cf_name;
}

void crocksdb_flushjobinfo_file_path(const FlushJobInfo* info, Slice* path) {
  *path = info->file_path;
}

const TableProperties* crocksdb_flushjobinfo_table_properties(
    const FlushJobInfo* info) {
  return &info->table_properties;
}

bool crocksdb_flushjobinfo_triggered_writes_slowdown(const FlushJobInfo* info) {
  return info->triggered_writes_slowdown;
}

bool crocksdb_flushjobinfo_triggered_writes_stop(const FlushJobInfo* info) {
  return info->triggered_writes_stop;
}

/* CompactionJobInfo */

void crocksdb_compactionjobinfo_status(const CompactionJobInfo* info,
                                       Status* s) {
  *s = info->status;
}

void crocksdb_compactionjobinfo_cf_name(const CompactionJobInfo* info,
                                        Slice* name) {
  *name = info->cf_name;
}

size_t crocksdb_compactionjobinfo_input_files_count(
    const CompactionJobInfo* info) {
  return info->input_files.size();
}

void crocksdb_compactionjobinfo_input_file_at(const CompactionJobInfo* info,
                                              size_t pos, Slice* file) {
  *file = info->input_files[pos];
}

size_t crocksdb_compactionjobinfo_output_files_count(
    const CompactionJobInfo* info) {
  return info->output_files.size();
}

void crocksdb_compactionjobinfo_output_file_at(const CompactionJobInfo* info,
                                               size_t pos, Slice* file) {
  *file = info->output_files[pos];
}

const TablePropertiesCollection* crocksdb_compactionjobinfo_table_properties(
    const CompactionJobInfo* info) {
  return &info->table_properties;
}

uint64_t crocksdb_compactionjobinfo_elapsed_micros(
    const CompactionJobInfo* info) {
  return info->stats.elapsed_micros;
}

uint64_t crocksdb_compactionjobinfo_num_corrupt_keys(
    const CompactionJobInfo* info) {
  return info->stats.num_corrupt_keys;
}

int crocksdb_compactionjobinfo_base_input_level(const CompactionJobInfo* info) {
  return info->base_input_level;
}

int crocksdb_compactionjobinfo_output_level(const CompactionJobInfo* info) {
  return info->output_level;
}

size_t crocksdb_compactionjobinfo_num_input_files(
    const CompactionJobInfo* info) {
  return info->stats.num_input_files;
}

size_t crocksdb_compactionjobinfo_num_input_files_at_output_level(
    const CompactionJobInfo* info) {
  return info->stats.num_input_files_at_output_level;
}

uint64_t crocksdb_compactionjobinfo_input_records(
    const CompactionJobInfo* info) {
  return info->stats.num_input_records;
}

uint64_t crocksdb_compactionjobinfo_output_records(
    const CompactionJobInfo* info) {
  return info->stats.num_output_records;
}

uint64_t crocksdb_compactionjobinfo_total_input_bytes(
    const CompactionJobInfo* info) {
  return info->stats.total_input_bytes;
}

uint64_t crocksdb_compactionjobinfo_total_output_bytes(
    const CompactionJobInfo* info) {
  return info->stats.total_output_bytes;
}

CompactionReason crocksdb_compactionjobinfo_compaction_reason(
    const CompactionJobInfo* info) {
  return info->compaction_reason;
}

/* SubcompactionJobInfo */

void crocksdb_subcompactionjobinfo_status(const SubcompactionJobInfo* info,
                                          Status* s) {
  *s = info->status;
}

void crocksdb_subcompactionjobinfo_cf_name(const SubcompactionJobInfo* info,
                                           Slice* cf_name) {
  *cf_name = info->cf_name;
}

uint64_t crocksdb_subcompactionjobinfo_thread_id(
    const SubcompactionJobInfo* info) {
  return info->thread_id;
}

int crocksdb_subcompactionjobinfo_base_input_level(
    const SubcompactionJobInfo* info) {
  return info->base_input_level;
}

int crocksdb_subcompactionjobinfo_output_level(
    const SubcompactionJobInfo* info) {
  return info->output_level;
}

/* ExternalFileIngestionInfo */

void crocksdb_externalfileingestioninfo_cf_name(
    const ExternalFileIngestionInfo* info, Slice* cf_name) {
  *cf_name = info->cf_name;
}

void crocksdb_externalfileingestioninfo_internal_file_path(
    const ExternalFileIngestionInfo* info, Slice* file) {
  *file = info->internal_file_path;
}

const TableProperties* crocksdb_externalfileingestioninfo_table_properties(
    const ExternalFileIngestionInfo* info) {
  return &info->table_properties;
}

int crocksdb_externalfileingestioninfo_picked_level(
    const ExternalFileIngestionInfo* info) {
  return info->picked_level;
}

/* External write stall info */
void crocksdb_writestallinfo_cf_name(const WriteStallInfo* info, Slice* name) {
  *name = info->cf_name;
}

WriteStallCondition crocksdb_writestallinfo_cur(const WriteStallInfo* info) {
  return info->condition.cur;
}

WriteStallCondition crocksdb_writestallinfo_prev(const WriteStallInfo* info) {
  return info->condition.prev;
}

/* event listener */

struct crocksdb_eventlistener_t : public EventListener {
  void* state_;
  void (*destructor_)(void*);
  void (*on_flush_begin)(void*, crocksdb_t*, const FlushJobInfo*);
  void (*on_flush_completed)(void*, crocksdb_t*, const FlushJobInfo*);
  void (*on_compaction_begin)(void*, crocksdb_t*, const CompactionJobInfo*);
  void (*on_compaction_completed)(void*, crocksdb_t*, const CompactionJobInfo*);
  void (*on_subcompaction_begin)(void*, const SubcompactionJobInfo*);
  void (*on_subcompaction_completed)(void*, const SubcompactionJobInfo*);
  void (*on_external_file_ingested)(void*, crocksdb_t*,
                                    const ExternalFileIngestionInfo*);
  void (*on_background_error)(void*, rocksdb::BackgroundErrorReason, Status*);
  void (*on_stall_conditions_changed)(void*, const WriteStallInfo*);

  virtual void OnFlushBegin(DB* db, const FlushJobInfo& info) {
    crocksdb_t c_db = {db};
    on_flush_begin(state_, &c_db, &info);
  }

  virtual void OnFlushCompleted(DB* db, const FlushJobInfo& info) {
    crocksdb_t c_db = {db};
    on_flush_completed(state_, &c_db, &info);
  }

  virtual void OnCompactionBegin(DB* db, const CompactionJobInfo& info) {
    crocksdb_t c_db = {db};
    on_compaction_begin(state_, &c_db, &info);
  }

  virtual void OnCompactionCompleted(DB* db, const CompactionJobInfo& info) {
    crocksdb_t c_db = {db};
    on_compaction_completed(state_, &c_db, &info);
  }

  virtual void OnSubcompactionBegin(const SubcompactionJobInfo& info) {
    on_subcompaction_begin(state_, &info);
  }

  virtual void OnSubcompactionCompleted(const SubcompactionJobInfo& info) {
    on_subcompaction_completed(state_, &info);
  }

  virtual void OnExternalFileIngested(DB* db,
                                      const ExternalFileIngestionInfo& info) {
    crocksdb_t c_db = {db};
    on_external_file_ingested(state_, &c_db, &info);
  }

  virtual void OnBackgroundError(BackgroundErrorReason reason, Status* status) {
    on_background_error(state_, reason, status);
  }

  virtual void OnStallConditionsChanged(const WriteStallInfo& info) {
    on_stall_conditions_changed(state_, &info);
  }

  virtual ~crocksdb_eventlistener_t() { destructor_(state_); }
};

crocksdb_eventlistener_t* crocksdb_eventlistener_create(
    void* state_, void (*destructor_)(void*), on_flush_begin_cb on_flush_begin,
    on_flush_completed_cb on_flush_completed,
    on_compaction_begin_cb on_compaction_begin,
    on_compaction_completed_cb on_compaction_completed,
    on_subcompaction_begin_cb on_subcompaction_begin,
    on_subcompaction_completed_cb on_subcompaction_completed,
    on_external_file_ingested_cb on_external_file_ingested,
    on_background_error_cb on_background_error,
    on_stall_conditions_changed_cb on_stall_conditions_changed) {
  crocksdb_eventlistener_t* et = new crocksdb_eventlistener_t;
  et->state_ = state_;
  et->destructor_ = destructor_;
  et->on_flush_begin = on_flush_begin;
  et->on_flush_completed = on_flush_completed;
  et->on_compaction_begin = on_compaction_begin;
  et->on_compaction_completed = on_compaction_completed;
  et->on_subcompaction_begin = on_subcompaction_begin;
  et->on_subcompaction_completed = on_subcompaction_completed;
  et->on_external_file_ingested = on_external_file_ingested;
  et->on_background_error = on_background_error;
  et->on_stall_conditions_changed = on_stall_conditions_changed;
  return et;
}

void crocksdb_eventlistener_destroy(crocksdb_eventlistener_t* t) { delete t; }

void crocksdb_options_add_eventlistener(TitanDBOptions* opt,
                                        crocksdb_eventlistener_t* t) {
  opt->listeners.emplace_back(std::shared_ptr<EventListener>(t));
}

crocksdb_cuckoo_table_options_t* crocksdb_cuckoo_options_create() {
  return new crocksdb_cuckoo_table_options_t;
}

void crocksdb_cuckoo_options_destroy(crocksdb_cuckoo_table_options_t* options) {
  delete options;
}

void crocksdb_cuckoo_options_set_hash_ratio(
    crocksdb_cuckoo_table_options_t* options, double v) {
  options->rep.hash_table_ratio = v;
}

void crocksdb_cuckoo_options_set_max_search_depth(
    crocksdb_cuckoo_table_options_t* options, uint32_t v) {
  options->rep.max_search_depth = v;
}

void crocksdb_cuckoo_options_set_cuckoo_block_size(
    crocksdb_cuckoo_table_options_t* options, uint32_t v) {
  options->rep.cuckoo_block_size = v;
}

void crocksdb_cuckoo_options_set_identity_as_first_hash(
    crocksdb_cuckoo_table_options_t* options, unsigned char v) {
  options->rep.identity_as_first_hash = v;
}

void crocksdb_cuckoo_options_set_use_module_hash(
    crocksdb_cuckoo_table_options_t* options, unsigned char v) {
  options->rep.use_module_hash = v;
}

void crocksdb_options_set_cuckoo_table_factory(
    crocksdb_options_t* opt, crocksdb_cuckoo_table_options_t* table_options) {
  if (table_options) {
    opt->rep.table_factory.reset(
        rocksdb::NewCuckooTableFactory(table_options->rep));
  }
}

crocksdb_options_t* crocksdb_options_create() { return new crocksdb_options_t; }

crocksdb_options_t* crocksdb_options_copy(const crocksdb_options_t* other) {
  return new crocksdb_options_t{Options(other->rep)};
}

void crocksdb_options_destroy(crocksdb_options_t* options) { delete options; }

void crocksdb_column_family_descriptor_destroy(
    crocksdb_column_family_descriptor* cf_desc) {
  delete cf_desc;
}

const char* crocksdb_name_from_column_family_descriptor(
    const crocksdb_column_family_descriptor* cf_desc) {
  return cf_desc->rep.name.c_str();
}

crocksdb_options_t* crocksdb_options_from_column_family_descriptor(
    const crocksdb_column_family_descriptor* cf_desc) {
  crocksdb_options_t* options = new crocksdb_options_t;
  *static_cast<ColumnFamilyOptions*>(&options->rep) = cf_desc->rep.options;
  return options;
}

void crocksdb_options_increase_parallelism(crocksdb_options_t* opt,
                                           int total_threads) {
  opt->rep.IncreaseParallelism(total_threads);
}

void crocksdb_options_optimize_for_point_lookup(crocksdb_options_t* opt,
                                                uint64_t block_cache_size_mb) {
  opt->rep.OptimizeForPointLookup(block_cache_size_mb);
}

void crocksdb_options_optimize_level_style_compaction(
    crocksdb_options_t* opt, uint64_t memtable_memory_budget) {
  opt->rep.OptimizeLevelStyleCompaction(memtable_memory_budget);
}

void crocksdb_options_optimize_universal_style_compaction(
    crocksdb_options_t* opt, uint64_t memtable_memory_budget) {
  opt->rep.OptimizeUniversalStyleCompaction(memtable_memory_budget);
}

void crocksdb_options_set_compaction_filter(
    crocksdb_options_t* opt, crocksdb_compactionfilter_t* filter) {
  opt->rep.compaction_filter = filter;
}

void crocksdb_options_set_compaction_filter_factory(
    crocksdb_options_t* opt, crocksdb_compactionfilterfactory_t* factory) {
  opt->rep.compaction_filter_factory =
      std::shared_ptr<CompactionFilterFactory>(factory);
}

void crocksdb_options_set_comparator(crocksdb_options_t* opt,
                                     crocksdb_comparator_t* cmp) {
  opt->rep.comparator = cmp;
}

void crocksdb_options_set_merge_operator(
    crocksdb_options_t* opt, crocksdb_mergeoperator_t* merge_operator) {
  opt->rep.merge_operator = std::shared_ptr<MergeOperator>(merge_operator);
}

void crocksdb_options_set_create_if_missing(TitanDBOptions* opt,
                                            unsigned char v) {
  opt->create_if_missing = v;
}

void crocksdb_options_set_create_missing_column_families(TitanDBOptions* opt,
                                                         unsigned char v) {
  opt->create_missing_column_families = v;
}

void crocksdb_options_set_error_if_exists(TitanDBOptions* opt,
                                          unsigned char v) {
  opt->error_if_exists = v;
}

void crocksdb_options_set_paranoid_checks(TitanDBOptions* opt,
                                          unsigned char v) {
  opt->paranoid_checks = v;
}

void crocksdb_options_set_env(TitanDBOptions* opt, Env* env) { opt->env = env; }

crocksdb_logger_t* crocksdb_logger_create(void* rep, void (*destructor_)(void*),
                                          crocksdb_logger_logv_cb logv) {
  crocksdb_logger_t* logger = new crocksdb_logger_t;
  crocksdb_logger_impl_t* li = new crocksdb_logger_impl_t;
  li->rep = rep;
  li->destructor_ = destructor_;
  li->logv_internal_ = logv;
  logger->rep = std::shared_ptr<Logger>(li);
  return logger;
}

void crocksdb_options_set_info_log(TitanDBOptions* opt, crocksdb_logger_t* l) {
  opt->info_log = l->rep;
}

void crocksdb_options_set_info_log_level(TitanDBOptions* opt, InfoLogLevel v) {
  opt->info_log_level = v;
}

void crocksdb_options_set_db_write_buffer_size(TitanDBOptions* opt, size_t s) {
  opt->db_write_buffer_size = s;
}

void crocksdb_options_set_write_buffer_size(crocksdb_options_t* opt, size_t s) {
  opt->rep.write_buffer_size = s;
}

size_t crocksdb_options_get_write_buffer_size(crocksdb_options_t* opt) {
  return opt->rep.write_buffer_size;
}

void crocksdb_options_set_max_open_files(TitanDBOptions* opt, int n) {
  opt->max_open_files = n;
}

void crocksdb_options_set_max_total_wal_size(TitanDBOptions* opt, uint64_t n) {
  opt->max_total_wal_size = n;
}

void crocksdb_options_set_target_file_size_base(crocksdb_options_t* opt,
                                                uint64_t n) {
  opt->rep.target_file_size_base = n;
}

uint64_t crocksdb_options_get_target_file_size_base(
    const crocksdb_options_t* opt) {
  return opt->rep.target_file_size_base;
}

void crocksdb_options_set_target_file_size_multiplier(crocksdb_options_t* opt,
                                                      int n) {
  opt->rep.target_file_size_multiplier = n;
}

void crocksdb_options_set_max_bytes_for_level_base(crocksdb_options_t* opt,
                                                   uint64_t n) {
  opt->rep.max_bytes_for_level_base = n;
}

uint64_t crocksdb_options_get_max_bytes_for_level_base(
    crocksdb_options_t* opt) {
  return opt->rep.max_bytes_for_level_base;
}

void crocksdb_options_set_level_compaction_dynamic_level_bytes(
    crocksdb_options_t* opt, unsigned char v) {
  opt->rep.level_compaction_dynamic_level_bytes = v;
}

unsigned char crocksdb_options_get_level_compaction_dynamic_level_bytes(
    const crocksdb_options_t* options) {
  return options->rep.level_compaction_dynamic_level_bytes;
}

void crocksdb_options_set_max_bytes_for_level_multiplier(
    crocksdb_options_t* opt, double n) {
  opt->rep.max_bytes_for_level_multiplier = n;
}

double crocksdb_options_get_max_bytes_for_level_multiplier(
    crocksdb_options_t* opt) {
  return opt->rep.max_bytes_for_level_multiplier;
}

void crocksdb_options_set_max_compaction_bytes(crocksdb_options_t* opt,
                                               uint64_t n) {
  opt->rep.max_compaction_bytes = n;
}

uint64_t crocksdb_options_get_max_compaction_bytes(crocksdb_options_t* opt) {
  return opt->rep.max_compaction_bytes;
}

void crocksdb_options_set_max_bytes_for_level_multiplier_additional(
    crocksdb_options_t* opt, int* level_values, size_t num_levels) {
  opt->rep.max_bytes_for_level_multiplier_additional.resize(num_levels);
  for (size_t i = 0; i < num_levels; ++i) {
    opt->rep.max_bytes_for_level_multiplier_additional[i] = level_values[i];
  }
}

crocksdb_sst_partitioner_factory_t*
crocksdb_options_get_sst_partitioner_factory(crocksdb_options_t* opt) {
  crocksdb_sst_partitioner_factory_t* factory =
      new crocksdb_sst_partitioner_factory_t;
  factory->rep = opt->rep.sst_partitioner_factory;
  return factory;
}

void crocksdb_options_set_sst_partitioner_factory(
    crocksdb_options_t* opt, crocksdb_sst_partitioner_factory_t* factory) {
  opt->rep.sst_partitioner_factory = factory->rep;
}

void crocksdb_options_set_statistics(TitanDBOptions* opt,
                                     crocksdb_statistics_t* v) {
  if (v) {
    opt->statistics = v->statistics;
  } else {
    opt->statistics = nullptr;
  }
}

void crocksdb_options_set_num_levels(crocksdb_options_t* opt, int n) {
  opt->rep.num_levels = n;
}

int crocksdb_options_get_num_levels(crocksdb_options_t* opt) {
  return opt->rep.num_levels;
}

void crocksdb_options_set_level0_file_num_compaction_trigger(
    crocksdb_options_t* opt, int n) {
  opt->rep.level0_file_num_compaction_trigger = n;
}

int crocksdb_options_get_level0_file_num_compaction_trigger(
    crocksdb_options_t* opt) {
  return opt->rep.level0_file_num_compaction_trigger;
}

void crocksdb_options_set_level0_slowdown_writes_trigger(
    crocksdb_options_t* opt, int n) {
  opt->rep.level0_slowdown_writes_trigger = n;
}

int crocksdb_options_get_level0_slowdown_writes_trigger(
    crocksdb_options_t* opt) {
  return opt->rep.level0_slowdown_writes_trigger;
}

void crocksdb_options_set_level0_stop_writes_trigger(crocksdb_options_t* opt,
                                                     int n) {
  opt->rep.level0_stop_writes_trigger = n;
}

int crocksdb_options_get_level0_stop_writes_trigger(crocksdb_options_t* opt) {
  return opt->rep.level0_stop_writes_trigger;
}

void crocksdb_options_set_wal_recovery_mode(TitanDBOptions* opt,
                                            WALRecoveryMode mode) {
  opt->wal_recovery_mode = mode;
}

void crocksdb_options_set_compression(crocksdb_options_t* opt,
                                      CompressionType t) {
  opt->rep.compression = t;
}

CompressionType crocksdb_options_get_compression(crocksdb_options_t* opt) {
  return opt->rep.compression;
}

void crocksdb_options_set_compression_per_level(crocksdb_options_t* opt,
                                                CompressionType* level_values,
                                                size_t num_levels) {
  opt->rep.compression_per_level.resize(num_levels);
  for (size_t i = 0; i < num_levels; ++i) {
    opt->rep.compression_per_level[i] = level_values[i];
  }
}

size_t crocksdb_options_get_compression_level_number(crocksdb_options_t* opt) {
  return opt->rep.compression_per_level.size();
}

void crocksdb_options_get_compression_per_level(crocksdb_options_t* opt,
                                                CompressionType* level_values) {
  for (size_t i = 0; i < opt->rep.compression_per_level.size(); i++) {
    level_values[i] = opt->rep.compression_per_level[i];
  }
}

void crocksdb_options_set_compression_options(crocksdb_options_t* opt,
                                              int w_bits, int level,
                                              int strategy, int max_dict_bytes,
                                              int zstd_max_train_bytes) {
  opt->rep.compression_opts.window_bits = w_bits;
  opt->rep.compression_opts.level = level;
  opt->rep.compression_opts.strategy = strategy;
  opt->rep.compression_opts.max_dict_bytes = max_dict_bytes;
  opt->rep.compression_opts.zstd_max_train_bytes = zstd_max_train_bytes;
}

void crocksdb_options_set_bottommost_compression_options(
    crocksdb_options_t* opt, int w_bits, int level, int strategy,
    int max_dict_bytes, int zstd_max_train_bytes) {
  opt->rep.bottommost_compression_opts.window_bits = w_bits;
  opt->rep.bottommost_compression_opts.level = level;
  opt->rep.bottommost_compression_opts.strategy = strategy;
  opt->rep.bottommost_compression_opts.max_dict_bytes = max_dict_bytes;
  opt->rep.bottommost_compression_opts.zstd_max_train_bytes =
      zstd_max_train_bytes;
  opt->rep.bottommost_compression_opts.enabled = true;
}

void crocksdb_options_set_use_direct_reads(TitanDBOptions* opt,
                                           unsigned char v) {
  opt->use_direct_reads = v;
}

void crocksdb_options_set_use_direct_io_for_flush_and_compaction(
    TitanDBOptions* opt, unsigned char v) {
  opt->use_direct_io_for_flush_and_compaction = v;
}

void crocksdb_options_set_prefix_extractor(
    crocksdb_options_t* opt, crocksdb_slicetransform_t* prefix_extractor) {
  opt->rep.prefix_extractor.reset(prefix_extractor);
}

void crocksdb_options_set_optimize_filters_for_hits(crocksdb_options_t* opt,
                                                    unsigned char v) {
  opt->rep.optimize_filters_for_hits = v;
}

void crocksdb_options_set_memtable_insert_with_hint_prefix_extractor(
    crocksdb_options_t* opt, crocksdb_slicetransform_t* prefix_extractor) {
  opt->rep.memtable_insert_with_hint_prefix_extractor.reset(prefix_extractor);
}

void crocksdb_options_set_use_fsync(TitanDBOptions* opt, int use_fsync) {
  opt->use_fsync = use_fsync;
}

void crocksdb_options_add_db_paths(TitanDBOptions* opt, Slice db_path,
                                   uint64_t target_size) {
  opt->db_paths.emplace_back(DbPath(db_path.ToString(), target_size));
}

size_t crocksdb_options_get_db_paths_num(crocksdb_options_t* opt) {
  return opt->rep.db_paths.size();
}

const char* crocksdb_options_get_db_path(crocksdb_options_t* opt,
                                         size_t index) {
  return opt->rep.db_paths[index].path.data();
}

uint64_t crocksdb_options_get_path_target_size(crocksdb_options_t* opt,
                                               size_t index) {
  return opt->rep.db_paths[index].target_size;
}

void crocksdb_options_set_db_log_dir(TitanDBOptions* opt, Slice db_log_dir) {
  opt->db_log_dir = db_log_dir.ToString();
}

void crocksdb_options_set_wal_dir(TitanDBOptions* opt, Slice v) {
  opt->wal_dir = v.ToString();
}

void crocksdb_options_set_wal_ttl_seconds(TitanDBOptions* opt, uint64_t ttl) {
  opt->WAL_ttl_seconds = ttl;
}

void crocksdb_options_set_wal_size_limit_mb(TitanDBOptions* opt,
                                            uint64_t limit) {
  opt->WAL_size_limit_MB = limit;
}

void crocksdb_options_set_manifest_preallocation_size(TitanDBOptions* opt,
                                                      size_t v) {
  opt->manifest_preallocation_size = v;
}

void crocksdb_options_set_allow_mmap_reads(TitanDBOptions* opt,
                                           unsigned char v) {
  opt->allow_mmap_reads = v;
}

void crocksdb_options_set_allow_mmap_writes(TitanDBOptions* opt,
                                            unsigned char v) {
  opt->allow_mmap_writes = v;
}

void crocksdb_options_set_is_fd_close_on_exec(TitanDBOptions* opt,
                                              unsigned char v) {
  opt->is_fd_close_on_exec = v;
}

void crocksdb_options_set_stats_dump_period_sec(TitanDBOptions* opt,
                                                unsigned int v) {
  opt->stats_dump_period_sec = v;
}

void crocksdb_options_set_advise_random_on_open(TitanDBOptions* opt,
                                                unsigned char v) {
  opt->advise_random_on_open = v;
}

void crocksdb_options_set_access_hint_on_compaction_start(
    TitanDBOptions* opt, DBOptions::AccessHint v) {
  opt->access_hint_on_compaction_start = v;
}

void crocksdb_options_set_use_adaptive_mutex(TitanDBOptions* opt,
                                             unsigned char v) {
  opt->use_adaptive_mutex = v;
}

void crocksdb_options_set_bytes_per_sync(TitanDBOptions* opt, uint64_t v) {
  opt->bytes_per_sync = v;
}

void crocksdb_options_set_enable_pipelined_write(TitanDBOptions* opt,
                                                 unsigned char v) {
  opt->enable_pipelined_write = v;
}

void crocksdb_options_set_enable_pipelined_commit(TitanDBOptions* opt,
                                                  unsigned char v) {
  opt->enable_pipelined_commit = v;
}

void crocksdb_options_set_unordered_write(TitanDBOptions* opt,
                                          unsigned char v) {
  opt->unordered_write = v;
}

void crocksdb_options_set_allow_concurrent_memtable_write(TitanDBOptions* opt,
                                                          unsigned char v) {
  opt->allow_concurrent_memtable_write = v;
}

void crocksdb_options_set_manual_wal_flush(TitanDBOptions* opt,
                                           unsigned char v) {
  opt->manual_wal_flush = v;
}

void crocksdb_options_set_enable_write_thread_adaptive_yield(
    TitanDBOptions* opt, unsigned char v) {
  opt->enable_write_thread_adaptive_yield = v;
}

void crocksdb_options_set_max_sequential_skip_in_iterations(
    crocksdb_options_t* opt, uint64_t v) {
  opt->rep.max_sequential_skip_in_iterations = v;
}

void crocksdb_options_set_max_write_buffer_number(crocksdb_options_t* opt,
                                                  int n) {
  opt->rep.max_write_buffer_number = n;
}

int crocksdb_options_get_max_write_buffer_number(crocksdb_options_t* opt) {
  return opt->rep.max_write_buffer_number;
}

void crocksdb_options_set_min_write_buffer_number_to_merge(
    crocksdb_options_t* opt, int n) {
  opt->rep.min_write_buffer_number_to_merge = n;
}

int crocksdb_options_get_min_write_buffer_number_to_merge(
    crocksdb_options_t* opt) {
  return opt->rep.min_write_buffer_number_to_merge;
}

void crocksdb_options_set_max_write_buffer_number_to_maintain(
    crocksdb_options_t* opt, int n) {
  opt->rep.max_write_buffer_number_to_maintain = n;
}

void crocksdb_options_set_max_background_jobs(TitanDBOptions* opt, int n) {
  opt->max_background_jobs = n;
}

int crocksdb_options_get_max_background_jobs(const TitanDBOptions* opt) {
  return opt->max_background_jobs;
}

void crocksdb_options_set_max_background_compactions(TitanDBOptions* opt,
                                                     int n) {
  opt->max_background_compactions = n;
}

int crocksdb_options_get_max_background_compactions(const TitanDBOptions* opt) {
  return opt->max_background_compactions;
}

void crocksdb_options_set_max_background_flushes(TitanDBOptions* opt, int n) {
  opt->max_background_flushes = n;
}

int crocksdb_options_get_max_background_flushes(const TitanDBOptions* opt) {
  return opt->max_background_flushes;
}

void crocksdb_options_set_max_log_file_size(TitanDBOptions* opt, size_t v) {
  opt->max_log_file_size = v;
}

void crocksdb_options_set_log_file_time_to_roll(TitanDBOptions* opt, size_t v) {
  opt->log_file_time_to_roll = v;
}

void crocksdb_options_set_keep_log_file_num(TitanDBOptions* opt, size_t v) {
  opt->keep_log_file_num = v;
}

void crocksdb_options_set_recycle_log_file_num(TitanDBOptions* opt, size_t v) {
  opt->recycle_log_file_num = v;
}

void crocksdb_options_set_soft_rate_limit(crocksdb_options_t* opt, double v) {
  opt->rep.soft_rate_limit = v;
}

void crocksdb_options_set_hard_rate_limit(crocksdb_options_t* opt, double v) {
  opt->rep.hard_rate_limit = v;
}

void crocksdb_options_set_soft_pending_compaction_bytes_limit(
    crocksdb_options_t* opt, size_t v) {
  opt->rep.soft_pending_compaction_bytes_limit = v;
}

size_t crocksdb_options_get_soft_pending_compaction_bytes_limit(
    crocksdb_options_t* opt) {
  return opt->rep.soft_pending_compaction_bytes_limit;
}

void crocksdb_options_set_hard_pending_compaction_bytes_limit(
    crocksdb_options_t* opt, size_t v) {
  opt->rep.hard_pending_compaction_bytes_limit = v;
}

size_t crocksdb_options_get_hard_pending_compaction_bytes_limit(
    crocksdb_options_t* opt) {
  return opt->rep.hard_pending_compaction_bytes_limit;
}

void crocksdb_options_set_rate_limit_delay_max_milliseconds(
    crocksdb_options_t* opt, unsigned int v) {
  opt->rep.rate_limit_delay_max_milliseconds = v;
}

void crocksdb_options_set_max_manifest_file_size(TitanDBOptions* opt,
                                                 uint64_t v) {
  opt->max_manifest_file_size = v;
}

void crocksdb_options_set_table_cache_numshardbits(TitanDBOptions* opt, int v) {
  opt->table_cache_numshardbits = v;
}

void crocksdb_options_set_writable_file_max_buffer_size(TitanDBOptions* opt,
                                                        size_t v) {
  opt->writable_file_max_buffer_size = v;
}

void crocksdb_options_set_arena_block_size(crocksdb_options_t* opt, size_t v) {
  opt->rep.arena_block_size = v;
}

void crocksdb_options_set_disable_auto_compactions(crocksdb_options_t* opt,
                                                   int disable) {
  opt->rep.disable_auto_compactions = disable;
}

int crocksdb_options_get_disable_auto_compactions(
    const crocksdb_options_t* opt) {
  return opt->rep.disable_auto_compactions;
}

void crocksdb_options_set_disable_write_stall(crocksdb_options_t* opt,
                                              unsigned char disable) {
  opt->rep.disable_write_stall = disable;
}

unsigned char crocksdb_options_get_disable_write_stall(
    const crocksdb_options_t* opt) {
  return opt->rep.disable_write_stall;
}

void crocksdb_options_set_delete_obsolete_files_period_micros(
    TitanDBOptions* opt, uint64_t v) {
  opt->delete_obsolete_files_period_micros = v;
}

void crocksdb_options_prepare_for_bulk_load(crocksdb_options_t* opt) {
  opt->rep.PrepareForBulkLoad();
}

void crocksdb_options_set_memtable_vector_rep(crocksdb_options_t* opt) {
  opt->rep.memtable_factory.reset(new rocksdb::VectorRepFactory);
}

void crocksdb_options_set_memtable_prefix_bloom_size_ratio(
    crocksdb_options_t* opt, double v) {
  opt->rep.memtable_prefix_bloom_size_ratio = v;
}

void crocksdb_options_set_memtable_huge_page_size(crocksdb_options_t* opt,
                                                  size_t v) {
  opt->rep.memtable_huge_page_size = v;
}
const char* crocksdb_options_get_memtable_factory_name(
    crocksdb_options_t* opt) {
  if (!opt->rep.memtable_factory) {
    return nullptr;
  }
  return opt->rep.memtable_factory->Name();
}

void crocksdb_options_set_hash_skip_list_rep(
    crocksdb_options_t* opt, size_t bucket_count, int32_t skiplist_height,
    int32_t skiplist_branching_factor) {
  rocksdb::MemTableRepFactory* factory = rocksdb::NewHashSkipListRepFactory(
      bucket_count, skiplist_height, skiplist_branching_factor);
  opt->rep.memtable_factory.reset(factory);
}

void crocksdb_options_set_hash_link_list_rep(crocksdb_options_t* opt,
                                             size_t bucket_count) {
  opt->rep.memtable_factory.reset(
      rocksdb::NewHashLinkListRepFactory(bucket_count));
}

void crocksdb_options_set_doubly_skip_list_rep(crocksdb_options_t* opt) {
  rocksdb::MemTableRepFactory* factory = new rocksdb::DoublySkipListFactory();
  opt->rep.memtable_factory.reset(factory);
}

void crocksdb_options_set_plain_table_factory(crocksdb_options_t* opt,
                                              uint32_t user_key_len,
                                              int bloom_bits_per_key,
                                              double hash_table_ratio,
                                              size_t index_sparseness) {
  rocksdb::PlainTableOptions options;
  options.user_key_len = user_key_len;
  options.bloom_bits_per_key = bloom_bits_per_key;
  options.hash_table_ratio = hash_table_ratio;
  options.index_sparseness = index_sparseness;

  rocksdb::TableFactory* factory = rocksdb::NewPlainTableFactory(options);
  opt->rep.table_factory.reset(factory);
}

void crocksdb_options_set_max_successive_merges(crocksdb_options_t* opt,
                                                size_t v) {
  opt->rep.max_successive_merges = v;
}

void crocksdb_options_set_bloom_locality(crocksdb_options_t* opt, uint32_t v) {
  opt->rep.bloom_locality = v;
}

void crocksdb_options_set_inplace_update_support(crocksdb_options_t* opt,
                                                 unsigned char v) {
  opt->rep.inplace_update_support = v;
}

void crocksdb_options_set_inplace_update_num_locks(crocksdb_options_t* opt,
                                                   size_t v) {
  opt->rep.inplace_update_num_locks = v;
}

void crocksdb_options_set_report_bg_io_stats(crocksdb_options_t* opt, int v) {
  opt->rep.report_bg_io_stats = v;
}

void crocksdb_options_set_compaction_readahead_size(TitanDBOptions* opt,
                                                    size_t v) {
  opt->compaction_readahead_size = v;
}

void crocksdb_options_set_compaction_style(crocksdb_options_t* opt,
                                           CompactionStyle style) {
  opt->rep.compaction_style = style;
}

void crocksdb_options_set_universal_compaction_options(
    crocksdb_options_t* opt, crocksdb_universal_compaction_options_t* uco) {
  opt->rep.compaction_options_universal = *(uco->rep);
}

void crocksdb_options_set_fifo_compaction_options(
    crocksdb_options_t* opt, crocksdb_fifo_compaction_options_t* fifo) {
  opt->rep.compaction_options_fifo = fifo->rep;
}

void crocksdb_options_set_compaction_priority(crocksdb_options_t* opt,
                                              CompactionPri priority) {
  opt->rep.compaction_pri = priority;
}

void crocksdb_options_set_delayed_write_rate(TitanDBOptions* opt,
                                             uint64_t delayed_write_rate) {
  opt->delayed_write_rate = delayed_write_rate;
}

void crocksdb_options_set_force_consistency_checks(crocksdb_options_t* opt,
                                                   unsigned char v) {
  opt->rep.force_consistency_checks = v;
}

unsigned char crocksdb_options_get_force_consistency_checks(
    crocksdb_options_t* opt) {
  return opt->rep.force_consistency_checks;
}

crocksdb_statistics_t* crocksdb_statistics_create() {
  auto s = new crocksdb_statistics_t;
  s->statistics = CreateDBStatistics();
  return s;
}

void crocksdb_statistics_reset(crocksdb_statistics_t* s) {
  s->statistics.get()->Reset();
}

void crocksdb_statistics_destroy(crocksdb_statistics_t* ptr) { delete ptr; }

char* crocksdb_statistics_get_string(crocksdb_statistics_t* ptr) {
  rocksdb::Statistics* statistics = ptr->statistics.get();
  return strdup(statistics->ToString().c_str());
}

uint64_t crocksdb_statistics_get_ticker_count(crocksdb_statistics_t* ptr,
                                              uint32_t ticker_type) {
  rocksdb::Statistics* statistics = ptr->statistics.get();
  return statistics->getTickerCount(ticker_type);
}

uint64_t crocksdb_statistics_get_and_reset_ticker_count(
    crocksdb_statistics_t* ptr, uint32_t ticker_type) {
  rocksdb::Statistics* statistics = ptr->statistics.get();
  return statistics->getAndResetTickerCount(ticker_type);
}

char* crocksdb_statistics_get_histogram_string(crocksdb_statistics_t* ptr,
                                               uint32_t type) {
  rocksdb::Statistics* statistics = ptr->statistics.get();
  return strdup(statistics->getHistogramString(type).c_str());
}

void crocksdb_statistics_get_histogram(crocksdb_statistics_t* ptr,
                                       uint32_t type, double* median,
                                       double* percentile95,
                                       double* percentile99, double* average,
                                       double* standard_deviation,
                                       double* max) {
  rocksdb::Statistics* statistics = ptr->statistics.get();
  crocksdb_histogramdata_t data;
  statistics->histogramData(type, &data.rep);
  *median = data.rep.median;
  *percentile95 = data.rep.percentile95;
  *percentile99 = data.rep.percentile99;
  *average = data.rep.average;
  *standard_deviation = data.rep.standard_deviation;
  *max = data.rep.max;
}

void crocksdb_options_set_ratelimiter(TitanDBOptions* opt,
                                      crocksdb_ratelimiter_t* limiter) {
  opt->rate_limiter = limiter->rep;
}

void crocksdb_options_set_vector_memtable_factory(crocksdb_options_t* opt,
                                                  uint64_t reserved_bytes) {
  opt->rep.memtable_factory.reset(new VectorRepFactory(reserved_bytes));
}

void crocksdb_options_set_atomic_flush(TitanDBOptions* opt,
                                       unsigned char enable) {
  opt->atomic_flush = enable;
}

unsigned char crocksdb_load_latest_options(
    const char* dbpath, Env* env, crocksdb_options_t* db_options,
    crocksdb_column_family_descriptor*** cf_descs, size_t* cf_descs_len,
    unsigned char ignore_unknown_options, Status* s) {
  std::vector<ColumnFamilyDescriptor> tmp_cf_descs;
  *s = rocksdb::LoadLatestOptions(dbpath, env, &db_options->rep, &tmp_cf_descs,
                                  ignore_unknown_options);

  if (!s->ok()) {
    return false;
  }

  *cf_descs_len = tmp_cf_descs.size();
  (*cf_descs) = (crocksdb_column_family_descriptor**)malloc(
      sizeof(crocksdb_column_family_descriptor*) * (*cf_descs_len));
  for (std::size_t i = 0; i < *cf_descs_len; ++i) {
    (*cf_descs)[i] =
        new crocksdb_column_family_descriptor{std::move(tmp_cf_descs[i])};
  }

  return true;
}

crocksdb_ratelimiter_t* crocksdb_ratelimiter_create(int64_t rate_bytes_per_sec,
                                                    int64_t refill_period_us,
                                                    int32_t fairness) {
  crocksdb_ratelimiter_t* rate_limiter = new crocksdb_ratelimiter_t;
  rate_limiter->rep = std::shared_ptr<RateLimiter>(
      NewGenericRateLimiter(rate_bytes_per_sec, refill_period_us, fairness));
  return rate_limiter;
}

crocksdb_ratelimiter_t* crocksdb_ratelimiter_create_with_auto_tuned(
    int64_t rate_bytes_per_sec, int64_t refill_period_us, int32_t fairness,
    rocksdb::RateLimiter::Mode mode, unsigned char auto_tuned) {
  crocksdb_ratelimiter_t* rate_limiter = new crocksdb_ratelimiter_t;
  rate_limiter->rep = std::shared_ptr<RateLimiter>(NewGenericRateLimiter(
      rate_bytes_per_sec, refill_period_us, fairness, mode, auto_tuned));
  return rate_limiter;
}

crocksdb_ratelimiter_t*
crocksdb_writeampbasedratelimiter_create_with_auto_tuned(
    int64_t rate_bytes_per_sec, int64_t refill_period_us, int32_t fairness,
    rocksdb::RateLimiter::Mode mode, unsigned char auto_tuned) {
  crocksdb_ratelimiter_t* rate_limiter = new crocksdb_ratelimiter_t;
  rate_limiter->rep = std::shared_ptr<RateLimiter>(NewWriteAmpBasedRateLimiter(
      rate_bytes_per_sec, refill_period_us, fairness, mode, auto_tuned));
  return rate_limiter;
}

void crocksdb_ratelimiter_destroy(crocksdb_ratelimiter_t* limiter) {
  delete limiter;
}

void crocksdb_ratelimiter_set_bytes_per_second(crocksdb_ratelimiter_t* limiter,
                                               int64_t rate_bytes_per_sec) {
  limiter->rep->SetBytesPerSecond(rate_bytes_per_sec);
}

void crocksdb_ratelimiter_set_auto_tuned(crocksdb_ratelimiter_t* limiter,
                                         unsigned char auto_tuned) {
  limiter->rep->SetAutoTuned(auto_tuned);
}

int64_t crocksdb_ratelimiter_get_singleburst_bytes(
    crocksdb_ratelimiter_t* limiter) {
  return limiter->rep->GetSingleBurstBytes();
}

void crocksdb_ratelimiter_request(crocksdb_ratelimiter_t* limiter,
                                  int64_t bytes, Env::IOPriority pri,
                                  RateLimiter::OpType ty) {
  limiter->rep->Request(bytes, pri, nullptr, ty);
}

int64_t crocksdb_ratelimiter_get_total_bytes_through(
    crocksdb_ratelimiter_t* limiter, rocksdb::Env::IOPriority pri) {
  return limiter->rep->GetTotalBytesThrough(pri);
}

int64_t crocksdb_ratelimiter_get_bytes_per_second(
    crocksdb_ratelimiter_t* limiter) {
  return limiter->rep->GetBytesPerSecond();
}

unsigned char crocksdb_ratelimiter_get_auto_tuned(
    crocksdb_ratelimiter_t* limiter) {
  return limiter->rep->GetAutoTuned();
}

int64_t crocksdb_ratelimiter_get_total_requests(crocksdb_ratelimiter_t* limiter,
                                                rocksdb::Env::IOPriority pri) {
  return limiter->rep->GetTotalRequests(pri);
}

/*
TODO:
DB::OpenForReadOnly
DB::KeyMayExist
DB::GetOptions
DB::GetSortedWalFiles
DB::GetLatestSequenceNumber
DB::GetUpdatesSince
DB::GetDbIdentity
DB::RunManualCompaction
custom cache
table_properties_collectors
*/

crocksdb_compactionfilter_t* crocksdb_compactionfilter_create(
    void* state, void (*destructor)(void*),
    CompactionFilter::Decision (*filter)(
        void*, int level, const char* key, size_t key_length, uint64_t seqno,
        CompactionFilter::ValueType value_type, const char* existing_value,
        size_t value_length, char** new_value, size_t* new_value_length,
        char** skip_until, size_t* skip_until_length),
    const char* (*name)(void*)) {
  crocksdb_compactionfilter_t* result = new crocksdb_compactionfilter_t;
  result->state_ = state;
  result->destructor_ = destructor;
  result->filter_ = filter;
  result->name_ = name;
  return result;
}

void crocksdb_compactionfilter_destroy(crocksdb_compactionfilter_t* filter) {
  delete filter;
}

unsigned char crocksdb_compactionfiltercontext_is_full_compaction(
    crocksdb_compactionfiltercontext_t* context) {
  return context->rep.is_full_compaction;
}

unsigned char crocksdb_compactionfiltercontext_is_manual_compaction(
    crocksdb_compactionfiltercontext_t* context) {
  return context->rep.is_manual_compaction;
}

unsigned char crocksdb_compactionfiltercontext_is_bottommost_level(
    crocksdb_compactionfiltercontext_t* context) {
  return context->rep.is_bottommost_level;
}

void crocksdb_compactionfiltercontext_file_numbers(
    crocksdb_compactionfiltercontext_t* context, const uint64_t** buffer,
    size_t* len) {
  *buffer = context->rep.file_numbers.data();
  *len = context->rep.file_numbers.size();
}

const TableProperties* crocksdb_compactionfiltercontext_table_properties(
    crocksdb_compactionfiltercontext_t* context, size_t offset) {
  return context->rep.table_properties[offset].get();
}

const char* crocksdb_compactionfiltercontext_start_key(
    crocksdb_compactionfiltercontext_t* context, size_t* key_len) {
  const Slice& result = context->rep.start_key;
  *key_len = result.size();
  return result.data();
}

const char* crocksdb_compactionfiltercontext_end_key(
    crocksdb_compactionfiltercontext_t* context, size_t* key_len) {
  const Slice& result = context->rep.end_key;
  *key_len = result.size();
  return result.data();
}

crocksdb_compactionfilterfactory_t* crocksdb_compactionfilterfactory_create(
    void* state, void (*destructor)(void*),
    crocksdb_compactionfilter_t* (*create_compaction_filter)(
        void*, crocksdb_compactionfiltercontext_t* context),
    unsigned char (*should_filter_table_file_creation)(
        void*, TableFileCreationReason reason),
    const char* (*name)(void*)) {
  crocksdb_compactionfilterfactory_t* result =
      new crocksdb_compactionfilterfactory_t;
  result->state_ = state;
  result->destructor_ = destructor;
  result->create_compaction_filter_ = create_compaction_filter;
  result->should_filter_table_file_creation_ =
      should_filter_table_file_creation;
  result->name_ = name;
  return result;
}

void crocksdb_compactionfilterfactory_destroy(
    crocksdb_compactionfilterfactory_t* factory) {
  delete factory;
}

crocksdb_comparator_t* crocksdb_comparator_create(
    void* state, void (*destructor)(void*),
    int (*compare)(void*, const char* a, size_t alen, const char* b,
                   size_t blen),
    const char* (*name)(void*)) {
  crocksdb_comparator_t* result = new crocksdb_comparator_t;
  result->state_ = state;
  result->destructor_ = destructor;
  result->compare_ = compare;
  result->name_ = name;
  return result;
}

void crocksdb_comparator_destroy(crocksdb_comparator_t* cmp) { delete cmp; }

crocksdb_filterpolicy_t* crocksdb_filterpolicy_create(
    void* state, void (*destructor)(void*),
    char* (*create_filter)(void*, const char* const* key_array,
                           const size_t* key_length_array, int num_keys,
                           size_t* filter_length),
    unsigned char (*key_may_match)(void*, const char* key, size_t length,
                                   const char* filter, size_t filter_length),
    void (*delete_filter)(void*, const char* filter, size_t filter_length),
    const char* (*name)(void*)) {
  crocksdb_filterpolicy_t* result = new crocksdb_filterpolicy_t;
  result->state_ = state;
  result->destructor_ = destructor;
  result->create_ = create_filter;
  result->key_match_ = key_may_match;
  result->delete_filter_ = delete_filter;
  result->name_ = name;
  return result;
}

void crocksdb_filterpolicy_destroy(crocksdb_filterpolicy_t* filter) {
  delete filter;
}

crocksdb_filterpolicy_t* crocksdb_filterpolicy_create_bloom_format(
    int bits_per_key, bool original_format) {
  // Make a crocksdb_filterpolicy_t, but override all of its methods so
  // they delegate to a NewBloomFilterPolicy() instead of user
  // supplied C functions.
  struct Wrapper : public crocksdb_filterpolicy_t {
    const FilterPolicy* rep_;
    ~Wrapper() { delete rep_; }
    const char* Name() const override { return rep_->Name(); }
    void CreateFilter(const Slice* keys, int n,
                      std::string* dst) const override {
      return rep_->CreateFilter(keys, n, dst);
    }
    bool KeyMayMatch(const Slice& key, const Slice& filter) const override {
      return rep_->KeyMayMatch(key, filter);
    }
    virtual FilterBitsBuilder* GetFilterBitsBuilder() const override {
      return rep_->GetFilterBitsBuilder();
    }
    virtual FilterBitsReader* GetFilterBitsReader(
        const Slice& contents) const override {
      return rep_->GetFilterBitsReader(contents);
    }
    static void DoNothing(void*) {}
  };
  Wrapper* wrapper = new Wrapper;
  wrapper->rep_ = NewBloomFilterPolicy(bits_per_key, original_format);
  wrapper->state_ = nullptr;
  wrapper->delete_filter_ = nullptr;
  wrapper->destructor_ = &Wrapper::DoNothing;
  return wrapper;
}

crocksdb_filterpolicy_t* crocksdb_filterpolicy_create_bloom_full(
    int bits_per_key) {
  return crocksdb_filterpolicy_create_bloom_format(bits_per_key, false);
}

crocksdb_filterpolicy_t* crocksdb_filterpolicy_create_bloom(int bits_per_key) {
  return crocksdb_filterpolicy_create_bloom_format(bits_per_key, true);
}

crocksdb_mergeoperator_t* crocksdb_mergeoperator_create(
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
    const char* (*name)(void*)) {
  crocksdb_mergeoperator_t* result = new crocksdb_mergeoperator_t;
  result->state_ = state;
  result->destructor_ = destructor;
  result->full_merge_ = full_merge;
  result->partial_merge_ = partial_merge;
  result->delete_value_ = delete_value;
  result->name_ = name;
  return result;
}

void crocksdb_mergeoperator_destroy(crocksdb_mergeoperator_t* merge_operator) {
  delete merge_operator;
}

struct TableFilterCtx {
  TableFilterCtx(void* ctx, void (*destroy)(void*))
      : ctx_(ctx), destroy_(destroy) {}
  ~TableFilterCtx() { destroy_(ctx_); }

  void* ctx_;
  void (*destroy_)(void*);
};

struct TableFilter {
  // After passing TableFilter to ReadOptions, ReadOptions will be copyed
  // several times, so we need use shared_ptr to control the ctx_ resource
  // destroy ctx_ only when the last ReadOptions out of its life time.
  TableFilter(void* ctx,
              unsigned char (*table_filter)(void*, const TableProperties*),
              void (*destroy)(void*))
      : ctx_(std::make_shared<TableFilterCtx>(ctx, destroy)),
        table_filter_(table_filter) {}

  TableFilter(const TableFilter& f)
      : ctx_(f.ctx_), table_filter_(f.table_filter_) {}

  bool operator()(const TableProperties& prop) {
    return table_filter_(ctx_->ctx_, &prop);
  }

  shared_ptr<TableFilterCtx> ctx_;
  unsigned char (*table_filter_)(void*, const TableProperties*);

 private:
  TableFilter() {}
};

void crocksdb_readoptions_set_table_filter(
    ReadOptions* opt, void* ctx,
    unsigned char (*table_filter)(void*, const TableProperties*),
    void (*destroy)(void*)) {
  opt->table_filter = TableFilter(ctx, table_filter, destroy);
}

void crocksdb_writeoptions_init(WriteOptions* opt) { *opt = WriteOptions(); }

void crocksdb_compactrangeoptions_init(CompactRangeOptions* opt) {
  *opt = CompactRangeOptions();
}

void crocksdb_flushoptions_init(FlushOptions* opt) { *opt = FlushOptions(); }

crocksdb_memory_allocator_t* crocksdb_jemalloc_nodump_allocator_create(
    Status* s) {
  crocksdb_memory_allocator_t* allocator = new crocksdb_memory_allocator_t;
  rocksdb::JemallocAllocatorOptions options;
  *s = rocksdb::NewJemallocNodumpAllocator(options, &allocator->rep);
  if (!s->ok()) {
    delete allocator;
    return nullptr;
  }
  return allocator;
}

void crocksdb_memory_allocator_destroy(crocksdb_memory_allocator_t* allocator) {
  delete allocator;
}

crocksdb_lru_cache_options_t* crocksdb_lru_cache_options_create() {
  return new crocksdb_lru_cache_options_t;
}

void crocksdb_lru_cache_options_destroy(crocksdb_lru_cache_options_t* opt) {
  delete opt;
}

void crocksdb_lru_cache_options_set_capacity(crocksdb_lru_cache_options_t* opt,
                                             size_t capacity) {
  opt->rep.capacity = capacity;
}

void crocksdb_lru_cache_options_set_num_shard_bits(
    crocksdb_lru_cache_options_t* opt, int num_shard_bits) {
  opt->rep.num_shard_bits = num_shard_bits;
}

void crocksdb_lru_cache_options_set_strict_capacity_limit(
    crocksdb_lru_cache_options_t* opt, unsigned char strict_capacity_limit) {
  opt->rep.strict_capacity_limit = strict_capacity_limit;
}

void crocksdb_lru_cache_options_set_high_pri_pool_ratio(
    crocksdb_lru_cache_options_t* opt, double high_pri_pool_ratio) {
  opt->rep.high_pri_pool_ratio = high_pri_pool_ratio;
}

void crocksdb_lru_cache_options_set_memory_allocator(
    crocksdb_lru_cache_options_t* opt, crocksdb_memory_allocator_t* allocator) {
  opt->rep.memory_allocator = allocator->rep;
}

crocksdb_cache_t* crocksdb_cache_create_lru(crocksdb_lru_cache_options_t* opt) {
  crocksdb_cache_t* c = new crocksdb_cache_t;
  c->rep = NewLRUCache(opt->rep);
  return c;
}

void crocksdb_cache_destroy(crocksdb_cache_t* cache) { delete cache; }

void crocksdb_cache_set_capacity(crocksdb_cache_t* cache, size_t capacity) {
  cache->rep->SetCapacity(capacity);
}

Env* crocksdb_default_env_create() { return Env::Default(); }

Env* crocksdb_mem_env_create(Env* base) { return rocksdb::NewMemEnv(base); }

struct CTRBlockCipher : public BlockCipher {
  CTRBlockCipher(size_t block_size, const std::string& cipertext)
      : block_size_(block_size), cipertext_(cipertext) {
    assert(block_size == cipertext.size());
  }

  virtual size_t BlockSize() { return block_size_; }

  virtual Status Encrypt(char* data) {
    const char* ciper_ptr = cipertext_.c_str();
    for (size_t i = 0; i < block_size_; i++) {
      data[i] = data[i] ^ ciper_ptr[i];
    }

    return Status::OK();
  }

  virtual Status Decrypt(char* data) {
    Encrypt(data);
    return Status::OK();
  }

 protected:
  std::string cipertext_;
  size_t block_size_;
};

Env* crocksdb_ctr_encrypted_env_create(Env* base_env, const char* ciphertext,
                                       size_t ciphertext_len) {
  auto block_cipher = new CTRBlockCipher(
      ciphertext_len, std::string(ciphertext, ciphertext_len));
  auto encryption_provider = new CTREncryptionProvider(*block_cipher);
  Env* env = NewEncryptedEnv(base_env, encryption_provider);
  return new EncryptedEnvWrapper(env, encryption_provider, block_cipher);
}

void crocksdb_env_set_background_threads(Env* env, int n, Env::Priority pri) {
  env->SetBackgroundThreads(n, pri);
}

void crocksdb_env_join_all_threads(Env* env) { env->WaitForJoin(); }

void crocksdb_env_file_exists(Env* env, Slice path, Status* s) {
  *s = env->FileExists(path.ToString());
}

void crocksdb_env_delete_file(Env* env, Slice path, Status* s) {
  *s = env->DeleteFile(path.ToString());
}

void crocksdb_env_destroy(Env* env) {
  if (env != Env::Default()) {
    delete env;
  }
}

crocksdb_envoptions_t* crocksdb_envoptions_create() {
  crocksdb_envoptions_t* opt = new crocksdb_envoptions_t;
  return opt;
}

void crocksdb_envoptions_destroy(crocksdb_envoptions_t* opt) { delete opt; }

crocksdb_sequential_file_t* crocksdb_sequential_file_create(
    Env* env, Slice path, const crocksdb_envoptions_t* opts, Status* s) {
  std::unique_ptr<SequentialFile> result;
  auto p = path.ToString();
  *s = env->NewSequentialFile(p, &result, opts->rep);
  if (!s->ok()) {
    return nullptr;
  }
  auto file = new crocksdb_sequential_file_t;
  file->rep = result.release();
  return file;
}

size_t crocksdb_sequential_file_read(crocksdb_sequential_file_t* file, size_t n,
                                     char* buf, Status* s) {
  Slice result;
  *s = file->rep->Read(n, &result, buf);
  if (!s->ok()) {
    return 0;
  }
  return result.size();
}

void crocksdb_sequential_file_skip(crocksdb_sequential_file_t* file, size_t n,
                                   Status* s) {
  *s = file->rep->Skip(n);
}

void crocksdb_sequential_file_destroy(crocksdb_sequential_file_t* file) {
  delete file->rep;
  delete file;
}

#ifdef OPENSSL
crocksdb_file_encryption_info_t* crocksdb_file_encryption_info_create() {
  crocksdb_file_encryption_info_t* file_info =
      new crocksdb_file_encryption_info_t;
  file_info->rep = new FileEncryptionInfo;
  return file_info;
}

void crocksdb_file_encryption_info_destroy(
    crocksdb_file_encryption_info_t* file_info) {
  delete file_info->rep;
  delete file_info;
}

EncryptionMethod crocksdb_file_encryption_info_method(
    crocksdb_file_encryption_info_t* file_info) {
  assert(file_info != nullptr);
  assert(file_info->rep != nullptr);
  return file_info->rep->method;
}

const char* crocksdb_file_encryption_info_key(
    crocksdb_file_encryption_info_t* file_info, size_t* keylen) {
  assert(file_info != nullptr);
  assert(file_info->rep != nullptr);
  assert(keylen != nullptr);
  *keylen = file_info->rep->key.size();
  return file_info->rep->key.c_str();
}

const char* crocksdb_file_encryption_info_iv(
    crocksdb_file_encryption_info_t* file_info, size_t* ivlen) {
  assert(file_info != nullptr);
  assert(file_info->rep != nullptr);
  assert(ivlen != nullptr);
  *ivlen = file_info->rep->iv.size();
  return file_info->rep->iv.c_str();
}

void crocksdb_file_encryption_info_set_method(
    crocksdb_file_encryption_info_t* file_info, EncryptionMethod method) {
  assert(file_info != nullptr);
  file_info->rep->method = method;
}

void crocksdb_file_encryption_info_set_key(
    crocksdb_file_encryption_info_t* file_info, const char* key,
    size_t keylen) {
  assert(file_info != nullptr);
  file_info->rep->key = std::string(key, keylen);
}

void crocksdb_file_encryption_info_set_iv(
    crocksdb_file_encryption_info_t* file_info, const char* iv, size_t ivlen) {
  assert(file_info != nullptr);
  file_info->rep->iv = std::string(iv, ivlen);
}

struct crocksdb_encryption_key_manager_impl_t : public KeyManager {
  void* state;
  void (*destructor)(void*);
  crocksdb_encryption_key_manager_get_file_cb get_file;
  crocksdb_encryption_key_manager_new_file_cb new_file;
  crocksdb_encryption_key_manager_delete_file_cb delete_file;
  crocksdb_encryption_key_manager_link_file_cb link_file;

  virtual ~crocksdb_encryption_key_manager_impl_t() { destructor(state); }

  Status GetFile(const std::string& fname,
                 FileEncryptionInfo* file_info) override {
    crocksdb_file_encryption_info_t info;
    info.rep = file_info;
    Status s;
    get_file(state, fname.c_str(), &info, &s);
    return s;
  }

  Status NewFile(const std::string& fname,
                 FileEncryptionInfo* file_info) override {
    crocksdb_file_encryption_info_t info;
    info.rep = file_info;
    Status s;
    new_file(state, fname.c_str(), &info, &s);
    return s;
  }

  Status DeleteFile(const std::string& fname) override {
    Status s;
    delete_file(state, fname.c_str(), &s);
    return s;
  }

  Status LinkFile(const std::string& src_fname,
                  const std::string& dst_fname) override {
    Status s;
    link_file(state, src_fname.c_str(), dst_fname.c_str(), &s);
    return s;
  }
};

crocksdb_encryption_key_manager_t* crocksdb_encryption_key_manager_create(
    void* state, void (*destructor)(void*),
    crocksdb_encryption_key_manager_get_file_cb get_file,
    crocksdb_encryption_key_manager_new_file_cb new_file,
    crocksdb_encryption_key_manager_delete_file_cb delete_file,
    crocksdb_encryption_key_manager_link_file_cb link_file) {
  std::shared_ptr<crocksdb_encryption_key_manager_impl_t> key_manager_impl =
      std::make_shared<crocksdb_encryption_key_manager_impl_t>();
  key_manager_impl->state = state;
  key_manager_impl->destructor = destructor;
  key_manager_impl->get_file = get_file;
  key_manager_impl->new_file = new_file;
  key_manager_impl->delete_file = delete_file;
  key_manager_impl->link_file = link_file;
  crocksdb_encryption_key_manager_t* key_manager =
      new crocksdb_encryption_key_manager_t;
  key_manager->rep = key_manager_impl;
  return key_manager;
}

void crocksdb_encryption_key_manager_destroy(
    crocksdb_encryption_key_manager_t* key_manager) {
  delete key_manager;
}

void crocksdb_encryption_key_manager_get_file(
    crocksdb_encryption_key_manager_t* key_manager, const char* fname,
    crocksdb_file_encryption_info_t* file_info, Status* s) {
  *s = key_manager->rep->GetFile(fname, file_info->rep);
}

void crocksdb_encryption_key_manager_new_file(
    crocksdb_encryption_key_manager_t* key_manager, const char* fname,
    crocksdb_file_encryption_info_t* file_info, Status* s) {
  *s = key_manager->rep->NewFile(fname, file_info->rep);
}

void crocksdb_encryption_key_manager_delete_file(
    crocksdb_encryption_key_manager_t* key_manager, const char* fname,
    Status* s) {
  *s = key_manager->rep->DeleteFile(fname);
}

void crocksdb_encryption_key_manager_link_file(
    crocksdb_encryption_key_manager_t* key_manager, const char* src_fname,
    const char* dst_fname, Status* s) {
  *s = key_manager->rep->LinkFile(src_fname, dst_fname);
}

Env* crocksdb_key_managed_encrypted_env_create(
    Env* base_env, crocksdb_encryption_key_manager_t* key_manager) {
  return NewKeyManagedEncryptedEnv(base_env, key_manager->rep);
}
#endif

struct crocksdb_file_system_inspector_impl_t : public FileSystemInspector {
  void* state;
  void (*destructor)(void*);
  crocksdb_file_system_inspector_read_cb read;
  crocksdb_file_system_inspector_write_cb write;

  virtual ~crocksdb_file_system_inspector_impl_t() { destructor(state); }

  Status Read(size_t len, size_t* allowed) {
    Status s;
    *allowed = read(state, len, &s);
    return s;
  }

  Status Write(size_t len, size_t* allowed) {
    Status s;
    *allowed = write(state, len, &s);
    return s;
  }
};

crocksdb_file_system_inspector_t* crocksdb_file_system_inspector_create(
    void* state, void (*destructor)(void*),
    crocksdb_file_system_inspector_read_cb read,
    crocksdb_file_system_inspector_write_cb write) {
  std::shared_ptr<crocksdb_file_system_inspector_impl_t> inspector_impl =
      std::make_shared<crocksdb_file_system_inspector_impl_t>();
  inspector_impl->state = state;
  inspector_impl->destructor = destructor;
  inspector_impl->read = read;
  inspector_impl->write = write;
  crocksdb_file_system_inspector_t* inspector =
      new crocksdb_file_system_inspector_t;
  inspector->rep = inspector_impl;
  return inspector;
}

void crocksdb_file_system_inspector_destroy(
    crocksdb_file_system_inspector_t* inspector) {
  delete inspector;
}

size_t crocksdb_file_system_inspector_read(
    crocksdb_file_system_inspector_t* inspector, size_t len, Status* s) {
  size_t allowed = 0;
  *s = inspector->rep->Read(len, &allowed);
  return allowed;
}

size_t crocksdb_file_system_inspector_write(
    crocksdb_file_system_inspector_t* inspector, size_t len, Status* s) {
  size_t allowed = 0;
  *s = inspector->rep->Write(len, &allowed);
  return allowed;
}

Env* crocksdb_file_system_inspected_env_create(
    Env* base_env, crocksdb_file_system_inspector_t* inspector) {
  return NewFileSystemInspectedEnv(base_env, inspector->rep);
}

crocksdb_sstfilereader_t* crocksdb_sstfilereader_create(
    const crocksdb_options_t* io_options) {
  auto reader = new crocksdb_sstfilereader_t;
  reader->rep = new SstFileReader(io_options->rep);
  return reader;
}

void crocksdb_sstfilereader_open(crocksdb_sstfilereader_t* reader,
                                 const char* name, Status* s) {
  *s = reader->rep->Open(std::string(name));
}

crocksdb_iterator_t* crocksdb_sstfilereader_new_iterator(
    crocksdb_sstfilereader_t* reader, const ReadOptions* options) {
  auto it = new crocksdb_iterator_t;
  it->rep = reader->rep->NewIterator(*options);
  return it;
}

const TableProperties* crocksdb_sstfilereader_get_table_properties(
    const crocksdb_sstfilereader_t* reader) {
  return reader->rep->GetTableProperties().get();
}

void crocksdb_sstfilereader_verify_checksum(crocksdb_sstfilereader_t* reader,
                                            Status* s) {
  *s = reader->rep->VerifyChecksum();
}

void crocksdb_sstfilereader_destroy(crocksdb_sstfilereader_t* reader) {
  delete reader->rep;
  delete reader;
}

crocksdb_sstfilewriter_t* crocksdb_sstfilewriter_create(
    const crocksdb_envoptions_t* env, const crocksdb_options_t* io_options) {
  crocksdb_sstfilewriter_t* writer = new crocksdb_sstfilewriter_t;
  writer->rep = new SstFileWriter(env->rep, io_options->rep);
  return writer;
}

crocksdb_sstfilewriter_t* crocksdb_sstfilewriter_create_cf(
    const crocksdb_envoptions_t* env, const crocksdb_options_t* io_options,
    crocksdb_column_family_handle_t* column_family) {
  crocksdb_sstfilewriter_t* writer = new crocksdb_sstfilewriter_t;
  writer->rep =
      new SstFileWriter(env->rep, io_options->rep, column_family->rep);
  return writer;
}

void crocksdb_sstfilewriter_open(crocksdb_sstfilewriter_t* writer,
                                 const char* name, Status* s) {
  *s = writer->rep->Open(std::string(name));
}

void crocksdb_sstfilewriter_put(crocksdb_sstfilewriter_t* writer,
                                const char* key, size_t keylen, const char* val,
                                size_t vallen, Status* s) {
  *s = writer->rep->Put(Slice(key, keylen), Slice(val, vallen));
}

void crocksdb_sstfilewriter_merge(crocksdb_sstfilewriter_t* writer,
                                  const char* key, size_t keylen,
                                  const char* val, size_t vallen, Status* s) {
  *s = writer->rep->Merge(Slice(key, keylen), Slice(val, vallen));
}

void crocksdb_sstfilewriter_delete(crocksdb_sstfilewriter_t* writer,
                                   const char* key, size_t keylen, Status* s) {
  *s = writer->rep->Delete(Slice(key, keylen));
}

void crocksdb_sstfilewriter_delete_range(crocksdb_sstfilewriter_t* writer,
                                         const char* begin_key,
                                         size_t begin_keylen,
                                         const char* end_key, size_t end_keylen,
                                         Status* s) {
  *s = writer->rep->DeleteRange(Slice(begin_key, begin_keylen),
                                Slice(end_key, end_keylen));
}

void crocksdb_sstfilewriter_finish(crocksdb_sstfilewriter_t* writer,
                                   crocksdb_externalsstfileinfo_t* info,
                                   Status* s) {
  *s = writer->rep->Finish(&info->rep);
}

uint64_t crocksdb_sstfilewriter_file_size(crocksdb_sstfilewriter_t* writer) {
  return writer->rep->FileSize();
}

void crocksdb_sstfilewriter_destroy(crocksdb_sstfilewriter_t* writer) {
  delete writer->rep;
  delete writer;
}

crocksdb_externalsstfileinfo_t* crocksdb_externalsstfileinfo_create() {
  return new crocksdb_externalsstfileinfo_t;
};

void crocksdb_externalsstfileinfo_destroy(
    crocksdb_externalsstfileinfo_t* info) {
  delete info;
}

const char* crocksdb_externalsstfileinfo_file_path(
    crocksdb_externalsstfileinfo_t* info, size_t* size) {
  *size = info->rep.file_path.size();
  return info->rep.file_path.c_str();
}

const char* crocksdb_externalsstfileinfo_smallest_key(
    crocksdb_externalsstfileinfo_t* info, size_t* size) {
  *size = info->rep.smallest_key.size();
  return info->rep.smallest_key.c_str();
}

const char* crocksdb_externalsstfileinfo_largest_key(
    crocksdb_externalsstfileinfo_t* info, size_t* size) {
  *size = info->rep.largest_key.size();
  return info->rep.largest_key.c_str();
}

uint64_t crocksdb_externalsstfileinfo_sequence_number(
    crocksdb_externalsstfileinfo_t* info) {
  return info->rep.sequence_number;
}

uint64_t crocksdb_externalsstfileinfo_file_size(
    crocksdb_externalsstfileinfo_t* info) {
  return info->rep.file_size;
}

uint64_t crocksdb_externalsstfileinfo_num_entries(
    crocksdb_externalsstfileinfo_t* info) {
  return info->rep.num_entries;
}

void crocksdb_ingestexternalfileoptions_init(IngestExternalFileOptions* opt) {
  *opt = IngestExternalFileOptions();
}

void crocksdb_ingest_external_file(crocksdb_t* db, const char* const* file_list,
                                   const size_t list_len,
                                   const IngestExternalFileOptions* opt,
                                   Status* s) {
  std::vector<std::string> files(list_len);
  for (size_t i = 0; i < list_len; ++i) {
    files[i] = std::string(file_list[i]);
  }
  *s = db->rep->IngestExternalFile(files, *opt);
}

void crocksdb_ingest_external_file_cf(crocksdb_t* db,
                                      crocksdb_column_family_handle_t* handle,
                                      const char* const* file_list,
                                      const size_t list_len,
                                      const IngestExternalFileOptions* opt,
                                      Status* s) {
  std::vector<std::string> files(list_len);
  for (size_t i = 0; i < list_len; ++i) {
    files[i] = std::string(file_list[i]);
  }
  *s = db->rep->IngestExternalFile(handle->rep, files, *opt);
}

unsigned char crocksdb_ingest_external_file_optimized(
    crocksdb_t* db, crocksdb_column_family_handle_t* handle,
    const char* const* file_list, const size_t list_len,
    const IngestExternalFileOptions* opt, Status* s) {
  std::vector<std::string> files(list_len);
  for (size_t i = 0; i < list_len; ++i) {
    files[i] = std::string(file_list[i]);
  }
  bool has_flush = false;
  // If the file being ingested is overlapped with the memtable, it
  // will block writes and wait for flushing, which can cause high
  // write latency. So we set `allow_blocking_flush = false`.
  auto ingest_opts = *opt;
  ingest_opts.allow_blocking_flush = false;
  *s = db->rep->IngestExternalFile(handle->rep, files, ingest_opts);
  if (s->IsInvalidArgument() &&
      s->ToString().find("External file requires flush") != std::string::npos) {
    // When `allow_blocking_flush = false` and the file being ingested
    // is overlapped with the memtable, `IngestExternalFile` returns
    // an invalid argument error. It is tricky to search for the
    // specific error message here but don't worry, the unit test
    // ensures that we get this right. Then we can try to flush the
    // memtable outside without blocking writes. We also set
    // `allow_write_stall = false` to prevent the flush from
    // triggering write stall.
    has_flush = true;
    FlushOptions flush_opts;
    flush_opts.wait = true;
    flush_opts.allow_write_stall = false;
    // We don't check the status of this flush because we will
    // fallback to a blocking ingestion anyway.
    db->rep->Flush(flush_opts, handle->rep);
    *s = db->rep->IngestExternalFile(handle->rep, files, *opt);
  }
  return has_flush;
}

crocksdb_slicetransform_t* crocksdb_slicetransform_create(
    void* state, void (*destructor)(void*),
    char* (*transform)(void*, const char* key, size_t length,
                       size_t* dst_length),
    unsigned char (*in_domain)(void*, const char* key, size_t length),
    unsigned char (*in_range)(void*, const char* key, size_t length),
    const char* (*name)(void*)) {
  crocksdb_slicetransform_t* result = new crocksdb_slicetransform_t;
  result->state_ = state;
  result->destructor_ = destructor;
  result->transform_ = transform;
  result->in_domain_ = in_domain;
  result->in_range_ = in_range;
  result->name_ = name;
  return result;
}

void crocksdb_slicetransform_destroy(crocksdb_slicetransform_t* st) {
  delete st;
}

crocksdb_slicetransform_t* crocksdb_slicetransform_create_fixed_prefix(
    size_t prefixLen) {
  struct Wrapper : public crocksdb_slicetransform_t {
    const SliceTransform* rep_;
    ~Wrapper() { delete rep_; }
    const char* Name() const override { return rep_->Name(); }
    Slice Transform(const Slice& src) const override {
      return rep_->Transform(src);
    }
    bool InDomain(const Slice& src) const override {
      return rep_->InDomain(src);
    }
    bool InRange(const Slice& src) const override { return rep_->InRange(src); }
    static void DoNothing(void*) {}
  };
  Wrapper* wrapper = new Wrapper;
  wrapper->rep_ = rocksdb::NewFixedPrefixTransform(prefixLen);
  wrapper->state_ = nullptr;
  wrapper->destructor_ = &Wrapper::DoNothing;
  return wrapper;
}

crocksdb_slicetransform_t* crocksdb_slicetransform_create_noop() {
  struct Wrapper : public crocksdb_slicetransform_t {
    const SliceTransform* rep_;
    ~Wrapper() { delete rep_; }
    const char* Name() const override { return rep_->Name(); }
    Slice Transform(const Slice& src) const override {
      return rep_->Transform(src);
    }
    bool InDomain(const Slice& src) const override {
      return rep_->InDomain(src);
    }
    bool InRange(const Slice& src) const override { return rep_->InRange(src); }
    static void DoNothing(void*) {}
  };
  Wrapper* wrapper = new Wrapper;
  wrapper->rep_ = rocksdb::NewNoopTransform();
  wrapper->state_ = nullptr;
  wrapper->destructor_ = &Wrapper::DoNothing;
  return wrapper;
}

crocksdb_universal_compaction_options_t*
crocksdb_universal_compaction_options_create() {
  crocksdb_universal_compaction_options_t* result =
      new crocksdb_universal_compaction_options_t;
  result->rep = new rocksdb::CompactionOptionsUniversal;
  return result;
}

void crocksdb_universal_compaction_options_set_size_ratio(
    crocksdb_universal_compaction_options_t* uco, int ratio) {
  uco->rep->size_ratio = ratio;
}

void crocksdb_universal_compaction_options_set_min_merge_width(
    crocksdb_universal_compaction_options_t* uco, int w) {
  uco->rep->min_merge_width = w;
}

void crocksdb_universal_compaction_options_set_max_merge_width(
    crocksdb_universal_compaction_options_t* uco, int w) {
  uco->rep->max_merge_width = w;
}

void crocksdb_universal_compaction_options_set_max_size_amplification_percent(
    crocksdb_universal_compaction_options_t* uco, int p) {
  uco->rep->max_size_amplification_percent = p;
}

void crocksdb_universal_compaction_options_set_compression_size_percent(
    crocksdb_universal_compaction_options_t* uco, int p) {
  uco->rep->compression_size_percent = p;
}

void crocksdb_universal_compaction_options_set_stop_style(
    crocksdb_universal_compaction_options_t* uco,
    rocksdb::CompactionStopStyle style) {
  uco->rep->stop_style = style;
}

void crocksdb_universal_compaction_options_destroy(
    crocksdb_universal_compaction_options_t* uco) {
  delete uco->rep;
  delete uco;
}

crocksdb_fifo_compaction_options_t* crocksdb_fifo_compaction_options_create() {
  crocksdb_fifo_compaction_options_t* result =
      new crocksdb_fifo_compaction_options_t;
  result->rep = CompactionOptionsFIFO();
  return result;
}

void crocksdb_fifo_compaction_options_set_max_table_files_size(
    crocksdb_fifo_compaction_options_t* fifo_opts, uint64_t size) {
  fifo_opts->rep.max_table_files_size = size;
}

void crocksdb_fifo_compaction_options_set_allow_compaction(
    crocksdb_fifo_compaction_options_t* fifo_opts,
    unsigned char allow_compaction) {
  fifo_opts->rep.allow_compaction = allow_compaction;
}

void crocksdb_fifo_compaction_options_destroy(
    crocksdb_fifo_compaction_options_t* fifo_opts) {
  delete fifo_opts;
}

void crocksdb_options_set_min_level_to_compress(crocksdb_options_t* opt,
                                                int level) {
  if (level >= 0) {
    assert(level <= opt->rep.num_levels);
    opt->rep.compression_per_level.resize(opt->rep.num_levels);
    for (int i = 0; i < level; i++) {
      opt->rep.compression_per_level[i] = rocksdb::kNoCompression;
    }
    for (int i = level; i < opt->rep.num_levels; i++) {
      opt->rep.compression_per_level[i] = opt->rep.compression;
    }
  }
}

size_t crocksdb_livefiles_count(const crocksdb_livefiles_t* lf) {
  return static_cast<int>(lf->rep.size());
}

const char* crocksdb_livefiles_name(const crocksdb_livefiles_t* lf, int index) {
  return lf->rep[index].name.c_str();
}

int crocksdb_livefiles_level(const crocksdb_livefiles_t* lf, int index) {
  return lf->rep[index].level;
}

size_t crocksdb_livefiles_size(const crocksdb_livefiles_t* lf, int index) {
  return lf->rep[index].size;
}

const char* crocksdb_livefiles_smallestkey(const crocksdb_livefiles_t* lf,
                                           int index, size_t* size) {
  *size = lf->rep[index].smallestkey.size();
  return lf->rep[index].smallestkey.data();
}

const char* crocksdb_livefiles_largestkey(const crocksdb_livefiles_t* lf,
                                          int index, size_t* size) {
  *size = lf->rep[index].largestkey.size();
  return lf->rep[index].largestkey.data();
}

extern void crocksdb_livefiles_destroy(const crocksdb_livefiles_t* lf) {
  delete lf;
}

void crocksdb_get_options_from_string(const crocksdb_options_t* base_options,
                                      const char* opts_str,
                                      crocksdb_options_t* new_options,
                                      Status* s) {
  *s = GetOptionsFromString(base_options->rep, std::string(opts_str),
                            &new_options->rep);
}

void crocksdb_delete_files_in_range(crocksdb_t* db, const char* start_key,
                                    size_t start_key_len, const char* limit_key,
                                    size_t limit_key_len,
                                    unsigned char include_end, Status* s) {
  Slice a, b;
  *s = DeleteFilesInRange(
      db->rep, db->rep->DefaultColumnFamily(),
      (start_key ? (a = Slice(start_key, start_key_len), &a) : nullptr),
      (limit_key ? (b = Slice(limit_key, limit_key_len), &b) : nullptr),
      include_end);
}

void crocksdb_delete_files_in_range_cf(
    crocksdb_t* db, crocksdb_column_family_handle_t* column_family,
    const char* start_key, size_t start_key_len, const char* limit_key,
    size_t limit_key_len, unsigned char include_end, Status* s) {
  Slice a, b;
  *s = DeleteFilesInRange(
      db->rep, column_family->rep,
      (start_key ? (a = Slice(start_key, start_key_len), &a) : nullptr),
      (limit_key ? (b = Slice(limit_key, limit_key_len), &b) : nullptr),
      include_end);
}

void crocksdb_delete_files_in_ranges_cf(
    crocksdb_t* db, crocksdb_column_family_handle_t* cf,
    const char* const* start_keys, const size_t* start_keys_lens,
    const char* const* limit_keys, const size_t* limit_keys_lens,
    size_t num_ranges, unsigned char include_end, Status* s) {
  std::vector<Slice> starts(num_ranges);
  std::vector<Slice> limits(num_ranges);
  std::vector<RangePtr> ranges(num_ranges);
  for (auto i = 0; i < num_ranges; i++) {
    const Slice* start = nullptr;
    if (start_keys[i]) {
      starts[i] = Slice(start_keys[i], start_keys_lens[i]);
      start = &starts[i];
    }
    const Slice* limit = nullptr;
    if (limit_keys[i]) {
      limits[i] = Slice(limit_keys[i], limit_keys_lens[i]);
      limit = &limits[i];
    }
    ranges[i] = RangePtr(start, limit);
  }
  *s = DeleteFilesInRanges(db->rep, cf->rep, &ranges[0], num_ranges,
                           include_end);
}

void crocksdb_free(void* ptr) { free(ptr); }

crocksdb_logger_t* crocksdb_create_env_logger(const char* fname, Env* env) {
  crocksdb_logger_t* logger = new crocksdb_logger_t;
  Status s = NewEnvLogger(std::string(fname), env, &logger->rep);
  if (!s.ok()) {
    delete logger;
    return NULL;
  }
  return logger;
}

crocksdb_logger_t* crocksdb_create_log_from_options(const char* path,
                                                    crocksdb_options_t* opts,
                                                    Status* s) {
  std::shared_ptr<Logger> l;
  *s = CreateLoggerFromOptions(std::string(path), opts->rep, &l);
  if (s->ok()) {
    auto logger = new crocksdb_logger_t;
    logger->rep = l;
    return logger;
  }
  return nullptr;
}

void crocksdb_log_destroy(crocksdb_logger_t* logger) { delete logger; }

crocksdb_pinnableslice_t* crocksdb_get_pinned(crocksdb_t* db,
                                              const ReadOptions* options,
                                              const char* key, size_t keylen,
                                              Status* s) {
  auto v = new crocksdb_pinnableslice_t;
  *s = db->rep->Get(*options, db->rep->DefaultColumnFamily(),
                    Slice(key, keylen), &v->rep);
  if (s->ok()) {
    return v;
  } else {
    delete v;
    return nullptr;
  }
}

crocksdb_pinnableslice_t* crocksdb_get_pinned_cf(
    crocksdb_t* db, const ReadOptions* options,
    crocksdb_column_family_handle_t* column_family, const char* key,
    size_t keylen, Status* s) {
  auto v = new crocksdb_pinnableslice_t;
  *s = db->rep->Get(*options, column_family->rep, Slice(key, keylen), &v->rep);
  if (s->ok()) {
    return v;
  } else {
    delete v;
    return nullptr;
  }
}

void crocksdb_pinnableslice_destroy(crocksdb_pinnableslice_t* v) { delete v; }

const char* crocksdb_pinnableslice_value(const crocksdb_pinnableslice_t* v,
                                         size_t* vlen) {
  // v can't be null.
  *vlen = v->rep.size();
  return v->rep.data();
}

size_t crocksdb_get_supported_compression_number() {
  return rocksdb::GetSupportedCompressions().size();
}

void crocksdb_get_supported_compression(CompressionType* v, size_t l) {
  auto compressions = rocksdb::GetSupportedCompressions();
  assert(compressions.size() == l);
  for (size_t i = 0; i < compressions.size(); i++) {
    v[i] = compressions[i];
  }
}

/* Table Properties */

void crocksdb_user_collected_properties_add(UserCollectedProperties* props,
                                            Slice key, Slice value) {
  props->emplace(std::make_pair(key.ToString(), value.ToString()));
}

struct crocksdb_user_collected_properties_iterator_t {
  UserCollectedProperties::const_iterator cur_;
  UserCollectedProperties::const_iterator end_;
};

crocksdb_user_collected_properties_iterator_t*
crocksdb_user_collected_properties_iter_create(
    const UserCollectedProperties* props) {
  auto it = new crocksdb_user_collected_properties_iterator_t;
  it->cur_ = props->begin();
  it->end_ = props->end();
  return it;
}

void crocksdb_user_collected_properties_iter_destroy(
    crocksdb_user_collected_properties_iterator_t* it) {
  delete it;
}

bool crocksdb_user_collected_properties_iter_next(
    crocksdb_user_collected_properties_iterator_t* it, Slice* key, Slice* val) {
  if (it->cur_ != it->end_) {
    *key = it->cur_->first;
    *val = it->cur_->second;
    ++(it->cur_);
    return true;
  }
  return false;
}

uint64_t crocksdb_table_properties_get_data_size(const TableProperties* props) {
  return props->data_size;
}
uint64_t crocksdb_table_properties_get_index_size(
    const TableProperties* props) {
  return props->index_size;
}
uint64_t crocksdb_table_properties_get_index_partitions(
    const TableProperties* props) {
  return props->index_partitions;
}
uint64_t crocksdb_table_properties_get_top_level_index_size(
    const TableProperties* props) {
  return props->top_level_index_size;
}
uint64_t crocksdb_table_properties_get_index_key_is_user_key(
    const TableProperties* props) {
  return props->index_key_is_user_key;
}
uint64_t crocksdb_table_properties_get_index_value_is_delta_encoded(
    const TableProperties* props) {
  return props->index_value_is_delta_encoded;
}
uint64_t crocksdb_table_properties_get_filter_size(
    const TableProperties* props) {
  return props->filter_size;
}
uint64_t crocksdb_table_properties_get_raw_key_size(
    const TableProperties* props) {
  return props->raw_key_size;
}
uint64_t crocksdb_table_properties_get_raw_value_size(
    const TableProperties* props) {
  return props->raw_value_size;
}
uint64_t crocksdb_table_properties_get_num_data_blocks(
    const TableProperties* props) {
  return props->num_data_blocks;
}
uint64_t crocksdb_table_properties_get_num_entries(
    const TableProperties* props) {
  return props->num_entries;
}
uint64_t crocksdb_table_properties_get_num_deletions(
    const TableProperties* props) {
  return props->num_deletions;
}
uint64_t crocksdb_table_properties_get_num_merge_operands(
    const TableProperties* props) {
  return props->num_merge_operands;
}
uint64_t crocksdb_table_properties_get_num_range_deletions(
    const TableProperties* props) {
  return props->num_range_deletions;
}
uint64_t crocksdb_table_properties_get_format_version(
    const TableProperties* props) {
  return props->format_version;
}
uint64_t crocksdb_table_properties_get_fixed_key_len(
    const TableProperties* props) {
  return props->fixed_key_len;
}
uint64_t crocksdb_table_properties_get_column_family_id(
    const TableProperties* props) {
  return props->column_family_id;
}
uint64_t crocksdb_table_properties_get_creation_time(
    const TableProperties* props) {
  return props->creation_time;
}
uint64_t crocksdb_table_properties_get_oldest_key_time(
    const TableProperties* props) {
  return props->oldest_key_time;
}
uint64_t crocksdb_table_properties_get_file_creation_time(
    const TableProperties* props) {
  return props->file_creation_time;
}

void crocksdb_table_properties_get_column_family_name(
    const TableProperties* props, Slice* val) {
  *val = props->column_family_name;
}
void crocksdb_table_properties_get_filter_policy_name(
    const TableProperties* props, Slice* val) {
  *val = props->filter_policy_name;
}
void crocksdb_table_properties_get_comparator_name(const TableProperties* props,
                                                   Slice* val) {
  *val = props->comparator_name;
}
void crocksdb_table_properties_get_merge_operator_name(
    const TableProperties* props, Slice* val) {
  *val = props->merge_operator_name;
}
void crocksdb_table_properties_get_prefix_extractor_name(
    const TableProperties* props, Slice* val) {
  *val = props->prefix_extractor_name;
}
void crocksdb_table_properties_get_property_collectors_names(
    const TableProperties* props, Slice* val) {
  *val = props->property_collectors_names;
}
void crocksdb_table_properties_get_compression_name(
    const TableProperties* props, Slice* val) {
  *val = props->compression_name;
}
void crocksdb_table_properties_get_compression_options(
    const TableProperties* props, Slice* val) {
  *val = props->compression_options;
}

char* crocksdb_table_properties_to_string(const TableProperties* props,
                                          size_t* len) {
  auto s = props->ToString();
  *len = s.size();
  return strndup(s.data(), s.size());
}

const UserCollectedProperties* crocksdb_table_properties_get_user_properties(
    const TableProperties* props) {
  return &props->user_collected_properties;
}

bool crocksdb_user_collected_properties_get(
    const UserCollectedProperties* props, Slice key, Slice* value) {
  auto val = props->find(key.ToString());
  if (val == props->end()) {
    return false;
  }
  *value = val->second;
  return true;
}

size_t crocksdb_user_collected_properties_len(
    const UserCollectedProperties* props) {
  return props->size();
}

/* Table Properties Collection */

const TableProperties* crocksdb_table_properties_collection_get(
    const TablePropertiesCollection* props, Slice key) {
  auto val = props->find(key.ToString());
  if (val != props->end()) {
    return val->second.get();
  } else {
    return nullptr;
  }
}

size_t crocksdb_table_properties_collection_len(
    const TablePropertiesCollection* props) {
  return props->size();
}

void crocksdb_table_properties_collection_destroy(
    TablePropertiesCollection* t) {
  delete t;
}

struct crocksdb_table_properties_collection_iterator_t {
  TablePropertiesCollection::const_iterator cur_;
  TablePropertiesCollection::const_iterator end_;
};

crocksdb_table_properties_collection_iterator_t*
crocksdb_table_properties_collection_iter_create(
    const TablePropertiesCollection* collection) {
  auto it = new crocksdb_table_properties_collection_iterator_t;
  it->cur_ = collection->begin();
  it->end_ = collection->end();
  return it;
}

void crocksdb_table_properties_collection_iter_destroy(
    crocksdb_table_properties_collection_iterator_t* it) {
  delete it;
}

bool crocksdb_table_properties_collection_iter_next(
    crocksdb_table_properties_collection_iterator_t* it, Slice* key,
    const TableProperties** val) {
  if (it->cur_ != it->end_) {
    *key = it->cur_->first;
    *val = it->cur_->second.get();
    ++(it->cur_);
    return true;
  }
  return false;
}

/* Table Properties Collector */

struct crocksdb_table_properties_collector_t : public TablePropertiesCollector {
  void* state_;
  const char* (*name_)(void*);
  void (*destruct_)(void*);
  void (*add_)(void*, Slice key, Slice value, EntryType entry_type,
               SequenceNumber seq, uint64_t file_size, Status* s);
  void (*finish_)(void*, UserCollectedProperties* props, Status*);

  virtual ~crocksdb_table_properties_collector_t() { destruct_(state_); }

  virtual Status AddUserKey(const Slice& key, const Slice& value,
                            EntryType entry_type, SequenceNumber seq,
                            uint64_t file_size) override {
    Status s;
    add_(state_, key, value, entry_type, seq, file_size, &s);
    return s;
  }

  virtual Status Finish(UserCollectedProperties* rep) override {
    Status s;
    finish_(state_, rep, &s);
    return s;
  }

  virtual UserCollectedProperties GetReadableProperties() const override {
    // Seems rocksdb will not return the readable properties and we don't need
    // them too.
    return UserCollectedProperties();
  }

  const char* Name() const override { return name_(state_); }
};

crocksdb_table_properties_collector_t*
crocksdb_table_properties_collector_create(
    void* state, const char* (*name)(void*), void (*destruct)(void*),
    void (*add)(void*, Slice key, Slice value, EntryType entry_type,
                SequenceNumber seq, uint64_t file_size, Status*),
    void (*finish)(void*, UserCollectedProperties*, Status*)) {
  auto c = new crocksdb_table_properties_collector_t;
  c->state_ = state;
  c->name_ = name;
  c->destruct_ = destruct;
  c->add_ = add;
  c->finish_ = finish;
  return c;
}

void crocksdb_table_properties_collector_destroy(
    crocksdb_table_properties_collector_t* c) {
  delete c;
}

/* Table Properties Collector Factory */

struct crocksdb_table_properties_collector_factory_t
    : public TablePropertiesCollectorFactory {
  void* state_;
  const char* (*name_)(void*);
  void (*destruct_)(void*);
  crocksdb_table_properties_collector_t* (*create_table_properties_collector_)(
      void*, TablePropertiesCollectorFactory::Context context);

  virtual ~crocksdb_table_properties_collector_factory_t() {
    destruct_(state_);
  }

  virtual TablePropertiesCollector* CreateTablePropertiesCollector(
      TablePropertiesCollectorFactory::Context ctx) override {
    return create_table_properties_collector_(state_, ctx);
  }

  const char* Name() const override { return name_(state_); }
};

crocksdb_table_properties_collector_factory_t*
crocksdb_table_properties_collector_factory_create(
    void* state, const char* (*name)(void*), void (*destruct)(void*),
    crocksdb_table_properties_collector_t* (*create_table_properties_collector)(
        void*, TablePropertiesCollectorFactory::Context context)) {
  auto f = new crocksdb_table_properties_collector_factory_t;
  f->state_ = state;
  f->name_ = name;
  f->destruct_ = destruct;
  f->create_table_properties_collector_ = create_table_properties_collector;
  return f;
}

void crocksdb_table_properties_collector_factory_destroy(
    crocksdb_table_properties_collector_factory_t* f) {
  delete f;
}

void crocksdb_options_add_table_properties_collector_factory(
    crocksdb_options_t* opt, crocksdb_table_properties_collector_factory_t* f) {
  opt->rep.table_properties_collector_factories.push_back(
      std::shared_ptr<TablePropertiesCollectorFactory>(f));
}

void crocksdb_options_set_compact_on_deletion(crocksdb_options_t* opt,
                                              size_t sliding_window_size,
                                              size_t deletion_trigger) {
  opt->rep.table_properties_collector_factories.push_back(
      rocksdb::NewCompactOnDeletionCollectorFactory(sliding_window_size,
                                                    deletion_trigger));
}

/* Get Table Properties */

TablePropertiesCollection* crocksdb_get_properties_of_all_tables(crocksdb_t* db,
                                                                 Status* s) {
  auto v = new TablePropertiesCollection;
  *s = db->rep->GetPropertiesOfAllTables(v);
  if (s->ok()) {
    return v;
  } else {
    delete v;
    return nullptr;
  }
}

TablePropertiesCollection* crocksdb_get_properties_of_all_tables_cf(
    crocksdb_t* db, crocksdb_column_family_handle_t* cf, Status* s) {
  auto v = new TablePropertiesCollection;
  *s = db->rep->GetPropertiesOfAllTables(cf->rep, v);
  if (s->ok()) {
    return v;
  } else {
    delete v;
    return nullptr;
  }
}

TablePropertiesCollection* crocksdb_get_properties_of_tables_in_range(
    crocksdb_t* db, crocksdb_column_family_handle_t* cf, int num_ranges,
    const char* const* start_keys, const size_t* start_keys_lens,
    const char* const* limit_keys, const size_t* limit_keys_lens, Status* s) {
  std::vector<Range> ranges;
  for (int i = 0; i < num_ranges; i++) {
    ranges.emplace_back(Range(Slice(start_keys[i], start_keys_lens[i]),
                              Slice(limit_keys[i], limit_keys_lens[i])));
  }
  auto v = new TablePropertiesCollection;
  *s = db->rep->GetPropertiesOfTablesInRange(cf->rep, ranges.data(),
                                             ranges.size(), v);
  if (s->ok()) {
    return v;
  } else {
    delete v;
    return nullptr;
  }
}

void crocksdb_set_bottommost_compression(crocksdb_options_t* opt,
                                         CompressionType c) {
  opt->rep.bottommost_compression = c;
}
// Get All Key Versions
void crocksdb_keyversions_destroy(crocksdb_keyversions_t* kvs) { delete kvs; }

crocksdb_keyversions_t* crocksdb_get_all_key_versions(
    crocksdb_t* db, const char* begin_key, size_t begin_keylen,
    const char* end_key, size_t end_keylen, Status* s) {
  auto v = new crocksdb_keyversions_t;
  constexpr size_t kMaxNumKeys = std::numeric_limits<size_t>::max();
  *s = GetAllKeyVersions(db->rep, Slice(begin_key, begin_keylen),
                         Slice(end_key, end_keylen), kMaxNumKeys, &v->rep);
  if (s->ok()) {
    return v;
  } else {
    delete v;
    return nullptr;
  }
}

size_t crocksdb_keyversions_count(const crocksdb_keyversions_t* kvs) {
  return kvs->rep.size();
}

const char* crocksdb_keyversions_key(const crocksdb_keyversions_t* kvs,
                                     int index) {
  return kvs->rep[index].user_key.c_str();
}

const char* crocksdb_keyversions_value(const crocksdb_keyversions_t* kvs,
                                       int index) {
  return kvs->rep[index].value.c_str();
}

uint64_t crocksdb_keyversions_seq(const crocksdb_keyversions_t* kvs,
                                  int index) {
  return kvs->rep[index].sequence;
}

int crocksdb_keyversions_type(const crocksdb_keyversions_t* kvs, int index) {
  return kvs->rep[index].type;
}

struct ExternalSstFileModifier {
  ExternalSstFileModifier(Env* env, const EnvOptions& env_options,
                          ColumnFamilyHandle* handle)
      : env_(env), env_options_(env_options), handle_(handle) {}

  Status Open(std::string file) {
    file_ = file;
    // Get External Sst File Size
    uint64_t file_size;
    auto status = env_->GetFileSize(file_, &file_size);
    if (!status.ok()) {
      return status;
    }

    // Open External Sst File
    std::unique_ptr<RandomAccessFile> sst_file;
    std::unique_ptr<RandomAccessFileReader> sst_file_reader;
    status = env_->NewRandomAccessFile(file_, &sst_file, env_options_);
    if (!status.ok()) {
      return status;
    }
    sst_file_reader.reset(
        new RandomAccessFileReader(std::move(sst_file), file_));

    // Get Table Reader
    ColumnFamilyDescriptor desc;
    handle_->GetDescriptor(&desc);
    auto cfd = reinterpret_cast<ColumnFamilyHandleImpl*>(handle_)->cfd();
    auto ioptions = *cfd->ioptions();
    auto table_opt =
        TableReaderOptions(ioptions, desc.options.prefix_extractor.get(),
                           env_options_, cfd->internal_comparator());
    // Get around global seqno check.
    table_opt.largest_seqno = kMaxSequenceNumber;
    status = ioptions.table_factory->NewTableReader(
        table_opt, std::move(sst_file_reader), file_size, &table_reader_);
    return status;
  }

  Status SetGlobalSeqNo(uint64_t seq_no, uint64_t* pre_seq_no) {
    if (table_reader_ == nullptr) {
      return Status::InvalidArgument(
          "File is not open or seq-no has been modified");
    }
    // Get the external file properties
    auto props = table_reader_->GetTableProperties();
    const auto& uprops = props->user_collected_properties;
    // Validate version and seqno offset
    auto version_iter = uprops.find(ExternalSstFilePropertyNames::kVersion);
    if (version_iter == uprops.end()) {
      return Status::Corruption("External file version not found");
    }
    uint32_t version = DecodeFixed32(version_iter->second.c_str());
    if (version != 2) {
      return Status::NotSupported("External file version should be 2");
    }

    auto seqno_iter = uprops.find(ExternalSstFilePropertyNames::kGlobalSeqno);
    if (seqno_iter == uprops.end()) {
      return Status::Corruption(
          "External file global sequence number not found");
    }
    *pre_seq_no = DecodeFixed64(seqno_iter->second.c_str());
    uint64_t offset = props->properties_offsets.at(
        ExternalSstFilePropertyNames::kGlobalSeqno);
    if (offset == 0) {
      return Status::Corruption("Was not able to find file global seqno field");
    }

    if (*pre_seq_no == seq_no) {
      // This file already have the correct global seqno
      return Status::OK();
    }

    std::unique_ptr<RandomRWFile> rwfile;
    auto status = env_->NewRandomRWFile(file_, &rwfile, env_options_);
    if (!status.ok()) {
      return status;
    }

    // Write the new seqno in the global sequence number field in the file
    std::string seqno_val;
    PutFixed64(&seqno_val, seq_no);
    status = rwfile->Write(offset, seqno_val);
    return status;
  }

 private:
  Env* env_;
  EnvOptions env_options_;
  ColumnFamilyHandle* handle_;
  std::string file_;
  std::unique_ptr<TableReader> table_reader_;
};

// !!! this function is dangerous because it uses rocksdb's non-public API !!!
// find the offset of external sst file's `global seq no` and modify it.
uint64_t crocksdb_set_external_sst_file_global_seq_no(
    crocksdb_t* db, crocksdb_column_family_handle_t* column_family,
    const char* file, uint64_t seq_no, Status* s) {
  auto env = db->rep->GetEnv();
  EnvOptions env_options(db->rep->GetDBOptions());
  ExternalSstFileModifier modifier(env, env_options, column_family->rep);
  *s = modifier.Open(std::string(file));
  if (s->ok()) {
    uint64_t pre_seq_no;
    *s = modifier.SetGlobalSeqNo(seq_no, &pre_seq_no);
    return pre_seq_no;
  } else {
    return 0;
  }
}

void crocksdb_get_column_family_meta_data(
    crocksdb_t* db, crocksdb_column_family_handle_t* cf,
    crocksdb_column_family_meta_data_t* meta) {
  db->rep->GetColumnFamilyMetaData(cf->rep, &meta->rep);
}

crocksdb_column_family_meta_data_t* crocksdb_column_family_meta_data_create() {
  return new crocksdb_column_family_meta_data_t();
}

void crocksdb_column_family_meta_data_destroy(
    crocksdb_column_family_meta_data_t* meta) {
  delete meta;
}

size_t crocksdb_column_family_meta_data_level_count(
    const crocksdb_column_family_meta_data_t* meta) {
  return meta->rep.levels.size();
}

const crocksdb_level_meta_data_t* crocksdb_column_family_meta_data_level_data(
    const crocksdb_column_family_meta_data_t* meta, size_t n) {
  return reinterpret_cast<const crocksdb_level_meta_data_t*>(
      &meta->rep.levels[n]);
}

size_t crocksdb_level_meta_data_file_count(
    const crocksdb_level_meta_data_t* meta) {
  return meta->rep.files.size();
}

const crocksdb_sst_file_meta_data_t* crocksdb_level_meta_data_file_data(
    const crocksdb_level_meta_data_t* meta, size_t n) {
  return reinterpret_cast<const crocksdb_sst_file_meta_data_t*>(
      &meta->rep.files[n]);
}

size_t crocksdb_sst_file_meta_data_size(
    const crocksdb_sst_file_meta_data_t* meta) {
  return meta->rep.size;
}

const char* crocksdb_sst_file_meta_data_name(
    const crocksdb_sst_file_meta_data_t* meta) {
  return meta->rep.name.data();
}

const char* crocksdb_sst_file_meta_data_smallestkey(
    const crocksdb_sst_file_meta_data_t* meta, size_t* len) {
  *len = meta->rep.smallestkey.size();
  return meta->rep.smallestkey.data();
}

const char* crocksdb_sst_file_meta_data_largestkey(
    const crocksdb_sst_file_meta_data_t* meta, size_t* len) {
  *len = meta->rep.largestkey.size();
  return meta->rep.largestkey.data();
}

void crocksdb_compaction_options_init(CompactionOptions* opt) {
  *opt = CompactionOptions();
}

void crocksdb_compact_files_cf(crocksdb_t* db,
                               crocksdb_column_family_handle_t* cf,
                               const CompactionOptions* opts,
                               const char** input_file_names,
                               size_t input_file_count, int output_level,
                               Status* s) {
  std::vector<std::string> input_files;
  for (size_t i = 0; i < input_file_count; i++) {
    input_files.push_back(input_file_names[i]);
  }
  *s = db->rep->CompactFiles(*opts, cf->rep, input_files, output_level);
}

/* PerfContext */

PerfLevel crocksdb_get_perf_level(void) { return rocksdb::GetPerfLevel(); }

void crocksdb_set_perf_level(PerfLevel level) { rocksdb::SetPerfLevel(level); }

struct crocksdb_perf_context_t {
  PerfContext rep;
};

crocksdb_perf_context_t* crocksdb_get_perf_context(void) {
  return reinterpret_cast<crocksdb_perf_context_t*>(
      rocksdb::get_perf_context());
}

void crocksdb_perf_context_reset(crocksdb_perf_context_t* ctx) {
  ctx->rep.Reset();
}

uint64_t crocksdb_perf_context_user_key_comparison_count(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.user_key_comparison_count;
}

uint64_t crocksdb_perf_context_block_cache_hit_count(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.block_cache_hit_count;
}

uint64_t crocksdb_perf_context_block_read_count(crocksdb_perf_context_t* ctx) {
  return ctx->rep.block_read_count;
}

uint64_t crocksdb_perf_context_block_read_byte(crocksdb_perf_context_t* ctx) {
  return ctx->rep.block_read_byte;
}

uint64_t crocksdb_perf_context_block_read_time(crocksdb_perf_context_t* ctx) {
  return ctx->rep.block_read_time;
}

uint64_t crocksdb_perf_context_block_cache_index_hit_count(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.block_cache_index_hit_count;
}

uint64_t crocksdb_perf_context_index_block_read_count(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.index_block_read_count;
}

uint64_t crocksdb_perf_context_block_cache_filter_hit_count(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.block_cache_filter_hit_count;
}

uint64_t crocksdb_perf_context_filter_block_read_count(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.filter_block_read_count;
}

uint64_t crocksdb_perf_context_block_checksum_time(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.block_checksum_time;
}

uint64_t crocksdb_perf_context_block_decompress_time(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.block_decompress_time;
}

uint64_t crocksdb_perf_context_get_read_bytes(crocksdb_perf_context_t* ctx) {
  return ctx->rep.get_read_bytes;
}

uint64_t crocksdb_perf_context_multiget_read_bytes(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.multiget_read_bytes;
}

uint64_t crocksdb_perf_context_iter_read_bytes(crocksdb_perf_context_t* ctx) {
  return ctx->rep.iter_read_bytes;
}

uint64_t crocksdb_perf_context_internal_key_skipped_count(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.internal_key_skipped_count;
}

uint64_t crocksdb_perf_context_internal_delete_skipped_count(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.internal_delete_skipped_count;
}

uint64_t crocksdb_perf_context_internal_recent_skipped_count(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.internal_recent_skipped_count;
}

uint64_t crocksdb_perf_context_internal_merge_count(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.internal_merge_count;
}

uint64_t crocksdb_perf_context_get_snapshot_time(crocksdb_perf_context_t* ctx) {
  return ctx->rep.get_snapshot_time;
}

uint64_t crocksdb_perf_context_get_from_memtable_time(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.get_from_memtable_time;
}

uint64_t crocksdb_perf_context_get_from_memtable_count(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.get_from_memtable_count;
}

uint64_t crocksdb_perf_context_get_post_process_time(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.get_post_process_time;
}

uint64_t crocksdb_perf_context_get_from_output_files_time(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.get_from_output_files_time;
}

uint64_t crocksdb_perf_context_seek_on_memtable_time(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.seek_on_memtable_time;
}

uint64_t crocksdb_perf_context_seek_on_memtable_count(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.seek_on_memtable_count;
}

uint64_t crocksdb_perf_context_next_on_memtable_count(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.next_on_memtable_count;
}

uint64_t crocksdb_perf_context_prev_on_memtable_count(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.prev_on_memtable_count;
}

uint64_t crocksdb_perf_context_seek_child_seek_time(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.seek_child_seek_time;
}

uint64_t crocksdb_perf_context_seek_child_seek_count(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.seek_child_seek_count;
}

uint64_t crocksdb_perf_context_seek_min_heap_time(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.seek_min_heap_time;
}

uint64_t crocksdb_perf_context_seek_max_heap_time(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.seek_max_heap_time;
}

uint64_t crocksdb_perf_context_seek_internal_seek_time(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.seek_internal_seek_time;
}

uint64_t crocksdb_perf_context_find_next_user_entry_time(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.find_next_user_entry_time;
}

uint64_t crocksdb_perf_context_write_wal_time(crocksdb_perf_context_t* ctx) {
  return ctx->rep.write_wal_time;
}

uint64_t crocksdb_perf_context_write_memtable_time(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.write_memtable_time;
}

uint64_t crocksdb_perf_context_write_delay_time(crocksdb_perf_context_t* ctx) {
  return ctx->rep.write_delay_time;
}

uint64_t crocksdb_perf_context_write_pre_and_post_process_time(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.write_pre_and_post_process_time;
}

uint64_t crocksdb_perf_context_db_mutex_lock_nanos(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.db_mutex_lock_nanos;
}

uint64_t crocksdb_perf_context_write_thread_wait_nanos(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.write_thread_wait_nanos;
}

uint64_t crocksdb_perf_context_write_scheduling_flushes_compactions_time(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.write_scheduling_flushes_compactions_time;
}

uint64_t crocksdb_perf_context_db_condition_wait_nanos(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.db_condition_wait_nanos;
}

uint64_t crocksdb_perf_context_merge_operator_time_nanos(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.merge_operator_time_nanos;
}

uint64_t crocksdb_perf_context_read_index_block_nanos(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.read_index_block_nanos;
}

uint64_t crocksdb_perf_context_read_filter_block_nanos(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.read_filter_block_nanos;
}

uint64_t crocksdb_perf_context_new_table_block_iter_nanos(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.new_table_block_iter_nanos;
}

uint64_t crocksdb_perf_context_new_table_iterator_nanos(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.new_table_iterator_nanos;
}

uint64_t crocksdb_perf_context_block_seek_nanos(crocksdb_perf_context_t* ctx) {
  return ctx->rep.block_seek_nanos;
}

uint64_t crocksdb_perf_context_find_table_nanos(crocksdb_perf_context_t* ctx) {
  return ctx->rep.find_table_nanos;
}

uint64_t crocksdb_perf_context_bloom_memtable_hit_count(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.bloom_memtable_hit_count;
}

uint64_t crocksdb_perf_context_bloom_memtable_miss_count(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.bloom_memtable_miss_count;
}

uint64_t crocksdb_perf_context_bloom_sst_hit_count(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.bloom_sst_hit_count;
}

uint64_t crocksdb_perf_context_bloom_sst_miss_count(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.bloom_sst_miss_count;
}

uint64_t crocksdb_perf_context_env_new_sequential_file_nanos(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.env_new_sequential_file_nanos;
}

uint64_t crocksdb_perf_context_env_new_random_access_file_nanos(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.env_new_random_access_file_nanos;
}

uint64_t crocksdb_perf_context_env_new_writable_file_nanos(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.env_new_writable_file_nanos;
}

uint64_t crocksdb_perf_context_env_reuse_writable_file_nanos(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.env_reuse_writable_file_nanos;
}

uint64_t crocksdb_perf_context_env_new_random_rw_file_nanos(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.env_new_random_rw_file_nanos;
}

uint64_t crocksdb_perf_context_env_new_directory_nanos(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.env_new_directory_nanos;
}

uint64_t crocksdb_perf_context_env_file_exists_nanos(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.env_file_exists_nanos;
}

uint64_t crocksdb_perf_context_env_get_children_nanos(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.env_get_children_nanos;
}

uint64_t crocksdb_perf_context_env_get_children_file_attributes_nanos(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.env_get_children_file_attributes_nanos;
}

uint64_t crocksdb_perf_context_env_delete_file_nanos(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.env_delete_file_nanos;
}

uint64_t crocksdb_perf_context_env_create_dir_nanos(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.env_create_dir_nanos;
}

uint64_t crocksdb_perf_context_env_create_dir_if_missing_nanos(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.env_create_dir_if_missing_nanos;
}

uint64_t crocksdb_perf_context_env_delete_dir_nanos(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.env_delete_dir_nanos;
}

uint64_t crocksdb_perf_context_env_get_file_size_nanos(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.env_get_file_size_nanos;
}

uint64_t crocksdb_perf_context_env_get_file_modification_time_nanos(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.env_get_file_modification_time_nanos;
}

uint64_t crocksdb_perf_context_env_rename_file_nanos(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.env_rename_file_nanos;
}

uint64_t crocksdb_perf_context_env_link_file_nanos(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.env_link_file_nanos;
}

uint64_t crocksdb_perf_context_env_lock_file_nanos(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.env_lock_file_nanos;
}

uint64_t crocksdb_perf_context_env_unlock_file_nanos(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.env_unlock_file_nanos;
}

uint64_t crocksdb_perf_context_env_new_logger_nanos(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.env_new_logger_nanos;
}

uint64_t crocksdb_perf_context_get_cpu_nanos(crocksdb_perf_context_t* ctx) {
  return ctx->rep.get_cpu_nanos;
}

uint64_t crocksdb_perf_context_iter_next_cpu_nanos(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.iter_next_cpu_nanos;
}

uint64_t crocksdb_perf_context_iter_prev_cpu_nanos(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.iter_next_cpu_nanos;
}

uint64_t crocksdb_perf_context_iter_seek_cpu_nanos(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.iter_next_cpu_nanos;
}

uint64_t crocksdb_perf_context_encrypt_data_nanos(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.encrypt_data_nanos;
}

uint64_t crocksdb_perf_context_decrypt_data_nanos(
    crocksdb_perf_context_t* ctx) {
  return ctx->rep.decrypt_data_nanos;
}

// IOStatsContext

struct crocksdb_iostats_context_t {
  IOStatsContext rep;
};

crocksdb_iostats_context_t* crocksdb_get_iostats_context(void) {
  return reinterpret_cast<crocksdb_iostats_context_t*>(
      rocksdb::get_iostats_context());
}

void crocksdb_iostats_context_reset(crocksdb_iostats_context_t* ctx) {
  ctx->rep.Reset();
}

uint64_t crocksdb_iostats_context_bytes_written(
    crocksdb_iostats_context_t* ctx) {
  return ctx->rep.bytes_written;
}

uint64_t crocksdb_iostats_context_bytes_read(crocksdb_iostats_context_t* ctx) {
  return ctx->rep.bytes_read;
}

uint64_t crocksdb_iostats_context_open_nanos(crocksdb_iostats_context_t* ctx) {
  return ctx->rep.open_nanos;
}

uint64_t crocksdb_iostats_context_allocate_nanos(
    crocksdb_iostats_context_t* ctx) {
  return ctx->rep.allocate_nanos;
}

uint64_t crocksdb_iostats_context_write_nanos(crocksdb_iostats_context_t* ctx) {
  return ctx->rep.write_nanos;
}

uint64_t crocksdb_iostats_context_read_nanos(crocksdb_iostats_context_t* ctx) {
  return ctx->rep.read_nanos;
}

uint64_t crocksdb_iostats_context_range_sync_nanos(
    crocksdb_iostats_context_t* ctx) {
  return ctx->rep.range_sync_nanos;
}

uint64_t crocksdb_iostats_context_fsync_nanos(crocksdb_iostats_context_t* ctx) {
  return ctx->rep.fsync_nanos;
}

uint64_t crocksdb_iostats_context_prepare_write_nanos(
    crocksdb_iostats_context_t* ctx) {
  return ctx->rep.prepare_write_nanos;
}

uint64_t crocksdb_iostats_context_logger_nanos(
    crocksdb_iostats_context_t* ctx) {
  return ctx->rep.logger_nanos;
}

crocksdb_sst_partitioner_request_t* crocksdb_sst_partitioner_request_create() {
  auto* req = new crocksdb_sst_partitioner_request_t;
  req->rep =
      new PartitionerRequest(req->prev_user_key, req->current_user_key, 0);
  return req;
}

void crocksdb_sst_partitioner_request_destroy(
    crocksdb_sst_partitioner_request_t* req) {
  delete req->rep;
  delete req;
}

const char* crocksdb_sst_partitioner_request_prev_user_key(
    crocksdb_sst_partitioner_request_t* req, size_t* len) {
  const Slice* prev_key = req->rep->prev_user_key;
  *len = prev_key->size();
  return prev_key->data();
}

const char* crocksdb_sst_partitioner_request_current_user_key(
    crocksdb_sst_partitioner_request_t* req, size_t* len) {
  const Slice* current_key = req->rep->current_user_key;
  *len = current_key->size();
  return current_key->data();
}

uint64_t crocksdb_sst_partitioner_request_current_output_file_size(
    crocksdb_sst_partitioner_request_t* req) {
  return req->rep->current_output_file_size;
}

void crocksdb_sst_partitioner_request_set_prev_user_key(
    crocksdb_sst_partitioner_request_t* req, const char* key, size_t len) {
  req->prev_user_key = Slice(key, len);
  req->rep->prev_user_key = &req->prev_user_key;
}

void crocksdb_sst_partitioner_request_set_current_user_key(
    crocksdb_sst_partitioner_request_t* req, const char* key, size_t len) {
  req->current_user_key = Slice(key, len);
  req->rep->current_user_key = &req->current_user_key;
}

void crocksdb_sst_partitioner_request_set_current_output_file_size(
    crocksdb_sst_partitioner_request_t* req,
    uint64_t current_output_file_size) {
  req->rep->current_output_file_size = current_output_file_size;
}

struct crocksdb_sst_partitioner_impl_t : public SstPartitioner {
  void* underlying;
  void (*destructor)(void*);
  crocksdb_sst_partitioner_should_partition_cb should_partition_cb;
  crocksdb_sst_partitioner_can_do_trivial_move_cb can_do_trivial_move_cb;

  virtual ~crocksdb_sst_partitioner_impl_t() { destructor(underlying); }

  const char* Name() const override { return "crocksdb_sst_partitioner_impl"; }

  PartitionerResult ShouldPartition(
      const PartitionerRequest& request) override {
    crocksdb_sst_partitioner_request_t req;
    req.rep = const_cast<PartitionerRequest*>(&request);
    return should_partition_cb(underlying, &req);
  }

  bool CanDoTrivialMove(const Slice& smallest_user_key,
                        const Slice& largest_user_key) override {
    return can_do_trivial_move_cb(
        underlying, smallest_user_key.data(), smallest_user_key.size(),
        largest_user_key.data(), largest_user_key.size());
  }
};

crocksdb_sst_partitioner_t* crocksdb_sst_partitioner_create(
    void* underlying, void (*destructor)(void*),
    crocksdb_sst_partitioner_should_partition_cb should_partition_cb,
    crocksdb_sst_partitioner_can_do_trivial_move_cb can_do_trivial_move_cb) {
  crocksdb_sst_partitioner_impl_t* sst_partitioner_impl =
      new crocksdb_sst_partitioner_impl_t;
  sst_partitioner_impl->underlying = underlying;
  sst_partitioner_impl->destructor = destructor;
  sst_partitioner_impl->should_partition_cb = should_partition_cb;
  sst_partitioner_impl->can_do_trivial_move_cb = can_do_trivial_move_cb;
  crocksdb_sst_partitioner_t* sst_partitioner = new crocksdb_sst_partitioner_t;
  sst_partitioner->rep.reset(sst_partitioner_impl);
  return sst_partitioner;
}

void crocksdb_sst_partitioner_destroy(crocksdb_sst_partitioner_t* partitioner) {
  delete partitioner;
}

rocksdb::PartitionerResult crocksdb_sst_partitioner_should_partition(
    crocksdb_sst_partitioner_t* partitioner,
    crocksdb_sst_partitioner_request_t* req) {
  return partitioner->rep->ShouldPartition(*req->rep);
}

unsigned char crocksdb_sst_partitioner_can_do_trivial_move(
    crocksdb_sst_partitioner_t* partitioner, const char* smallest_user_key,
    size_t smallest_user_key_len, const char* largest_user_key,
    size_t largest_user_key_len) {
  Slice smallest_key(smallest_user_key, smallest_user_key_len);
  Slice largest_key(largest_user_key, largest_user_key_len);
  return partitioner->rep->CanDoTrivialMove(smallest_key, largest_key);
}

crocksdb_sst_partitioner_context_t* crocksdb_sst_partitioner_context_create() {
  auto* rep = new SstPartitioner::Context;
  auto* context = new crocksdb_sst_partitioner_context_t;
  context->rep = rep;
  return context;
}

void crocksdb_sst_partitioner_context_destroy(
    crocksdb_sst_partitioner_context_t* context) {
  delete context->rep;
  delete context;
}

unsigned char crocksdb_sst_partitioner_context_is_full_compaction(
    crocksdb_sst_partitioner_context_t* context) {
  return context->rep->is_full_compaction;
}

unsigned char crocksdb_sst_partitioner_context_is_manual_compaction(
    crocksdb_sst_partitioner_context_t* context) {
  return context->rep->is_manual_compaction;
}

int crocksdb_sst_partitioner_context_output_level(
    crocksdb_sst_partitioner_context_t* context) {
  return context->rep->output_level;
}

const char* crocksdb_sst_partitioner_context_smallest_key(
    crocksdb_sst_partitioner_context_t* context, size_t* key_len) {
  auto& smallest_key = context->rep->smallest_user_key;
  *key_len = smallest_key.size();
  return smallest_key.data();
}

const char* crocksdb_sst_partitioner_context_largest_key(
    crocksdb_sst_partitioner_context_t* context, size_t* key_len) {
  auto& largest_key = context->rep->largest_user_key;
  *key_len = largest_key.size();
  return largest_key.data();
}

void crocksdb_sst_partitioner_context_set_is_full_compaction(
    crocksdb_sst_partitioner_context_t* context,
    unsigned char is_full_compaction) {
  context->rep->is_full_compaction = is_full_compaction;
}

void crocksdb_sst_partitioner_context_set_is_manual_compaction(
    crocksdb_sst_partitioner_context_t* context,
    unsigned char is_manual_compaction) {
  context->rep->is_manual_compaction = is_manual_compaction;
}

void crocksdb_sst_partitioner_context_set_output_level(
    crocksdb_sst_partitioner_context_t* context, int output_level) {
  context->rep->output_level = output_level;
}

void crocksdb_sst_partitioner_context_set_smallest_key(
    crocksdb_sst_partitioner_context_t* context, const char* smallest_key,
    size_t key_len) {
  context->rep->smallest_user_key = Slice(smallest_key, key_len);
}

void crocksdb_sst_partitioner_context_set_largest_key(
    crocksdb_sst_partitioner_context_t* context, const char* largest_key,
    size_t key_len) {
  context->rep->largest_user_key = Slice(largest_key, key_len);
}

struct crocksdb_sst_partitioner_factory_impl_t : public SstPartitionerFactory {
  void* underlying;
  void (*destructor)(void*);
  crocksdb_sst_partitioner_factory_name_cb name_cb;
  crocksdb_sst_partitioner_factory_create_partitioner_cb create_partitioner_cb;

  virtual ~crocksdb_sst_partitioner_factory_impl_t() { destructor(underlying); }

  const char* Name() const override { return name_cb(underlying); }

  std::unique_ptr<SstPartitioner> CreatePartitioner(
      const SstPartitioner::Context& partitioner_context) const override {
    crocksdb_sst_partitioner_context_t context;
    context.rep = const_cast<SstPartitioner::Context*>(&partitioner_context);
    crocksdb_sst_partitioner_t* partitioner =
        create_partitioner_cb(underlying, &context);
    if (partitioner == nullptr) {
      return nullptr;
    }
    std::unique_ptr<SstPartitioner> rep = std::move(partitioner->rep);
    crocksdb_sst_partitioner_destroy(partitioner);
    return rep;
  }
};

crocksdb_sst_partitioner_factory_t* crocksdb_sst_partitioner_factory_create(
    void* underlying, void (*destructor)(void*),
    crocksdb_sst_partitioner_factory_name_cb name_cb,
    crocksdb_sst_partitioner_factory_create_partitioner_cb
        create_partitioner_cb) {
  crocksdb_sst_partitioner_factory_impl_t* factory_impl =
      new crocksdb_sst_partitioner_factory_impl_t;
  factory_impl->underlying = underlying;
  factory_impl->destructor = destructor;
  factory_impl->name_cb = name_cb;
  factory_impl->create_partitioner_cb = create_partitioner_cb;
  crocksdb_sst_partitioner_factory_t* factory =
      new crocksdb_sst_partitioner_factory_t;
  factory->rep.reset(factory_impl);
  return factory;
}

void crocksdb_sst_partitioner_factory_destroy(
    crocksdb_sst_partitioner_factory_t* factory) {
  delete factory;
}

const char* crocksdb_sst_partitioner_factory_name(
    crocksdb_sst_partitioner_factory_t* factory) {
  return factory->rep->Name();
}

crocksdb_sst_partitioner_t* crocksdb_sst_partitioner_factory_create_partitioner(
    crocksdb_sst_partitioner_factory_t* factory,
    crocksdb_sst_partitioner_context_t* context) {
  std::unique_ptr<SstPartitioner> rep =
      factory->rep->CreatePartitioner(*context->rep);
  if (rep == nullptr) {
    return nullptr;
  }
  crocksdb_sst_partitioner_t* partitioner = new crocksdb_sst_partitioner_t;
  partitioner->rep = std::move(rep);
  return partitioner;
}

/* Tools */

void crocksdb_run_ldb_tool(int argc, char** argv,
                           const crocksdb_options_t* opts) {
  LDBTool().Run(argc, argv, opts->rep);
}

void crocksdb_run_sst_dump_tool(int argc, char** argv,
                                const crocksdb_options_t* opts) {
  SSTDumpTool().Run(argc, argv, opts->rep);
}

/* Titan */
struct ctitandb_options_t {
  TitanOptions rep;
};

crocksdb_t* ctitandb_open_column_families(
    const char* name, const ctitandb_options_t* tdb_options,
    int num_column_families, const char** column_family_names,
    const ctitandb_options_t** titan_column_family_options,
    crocksdb_column_family_handle_t** column_family_handles, Status* s) {
  std::vector<TitanCFDescriptor> column_families;
  for (int i = 0; i < num_column_families; i++) {
    column_families.push_back(
        TitanCFDescriptor(std::string(column_family_names[i]),
                          TitanCFOptions(titan_column_family_options[i]->rep)));
  }

  TitanDB* db;
  std::vector<ColumnFamilyHandle*> handles;
  *s = TitanDB::Open(tdb_options->rep, std::string(name), column_families,
                     &handles, &db);
  if (!s->ok()) {
    return nullptr;
  }
  for (size_t i = 0; i < handles.size(); i++) {
    crocksdb_column_family_handle_t* c_handle =
        new crocksdb_column_family_handle_t;
    c_handle->rep = handles[i];
    column_family_handles[i] = c_handle;
  }
  auto result = new crocksdb_t;
  result->rep = db;
  return result;
}

// Caller should make sure `db` is created from ctitandb_open_column_families.
//
// TODO: ctitandb_open_column_family should return a ctitandb_t. Caller can
// use ctitandb_t for titan specific functions.
crocksdb_column_family_handle_t* ctitandb_create_column_family(
    crocksdb_t* db, const ctitandb_options_t* titan_column_family_options,
    const char* column_family_name, Status* s) {
  // Blindly cast db into TitanDB.
  TitanDB* titan_db = reinterpret_cast<TitanDB*>(db->rep);
  auto handle = new crocksdb_column_family_handle_t;
  *s = titan_db->CreateColumnFamily(
      TitanCFDescriptor(std::string(column_family_name),
                        titan_column_family_options->rep),
      &(handle->rep));
  if (s->ok()) {
    return handle;
  } else {
    delete handle;
    return nullptr;
  }
}

/* TitanDBOptions */

TitanDBOptions* ctitandb_options_create() { return new TitanDBOptions; }

void ctitandb_options_destroy(TitanDBOptions* opts) { delete opts; }

ctitandb_options_t* ctitandb_options_copy(ctitandb_options_t* src) {
  if (src == nullptr) {
    return nullptr;
  }
  return new ctitandb_options_t{src->rep};
}

void ctitandb_options_set_rocksdb_options(
    ctitandb_options_t* opts, const crocksdb_options_t* rocksdb_opts) {
  *(DBOptions*)&opts->rep = rocksdb_opts->rep;
  *(ColumnFamilyOptions*)&opts->rep = rocksdb_opts->rep;
}

ctitandb_options_t* ctitandb_get_titan_options_cf(
    const crocksdb_t* db, crocksdb_column_family_handle_t* column_family) {
  ctitandb_options_t* options = new ctitandb_options_t;
  TitanDB* titan_db = reinterpret_cast<TitanDB*>(db->rep);
  options->rep = titan_db->GetTitanOptions(column_family->rep);
  return options;
}

ctitandb_options_t* ctitandb_get_titan_db_options(crocksdb_t* db) {
  ctitandb_options_t* options = new ctitandb_options_t;
  TitanDB* titan_db = reinterpret_cast<TitanDB*>(db->rep);
  *static_cast<TitanDBOptions*>(&options->rep) = titan_db->GetTitanDBOptions();
  return options;
}

void ctitandb_options_dirname(TitanDBOptions* opts, Slice* name) {
  *name = opts->dirname;
}

void ctitandb_options_set_dirname(TitanDBOptions* opts, Slice name) {
  opts->dirname = name.ToString();
}

uint64_t ctitandb_options_min_blob_size(ctitandb_options_t* opts) {
  return opts->rep.min_blob_size;
}

void ctitandb_options_set_min_blob_size(ctitandb_options_t* opts,
                                        uint64_t size) {
  opts->rep.min_blob_size = size;
}

int ctitandb_options_blob_file_compression(ctitandb_options_t* opts) {
  return opts->rep.blob_file_compression;
}

void ctitandb_options_set_blob_file_compression(ctitandb_options_t* opts,
                                                CompressionType type) {
  opts->rep.blob_file_compression = type;
}

void ctitandb_options_set_compression_options(ctitandb_options_t* opt,
                                              int w_bits, int level,
                                              int strategy, int max_dict_bytes,
                                              int zstd_max_train_bytes) {
  opt->rep.blob_file_compression_options.window_bits = w_bits;
  opt->rep.blob_file_compression_options.level = level;
  opt->rep.blob_file_compression_options.strategy = strategy;
  opt->rep.blob_file_compression_options.max_dict_bytes = max_dict_bytes;
  opt->rep.blob_file_compression_options.zstd_max_train_bytes =
      zstd_max_train_bytes;
}

void ctitandb_options_set_gc_merge_rewrite(ctitandb_options_t* opts,
                                           unsigned char enable) {
  opts->rep.gc_merge_rewrite = enable;
}

void ctitandb_decode_blob_index(const char* value, size_t value_size,
                                ctitandb_blob_index_t* index, Status* s) {
  Slice v(value, value_size);
  BlobIndex bi;
  *s = bi.DecodeFrom(&v);
  if (!s->ok()) {
    return;
  }
  index->file_number = bi.file_number;
  index->blob_offset = bi.blob_handle.offset;
  index->blob_size = bi.blob_handle.size;
}

void ctitandb_encode_blob_index(const ctitandb_blob_index_t* index,
                                char** value, size_t* value_size) {
  BlobIndex bi;
  bi.file_number = index->file_number;
  bi.blob_handle.offset = index->blob_offset;
  bi.blob_handle.size = index->blob_size;
  std::string result;
  bi.EncodeTo(&result);
  *value = CopyString(result);
  *value_size = result.size();
}

void ctitandb_options_set_disable_background_gc(TitanDBOptions* options,
                                                unsigned char disable) {
  options->disable_background_gc = disable;
}

void ctitandb_options_set_level_merge(ctitandb_options_t* options,
                                      unsigned char enable) {
  options->rep.level_merge = enable;
}

void ctitandb_options_set_range_merge(ctitandb_options_t* options,
                                      unsigned char enable) {
  options->rep.range_merge = enable;
}

void ctitandb_options_set_max_sorted_runs(ctitandb_options_t* options,
                                          int size) {
  options->rep.max_sorted_runs = size;
}

void ctitandb_options_set_max_gc_batch_size(ctitandb_options_t* options,
                                            uint64_t size) {
  options->rep.max_gc_batch_size = size;
}

void ctitandb_options_set_min_gc_batch_size(ctitandb_options_t* options,
                                            uint64_t size) {
  options->rep.min_gc_batch_size = size;
}

void ctitandb_options_set_blob_file_discardable_ratio(
    ctitandb_options_t* options, double ratio) {
  options->rep.blob_file_discardable_ratio = ratio;
}

void ctitandb_options_set_sample_file_size_ratio(ctitandb_options_t* options,
                                                 double ratio) {
  options->rep.sample_file_size_ratio = ratio;
}

void ctitandb_options_set_merge_small_file_threshold(
    ctitandb_options_t* options, uint64_t size) {
  options->rep.merge_small_file_threshold = size;
}

void ctitandb_options_set_max_background_gc(TitanDBOptions* options,
                                            int32_t size) {
  options->max_background_gc = size;
}

void ctitandb_options_set_purge_obsolete_files_period_sec(
    TitanDBOptions* options, unsigned int period) {
  options->purge_obsolete_files_period_sec = period;
}

void ctitandb_options_set_blob_cache(ctitandb_options_t* options,
                                     crocksdb_cache_t* cache) {
  if (cache) {
    options->rep.blob_cache = cache->rep;
  }
}

size_t ctitandb_options_get_blob_cache_usage(ctitandb_options_t* opt) {
  if (opt && opt->rep.blob_cache != nullptr) {
    return opt->rep.blob_cache->GetUsage();
  }
  return 0;
}

void ctitandb_options_set_blob_cache_capacity(ctitandb_options_t* opt,
                                              size_t capacity, Status* s) {
  if (opt && opt->rep.blob_cache != nullptr) {
    opt->rep.blob_cache->SetCapacity(capacity);
    *s = Status::OK();
  } else {
    *s = Status::InvalidArgument("Blob cache was disabled.");
  }
}

size_t ctitandb_options_get_blob_cache_capacity(ctitandb_options_t* opt) {
  if (opt && opt->rep.blob_cache != nullptr) {
    return opt->rep.blob_cache->GetCapacity();
  }
  return 0;
}

void ctitandb_options_set_discardable_ratio(ctitandb_options_t* options,
                                            double ratio) {
  options->rep.blob_file_discardable_ratio = ratio;
}

void ctitandb_options_set_sample_ratio(ctitandb_options_t* options,
                                       double ratio) {
  options->rep.sample_file_size_ratio = ratio;
}

void ctitandb_options_set_blob_run_mode(ctitandb_options_t* options,
                                        TitanBlobRunMode mode) {
  options->rep.blob_run_mode = mode;
}

/* TitanReadOptions */

void ctitandb_readoptions_init(TitanReadOptions* opt) {
  *opt = TitanReadOptions();
}

crocksdb_iterator_t* ctitandb_create_iterator(
    crocksdb_t* db, const TitanReadOptions* titan_options) {
  crocksdb_iterator_t* result = new crocksdb_iterator_t;
  result->rep = static_cast<TitanDB*>(db->rep)->NewIterator(*titan_options);
  return result;
}

crocksdb_iterator_t* ctitandb_create_iterator_cf(
    crocksdb_t* db, const TitanReadOptions* titan_options,
    crocksdb_column_family_handle_t* column_family) {
  crocksdb_iterator_t* result = new crocksdb_iterator_t;
  result->rep = static_cast<TitanDB*>(db->rep)->NewIterator(*titan_options,
                                                            column_family->rep);
  return result;
}

void ctitandb_create_iterators(
    crocksdb_t* db, const TitanReadOptions* titan_options,
    crocksdb_column_family_handle_t** column_families,
    crocksdb_iterator_t** iterators, size_t size, Status* s) {
  std::vector<ColumnFamilyHandle*> column_families_vec(size);
  for (size_t i = 0; i < size; i++) {
    column_families_vec.push_back(column_families[i]->rep);
  }

  std::vector<Iterator*> res;
  *s = static_cast<TitanDB*>(db->rep)->NewIterators(*titan_options,
                                                    column_families_vec, &res);
  if (!s->ok()) {
    for (size_t i = 0; i < res.size(); i++) {
      delete res[i];
    }
    return;
  }
  assert(res.size() == size);

  for (size_t i = 0; i < size; i++) {
    iterators[i] = new crocksdb_iterator_t;
    iterators[i]->rep = res[i];
  }
}

void ctitandb_delete_files_in_range(crocksdb_t* db, const char* start_key,
                                    size_t start_key_len, const char* limit_key,
                                    size_t limit_key_len,
                                    unsigned char include_end, Status* s) {
  Slice a, b;
  RangePtr range(
      start_key ? (a = Slice(start_key, start_key_len), &a) : nullptr,
      limit_key ? (b = Slice(limit_key, limit_key_len), &b) : nullptr);

  *s = static_cast<TitanDB*>(db->rep)->DeleteFilesInRanges(
      db->rep->DefaultColumnFamily(), &range, 1, include_end);
}

void ctitandb_delete_files_in_range_cf(
    crocksdb_t* db, crocksdb_column_family_handle_t* column_family,
    const char* start_key, size_t start_key_len, const char* limit_key,
    size_t limit_key_len, unsigned char include_end, Status* s) {
  Slice a, b;
  RangePtr range(
      start_key ? (a = Slice(start_key, start_key_len), &a) : nullptr,
      limit_key ? (b = Slice(limit_key, limit_key_len), &b) : nullptr);

  *s = static_cast<TitanDB*>(db->rep)->DeleteFilesInRanges(
      column_family->rep, &range, 1, include_end);
}

void ctitandb_delete_files_in_ranges_cf(
    crocksdb_t* db, crocksdb_column_family_handle_t* cf,
    const char* const* start_keys, const size_t* start_keys_lens,
    const char* const* limit_keys, const size_t* limit_keys_lens,
    size_t num_ranges, unsigned char include_end, Status* s) {
  std::vector<Slice> starts(num_ranges);
  std::vector<Slice> limits(num_ranges);
  std::vector<RangePtr> ranges(num_ranges);
  for (auto i = 0; i < num_ranges; i++) {
    const Slice* start = nullptr;
    if (start_keys[i]) {
      starts[i] = Slice(start_keys[i], start_keys_lens[i]);
      start = &starts[i];
    }
    const Slice* limit = nullptr;
    if (limit_keys[i]) {
      limits[i] = Slice(limit_keys[i], limit_keys_lens[i]);
      limit = &limits[i];
    }
    ranges[i] = RangePtr(start, limit);
  }
  *s = static_cast<TitanDB*>(db->rep)->DeleteFilesInRanges(
      cf->rep, &ranges[0], num_ranges, include_end);
}

void ctitandb_delete_blob_files_in_range(crocksdb_t* db, const char* start_key,
                                         size_t start_key_len,
                                         const char* limit_key,
                                         size_t limit_key_len,
                                         unsigned char include_end, Status* s) {
  Slice a, b;
  RangePtr range(
      start_key ? (a = Slice(start_key, start_key_len), &a) : nullptr,
      limit_key ? (b = Slice(limit_key, limit_key_len), &b) : nullptr);

  *s = static_cast<TitanDB*>(db->rep)->DeleteBlobFilesInRanges(
      db->rep->DefaultColumnFamily(), &range, 1, include_end);
}

void ctitandb_delete_blob_files_in_range_cf(
    crocksdb_t* db, crocksdb_column_family_handle_t* column_family,
    const char* start_key, size_t start_key_len, const char* limit_key,
    size_t limit_key_len, unsigned char include_end, Status* s) {
  Slice a, b;
  RangePtr range(
      start_key ? (a = Slice(start_key, start_key_len), &a) : nullptr,
      limit_key ? (b = Slice(limit_key, limit_key_len), &b) : nullptr);

  *s = static_cast<TitanDB*>(db->rep)->DeleteBlobFilesInRanges(
      column_family->rep, &range, 1, include_end);
}

void ctitandb_delete_blob_files_in_ranges_cf(
    crocksdb_t* db, crocksdb_column_family_handle_t* cf,
    const char* const* start_keys, const size_t* start_keys_lens,
    const char* const* limit_keys, const size_t* limit_keys_lens,
    size_t num_ranges, unsigned char include_end, Status* s) {
  std::vector<Slice> starts(num_ranges);
  std::vector<Slice> limits(num_ranges);
  std::vector<RangePtr> ranges(num_ranges);
  for (auto i = 0; i < num_ranges; i++) {
    const Slice* start = nullptr;
    if (start_keys[i]) {
      starts[i] = Slice(start_keys[i], start_keys_lens[i]);
      start = &starts[i];
    }
    const Slice* limit = nullptr;
    if (limit_keys[i]) {
      limits[i] = Slice(limit_keys[i], limit_keys_lens[i]);
      limit = &limits[i];
    }
    ranges[i] = RangePtr(start, limit);
  }
  *s = static_cast<TitanDB*>(db->rep)->DeleteBlobFilesInRanges(
      cf->rep, &ranges[0], num_ranges, include_end);
}

void crocksdb_free_cplus_array(const char* arr) { delete[] arr; }

const char* crocksdb_to_cplus_array(Slice s) {
  char* const result = new char[s.size() + 1];  // +1 for null terminator
  memcpy(result, s.data(), s.size());
  result[s.size()] = '\0';  // null terminator for C style string
  return result;
}

}  // end extern "C"
