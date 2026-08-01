# `LibRaw`
[🔗](https://github.com/qweliant/libraw/blob/main/lib/lib_raw.ex#L1)

Native camera RAW decoding for the BEAM via a Rustler NIF wrapping libraw.

## Prerequisites

libraw must be installed on every machine that runs this library, including
machines using the precompiled NIF — it is linked dynamically at runtime:

    # macOS
    brew install libraw

    # Debian / Ubuntu
    apt install libraw-dev

A Rust toolchain is **not** required on supported targets (macOS and Linux
on x86_64 and ARM64), where a precompiled NIF is downloaded instead.

The precompiled NIFs link a specific libraw ABI — `libraw.so.23` on Linux
(libraw 0.21.x, Ubuntu 24.04+) and `libraw.25.dylib` on macOS (libraw
0.22.x, current Homebrew). If your system provides a different major
version, set `LIBRAW_BUILD=1` to build from source against whatever you
have. See the [README](readme.html) for the full platform table.

Requires Elixir 1.15 or later.

## Examples

    {:ok, image} = LibRaw.decode("/path/to/photo.CR3")
    # => %{pixels: <<...>>, width: 6000, height: 4000, colors: 3, bps: 8}

    {:ok, meta} = LibRaw.metadata("/path/to/photo.CR3")
    # => %{camera_make: "Canon", camera_model: "EOS R5", iso: 400.0, ...}

# `decode_opts`

```elixir
@type decode_opts() :: [
  use_camera_wb: boolean(),
  no_auto_bright: boolean(),
  output_bps: 8 | 16,
  gamma: :srgb | :linear | {number(), number()}
]
```

# `image`

```elixir
@type image() :: %{
  pixels: binary(),
  width: non_neg_integer(),
  height: non_neg_integer(),
  colors: non_neg_integer(),
  bps: non_neg_integer()
}
```

# `metadata`

```elixir
@type metadata() :: %{
  camera_make: String.t(),
  camera_model: String.t(),
  captured_at: DateTime.t() | nil,
  iso: float(),
  shutter: float(),
  aperture: float(),
  orientation: integer()
}
```

# `decode`

```elixir
@spec decode(Path.t(), decode_opts()) :: {:ok, image()} | {:error, term()}
```

Decode a RAW image file, returning processed pixel data.

Runs the full libraw pipeline: open → unpack → dcraw_process →
in-memory bitmap. The NIF call is dispatched to a Dirty CPU scheduler
because decoding typically takes 100–500 ms.

## Parameters

  * `path` — `Path.t()`. Filesystem path to a camera RAW file. Passed
    to libraw verbatim; non-ASCII paths work because the NIF takes a
    UTF-8 binary, not a charlist.
  * `opts` — keyword list. See *Options* below.

## Options

  * `:use_camera_wb` — use the white balance stored in the file
    (default: `true`).
  * `:no_auto_bright` — disable libraw's automatic brightening
    (default: `false`).
  * `:output_bps` — bits per sample, either `8` or `16`
    (default: `8`). Other values return `{:error, {:invalid_output_bps, n}}`
    without touching the NIF.
  * `:gamma` — gamma curve: `:srgb`, `:linear`, or a `{g0, g1}` tuple
    of numbers (default: `:srgb`). Anything else returns
    `{:error, {:invalid_gamma, term}}`.

## Returns

  * `{:ok, image}` — `image` is a map with the keys:
    * `:pixels` — raw pixel bytes as a binary, row-major, interleaved
      per channel.
    * `:width` / `:height` — pixel dimensions.
    * `:colors` — channel count (typically `3` for RGB).
    * `:bps` — bits per sample, matches `:output_bps`.
  * `{:error, reason}` — option-validation failures (see *Options*),
    a libraw error string for unsupported / corrupted files, or
    `"invalid path: contains null byte"` for paths containing `\0`.

# `metadata`

```elixir
@spec metadata(Path.t()) :: {:ok, metadata()} | {:error, term()}
```

Read EXIF metadata from a RAW image file without running the full
decode pipeline.

Only `libraw_open_file` is called, so this is dramatically cheaper
than `decode/2` — typically single-digit milliseconds. Still
scheduled on a Dirty CPU scheduler because the open touches disk.

## Parameters

  * `path` — `Path.t()`. Filesystem path to a camera RAW file.

## Returns

  * `{:ok, metadata}` — `metadata` is a map with the keys:
    * `:camera_make` — manufacturer name (e.g. `"Canon"`).
    * `:camera_model` — model name (e.g. `"EOS R5"`).
    * `:captured_at` — `DateTime` in UTC, or `nil` if libraw
      reports no shooting timestamp (returns `0` from libraw).
    * `:iso` — ISO speed as a float.
    * `:shutter` — shutter speed in seconds as a float (e.g. `0.004`
      for 1/250s).
    * `:aperture` — f-number as a float.
    * `:orientation` — EXIF orientation / flip code as an integer.
  * `{:error, reason}` — a libraw error string for unsupported /
    corrupted files, or `"invalid path: contains null byte"`.

---

*Consult [api-reference.md](api-reference.md) for complete listing*
