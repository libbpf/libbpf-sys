# libbpf-sys [![Travis CI build status badge](https://travis-ci.org/alexforster/libbpf-sys.svg?branch=master)](https://travis-ci.org/alexforster/libbpf-sys) [![crates.io version number badge](https://img.shields.io/crates/v/libbpf-sys.svg)](https://crates.io/crates/libbpf-sys)

**Rust bindings to _libbpf_ from the Linux kernel**

**Maintainer:** Alex Forster \<alex@alexforster.com\><br/>
**License:** `BSD-2-Clause`

_libbpf-sys_ is the packaged result of using _bindgen_ to automatically generate Rust FFI bindings to _libbpf_ from the Linux kernel.

**Warning:** this crate does not provide a high-level or "safe" API wrapper around _libbpf_. If you are looking for an easier way to use _libbpf_, check out these other crates that implement higher-level APIs using _libbpf-sys_...

 * **afxdp:** a Rust interface for AF_XDP – [GitHub](https://github.com/aterlo/afxdp-rs) | [Crates.io](https://crates.io/crates/afxdp)
 * **libbpf-cargo:** Cargo plugin to build bpf programs – [GitHub](https://github.com/libbpf/libbpf-rs) | [Crates.io](https://crates.io/crates/libbpf-cargo)
 * **libbpf-rs:** a safe, idiomatic, and opinionated wrapper around libbpf-sys – [GitHub](https://github.com/libbpf/libbpf-rs) | [Crates.io](https://crates.io/crates/libbpf-rs)
 * **rebpf:** write and load eBPF programs in Rust – [GitHub](https://github.com/uccidibuti/rebpf) | [Crates.io](https://crates.io/crates/rebpf)
 * **xsk-rs:** a Rust interface for Linux AF_XDP sockets – [Github](https://github.com/DouglasGray/xsk-rs) | [Crates.io](https://crates.io/crates/xsk-rs)

The community is encouraged to build higher-level crates using _libbpf-sys_. Please let me know if you do!

### Building

As part of the `cargo build` process, an included copy of _libbpf_ is compiled and statically linked into the resulting binary. This means that, in order to build a project that depends on this crate, your system must provide a working C compiler toolchain (GCC and Clang should both work). Additionally, your system must provide development headers for _zlib_ and _libelf_, and they must be discoverable via _pkgconfig_.

### Distribution

When you add this crate as a dependency to your project, your resulting binaries will dynamically link with `libz` and `libelf`. This means that the systems where you run your binaries must have these libraries installed.

### Versioning

Because the API of this crate is automatically generated from _libbpf_ sources, it uses a versioning scheme based on the version of _libbpf_ that it provides.

The "Major.Minor.Patch" semver numbers correspond exactly to the _libbpf_ version that each release provides. For example, the `0.0.8-1` release of this crate provides the API for the _libbpf v0.0.8_ release.

In order to allow for human error, all releases have a "pre-release" version number appended to them. For example, both the `0.0.6-1` and `0.0.6-2` releases of this crate contain bindings to _libbpf v0.0.6_, but the later release contains bugfixes and/or enhancements to the crate itself.

**Note:** because this crate uses a "pre-release" version number, it is impossible to use wildcard-style version syntax in your `Cargo.toml` file. You must specify the exact version of `libbpf-sys`, and you must not assume that any other version will be API or ABI compatible.

### Licensing and Dependencies

This crate is released under the BSD 2-Clause license, and is careful to avoid infecting users with viral licenses.

It currently depends on the following third-party libraries:

|            | Website                                                       | License                                  | Linkage |
|------------|---------------------------------------------------------------|------------------------------------------|---------|
| **libbpf** | [github.com/libbpf/libbpf](https://github.com/libbpf/libbpf/) | `LGPL-2.1-only OR BSD-2-Clause`          | Static  |
| **libelf** | [sourceware.org/elfutils](https://sourceware.org/elfutils/)   | `LGPL-2.1-or-later OR LGPL-3.0-or-later` | Dynamic |
| **zlib**   | [zlib.net](https://www.zlib.net/)                             | `Zlib`                                   | Dynamic |
