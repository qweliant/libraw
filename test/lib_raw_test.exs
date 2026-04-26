defmodule LibRawTest do
  use ExUnit.Case, async: true

  describe "decode/2" do
    test "returns an error tuple for a missing file" do
      assert {:error, _reason} = LibRaw.decode("nonexistent.cr3")
    end

    test "rejects an invalid :output_bps before touching the NIF" do
      assert LibRaw.decode("nonexistent.cr3", output_bps: 99) ==
               {:error, {:invalid_output_bps, 99}}
    end

    test "rejects an invalid :gamma before touching the NIF" do
      assert LibRaw.decode("nonexistent.cr3", gamma: :bad) ==
               {:error, {:invalid_gamma, :bad}}
    end
  end

  describe "metadata/1" do
    test "returns an error tuple for a missing file" do
      assert {:error, _reason} = LibRaw.metadata("nonexistent.cr3")
    end
  end
end
