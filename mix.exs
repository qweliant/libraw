defmodule LibRaw.MixProject do
  use Mix.Project

  @version "0.3.0-rc1"
  @source_url "https://github.com/qweliant/libraw"

  def project do
    [
      app: :libraw,
      version: @version,
      # 1.15 is a hard floor, not a preference: both rustler and
      # rustler_precompiled declare `elixir: "~> 1.15"` themselves, so Mix
      # refuses to compile them on 1.14 — the precompiled path is affected
      # too, not just source builds.
      elixir: "~> 1.15",
      start_permanent: Mix.env() == :prod,
      deps: deps(),
      aliases: aliases(),
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

  def cli do
    [preferred_envs: ["test.smoke": :test]]
  end

  defp deps do
    [
      # Optional: only needed for source builds (LIBRAW_BUILD=1 or an
      # unsupported target). Keeping it optional means consumers on a
      # precompiled target don't pull rustler + jason into their dep tree.
      {:rustler, "~> 0.37", runtime: false, optional: true},
      {:rustler_precompiled, "~> 0.8"},
      {:ex_doc, "~> 0.31", only: :dev, runtime: false}
    ]
  end

  defp aliases do
    [
      "test.smoke": ["test --include integration"]
    ]
  end

  defp package do
    [
      licenses: ["MIT"],
      links: %{"GitHub" => @source_url},
      files: ~w(
        lib
        checksum-*.exs
        native/libraw_nif/src
        native/libraw_nif/Cargo.toml
        native/libraw_nif/Cargo.lock
        native/libraw_nif/build.rs
        mix.exs
        README.md
        CHANGELOG.md
        LICENSE
      )
    ]
  end

  defp docs do
    [
      main: "LibRaw",
      source_url: @source_url,
      extras: ["README.md", "CHANGELOG.md"]
    ]
  end
end
