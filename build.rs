// build.rs

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let src_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let out_dir_str = out_dir.to_str().unwrap();

    if cfg!(target_os = "linux") {
        let status = Command::new("make")
            .arg("install")
            .env("BUILD_STATIC_ONLY", "y")
            .env("PREFIX", "/")
            .env("LIBDIR", "")
            .env("OBJDIR", out_dir.join("obj").to_str().unwrap())
            .env("DESTDIR", out_dir_str)
            .env("CFLAGS", "-g -O2 -Werror -Wall -fPIC")
            .current_dir(src_dir.join("libbpf/src"))
            .status()
            .unwrap();

        assert!(status.success());

        let status = Command::new("make")
            .arg("clean")
            .current_dir(src_dir.join("libbpf/src"))
            .status()
            .unwrap();
        assert!(status.success());

        cc::Build::new()
            .file("bindings.c")
            .include(src_dir.join("libbpf/include"))
            .include(src_dir.join("libbpf/include/uapi"))
            .out_dir(out_dir_str)
            .compile("bindings");

        // create libbpf rust binding in OUT_DIR
        // _create_binding("bindings"); // uncomment to create binding.rs in OUT_DIR
        
        println!("cargo:rustc-link-search=native={}", out_dir_str);
        println!("cargo:rustc-link-lib=elf");
        println!("cargo:rustc-link-lib=static=bpf");
    }
}

 
fn _create_binding(name: &str) {
    let bind = bindgen::Builder::default()
        .header(&format!("{}.h", name))
        .derive_default(true)
        .whitelist_function("bpf_.+")
        .whitelist_function("btf_.+")
        .whitelist_function("libbpf_.+")
        .whitelist_function("xsk_.+")
        .whitelist_function("xdp_.+")
        .whitelist_function("perf_buffer_.+")
        .whitelist_type("xdp_.+")
        .whitelist_type("bpf_.+")
        .whitelist_var("BPF_.+")
        .whitelist_var("BTF_.+")
        .whitelist_var("XSK_.+")
        .whitelist_var("XDP_.+")
        .default_enum_style(bindgen::EnumVariation::Consts)
        .prepend_enum_name(false)
        .layout_tests(false)        
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect(&format!("Unable to generate {}", name));

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bind
        .write_to_file(out_path.join(&format!("{}.rs", name)))
        .expect(&format!("Couldn't write {} binding!", name));    
}
