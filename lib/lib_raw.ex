defmodule LibRaw do
  @moduledoc """
  Native camera RAW decoding for the BEAM via a Rustler NIF wrapping libraw.

  ## Prerequisites

  libraw must be installed on the system:

      # macOS
      brew install libraw

      # Debian / Ubuntu
      apt install libraw-dev

  ## Examples

      {:ok, image} = LibRaw.decode("/path/to/photo.CR3")
      # => %{pixels: <<...>>, width: 6000, height: 4000, colors: 3, bps: 8}

      {:ok, meta} = LibRaw.metadata("/path/to/photo.CR3")
      # => %{camera_make: "Canon", camera_model: "EOS R5", iso: 400.0, ...}
  """

  alias LibRaw.NIF

  @type decode_opts :: [
          use_camera_wb: boolean(),
          no_auto_bright: boolean(),
          output_bps: 8 | 16,
          gamma: :srgb | :linear | {number(), number()}
        ]

  @type image :: %{
          pixels: binary(),
          width: non_neg_integer(),
          height: non_neg_integer(),
          colors: non_neg_integer(),
          bps: non_neg_integer()
        }

  @type metadata :: %{
          camera_make: String.t(),
          camera_model: String.t(),
          captured_at: DateTime.t() | nil,
          iso: float(),
          shutter: float(),
          aperture: float(),
          orientation: integer()
        }

  # sRGB gamma curve parameters used by dcraw
  @gamma_srgb {2.222, 12.92}
  # Linear (no gamma correction)
  @gamma_linear {1.0, 1.0}

  @doc """
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
      `"invalid path: contains null byte"` for paths containing `\\0`.
  """
  @spec decode(Path.t(), decode_opts()) :: {:ok, image()} | {:error, term()}
  def decode(path, opts \\ []) do
    use_camera_wb = Keyword.get(opts, :use_camera_wb, true)
    no_auto_bright = Keyword.get(opts, :no_auto_bright, false)
    output_bps = Keyword.get(opts, :output_bps, 8)
    gamma_opt = Keyword.get(opts, :gamma, :srgb)

    with :ok <- validate_bps(output_bps),
         {:ok, {g0, g1}} <- resolve_gamma(gamma_opt) do
      NIF.decode_nif(
        path,
        bool_to_int(use_camera_wb),
        bool_to_int(no_auto_bright),
        output_bps,
        g0,
        g1
      )
    end
  end

  @doc """
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
  """
  @spec metadata(Path.t()) :: {:ok, metadata()} | {:error, term()}
  def metadata(path) do
    case NIF.metadata_nif(path) do
      {:ok, raw} ->
        {:ok, Map.update!(raw, :captured_at, &parse_timestamp/1)}

      error ->
        error
    end
  end

  # --- private helpers ---

  defp validate_bps(bps) when bps in [8, 16], do: :ok
  defp validate_bps(bps), do: {:error, {:invalid_output_bps, bps}}

  defp resolve_gamma(:srgb), do: {:ok, @gamma_srgb}
  defp resolve_gamma(:linear), do: {:ok, @gamma_linear}
  defp resolve_gamma({g0, g1}) when is_number(g0) and is_number(g1), do: {:ok, {g0 * 1.0, g1 * 1.0}}

  defp resolve_gamma(other), do: {:error, {:invalid_gamma, other}}

  defp bool_to_int(true), do: 1
  defp bool_to_int(false), do: 0

  # The NIF returns the Unix timestamp as an integer (seconds since epoch).
  # 0 means "not set" in libraw.
  defp parse_timestamp(0), do: nil
  defp parse_timestamp(unix) when is_integer(unix) do
    DateTime.from_unix!(unix, :second)
  end
  defp parse_timestamp(nil), do: nil
end
