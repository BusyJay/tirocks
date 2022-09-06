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

#include <iostream>
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
using rocksdb::PerfFlags;
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

struct crocksdb_backup_engine_info_t {
  std::vector<BackupInfo> rep;
};
struct crocksdb_restore_options_t {
  RestoreOptions rep;
};
struct crocksdb_fifo_compaction_options_t {
  CompactionOptionsFIFO rep;
};
struct crocksdb_column_family_descriptor {
  ColumnFamilyDescriptor rep;
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
struct crocksdb_cache_t {
  shared_ptr<Cache> rep;
};
struct crocksdb_statistics_t {
  std::shared_ptr<Statistics> statistics;
};
struct crocksdb_livefiles_t {
  std::vector<LiveFileMetaData> rep;
};
struct crocksdb_sequential_file_t {
  SequentialFile* rep;
};
struct crocksdb_ratelimiter_t {
  std::shared_ptr<RateLimiter> rep;
};

struct crocksdb_keyversions_t {
  std::vector<KeyVersion> rep;
};

void crocksdb_property_name_num_files_at_level_prefix(Slice* s) {
  *s = DB::Properties::kNumFilesAtLevelPrefix;
}

void crocksdb_property_name_compression_ratio_at_level_prefix(Slice* s) {
  *s = DB::Properties::kCompressionRatioAtLevelPrefix;
}

void crocksdb_property_name_stats(Slice* s) { *s = DB::Properties::kStats; }

void crocksdb_property_name_ss_tables(Slice* s) {
  *s = DB::Properties::kSSTables;
}

void crocksdb_property_name_cf_stats(Slice* s) {
  *s = DB::Properties::kCFStats;
}

void crocksdb_property_name_cf_stats_no_file_histogram(Slice* s) {
  *s = DB::Properties::kCFStatsNoFileHistogram;
}

void crocksdb_property_name_cf_file_histogram(Slice* s) {
  *s = DB::Properties::kCFFileHistogram;
}

void crocksdb_property_name_db_stats(Slice* s) {
  *s = DB::Properties::kDBStats;
}

void crocksdb_property_name_level_stats(Slice* s) {
  *s = DB::Properties::kLevelStats;
}

void crocksdb_property_name_num_immutable_mem_table(Slice* s) {
  *s = DB::Properties::kNumImmutableMemTable;
}

void crocksdb_property_name_num_immutable_mem_table_flushed(Slice* s) {
  *s = DB::Properties::kNumImmutableMemTableFlushed;
}

void crocksdb_property_name_mem_table_flush_pending(Slice* s) {
  *s = DB::Properties::kMemTableFlushPending;
}

void crocksdb_property_name_num_running_flushes(Slice* s) {
  *s = DB::Properties::kNumRunningFlushes;
}

void crocksdb_property_name_compaction_pending(Slice* s) {
  *s = DB::Properties::kCompactionPending;
}

void crocksdb_property_name_num_running_compactions(Slice* s) {
  *s = DB::Properties::kNumRunningCompactions;
}

void crocksdb_property_name_background_errors(Slice* s) {
  *s = DB::Properties::kBackgroundErrors;
}

void crocksdb_property_name_cur_size_active_mem_table(Slice* s) {
  *s = DB::Properties::kCurSizeActiveMemTable;
}

void crocksdb_property_name_cur_size_all_mem_tables(Slice* s) {
  *s = DB::Properties::kCurSizeAllMemTables;
}

void crocksdb_property_name_size_all_mem_tables(Slice* s) {
  *s = DB::Properties::kSizeAllMemTables;
}

void crocksdb_property_name_num_entries_active_mem_table(Slice* s) {
  *s = DB::Properties::kNumEntriesActiveMemTable;
}

void crocksdb_property_name_num_entries_imm_mem_tables(Slice* s) {
  *s = DB::Properties::kNumEntriesImmMemTables;
}

void crocksdb_property_name_num_deletes_active_mem_table(Slice* s) {
  *s = DB::Properties::kNumDeletesActiveMemTable;
}

void crocksdb_property_name_num_deletes_imm_mem_tables(Slice* s) {
  *s = DB::Properties::kNumDeletesImmMemTables;
}

void crocksdb_property_name_estimate_num_keys(Slice* s) {
  *s = DB::Properties::kEstimateNumKeys;
}

void crocksdb_property_name_estimate_table_readers_mem(Slice* s) {
  *s = DB::Properties::kEstimateTableReadersMem;
}

void crocksdb_property_name_is_file_deletions_enabled(Slice* s) {
  *s = DB::Properties::kIsFileDeletionsEnabled;
}

void crocksdb_property_name_num_snapshots(Slice* s) {
  *s = DB::Properties::kNumSnapshots;
}

void crocksdb_property_name_oldest_snapshot_time(Slice* s) {
  *s = DB::Properties::kOldestSnapshotTime;
}

void crocksdb_property_name_oldest_snapshot_sequence(Slice* s) {
  *s = DB::Properties::kOldestSnapshotSequence;
}

void crocksdb_property_name_num_live_versions(Slice* s) {
  *s = DB::Properties::kNumLiveVersions;
}

void crocksdb_property_name_current_super_version_number(Slice* s) {
  *s = DB::Properties::kCurrentSuperVersionNumber;
}

void crocksdb_property_name_estimate_live_data_size(Slice* s) {
  *s = DB::Properties::kEstimateLiveDataSize;
}

void crocksdb_property_name_min_log_number_to_keep(Slice* s) {
  *s = DB::Properties::kMinLogNumberToKeep;
}

void crocksdb_property_name_min_obsolete_sst_number_to_keep(Slice* s) {
  *s = DB::Properties::kMinObsoleteSstNumberToKeep;
}

void crocksdb_property_name_total_sst_files_size(Slice* s) {
  *s = DB::Properties::kTotalSstFilesSize;
}

void crocksdb_property_name_live_sst_files_size(Slice* s) {
  *s = DB::Properties::kLiveSstFilesSize;
}

void crocksdb_property_name_base_level(Slice* s) {
  *s = DB::Properties::kBaseLevel;
}

void crocksdb_property_name_estimate_pending_compaction_bytes(Slice* s) {
  *s = DB::Properties::kEstimatePendingCompactionBytes;
}

void crocksdb_property_name_aggregated_table_properties(Slice* s) {
  *s = DB::Properties::kAggregatedTableProperties;
}

void crocksdb_property_name_aggregated_table_properties_at_level(Slice* s) {
  *s = DB::Properties::kAggregatedTablePropertiesAtLevel;
}

void crocksdb_property_name_actual_delayed_write_rate(Slice* s) {
  *s = DB::Properties::kActualDelayedWriteRate;
}

void crocksdb_property_name_is_write_stopped(Slice* s) {
  *s = DB::Properties::kIsWriteStopped;
}

void crocksdb_property_name_is_write_stalled(Slice* s) {
  *s = DB::Properties::kIsWriteStalled;
}

void crocksdb_property_name_estimate_oldest_key_time(Slice* s) {
  *s = DB::Properties::kEstimateOldestKeyTime;
}

void crocksdb_property_name_block_cache_capacity(Slice* s) {
  *s = DB::Properties::kBlockCacheCapacity;
}

void crocksdb_property_name_block_cache_usage(Slice* s) {
  *s = DB::Properties::kBlockCacheUsage;
}

void crocksdb_property_name_block_cache_pinned_usage(Slice* s) {
  *s = DB::Properties::kBlockCachePinnedUsage;
}

void crocksdb_property_name_options_statistics(Slice* s) {
  *s = DB::Properties::kOptionsStatistics;
}

struct crocksdb_map_property_t {
  std::map<std::string, std::string> rep;
};

struct crocksdb_compactionfilter_t : public CompactionFilter {
  void* state_;
  void (*destructor_)(void*);
  CompactionFilter::Decision (*filter_)(void*, int level, Slice key,
                                        SequenceNumber seqno,
                                        CompactionFilter::ValueType value_type,
                                        Slice existing_value, Slice* new_value,
                                        Slice* skip_until);

  const char* (*name_)(void*);

  virtual ~crocksdb_compactionfilter_t() { (*destructor_)(state_); }

  virtual Decision FilterV3(int level, const Slice& key, SequenceNumber seqno,
                            ValueType value_type, const Slice& existing_value,
                            std::string* new_value,
                            std::string* skip_until) const override {
    Slice new_value_slice;
    Slice skip_until_slice;
    size_t new_value_length, skip_until_length = 0;

    Decision result =
        (*filter_)(state_, level, key, seqno, value_type, existing_value,
                   &new_value_slice, &skip_until_slice);
    if (result == Decision::kChangeValue) {
      new_value->assign(new_value_slice.data(), new_value_slice.size());
    } else if (result == Decision::kRemoveAndSkipUntil) {
      skip_until->assign(skip_until_slice.data(), skip_until_slice.size());
    }
    return result;
  }

  virtual const char* Name() const override { return (*name_)(state_); }
};

struct CRocksDBCompactionFilterFactory : public CompactionFilterFactory {
  void* state_;
  void (*destructor_)(void*);
  CompactionFilter* (*create_compaction_filter_)(
      void*, const CompactionFilter::Context* context);
  bool (*should_filter_table_file_creation_)(void*,
                                             TableFileCreationReason reason);
  const char* (*name_)(void*);

  virtual ~CRocksDBCompactionFilterFactory() { (*destructor_)(state_); }

  virtual std::unique_ptr<CompactionFilter> CreateCompactionFilter(
      const CompactionFilter::Context& context) override {
    CompactionFilter* cf = (*create_compaction_filter_)(state_, &context);
    return std::unique_ptr<CompactionFilter>(cf);
  }

  virtual bool ShouldFilterTableFileCreation(
      TableFileCreationReason reason) const override {
    return (*should_filter_table_file_creation_)(state_, reason);
  }

  virtual const char* Name() const override { return (*name_)(state_); }
};

struct crocksdb_compactionfilterfactory_t {
  std::shared_ptr<CompactionFilterFactory> rep;
};

struct crocksdb_comparator_t : public Comparator {
  void* state_;
  void (*destructor_)(void*);
  int (*compare_)(void*, Slice lhs, Slice rhs);
  const char* (*name_)(void*);

  virtual ~crocksdb_comparator_t() { (*destructor_)(state_); }

  virtual int Compare(const Slice& a, const Slice& b) const override {
    return (*compare_)(state_, a, b);
  }

  virtual const char* Name() const override { return (*name_)(state_); }

  // No-ops since the C binding does not support key shortening methods.
  virtual void FindShortestSeparator(std::string*,
                                     const Slice&) const override {}
  virtual void FindShortSuccessor(std::string*) const override {}
};

struct crocksdb_filterpolicy_t {
  std::shared_ptr<const FilterPolicy> policy;
};

struct FilterPolicyWrapper : public FilterPolicy {
  void* state_;
  void (*destructor_)(void*);
  const char* (*name_)(void*);
  char* (*create_)(void*, const char* const* key_array,
                   const size_t* key_length_array, int num_keys,
                   size_t* filter_length);
  unsigned char (*key_match_)(void*, const char* key, size_t length,
                              const char* filter, size_t filter_length);
  void (*delete_filter_)(void*, const char* filter, size_t filter_length);

  virtual ~FilterPolicyWrapper() { (*destructor_)(state_); }

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

void slice_collect(void* target, Slice s) {
  ((std::string*)target)->append(s.data(), s.size());
}

struct CRocksDBMergeOperator : public MergeOperator {
  void* state_;
  void (*destructor_)(void*);
  name_cb name_;
  full_merge_cb full_merge_;
  partial_merge_cb partial_merge_;
  partial_merge_mult_cb partial_merge_mult_;
  allow_single_operand_cb allow_single_operand_;
  should_merge_cb should_merge_;

  virtual ~CRocksDBMergeOperator() { (*destructor_)(state_); }

  virtual const char* Name() const override { return (*name_)(state_); }

  virtual bool FullMergeV2(const MergeOperationInput& merge_in,
                           MergeOperationOutput* merge_out) const override {
    return full_merge_(state_, &merge_in, merge_out);
  }

  virtual bool PartialMergeMulti(const Slice& key,
                                 const std::deque<Slice>& operand_list,
                                 std::string* new_value,
                                 Logger*) const override {
    std::vector<Slice> operand_vec(operand_list.size());
    for (size_t i = 0; i < operand_list.size(); i++) {
      operand_vec.push_back(operand_list[i]);
    }
    return partial_merge_mult_(state_, key, operand_vec.data(),
                               operand_vec.size(), new_value, slice_collect);
  }

  bool PartialMerge(const Slice& key, const Slice& left_op,
                    const Slice& right_op, std::string* new_value,
                    Logger* /*logger*/) const override {
    return partial_merge_(state_, key, left_op, right_op, new_value,
                          slice_collect);
  }

  bool AllowSingleOperand() const override {
    return allow_single_operand_(state_);
  }

  bool ShouldMerge(const std::vector<Slice>& ops) const override {
    return should_merge_(state_, ops.data(), ops.size());
  }
};

struct crocksdb_mergeoperator_t {
  std::shared_ptr<MergeOperator> rep;
};

struct CRocksDBSliceTransform : public SliceTransform {
  void* state_;
  void (*destructor_)(void*);
  void (*transform_)(void*, Slice, Slice* dst);
  bool (*in_domain_)(void*, const Slice);
  bool (*full_length_enabled_)(void*, size_t*);
  bool (*same_result_when_appended_)(void*, Slice);
  const char* (*name_)(void*);

  virtual ~CRocksDBSliceTransform() { (*destructor_)(state_); }

  virtual const char* Name() const override { return (*name_)(state_); }

  virtual Slice Transform(const Slice& src) const override {
    Slice dst;
    (*transform_)(state_, src, &dst);
    return dst;
  }

  virtual bool InDomain(const Slice& src) const override {
    return (*in_domain_)(state_, src);
  }

  virtual bool FullLengthEnabled(size_t* len) const override {
    return (*full_length_enabled_)(state_, len);
  }

