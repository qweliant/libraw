defmodule LibRaw.DecodeFixtureTest do
  @moduledoc """
  End-to-end smoke test against a real RAW file.

  Excluded by default (`@moduletag :integration`) so the unit suite runs
  without external assets. Run with `mix test.smoke` (or
  `mix test --include integration`) before publishing.

  Drop any RAW file at `test/fixtures/sample.raw` — CR2, CR3, NEF, ARW,
  DNG, RAF, etc. all work. The file is gitignored.
  """

  use ExUnit.Case, async: true

  @moduletag :integration

  @fixture Path.join(__DIR__, "fixtures/sample.raw")

  setup_all do
    File.exists?(@fixture) ||
      flunk("""
      Missing fixture: #{@fixture}

      Drop any RAW file (CR2, CR3, NEF, ARW, DNG, RAF, ...) at the path
      above. Then run: mix test.smoke
      """)

    :ok
  end

  describe "decode/2" do
    test "decodes an 8-bit RGB image of the expected byte size" do
      assert {:ok, img} = LibRaw.decode(@fixture)

      assert is_binary(img.pixels)
      assert img.width > 0
      assert img.height > 0
      assert img.colors in [1, 3, 4]
      assert img.bps == 8

      assert byte_size(img.pixels) == img.width * img.height * img.colors
    end

    test "16-bit output doubles the pixel buffer" do
      {:ok, img8} = LibRaw.decode(@fixture)
      {:ok, img16} = LibRaw.decode(@fixture, output_bps: 16)

      assert img16.bps == 16
      assert img16.width == img8.width
      assert img16.height == img8.height
      assert byte_size(img16.pixels) == byte_size(img8.pixels) * 2
    end

    test "linear gamma decode succeeds and matches default dimensions" do
      {:ok, srgb} = LibRaw.decode(@fixture)
      {:ok, lin} = LibRaw.decode(@fixture, gamma: :linear)

      assert lin.width == srgb.width
      assert lin.height == srgb.height
      assert byte_size(lin.pixels) == byte_size(srgb.pixels)
      # A linear-gamma decode should produce a different bit pattern than
      # the sRGB default. If they match, the gamma options aren't taking
      # effect.
      refute lin.pixels == srgb.pixels
    end
  end

  describe "metadata/1" do
    test "returns reasonable EXIF fields" do
      assert {:ok, meta} = LibRaw.metadata(@fixture)

      assert is_binary(meta.camera_make)
      assert is_binary(meta.camera_model)
      assert is_float(meta.iso)
      assert is_float(meta.shutter)
      assert is_float(meta.aperture)
      assert is_integer(meta.orientation)
      assert is_nil(meta.captured_at) or match?(%DateTime{}, meta.captured_at)
    end
  end
end
