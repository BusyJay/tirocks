use std::fs;
use std::process::{self, Command};
use std::{env, str};

fn print_help() {
    eprintln!("cargo xtask [subcommand]");
    eprintln!();
    eprintln!("Supported subcommands are:");
    eprintln!("\tbindgen\tGenerate rust-bindgen for tirocks-sys package");
    eprintln!("\tsubmodule\tInit necessary submodules for compilation");
    eprintln!("\tclang-format\tFormat cpp code in tiocks-sys package");
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

fn bindgen() {
    fs::create_dir_all("tirocks-sys/bindings").unwrap();
    exec(cargo().args(&[
        "build",
        "-p",
        "tirocks-sys",
        "--features",
        "update-bindings",
    ]));
}

fn cmd(c: &str) -> Command {
    Command::new(c)
}

fn submodule() {
    exec(cmd("git").args(&["submodule", "update", "--init"]));
}

fn clang_format() {
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
        "clang-format" => clang_format(),
        _ => print_help(),
    }
}