  virtual bool SameResultWhenAppended(const Slice& prefix) const override {
    return (*same_result_when_appended_)(state_, prefix);
  }
};

struct crocksdb_slicetransform_t {
  std::shared_ptr<const SliceTransform> rep;
};

struct crocksdb_universal_compaction_options_t {
  rocksdb::CompactionOptionsUniversal* rep;
};

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

DB* crocksdb_open(const Options* options, Slice name, Status* s) {
  DB* db;
  *s = DB::Open(*options, name.ToString(), &db);
  if (!s->ok()) {
    return nullptr;
  }
  return db;
}

DB* crocksdb_open_with_ttl(const Options* options, Slice name, int ttl,
                           bool read_only, Status* s) {
  DBWithTTL* db;
  *s = DBWithTTL::Open(*options, name.ToString(), &db, ttl, read_only);
  if (!s->ok()) {
    return nullptr;
  }
  return db;
}

DB* crocksdb_open_for_read_only(const Options* options, Slice name,
                                bool error_if_log_file_exist, Status* s) {
  DB* db;
  *s = DB::OpenForReadOnly(*options, name.ToString(), &db,
                           error_if_log_file_exist);
  if (!s->ok()) {
    return nullptr;
  }
  return db;
}

void crocksdb_resume(DB* db, Status* s) { *s = db->Resume(); }

BackupEngine* crocksdb_backup_engine_open(const Options* options, Slice path,
                                          Status* s) {
  BackupEngine* be;
  *s = BackupEngine::Open(options->env, BackupableDBOptions(path.ToString()),
                          &be);
  if (!s->ok()) {
    return nullptr;
  }
  return be;
}

void crocksdb_backup_engine_create_new_backup(BackupEngine* be, DB* db,
                                              Status* s) {
  *s = be->CreateNewBackup(db);
}

void crocksdb_backup_engine_purge_old_backups(BackupEngine* be,
                                              uint32_t num_backups_to_keep,
                                              Status* s) {
  *s = be->PurgeOldBackups(num_backups_to_keep);
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
    BackupEngine* be, Slice db_dir, Slice wal_dir,
    const crocksdb_restore_options_t* restore_options, Status* s) {
  *s = be->RestoreDBFromLatestBackup(db_dir.ToString(), wal_dir.ToString(),
                                     restore_options->rep);
}

const crocksdb_backup_engine_info_t* crocksdb_backup_engine_get_backup_info(
    BackupEngine* be) {
  crocksdb_backup_engine_info_t* result = new crocksdb_backup_engine_info_t;
  be->GetBackupInfo(&result->rep);
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

void crocksdb_backup_engine_close(BackupEngine* be) { delete be; }

void crocksdb_close(DB* db, Status* s) { *s = db->Close(); }

void crocksdb_destroy(DB* db) { delete db; }

void crocksdb_pause_bg_work(DB* db, Status* s) {
  *s = db->PauseBackgroundWork();
}

void crocksdb_continue_bg_work(DB* db, Status* s) {
  *s = db->ContinueBackgroundWork();
}

DB* crocksdb_open_column_families(
    const DBOptions* db_options, Slice name, int num_column_families,
    const Slice* column_family_names,
    const ColumnFamilyOptions* const* column_family_options,
    ColumnFamilyHandle** column_family_handles, Status* s) {
  std::vector<ColumnFamilyDescriptor> column_families;
  for (int i = 0; i < num_column_families; i++) {
    column_families.push_back(ColumnFamilyDescriptor(
        column_family_names[i].ToString(), *column_family_options[i]));
  }

  DB* db;
  std::vector<ColumnFamilyHandle*> handles;
  *s = DB::Open(*db_options, name.ToString(), column_families, &handles, &db);
  if (!s->ok()) {
    return nullptr;
  }

  for (size_t i = 0; i < handles.size(); i++) {
    column_family_handles[i] = handles[i];
  }
  return db;
}

DB* crocksdb_open_column_families_with_ttl(
    const DBOptions* db_options, Slice name, int num_column_families,
    const Slice* column_family_names,
    const ColumnFamilyOptions* const* column_family_options,
    const int32_t* ttl_array, bool read_only,
    ColumnFamilyHandle** column_family_handles, Status* s) {
  std::vector<ColumnFamilyDescriptor> column_families;
  std::vector<int32_t> ttls;
  for (int i = 0; i < num_column_families; i++) {
    column_families.push_back(ColumnFamilyDescriptor(
        column_family_names[i].ToString(), *column_family_options[i]));
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
    column_family_handles[i] = handles[i];
  }
  return db;
}

DB* crocksdb_open_for_read_only_column_families(
    const DBOptions* db_options, Slice name, int num_column_families,
    const Slice* column_family_names,
    const ColumnFamilyOptions* const* column_family_options,
    ColumnFamilyHandle** column_family_handles, bool error_if_log_file_exist,
    Status* s) {
  std::vector<ColumnFamilyDescriptor> column_families;
  for (int i = 0; i < num_column_families; i++) {
    column_families.push_back(ColumnFamilyDescriptor(
        column_family_names[i].ToString(), *column_family_options[i]));
  }

  DB* db;
  std::vector<ColumnFamilyHandle*> handles;
  *s = DB::OpenForReadOnly(*db_options, name.ToString(), column_families,
                           &handles, &db, error_if_log_file_exist);
  if (!s->ok()) {
    return nullptr;
  }

  for (size_t i = 0; i < handles.size(); i++) {
    column_family_handles[i] = handles[i];
  }
  return db;
}

void crocksdb_list_column_families(const DBOptions* options, Slice name,
                                   void* ctx, bytes_receiver_cb fp, Status* s) {
  std::vector<std::string> fams;
  *s = DB::ListColumnFamilies(*options, name.ToString(), &fams);
  if (!s->ok()) {
    return;
  }

  for (size_t i = 0; i < fams.size(); i++) {
    fp(ctx, fams[i]);
  }
}

ColumnFamilyHandle* crocksdb_create_column_family(
    DB* db, const ColumnFamilyOptions* column_family_options,
    Slice column_family_name, Status* s) {
  ColumnFamilyHandle* handle;
  *s = db->CreateColumnFamily(*column_family_options,
                              column_family_name.ToString(), &handle);
  if (!s->ok()) {
    return nullptr;
  }
  return handle;
}

void crocksdb_drop_column_family(DB* db, ColumnFamilyHandle* handle,
                                 Status* s) {
  *s = db->DropColumnFamily(handle);
}

ColumnFamilyHandle* crocksdb_get_default_column_family(DB* db) {
  return db->DefaultColumnFamily();
}

uint32_t crocksdb_column_family_handle_id(ColumnFamilyHandle* handle) {
  return handle->GetID();
}

void crocksdb_column_family_handle_name(ColumnFamilyHandle* handle,
                                        Slice* name) {
  *name = handle->GetName();
}

void crocksdb_column_family_handle_destroy(DB* db, ColumnFamilyHandle* handle,
                                           Status* s) {
  *s = db->DestroyColumnFamilyHandle(handle);
}

void crocksdb_put(DB* db, const WriteOptions* options, Slice key, Slice val,
                  Status* s) {
  *s = db->Put(*options, key, val);
}

void crocksdb_put_cf(DB* db, const WriteOptions* options,
                     ColumnFamilyHandle* column_family, Slice key, Slice val,
                     Status* s) {
  *s = db->Put(*options, column_family, key, val);
}

void crocksdb_delete(DB* db, const WriteOptions* options, Slice key,
                     Status* s) {
  *s = db->Delete(*options, key);
}

void crocksdb_delete_cf(DB* db, const WriteOptions* options,
                        ColumnFamilyHandle* column_family, Slice key,
                        Status* s) {
  *s = db->Delete(*options, column_family, key);
}

void crocksdb_single_delete(DB* db, const WriteOptions* options, Slice key,
                            Status* s) {
  *s = db->SingleDelete(*options, key);
}

void crocksdb_single_delete_cf(DB* db, const WriteOptions* options,
                               ColumnFamilyHandle* column_family, Slice key,
                               Status* s) {
  *s = db->SingleDelete(*options, column_family, key);
}

void crocksdb_delete_range_cf(DB* db, const WriteOptions* options,
                              ColumnFamilyHandle* column_family,
                              Slice begin_key, Slice end_key, Status* s) {
  *s = db->DeleteRange(*options, column_family, begin_key, end_key);
}

void crocksdb_merge(DB* db, const WriteOptions* options, const char* key,
                    size_t keylen, const char* val, size_t vallen, Status* s) {
  *s = db->Merge(*options, Slice(key, keylen), Slice(val, vallen));
}

void crocksdb_merge_cf(DB* db, const WriteOptions* options,
                       ColumnFamilyHandle* column_family, const char* key,
                       size_t keylen, const char* val, size_t vallen,
                       Status* s) {
  *s = db->Merge(*options, column_family, Slice(key, keylen),
                 Slice(val, vallen));
}

void crocksdb_write(DB* db, const WriteOptions* options, WriteBatch* batch,
                    Status* s) {
  *s = db->Write(*options, batch);
}

void crocksdb_write_multi_batch(DB* db, const WriteOptions* options,
                                WriteBatch** batches, size_t batch_size,
                                Status* s) {
  std::vector<WriteBatch*> ws;
  for (size_t i = 0; i < batch_size; i++) {
    ws.push_back(batches[i]);
  }
  *s = db->MultiBatchWrite(*options, std::move(ws));
}

void crocksdb_get(DB* db, const ReadOptions* options, Slice key, void* ctx,
                  bytes_receiver_cb fp, Status* s) {
  std::string tmp;
  *s = db->Get(*options, key, &tmp);
  if (s->ok()) {
    fp(ctx, tmp);
  }
}

void crocksdb_get_cf(DB* db, const ReadOptions* options,
                     ColumnFamilyHandle* column_family, Slice key, void* ctx,
                     bytes_receiver_cb fp, Status* s) {
  std::string tmp;
  *s = db->Get(*options, column_family, key, &tmp);
  if (s->ok()) {
    fp(ctx, tmp);
  }
}

void crocksdb_multi_get(DB* db, const ReadOptions* options, size_t num_keys,
                        const Slice* keys_list, void* ctx, bytes_receiver_cb fp,
                        Status* status_list) {
  std::vector<Slice> keys(keys_list, keys_list + num_keys);
  std::vector<std::string> values(num_keys);
  std::vector<Status> statuses = db->MultiGet(*options, keys, &values);
  for (size_t i = 0; i < num_keys; i++) {
    status_list[i] = statuses[i];
    if (status_list[i].ok()) {
      fp(ctx, values[i]);
    }
  }
}

void crocksdb_multi_get_cf(DB* db, const ReadOptions* options,
                           ColumnFamilyHandle** column_families,
                           size_t num_keys, const Slice* keys_list, void* ctx,
                           bytes_receiver_cb fp, Status* status_list) {
  std::vector<Slice> keys(keys_list, keys_list + num_keys);
  std::vector<ColumnFamilyHandle*> cfs(column_families,
                                       column_families + num_keys);
  std::vector<std::string> values(num_keys);
  std::vector<Status> statuses = db->MultiGet(*options, cfs, keys, &values);
  for (size_t i = 0; i < num_keys; i++) {
    status_list[i] = statuses[i];
    if (status_list[i].ok()) {
      fp(ctx, values[i]);
    }
  }
}

Iterator* crocksdb_create_iterator(DB* db, const ReadOptions* options) {
  return db->NewIterator(*options);
}

Iterator* crocksdb_create_iterator_cf(DB* db, const ReadOptions* options,
                                      ColumnFamilyHandle* column_family) {
  return db->NewIterator(*options, column_family);
}

void crocksdb_create_iterators(DB* db, const ReadOptions* opts,
                               ColumnFamilyHandle** column_families,
                               Iterator** iterators, size_t size, Status* s) {
  std::vector<ColumnFamilyHandle*> column_families_vec(column_families,
                                                       column_families + size);

  std::vector<Iterator*> res(size);
  *s = db->NewIterators(*opts, column_families_vec, &res);
  if (!s->ok()) {
    for (size_t i = 0; i < res.size(); i++) {
      delete res[i];
    }
    return;
  }
  assert(res.size() == size);

  for (size_t i = 0; i < size; i++) {
    iterators[i] = res[i];
  }
}

const Snapshot* crocksdb_create_snapshot(DB* db) { return db->GetSnapshot(); }

void crocksdb_release_snapshot(DB* db, const Snapshot* snapshot) {
  db->ReleaseSnapshot(snapshot);
}

void crocksdb_get_snapshot_sequence_number(const Snapshot* snapshot,
                                           SequenceNumber* n) {
  *n = snapshot->GetSequenceNumber();
}

crocksdb_map_property_t* crocksdb_create_map_property() {
  return new crocksdb_map_property_t;
}

void crocksdb_map_property_destroy(crocksdb_map_property_t* info) {
  delete info;
}

bool crocksdb_get_map_property_cf(DB* db, ColumnFamilyHandle* column_family,
                                  Slice property,
                                  crocksdb_map_property_t* info) {
  info->rep.clear();
  return db->GetMapProperty(column_family, property, &info->rep);
}

bool crocksdb_map_property_value(crocksdb_map_property_t* info, Slice propname,
                                 Slice* value) {
  auto iter = info->rep.find(propname.ToString());
  if (iter != info->rep.end()) {
    *value = iter->second;
    return true;
  } else {
    return false;
  }
}

bool crocksdb_map_property_int_value(crocksdb_map_property_t* info,
                                     Slice propname, uint64_t* val) {
  auto iter = info->rep.find(propname.ToString());
  if (iter != info->rep.end()) {
    *val = (uint64_t)stoll(iter->second, nullptr);
    return true;
  } else {
    return false;
  }
}

void crocksdb_property_value(DB* db, Slice propname, void* ctx,
                             bytes_receiver_cb fp) {
  std::string tmp;
  if (db->GetProperty(propname, &tmp)) {
    fp(ctx, tmp);
  }
}

void crocksdb_property_value_cf(DB* db, ColumnFamilyHandle* column_family,
                                Slice propname, void* ctx,
                                bytes_receiver_cb fp) {
  std::string tmp;
  if (db->GetProperty(column_family, Slice(propname), &tmp)) {
    fp(ctx, tmp);
  }
}

bool crocksdb_property_int_value_cf(DB* db, ColumnFamilyHandle* column_family,
                                    Slice propname, uint64_t* val) {
  return db->GetIntProperty(column_family, propname, val);
}

bool crocksdb_property_aggregated_int_value(DB* db, Slice propname,
                                            uint64_t* val) {
  return db->GetAggregatedIntProperty(propname, val);
}

void crocksdb_sizeapproximationoptions_init(SizeApproximationOptions* opt) {
  *opt = SizeApproximationOptions();
}
void crocksdb_approximate_sizes_cf(DB* db, const SizeApproximationOptions* opt,
                                   ColumnFamilyHandle* column_family,
                                   const Range* ranges, int num_ranges,
                                   uint64_t* sizes, Status* s) {
  *s = db->GetApproximateSizes(*opt, column_family, ranges, num_ranges, sizes);
}

void crocksdb_approximate_memtable_stats_cf(DB* db, ColumnFamilyHandle* cf,
                                            const Range* range, uint64_t* count,
                                            uint64_t* size) {
  db->GetApproximateMemTableStats(cf, *range, count, size);
}

void crocksdb_delete_file(DB* db, Slice name, Status* s) {
  *s = db->DeleteFile(name.ToString());
}

crocksdb_livefiles_t* crocksdb_livefiles(DB* db) {
  crocksdb_livefiles_t* result = new crocksdb_livefiles_t;
  db->GetLiveFilesMetaData(&result->rep);
  return result;
}

void crocksdb_compact_range(DB* db, const Slice* start_key,
                            const Slice* limit_key, Status* s) {
  *s = db->CompactRange(CompactRangeOptions(), start_key, limit_key);
}

void crocksdb_compact_range_cf(DB* db, ColumnFamilyHandle* column_family,
                               const Slice* start_key, const Slice* limit_key,
                               Status* s) {
  *s = db->CompactRange(CompactRangeOptions(), column_family, start_key,
                        limit_key);
}

void crocksdb_compact_range_opt(DB* db, const CompactRangeOptions* opt,
                                const Slice* start_key, const Slice* limit_key,
                                Status* s) {
  *s = db->CompactRange(*opt, start_key, limit_key);
}

void crocksdb_compact_range_cf_opt(DB* db, const CompactRangeOptions* opt,
                                   ColumnFamilyHandle* column_family,
                                   const Slice* start_key,
                                   const Slice* limit_key, Status* s) {
  *s = db->CompactRange(*opt, column_family, start_key, limit_key);
}

void crocksdb_flush(DB* db, const FlushOptions* options, Status* s) {
  *s = db->Flush(*options);
}

void crocksdb_flush_cf(DB* db, const FlushOptions* options,
                       ColumnFamilyHandle* column_family, Status* s) {
  *s = db->Flush(*options, column_family);
}

void crocksdb_flush_cfs(DB* db, const FlushOptions* options,
                        ColumnFamilyHandle* const* column_families,
                        int num_handles, Status* s) {
  std::vector<rocksdb::ColumnFamilyHandle*> handles(
      column_families, column_families + num_handles);
  *s = db->Flush(*options, handles);
}

void crocksdb_flush_wal(DB* db, bool sync, Status* s) {
  *s = db->FlushWAL(sync);
}

void crocksdb_sync_wal(DB* db, Status* s) { *s = db->SyncWAL(); }

void crocksdb_get_latest_sequence_number(DB* db, SequenceNumber* n) {
  *n = db->GetLatestSequenceNumber();
}

void crocksdb_disable_file_deletions(DB* db, Status* s) {
  *s = db->DisableFileDeletions();
}

void crocksdb_enable_file_deletions(DB* db, bool force, Status* s) {
  *s = db->EnableFileDeletions(force);
}

DBOptions* crocksdb_get_db_options(DB* db) {
  return new DBOptions(db->GetDBOptions());
}

void crocksdb_set_db_options(DB* db, const Slice* names, const Slice* values,
                             size_t num_options, Status* s) {
  std::unordered_map<std::string, std::string> options;
  for (size_t i = 0; i < num_options; i++) {
    options.emplace(names[i].ToString(), values[i].ToString());
  }
  *s = db->SetDBOptions(options);
}

Options* crocksdb_get_options_cf(const DB* db,
                                 ColumnFamilyHandle* column_family) {
  return new Options(db->GetOptions(column_family));
}

void crocksdb_set_options_cf(DB* db, ColumnFamilyHandle* cf, const Slice* names,
                             const Slice* values, size_t num_options,
                             Status* s) {
  std::unordered_map<std::string, std::string> options;
  for (size_t i = 0; i < num_options; i++) {
    options.emplace(names[i].ToString(), values[i].ToString());
  }
  *s = db->SetOptions(cf, options);
}

void crocksdb_destroy_db(Slice name, const Options* options,
                         const Slice* cf_name,
                         const ColumnFamilyOptions* const* cf_opt,
                         size_t cf_count, Status* s) {
  std::vector<ColumnFamilyDescriptor> desc(cf_count);
  for (size_t i = 0; i < cf_count; ++i) {
    desc.push_back(ColumnFamilyDescriptor(cf_name[i].ToString(), *cf_opt[i]));
  }
  *s = DestroyDB(name.ToString(), *options, desc);
}

void crocksdb_repair_db(Slice name, const DBOptions* options,
                        const Slice* cf_name,
                        const ColumnFamilyOptions* const* cf_opt,
                        size_t cf_count, Status* s) {
  std::vector<ColumnFamilyDescriptor> desc(cf_count);
  for (size_t i = 0; i < cf_count; ++i) {
    desc.push_back(ColumnFamilyDescriptor(cf_name[i].ToString(), *cf_opt[i]));
  }
  *s = RepairDB(name.ToString(), *options, desc);
}

void crocksdb_iter_destroy(Iterator* iter) { delete iter; }

bool crocksdb_iter_valid(const Iterator* iter) { return iter->Valid(); }

void crocksdb_iter_seek_to_first(Iterator* iter) { iter->SeekToFirst(); }

void crocksdb_iter_seek_to_last(Iterator* iter) { iter->SeekToLast(); }

void crocksdb_iter_seek(Iterator* iter, Slice key) { iter->Seek(key); }

void crocksdb_iter_seek_for_prev(Iterator* iter, Slice key) {
  iter->SeekForPrev(key);
}

void crocksdb_iter_next(Iterator* iter) { iter->Next(); }

void crocksdb_iter_prev(Iterator* iter) { iter->Prev(); }

void crocksdb_iter_key(const Iterator* iter, Slice* key) { *key = iter->key(); }

void crocksdb_iter_value(const Iterator* iter, Slice* val) {
  *val = iter->value();
}

bool crocksdb_iter_seqno(const Iterator* iter, SequenceNumber* no) {
  return iter->seqno(no);
}

void crocksdb_iter_get_error(const Iterator* iter, Status* s) {
  *s = iter->status();
}

void crocksdb_iter_refresh(Iterator* iter, Status* s) { *s = iter->Refresh(); }

WriteBatch* crocksdb_writebatch_create() { return new WriteBatch; }

WriteBatch* crocksdb_writebatch_create_with_capacity(size_t reserved_bytes) {
  return new WriteBatch(reserved_bytes);
}

WriteBatch* crocksdb_writebatch_create_from(Slice data) {
  return new WriteBatch(data.ToString());
}

void crocksdb_writebatch_destroy(WriteBatch* b) { delete b; }

void crocksdb_writebatch_clear(WriteBatch* b) { b->Clear(); }

int crocksdb_writebatch_count(WriteBatch* b) { return b->Count(); }

void crocksdb_writebatch_put(WriteBatch* b, Slice key, Slice val, Status* s) {
  *s = b->Put(key, val);
}

void crocksdb_writebatch_put_cf(WriteBatch* b,
                                ColumnFamilyHandle* column_family, Slice key,
                                Slice val, Status* s) {
  *s = b->Put(column_family, key, val);
}

void crocksdb_writebatch_putv(WriteBatch* b, int num_keys, const Slice* keys,
                              int num_values, const Slice* values, Status* s) {
  *s = b->Put(SliceParts(keys, num_keys), SliceParts(values, num_values));
}

void crocksdb_writebatch_putv_cf(WriteBatch* b,
                                 ColumnFamilyHandle* column_family,
                                 int num_keys, const Slice* keys,
                                 int num_values, const Slice* values,
                                 Status* s) {
  *s = b->Put(column_family, SliceParts(keys, num_keys),
              SliceParts(values, num_values));
}

void crocksdb_writebatch_merge(WriteBatch* b, Slice key, Slice val, Status* s) {
  *s = b->Merge(key, val);
}

void crocksdb_writebatch_merge_cf(WriteBatch* b,
                                  ColumnFamilyHandle* column_family, Slice key,
                                  Slice val, Status* s) {
  *s = b->Merge(column_family, key, val);
}

void crocksdb_writebatch_mergev(WriteBatch* b, int num_keys, const Slice* keys,
                                int num_values, const Slice* values,
                                Status* s) {
  *s = b->Merge(SliceParts(keys, num_keys), SliceParts(values, num_values));
}

void crocksdb_writebatch_mergev_cf(WriteBatch* b,
                                   ColumnFamilyHandle* column_family,
                                   int num_keys, const Slice* keys,
                                   int num_values, const Slice* values,
                                   Status* s) {
  *s = b->Merge(column_family, SliceParts(keys, num_keys),
                SliceParts(values, num_values));
}

void crocksdb_writebatch_delete(WriteBatch* b, Slice key, Status* s) {
  *s = b->Delete(key);
}

void crocksdb_writebatch_delete_cf(WriteBatch* b,
                                   ColumnFamilyHandle* column_family, Slice key,
                                   Status* s) {
  *s = b->Delete(column_family, key);
}

void crocksdb_writebatch_single_delete(WriteBatch* b, Slice key, Status* s) {
  *s = b->SingleDelete(key);
}

void crocksdb_writebatch_single_delete_cf(WriteBatch* b,
                                          ColumnFamilyHandle* column_family,
                                          Slice key, Status* s) {
  *s = b->SingleDelete(column_family, key);
}

void crocksdb_writebatch_single_deletev(WriteBatch* b, int num_keys,
                                        const Slice* keys, Status* s) {
  *s = b->SingleDelete(SliceParts(keys, num_keys));
}

void crocksdb_writebatch_single_deletev_cf(WriteBatch* b,
                                           ColumnFamilyHandle* column_family,
                                           int num_keys, const Slice* keys,
                                           Status* s) {
  *s = b->SingleDelete(column_family, SliceParts(keys, num_keys));
}

void crocksdb_writebatch_deletev(WriteBatch* b, int num_keys, const Slice* keys,
                                 Status* s) {
  *s = b->Delete(SliceParts(keys, num_keys));
}

void crocksdb_writebatch_deletev_cf(WriteBatch* b,
                                    ColumnFamilyHandle* column_family,
                                    int num_keys, const Slice* keys,
                                    Status* s) {
  *s = b->Delete(column_family, SliceParts(keys, num_keys));
}

void crocksdb_writebatch_delete_range(WriteBatch* b, Slice start_key,
                                      Slice end_key, Status* s) {
  *s = b->DeleteRange(start_key, end_key);
}

void crocksdb_writebatch_delete_range_cf(WriteBatch* b,
                                         ColumnFamilyHandle* column_family,
                                         Slice start_key, Slice end_key,
                                         Status* s) {
  *s = b->DeleteRange(column_family, start_key, end_key);
}

void crocksdb_writebatch_delete_rangev(WriteBatch* b, int num_start_keys,
                                       const Slice* start_keys,
                                       int num_end_keys, const Slice* end_keys,
                                       Status* s) {
  *s = b->DeleteRange(SliceParts(start_keys, num_start_keys),
                      SliceParts(end_keys, num_end_keys));
}

void crocksdb_writebatch_delete_rangev_cf(WriteBatch* b,
                                          ColumnFamilyHandle* column_family,
                                          int num_start_keys,
                                          const Slice* start_keys,
                                          int num_end_keys,
                                          const Slice* end_keys, Status* s) {
  *s = b->DeleteRange(column_family, SliceParts(start_keys, num_start_keys),
                      SliceParts(end_keys, num_end_keys));
}

void crocksdb_writebatch_put_log_data(WriteBatch* b, Slice blob, Status* s) {
  *s = b->PutLogData(blob);
}

void crocksdb_writebatch_iterate(WriteBatch* b, void* state,
                                 void (*put)(void*, Slice k, Slice v),
                                 void (*deleted)(void*, Slice k), Status* s) {
  class HandlerWrapper : public WriteBatch::Handler {
   public:
    void* state_;
    void (*put_)(void*, Slice k, Slice v);
    void (*deleted_)(void*, Slice k);
    void Put(const Slice& key, const Slice& value) override {
      (*put_)(state_, key, value);
    }
    void Delete(const Slice& key) override { (*deleted_)(state_, key); }
  };
  HandlerWrapper handler;
  handler.state_ = state;
  handler.put_ = put;
  handler.deleted_ = deleted;
  *s = b->Iterate(&handler);
}

void crocksdb_writebatch_iterate_cf(
    WriteBatch* b, void* state, void (*put)(void*, Slice k, Slice v),
    Status (*put_cf)(void*, uint32_t cf, Slice k, Slice v),
    void (*deleted)(void*, Slice k),
    Status (*deleted_cf)(void*, uint32_t cf, Slice k), Status* s) {
  class HandlerWrapper : public WriteBatch::Handler {
   public:
    void* state_;
    void (*put_)(void*, Slice k, Slice v);
    Status (*put_cf_)(void*, uint32_t cf, Slice k, Slice v);
    void (*deleted_)(void*, Slice k);
    Status (*deleted_cf_)(void*, uint32_t cf, Slice k);

    void Put(const Slice& key, const Slice& value) override {
      (*put_)(state_, key, value);
    }

    Status PutCF(uint32_t column_family_id, const Slice& key,
                 const Slice& value) override {
      return (*put_cf_)(state_, column_family_id, key, value);
    }

    void Delete(const Slice& key) override { (*deleted_)(state_, key); }

    Status DeleteCF(uint32_t column_family_id, const Slice& key) override {
      return (*deleted_cf_)(state_, column_family_id, key);
    }
  };
  HandlerWrapper handler;
  handler.state_ = state;
  handler.put_ = put;
  handler.put_cf_ = put_cf;
  handler.deleted_ = deleted;
  handler.deleted_cf_ = deleted_cf;
  *s = b->Iterate(&handler);
}

void crocksdb_writebatch_data(const WriteBatch* b, Slice* data) {
  *data = b->Data();
}

void crocksdb_writebatch_set_save_point(WriteBatch* b) { b->SetSavePoint(); }

void crocksdb_writebatch_pop_save_point(WriteBatch* b, Status* s) {
  *s = b->PopSavePoint();
}

void crocksdb_writebatch_rollback_to_save_point(WriteBatch* b, Status* s) {
  *s = b->RollbackToSavePoint();
}

void crocksdb_writebatch_set_content(WriteBatch* b, Slice data) {
  rocksdb::WriteBatchInternal::SetContents(b, data);
}

void crocksdb_writebatch_append_content(WriteBatch* dest, Slice data,
                                        Status* s) {
  *s = rocksdb::WriteBatchInternal::AppendContents(dest, data);
}

void crocksdb_writebatch_append(WriteBatch* dest, const WriteBatch* src,
                                Status* s) {
  *s = rocksdb::WriteBatchInternal::Append(dest, src);
}

int crocksdb_writebatch_ref_count(Slice data) {
  rocksdb::WriteBatch::WriteBatchRef ref(data);
  return ref.Count();
}

WriteBatch::Iterator* crocksdb_writebatch_ref_iterator_create(Slice data) {
  rocksdb::WriteBatch::WriteBatchRef ref(data);
  return ref.NewIterator();
}

WriteBatch::Iterator* crocksdb_writebatch_iterator_create(WriteBatch* dest) {
  return dest->NewIterator();
}

void crocksdb_writebatch_iterator_destroy(WriteBatch::Iterator* it) {
  delete it;
}

bool crocksdb_writebatch_iterator_valid(WriteBatch::Iterator* it) {
  return it->Valid();
}

void crocksdb_writebatch_iterator_next(WriteBatch::Iterator* it) { it->Next(); }

void crocksdb_writebatch_iterator_key(WriteBatch::Iterator* it, Slice* key) {
  *key = it->Key();
}

void crocksdb_writebatch_iterator_value(WriteBatch::Iterator* it,
                                        Slice* value) {
  *value = it->Value();
}

char crocksdb_writebatch_iterator_value_type(WriteBatch::Iterator* it) {
  return it->GetValueType();
}

uint32_t crocksdb_writebatch_iterator_column_family_id(
    WriteBatch::Iterator* it) {
  return it->GetColumnFamilyId();
}

BlockBasedTableOptions* crocksdb_block_based_options_create() {
  return new BlockBasedTableOptions;
}

void crocksdb_block_based_options_destroy(BlockBasedTableOptions* options) {
  delete options;
}

void crocksdb_block_based_options_set_metadata_block_size(
    BlockBasedTableOptions* options, uint64_t block_size) {
  options->metadata_block_size = block_size;
}

void crocksdb_block_based_options_set_block_size(
    BlockBasedTableOptions* options, size_t block_size) {
  options->block_size = block_size;
}

void crocksdb_block_based_options_set_block_size_deviation(
    BlockBasedTableOptions* options, int block_size_deviation) {
  options->block_size_deviation = block_size_deviation;
}

void crocksdb_block_based_options_set_block_restart_interval(
    BlockBasedTableOptions* options, int block_restart_interval) {
  options->block_restart_interval = block_restart_interval;
}

void crocksdb_block_based_options_set_filter_policy(
    BlockBasedTableOptions* options, crocksdb_filterpolicy_t* filter_policy) {
  options->filter_policy = filter_policy->policy;
}

void crocksdb_block_based_options_set_no_block_cache(
    BlockBasedTableOptions* options, bool no_block_cache) {
  options->no_block_cache = no_block_cache;
}

void crocksdb_block_based_options_set_block_cache(
    BlockBasedTableOptions* options, crocksdb_cache_t* block_cache) {
  options->block_cache = block_cache->rep;
}

void crocksdb_block_based_options_set_block_cache_compressed(
    BlockBasedTableOptions* options, crocksdb_cache_t* block_cache_compressed) {
  options->block_cache_compressed = block_cache_compressed->rep;
}

void crocksdb_block_based_options_set_whole_key_filtering(
    BlockBasedTableOptions* options, bool v) {
  options->whole_key_filtering = v;
}

void crocksdb_block_based_options_set_format_version(
    BlockBasedTableOptions* options, uint32_t v) {
  options->format_version = v;
}

void crocksdb_block_based_options_set_index_type(
    BlockBasedTableOptions* options, BlockBasedTableOptions::IndexType v) {
  options->index_type = v;
}

void crocksdb_block_based_options_set_hash_index_allow_collision(
    BlockBasedTableOptions* options, bool v) {
  options->hash_index_allow_collision = v;
}

void crocksdb_block_based_options_set_partition_filters(
    BlockBasedTableOptions* options, bool v) {
  options->partition_filters = v;
}

void crocksdb_block_based_options_set_cache_index_and_filter_blocks(
    BlockBasedTableOptions* options, bool v) {
  options->cache_index_and_filter_blocks = v;
}

void crocksdb_block_based_options_set_pin_top_level_index_and_filter(
    BlockBasedTableOptions* options, bool v) {
  options->pin_top_level_index_and_filter = v;
}

void crocksdb_block_based_options_set_cache_index_and_filter_blocks_with_high_priority(
    BlockBasedTableOptions* options, bool v) {
  options->cache_index_and_filter_blocks_with_high_priority = v;
}

void crocksdb_block_based_options_set_pin_l0_filter_and_index_blocks_in_cache(
    BlockBasedTableOptions* options, bool v) {
  options->pin_l0_filter_and_index_blocks_in_cache = v;
}

void crocksdb_block_based_options_set_read_amp_bytes_per_bit(
    BlockBasedTableOptions* options, uint32_t v) {
  options->read_amp_bytes_per_bit = v;
}

void crocksdb_block_based_options_set_prepopulate_block_cache(
    BlockBasedTableOptions* options,
    BlockBasedTableOptions::PrepopulateBlockCache v) {
  options->prepopulate_block_cache = v;
}

void crocksdb_block_based_options_set_checksum(BlockBasedTableOptions* options,
                                               ChecksumType v) {
  options->checksum = v;
}

struct crocksdb_tablefactory_t {
  shared_ptr<TableFactory> factory;
};

crocksdb_tablefactory_t* crocksdb_tablefactory_create_block_based(
    const BlockBasedTableOptions* opt) {
  auto result = new crocksdb_tablefactory_t;
  result->factory.reset(NewBlockBasedTableFactory(*opt));
  return result;
}

crocksdb_tablefactory_t* crocksdb_tablefactory_create_plain(
    const PlainTableOptions* opt) {
  auto result = new crocksdb_tablefactory_t;
  result->factory.reset(NewPlainTableFactory(*opt));
  return result;
}
void crocksdb_tablefactory_destroy(crocksdb_tablefactory_t* factory) {
  delete factory;
}

void crocksdb_options_set_table_factory(ColumnFamilyOptions* opt,
                                        const crocksdb_tablefactory_t* table) {
  opt->table_factory = table->factory;
}

void crocksdb_options_set_max_subcompactions(DBOptions* opt, uint32_t v) {
  opt->max_subcompactions = v;
}

void crocksdb_options_set_wal_bytes_per_sync(DBOptions* opt, uint64_t v) {
  opt->wal_bytes_per_sync = v;
}

static BlockBasedTableOptions* get_block_based_table_options(
    const ColumnFamilyOptions* opt) {
  if (opt->table_factory != nullptr) {
    if (strcmp(opt->table_factory->Name(), block_base_table_str) == 0) {
      return opt->table_factory->GetOptions<BlockBasedTableOptions>();
    }
  }
  return nullptr;
}

size_t crocksdb_options_get_block_cache_usage(const ColumnFamilyOptions* opt) {
  auto opts = get_block_based_table_options(opt);
  if (opts && opts->block_cache) {
    return opts->block_cache->GetUsage();
  }
  return 0;
}

void crocksdb_options_set_block_cache_capacity(ColumnFamilyOptions* opt,
                                               size_t capacity, Status* s) {
  auto opts = get_block_based_table_options(opt);
  if (opts && opts->block_cache) {
    opts->block_cache->SetCapacity(capacity);
    *s = Status::OK();
  } else {
    *s = Status::InvalidArgument("failed to get block based table options");
  }
}

size_t crocksdb_options_get_block_cache_capacity(
    const ColumnFamilyOptions* opt) {
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

struct CRocksDBEventListener : public EventListener {
  void* state_;
  void (*destructor_)(void*);
  void (*on_flush_begin)(void*, DB*, const FlushJobInfo*);
  void (*on_flush_completed)(void*, DB*, const FlushJobInfo*);
  void (*on_compaction_begin)(void*, DB*, const CompactionJobInfo*);
  void (*on_compaction_completed)(void*, DB*, const CompactionJobInfo*);
  void (*on_subcompaction_begin)(void*, const SubcompactionJobInfo*);
  void (*on_subcompaction_completed)(void*, const SubcompactionJobInfo*);
  void (*on_external_file_ingested)(void*, DB*,
                                    const ExternalFileIngestionInfo*);
  void (*on_background_error)(void*, rocksdb::BackgroundErrorReason, Status*);
  void (*on_stall_conditions_changed)(void*, const WriteStallInfo*);

  virtual void OnFlushBegin(DB* db, const FlushJobInfo& info) {
    on_flush_begin(state_, db, &info);
  }

  virtual void OnFlushCompleted(DB* db, const FlushJobInfo& info) {
    on_flush_completed(state_, db, &info);
  }

  virtual void OnCompactionBegin(DB* db, const CompactionJobInfo& info) {
    on_compaction_begin(state_, db, &info);
  }

  virtual void OnCompactionCompleted(DB* db, const CompactionJobInfo& info) {
    on_compaction_completed(state_, db, &info);
  }

  virtual void OnSubcompactionBegin(const SubcompactionJobInfo& info) {
    on_subcompaction_begin(state_, &info);
  }

  virtual void OnSubcompactionCompleted(const SubcompactionJobInfo& info) {
    on_subcompaction_completed(state_, &info);
  }

  virtual void OnExternalFileIngested(DB* db,
                                      const ExternalFileIngestionInfo& info) {
    on_external_file_ingested(state_, db, &info);
  }

  virtual void OnBackgroundError(BackgroundErrorReason reason, Status* status) {
    on_background_error(state_, reason, status);
  }

  virtual void OnStallConditionsChanged(const WriteStallInfo& info) {
    on_stall_conditions_changed(state_, &info);
  }

  virtual ~CRocksDBEventListener() { destructor_(state_); }
};

struct crocksdb_eventlistener_t {
  std::shared_ptr<EventListener> rep;
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
  auto et = std::make_shared<CRocksDBEventListener>();
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
  auto t = new crocksdb_eventlistener_t;
  t->rep = et;
  return t;
}

void crocksdb_eventlistener_destroy(crocksdb_eventlistener_t* t) { delete t; }

void crocksdb_options_add_eventlistener(DBOptions* opt,
                                        crocksdb_eventlistener_t* t) {
  opt->listeners.emplace_back(t->rep);
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
    ColumnFamilyOptions* opt, crocksdb_cuckoo_table_options_t* table_options) {
  opt->table_factory.reset(rocksdb::NewCuckooTableFactory(table_options->rep));
}

Options* crocksdb_options_create() { return new Options(); }

DBOptions* crocksdb_options_get_dboptions(Options* opt) {
  return (DBOptions*)opt;
}

DBOptions* crocksdb_dboptions_create() { return new DBOptions(); }

ColumnFamilyOptions* crocksdb_options_get_cfoptions(Options* opt) {
  return (ColumnFamilyOptions*)opt;
}

ColumnFamilyOptions* crocksdb_cfoptions_create() {
  return new ColumnFamilyOptions();
}

void crocksdb_options_destroy(Options* opt) { delete opt; }

void crocksdb_dboptions_destroy(DBOptions* opt) { delete opt; }

void crocksdb_cfoptions_destroy(ColumnFamilyOptions* opt) { delete opt; }

void crocksdb_column_family_descriptor_destroy(
    crocksdb_column_family_descriptor* cf_desc) {
  delete cf_desc;
}

const char* crocksdb_name_from_column_family_descriptor(
    const crocksdb_column_family_descriptor* cf_desc) {
  return cf_desc->rep.name.c_str();
}

const ColumnFamilyOptions* crocksdb_options_from_column_family_descriptor(
    const crocksdb_column_family_descriptor* cf_desc) {
  return &cf_desc->rep.options;
}

void crocksdb_options_increase_parallelism(DBOptions* opt, int total_threads) {
  opt->IncreaseParallelism(total_threads);
}

void crocksdb_options_optimize_for_point_lookup(ColumnFamilyOptions* opt,
                                                uint64_t block_cache_size_mb) {
  opt->OptimizeForPointLookup(block_cache_size_mb);
}

void crocksdb_options_optimize_level_style_compaction(
    ColumnFamilyOptions* opt, uint64_t memtable_memory_budget) {
  opt->OptimizeLevelStyleCompaction(memtable_memory_budget);
}

void crocksdb_options_optimize_universal_style_compaction(
    ColumnFamilyOptions* opt, uint64_t memtable_memory_budget) {
  opt->OptimizeUniversalStyleCompaction(memtable_memory_budget);
}

void crocksdb_options_set_compaction_filter_factory(
    ColumnFamilyOptions* opt, crocksdb_compactionfilterfactory_t* factory) {
  opt->compaction_filter_factory = factory->rep;
}

void crocksdb_options_set_comparator(ColumnFamilyOptions* opt,
                                     crocksdb_comparator_t* cmp) {
  opt->comparator = cmp;
}

bool crocksdb_options_match_comparator(const ColumnFamilyOptions* opt,
                                       crocksdb_comparator_t* cmp) {
  return opt->comparator == cmp ||
         (opt->comparator == BytewiseComparator() && cmp == nullptr);
}

void crocksdb_options_set_merge_operator(
    ColumnFamilyOptions* opt, crocksdb_mergeoperator_t* merge_operator) {
  opt->merge_operator = merge_operator->rep;
}

void crocksdb_options_set_create_if_missing(DBOptions* opt, bool v) {
  opt->create_if_missing = v;
}

void crocksdb_options_set_create_missing_column_families(DBOptions* opt,
                                                         bool v) {
  opt->create_missing_column_families = v;
}

void crocksdb_options_set_error_if_exists(DBOptions* opt, bool v) {
  opt->error_if_exists = v;
}

void crocksdb_options_set_paranoid_checks(DBOptions* opt, bool v) {
  opt->paranoid_checks = v;
}

void crocksdb_options_set_env(DBOptions* opt, Env* env) { opt->env = env; }
bool crocksdb_options_match_env(const DBOptions* opt, Env* env) {
  return opt->env == env || (opt->env == Env::Default() && env == nullptr);
}

crocksdb_logger_t* crocksdb_logger_create(void* rep, void (*destructor_)(void*),
                                          crocksdb_logger_logv_cb logv) {
  crocksdb_logger_t* logger = new crocksdb_logger_t;
  auto li = std::make_shared<crocksdb_logger_impl_t>();
  li->rep = rep;
  li->destructor_ = destructor_;
  li->logv_internal_ = logv;
  logger->rep = li;
  return logger;
}

void crocksdb_options_set_info_log(DBOptions* opt, crocksdb_logger_t* l) {
  opt->info_log = l->rep;
}

void crocksdb_options_set_info_log_level(DBOptions* opt, InfoLogLevel v) {
  opt->info_log_level = v;
}

void crocksdb_options_set_db_write_buffer_size(DBOptions* opt, size_t s) {
  opt->db_write_buffer_size = s;
}

void crocksdb_options_set_write_buffer_size(ColumnFamilyOptions* opt,
                                            size_t s) {
  opt->write_buffer_size = s;
}

size_t crocksdb_options_get_write_buffer_size(const ColumnFamilyOptions* opt) {
  return opt->write_buffer_size;
}

void crocksdb_options_set_max_open_files(DBOptions* opt, int n) {
  opt->max_open_files = n;
}

void crocksdb_options_set_max_total_wal_size(DBOptions* opt, uint64_t n) {
  opt->max_total_wal_size = n;
}

void crocksdb_options_set_target_file_size_base(ColumnFamilyOptions* opt,
                                                uint64_t n) {
  opt->target_file_size_base = n;
}

uint64_t crocksdb_options_get_target_file_size_base(
    const ColumnFamilyOptions* opt) {
  return opt->target_file_size_base;
}

void crocksdb_options_set_target_file_size_multiplier(ColumnFamilyOptions* opt,
                                                      int n) {
  opt->target_file_size_multiplier = n;
}

void crocksdb_options_set_max_bytes_for_level_base(ColumnFamilyOptions* opt,
                                                   uint64_t n) {
  opt->max_bytes_for_level_base = n;
}

uint64_t crocksdb_options_get_max_bytes_for_level_base(
    ColumnFamilyOptions* opt) {
  return opt->max_bytes_for_level_base;
}

void crocksdb_options_set_level_compaction_dynamic_level_bytes(
    ColumnFamilyOptions* opt, bool v) {
  opt->level_compaction_dynamic_level_bytes = v;
}

bool crocksdb_options_get_level_compaction_dynamic_level_bytes(
    const ColumnFamilyOptions* options) {
  return options->level_compaction_dynamic_level_bytes;
}

void crocksdb_options_set_max_bytes_for_level_multiplier(
    ColumnFamilyOptions* opt, double n) {
  opt->max_bytes_for_level_multiplier = n;
}

double crocksdb_options_get_max_bytes_for_level_multiplier(
    const ColumnFamilyOptions* opt) {
  return opt->max_bytes_for_level_multiplier;
}

void crocksdb_options_set_max_compaction_bytes(ColumnFamilyOptions* opt,
                                               uint64_t n) {
  opt->max_compaction_bytes = n;
}

uint64_t crocksdb_options_get_max_compaction_bytes(
    const ColumnFamilyOptions* opt) {
  return opt->max_compaction_bytes;
}

void crocksdb_options_set_max_bytes_for_level_multiplier_additional(
    ColumnFamilyOptions* opt, const int* level_values, size_t num_levels) {
  opt->max_bytes_for_level_multiplier_additional.resize(num_levels);
  for (size_t i = 0; i < num_levels; ++i) {
    opt->max_bytes_for_level_multiplier_additional[i] = level_values[i];
  }
}

crocksdb_sst_partitioner_factory_t*
crocksdb_options_get_sst_partitioner_factory(ColumnFamilyOptions* opt) {
  crocksdb_sst_partitioner_factory_t* factory =
      new crocksdb_sst_partitioner_factory_t;
  factory->rep = opt->sst_partitioner_factory;
  return factory;
}

void crocksdb_options_set_sst_partitioner_factory(
    ColumnFamilyOptions* opt, crocksdb_sst_partitioner_factory_t* factory) {
  opt->sst_partitioner_factory = factory->rep;
}

void crocksdb_options_set_statistics(DBOptions* opt, crocksdb_statistics_t* v) {
  if (v) {
    opt->statistics = v->statistics;
  } else {
    opt->statistics = nullptr;
  }
}

void crocksdb_options_set_num_levels(ColumnFamilyOptions* opt, int n) {
  opt->num_levels = n;
}

int crocksdb_options_get_num_levels(const ColumnFamilyOptions* opt) {
  return opt->num_levels;
}

void crocksdb_options_set_level0_file_num_compaction_trigger(
    ColumnFamilyOptions* opt, int n) {
  opt->level0_file_num_compaction_trigger = n;
}

int crocksdb_options_get_level0_file_num_compaction_trigger(
    const ColumnFamilyOptions* opt) {
  return opt->level0_file_num_compaction_trigger;
}

void crocksdb_options_set_level0_slowdown_writes_trigger(
    ColumnFamilyOptions* opt, int n) {
  opt->level0_slowdown_writes_trigger = n;
}

int crocksdb_options_get_level0_slowdown_writes_trigger(
    const ColumnFamilyOptions* opt) {
  return opt->level0_slowdown_writes_trigger;
}

void crocksdb_options_set_level0_stop_writes_trigger(ColumnFamilyOptions* opt,
                                                     int n) {
  opt->level0_stop_writes_trigger = n;
}

int crocksdb_options_get_level0_stop_writes_trigger(
    const ColumnFamilyOptions* opt) {
  return opt->level0_stop_writes_trigger;
}

void crocksdb_options_set_wal_recovery_mode(DBOptions* opt,
                                            WALRecoveryMode mode) {
  opt->wal_recovery_mode = mode;
}

void crocksdb_options_set_compression(ColumnFamilyOptions* opt,
                                      CompressionType t) {
  opt->compression = t;
}

CompressionType crocksdb_options_get_compression(
    const ColumnFamilyOptions* opt) {
  return opt->compression;
}

void crocksdb_options_set_compression_per_level(
    ColumnFamilyOptions* opt, const CompressionType* level_values,
    size_t num_levels) {
  opt->compression_per_level.resize(num_levels);
  for (size_t i = 0; i < num_levels; ++i) {
    opt->compression_per_level[i] = level_values[i];
  }
}

const CompressionType* crocksdb_options_get_compression_per_level(
    const ColumnFamilyOptions* opt, size_t* num_levels) {
  *num_levels = opt->compression_per_level.size();
  return opt->compression_per_level.data();
}

void crocksdb_options_set_compression_options(
    ColumnFamilyOptions* opt, const CompressionOptions* compression_opt) {
  opt->compression_opts = *compression_opt;
}

CompressionOptions* crocksdb_options_get_compression_options(
    ColumnFamilyOptions* opt) {
  return &opt->compression_opts;
}

CompressionOptions* crocksdb_options_get_bottommost_compression_options(
    ColumnFamilyOptions* opt) {
  return &opt->bottommost_compression_opts;
}

void crocksdb_options_set_use_direct_reads(DBOptions* opt, bool v) {
  opt->use_direct_reads = v;
}

void crocksdb_options_set_use_direct_io_for_flush_and_compaction(DBOptions* opt,
                                                                 bool v) {
  opt->use_direct_io_for_flush_and_compaction = v;
}

void crocksdb_options_set_prefix_extractor(
    ColumnFamilyOptions* opt, crocksdb_slicetransform_t* prefix_extractor) {
  opt->prefix_extractor = prefix_extractor->rep;
}

void crocksdb_options_set_optimize_filters_for_hits(ColumnFamilyOptions* opt,
                                                    bool v) {
  opt->optimize_filters_for_hits = v;
}

void crocksdb_options_set_memtable_insert_with_hint_prefix_extractor(
    ColumnFamilyOptions* opt, crocksdb_slicetransform_t* prefix_extractor) {
  opt->memtable_insert_with_hint_prefix_extractor = prefix_extractor->rep;
}

void crocksdb_options_set_use_fsync(DBOptions* opt, bool use_fsync) {
  opt->use_fsync = use_fsync;
}

void crocksdb_options_add_db_paths(DBOptions* opt, Slice db_path,
                                   uint64_t target_size) {
  opt->db_paths.emplace_back(DbPath(db_path.ToString(), target_size));
}

size_t crocksdb_options_get_db_paths_num(const DBOptions* opt) {
  return opt->db_paths.size();
}

void crocksdb_options_get_db_path(const DBOptions* opt, size_t index,
                                  Slice* path) {
  *path = opt->db_paths[index].path;
}

uint64_t crocksdb_options_get_path_target_size(const DBOptions* opt,
                                               size_t index) {
  return opt->db_paths[index].target_size;
}

void crocksdb_options_set_db_log_dir(DBOptions* opt, Slice db_log_dir) {
  opt->db_log_dir = db_log_dir.ToString();
}

void crocksdb_options_set_wal_dir(DBOptions* opt, Slice v) {
  opt->wal_dir = v.ToString();
}

void crocksdb_options_set_wal_ttl_seconds(DBOptions* opt, uint64_t ttl) {
  opt->WAL_ttl_seconds = ttl;
}

void crocksdb_options_set_wal_size_limit_mb(DBOptions* opt, uint64_t limit) {
  opt->WAL_size_limit_MB = limit;
}

void crocksdb_options_set_manifest_preallocation_size(DBOptions* opt,
                                                      size_t v) {
  opt->manifest_preallocation_size = v;
}

void crocksdb_options_set_allow_mmap_reads(DBOptions* opt, bool v) {
  opt->allow_mmap_reads = v;
}

void crocksdb_options_set_allow_mmap_writes(DBOptions* opt, bool v) {
  opt->allow_mmap_writes = v;
}

void crocksdb_options_set_is_fd_close_on_exec(DBOptions* opt, bool v) {
  opt->is_fd_close_on_exec = v;
}

void crocksdb_options_set_stats_dump_period_sec(DBOptions* opt,
                                                unsigned int v) {
  opt->stats_dump_period_sec = v;
}

void crocksdb_options_set_advise_random_on_open(DBOptions* opt, bool v) {
  opt->advise_random_on_open = v;
}

void crocksdb_options_set_access_hint_on_compaction_start(
    DBOptions* opt, DBOptions::AccessHint v) {
  opt->access_hint_on_compaction_start = v;
}

void crocksdb_options_set_use_adaptive_mutex(DBOptions* opt, bool v) {
  opt->use_adaptive_mutex = v;
}

void crocksdb_options_set_bytes_per_sync(DBOptions* opt, uint64_t v) {
  opt->bytes_per_sync = v;
}

void crocksdb_options_set_enable_pipelined_write(DBOptions* opt, bool v) {
  opt->enable_pipelined_write = v;
}

void crocksdb_options_set_multi_batch_write(DBOptions* opt, bool v) {
  opt->enable_multi_batch_write = v;
}

bool crocksdb_options_multi_batch_write(const DBOptions* opt) {
  return opt->enable_multi_batch_write;
}

void crocksdb_options_set_unordered_write(DBOptions* opt, bool v) {
  opt->unordered_write = v;
}

void crocksdb_options_set_allow_concurrent_memtable_write(DBOptions* opt,
                                                          bool v) {
  opt->allow_concurrent_memtable_write = v;
}

void crocksdb_options_set_manual_wal_flush(DBOptions* opt, bool v) {
  opt->manual_wal_flush = v;
}

void crocksdb_options_set_enable_write_thread_adaptive_yield(DBOptions* opt,
                                                             bool v) {
  opt->enable_write_thread_adaptive_yield = v;
}

void crocksdb_options_set_max_sequential_skip_in_iterations(
    ColumnFamilyOptions* opt, uint64_t v) {
  opt->max_sequential_skip_in_iterations = v;
}

void crocksdb_options_set_max_write_buffer_number(ColumnFamilyOptions* opt,
                                                  int n) {
  opt->max_write_buffer_number = n;
}

int crocksdb_options_get_max_write_buffer_number(
    const ColumnFamilyOptions* opt) {
  return opt->max_write_buffer_number;
}

void crocksdb_options_set_min_write_buffer_number_to_merge(
    ColumnFamilyOptions* opt, int n) {
  opt->min_write_buffer_number_to_merge = n;
}

int crocksdb_options_get_min_write_buffer_number_to_merge(
    const ColumnFamilyOptions* opt) {
  return opt->min_write_buffer_number_to_merge;
}

void crocksdb_options_set_max_write_buffer_number_to_maintain(
    ColumnFamilyOptions* opt, int n) {
  opt->max_write_buffer_number_to_maintain = n;
}

void crocksdb_options_set_max_background_jobs(DBOptions* opt, int n) {
  opt->max_background_jobs = n;
}

int crocksdb_options_get_max_background_jobs(const DBOptions* opt) {
  return opt->max_background_jobs;
}

void crocksdb_options_set_max_background_compactions(DBOptions* opt, int n) {
  opt->max_background_compactions = n;
}

int crocksdb_options_get_max_background_compactions(const DBOptions* opt) {
  return opt->max_background_compactions;
}

void crocksdb_options_set_max_background_flushes(DBOptions* opt, int n) {
  opt->max_background_flushes = n;
}

int crocksdb_options_get_max_background_flushes(const DBOptions* opt) {
  return opt->max_background_flushes;
}

void crocksdb_options_set_max_log_file_size(DBOptions* opt, size_t v) {
  opt->max_log_file_size = v;
}

void crocksdb_options_set_log_file_time_to_roll(DBOptions* opt, size_t v) {
  opt->log_file_time_to_roll = v;
}

void crocksdb_options_set_keep_log_file_num(DBOptions* opt, size_t v) {
  opt->keep_log_file_num = v;
}

void crocksdb_options_set_recycle_log_file_num(DBOptions* opt, size_t v) {
  opt->recycle_log_file_num = v;
}

void crocksdb_options_set_soft_pending_compaction_bytes_limit(
    ColumnFamilyOptions* opt, uint64_t v) {
  opt->soft_pending_compaction_bytes_limit = v;
}

uint64_t crocksdb_options_get_soft_pending_compaction_bytes_limit(
    const ColumnFamilyOptions* opt) {
  return opt->soft_pending_compaction_bytes_limit;
}

void crocksdb_options_set_hard_pending_compaction_bytes_limit(
    ColumnFamilyOptions* opt, uint64_t v) {
  opt->hard_pending_compaction_bytes_limit = v;
}

uint64_t crocksdb_options_get_hard_pending_compaction_bytes_limit(
    const ColumnFamilyOptions* opt) {
  return opt->hard_pending_compaction_bytes_limit;
}

void crocksdb_options_set_max_manifest_file_size(DBOptions* opt, uint64_t v) {
  opt->max_manifest_file_size = v;
}

void crocksdb_options_set_table_cache_numshardbits(DBOptions* opt, int v) {
  opt->table_cache_numshardbits = v;
}

void crocksdb_options_set_writable_file_max_buffer_size(DBOptions* opt,
                                                        size_t v) {
  opt->writable_file_max_buffer_size = v;
}

void crocksdb_options_set_arena_block_size(ColumnFamilyOptions* opt, size_t v) {
  opt->arena_block_size = v;
}

void crocksdb_options_set_disable_auto_compactions(ColumnFamilyOptions* opt,
                                                   bool disable) {
  opt->disable_auto_compactions = disable;
}

bool crocksdb_options_get_disable_auto_compactions(
    const ColumnFamilyOptions* opt) {
  return opt->disable_auto_compactions;
}

void crocksdb_options_set_disable_write_stall(ColumnFamilyOptions* opt,
                                              bool disable) {
  opt->disable_write_stall = disable;
}

bool crocksdb_options_get_disable_write_stall(const ColumnFamilyOptions* opt) {
  return opt->disable_write_stall;
}

void crocksdb_options_set_delete_obsolete_files_period_micros(DBOptions* opt,
                                                              uint64_t v) {
  opt->delete_obsolete_files_period_micros = v;
}

void crocksdb_options_prepare_for_bulk_load(Options* opt) {
  opt->PrepareForBulkLoad();
}

void crocksdb_options_set_memtable_prefix_bloom_size_ratio(
    ColumnFamilyOptions* opt, double v) {
  opt->memtable_prefix_bloom_size_ratio = v;
}

void crocksdb_options_set_memtable_huge_page_size(ColumnFamilyOptions* opt,
                                                  size_t v) {
  opt->memtable_huge_page_size = v;
}
void crocksdb_options_get_memtable_factory_name(const ColumnFamilyOptions* opt,
                                                Slice* name) {
  if (!opt->memtable_factory) {
    *name = Slice();
  } else {
    *name = opt->memtable_factory->Name();
  }
}

struct crocksdb_memtablerepfactory_t {
  shared_ptr<MemTableRepFactory> factory;
};

crocksdb_memtablerepfactory_t*
crocksdb_memtablerepfactory_create_hash_skip_list(
    size_t bucket_count, int32_t skiplist_height,
    int32_t skiplist_branching_factor) {
  auto result = new crocksdb_memtablerepfactory_t;
  result->factory.reset(rocksdb::NewHashSkipListRepFactory(
      bucket_count, skiplist_height, skiplist_branching_factor));
  return result;
}

crocksdb_memtablerepfactory_t*
crocksdb_memtablerepfactory_create_hash_link_list(size_t bucket_count) {
  auto result = new crocksdb_memtablerepfactory_t;
  result->factory.reset(rocksdb::NewHashLinkListRepFactory(bucket_count));
  return result;
}

crocksdb_memtablerepfactory_t*
crocksdb_memtablerepfactory_create_doubly_skip_list(size_t lookahead) {
  auto result = new crocksdb_memtablerepfactory_t;
  result->factory.reset(new rocksdb::DoublySkipListFactory(lookahead));
  return result;
}

crocksdb_memtablerepfactory_t* crocksdb_memtablerepfactory_create_vector(
    uint64_t reserved_bytes) {
  auto result = new crocksdb_memtablerepfactory_t;
  result->factory.reset(new rocksdb::VectorRepFactory(reserved_bytes));
  return result;
}

void crocksdb_memtablerepfactory_destroy(
    crocksdb_memtablerepfactory_t* factory) {
  delete factory;
}

void crocksdb_memtablerepfactory_name(crocksdb_memtablerepfactory_t* factory,
                                      Slice* name) {
  *name = factory->factory->Name();
}

void crocksdb_options_set_memtable_factory(
    ColumnFamilyOptions* opt, const crocksdb_memtablerepfactory_t* factory) {
  opt->memtable_factory = factory->factory;
}

void crocksdb_options_set_max_successive_merges(ColumnFamilyOptions* opt,
                                                size_t v) {
  opt->max_successive_merges = v;
}

void crocksdb_options_set_bloom_locality(ColumnFamilyOptions* opt, uint32_t v) {
  opt->bloom_locality = v;
}

void crocksdb_options_set_inplace_update_support(ColumnFamilyOptions* opt,
                                                 bool v) {
  opt->inplace_update_support = v;
}

void crocksdb_options_set_inplace_update_num_locks(ColumnFamilyOptions* opt,
                                                   size_t v) {
  opt->inplace_update_num_locks = v;
}

void crocksdb_options_set_report_bg_io_stats(ColumnFamilyOptions* opt, bool v) {
  opt->report_bg_io_stats = v;
}

void crocksdb_options_set_compaction_readahead_size(DBOptions* opt, size_t v) {
  opt->compaction_readahead_size = v;
}

void crocksdb_options_set_compaction_style(ColumnFamilyOptions* opt,
                                           CompactionStyle style) {
  opt->compaction_style = style;
}

void crocksdb_options_set_universal_compaction_options(
    ColumnFamilyOptions* opt, const CompactionOptionsUniversal* univ) {
  opt->compaction_options_universal = *univ;
}

CompactionOptionsUniversal* crocksdb_options_get_universal_compaction_options(
    ColumnFamilyOptions* opt) {
  return &opt->compaction_options_universal;
}

void crocksdb_options_set_fifo_compaction_options(
    ColumnFamilyOptions* opt, const CompactionOptionsFIFO* fifo) {
  opt->compaction_options_fifo = *fifo;
}

CompactionOptionsFIFO* crocksdb_options_get_fifo_compaction_options(
    ColumnFamilyOptions* opt) {
  return &opt->compaction_options_fifo;
}

void crocksdb_options_set_compaction_priority(ColumnFamilyOptions* opt,
                                              CompactionPri priority) {
  opt->compaction_pri = priority;
}

void crocksdb_options_set_delayed_write_rate(DBOptions* opt,
                                             uint64_t delayed_write_rate) {
  opt->delayed_write_rate = delayed_write_rate;
}

void crocksdb_options_set_force_consistency_checks(ColumnFamilyOptions* opt,
                                                   bool v) {
  opt->force_consistency_checks = v;
}

bool crocksdb_options_get_force_consistency_checks(
    const ColumnFamilyOptions* opt) {
  return opt->force_consistency_checks;
}

crocksdb_statistics_t* crocksdb_statistics_create() {
  auto s = new crocksdb_statistics_t;
  s->statistics = rocksdb::titandb::CreateDBStatistics();
  return s;
}

void crocksdb_statistics_reset(crocksdb_statistics_t* s) {
  s->statistics.get()->Reset();
}

void crocksdb_statistics_destroy(crocksdb_statistics_t* ptr) { delete ptr; }

void crocksdb_statistics_get_string(crocksdb_statistics_t* ptr, void* ctx,
                                    bytes_receiver_cb fp) {
  rocksdb::Statistics* statistics = ptr->statistics.get();
  std::string s = statistics->ToString();
  fp(ctx, s);
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

void crocksdb_statistics_get_histogram_string(crocksdb_statistics_t* ptr,
                                              uint32_t type, void* ctx,
                                              bytes_receiver_cb fp) {
  auto s = ptr->statistics->getHistogramString(type);
  fp(ctx, s);
}

void crocksdb_statistics_get_histogram(crocksdb_statistics_t* ptr,
                                       uint32_t type, HistogramData* data) {
  ptr->statistics->histogramData(type, data);
}

void crocksdb_options_set_ratelimiter(DBOptions* opt,
                                      crocksdb_ratelimiter_t* limiter) {
  opt->rate_limiter = limiter->rep;
}

void crocksdb_options_set_atomic_flush(DBOptions* opt, bool enable) {
  opt->atomic_flush = enable;
}

void crocksdb_load_latest_options(Slice dbpath, Env* env,
                                  DBOptions** db_options, void* cf_opts_context,
                                  CFDescriptorReceiver desc_receiver,
                                  bool ignore_unknown_options, Status* s) {
  std::vector<ColumnFamilyDescriptor> tmp_cf_descs;
  DBOptions db_opt;
  *s = rocksdb::LoadLatestOptions(dbpath.ToString(), env, &db_opt,
                                  &tmp_cf_descs, ignore_unknown_options);

  if (!s->ok()) {
    return;
  }

  *db_options = new DBOptions(db_opt);
  for (size_t i = 0; i < tmp_cf_descs.size(); ++i) {
    ColumnFamilyOptions* opt = new ColumnFamilyOptions(tmp_cf_descs[i].options);
    desc_receiver(cf_opts_context, tmp_cf_descs[i].name, opt);
  }

  return;
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

CompactionFilter* crocksdb_compactionfilter_create(
    void* state, void (*destructor)(void*),
    CompactionFilter::Decision (*filter)(void*, int level, Slice key,
                                         SequenceNumber seqno,
                                         CompactionFilter::ValueType value_type,
                                         Slice existing_value, Slice* new_value,
                                         Slice* skip_until),
    const char* (*name)(void*)) {
  crocksdb_compactionfilter_t* result = new crocksdb_compactionfilter_t;
  result->state_ = state;
  result->destructor_ = destructor;
  result->filter_ = filter;
  result->name_ = name;
  return result;
}

bool crocksdb_compactionfiltercontext_is_full_compaction(
    const CompactionFilter::Context* context) {
  return context->is_full_compaction;
}

bool crocksdb_compactionfiltercontext_is_manual_compaction(
    const CompactionFilter::Context* context) {
  return context->is_manual_compaction;
}

bool crocksdb_compactionfiltercontext_is_bottommost_level(
    const CompactionFilter::Context* context) {
  return context->is_bottommost_level;
}

void crocksdb_compactionfiltercontext_file_numbers(
    const CompactionFilter::Context* context, const uint64_t** buffer,
    size_t* len) {
  *buffer = context->file_numbers.data();
  *len = context->file_numbers.size();
}

const TableProperties* crocksdb_compactionfiltercontext_table_properties(
    const CompactionFilter::Context* context, size_t offset) {
  return context->table_properties[offset].get();
}

void crocksdb_compactionfiltercontext_start_key(
    const CompactionFilter::Context* context, Slice* start_key) {
  *start_key = context->start_key;
}

void crocksdb_compactionfiltercontext_end_key(
    const CompactionFilter::Context* context, Slice* end_key) {
  *end_key = context->end_key;
}

crocksdb_compactionfilterfactory_t* crocksdb_compactionfilterfactory_create(
    void* state, void (*destructor)(void*),
    CompactionFilter* (*create_compaction_filter)(
        void*, const CompactionFilter::Context* context),
    bool (*should_filter_table_file_creation)(void*,
                                              TableFileCreationReason reason),
    const char* (*name)(void*)) {
  auto result = std::make_shared<CRocksDBCompactionFilterFactory>();
  result->state_ = state;
  result->destructor_ = destructor;
  result->create_compaction_filter_ = create_compaction_filter;
  result->should_filter_table_file_creation_ =
      should_filter_table_file_creation;
  result->name_ = name;
  auto t = new crocksdb_compactionfilterfactory_t;
  t->rep = result;
  return t;
}

void crocksdb_compactionfilterfactory_destroy(
    crocksdb_compactionfilterfactory_t* factory) {
  delete factory;
}

crocksdb_comparator_t* crocksdb_comparator_create(
    void* state, void (*destructor)(void*),
    int (*compare)(void*, Slice lhs, Slice rhs), const char* (*name)(void*)) {
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
  FilterPolicyWrapper* wrapper = new FilterPolicyWrapper;
  wrapper->state_ = state;
  wrapper->destructor_ = destructor;
  wrapper->create_ = create_filter;
  wrapper->key_match_ = key_may_match;
  wrapper->delete_filter_ = delete_filter;
  wrapper->name_ = name;
  result->policy.reset(wrapper);
  return result;
}

void crocksdb_filterpolicy_destroy(crocksdb_filterpolicy_t* filter) {
  delete filter;
}

crocksdb_filterpolicy_t* crocksdb_filterpolicy_create_bloom_format(
    double bits_per_key, bool use_block_based_builder) {
  auto result = new crocksdb_filterpolicy_t;
  result->policy.reset(
      NewBloomFilterPolicy(bits_per_key, use_block_based_builder));
  return result;
}
crocksdb_mergeoperator_t* crocksdb_mergeoperator_create(
    void* state, void (*destructor)(void*), full_merge_cb full,
    partial_merge_cb partial, partial_merge_mult_cb partial_mult, name_cb name,
    allow_single_operand_cb allow_single, should_merge_cb should_merge) {
  auto result = std::make_shared<CRocksDBMergeOperator>();
  result->state_ = state;
  result->destructor_ = destructor;
  result->full_merge_ = full;
  result->partial_merge_ = partial;
  result->partial_merge_mult_ = partial_mult;
  result->name_ = name;
  result->allow_single_operand_ = allow_single;
  result->should_merge_ = should_merge;

  auto t = new crocksdb_mergeoperator_t;
  t->rep = result;
  return t;
}

void crocksdb_mergeoperator_destroy(crocksdb_mergeoperator_t* merge_operator) {
  delete merge_operator;
}

void crocksdb_mergeoperationinput_key(
    const MergeOperator::MergeOperationInput* input, Slice* key) {
  *key = input->key;
}

void crocksdb_mergeoperationinput_existing_value(
    const MergeOperator::MergeOperationInput* input,
    const Slice** existing_value) {
  *existing_value = input->existing_value;
}

void crocksdb_mergeoperationinput_operand_list(
    const MergeOperator::MergeOperationInput* input, const Slice** ops,
    size_t* count) {
  *ops = input->operand_list.data();
  *count = input->operand_list.size();
}

void crocksdb_mergeoperationoutput_set_new_value(
    MergeOperator::MergeOperationOutput* output, Slice new_value) {
  output->new_value.assign(new_value.data(), new_value.size());
}

void crocksdb_mergeoperationoutput_append_new_value(
    MergeOperator::MergeOperationOutput* output, Slice new_value) {
  output->new_value.append(new_value.data(), new_value.size());
}

void crocksdb_mergeoperationoutput_set_existing_operand(
    MergeOperator::MergeOperationOutput* output,
    const MergeOperator::MergeOperationInput* input, size_t count) {
  output->existing_operand = input->operand_list[count];
}

void crocksdb_mergeoperationoutput_set_existing_operand_to_value(
    MergeOperator::MergeOperationOutput* output,
    const MergeOperator::MergeOperationInput* input) {
  output->existing_operand = *input->existing_value;
}

void crocksdb_readoptions_set_snapshot(ReadOptions* opt, const Snapshot* v) {
  opt->snapshot = v;
}

const Snapshot* crocksdb_readoptions_snapshot(const ReadOptions* opt) {
  return opt->snapshot;
}

void crocksdb_readoptions_set_iterate_lower_bound(ReadOptions* opt,
                                                  const Slice* v) {
  opt->iterate_lower_bound = v;
}

const Slice* crocksdb_readoptions_iterate_lower_bound(const ReadOptions* opt) {
  return opt->iterate_lower_bound;
}

void crocksdb_readoptions_set_iterate_upper_bound(ReadOptions* opt,
                                                  const Slice* v) {
  opt->iterate_upper_bound = v;
}

const Slice* crocksdb_readoptions_iterate_upper_bound(const ReadOptions* opt) {
  return opt->iterate_upper_bound;
}

void crocksdb_readoptions_set_readahead_size(ReadOptions* opt, size_t v) {
  opt->readahead_size = v;
}

void crocksdb_readoptions_set_max_skippable_internal_keys(ReadOptions* opt,
                                                          uint64_t v) {
  opt->max_skippable_internal_keys = v;
}

void crocksdb_readoptions_set_read_tier(ReadOptions* opt, ReadTier v) {
  opt->read_tier = v;
}

void crocksdb_readoptions_set_verify_checksums(ReadOptions* opt, bool v) {
  opt->verify_checksums = v;
}

void crocksdb_readoptions_set_fill_cache(ReadOptions* opt, bool v) {
  opt->fill_cache = v;
}

void crocksdb_readoptions_set_tailing(ReadOptions* opt, bool v) {
  opt->tailing = v;
}

void crocksdb_readoptions_set_total_order_seek(ReadOptions* opt, bool v) {
  opt->total_order_seek = v;
}

void crocksdb_readoptions_set_auto_prefix_mode(ReadOptions* opt, bool v) {
  opt->auto_prefix_mode = v;
}

void crocksdb_readoptions_set_prefix_same_as_start(ReadOptions* opt, bool v) {
  opt->prefix_same_as_start = v;
}

void crocksdb_readoptions_set_pin_data(ReadOptions* opt, bool v) {
  opt->pin_data = v;
}

void crocksdb_readoptions_set_background_purge_on_iterator_cleanup(
    ReadOptions* opt, bool v) {
  opt->background_purge_on_iterator_cleanup = v;
}

void crocksdb_readoptions_set_ignore_range_deletions(ReadOptions* opt, bool v) {
  opt->ignore_range_deletions = v;
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
  TableFilter(void* ctx, bool (*table_filter)(void*, const TableProperties*),
              void (*destroy)(void*))
      : ctx_(std::make_shared<TableFilterCtx>(ctx, destroy)),
        table_filter_(table_filter) {}

  TableFilter(const TableFilter& f)
      : ctx_(f.ctx_), table_filter_(f.table_filter_) {}

  bool operator()(const TableProperties& prop) {
    return table_filter_(ctx_->ctx_, &prop);
  }

  shared_ptr<TableFilterCtx> ctx_;
  bool (*table_filter_)(void*, const TableProperties*);

 private:
  TableFilter() {}
};

void crocksdb_readoptions_set_table_filter(
    ReadOptions* opt, void* ctx,
    bool (*table_filter)(void*, const TableProperties*),
    void (*destroy)(void*)) {
  opt->table_filter = TableFilter(ctx, table_filter, destroy);
}

void crocksdb_readoptions_set_timestamp(ReadOptions* opt, const Slice* v) {
  opt->timestamp = v;
}

void crocksdb_readoptions_set_iter_start_ts(ReadOptions* opt, const Slice* v) {
  opt->iter_start_ts = v;
}

void crocksdb_readoptions_set_value_size_soft_limit(ReadOptions* opt,
                                                    uint64_t v) {
  opt->value_size_soft_limit = v;
}

void crocksdb_readoptions_set_adaptive_readahead(ReadOptions* opt, bool v) {
  opt->adaptive_readahead = v;
}

void crocksdb_writeoptions_init(WriteOptions* opt) { *opt = WriteOptions(); }

void crocksdb_compactrangeoptions_init(CompactRangeOptions* opt) {
  new (opt) CompactRangeOptions();
}

void crocksdb_compactrangeoptions_inplace_destroy(CompactRangeOptions* opt) {
  opt->~CompactRangeOptions();
}

void crocksdb_compactrangeoptions_set_exclusive_manual_compaction(
    CompactRangeOptions* opt, bool v) {
  opt->exclusive_manual_compaction = v;
}

void crocksdb_compactrangeoptions_set_change_level(CompactRangeOptions* opt,
                                                   bool v) {
  opt->change_level = v;
}

void crocksdb_compactrangeoptions_set_target_level(CompactRangeOptions* opt,
                                                   int v) {
  opt->target_level = v;
}

void crocksdb_compactrangeoptions_set_target_path_id(CompactRangeOptions* opt,
                                                     uint32_t v) {
  opt->target_path_id = v;
}

void crocksdb_compactrangeoptions_set_bottommost_level_compaction(
    CompactRangeOptions* opt, BottommostLevelCompaction v) {
  opt->bottommost_level_compaction = v;
}

void crocksdb_compactrangeoptions_set_allow_write_stall(
    CompactRangeOptions* opt, bool v) {
  opt->allow_write_stall = v;
}

void crocksdb_compactrangeoptions_set_max_subcompactions(
    CompactRangeOptions* opt, uint32_t v) {
  opt->max_subcompactions = v;
}

void crocksdb_compactrangeoptions_set_full_history_ts_low(
    CompactRangeOptions* opt, const Slice* v) {
  opt->full_history_ts_low = v;
}

void crocksdb_compactrangeoptions_set_canceled(CompactRangeOptions* opt,
                                               bool v) {
  opt->canceled->store(v, std::memory_order_release);
}

void crocksdb_flushoptions_init(FlushOptions* opt) { *opt = FlushOptions(); }

LRUCacheOptions* crocksdb_lru_cache_options_create() {
  return new LRUCacheOptions;
}

void crocksdb_lru_cache_options_destroy(LRUCacheOptions* opt) { delete opt; }

void crocksdb_lru_cache_options_set_capacity(LRUCacheOptions* opt,
                                             size_t capacity) {
  opt->capacity = capacity;
}

void crocksdb_lru_cache_options_set_num_shard_bits(LRUCacheOptions* opt,
                                                   int num_shard_bits) {
  opt->num_shard_bits = num_shard_bits;
}

void crocksdb_lru_cache_options_set_strict_capacity_limit(
    LRUCacheOptions* opt, bool strict_capacity_limit) {
  opt->strict_capacity_limit = strict_capacity_limit;
}

void crocksdb_lru_cache_options_set_high_pri_pool_ratio(
    LRUCacheOptions* opt, double high_pri_pool_ratio) {
  opt->high_pri_pool_ratio = high_pri_pool_ratio;
}

void crocksdb_jemallocallocatoroptions_init(JemallocAllocatorOptions* opt) {
  *opt = JemallocAllocatorOptions();
}

void crocksdb_lru_cache_options_set_use_jemalloc(
    LRUCacheOptions* opt, JemallocAllocatorOptions* j_opt, Status* s) {
  *s = rocksdb::NewJemallocNodumpAllocator(*j_opt, &opt->memory_allocator);
}

crocksdb_cache_t* crocksdb_cache_create_lru(LRUCacheOptions* opt) {
  crocksdb_cache_t* c = new crocksdb_cache_t;
  c->rep = NewLRUCache(*opt);
  return c;
}

void crocksdb_cache_destroy(crocksdb_cache_t* cache) { delete cache; }

void crocksdb_cache_set_capacity(crocksdb_cache_t* cache, size_t capacity) {
  cache->rep->SetCapacity(capacity);
}

size_t crocksdb_cache_usage(const crocksdb_cache_t* c) {
  return c->rep->GetUsage();
}

size_t crocksdb_cache_capacity(const crocksdb_cache_t* c) {
  return c->rep->GetCapacity();
}

Env* crocksdb_default_env_create() { return Env::Default(); }

Env* crocksdb_mem_env_create(Env* base) { return rocksdb::NewMemEnv(base); }

struct CTRBlockCipher : public BlockCipher {
  CTRBlockCipher(size_t block_size, const std::string& cipertext)
      : block_size_(block_size), cipertext_(cipertext) {
    assert(block_size == cipertext.size());
  }

  const char* Name() const override { return "CTRBlockCipher"; }

  virtual size_t BlockSize() override { return block_size_; }

  virtual Status Encrypt(char* data) override {
    const char* ciper_ptr = cipertext_.c_str();
    for (size_t i = 0; i < block_size_; i++) {
      data[i] = data[i] ^ ciper_ptr[i];
    }

    return Status::OK();
  }

  virtual Status Decrypt(char* data) override {
    Encrypt(data);
    return Status::OK();
  }

 protected:
  std::string cipertext_;
  size_t block_size_;
};

Env* crocksdb_ctr_encrypted_env_create(Env* base_env, const char* ciphertext,
                                       size_t ciphertext_len) {
  auto block_cipher = std::make_shared<CTRBlockCipher>(
      ciphertext_len, std::string(ciphertext, ciphertext_len));
  auto encryption_provider = EncryptionProvider::NewCTRProvider(block_cipher);
  return NewEncryptedEnv(base_env, encryption_provider);
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

EnvOptions* crocksdb_envoptions_create() { return new EnvOptions; }

void crocksdb_envoptions_destroy(EnvOptions* opt) { delete opt; }

crocksdb_sequential_file_t* crocksdb_sequential_file_create(
    Env* env, Slice path, const EnvOptions* opts, Status* s) {
  std::unique_ptr<SequentialFile> result;
  auto p = path.ToString();
  *s = env->NewSequentialFile(p, &result, *opts);
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
void crocksdb_file_encryption_info_init(FileEncryptionInfo* info,
                                        EncryptionMethod method, Slice key,
                                        Slice iv) {
  info->method = method;
  info->key = key.ToString();
  info->iv = iv.ToString();
}

struct crocksdb_encryption_key_manager_impl_t : public KeyManager {
  void* state;
  void (*destructor)(void*);
  crocksdb_encryption_key_manager_get_file_cb get_file;
  crocksdb_encryption_key_manager_new_file_cb new_file;
  crocksdb_encryption_key_manager_delete_file_cb delete_file;
  crocksdb_encryption_key_manager_link_file_cb link_file;

  virtual ~crocksdb_encryption_key_manager_impl_t() { destructor(state); }

  Status GetFile(const std::string& fname, FileEncryptionInfo* info) override {
    Status s;
    get_file(state, fname, info, &s);
    return s;
  }

  Status NewFile(const std::string& fname, FileEncryptionInfo* info) override {
    Status s;
    new_file(state, fname, info, &s);
    return s;
  }

  Status DeleteFile(const std::string& fname) override {
    Status s;
    delete_file(state, fname, &s);
    return s;
  }

  Status LinkFile(const std::string& src_fname,
                  const std::string& dst_fname) override {
    Status s;
    link_file(state, src_fname, dst_fname, &s);
    return s;
  }
};

KeyManager* crocksdb_encryption_key_manager_create(
    void* state, void (*destructor)(void*),
    crocksdb_encryption_key_manager_get_file_cb get_file,
    crocksdb_encryption_key_manager_new_file_cb new_file,
    crocksdb_encryption_key_manager_delete_file_cb delete_file,
    crocksdb_encryption_key_manager_link_file_cb link_file) {
  auto key_manager_impl = new crocksdb_encryption_key_manager_impl_t;
  key_manager_impl->state = state;
  key_manager_impl->destructor = destructor;
  key_manager_impl->get_file = get_file;
  key_manager_impl->new_file = new_file;
  key_manager_impl->delete_file = delete_file;
  key_manager_impl->link_file = link_file;
  return key_manager_impl;
}

void crocksdb_encryption_key_manager_destroy(KeyManager* key_manager) {
  delete key_manager;
}

Env* crocksdb_key_managed_encrypted_env_create(Env* base_env,
                                               KeyManager* key_manager) {
  std::shared_ptr<KeyManager> p(key_manager);
  return NewKeyManagedEncryptedEnv(base_env, p);
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

SstFileReader* crocksdb_sstfilereader_create(const Options* io_options) {
  return new SstFileReader(*io_options);
}

void crocksdb_sstfilereader_open(SstFileReader* reader, Slice name, Status* s) {
  *s = reader->Open(name.ToString());
}

Iterator* crocksdb_sstfilereader_new_iterator(SstFileReader* reader,
                                              const ReadOptions* options) {
  return reader->NewIterator(*options);
}

const TableProperties* crocksdb_sstfilereader_get_table_properties(
    const SstFileReader* reader) {
  return reader->GetTableProperties().get();
}

void crocksdb_sstfilereader_verify_checksum(SstFileReader* reader, Status* s) {
  *s = reader->VerifyChecksum();
}

void crocksdb_sstfilereader_destroy(SstFileReader* reader) { delete reader; }

SstFileWriter* crocksdb_sstfilewriter_create(const EnvOptions* env,
                                             const Options* io_options) {
  return new SstFileWriter(*env, *io_options);
}

SstFileWriter* crocksdb_sstfilewriter_create_cf(
    const EnvOptions* env, const Options* io_options,
    ColumnFamilyHandle* column_family) {
  return new SstFileWriter(*env, *io_options, column_family);
}

void crocksdb_sstfilewriter_open(SstFileWriter* writer, Slice name, Status* s) {
  *s = writer->Open(name.ToString());
}

void crocksdb_sstfilewriter_put(SstFileWriter* writer, Slice key, Slice val,
                                Status* s) {
  *s = writer->Put(key, val);
}

void crocksdb_sstfilewriter_merge(SstFileWriter* writer, Slice key, Slice val,
                                  Status* s) {
  *s = writer->Merge(key, val);
}

void crocksdb_sstfilewriter_delete(SstFileWriter* writer, Slice key,
                                   Status* s) {
  *s = writer->Delete(key);
}

void crocksdb_sstfilewriter_delete_range(SstFileWriter* writer, Slice begin_key,
                                         Slice end_key, Status* s) {
  *s = writer->DeleteRange(begin_key, end_key);
}

void crocksdb_sstfilewriter_finish(SstFileWriter* writer,
                                   ExternalSstFileInfo* info, Status* s) {
  *s = writer->Finish(info);
}

uint64_t crocksdb_sstfilewriter_file_size(SstFileWriter* writer) {
  return writer->FileSize();
}

void crocksdb_sstfilewriter_destroy(SstFileWriter* writer) { delete writer; }

ExternalSstFileInfo* crocksdb_externalsstfileinfo_create() {
  return new ExternalSstFileInfo;
};

void crocksdb_externalsstfileinfo_destroy(ExternalSstFileInfo* info) {
  delete info;
}

void crocksdb_externalsstfileinfo_file_path(const ExternalSstFileInfo* info,
                                            Slice* path) {
  *path = info->file_path;
}

void crocksdb_externalsstfileinfo_smallest_key(const ExternalSstFileInfo* info,
                                               Slice* key) {
  *key = info->smallest_key;
}

void crocksdb_externalsstfileinfo_largest_key(const ExternalSstFileInfo* info,
                                              Slice* key) {
  *key = info->largest_key;
}

SequenceNumber crocksdb_externalsstfileinfo_sequence_number(
    const ExternalSstFileInfo* info) {
  return info->sequence_number;
}

uint64_t crocksdb_externalsstfileinfo_file_size(
    const ExternalSstFileInfo* info) {
  return info->file_size;
}

uint64_t crocksdb_externalsstfileinfo_num_entries(
    const ExternalSstFileInfo* info) {
  return info->num_entries;
}

void crocksdb_ingestexternalfileoptions_init(IngestExternalFileOptions* opt) {
  *opt = IngestExternalFileOptions();
}

void crocksdb_ingest_external_file(DB* db, const Slice* file_list,
                                   size_t list_len,
                                   const IngestExternalFileOptions* opt,
                                   Status* s) {
  std::vector<std::string> files(list_len);
  for (size_t i = 0; i < list_len; ++i) {
    files[i] = file_list[i].ToString();
  }
  *s = db->IngestExternalFile(files, *opt);
}

void crocksdb_ingest_external_file_cf(DB* db, ColumnFamilyHandle* handle,
                                      const Slice* file_list, size_t list_len,
                                      const IngestExternalFileOptions* opt,
                                      Status* s) {
  std::vector<std::string> files(list_len);
  for (size_t i = 0; i < list_len; ++i) {
    files[i] = file_list[i].ToString();
  }
  *s = db->IngestExternalFile(handle, files, *opt);
}

void crocksdb_ingest_external_file_multi_cf(
    DB* db, ColumnFamilyHandle* const* handle, const Slice* const* file_list,
    const size_t* list_len, const IngestExternalFileOptions* const* opt,
    size_t arg_count, Status* s) {
  std::vector<IngestExternalFileArg> args(arg_count);
  for (size_t i = 0; i < arg_count; ++i) {
    IngestExternalFileArg arg;
    arg.column_family = handle[i];
    arg.external_files.reserve(list_len[i]);
    for (size_t j = 0; j < list_len[i]; ++j) {
      arg.external_files.push_back(file_list[i][j].ToString());
    }
    arg.options = *opt[i];
    args.push_back(arg);
  }
  *s = db->IngestExternalFiles(args);
}

bool crocksdb_ingest_external_file_optimized(
    DB* db, ColumnFamilyHandle* handle, const Slice* file_list, size_t list_len,
    const IngestExternalFileOptions* opt, Status* s) {
  std::vector<std::string> files(list_len);
  for (size_t i = 0; i < list_len; ++i) {
    files[i] = file_list[i].ToString();
  }
  bool has_flush = false;
  // If the file being ingested is overlapped with the memtable, it
  // will block writes and wait for flushing, which can cause high
  // write latency. So we set `allow_blocking_flush = false`.
  auto ingest_opts = *opt;
  ingest_opts.allow_blocking_flush = false;
  auto s1 = db->IngestExternalFile(handle, files, ingest_opts);
  if (s1.IsInvalidArgument() &&
      s1.ToString().find("External file requires flush") != std::string::npos) {
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
    db->Flush(flush_opts, handle);
    *s = db->IngestExternalFile(handle, files, *opt);
  } else {
    *s = s1;
  }
  return has_flush;
}

crocksdb_slicetransform_t* crocksdb_slicetransform_create(
    void* state, void (*destructor)(void*),
    void (*transform)(void*, Slice, Slice* dst),
    bool (*in_domain)(void*, Slice),
    bool (*full_length_enabled)(void*, size_t*),
    bool (*same_result_when_appended)(void*, Slice),
    const char* (*name)(void*)) {
  auto result = std::make_shared<CRocksDBSliceTransform>();
  result->state_ = state;
  result->destructor_ = destructor;
  result->transform_ = transform;
  result->in_domain_ = in_domain;
  result->full_length_enabled_ = full_length_enabled;
  result->same_result_when_appended_ = same_result_when_appended;
  result->name_ = name;
  auto t = new crocksdb_slicetransform_t;
  t->rep = result;
  return t;
}

void crocksdb_slicetransform_destroy(crocksdb_slicetransform_t* st) {
  delete st;
}

crocksdb_slicetransform_t* crocksdb_slicetransform_create_fixed_prefix(
    size_t prefixLen) {
  auto t = new crocksdb_slicetransform_t;
  t->rep = shared_ptr<const SliceTransform>(
      rocksdb::NewFixedPrefixTransform(prefixLen));
  return t;
}

crocksdb_slicetransform_t* crocksdb_slicetransform_create_noop() {
  auto t = new crocksdb_slicetransform_t;
  t->rep = shared_ptr<const SliceTransform>(rocksdb::NewNoopTransform());
  return t;
}

void crocksdb_options_set_min_level_to_compress(ColumnFamilyOptions* opt,
                                                int level) {
  if (level >= 0) {
    assert(level <= opt->num_levels);
    opt->compression_per_level.resize(opt->num_levels);
    for (int i = 0; i < level; i++) {
      opt->compression_per_level[i] = rocksdb::kNoCompression;
    }
    for (int i = level; i < opt->num_levels; i++) {
      opt->compression_per_level[i] = opt->compression;
    }
  }
}

size_t crocksdb_livefiles_count(const crocksdb_livefiles_t* lf) {
  return static_cast<int>(lf->rep.size());
}

const LiveFileMetaData* crocksdb_livefiles_get(const crocksdb_livefiles_t* lf,
                                               size_t index) {
  return &lf->rep[index];
}

void crocksdb_livefiles_name(const LiveFileMetaData* lf, Slice* name) {
  *name = lf->name;
}

int crocksdb_livefiles_level(const LiveFileMetaData* lf) { return lf->level; }

size_t crocksdb_livefiles_size(const LiveFileMetaData* lf) { return lf->size; }

void crocksdb_livefiles_smallestkey(const LiveFileMetaData* lf, Slice* key) {
  *key = lf->smallestkey;
}

void crocksdb_livefiles_largestkey(const LiveFileMetaData* lf, Slice* key) {
  *key = lf->largestkey;
}

extern void crocksdb_livefiles_destroy(crocksdb_livefiles_t* lf) { delete lf; }

void crocksdb_get_options_from_string(const Options* base_options,
                                      const char* opts_str,
                                      Options* new_options, Status* s) {
  *s = GetOptionsFromString(*base_options, std::string(opts_str), new_options);
}

void crocksdb_delete_files_in_ranges_cf(DB* db, ColumnFamilyHandle* cf,
                                        const RangePtr* ranges,
                                        size_t num_ranges, bool include_end,
                                        Status* s) {
  *s = DeleteFilesInRanges(db, cf, ranges, num_ranges, include_end);
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

crocksdb_logger_t* crocksdb_create_log_from_options(Slice path,
                                                    const DBOptions* opts,
                                                    Status* s) {
  std::shared_ptr<Logger> l;
  *s = CreateLoggerFromOptions(path.ToString(), *opts, &l);
  if (s->ok()) {
    auto logger = new crocksdb_logger_t;
    logger->rep = l;
    return logger;
  }
  return nullptr;
}

void crocksdb_log_destroy(crocksdb_logger_t* logger) { delete logger; }

PinnableSlice* crocksdb_pinnableslice_create() { return new PinnableSlice; }

void crocksdb_get_pinned(DB* db, const ReadOptions* options, Slice key,
                         PinnableSlice* val, Status* s) {
  *s = db->Get(*options, db->DefaultColumnFamily(), key, val);
}

void crocksdb_get_pinned_cf(DB* db, const ReadOptions* options,
                            ColumnFamilyHandle* column_family, Slice key,
                            PinnableSlice* val, Status* s) {
  *s = db->Get(*options, column_family, key, val);
}

void crocksdb_multiget_pinned_cf(DB* db, const ReadOptions* options,
                                 ColumnFamilyHandle* column_family,
                                 const size_t num_keys, const Slice* keys,
                                 PinnableSlice* values, Status* s,
                                 bool input_sorted) {
  db->MultiGet(*options, column_family, num_keys, keys, values, s,
               input_sorted);
}

void crocksdb_pinnableslice_destroy(PinnableSlice* v) { delete v; }

void crocksdb_pinnableslice_reset(PinnableSlice* v) { v->Reset(); }

void crocksdb_pinnableslice_value(const PinnableSlice* v, Slice* val) {
  // v can't be null.
  *val = *v;
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

void crocksdb_table_properties_to_string(const TableProperties* props,
                                         void* ctx, bytes_receiver_cb fp) {
  auto s = props->ToString();
  fp(ctx, s);
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

TablePropertiesCollection* crocksdb_table_properties_collection_create() {
  return new TablePropertiesCollection;
}

void crocksdb_table_properties_collection_clear(TablePropertiesCollection* c) {
  c->clear();
}

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

struct CRocksDBPropertiesCollectorFactory
    : public TablePropertiesCollectorFactory {
  void* state_;
  const char* (*name_)(void*);
  void (*destruct_)(void*);
  crocksdb_table_properties_collector_t* (*create_table_properties_collector_)(
      void*, uint32_t column_family_id);

  virtual ~CRocksDBPropertiesCollectorFactory() { destruct_(state_); }

  virtual TablePropertiesCollector* CreateTablePropertiesCollector(
      TablePropertiesCollectorFactory::Context ctx) override {
    return create_table_properties_collector_(state_, ctx.column_family_id);
  }

  const char* Name() const override { return name_(state_); }
};

struct crocksdb_table_properties_collector_factory_t {
  std::shared_ptr<TablePropertiesCollectorFactory> rep;
};

crocksdb_table_properties_collector_factory_t*
crocksdb_table_properties_collector_factory_create(
    void* state, const char* (*name)(void*), void (*destruct)(void*),
    crocksdb_table_properties_collector_t* (*create_table_properties_collector)(
        void*, uint32_t column_family_id)) {
  auto f = std::make_shared<CRocksDBPropertiesCollectorFactory>();
  f->state_ = state;
  f->name_ = name;
  f->destruct_ = destruct;
  f->create_table_properties_collector_ = create_table_properties_collector;
  auto res = new crocksdb_table_properties_collector_factory_t;
  res->rep = f;
  return res;
}

void crocksdb_table_properties_collector_factory_destroy(
    crocksdb_table_properties_collector_factory_t* f) {
  delete f;
}

void crocksdb_options_add_table_properties_collector_factory(
    ColumnFamilyOptions* opt,
    crocksdb_table_properties_collector_factory_t* f) {
  opt->table_properties_collector_factories.push_back(f->rep);
}

crocksdb_table_properties_collector_factory_t*
crocksdb_table_properties_collector_factory_create_compact_on_deletion(
    size_t sliding_window_size, size_t deletion_trigger) {
  auto factory = rocksdb::NewCompactOnDeletionCollectorFactory(
      sliding_window_size, deletion_trigger);
  auto res = new crocksdb_table_properties_collector_factory_t;
  res->rep = factory;
  return res;
}

/* Get Table Properties */

void crocksdb_get_properties_of_all_tables(DB* db, TablePropertiesCollection* v,
                                           Status* s) {
  *s = db->GetPropertiesOfAllTables(v);
}

void crocksdb_get_properties_of_all_tables_cf(DB* db, ColumnFamilyHandle* cf,
                                              TablePropertiesCollection* v,
                                              Status* s) {
  *s = db->GetPropertiesOfAllTables(cf, v);
}

void crocksdb_get_properties_of_tables_in_range(DB* db, ColumnFamilyHandle* cf,
                                                TablePropertiesCollection* v,
                                                int num_ranges,
                                                const Range* ranges,
                                                Status* s) {
  *s = db->GetPropertiesOfTablesInRange(cf, ranges, num_ranges, v);
}

void crocksdb_options_set_bottommost_compression(ColumnFamilyOptions* opt,
                                                 CompressionType c) {
  opt->bottommost_compression = c;
}
// Get All Key Versions
void crocksdb_keyversions_destroy(crocksdb_keyversions_t* kvs) { delete kvs; }

crocksdb_keyversions_t* crocksdb_get_all_key_versions(
    DB* db, const char* begin_key, size_t begin_keylen, const char* end_key,
    size_t end_keylen, Status* s) {
  auto v = new crocksdb_keyversions_t;
  constexpr size_t kMaxNumKeys = std::numeric_limits<size_t>::max();
  *s = GetAllKeyVersions(db, Slice(begin_key, begin_keylen),
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
    std::unique_ptr<FSRandomAccessFile> sst_file;
    std::unique_ptr<RandomAccessFileReader> sst_file_reader;
    status = env_->GetFileSystem()->NewRandomAccessFile(
        file_, FileOptions(env_options_), &sst_file, nullptr);
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
        TableReaderOptions(ioptions, desc.options.prefix_extractor,
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
    uint64_t offset = props->external_sst_file_global_seqno_offset;
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
    DB* db, ColumnFamilyHandle* column_family, Slice file, uint64_t seq_no,
    Status* s) {
  auto env = db->GetEnv();
  EnvOptions env_options(db->GetDBOptions());
  ExternalSstFileModifier modifier(env, env_options, column_family);
  *s = modifier.Open(file.ToString());
  if (s->ok()) {
    uint64_t pre_seq_no;
    *s = modifier.SetGlobalSeqNo(seq_no, &pre_seq_no);
    return pre_seq_no;
  } else {
    return 0;
  }
}

void crocksdb_get_column_family_meta_data(DB* db, ColumnFamilyHandle* cf,
                                          ColumnFamilyMetaData* meta) {
  db->GetColumnFamilyMetaData(cf, meta);
}

ColumnFamilyMetaData* crocksdb_column_family_meta_data_create() {
  return new ColumnFamilyMetaData;
}

void crocksdb_column_family_meta_data_destroy(ColumnFamilyMetaData* data) {
  delete data;
}

size_t crocksdb_column_family_meta_data_level_count(
    const ColumnFamilyMetaData* meta) {
  return meta->levels.size();
}

const LevelMetaData* crocksdb_column_family_meta_data_level_data(
    const ColumnFamilyMetaData* meta, size_t n) {
  return &meta->levels[n];
}

size_t crocksdb_level_meta_data_file_count(const LevelMetaData* meta) {
  return meta->files.size();
}

const SstFileMetaData* crocksdb_level_meta_data_file_data(
    const LevelMetaData* meta, size_t n) {
  return &meta->files[n];
}

size_t crocksdb_sst_file_meta_data_size(const SstFileMetaData* meta) {
  return meta->size;
}

void crocksdb_sst_file_meta_data_name(const SstFileMetaData* meta,
                                      Slice* name) {
  *name = meta->name;
}

void crocksdb_sst_file_meta_data_smallestkey(const SstFileMetaData* meta,
                                             Slice* key) {
  *key = meta->smallestkey;
}

void crocksdb_sst_file_meta_data_largestkey(const SstFileMetaData* meta,
                                            Slice* key) {
  *key = meta->largestkey;
}

void crocksdb_compaction_options_init(CompactionOptions* opt) {
  *opt = CompactionOptions();
}

void crocksdb_compact_files_cf(DB* db, const CompactionOptions* opts,
                               ColumnFamilyHandle* cf,
                               const Slice* input_file_names,
                               size_t input_file_count, int output_level,
                               Status* s) {
  std::vector<std::string> input_files;
  for (size_t i = 0; i < input_file_count; i++) {
    input_files.push_back(input_file_names[i].ToString());
  }
  *s = db->CompactFiles(*opts, cf, input_files, output_level);
}

/* PerfContext */

PerfLevel crocksdb_get_perf_level(void) { return rocksdb::GetPerfLevel(); }

void crocksdb_set_perf_level(PerfLevel level) { rocksdb::SetPerfLevel(level); }

PerfFlags* crocksdb_create_perf_flags() { return new PerfFlags; }

void crocksdb_perf_flags_set(PerfFlags* flags, PerfFlag flag) {
  flags->set(flag);
}

void crocksdb_destroy_perf_flags(PerfFlags* flags) { delete flags; }

void crocksdb_set_perf_flags(const PerfFlags* flags) {
  rocksdb::SetPerfFlags(*flags);
}

PerfContext* crocksdb_get_perf_context() { return rocksdb::get_perf_context(); }

void crocksdb_perf_context_reset(PerfContext* ctx) { ctx->Reset(); }

uint64_t crocksdb_perf_context_user_key_comparison_count(
    const PerfContext* ctx) {
  return ctx->user_key_comparison_count;
}

uint64_t crocksdb_perf_context_block_cache_hit_count(const PerfContext* ctx) {
  return ctx->block_cache_hit_count;
}

uint64_t crocksdb_perf_context_block_read_count(const PerfContext* ctx) {
  return ctx->block_read_count;
}

uint64_t crocksdb_perf_context_block_read_byte(const PerfContext* ctx) {
  return ctx->block_read_byte;
}

uint64_t crocksdb_perf_context_block_read_time(const PerfContext* ctx) {
  return ctx->block_read_time;
}

uint64_t crocksdb_perf_context_block_cache_index_hit_count(
    const PerfContext* ctx) {
  return ctx->block_cache_index_hit_count;
}

uint64_t crocksdb_perf_context_index_block_read_count(const PerfContext* ctx) {
  return ctx->index_block_read_count;
}

uint64_t crocksdb_perf_context_block_cache_filter_hit_count(
    const PerfContext* ctx) {
  return ctx->block_cache_filter_hit_count;
}

uint64_t crocksdb_perf_context_filter_block_read_count(const PerfContext* ctx) {
  return ctx->filter_block_read_count;
}

uint64_t crocksdb_perf_context_compression_dict_block_read_count(
    const PerfContext* ctx) {
  return ctx->compression_dict_block_read_count;
}

uint64_t crocksdb_perf_context_block_checksum_time(const PerfContext* ctx) {
  return ctx->block_checksum_time;
}

uint64_t crocksdb_perf_context_block_decompress_time(const PerfContext* ctx) {
  return ctx->block_decompress_time;
}

uint64_t crocksdb_perf_context_get_read_bytes(const PerfContext* ctx) {
  return ctx->get_read_bytes;
}

uint64_t crocksdb_perf_context_multiget_read_bytes(const PerfContext* ctx) {
  return ctx->multiget_read_bytes;
}

uint64_t crocksdb_perf_context_iter_read_bytes(const PerfContext* ctx) {
  return ctx->iter_read_bytes;
}

uint64_t crocksdb_perf_context_internal_key_skipped_count(
    const PerfContext* ctx) {
  return ctx->internal_key_skipped_count;
}

uint64_t crocksdb_perf_context_internal_delete_skipped_count(
    const PerfContext* ctx) {
  return ctx->internal_delete_skipped_count;
}

uint64_t crocksdb_perf_context_internal_recent_skipped_count(
    const PerfContext* ctx) {
  return ctx->internal_recent_skipped_count;
}

uint64_t crocksdb_perf_context_internal_merge_count(const PerfContext* ctx) {
  return ctx->internal_merge_count;
}

uint64_t crocksdb_perf_context_get_snapshot_time(const PerfContext* ctx) {
  return ctx->get_snapshot_time;
}

uint64_t crocksdb_perf_context_get_from_memtable_time(const PerfContext* ctx) {
  return ctx->get_from_memtable_time;
}

uint64_t crocksdb_perf_context_get_from_memtable_count(const PerfContext* ctx) {
  return ctx->get_from_memtable_count;
}

uint64_t crocksdb_perf_context_get_post_process_time(const PerfContext* ctx) {
  return ctx->get_post_process_time;
}

uint64_t crocksdb_perf_context_get_from_output_files_time(
    const PerfContext* ctx) {
  return ctx->get_from_output_files_time;
}

uint64_t crocksdb_perf_context_seek_on_memtable_time(const PerfContext* ctx) {
  return ctx->seek_on_memtable_time;
}

uint64_t crocksdb_perf_context_seek_on_memtable_count(const PerfContext* ctx) {
  return ctx->seek_on_memtable_count;
}

uint64_t crocksdb_perf_context_next_on_memtable_count(const PerfContext* ctx) {
  return ctx->next_on_memtable_count;
}

uint64_t crocksdb_perf_context_prev_on_memtable_count(const PerfContext* ctx) {
  return ctx->prev_on_memtable_count;
}

uint64_t crocksdb_perf_context_seek_child_seek_time(const PerfContext* ctx) {
  return ctx->seek_child_seek_time;
}

uint64_t crocksdb_perf_context_seek_child_seek_count(const PerfContext* ctx) {
  return ctx->seek_child_seek_count;
}

uint64_t crocksdb_perf_context_seek_min_heap_time(const PerfContext* ctx) {
  return ctx->seek_min_heap_time;
}

uint64_t crocksdb_perf_context_seek_max_heap_time(const PerfContext* ctx) {
  return ctx->seek_max_heap_time;
}

uint64_t crocksdb_perf_context_seek_internal_seek_time(const PerfContext* ctx) {
  return ctx->seek_internal_seek_time;
}

uint64_t crocksdb_perf_context_find_next_user_entry_time(
    const PerfContext* ctx) {
  return ctx->find_next_user_entry_time;
}

uint64_t crocksdb_perf_context_write_wal_time(const PerfContext* ctx) {
  return ctx->write_wal_time;
}

uint64_t crocksdb_perf_context_write_memtable_time(const PerfContext* ctx) {
  return ctx->write_memtable_time;
}

uint64_t crocksdb_perf_context_write_delay_time(const PerfContext* ctx) {
  return ctx->write_delay_time;
}

uint64_t crocksdb_perf_context_write_pre_and_post_process_time(
    const PerfContext* ctx) {
  return ctx->write_pre_and_post_process_time;
}

uint64_t crocksdb_perf_context_db_mutex_lock_nanos(const PerfContext* ctx) {
  return ctx->db_mutex_lock_nanos;
}

uint64_t crocksdb_perf_context_write_thread_wait_nanos(const PerfContext* ctx) {
  return ctx->write_thread_wait_nanos;
}

uint64_t crocksdb_perf_context_write_scheduling_flushes_compactions_time(
    const PerfContext* ctx) {
  return ctx->write_scheduling_flushes_compactions_time;
}

uint64_t crocksdb_perf_context_db_condition_wait_nanos(const PerfContext* ctx) {
  return ctx->db_condition_wait_nanos;
}

uint64_t crocksdb_perf_context_merge_operator_time_nanos(
    const PerfContext* ctx) {
  return ctx->merge_operator_time_nanos;
}

uint64_t crocksdb_perf_context_read_index_block_nanos(const PerfContext* ctx) {
  return ctx->read_index_block_nanos;
}

uint64_t crocksdb_perf_context_read_filter_block_nanos(const PerfContext* ctx) {
  return ctx->read_filter_block_nanos;
}

uint64_t crocksdb_perf_context_new_table_block_iter_nanos(
    const PerfContext* ctx) {
  return ctx->new_table_block_iter_nanos;
}

uint64_t crocksdb_perf_context_new_table_iterator_nanos(
    const PerfContext* ctx) {
  return ctx->new_table_iterator_nanos;
}

uint64_t crocksdb_perf_context_block_seek_nanos(const PerfContext* ctx) {
  return ctx->block_seek_nanos;
}

uint64_t crocksdb_perf_context_find_table_nanos(const PerfContext* ctx) {
  return ctx->find_table_nanos;
}

uint64_t crocksdb_perf_context_bloom_memtable_hit_count(
    const PerfContext* ctx) {
  return ctx->bloom_memtable_hit_count;
}

uint64_t crocksdb_perf_context_bloom_memtable_miss_count(
    const PerfContext* ctx) {
  return ctx->bloom_memtable_miss_count;
}

uint64_t crocksdb_perf_context_bloom_sst_hit_count(const PerfContext* ctx) {
  return ctx->bloom_sst_hit_count;
}

uint64_t crocksdb_perf_context_bloom_sst_miss_count(const PerfContext* ctx) {
  return ctx->bloom_sst_miss_count;
}

uint64_t crocksdb_perf_context_key_lock_wait_time(const PerfContext* ctx) {
  return ctx->key_lock_wait_time;
}

uint64_t crocksdb_perf_context_key_lock_wait_count(const PerfContext* ctx) {
  return ctx->key_lock_wait_count;
}

uint64_t crocksdb_perf_context_env_new_sequential_file_nanos(
    const PerfContext* ctx) {
  return ctx->env_new_sequential_file_nanos;
}

uint64_t crocksdb_perf_context_env_new_random_access_file_nanos(
    const PerfContext* ctx) {
  return ctx->env_new_random_access_file_nanos;
}

uint64_t crocksdb_perf_context_env_new_writable_file_nanos(
    const PerfContext* ctx) {
  return ctx->env_new_writable_file_nanos;
}

uint64_t crocksdb_perf_context_env_reuse_writable_file_nanos(
    const PerfContext* ctx) {
  return ctx->env_reuse_writable_file_nanos;
}

uint64_t crocksdb_perf_context_env_new_random_rw_file_nanos(
    const PerfContext* ctx) {
  return ctx->env_new_random_rw_file_nanos;
}

uint64_t crocksdb_perf_context_env_new_directory_nanos(const PerfContext* ctx) {
  return ctx->env_new_directory_nanos;
}

uint64_t crocksdb_perf_context_env_file_exists_nanos(const PerfContext* ctx) {
  return ctx->env_file_exists_nanos;
}

uint64_t crocksdb_perf_context_env_get_children_nanos(const PerfContext* ctx) {
  return ctx->env_get_children_nanos;
}

uint64_t crocksdb_perf_context_env_get_children_file_attributes_nanos(
    const PerfContext* ctx) {
  return ctx->env_get_children_file_attributes_nanos;
}

uint64_t crocksdb_perf_context_env_delete_file_nanos(const PerfContext* ctx) {
  return ctx->env_delete_file_nanos;
}

uint64_t crocksdb_perf_context_env_create_dir_nanos(const PerfContext* ctx) {
  return ctx->env_create_dir_nanos;
}

uint64_t crocksdb_perf_context_env_create_dir_if_missing_nanos(
    const PerfContext* ctx) {
  return ctx->env_create_dir_if_missing_nanos;
}

uint64_t crocksdb_perf_context_env_delete_dir_nanos(const PerfContext* ctx) {
  return ctx->env_delete_dir_nanos;
}

uint64_t crocksdb_perf_context_env_get_file_size_nanos(const PerfContext* ctx) {
  return ctx->env_get_file_size_nanos;
}

uint64_t crocksdb_perf_context_env_get_file_modification_time_nanos(
    const PerfContext* ctx) {
  return ctx->env_get_file_modification_time_nanos;
}

uint64_t crocksdb_perf_context_env_rename_file_nanos(const PerfContext* ctx) {
  return ctx->env_rename_file_nanos;
}

uint64_t crocksdb_perf_context_env_link_file_nanos(const PerfContext* ctx) {
  return ctx->env_link_file_nanos;
}

uint64_t crocksdb_perf_context_env_lock_file_nanos(const PerfContext* ctx) {
  return ctx->env_lock_file_nanos;
}

uint64_t crocksdb_perf_context_env_unlock_file_nanos(const PerfContext* ctx) {
  return ctx->env_unlock_file_nanos;
}

uint64_t crocksdb_perf_context_env_new_logger_nanos(const PerfContext* ctx) {
  return ctx->env_new_logger_nanos;
}

uint64_t crocksdb_perf_context_get_cpu_nanos(const PerfContext* ctx) {
  return ctx->get_cpu_nanos;
}

uint64_t crocksdb_perf_context_iter_next_cpu_nanos(const PerfContext* ctx) {
  return ctx->iter_next_cpu_nanos;
}

uint64_t crocksdb_perf_context_iter_prev_cpu_nanos(const PerfContext* ctx) {
  return ctx->iter_next_cpu_nanos;
}

uint64_t crocksdb_perf_context_iter_seek_cpu_nanos(const PerfContext* ctx) {
  return ctx->iter_next_cpu_nanos;
}

uint64_t crocksdb_perf_context_encrypt_data_nanos(const PerfContext* ctx) {
  return ctx->encrypt_data_nanos;
}

uint64_t crocksdb_perf_context_decrypt_data_nanos(const PerfContext* ctx) {
  return ctx->decrypt_data_nanos;
}

IOStatsContext* crocksdb_get_iostats_context(void) {
  return rocksdb::get_iostats_context();
}

void crocksdb_iostats_context_reset(IOStatsContext* ctx) { ctx->Reset(); }

uint64_t crocksdb_iostats_context_thread_pool_id(const IOStatsContext* ctx) {
  return ctx->thread_pool_id;
}

uint64_t crocksdb_iostats_context_bytes_written(const IOStatsContext* ctx) {
  return ctx->bytes_written;
}

uint64_t crocksdb_iostats_context_bytes_read(const IOStatsContext* ctx) {
  return ctx->bytes_read;
}

uint64_t crocksdb_iostats_context_open_nanos(const IOStatsContext* ctx) {
  return ctx->open_nanos;
}

uint64_t crocksdb_iostats_context_allocate_nanos(const IOStatsContext* ctx) {
  return ctx->allocate_nanos;
}

uint64_t crocksdb_iostats_context_write_nanos(const IOStatsContext* ctx) {
  return ctx->write_nanos;
}

uint64_t crocksdb_iostats_context_read_nanos(const IOStatsContext* ctx) {
  return ctx->read_nanos;
}

uint64_t crocksdb_iostats_context_range_sync_nanos(const IOStatsContext* ctx) {
  return ctx->range_sync_nanos;
}

uint64_t crocksdb_iostats_context_fsync_nanos(const IOStatsContext* ctx) {
  return ctx->fsync_nanos;
}

uint64_t crocksdb_iostats_context_prepare_write_nanos(
    const IOStatsContext* ctx) {
  return ctx->prepare_write_nanos;
}

uint64_t crocksdb_iostats_context_logger_nanos(const IOStatsContext* ctx) {
  return ctx->logger_nanos;
}

uint64_t crocksdb_iostats_context_cpu_write_nanos(const IOStatsContext* ctx) {
  return ctx->cpu_write_nanos;
}

uint64_t crocksdb_iostats_context_cpu_read_nanos(const IOStatsContext* ctx) {
  return ctx->cpu_read_nanos;
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
    return should_partition_cb(underlying, &request);
  }

  bool CanDoTrivialMove(const Slice& smallest_user_key,
                        const Slice& largest_user_key) override {
    return can_do_trivial_move_cb(underlying, smallest_user_key,
                                  largest_user_key);
  }
};

SstPartitioner* crocksdb_sst_partitioner_create(
    void* underlying, void (*destructor)(void*),
    crocksdb_sst_partitioner_should_partition_cb should_partition_cb,
    crocksdb_sst_partitioner_can_do_trivial_move_cb can_do_trivial_move_cb) {
  crocksdb_sst_partitioner_impl_t* sst_partitioner_impl =
      new crocksdb_sst_partitioner_impl_t;
  sst_partitioner_impl->underlying = underlying;
  sst_partitioner_impl->destructor = destructor;
  sst_partitioner_impl->should_partition_cb = should_partition_cb;
  sst_partitioner_impl->can_do_trivial_move_cb = can_do_trivial_move_cb;
  return sst_partitioner_impl;
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
    auto* partitioner = create_partitioner_cb(underlying, &partitioner_context);
    if (partitioner == nullptr) {
      return nullptr;
    }
    return std::unique_ptr<SstPartitioner>(partitioner);
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

/* Tools */

void crocksdb_run_ldb_tool(size_t argc, const char* const* argv,
                           const Options* opts, size_t cf_count,
                           const Slice* cf_names,
                           const ColumnFamilyOptions* const* cf_opts) {
  std::vector<ColumnFamilyDescriptor> column_families;
  for (size_t i = 0; i < cf_count; i++) {
    column_families.push_back(
        ColumnFamilyDescriptor(cf_names[i].ToString(), *cf_opts[i]));
  }

  // Bad cast due to bad interface provided by rocksdb. It's cast back to const
  // later.
  LDBTool().Run(argc, (char**)argv, *opts, LDBOptions(), &column_families);
}

void crocksdb_run_sst_dump_tool(size_t argc, const char* const* argv,
                                const Options* opts) {
  // Bad cast due to bad interface provided by rocksdb. It's cast back to const
  // later.
  SSTDumpTool().Run(argc, (char**)argv, *opts);
}

/* Titan */

DB* ctitandb_open_column_families(
    Slice name, const TitanDBOptions* tdb_options, int num_column_families,
    const Slice* column_family_names,
    const TitanCFOptions* const* titan_column_family_options,
    ColumnFamilyHandle** column_family_handles, Status* s) {
  std::vector<TitanCFDescriptor> column_families;
  for (int i = 0; i < num_column_families; i++) {
    column_families.push_back(TitanCFDescriptor(
        column_family_names[i].ToString(), *titan_column_family_options[i]));
  }

  TitanDB* db;
  std::vector<ColumnFamilyHandle*> handles;
  *s = TitanDB::Open(*tdb_options, name.ToString(), column_families, &handles,
                     &db);
  if (!s->ok()) {
    return nullptr;
  }
  for (size_t i = 0; i < handles.size(); i++) {
    column_family_handles[i] = handles[i];
  }
  return db;
}

// Caller should make sure `db` is created from ctitandb_open_column_families.
//
// TODO: ctitandb_open_column_family should return a ctitandb_t. Caller can
// use ctitandb_t for titan specific functions.
ColumnFamilyHandle* ctitandb_create_column_family(
    DB* db, const TitanCFOptions* titan_column_family_options,
    Slice column_family_name, Status* s) {
  // Blindly cast db into TitanDB.
  TitanDB* titan_db = reinterpret_cast<TitanDB*>(db);
  ColumnFamilyHandle* handle = nullptr;
  *s = titan_db->CreateColumnFamily(
      TitanCFDescriptor(column_family_name.ToString(),
                        *titan_column_family_options),
      &handle);
  return handle;
}

/* TitanDBOptions */

TitanOptions* ctitandb_options_create() { return new TitanOptions; }

TitanDBOptions* ctitandb_options_get_dboptions(TitanOptions* opt) {
  return static_cast<TitanDBOptions*>(opt);
}

TitanCFOptions* ctitandb_options_get_cfoptions(TitanOptions* opt) {
  return static_cast<TitanCFOptions*>(opt);
}

void ctitandb_options_destroy(TitanOptions* opt) { delete opt; }

TitanDBOptions* ctitandb_dboptions_create() { return new TitanDBOptions(); }

void ctitandb_dboptions_destroy(TitanDBOptions* opt) { delete opt; }

TitanCFOptions* ctitandb_cfoptions_create() { return new TitanCFOptions(); }

TitanCFOptions* ctitandb_cfoptions_from_rocksdb(ColumnFamilyOptions* rocks) {
  return new TitanCFOptions(*rocks);
}

ColumnFamilyOptions* ctitandb_cfoptions_to_rocksdb(TitanCFOptions* titan) {
  return new ColumnFamilyOptions(*titan);
}

void ctitandb_cfoptions_destroy(TitanCFOptions* opt) { delete opt; }

TitanOptions* ctitandb_get_titan_options_cf(const DB* db,
                                            ColumnFamilyHandle* column_family) {
  const TitanDB* titan_db = reinterpret_cast<const TitanDB*>(db);
  auto opt = titan_db->GetTitanOptions(column_family);
  return new TitanOptions(opt);
}

TitanDBOptions* ctitandb_get_titan_db_options(const DB* db) {
  const TitanDB* titan_db = reinterpret_cast<const TitanDB*>(db);
  auto opt = titan_db->GetTitanDBOptions();
  return new TitanDBOptions(opt);
}

void ctitandb_options_get_dirname(const TitanDBOptions* opts, Slice* dir) {
  *dir = opts->dirname;
}

void ctitandb_options_set_dirname(TitanDBOptions* opts, Slice name) {
  opts->dirname = name.ToString();
}

uint64_t ctitandb_options_get_min_blob_size(const TitanCFOptions* opts) {
  return opts->min_blob_size;
}

void ctitandb_options_set_min_blob_size(TitanCFOptions* opts, uint64_t size) {
  opts->min_blob_size = size;
}

CompressionType ctitandb_options_blob_file_compression(
    const TitanCFOptions* opts) {
  return opts->blob_file_compression;
}

void ctitandb_options_set_blob_file_compression(TitanCFOptions* opts,
                                                CompressionType type) {
  opts->blob_file_compression = type;
}

CompressionOptions* ctitandb_options_get_blob_file_compression_options(
    TitanCFOptions* opt) {
  return &opt->blob_file_compression_options;
}

void ctitandb_decode_blob_index(Slice value, ctitandb_blob_index_t* index,
                                Status* s) {
  BlobIndex bi;
  *s = bi.DecodeFrom(&value);
  if (!s->ok()) {
    return;
  }
  index->file_number = bi.file_number;
  index->blob_offset = bi.blob_handle.offset;
  index->blob_size = bi.blob_handle.size;
}

void ctitandb_encode_blob_index(const ctitandb_blob_index_t* index, void* ctx,
                                bytes_receiver_cb fp) {
  BlobIndex bi;
  bi.file_number = index->file_number;
  bi.blob_handle.offset = index->blob_offset;
  bi.blob_handle.size = index->blob_size;
  std::string result;
  bi.EncodeTo(&result);
  fp(ctx, result);
}

void ctitandb_options_set_disable_background_gc(TitanDBOptions* options,
                                                bool disable) {
  options->disable_background_gc = disable;
}

void ctitandb_options_set_level_merge(TitanCFOptions* options, bool enable) {
  options->level_merge = enable;
}

void ctitandb_options_set_range_merge(TitanCFOptions* options, bool enable) {
  options->range_merge = enable;
}

void ctitandb_options_set_max_sorted_runs(TitanCFOptions* options, int size) {
  options->max_sorted_runs = size;
}

void ctitandb_options_set_max_gc_batch_size(TitanCFOptions* options,
                                            uint64_t size) {
  options->max_gc_batch_size = size;
}

void ctitandb_options_set_min_gc_batch_size(TitanCFOptions* options,
                                            uint64_t size) {
  options->min_gc_batch_size = size;
}

void ctitandb_options_set_blob_file_discardable_ratio(TitanCFOptions* options,
                                                      double ratio) {
  options->blob_file_discardable_ratio = ratio;
}

void ctitandb_options_set_merge_small_file_threshold(TitanCFOptions* options,
                                                     uint64_t size) {
  options->merge_small_file_threshold = size;
}

void ctitandb_options_set_max_background_gc(TitanDBOptions* options,
                                            int32_t size) {
  options->max_background_gc = size;
}

void ctitandb_options_set_purge_obsolete_files_period_sec(
    TitanDBOptions* options, uint32_t period) {
  options->purge_obsolete_files_period_sec = period;
}

void ctitandb_options_set_blob_cache(TitanCFOptions* options,
                                     crocksdb_cache_t* cache) {
  options->blob_cache = cache->rep;
}

size_t ctitandb_options_get_blob_cache_usage(const TitanCFOptions* opt) {
  if (opt->blob_cache != nullptr) {
    return opt->blob_cache->GetUsage();
  }
  return 0;
}

void ctitandb_options_set_blob_cache_capacity(TitanCFOptions* opt,
                                              size_t capacity, Status* s) {
  if (opt->blob_cache != nullptr) {
    opt->blob_cache->SetCapacity(capacity);
    *s = Status::OK();
  } else {
    *s = Status::InvalidArgument("Blob cache was disabled.");
  }
}

size_t ctitandb_options_get_blob_cache_capacity(const TitanCFOptions* opt) {
  if (opt->blob_cache != nullptr) {
    return opt->blob_cache->GetCapacity();
  }
  return 0;
}

void ctitandb_options_set_discardable_ratio(TitanCFOptions* options,
                                            double ratio) {
  options->blob_file_discardable_ratio = ratio;
}

void ctitandb_options_set_blob_run_mode(TitanCFOptions* options,
                                        TitanBlobRunMode mode) {
  options->blob_run_mode = mode;
}

/* TitanReadOptions */

void ctitandb_readoptions_init(TitanReadOptions* opt) {
  new (opt) TitanReadOptions;
}

void ctitandb_readoptions_inplace_destroy(TitanReadOptions* opt) {
  opt->~TitanReadOptions();
}

void ctitandb_readoptions_set_key_only(TitanReadOptions* opt, bool v) {
  opt->key_only = v;
}

Iterator* ctitandb_create_iterator(DB* db,
                                   const TitanReadOptions* titan_options) {
  return static_cast<TitanDB*>(db)->NewIterator(*titan_options);
}

Iterator* ctitandb_create_iterator_cf(DB* db,
                                      const TitanReadOptions* titan_options,
                                      ColumnFamilyHandle* column_family) {
  return static_cast<TitanDB*>(db)->NewIterator(*titan_options, column_family);
}

void ctitandb_create_iterators(DB* db, const TitanReadOptions* titan_options,
                               ColumnFamilyHandle** column_families,
                               Iterator** iterators, size_t size, Status* s) {
  std::vector<ColumnFamilyHandle*> column_families_vec(column_families,
                                                       column_families + size);

  std::vector<Iterator*> res;
  *s = static_cast<TitanDB*>(db)->NewIterators(*titan_options,
                                               column_families_vec, &res);
  if (!s->ok()) {
    for (size_t i = 0; i < res.size(); i++) {
      delete res[i];
    }
    return;
  }
  assert(res.size() == size);

  for (size_t i = 0; i < size; i++) {
    iterators[i] = res[i];
  }
}

void ctitandb_delete_files_in_ranges_cf(DB* db, ColumnFamilyHandle* cf,
                                        const RangePtr* ranges,
                                        size_t num_ranges, bool include_end,
                                        Status* s) {
  *s = static_cast<TitanDB*>(db)->DeleteFilesInRanges(cf, ranges, num_ranges,
                                                      include_end);
}

void ctitandb_delete_blob_files_in_ranges_cf(DB* db, ColumnFamilyHandle* cf,
                                             const RangePtr* ranges,
                                             size_t num_ranges,
                                             bool include_end, Status* s) {
  *s = static_cast<TitanDB*>(db)->DeleteBlobFilesInRanges(
      cf, ranges, num_ranges, include_end);
}

void ctitandb_property_name_num_blob_files_at_level_prefix(Slice* s) {
  *s = TitanDB::Properties::kNumBlobFilesAtLevelPrefix;
}

void ctitandb_property_name_live_blob_size(Slice* s) {
  *s = TitanDB::Properties::kLiveBlobSize;
}

void ctitandb_property_name_num_live_blob_file(Slice* s) {
  *s = TitanDB::Properties::kNumLiveBlobFile;
}

void ctitandb_property_name_num_obsolete_blob_file(Slice* s) {
  *s = TitanDB::Properties::kNumObsoleteBlobFile;
}

void ctitandb_property_name_live_blob_file_size(Slice* s) {
  *s = TitanDB::Properties::kLiveBlobFileSize;
}

void ctitandb_property_name_obsolete_blob_file_size(Slice* s) {
  *s = TitanDB::Properties::kObsoleteBlobFileSize;
}

void ctitandb_property_name_num_discardable_ratio_le0_file(Slice* s) {
  *s = TitanDB::Properties::kNumDiscardableRatioLE0File;
}

void ctitandb_property_name_num_discardable_ratio_le20_file(Slice* s) {
  *s = TitanDB::Properties::kNumDiscardableRatioLE20File;
}

void ctitandb_property_name_num_discardable_ratio_le50_file(Slice* s) {
  *s = TitanDB::Properties::kNumDiscardableRatioLE50File;
}

void ctitandb_property_name_num_discardable_ratio_le80_file(Slice* s) {
  *s = TitanDB::Properties::kNumDiscardableRatioLE80File;
}

void ctitandb_property_name_num_discardable_ratio_le100_file(Slice* s) {
  *s = TitanDB::Properties::kNumDiscardableRatioLE100File;
}

void crocksdb_free_cplus_array(const char* arr) { delete[] arr; }

const char* crocksdb_to_cplus_array(Slice s) {
  char* const result = new char[s.size() + 1];  // +1 for null terminator
  memcpy(result, s.data(), s.size());
  result[s.size()] = '\0';  // null terminator for C style string
  return result;
}

}  // end extern "C"
