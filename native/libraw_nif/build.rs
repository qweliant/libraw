fn main() {
    // We disable pkg-config's auto-emitted cargo metadata so we can rewrite
    // the link-libs ourselves: homebrew's libraw.pc declares `-lstdc++`, but
    // macOS toolchains link the C++ stdlib as `c++`, not `stdc++`.
    let libraw = pkg_config::Config::new()
        .atleast_version("0.20")
        .cargo_metadata(false)
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

    for path in &libraw.link_paths {
        println!("cargo:rustc-link-search=native={}", path.display());
    }
    for path in &libraw.framework_paths {
        println!("cargo:rustc-link-search=framework={}", path.display());
    }

    let is_macos = std::env::var("CARGO_CFG_TARGET_OS")
        .map(|s| s == "macos")
        .unwrap_or(false);

    for lib in &libraw.libs {
        let name = if is_macos && lib == "stdc++" { "c++" } else { lib.as_str() };
        println!("cargo:rustc-link-lib={}", name);
    }
    for fw in &libraw.frameworks {
        println!("cargo:rustc-link-lib=framework={}", fw);
    }

    // BEAM resolves the enif_* symbols at NIF load time, not link time.
    if is_macos {
        println!("cargo:rustc-cdylib-link-arg=-undefined");
        println!("cargo:rustc-cdylib-link-arg=dynamic_lookup");
    }
}
