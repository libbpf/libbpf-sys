// tests/tests.rs

#[cfg(test)]
mod tests {
    use libbpf_sys::*;

    unsafe extern "C" fn print_fn(
        _level: libbpf_print_level,
        _arg1: *const std::os::raw::c_char,
        _ap: *mut __va_list_tag,
    ) -> std::os::raw::c_int {
        0
    }

    #[test]
    fn test() {
        unsafe {
            // just tests that we can call into the library
            assert!(libbpf_set_print(Some(print_fn as _)).is_some());
        }
    }

    /// Smoke test that the [`API_HEADERS`] headers we ship allow for
    /// successful build.
    #[cfg(feature = "vendored-libbpf")]
    #[test]
    fn exported_headers_compile() -> std::io::Result<()> {
        use std::fs;
        use std::process::Command;
        use std::time::{SystemTime, UNIX_EPOCH};

        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "libbpf-sys-api-headers-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&dir)?;

        let result = (|| {
            for &(name, contents) in API_HEADERS {
                fs::write(dir.join(name), contents)?;
            }
            fs::write(
                dir.join("probe.c"),
                "#include \"bpf.h\"\n#include \"libbpf.h\"\n#include \"btf.h\"\n",
            )?;

            Command::new("cc")
                .args(["-fsyntax-only", "-I", ".", "probe.c"])
                .current_dir(&dir)
                // Use the exported set without inherited libbpf include paths.
                .env_remove("CPATH")
                .env_remove("C_INCLUDE_PATH")
                .output()
        })();
        fs::remove_dir_all(&dir)?;

        let output = result?;
        assert!(
            output.status.success(),
            "exported headers failed to compile:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        Ok(())
    }
}
