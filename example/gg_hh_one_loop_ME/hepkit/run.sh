#!/usr/bin/env bash
set -euo pipefail
script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_dir="$(cd -- "$script_dir/../../.." && pwd)"
cd "$repo_dir"
input_dir="example/gg_hh_one_loop_ME/fastsecdec/inputs"
if [[ ! -f "$input_dir/manifest.json" ]]; then
    echo "First generate the shared native catalogue using fastsecdec/run.sh." >&2
    exit 1
fi
nix-shell --run 'cargo build --locked -j 4 -p fastsecdec --example gghh_one_loop_reference'
exec ./target/debug/examples/gghh_one_loop_reference \
    "$input_dir" "${1:-$script_dir/result}" "${@:2}"
