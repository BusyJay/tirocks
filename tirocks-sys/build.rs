// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use cc::Build;
use cmake::Config;
use std::path::{Path, PathBuf};
use std::{env, str};

// On these platforms jemalloc-sys will use a prefixed jemalloc which cannot be linked together
// with RocksDB.
// See https://github.com/gnzlbg/jemallocator/blob/bfc89192971e026e6423d9ee5aaa02bc56585c58/jemalloc-sys/build.rs#L45
const NO_JEMALLOC_TARGETS: &[&str] = &["android", "dragonfly", "musl", "darwin"];

// Generate the bindings to rocksdb C-API.
// Try to disable the generation of platform-related bindings.
#[cfg(feature = "update-bindings")]
fn bindgen_rocksdb(file_path: &Path) {
    println!("cargo:rerun-if-env-changed=TEST_BIND");
    let gen_tests = env::var("TEST_BIND").map_or(false, |s| s == "1");

    let mut builder = bindgen::Builder::default();

    if env::var("CARGO_CFG_TARGET_OS").map_or(false, |s| s == "windows") {
        builder = builder.clang_arg("-D _WIN32_WINNT=0x600");
    }
    // TODO: generate bindings of encryption
    let builder = builder
        .header("crocksdb/crocksdb/c.h")
        .header("rocksdb/include/rocksdb/statistics.h")
        .header("titan/include/titan/statistics.h")
        .clang_arg("-xc++")
        .clang_arg("-Irocksdb/include")
        .clang_arg("-Ititan/include")
        .clang_arg("-std=c++11")
        .rustfmt_bindings(true)
        .allowlist_function(r"\bcrocksdb_.*")
        .allowlist_type(r"\bcrocksdb_.*")
        .allowlist_var(r"\bcrocksdb_.*")
        .allowlist_function(r"\bctitandb_.*")
        .allowlist_type(r"\bctitandb_.*")
        .allowlist_type(r"\brocksdb::Tickers")
        .allowlist_type(r"\brocksdb::Histograms")
        .allowlist_type(r"\brocksdb::titandb::TickerType")
        .allowlist_type(r"\brocksdb::titandb::HistogramType")
        // Block all system headers
        .blocklist_file(r"^/.*")
        .derive_copy(false)
        .size_t_is_usize(true)
        .disable_header_comment()
        .ctypes_prefix("libc")
        .layout_tests(gen_tests)
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: false,
        });
    println!("running {}", builder.command_line_flags().join(" "));
    let bindings = builder
        .generate()
        .expect("unable to generate rocksdb bindings");

    bindings
        .write_to_file(file_path)
        .expect("unable to write rocksdb bindings");
}

/// Determine if need to update bindings. When `update-bindings` feature
/// is enabled, it will regenerate the bindings.
fn config_binding_path() {
    // Cargo treats nonexistent files changed, so we only emit the rerun-if-changed
    // directive when we expect the target-specific pre-generated binding file to be
    // present.
    println!("cargo:rerun-if-changed=bindings/bindings.rs");

    let file_path =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("bindings/bindings.rs");

    #[cfg(feature = "update-bindings")]
    bindgen_rocksdb(&file_path);

    println!(
        "cargo:rustc-env=BINDING_PATH={}",
        file_path.to_str().unwrap()
    );
}

fn link_cpp(build: &mut Build) {
    let tool = build.get_compiler();
    let stdlib = if tool.is_like_gnu() {
        "libstdc++.a"
    } else if tool.is_like_clang() {
        "libc++.a"
    } else {
        // Don't link to c++ statically on windows.
        return;
    };
    let output = tool
        .to_command()
        .arg("--print-file-name")
        .arg(stdlib)
        .output()
        .unwrap();
    if !output.status.success() || output.stdout.is_empty() {
        // fallback to dynamically
        return;
    }
    let path = match str::from_utf8(&output.stdout) {
        Ok(path) => PathBuf::from(path),
        Err(_) => return,
    };
    if !path.is_absolute() {
        return;
    }
    // remove lib prefix and .a postfix.
    let libname = &stdlib[3..stdlib.len() - 2];
    // optional static linking
    if cfg!(feature = "static-libcpp") {
        println!("cargo:rustc-link-lib=static={}", &libname);
    } else {
        println!("cargo:rustc-link-lib=dylib={}", &libname);
    }
    println!(
        "cargo:rustc-link-search=native={}",
        path.parent().unwrap().display()
    );
    build.cpp_link_stdlib(None);
}

fn patch_libz_env() {
    // cmake script expect libz.a being under ${DEP_Z_ROOT}/lib, but libz-sys crate put it
    // under ${DEP_Z_ROOT}/build. Append the path to CMAKE_PREFIX_PATH to get around it.
    let zlib_root = env::var("DEP_Z_ROOT").unwrap();
    let prefix_path = if let Ok(prefix_path) = env::var("CMAKE_PREFIX_PATH") {
        format!("{};{}/build", prefix_path, zlib_root)
    } else {
        format!("{}/build", zlib_root)
    };
    // To avoid linking system library, set lib path explicitly.
    println!("cargo:rustc-link-search=native={}/build", zlib_root);
    println!("cargo:rustc-link-search=native={}/lib", zlib_root);
    env::set_var("CMAKE_PREFIX_PATH", prefix_path);
}

