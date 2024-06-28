// build.rs

use std::env;
use std::ffi;
use std::path;
use std::process;
use std::process::ExitStatus;

use nix::fcntl;

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
        .allowlist_type("bpf_.+")
        .allowlist_type("btf_.+")
        .allowlist_type("xdp_.+")
        .allowlist_type("perf_.+")
        .allowlist_var("BPF_.+")
        .allowlist_var("BTF_.+")
        .allowlist_var("XDP_.+")
        .allowlist_var("PERF_.+")
        .allowlist_type("__va_list_tag")
        .blocklist_type("vdprintf")
        .blocklist_type("libbpf_print_fn_t")
        .blocklist_function("libbpf_set_print")
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

    let android = build_android();

    let vendored_libbpf = cfg!(feature = "vendored-libbpf") || android;
    let vendored_libelf = cfg!(feature = "vendored-libelf") || android;
    let vendored_zlib = cfg!(feature = "vendored-zlib") || android;

    println!("Using feature vendored-libbpf={}", vendored_libbpf);
    println!("Using feature vendored-libelf={}", vendored_libelf);
    println!("Using feature vendored-zlib={}", vendored_zlib);

    let static_libbpf = cfg!(feature = "static-libbpf") || android;
    let static_libelf = cfg!(feature = "static-libelf") || android;
    let static_zlib = cfg!(feature = "static-zlib") || android;

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
        pkg_check("aclocal");
    }

    let (compiler, mut cflags) = if vendored_libbpf || vendored_libelf || vendored_zlib {
        // pkg_check("make");
        pkg_check("pkg-config");

        let compiler = cc::Build::new().try_get_compiler().expect(
            "a C compiler is required to compile libbpf-sys using the vendored copy of libbpf",
        );
        let cflags = compiler.cflags_env();
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

fn build_android() -> bool {
    if cfg!(feature = "android") {
        true
    } else {
        env::var("CARGO_CFG_TARGET_OS")
            .expect("CARGO_CFG_TARGET_OS not set")
            .eq("android")
    }
}

fn subproc(prog: &str, workdir: &str, args: &[&str]) -> ExitStatus {
    process::Command::new(prog)
        .current_dir(workdir)
        .args(args)
        .status()
        .expect(&format!("could not execute `{prog}`"))
}

fn configure<P>(project_dir: P, args: &[&str])
where
    P: AsRef<str>,
{
    let project = project_dir.as_ref();

    let prog = "./configure";

    let _ = subproc("chmod", project, &["+x", prog]);

    let status = subproc(prog, project, args);

    assert!(
        status.success(),
        "configure({}) failed: {}",
        project,
        status
    );
}

fn autoconf<P>(project_dir: P)
where
    P: AsRef<str>,
{
    let project = project_dir.as_ref();

    let status = subproc("autoreconf", project, &["--install", "--force"]);

    assert!(
        status.success(),
        "autoreconfig({}) failed: {}",
        project,
        status
    );
}

fn make_zlib(compiler: &cc::Tool, src_dir: &path::Path, _: &path::Path) {
    // lock README such that if two crates are trying to compile
    // this at the same time (eg libbpf-rs libbpf-cargo)
    // they wont trample each other
    let file = std::fs::File::open(src_dir.join("zlib/README")).unwrap();
    let _lock = fcntl::Flock::lock(file, fcntl::FlockArg::LockExclusive).unwrap();

    let zlib_sources = [
        "adler32.c",
        "compress.c",
        "crc32.c",
        "deflate.c",
        "gzclose.c",
        "gzlib.c",
        "gzread.c",
        "gzwrite.c",
        "infback.c",
        "inffast.c",
        "inflate.c",
        "inftrees.c",
        "trees.c",
        "uncompr.c",
        "zutil.c",
    ];

    let cflags = [
        // We do support hidden visibility, so turn that on.
        "-DHAVE_HIDDEN",
        // We do support const, so turn that on.
        "-DZLIB_CONST",
        // Enable -O3 as per chromium.
        "-O3",
        // "-Wall",
        // "-Werror",
        // "-Wno-deprecated-non-prototype",
        // "-Wno-unused",
        // "-Wno-unused-parameter",
    ];

    let project_dir = src_dir.join("zlib");
    let project_dir = project_dir.to_str().unwrap();

    configure(project_dir, &[]);

    let mut builder = cc::Build::new();

    builder.include(project_dir).files({
        zlib_sources
            .iter()
            .map(|source| format!("{project_dir}/{source}"))
    });

    if build_android() {
        for flag in cflags {
            builder.flag(flag);
        }
    } else {
        for flag in compiler.args() {
            builder.flag(flag);
        }
    }

    builder.flag_if_supported("-w").warnings(false).compile("z")
}

fn make_elfutils(compiler: &cc::Tool, src_dir: &path::Path, _: &path::Path) {
    // lock README such that if two crates are trying to compile
    // this at the same time (eg libbpf-rs libbpf-cargo)
    // they wont trample each other
    let file = std::fs::File::open(src_dir.join("elfutils/README")).unwrap();
    let _lock = fcntl::Flock::lock(file, fcntl::FlockArg::LockExclusive).unwrap();

    let project_dir = src_dir.join("elfutils");
    let project = project_dir.to_str().unwrap();

    autoconf(project);

    configure(
        project,
        &[
            "--enable-maintainer-mode",
            "--disable-debuginfod",
            "--disable-libdebuginfod",
            "--without-lzma",
            "--without-bzlib",
            "--without-zstd",
        ],
    );

    let mut builder = cc::Build::new();

    builder
        .include(project)
        .include(src_dir.join("zlib"))
        .include(format!("{project}/lib"))
        .include(format!("{project}/include"))
        .include(format!("{project}/libelf"));

    if build_android() {
        builder
            .flag("-DHAVE_CONFIG_H")
            .flag("-D_GNU_SOURCE")
            .flag("-DNAMES=1000")
            .flag("-std=gnu99")
            .flag("-D_FILE_OFFSET_BITS=64")
            .flag("-includeAndroidFixup.h")
            .include(src_dir.join("android"));
    } else {
        #[cfg(target_arch = "aarch64")]
        builder.flag("-Wno-error=stringop-overflow");

        builder.compiler(compiler.path());

        for flag in compiler.args() {
            if flag.ne("-static") {
                builder.flag(flag);
            }
        }
    }

    for entry in std::fs::read_dir(project_dir.join("libelf")).expect("Failed to `read_dir`") {
        let entry = entry.expect("Failed to `read_dir`");
        if entry.file_type().unwrap().is_file()
            && entry.file_name().to_str().unwrap().ends_with(".c")
        {
            builder.file(entry.path());
        }
    }

    builder
        .flag_if_supported("-w")
        .warnings(false)
        .compile("elf")
}

fn make_libbpf(compiler: &cc::Tool, _: &ffi::OsStr, src_dir: &path::Path, _: &path::Path) {
    let project_dir = src_dir.join("libbpf");

    let project = project_dir.to_str().unwrap();

    let mut builder = cc::Build::new();

    builder
        .include(src_dir)
        .include(src_dir.join("zlib"))
        .include(src_dir.join("elfutils").join("libelf"))
        .include(format!("{project}/src"))
        .include(format!("{project}/include"))
        .include(format!("{project}/include/uapi"));

    if build_android() {
        let cflags = ["-DCOMPAT_NEED_REALLOCARRAY"];

        builder.flag("-includeandroid/android.h");

        for flag in cflags {
            builder.flag(flag);
        }
    } else {
        builder.compiler(compiler.path());

        for flag in compiler.args() {
            builder.flag(flag);
        }
    }

    for entry in std::fs::read_dir(project_dir.join("src")).expect("Failed to `read_dir`") {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_file()
            && entry.file_name().to_str().unwrap().ends_with(".c")
        {
            builder.file(entry.path());
        }
    }

    builder
        .flag_if_supported("-w")
        .warnings(false)
        .compile("bpf");
}
