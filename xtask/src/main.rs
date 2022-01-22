use std::fs::{self, File};
use std::process::{self, Command};
use std::{
    env,
    io::{Read, Write},
    str,
};

fn print_help() {
    eprintln!("cargo xtask [subcommand]");
    eprintln!();
    eprintln!("Supported subcommands are:");
    eprintln!("\tbindgen\tGenerate rust-bindgen for tirocks-sys package");
    eprintln!("\tsubmodule\tInit necessary submodules for compilation");
    eprintln!("\tclang-lint\tLint cpp code in tiocks-sys package");
}

fn cargo() -> Command {
    match env::var("CARGO") {
        Ok(s) => Command::new(s),
        Err(_) => {
            eprintln!("no CARGO in environment variables, please invoke the binary by cargo xtask");
            process::exit(1);
        }
    }
}

fn exec(c: &mut Command) {
    if let Err(e) = c.status() {
        eprintln!("failed to execute {:?}: {}", c, e);
        process::exit(-1);
    }
}

fn remove_match(data: &str, pattern: impl Fn(&str) -> bool) -> String {
    let mut target = String::with_capacity(data.len());
    for l in data.lines() {
        if pattern(l) {
            continue;
        }
        target.push_str(l);
        target.push('\n');
    }
    target
}

fn bindgen() {
    fs::create_dir_all("tirocks-sys/bindings").unwrap();
    exec(
        cargo()
            .env("UPDATE_BIND", "1")
            .args(&["build", "-p", "tirocks-sys", "--features", "update-bindings"]),
    );
    for f in fs::read_dir("tirocks-sys/bindings").unwrap() {
        let p = f.unwrap().path();
        let mut content = String::new();
        File::open(&p)
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();
        let content = remove_match(&content, |l| {
            l.starts_with("pub type ") && l.contains("= ::std::os::raw::")
        });
        File::create(&p)
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();
    }
}

fn cmd(c: &str) -> Command {
    Command::new(c)
}

fn submodule() {
    exec(cmd("git").args(&["submodule", "update", "--init"]));
}

fn clang_lint() {
    exec(cmd("clang-tidy").args(&[
        "tirocks-sys/crocksdb/c.cc",
        "tirocks-sys/crocksdb/crocksdb/c.h",
        "--",
        "-Itirocks-sys/rocksdb/include",
        "-Itirocks-sys/titan/include",
        "-x",
        "c++",
        "-std=c++11",
    ]));
    exec(cmd("clang-format").args(&[
        "-i",
        "tirocks-sys/crocksdb/c.cc",
        "tirocks-sys/crocksdb/crocksdb/c.h",
    ]));
}

fn main() {
    let mut args = env::args();
    if args.len() != 2 {
        print_help();
        process::exit(1);
    }
    args.next();
    let subcommand = args.next().unwrap();
    match &*subcommand {
        "bindgen" => bindgen(),
        "submodule" => submodule(),
        "clang-lint" => clang_lint(),
        _ => print_help(),
    }
}
