#!/usr/bin/env bash
# Called from the host after cloning the exact linked FastSecDec revision.
set -euo pipefail
[[ $# == 2 ]] || { printf 'Usage: %s COMMUNITY_CHECKOUT NEW_DEPENDENCY_OUTPUT\n' "$0" >&2; exit 2; }
community_root=$(cd "$1" && pwd -P)
dependency_output=$(cd "$2" && pwd -P)
delivery_root=$(cd "$(dirname "$0")/../../.." && pwd -P)
[[ -f "$community_root/Cargo.toml" && -f "$delivery_root/bindings/python/Cargo.toml" ]]
bash "$delivery_root/scripts/bootstrap-dependencies.sh" "$delivery_root" "$dependency_output/owners"
# The checkout supplies instructions and tests only. The host must consume the
# binding and its core from its declared public Git source, never a local patch.
awk '
  /^\[/ { skip = ($0 == "[patch.\"https://github.com/alphal00p/fastSecDec\"]") }
  !skip { print }
' "$dependency_output/owners/overlay.toml" > "$dependency_output/overlay-community-git.toml"
dependency_cargo_home="$dependency_output/cargo-home"
mkdir "$dependency_cargo_home"
cp "$dependency_output/overlay-community-git.toml" "$dependency_cargo_home/config.toml"
printf 'Prepared: %s\n' "$dependency_output/overlay-community-git.toml"
printf 'For maturin/Pyodide, set CARGO_HOME=%s for that command.\n' "$dependency_cargo_home"
printf 'The pinned checkout at %s supplies the binding tests and examples.\n' "$delivery_root"
