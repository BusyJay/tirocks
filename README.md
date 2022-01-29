# tirocks

This library has been tested against RocksDB 6.4 on Linux and macOS.

Feedback and pull requests welcome! If a particular feature of RocksDB is important to you, please let us know by opening an issue, and we will prioritize it.

## Build

```
$ cargo xtask submodule # if you just cloned the repository
$ cargo build
```

Bindings are pre-generated, if the content in librocksdb_sys/crocksdb/crocksdb/c.h is updated, you may need to regenerate bindings:

```
$ cargo xtask bindgen
```

And running linting against C files:

```
$ cargo xtask clang-lint
```
