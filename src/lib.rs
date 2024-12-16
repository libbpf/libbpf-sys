// src/lib.rs

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[allow(clippy::all)]
mod bindings {
    #[cfg(all(feature = "bindgen", not(feature = "bindgen-source")))]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
    #[cfg(any(not(feature = "bindgen"), feature = "bindgen-source"))]
    include!("bindings.rs");
}

pub use bindings::*;

#[cfg(feature = "vendored-libbpf")]
macro_rules! header {
    ($file:literal) => {
        ($file, include_str!(concat!("../libbpf/src/", $file)))
    };
}

pub type libbpf_print_fn_t = ::std::option::Option<
    unsafe extern "C" fn(
        level: libbpf_print_level,
        arg1: *const ::std::os::raw::c_char,
        ap: *mut __va_list_tag,
    ) -> ::std::os::raw::c_int,
>;


extern "C" {

    pub fn vdprintf(
        __fd: ::std::os::raw::c_int,
        __fmt: *const ::std::os::raw::c_char,
        __args: *mut __va_list_tag,
    ) -> ::std::os::raw::c_int;

    pub fn libbpf_set_print(fn_: libbpf_print_fn_t) -> libbpf_print_fn_t;
}

/// Vendored libbpf headers
///
/// Tuple format is: (header filename, header contents)
#[cfg(feature = "vendored-libbpf")]
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
