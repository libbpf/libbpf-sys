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
        .size_t_is_usize(false)
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

fn pkg_check(pkg: &str) {
    if let Err(_) = process::Command::new(pkg).status() {
        panic!("{} is required to compile libbpf-sys using the vendored copy of libbpf", pkg);
    }
}

fn main() {
    let src_dir = path::PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());

    generate_bindings(src_dir.clone());

    if cfg!(not(feature = "static")) {
        println!("cargo:rustc-link-lib={}bpf\n", library_prefix());
        return;
    }

    let out_dir = path::PathBuf::from(env::var_os("OUT_DIR").unwrap());

    // check for all necessary compilation tools
    pkg_check("make");
    pkg_check("pkg-config");
    pkg_check("autoreconf");
    pkg_check("autopoint");
    pkg_check("flex");
    pkg_check("bison");
    pkg_check("gawk");

    let compiler = match cc::Build::new().try_get_compiler() {
        Ok(compiler) => compiler,
        Err(_) => panic!(
            "a C compiler is required to compile libbpf-sys using the vendored copy of libbpf"
        ),
    };

    // create obj_dir if it doesn't exist
    let obj_dir = path::PathBuf::from(&out_dir.join("obj").into_os_string());
    let _ = fs::create_dir(&obj_dir);

    // compile static zlib and static libelf
    make_zlib(&compiler, &src_dir, &out_dir);
    make_elfutils(&compiler, &src_dir, &out_dir);

    let cflags = if cfg!(feature = "vendored") {
        // make sure that the headerfiles from libelf and zlib
        // for libbpf come from the vendorized version
        
        let mut cflags = compiler.cflags_env();
        cflags.push(&format!(" -I{}/elfutils/libelf/", src_dir.display()));
        cflags.push(&format!(" -I{}/zlib/", src_dir.display()));
        cflags
    } else {
        compiler.cflags_env()
    };


    let status = process::Command::new("make")
        .arg("install")
        .arg("-j")
        .arg(&format!("{}", num_cpus::get()))
        .env("BUILD_STATIC_ONLY", "y")
        .env("PREFIX", "/")
        .env("LIBDIR", "")
        .env("OBJDIR", &obj_dir)
        .env("DESTDIR", &out_dir)
        .env("CC", compiler.path())
        .env("CFLAGS", cflags)
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
            if !path.is_empty() {
                println!("cargo:rustc-link-search=native={}", path);
            }
        }
    }
}

fn make_zlib(compiler: &cc::Tool, src_dir: &path::PathBuf, out_dir: &path::PathBuf) {
    use nix::fcntl;
    use std::os::fd::AsRawFd;

    // lock README such that if two crates are trying to compile
    // this at the same time (eg libbpf-rs libbpf-cargo)
    // they wont trample each other
    let file = std::fs::File::open(&src_dir.join("zlib/README")).unwrap();
    let fd = file.as_raw_fd();
    fcntl::flock(fd, fcntl::FlockArg::LockExclusive).unwrap();

    let status = process::Command::new("./configure")
        .arg("--static")
        .arg("--prefix")
        .arg(".")
        .arg("--libdir")
        .arg(out_dir)
        .env("CC", compiler.path())
        .env("CFLAGS", compiler.cflags_env())
        .current_dir(&src_dir.join("zlib"))
        .status()
        .expect("could not execute make");

    assert!(status.success(), "make failed");

    let status = process::Command::new("make")
        .arg("install")
        .arg("-j")
        .arg(&format!("{}", num_cpus::get()))
        .current_dir(&src_dir.join("zlib"))
        .status()
        .expect("could not execute make");

    assert!(status.success(), "make failed");

    let status = process::Command::new("make")
        .arg("distclean")
        .current_dir(&src_dir.join("zlib"))
        .status()
        .expect("could not execute make");

    assert!(status.success(), "make failed");
}

fn make_elfutils(compiler: &cc::Tool, src_dir: &path::PathBuf, out_dir: &path::PathBuf) {
    use nix::fcntl;
    use std::os::fd::AsRawFd;

    // lock README such that if two crates are trying to compile
    // this at the same time (eg libbpf-rs libbpf-cargo)
    // they wont trample each other
    let file = std::fs::File::open(&src_dir.join("elfutils/README")).unwrap();
    let fd = file.as_raw_fd();
    fcntl::flock(fd, fcntl::FlockArg::LockExclusive).unwrap();



    let flags = compiler
        .cflags_env()
        .into_string()
        .expect("failed to get cflags");
    let mut cflags: String = flags
        .split_whitespace()
        .filter_map(|arg| {
            if arg != "-static" {
                // compilation fails with -static flag
                Some(format!(" {arg}"))
            } else {
                None
            }
        })
        .collect();



    #[cfg(target_arch = "aarch64")]
    cflags.push_str(" -Wno-error=stringop-overflow"); 
    cflags.push_str(&format!(" -I{}/zlib/", src_dir.display()));

    let status = process::Command::new("sed")
        .arg("-i")
        .arg(r#"s/po doc tests/po doc/g"#)
        .arg("Makefile.am")
        .current_dir(&src_dir.join("elfutils"))
        .status()
        .expect("could not strip tests");

    assert!(status.success(), "make failed");

    let status = process::Command::new("autoreconf")
        .arg("--install")
        .arg("--force")
        .current_dir(&src_dir.join("elfutils"))
        .status()
        .expect("could not execute make");

    assert!(status.success(), "make failed");

    // location of libz.a
    let out_lib = format!("-L{}", out_dir.display());
    let status = process::Command::new("./configure")
        .arg("--enable-maintainer-mode")
        .arg("--disable-debuginfod")
        .arg("--disable-libdebuginfod")
        .arg("--without-zstd")
        .arg("--prefix")
        .arg(&src_dir.join("elfutils/prefix_dir"))
        .arg("--libdir")
        .arg(out_dir)
        .env("CC", compiler.path())
        .env("CXX", compiler.path())
        .env("CFLAGS", &cflags)
        .env("CXXFLAGS", &cflags)
        .env("LDFLAGS", &out_lib)
        .current_dir(&src_dir.join("elfutils"))
        .status()
        .expect("could not execute make");

    assert!(status.success(), "make failed");

    let status = process::Command::new("make")
        .arg("install")
        .arg("-j")
        .arg(&format!("{}", num_cpus::get()))
        .arg("BUILD_STATIC_ONLY=y")
        .current_dir(&src_dir.join("elfutils"))
        .status()
        .expect("could not execute make");

    assert!(status.success(), "make failed");

    let status = process::Command::new("make")
        .arg("distclean")
        .current_dir(&src_dir.join("elfutils"))
        .status()
        .expect("could not execute make");

    assert!(status.success(), "make failed");
}
