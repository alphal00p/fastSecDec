#!/usr/bin/env bash
# Complete above-threshold reproduction in a fresh, caller-selected directory.
set -euo pipefail
[[ $# == 1 ]] || { printf 'Usage: %s FRESH_OUTPUT_DIRECTORY\n' "$0" >&2; exit 2; }
output=$(realpath -m -- "$1")
[[ ! -e "$output" ]] || { printf 'Output already exists: %s\n' "$output" >&2; exit 2; }
repo=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)
source_dir="$repo/example/gg_hh_one_loop_ME"
binary="${FASTSECDEC_BIN:-$repo/target/release/fastsecdec}"
exporter="${GGHH_EXPORTER_BIN:-$repo/target/release/examples/gghh_one_loop_me}"
reference="${GGHH_REFERENCE_BIN:-$repo/target/release/examples/gghh_one_loop_reference}"
comparison="${GGHH_COMPARE_BIN:-$repo/target/release/examples/gghh_one_loop_compare}"
export SYMBOLICA_HIDE_BANNER=1
: "${SYMBOLICA_LICENSE:?Set your Symbolica license in the environment}"
cd "$repo"
if [[ "${FSD_SKIP_BUILD:-0}" != 1 ]]; then
    nix-shell --run 'cargo build --release --locked -j4 -p fastsecdec-cli -p fastsecdec --bin fastsecdec --example gghh_one_loop_me --example gghh_one_loop_reference --example gghh_one_loop_compare'
fi
mkdir -p "$output/fastsecdec/work" "$output/fastsecdec/logs" "$output/hepkit" "$output/madloop"
"$exporter" "$output/fastsecdec/inputs" --sqrt-s 400
cp "$output/fastsecdec/inputs/physical-point.json" "$output/point.json"
for mode in result ward1 ward2; do
    ward=()
    [[ "$mode" == result ]] || ward=("--$mode")
    "$reference" "$output/fastsecdec/inputs" "$output/hepkit/$mode" "${ward[@]}"
done
# The existing external reference owns its ten-minute/15-GiB process-group limit.
# Copy only its small reproduction inputs, leaving the 300-GeV results intact.
for file in reproduce.py process.mg5 param_card.dat MadLoopParams.dat check_point.f; do
    cp "$source_dir/madloop/$file" "$output/madloop/$file"
done
"${GGHH_PYTHON:-python3}" "$output/madloop/reproduce.py"
for input in "$output"/fastsecdec/inputs/diagram_*; do
    diagram=$(basename -- "$input")
    index=${diagram:8:2}
    seed=$((78139 + 10#$index * 104729))
    artifact="$output/fastsecdec/work/$diagram.fsd"
    "$binary" generate "$input/run.toml" --contour --workers 1 \
        --output "$artifact" --json --status-json \
        > "$output/fastsecdec/logs/generate-$index.json" \
        2> "$output/fastsecdec/logs/generate-$index.status.jsonl"
    "$binary" integrate "$artifact" --parameters "$input/point.toml" \
        --contour fixed --lambda "${FSD_LAMBDA:-0.000001}" \
        --contour-validation pilot --points 4096 --shifts 32 \
        --workers "${FSD_WORKERS:-8}" --seed "$seed" \
        --relative-tolerance 0.0001 --target-order 0 --max-rounds 8 \
        --save-result "$output/fastsecdec/work/$diagram.result.json" \
        --checkpoint "$output/fastsecdec/work/$diagram.checkpoint.json" --json --status-json \
        > "$output/fastsecdec/logs/integrate-$index.stdout.json" \
        2> "$output/fastsecdec/logs/integrate-$index.status.jsonl"
done
"$exporter" summarize "$output/fastsecdec/inputs" "$output/fastsecdec/work" "$output/fastsecdec/result.json"
"$comparison" "$output"
