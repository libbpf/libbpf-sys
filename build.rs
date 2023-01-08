// build.rs

use std::env;
use std::fs;
use std::path;
use std::process;

#[cfg(feature = "bindgen")]
fn generate_bindings(src_dir: path::PathBuf) {
    use std::collections::HashSet;

    #[derive(Debug)]
    struct IgnoreMacros(HashSet<&'static str>);

    impl bindgen::callbacks::ParseCallbacks for IgnoreMacros {
        fn will_parse_macro(&self, name: &str) -> bindgen::callbacks::MacroParsingBehavior {
            if self.0.contains(name) {
                bindgen::callbacks::MacroParsingBehavior::Ignore
            } else {
                bindgen::callbacks::MacroParsingBehavior::Default
            }
        }
    }

    let ignored_macros = IgnoreMacros(
        vec![
            "BTF_KIND_FUNC",
            "BTF_KIND_FUNC_PROTO",
            "BTF_KIND_VAR",
            "BTF_KIND_DATASEC",
            "BTF_KIND_FLOAT",
            "BTF_KIND_DECL_TAG",
            "BTF_KIND_TYPE_TAG",
            "BTF_KIND_ENUM64",
        ]
        .into_iter()
        .collect(),
    );

    bindgen::Builder::default()
        .derive_default(true)
        .explicit_padding(true)
        .default_enum_style(bindgen::EnumVariation::Consts)
        .prepend_enum_name(false)
        .layout_tests(false)
        .generate_comments(false)
        .emit_builtins()
        .allowlist_function("bpf_.+")
        .allowlist_function("btf_.+")
        .allowlist_function("libbpf_.+")
        .allowlist_function("perf_.+")
        .allowlist_function("ring_buffer_.+")
        .allowlist_function("vdprintf")
        .allowlist_type("bpf_.+")
        .allowlist_type("btf_.+")
        .allowlist_type("xdp_.+")
        .allowlist_type("perf_.+")
        .allowlist_var("BPF_.+")
        .allowlist_var("BTF_.+")
        .allowlist_var("XDP_.+")
        .allowlist_var("PERF_.+")
        .parse_callbacks(Box::new(ignored_macros))
        .header("bindings.h")
        .clang_arg(format!("-I{}", src_dir.join("libbpf/include").display()))
        .clang_arg(format!(
            "-I{}",
            src_dir.join("libbpf/include/uapi").display()
        ))
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(&src_dir.join("src/bindings.rs"))
        .expect("Couldn't write bindings");
}

#[cfg(not(feature = "bindgen"))]
fn generate_bindings(_: path::PathBuf) {}

#[cfg(feature = "static")]
fn library_prefix() -> String {
    "static=".to_string()
}

#[cfg(not(feature = "static"))]
fn library_prefix() -> String {
    "".to_string()
}

fn main() {
    let src_dir = path::PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());

    generate_bindings(src_dir.clone());

    if cfg!(feature = "novendor") {
        println!("cargo:rustc-link-lib={}bpf\n", library_prefix());
        return;
    }

    let out_dir = path::PathBuf::from(env::var_os("OUT_DIR").unwrap());

    if let Err(_) = process::Command::new("make").status() {
        panic!("make is required to compile libbpf-sys using the vendored copy of libbpf");
    }

    if let Err(_) = process::Command::new("pkg-config").status() {
        panic!("pkg-config is required to compile libbpf-sys using the vendored copy of libbpf");
    }

    let compiler = match cc::Build::new().try_get_compiler() {
        Ok(compiler) => compiler,
        Err(_) => panic!(
            "a C compiler is required to compile libbpf-sys using the vendored copy of libbpf"
        ),
    };

    // create obj_dir if it doesn't exist
    let obj_dir = path::PathBuf::from(&out_dir.join("obj").into_os_string());
    let _ = fs::create_dir(&obj_dir);

    let status = process::Command::new("make")
        .arg("install")
        .env("BUILD_STATIC_ONLY", "y")
        .env("PREFIX", "/")
        .env("LIBDIR", "")
        .env("OBJDIR", &obj_dir)
        .env("DESTDIR", &out_dir)
        .env("CC", compiler.path())
        .env("CFLAGS", compiler.cflags_env())
        .current_dir(&src_dir.join("libbpf/src"))
        .status()
        .expect("could not execute make");

    assert!(status.success(), "make failed");

    let status = process::Command::new("make")
        .arg("clean")
        .current_dir(&src_dir.join("libbpf/src"))
        .status()
        .expect("could not execute make");

    assert!(status.success(), "make failed");

    println!(
        "cargo:rustc-link-search=native={}",
        out_dir.to_string_lossy()
    );
    println!("cargo:rustc-link-lib={}elf", library_prefix());
    println!("cargo:rustc-link-lib={}z", library_prefix());
    println!("cargo:rustc-link-lib=static=bpf");
    println!("cargo:include={}/include", out_dir.to_string_lossy());

    if let Ok(ld_path) = env::var("LD_LIBRARY_PATH") {
        for path in ld_path.split(":") {
            println!("cargo:rustc-link-search=native={}", path);
        }
    }
}
