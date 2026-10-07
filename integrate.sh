#!/usr/bin/env bash
set -euo pipefail

if (( $# < 2 )); then
    printf 'Usage: %s EXAMPLE {qmc|discrete_mc} [OPTIONS...]\n' "$0" >&2
    exit 2
fi

example=$1
method=$2
shift 2

case "$method" in
    qmc|discrete_mc) ;;
    *) printf 'Unknown integration method: %s (expected qmc or discrete_mc)\n' "$method" >&2; exit 2 ;;
esac

exec ./target/release/fastsecdec integrate "output/${example}.fsd" \
    --full-integral \
    --parameters "examples/${example}/point.toml" \
    --integration-settings "examples/${example}/${method}.toml" \
    --checkpoint "output/${example}.${method}.checkpoint.json" \
    --save-result "output/${example}.${method}.result.json" "$@"
