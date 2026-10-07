#!/usr/bin/env bash
set -euo pipefail

if (( $# < 1 )); then
    printf 'Usage: %s EXAMPLE [OPTIONS...]\n' "$0" >&2
    exit 2
fi

example=$1
shift

exec ./target/release/fastsecdec generate "examples/${example}/run.toml" \
    --output "output/${example}.fsd" "$@"
