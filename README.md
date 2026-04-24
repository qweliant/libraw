# LibRaw

An Elixir library for native camera RAW decoding on the BEAM, powered by a
[Rustler](https://github.com/rusterlium/rustler) NIF that wraps
[libraw](https://www.libraw.org/).

## Prerequisites

libraw must be installed before compiling or running `:libraw`:

```bash
# macOS
brew install libraw

# Debian / Ubuntu
apt install libraw-dev
```

> **LGPL compliance:** libraw is dynamically linked at runtime. Distributing
> your application requires that end users can replace the libraw shared
> library. See the [libraw license](https://www.libraw.org/license) for
> details.

## Installation

Add `:libraw` to your `mix.exs` dependencies:

```elixir
def deps do
  [
    {:libraw, "~> 0.1"}
  ]
end
```

## Usage

### Decode a RAW file

```elixir
{:ok, image} = LibRaw.decode("/path/to/photo.CR3")
# => %{pixels: <<...>>, width: 6000, height: 4000, colors: 3, bps: 8}

# 16-bit output with linear gamma
{:ok, image16} = LibRaw.decode("/path/to/photo.CR3",
  output_bps: 16,
  gamma: :linear
)

# Custom gamma curve
{:ok, image_custom} = LibRaw.decode("/path/to/photo.NEF",
  gamma: {2.4, 12.92},
  use_camera_wb: true,
  no_auto_bright: true
)
```

#### Options

| Option           | Type                            | Default  | Description                              |
|------------------|---------------------------------|----------|------------------------------------------|
| `use_camera_wb`  | `boolean`                       | `true`   | Use white balance stored in the file     |
| `no_auto_bright` | `boolean`                       | `false`  | Disable automatic brightening            |
| `output_bps`     | `8 \| 16`                      | `8`      | Bits per sample in the output            |
| `gamma`          | `:srgb \| :linear \| {g0, g1}` | `:srgb`  | Gamma curve                              |

#### Return value

```elixir
%{
  pixels: binary(),         # raw pixel bytes (interleaved RGB or RGBA)
  width:  non_neg_integer(),
  height: non_neg_integer(),
  colors: non_neg_integer(), # number of color channels (typically 3)
  bps:    non_neg_integer()  # bits per sample of the output
}
```

### Read metadata without decoding

```elixir
{:ok, meta} = LibRaw.metadata("/path/to/photo.CR3")
# => %{
#      camera_make:  "Canon",
#      camera_model: "EOS R5",
#      captured_at:  ~U[2023-06-15 10:32:11Z],  # DateTime UTC, or nil
#      iso:          400.0,
#      shutter:      0.002,                        # seconds
#      aperture:     2.8,                          # f-number
#      orientation:  0                             # EXIF flip code
#    }
```

## Architecture

```
lib/
  lib_raw.ex          Public API: decode/2, metadata/1, gamma resolution, timestamp parsing
  lib_raw/
    nif.ex            use Rustler + NIF stubs (nif_not_loaded fallbacks)
native/
  libraw_nif/
    Cargo.toml        deps: rustler = "0.33"; build-deps: cc = "1", pkg-config = "0.3"
    build.rs          pkg_config::probe("libraw") for dynamic linking; cc::Build compiles wrapper.c
    src/
      lib.rs          rustler::init! and two #[rustler::nif(schedule = "DirtyCpu")] functions
      wrapper.c       thin C shim — C compiler resolves all struct field offsets
      raw.rs          safe Rust RAII wrappers around libraw_data_t / libraw_processed_image_t
      error.rs        LibRawError enum + helpers
```

### Why a C shim?

Direct bindgen / libraw-sys approaches embed struct field offsets at compile
time, which can break across libraw versions 0.20, 0.21, and 0.22 as the
struct layout evolves.  `wrapper.c` is compiled with the same headers as the
installed libraw, so the C compiler always uses the correct offsets.  Rust
calls only opaque C functions and never touches libraw structs directly.

### Dirty CPU Schedulers

Both NIFs (`decode_nif` and `metadata_nif`) are annotated with
`schedule = "DirtyCpu"`.  Decoding a RAW file typically takes 100–500 ms,
which is far beyond the 1 ms NIF time budget for normal schedulers.  Running
on dirty schedulers prevents blocking the BEAM scheduler threads.

## License

MIT — see [LICENSE](LICENSE).
