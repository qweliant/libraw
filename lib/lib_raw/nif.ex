defmodule LibRaw.NIF do
  @moduledoc false

  use Rustler, otp_app: :libraw, crate: "libraw_nif"

  # Called on dirty CPU scheduler — decoding RAW files takes 100-500ms.
  @spec decode_nif(String.t(), integer(), integer(), integer(), float(), float()) ::
          {:ok,
           %{
             pixels: binary(),
             width: non_neg_integer(),
             height: non_neg_integer(),
             colors: non_neg_integer(),
             bps: non_neg_integer()
           }}
          | {:error, term()}
  def decode_nif(_path, _use_camera_wb, _no_auto_bright, _output_bps, _gamma0, _gamma1),
    do: :erlang.nif_error(:nif_not_loaded)

  # Called on dirty CPU scheduler — opening+reading EXIF is cheap but can block on I/O.
  @spec metadata_nif(String.t()) ::
          {:ok,
           %{
             camera_make: String.t(),
             camera_model: String.t(),
             captured_at: integer() | nil,
             iso: float(),
             shutter: float(),
             aperture: float(),
             orientation: integer()
           }}
          | {:error, term()}
  def metadata_nif(_path),
    do: :erlang.nif_error(:nif_not_loaded)
end
