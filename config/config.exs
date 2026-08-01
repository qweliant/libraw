import Config

# Build the NIF from source when working on this repo.
#
# Without this, `mix test` in a checkout tries to download a precompiled NIF
# for the current @version from GitHub Releases and fails with a 404 whenever
# that release does not exist yet — which is always the case while developing
# the version that is about to be released.
#
# Mix only evaluates the *top-level* project's config, so this never applies
# to applications that depend on :libraw — they still get the precompiled
# path. `config/` is also excluded from the Hex package `files` list.
#
# Contributors without a Rust toolchain can still use the precompiled NIF for
# an already-released version by running with MIX_ENV=prod.
config :rustler_precompiled, :force_build, libraw: Mix.env() in [:dev, :test]
