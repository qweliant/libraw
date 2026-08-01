# Releasing

This package ships **precompiled NIF binaries**. The download URL is derived
from the project version at compile time, so the release process has one hard
constraint that is easy to get wrong.

> **The git tag MUST be exactly `v` + the `@version` string in `mix.exs`.**
>
> There is no redirect and no fallback. If the tag and `@version` disagree,
> every consumer's `mix deps.compile` gets a 404 and falls back to a source
> build (or fails outright).

## Why the coupling exists

`lib/lib_raw/nif.ex` builds `base_url` from `Mix.Project.config()[:version]`:

```elixir
version = Mix.Project.config()[:version]

use RustlerPrecompiled,
  crate: "libraw_nif",
  version: version,
  base_url: "https://github.com/qweliant/libraw/releases/download/v#{version}",
  nif_versions: ["2.15", "2.16", "2.17"],
  ...
```

`RustlerPrecompiled` joins that base URL with a filename that *also* embeds the
version (`lib_name/4` in `rustler_precompiled`):

```
lib<crate>-v<version>-nif-<nif_version>-<target>.so.tar.gz
```

So with `@version "0.3.0-rc1"`, an Apple Silicon machine on NIF 2.17 requests
exactly:

```
https://github.com/qweliant/libraw/releases/download/v0.3.0-rc1/liblibraw_nif-v0.3.0-rc1-nif-2.17-aarch64-apple-darwin.so.tar.gz
```

The version appears **twice** — in the release tag and in the artifact
filename. Both come from `@version`. That is why artifacts from one version can
never be reused for another.

`.github/workflows/release.yml` keeps the build side in sync by reading the
version out of `mix.exs` rather than off the tag:

```bash
PROJECT_VERSION=$(sed -n 's/^  @version "\(.*\)"/\1/p' mix.exs | head -n1)
```

and `philss/rustler-precompiled-action` prepends the `v` itself. Never pass the
tag name to the action — on an RC tag you would get a doubled `v` or a
`-rc1`-less filename.

## Release checklist

Every command below reads the version from one shell variable. Set it once, to
exactly the `@version` you are releasing, and the rest of the checklist cannot
drift from `mix.exs`:

```bash
VERSION=$(sed -n 's/^  @version "\(.*\)"/\1/p' mix.exs | head -n1)
echo "$VERSION"     # sanity-check before continuing
```

Deriving it rather than typing it is the point — a hand-typed version is the
one failure this whole document exists to prevent.

### 1. Bump the version

Edit `@version` in `mix.exs`. This single value drives the tag, the
`base_url`, `PROJECT_VERSION` in CI, and all 12 artifact filenames.

Update `CHANGELOG.md` in the same commit.

> **Avoid a `-dev` suffix.** `RustlerPrecompiled.Config` treats any version
> whose prerelease segment contains `dev` as `force_build?: true` and skips the
> download path entirely. `-rc1`, `-beta.1`, etc. are fine and do download.

### 2. Tag and push

Re-read `VERSION` after the bump commit, then tag:

```bash
VERSION=$(sed -n 's/^  @version "\(.*\)"/\1/p' mix.exs | head -n1)
git tag "v$VERSION"
git push origin "v$VERSION"
```

Or trigger the workflow manually (`Actions → Release → Run workflow`) and pass
`tag_name` — it must still equal `v` + `@version`. The workflow creates the tag
at the current SHA if it does not exist.

### 3. Wait for all 12 matrix jobs

The matrix is 3 NIF versions x 4 targets:

| | `2.15` | `2.16` | `2.17` |
|---|---|---|---|
| `aarch64-apple-darwin` | ✓ | ✓ | ✓ |
| `x86_64-apple-darwin` | ✓ | ✓ | ✓ |
| `x86_64-unknown-linux-gnu` | ✓ | ✓ | ✓ |
| `aarch64-unknown-linux-gnu` | ✓ | ✓ | ✓ |

`fail-fast: false` is set, so a single broken target does not cancel the rest —
but the `release` job needs **all** of them to succeed. A partial matrix means
a partial release, which means 404s for whichever platform is missing.

### 4. Confirm the release assets

The GitHub Release must carry **24 files**: 12 `.tar.gz` plus 12 `.sha256`.

```bash
gh release view "v$VERSION" --json assets --jq '.assets[].name' | sort
gh release view "v$VERSION" --json assets --jq '.assets[].name' | wc -l    # 24

# every asset must carry this exact version, and no other
gh release view "v$VERSION" --json assets --jq '.assets[].name' \
  | grep -vc -- "-v$VERSION-nif-"                                          # 0
```

**If you changed a runner label since the last release, check the libraw ABI.**
Each artifact hard-links whatever libraw soname its runner provided, and both
Linux targets must agree or half the Linux userbase gets a NIF that cannot
`dlopen`. This is not hypothetical: the original matrix paired `ubuntu-22.04`
(libraw 0.20 → `libraw.so.20`) with `ubuntu-24.04-arm` (libraw 0.21 →
`libraw.so.23`).

```bash
mkdir -p /tmp/abi && cd /tmp/abi
# -R is required: outside a checkout `gh` has no remote to infer and fails
# with "no git remotes found".
gh release download "v$VERSION" -R qweliant/libraw --pattern '*nif-2.17-*.so.tar.gz'
for f in *.so.tar.gz; do tar xzf "$f"; done

# objdump reads both ELF and Mach-O and ships with Xcode CLT, so this works
# on macOS as well — `readelf` does not exist there.
for f in *.so; do
  printf '%-58s ' "$f"
  { objdump -p "$f" 2>/dev/null | awk '/NEEDED/ && /libraw/ {print $2}'
    otool -L "$f" 2>/dev/null | grep -o '[^[:space:]]*libraw\.[0-9]*\.dylib'
  } | paste -sd' ' -
done
```

