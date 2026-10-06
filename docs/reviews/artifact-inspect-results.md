# Artifact inspection release acceptance

Date: 2026-10-07. The independent implementation reviews are
[native inspection](artifact-inspect-review.md) and
[saved generation observations](generation-record.md).

## Release and artifact invariance

The release CLI was rebuilt and the actual D05 ggHH example regenerated with
eight caller-owned workers and SymJIT O2. Generation completed in 62.506 seconds,
excluding the final artifact writes. This is a validation observation, not a
controlled performance comparison. The canonical local artifact pair is
`output/gghh_double_box.fsd.{json,dat}`.

The new `.dat` is byte-identical to the previous 3,666,802-byte artifact, and the
kernel layout/evaluator statistics in human JSON are unchanged. Identities remain:

- Artifact: `3b7e6abd92ac5adfe79f39af56cc8101abbcd83d19de26434117aec7c7ebf5df`.
- Kernel: `66f2af45b499f63bb15feb7148d0941eabb654ec2e83227443c023756208f09d`.

New JSON observations record eight workers and the requested
`coefficient_series` method. Final generation and inspection read those saved
facts and the same phase timings. Provenance paths remain relative. The complete
native generation metadata returned by inspection is unchanged from the prior
artifact. This comparison covers the complete generated evaluators and retained
maps without claiming a new integration estimate or convergence result.

## Actual terminal and machine output

Release pseudo-terminal checks passed for both the overview and sector 5 at
120, 80 and 40 columns. Each rendered line fits its terminal width. The evaluator
size column is emphasized; native expressions and section/table colors are
enabled only for a colored terminal. Plain output at 64 columns, `NO_COLOR` at
80 columns, redirected output and JSON contain no ANSI escapes. Per-factor
threshold-status messages and absolute workspace paths are absent.

The displayed top ten IDs are `5, 1, 7, 3, 21, 23, 25, 27, 6, 18`, agreeing with
descending exact evaluator byte size and the independent native probe. Sector 5
shows 93,605 exact-program bytes and its retained chart's coordinate equations.
Selected JSON is 5,587 bytes for this saved run; its chart array exactly equals
the full native metadata filtered by kernel ID five, without global factor data.
An out-of-range ID, a negative argument and a TOML run-card selector all fail
explicitly. Independent probes additionally cover namespace collisions,
historical metadata absence, missing generation observations and exact-only
artifacts.

## Checks and scope

- `cargo build --release -p fastsecdec-cli --locked`.
- `cargo clippy -p fastsecdec --lib -p fastsecdec-cli --bin fastsecdec --locked -- -D warnings`.
- `cargo check --manifest-path tests/portable-kernel/Cargo.toml --locked`.
- `cargo fmt --all --check` and `git diff --check`.

This host used its installed Rust 1.99 toolchain; `nix-shell` is unavailable.
The portable check establishes host compilation of the portable backend, not a
fresh Wasm/browser run. Other examples and committed tests/gates were left for
the user's deferred migration. Temporary sources, terminal captures, logs and
comparison artifacts remain only under ignored `output/artifact-inspect`.
