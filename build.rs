// build.rs

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let src_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    Command::new("make")
        .arg("install")
        .env("BUILD_STATIC_ONLY", "y")
        .env("PREFIX", "/")
        .env("LIBDIR", "")
        .env("DESTDIR", out_dir.as_os_str())
        .current_dir(src_dir.join("libbpf/src"))
        .status()
        .unwrap();

    Command::new("make")
        .arg("clean")
        .current_dir(src_dir.join("libbpf/src"))
        .status()
        .unwrap();

    println!("cargo:rustc-link-search=native={}", out_dir.to_str().unwrap());
    println!("cargo:rustc-link-lib=elf");
    println!("cargo:rustc-link-lib=static=bpf");
}
