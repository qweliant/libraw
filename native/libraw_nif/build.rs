fn main() {
    let libraw = pkg_config::Config::new()
        .atleast_version("0.20")
        .probe("libraw")
        .expect(
            "libraw >= 0.20 not found.\n\
             Install with:\n\
             \t macOS:          brew install libraw\n\
             \t Debian/Ubuntu:  apt install libraw-dev",
        );

    cc::Build::new()
        .file("src/wrapper.c")
        .includes(&libraw.include_paths)
        .compile("libraw_wrapper");

    // pkg_config already emits cargo:rustc-link-lib=raw for dynamic linking.
    // Re-emit the search paths so the linker can find libraw at link time.
    for path in &libraw.link_paths {
        println!("cargo:rustc-link-search=native={}", path.display());
    }
}
