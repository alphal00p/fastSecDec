# Generation output preflight

Date: 2026-10-07

An explicit output such as `output/gghh_double_box.fsd.json` used to fail only
when the completed artifact was written. The CLI now validates the basename
immediately after parsing arguments, before entering command execution. This
covers `generate`, `run`, and `run --resume`, including their default output
paths. Existing text, color and JSON error reporting remain in use.

`artifact::paths` remains the sole owner of basename validation and sibling-path
construction. It performs no filesystem access. The CLI preflight calls it;
the generation entry point also calls it before status publication, input I/O
or worker creation, and artifact persistence retains its own call at the write
boundary. No new path grammar or separate validator was introduced. Output
option help now states the basename convention directly.

Focused checks used the debug CLI and temporary files under ignored
`output/generation-preflight`. No tests or other examples were changed:

- Four invalid `.json`/`.dat` cases across generation, run and resumed run
  returned the basename error even with nonexistent run cards or references.
  They created no files. JSON/status mode produced one structured error and no
  status snapshots. The slowest negative check took 0.024 seconds on this host.
- A pseudo-terminal invocation returned the same error without entering the
  alternate screen or hiding the cursor, confirming dashboard setup was skipped.
- A valid basename with the same nonexistent card reached the input-file error,
  establishing the intended validation order.
- A valid small direct-input generation wrote its `.fsd.json` and `.fsd.dat`
  siblings, with no file at the basename itself.

This is an argument-admission correction. It does not change generation,
threshold policy, evaluator serialization or integration semantics. Native
Rust/build tools were used because `nix-shell` is unavailable on this host.
