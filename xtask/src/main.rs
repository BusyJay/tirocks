use std::fs;
use std::process::{self, Command};
use std::{env, str};

fn print_help() {
    eprintln!("cargo xtask [subcommand]");
    eprintln!();
    eprintln!("Supported subcommands are:");
    eprintln!("\tbindgen\tGenerate rust-bindgen for tirocks-sys package");
    eprintln!("\tsubmodule\tInit necessary submodules for compilation");
    eprintln!("\t\t\tYou can use environment variables {{TITAN,ROCKSDB}}_{{REPO,BRANCH}} to control what remote ref to use.");
    eprintln!("\tformat\tFormat all code, including C/CPP");
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
    match c.status() {
        Ok(e) => {
            if !e.success() {
                eprintln!("failed to execute {:?}", c);
                process::exit(e.code().unwrap_or(-1));
            }
        }
        Err(e) => {
            eprintln!("failed to execute {:?}: {}", c, e);
            process::exit(-1);
        }
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
    for submodule in &["rocksdb", "titan"] {
        let upper = submodule.to_ascii_uppercase();
        let mut remote_changed = false;
        let path = format!("tirocks-sys/{}", submodule);
        if let Ok(repo) = env::var(&format!("{}_REPO", upper)) {
            exec(cmd("git").args(&[
                "config",
                "--file=.gitmodules",
                &format!("submodule.\"{}\".url", path),
                &format!("https://github.com/{}/{}.git", repo, submodule),
            ]));
            remote_changed = true;
        }
        if let Ok(branch) = env::var(&format!("{}_BRANCH", upper)) {
            exec(cmd("git").args(&[
                "config",
                "--file=.gitmodules",
                &format!("submodule.\"{}\".branch", path),
                &branch,
            ]));
            remote_changed = true;
        }
        if remote_changed {
            exec(cmd("git").args(&["submodule", "sync", &path]));
            exec(cmd("git").args(&["submodule", "update", "--init", "--remote", &path]));
        } else {
            exec(cmd("git").args(&["submodule", "update", "--init", &path]));
        }
    }
}

fn format() {
    exec(cmd("clang-format").args(&[
        "-i",
        "tirocks-sys/crocksdb/c.cc",
        "tirocks-sys/crocksdb/crocksdb/c.h",
    ]));
    exec(cargo().args(&["fmt", "--all"]));
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
        "format" => format(),
        _ => print_help(),
    }
}
