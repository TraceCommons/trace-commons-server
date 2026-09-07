#!/usr/bin/env bash
#
# Read the declared MSRV floor out of a cargo workspace, and check that the
# workspace's resolved dependency graph can actually be built at it.
#
# Why this exists: until 2026-09-06 nothing built at the declared floor until a
# release was already being cut. Every job in ci.yml uses
# dtolnay/rust-toolchain@stable; the only pinned toolchains live in
# release-contributor.yml and release-apps.yml, which run on a tag push. So a
# dependency bump that raised the floor left main green for days and then
# failed all four contributor-v0.10.0 build jobs in 95 seconds. See the
# `msrv-floor` job in ci.yml.
#
# The floor is read from `cargo metadata`, never hardcoded here. A literal
# version string in this file would be one more copy of the floor to forget --
# which is the exact shape of the original bug. `cargo metadata --no-deps`
# resolves `rust-version.workspace = true` inheritance, so it reports the
# effective floor of every member rather than the literal text of one manifest,
# and it needs no network.
#
# Usage:
#   scripts/ci/msrv-floor.sh print <manifest-path>
#       Print the declared floor (the highest rust-version among the
#       workspace's own members). Nothing else goes to stdout, so it is safe to
#       capture into a shell variable or a GITHUB_OUTPUT.
#
#   scripts/ci/msrv-floor.sh audit <manifest-path>
#       Print the floor, then fail if any crate in the resolved dependency
#       graph declares a rust-version above it, naming every offender. This is
#       the metadata-level form of the release failure: it needs no toolchain
#       install and compiles nothing.
#
#   scripts/ci/msrv-floor.sh flatpak <manifest-path> <flatpak-manifest>
#       Fail if the rust toolchain tarball pinned in the flatpak manifest is
#       older than the crate's declared floor. The flatpak build is
#       network-sandboxed, so rustup cannot rescue it at build time.
set -euo pipefail

usage() {
  echo "usage: $0 {print|audit} <manifest-path>" >&2
  echo "       $0 flatpak <manifest-path> <flatpak-manifest>" >&2
  exit 2
}

[ "$#" -ge 2 ] || usage
mode="$1"
manifest="$2"
[ -f "$manifest" ] || { echo "no such manifest: $manifest" >&2; exit 2; }
case "$mode" in
  print | audit) [ "$#" -eq 2 ] || usage ;;
  flatpak) [ "$#" -eq 3 ] || usage ;;
  *) usage ;;
esac

# Pad a rust-version to three components so "1.96" and "1.96.0" compare equal
# rather than the shorter one sorting first, which is how jq orders arrays.
JQ_SEMVER='def semver: (. / "." | map(tonumber) | . + [0,0,0] | .[0:3]);'

floor="$(
  cargo metadata --no-deps --format-version 1 --manifest-path "$manifest" |
    jq -r "$JQ_SEMVER"'
      [.packages[] | select(.rust_version != null) | .rust_version]
      | if length == 0 then "" else (sort_by(semver) | last) end
    '
)"

if [ -z "$floor" ]; then
  echo "REFUSING: no member of $manifest declares a rust-version." >&2
  echo "The floor is meant to be a promise to people building from source;" >&2
  echo "an undeclared floor is not one. Add rust-version to the manifest." >&2
  exit 1
fi

if [ "$mode" = "print" ]; then
  printf '%s\n' "$floor"
  exit 0
fi

if [ "$mode" = "flatpak" ]; then
  flatpak_manifest="$3"
  [ -f "$flatpak_manifest" ] || {
    echo "no such flatpak manifest: $flatpak_manifest" >&2
    exit 2
  }
  # The manifest pins one tarball version for both arches (see the
  # `only-arches` sources); grab the first `rust-<version>-` url and trust the
  # rest match, the way the sha256-count check in release_pipeline.rs already
  # assumes.
  pinned="$(grep -m1 -oE 'rust-[0-9]+\.[0-9]+\.[0-9]+-' "$flatpak_manifest" |
    head -1 | sed -E 's/^rust-//; s/-$//')"
  echo "crate requires rust-version = $floor"
  echo "manifest pins rust ${pinned:-<none>}"
  if [ -z "$pinned" ]; then
    echo "REFUSING: could not find a pinned rust toolchain version in $flatpak_manifest." >&2
    exit 1
  fi
  lowest="$(printf '%s\n%s\n' "$floor" "$pinned" | sort -V | head -1)"
  if [ "$lowest" != "$floor" ]; then
    echo "REFUSING: the manifest pins rust $pinned but the crate requires $floor." >&2
    echo "The flatpak build is network-sandboxed, so rustup cannot fix this at build time." >&2
    echo "Bump the pinned tarball (url + sha256, both arches) in $flatpak_manifest" >&2
    echo "to a version >= $floor before releasing." >&2
    exit 1
  fi
  exit 0
fi

[ "$mode" = "audit" ] || usage

echo "declared floor for $manifest: $floor"

# The full graph, so this sees git and registry dependencies alike. --locked
# keeps it honest about what the committed lockfile actually resolves to.
offenders="$(
  cargo metadata --format-version 1 --locked --manifest-path "$manifest" |
    jq -r --arg floor "$floor" "$JQ_SEMVER"'
      ($floor | semver) as $f
      | [ .packages[]
          | select(.rust_version != null)
          | select((.rust_version | semver) > $f)
          | "  \(.name)@\(.version) requires rustc \(.rust_version)"
        ]
      | unique
      | .[]
    '
)"

if [ -n "$offenders" ]; then
  echo >&2
  echo "REFUSING: a dependency raised the MSRV floor above the declared one." >&2
  echo >&2
  echo "$manifest declares rust-version = $floor, but these crates in its" >&2
  echo "resolved dependency graph require a newer rustc:" >&2
  echo >&2
  echo "$offenders" >&2
  echo >&2
  echo "This is release-breaking, not advisory. The release workflows pin the" >&2
  echo "toolchain to the declared floor, so a tag push fails every build job" >&2
  echo "with 'rustc $floor is not supported by the following packages'." >&2
  echo "Either drop or feature-gate the dependency, or raise the floor" >&2
  echo "deliberately -- and if you raise it, move all of these together:" >&2
  echo "  - rust-version in Cargo.toml (workspace.package)" >&2
  echo "  - rust-version in crates/trace-commons-contributor-gtk/Cargo.toml" >&2
  echo "  - the toolchain: pins in .github/workflows/release-contributor.yml" >&2
  echo "    and .github/workflows/release-apps.yml" >&2
  echo "  - the pinned rust tarball (url + sha256, both arches) in" >&2
  echo "    crates/trace-commons-contributor-gtk/flatpak/ai.tracecommons.Contributor.yml" >&2
  exit 1
fi

echo "no dependency of $manifest declares a rust-version above $floor"
