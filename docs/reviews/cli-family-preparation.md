# CLI adoption of native family preparation

The CLI now passes an explicit opt-in policy to the existing
`ParametricIntegrand::from_graph_prepared` API. It supplies every original
parameter label and retains the returned native `FamilyPreparationReport` in
artifact provenance. The original DOT, model and restriction-card hashes remain
unchanged. `Original` remains the CLI default and still calls `from_graph`;
original runs omit the new field, preserving historical provenance serialization.
Requested preparation retains a native fallback report when no projection is
admitted. Direct parametric input rejects this graph-only option.

Configuration reuses the native enum without a second policy type:
`[generation.family_preparation.SingleUnitTerm]` with `max_states = 32`.
The native bound counts partial-fraction states. It is not a deadline.
Opted-in input inspection reports the actual active parameter count separately
from the original propagator count. Coordinate metadata continues to describe
the prepared space; it does not map discarded Schwinger coordinates back in.

The focused gate passed **38 tests**: two new CLI process controls, 27 CLI unit
tests, four existing CLI process tests, three dependency-provenance tests and
two named-coefficient process tests. The repeated massive-line control compares
the complete orders `[-1, 0, 1]` of the independently known `210 Gamma(eps)`
against both original physical and prepared NativeNamed runs. It uses ordinary
generation, separate-process cold artifact inspection and full integration,
and checks the retained active original label, source hashes and native report.
Other controls retain fallback, invalid-input errors, report tamper rejection,
original serialization and policy-sensitive input identity.

The first new-target run failed on two test fixture assumptions: direct input
requires a `terms` field, and native canonical symbols include their attribute
spelling. Only those fixtures changed before the successful rerun. Both logs
are retained under `output/diagnostics/cli-family-preparation-1/`, together with
Cargo artifact records, final source copies/hashes and `result.json`. Exact
Rust 1.98.1 builds, formatting and CLI all-target Clippy passed. Test processes
ran serially on CPU8 after the independent reference generation had reaped.
This is a CLI transport gate, not full on-shell capability or performance
acceptance. Independent review is recorded in
[cli-family-preparation-independent.md](cli-family-preparation-independent.md).
