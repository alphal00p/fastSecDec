#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../.."
repo=$PWD
here="$repo/example/gg_hh_one_loop_ME/fastsecdec"
binary="${FASTSECDEC_BIN:-$repo/target/release/fastsecdec}"
exporter="${GGHH_EXPORTER_BIN:-$repo/target/release/examples/gghh_one_loop_me}"
export SYMBOLICA_HIDE_BANNER=1
: "${SYMBOLICA_LICENSE:?Set your Symbolica license in the environment}"
if [[ "${FSD_SKIP_BUILD:-0}" != 1 ]]; then
    nix-shell --run 'cargo build --release --locked -j4 -p fastsecdec-cli -p fastsecdec --bin fastsecdec --example gghh_one_loop_me'
fi
if [[ ! -f "$here/inputs/manifest.json" ]]; then
    "$exporter" "$here/inputs"
fi
mkdir -p "$here/work" "$here/logs"
for input in "$here"/inputs/diagram_*; do
    diagram=$(basename "$input")
    index=${diagram:8:2}
    seed=$((78139 + 10#$index * 104729))
    "$binary" generate "$input/run.toml" --workers 1 \
        --output "$here/work/$diagram.fsd" --json --status-json \
        > "$here/logs/generate-$index.json" 2> "$here/logs/generate-$index.status.jsonl"
    "$binary" integrate "$here/work/$diagram.fsd" \
        --parameters "$input/point.toml" --points 4096 --shifts 32 \
        --workers "${FSD_WORKERS:-2}" --seed "$seed" \
        --relative-tolerance 0.00002 --target-order 0 --max-rounds 6 \
        --save-result "$here/work/$diagram.result.json" \
        --checkpoint "$here/work/$diagram.checkpoint.json" --json --status-json \
        > "$here/logs/integrate-$index.stdout.json" 2> "$here/logs/integrate-$index.status.jsonl"
done
"$exporter" summarize "$here/inputs" "$here/work" "$here/result.json"
