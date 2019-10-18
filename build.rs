// build.rs

use std::env;
use std::path::PathBuf;
use std::process::Command;

use bindgen;

fn main() {
    let src_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("src");
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

    bindgen::Builder::default()
        .header(out_dir.join("include/bpf/bpf.h").to_str().unwrap())
        .header(out_dir.join("include/bpf/btf.h").to_str().unwrap())
        .header(out_dir.join("include/bpf/libbpf.h").to_str().unwrap())
        .header(out_dir.join("include/bpf/xsk.h").to_str().unwrap())
        .whitelist_function("bpf_.+")
        .whitelist_function("btf_.+")
        .whitelist_function("libbpf_.+")
        .whitelist_function("xsk_.+")
        .whitelist_function("perf_buffer_.+")
        .whitelist_var("BPF_.*")
        .whitelist_var("BTF_.*")
        .whitelist_var("XSK_.*")
        .default_enum_style(bindgen::EnumVariation::Consts)
        .prepend_enum_name(false)
        .layout_tests(false)
        .clang_arg(format!(
            "-I{}",
            src_dir.join("libbpf/include").to_str().unwrap()
        ))
        .clang_arg(format!(
            "-I{}",
            src_dir.join("libbpf/include/uapi").to_str().unwrap()
        ))
        .clang_arg(format!("--target={}", env::var("TARGET").unwrap()))
        .emit_builtins()
        .generate()
        .unwrap()
        .write_to_file(out_dir.join("bindings.rs"))
        .unwrap();

    println!(
        "cargo:rustc-link-search=native={}",
        out_dir.to_str().unwrap()
    );
    println!("cargo:rustc-link-lib=elf");
    println!("cargo:rustc-link-lib=static=bpf");
}
