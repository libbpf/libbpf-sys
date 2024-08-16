// build.rs

use std::env;
use std::ffi;
use std::fs;
use std::fs::read_dir;
use std::path;
use std::path::Path;
use std::process;

use nix::fcntl;

fn emit_rerun_directives_for_contents(dir: &Path) {
    for result in read_dir(dir).unwrap() {
        let file = result.unwrap();
        println!("cargo:rerun-if-changed={}", file.path().display());
    }
}

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

    #[cfg(feature = "bindgen-source")]
    let out_dir = &src_dir.join("src");
    #[cfg(not(feature = "bindgen-source"))]
    let out_dir =
        &path::PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR should always be set"));

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
        .allowlist_function("user_ring_buffer_.+")
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
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldn't write bindings");
}

#[cfg(not(feature = "bindgen"))]
fn generate_bindings(_: path::PathBuf) {}

fn pkg_check(pkg: &str) {
    if process::Command::new(pkg)
        .stdout(process::Stdio::null())
        .stderr(process::Stdio::null())
        .status()
        .is_err()
    {
        panic!(
            "{} is required to compile libbpf-sys with the selected set of features",
            pkg
        );
    }
}

fn main() {
    let src_dir = path::PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());

    generate_bindings(src_dir.clone());

    let vendored_libbpf = cfg!(feature = "vendored-libbpf");
    let vendored_libelf = cfg!(feature = "vendored-libelf");
    let vendored_zlib = cfg!(feature = "vendored-zlib");
    println!("Using feature vendored-libbpf={}", vendored_libbpf);
    println!("Using feature vendored-libelf={}", vendored_libelf);
    println!("Using feature vendored-zlib={}", vendored_zlib);

    let static_libbpf = cfg!(feature = "static-libbpf");
    let static_libelf = cfg!(feature = "static-libelf");
    let static_zlib = cfg!(feature = "static-zlib");
    println!("Using feature static-libbpf={}", static_libbpf);
    println!("Using feature static-libelf={}", static_libelf);
    println!("Using feature static-zlib={}", static_zlib);

    if cfg!(feature = "novendor") {
        println!("cargo:warning=the `novendor` feature of `libbpf-sys` is deprecated; build without features instead");
        println!(
            "cargo:rustc-link-lib={}bpf",
            if static_libbpf { "static=" } else { "" }
        );
        return;
    }

    let out_dir = path::PathBuf::from(env::var_os("OUT_DIR").unwrap());

    // check for all necessary compilation tools
    if vendored_libelf {
        pkg_check("autoreconf");
        pkg_check("autopoint");
        pkg_check("flex");
        pkg_check("bison");
        pkg_check("gawk");
    }

    let (compiler, mut cflags) = if vendored_libbpf || vendored_libelf || vendored_zlib {
        pkg_check("make");
        pkg_check("pkg-config");

        let compiler = cc::Build::new().try_get_compiler().expect(
            "a C compiler is required to compile libbpf-sys using the vendored copy of libbpf",
        );
        let mut cflags = compiler.cflags_env();
        println!("cargo:rerun-if-env-changed=LIBBPF_SYS_EXTRA_CFLAGS");
        if let Some(extra_cflags) = env::var_os("LIBBPF_SYS_EXTRA_CFLAGS") {
            cflags.push(" ");
            cflags.push(extra_cflags);
        }
        (Some(compiler), cflags)
    } else {
        (None, ffi::OsString::new())
    };

    if vendored_zlib {
        make_zlib(compiler.as_ref().unwrap(), &src_dir, &out_dir);
        cflags.push(&format!(" -I{}/zlib/", src_dir.display()));
    }

    if vendored_libelf {
        make_elfutils(compiler.as_ref().unwrap(), &src_dir, &out_dir);
        cflags.push(&format!(" -I{}/elfutils/libelf/", src_dir.display()));
    }

    if vendored_libbpf {
        make_libbpf(compiler.as_ref().unwrap(), &cflags, &src_dir, &out_dir);
    }

    println!(
        "cargo:rustc-link-search=native={}",
        out_dir.to_string_lossy()
    );
    println!(
        "cargo:rustc-link-lib={}elf",
        if static_libelf { "static=" } else { "" }
    );
    println!(
        "cargo:rustc-link-lib={}z",
        if static_zlib { "static=" } else { "" }
    );
    println!(
        "cargo:rustc-link-lib={}bpf",
        if static_libbpf { "static=" } else { "" }
    );
    println!("cargo:include={}/include", out_dir.to_string_lossy());

    println!("cargo:rerun-if-env-changed=LD_LIBRARY_PATH");
    if let Ok(ld_path) = env::var("LD_LIBRARY_PATH") {
        for path in ld_path.split(':') {
            if !path.is_empty() {
                println!("cargo:rustc-link-search=native={}", path);
            }
        }
    }
}