Both Linux lines must print the **same** soname (`libraw.so.23`). The macOS
lines print an absolute Homebrew path and differ by prefix only —
`/opt/homebrew/...` on arm64, `/usr/local/...` on Intel — which is expected.

The README's *libraw version matters* table states the required versions to
users — update it if these change.

### 5. Generate the checksum file

```bash
cd -                                   # back to the repo root
rm -f checksum-Elixir.LibRaw.NIF.exs   # entries are keyed by filename, and
                                       # every filename changed with the version
mix rustler_precompiled.download LibRaw.NIF --all --print

grep -c '=>' checksum-Elixir.LibRaw.NIF.exs                 # 12
grep -c -- "-v$VERSION-nif-" checksum-Elixir.LibRaw.NIF.exs  # 12
```

This downloads every artifact from the release you just published and writes
`checksum-Elixir.LibRaw.NIF.exs` into the repo root.

Delete it first. A *successful* run rewrites the file wholesale, so a leftover
would be overwritten anyway — but a **failed** run never reaches the write at
all. The task raises on the first 404, which means a mistimed run (assets not
uploaded yet, wrong tag) leaves the *previous* version's file sitting in place,
looking perfectly valid. Publish that and every consumer fails checksum
verification. Deleting first turns that silent failure into a loud one.

- It is **gitignored** (see `.gitignore`) — do not commit it.
- It **is** shipped in the Hex package via the `files` list in `mix.exs`
  (`checksum-*.exs`).
- Consumers verify every downloaded `.tar.gz` against this map. A package
  published without it cannot install from precompiled binaries at all.

`mix hex.publish` will not let you skip this: because `checksum-*.exs` is in
the `files` list, the build aborts with

```
** (Mix) Stopping package build due to errors.
Missing files: checksum-*.exs
```

Run step 5 **after** step 4 — the download task hits the real release URLs, so
it fails if the assets are not up yet.

### 6. Publish to Hex

```bash
mix hex.publish
```

Verify the printed file list includes `checksum-Elixir.LibRaw.NIF.exs` and that
`Version:` matches `@version` before confirming.

### 7. Verify on a clean machine

```bash
mix deps.get && mix deps.compile
```

Expect no Rust toolchain invocation. Set `LIBRAW_BUILD=1` to force a source
build if you want to test the fallback path. Note that libraw itself must still
be installed (`brew install libraw` / `apt install libraw-dev`) — the NIF links
against it dynamically on every platform.

## Prereleases on Hex

Hex accepts prerelease versions (`mix hex.publish` publishes `0.3.0-rc1`
without complaint), but Hex's dependency solver **excludes prereleases from
ordinary version ranges**. Consumers must opt in explicitly:

| Consumer requirement | Resolves to `0.3.0-rc1`? |
|---|---|
| `{:libraw, "~> 0.3"}` | No — and while only the RC exists, resolution **fails** |
| `{:libraw, ">= 0.2.0"}` | No — picks the newest stable |
| `{:libraw, "~> 0.3.0-rc0"}` | Yes (until a stable `0.3.x` exists, which wins) |
| `{:libraw, "0.3.0-rc1"}` | Yes — exact pin |

This is deliberate: an RC cannot leak into anyone's build by accident. It also
means an RC is only useful to testers you tell to pin it explicitly.

## Promoting an RC to final

RC artifacts **cannot be reused**. The version is baked into both the release
tag and every artifact filename, so a `0.3.0` consumer will look for
`.../v0.3.0/liblibraw_nif-v0.3.0-nif-2.17-...` — files that do not exist under
the `v0.3.0-rc1` release.

The promotion is a full rebuild:

1. Bump `@version` from `0.3.0-rc1` to `0.3.0` in `mix.exs`.
2. In `CHANGELOG.md`, rename the `## [0.3.0-rc1]` heading to `## [0.3.0]`, set
   the release date, drop the RC preamble, and update the compare link at the
   bottom. The entry already carries the full 0.3.0 content — the RC is not a
   separate entry.
3. Strip the RC pinning callout from `README.md`. It tells readers `~> 0.3`
   resolves to nothing, which becomes false the moment the final publishes —
   and it ships to Hex *and* hexdocs, so stale advice there is user-facing.
4. Commit, then tag `v0.3.0` and push (a **new** tag — do not move `v0.3.0-rc1`).
5. Wait for all 12 matrix jobs again; confirm 24 assets on the `v0.3.0` release,
   and re-run the ABI check.
6. `rm` the old checksum file and re-run the download task; every filename
   changed, so the RC's file is entirely stale.
7. `mix hex.publish`.

Keep the RC release and its tag around; they are harmless and anyone who pinned
`"0.3.0-rc1"` still resolves.

## If you get the tag wrong

Published Hex versions are immutable, so a mismatched tag cannot be fixed by
retagging alone if the package is already on Hex.

- **Caught before `mix hex.publish`:** delete the bad tag and release
  (`git push --delete origin <tag>`, `gh release delete <tag>`), fix
  `@version`, re-tag, rebuild.
- **Caught after publishing:** you cannot re-point the URL. Either create a
  release under the tag the package is actually asking for and upload the 12
  artifacts there, or `mix hex.retire` the bad version and publish a patch with
  a corrected `@version`.
