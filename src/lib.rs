// src/lib.rs

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!("bindings.rs");

macro_rules! header {
    ($file:literal) => {
        ($file, include_str!(concat!("../libbpf/src/", $file)))
    };
}

/// Vendored libbpf headers
///
/// Tuple format is: (header filename, header contents)
#[cfg(not(feature = "novendor"))]
pub const API_HEADERS: [(&str, &str); 10] = [
    header!("bpf.h"),
    header!("libbpf.h"),
    header!("btf.h"),
    header!("bpf_helpers.h"),
    header!("bpf_helper_defs.h"),
    header!("bpf_tracing.h"),
    header!("bpf_endian.h"),
    header!("bpf_core_read.h"),
    header!("libbpf_common.h"),
    header!("usdt.bpf.h"),
];