fn make_zlib(compiler: &cc::Tool, src_dir: &path::Path, out_dir: &path::Path) {
    let src_dir = src_dir.join("zlib");
    // lock README such that if two crates are trying to compile
    // this at the same time (eg libbpf-rs libbpf-cargo)
    // they wont trample each other
    let file = std::fs::File::open(src_dir.join("README")).unwrap();
    let _lock = fcntl::Flock::lock(file, fcntl::FlockArg::LockExclusive).unwrap();

    let status = process::Command::new("./configure")
        .arg("--static")
        .arg("--prefix")
        .arg(".")
        .arg("--libdir")
        .arg(out_dir)
        .env("CC", compiler.path())
        .env("CFLAGS", compiler.cflags_env())
        .current_dir(&src_dir)
        .status()
        .expect("could not execute make");

    assert!(status.success(), "make failed");

    let status = process::Command::new("make")
        .arg("install")
        .arg("-j")
        .arg(&format!("{}", num_cpus()))
        .current_dir(&src_dir)
        .status()
        .expect("could not execute make");

    assert!(status.success(), "make failed");

    let status = process::Command::new("make")
        .arg("distclean")
        .current_dir(&src_dir)
        .status()
        .expect("could not execute make");

    assert!(status.success(), "make failed");
    emit_rerun_directives_for_contents(&src_dir);
}

fn make_elfutils(compiler: &cc::Tool, src_dir: &path::Path, out_dir: &path::Path) {
    // lock README such that if two crates are trying to compile
    // this at the same time (eg libbpf-rs libbpf-cargo)
    // they wont trample each other
    let file = std::fs::File::open(src_dir.join("elfutils/README")).unwrap();
    let _lock = fcntl::Flock::lock(file, fcntl::FlockArg::LockExclusive).unwrap();

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
        .arg(&format!("{}", num_cpus()))
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
    emit_rerun_directives_for_contents(&src_dir.join("elfutils").join("src"));
}

fn make_libbpf(
    compiler: &cc::Tool,
    cflags: &ffi::OsStr,
    src_dir: &path::Path,
    out_dir: &path::Path,
) {
    let src_dir = src_dir.join("libbpf/src");
    // create obj_dir if it doesn't exist
    let obj_dir = path::PathBuf::from(&out_dir.join("obj").into_os_string());
    let _ = fs::create_dir(&obj_dir);

    let status = process::Command::new("make")
        .arg("install")
        .arg("-j")
        .arg(&format!("{}", num_cpus()))
        .env("BUILD_STATIC_ONLY", "y")
        .env("PREFIX", "/")
        .env("LIBDIR", "")
        .env("OBJDIR", &obj_dir)
        .env("DESTDIR", out_dir)
        .env("CC", compiler.path())
        .env("CFLAGS", cflags)
        .current_dir(&src_dir)
        .status()
        .expect("could not execute make");

    assert!(status.success(), "make failed");

    let status = process::Command::new("make")
        .arg("clean")
        .current_dir(&src_dir)
        .status()
        .expect("could not execute make");

    assert!(status.success(), "make failed");
    emit_rerun_directives_for_contents(&src_dir);
}

fn num_cpus() -> usize {
    std::thread::available_parallelism().map_or(1, |count| count.get())
}
