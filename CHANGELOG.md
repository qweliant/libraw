# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2026-05-16

### Added

- Precompiled NIF binaries via `rustler_precompiled`. Supported targets:
  `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`,
  `aarch64-unknown-linux-gnu`. Users on these platforms no longer need a Rust
  toolchain installed.
- `.github/workflows/release.yml`: GitHub Actions workflow that builds and
  uploads NIF artifacts on every `v*` tag push. Runs on native macOS and Linux
  ARM/x86 runners; `brew install libraw` / `apt install libraw-dev` installs
  the required shared library into the build environment.
- `LIBRAW_BUILD=1` env-var opt-in for forcing a local source build (useful for
  development or unsupported targets).

### Changed

- `lib/lib_raw/nif.ex` now uses `RustlerPrecompiled` instead of plain `Rustler`.
  The libraw system library is still required at runtime on every platform
  (dynamic linking, LGPL).
- Installation docs updated: version constraint bumped to `~> 0.3`, new
  "Supported Platforms" table, Rust prerequisite scoped to source-build cases.

### Notes

- MUSL / Alpine Linux is intentionally excluded: libraw is not reliable on
  musl libc.

## [0.2.0] - 2026-05-02

### Fixed

- `:srgb` gamma now resolves to `{2.4, 12.92}` (the dcraw / libraw sRGB
  convention). Previously `{2.222, 12.92}`, which produced incorrect pixel
  values for every caller using the default.

### Changed

- **Breaking (source builds):** Bumped `rustler` to `~> 0.37` on both the
  Elixir and Rust sides (was `~> 0.33`). Consumers compiling the NIF from
  source need a compatible Rust toolchain; no API changes for Elixir callers.
- `decode/2` no longer makes an intermediate `Vec<u8>` copy of the libraw
  pixel buffer. The data is copied once, directly into the `OwnedBinary`
  returned to the BEAM. Saves ~72 MB of allocation+copy per 24-megapixel
  8-bit decode.

### Removed

- Unused `LibRawError::UnexpectedImageType` variant and dead helpers in
  `error.rs` (`result_to_term/2`, `From<LibRawError> for rustler::Error`).

### Internal

- Added `mix format` config and ran the formatter.
- Added an integration test (`mix test.smoke`) that exercises a real RAW
  file end-to-end. Excluded from `mix test` by default; drop any RAW at
  `test/fixtures/sample.raw` to run it.

## [0.1.0] - 2026-04-26

### Added

- Initial release.
- `LibRaw.decode/2` — full libraw pipeline (open → unpack → dcraw_process →
  in-memory bitmap), with options for white balance, auto-bright, output
  bits-per-sample, and gamma curve.
- `LibRaw.metadata/1` — EXIF metadata (make, model, captured_at, ISO,
  shutter, aperture, orientation) without the full decode pipeline.
- Both NIFs scheduled on Dirty CPU schedulers.
- Thin `wrapper.c` shim avoids bindgen / struct-offset breakage across
  libraw 0.20 / 0.21 / 0.22.

[0.3.0]: https://github.com/qweliant/libraw/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/qweliant/libraw/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/qweliant/libraw/releases/tag/v0.1.0