fn configure_common_rocksdb_args(cfg: &mut Config, name: &str) {
    let out_dir = format!("{}/{}", env::var("OUT_DIR").unwrap(), name);
    std::fs::create_dir_all(&out_dir).unwrap();
    cfg.out_dir(out_dir);
    if cfg!(feature = "portable") {
        cfg.define("PORTABLE", "ON");
    }
    if cfg!(feature = "sse") {
        cfg.define("FORCE_SSE42", "ON");
    }
    cfg.define("WITH_GFLAGS", "OFF")
        .register_dep("Z")
        .define("WITH_ZLIB", "ON")
        .register_dep("BZIP2")
        .define("WITH_BZ2", "ON")
        .register_dep("LZ4")
        .define("WITH_LZ4", "ON")
        .register_dep("ZSTD")
        .define("WITH_ZSTD", "ON")
        .register_dep("SNAPPY")
        .define("WITH_SNAPPY", "ON")
        .configure_arg("-Wno-dev");
}

fn figure_link_lib(dst: &Path, name: &str) {
    if cfg!(target_os = "windows") {
        let profile = match &*env::var("PROFILE").unwrap_or_else(|_| "debug".to_owned()) {
            "bench" | "release" => "Release",
            _ => "Debug",
        };
        println!(
            "cargo:rustc-link-search=native={}/build/{}",
            dst.display(),
            profile
        );
    } else {
        println!("cargo:rustc-link-search=native={}/build", dst.display());
    }
    println!("cargo:rustc-link-lib=static={}", name);
}

fn build_titan(build: &mut Build) {
    let cur_dir = std::env::current_dir().unwrap();
    let mut cfg = cmake::Config::new("titan");
    configure_common_rocksdb_args(&mut cfg, "titan");
    let dst = cfg
        .define("ROCKSDB_DIR", cur_dir.join("rocksdb"))
        .define("WITH_TITAN_TESTS", "OFF")
        .define("WITH_TITAN_TOOLS", "OFF")
        .build_target("titan")
        .very_verbose(true)
        .build();
    figure_link_lib(&dst, "titan");
    build.include(cur_dir.join("titan").join("include"));
    build.include(cur_dir.join("titan"));
}

fn build_rocksdb(build: &mut Build) {
    let target = env::var("TARGET").expect("TARGET was not set");
    let mut cfg = Config::new("rocksdb");
    cfg.out_dir(format!("{}/rocksdb", env::var("OUT_DIR").unwrap()));
    if cfg!(feature = "encryption") {
        cfg.register_dep("OPENSSL").define("WITH_OPENSSL", "ON");
        println!("cargo:rustc-link-lib=static=crypto");
    }
    if cfg!(feature = "jemalloc") && NO_JEMALLOC_TARGETS.iter().all(|i| !target.contains(i)) {
        cfg.register_dep("JEMALLOC").define("WITH_JEMALLOC", "ON");
        println!("cargo:rustc-link-lib=static=jemalloc");
    }
    configure_common_rocksdb_args(&mut cfg, "rocksdb");
    let dst = cfg
        .define("WITH_TESTS", "OFF")
        .define("WITH_TOOLS", "OFF")
        .build_target("rocksdb")
        .very_verbose(true)
        .build();
    figure_link_lib(&dst, "rocksdb");

    if cfg!(target_os = "windows") {
        build.define("OS_WIN", None);
    } else {
        build.define("ROCKSDB_PLATFORM_POSIX", None);
    }
    if cfg!(target_os = "macos") {
        build.define("OS_MACOSX", None);
    } else if cfg!(target_os = "freebsd") {
        build.define("OS_FREEBSD", None);
    }

    config_binding_path();

    let cur_dir = env::current_dir().unwrap();
    build.include(cur_dir.join("rocksdb").join("include"));
    build.include(cur_dir.join("rocksdb"));

    // Adding rocksdb specific compile macros.
    // TODO: should make sure crocksdb compile options is the same as rocksdb and titan.
    build.define("ROCKSDB_SUPPORT_THREAD_LOCAL", None);
    if cfg!(feature = "encryption") {
        build.define("OPENSSL", None);
    }

    println!("cargo:rustc-link-lib=static=z");
    println!("cargo:rustc-link-lib=static=bz2");
    println!("cargo:rustc-link-lib=static=lz4");
    println!("cargo:rustc-link-lib=static=zstd");
    println!("cargo:rustc-link-lib=static=snappy");
}

fn main() {
    patch_libz_env();
    let mut build = Build::new();
    build_titan(&mut build);
    build_rocksdb(&mut build);

    build.cpp(true).file("crocksdb/c.cc");
    if !cfg!(target_os = "windows") {
        build.flag("-std=c++11");
        build.flag("-fno-rtti");
    }
    link_cpp(&mut build);
    build.warnings(false).compile("libcrocksdb.a");
}
