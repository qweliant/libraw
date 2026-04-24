defmodule LibRaw.MixProject do
  use Mix.Project

  @version "0.1.0"
  @source_url "https://github.com/qweliant/libraw"

  def project do
    [
      app: :libraw,
      version: @version,
      elixir: "~> 1.14",
      start_permanent: Mix.env() == :prod,
      deps: deps(),
      description: "Rustler NIF wrapping libraw for native camera RAW decoding in the BEAM",
      package: package(),
      docs: docs(),
      name: "LibRaw"
    ]
  end

  def application do
    [
      extra_applications: [:logger]
    ]
  end

  defp deps do
    [
      {:rustler, "~> 0.33", runtime: false},
      {:rustler_precompiled, "~> 0.8"},
      {:ex_doc, "~> 0.31", only: :dev, runtime: false}
    ]
  end

  defp package do
    [
      licenses: ["MIT"],
      links: %{"GitHub" => @source_url},
      files: ~w(lib native mix.exs README.md LICENSE)
    ]
  end

  defp docs do
    [
      main: "LibRaw",
      source_url: @source_url,
      extras: ["README.md"]
    ]
  end
end
