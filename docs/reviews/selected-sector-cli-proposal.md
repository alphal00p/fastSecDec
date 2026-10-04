# Qualified sector selection and retained metadata: proposal

This document is design-only during the saved-result milestone gate. It builds
on the existing native `ResultScope`, parent manifest and exact-contribution
policy; it does not introduce another sector representation or reconstruction
of chart algebra.

## Selection boundary

Add explicit compiled-kernel sector selection to `run` and `integrate`, with
`--sectors 2,7` and a required `--exact-contributions include|exclude` when a
subset is requested. IDs refer to the saved kernel slice, not raw geometric
charts. One compiled representative can include several symmetry-equivalent
charts and its existing multiplicity. Selection never strips that multiplicity
or changes its kernel expression.

Use the native `ResultScope` in normalized CLI integration settings. The absent
historical field means `FullIntegral`; omit that default when serializing settings
to preserve existing full-run checkpoint identity. A direct native TOML form can
avoid another scope schema:

```toml
[integration.scope.SelectedSectors]
sector_ids = [2, 7]
exact_policy = "IncludeAll"
```

Both paths map to the same native value. Reject duplicate, unknown or otherwise
invalid IDs before worker construction. An explicit empty selection is still
selected scope: `IncludeAll` returns only the folded exact offset, while
`ExcludeAll` describes a zero contribution. Even an explicit selection naming
every stochastic ID remains selected scope unless the caller requests full
scope. No implicit promotion depends on a list length.

The native manifest owner should expose a small validated projection to an
`IntegrationProblem`, sharing the subset and exact-policy checks already used
by saved-result validation. It selects existing `SectorSpec` records, preserves
the full coefficient layout, and uses either the complete exact vector or a
correctly sized zero vector. It performs no integration or algebra. CLI code
should call that method instead of duplicating subset validation.

The existing outer artifact identity continues to isolate session/checkpoint
data. Selected scope is part of numerical settings identity, and native restore
also checks the projected problem's IDs, dimensions and exact vector. Changing
scope or exact policy cannot reuse another allocation's samples. Worker-count
changes remain permitted. The parent saved-result manifest remains complete,
using the inner scientific kernel identity; only the selected numerical scope
and contribution rows change.

## Execution and reporting

Keep original sector IDs throughout scheduling, kernel evaluation and report
rows. Replace positional worker-context assumptions with selected-ID lookup;
construct evaluator contexts only for selected sectors. Existing metadata-only
replay state can remain indexed over the full kernel for checkpoint format
compatibility. Unselected state stays untouched and creates no evaluator.
Accepted-state validation/submission ordering remains unchanged.

Use the existing QMC/Havana session for the projected problem. Democratic QMC
still combines the selected sectors using common-shift covariance, while adaptive
or Havana modes retain their established replica relation. Do not sum marginal
errors to form a subset total or derive a subset by subtracting rounded values
from a previously saved full result.

Every final report, live status and saved result must visibly state selected
scope and its exact policy. A selected target can be reached without establishing
full-integral convergence. Preserve the old `converged` field as a full-integral
claim and add explicitly scoped target-completion information for selected runs;
native `StoppingReason::TargetReached` is interpreted within the declared scope.
The loop may stop when the selected estimate meets its requested tolerance.
Native saved-result comparison/export continue rejecting unscoped full-integral
reference use of selected estimates. This changes neither stored-reference
extraction nor the evidence attached to an original target.

## Artifact inspection from retained native metadata

Close the pending retained-sector-inspection boundary using
`KernelSet::generation_metadata()`. The generation owner already retains
`DomainAssessment`, `ChartRecord`, `CoordinateMap` and exact `SectorMap` geometry.
The kernel transport owner already owns their portable canonical representation.
Expose that same native presentation/serialization boundary to the CLI; do not
build a competing JSON schema by parsing kernel payload internals or recomputing
maps, support, derivatives or valuations.

Inspection should show branch policy and assertion/certificate status, source
domain, chart and representative IDs, chart-to-kernel association, representative
permutation, exact exponent matrix/valuations and the retained coordinate images
and measure. Detailed canonical expressions can remain behind `--expressions`.
Keep these semantics explicit:

- `kernel_sector=None` means exact, cancelled or truncated-to-zero contribution;
  separate exact coefficients for each such chart are not retained.
- Projective images are gauge-fixed, not normalized simplex coordinates, and
  the measure is the positive real density factor, not an oriented complex
  determinant.
- Geometry valuation rows refer to the generation owner's deduplicated support
  ordering. Current metadata does not map every row back to a unique named U/F
  factor; display its native indices without inventing such labels.
- Legacy artifacts without retained metadata report that absence explicitly;
  do not manufacture a chart from the compiled expression or sector ID.

The first implementation can preserve existing artifact-inspect loading behavior.
Result-only `show-result` remains free of symbolic loading/JIT. A separate
metadata-only artifact loader would require a deliberate reuse/refactoring of
the existing checked transport path, rather than an unchecked JSON shortcut.

## Focused acceptance

Use a small native two-sector fixture with an independently known subset vector
and a separate folded exact offset. Exercise include/exclude policy, full versus
selected scope, native common-shift covariance, unknown/duplicate IDs, worker
changes on partial resume, scope/policy mismatch rejection and selected-reference
ineligibility. Verify that an explicit empty subset remains selected and that
the unqualified full-integral convergence claim stays false. Preserve full-run
missing-field checkpoint behavior.

Inspect a portable fixture with symmetry-related charts and compare CLI output
to the existing native metadata object, including exact geometry and measure;
exercise explicit absence for a valid legacy artifact. No test should require
matching Pathfinder sector numbering or counts. A missing referenced graph
should produce the existing structured process error at the input boundary;
one meaningful process test suffices, without mirroring filesystem suffix logic.

No production implementation, default sampling change, selected-reference schema,
new CAS capability or phase-two algorithm is authorized by this proposal alone.
