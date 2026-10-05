# Independent review of CLI QMC context lifetime

The prepared on-shell fullgraph run established generation, publication and
fresh-process loading for 1,026 sectors. Generation completed in
1,695.486132250 seconds and cold inspection in 136.719175988 seconds. The
two-worker numerical stage reached its 180-second deadline before completing
any sector: the last status contained 38,912 of 8,404,992 points, while the
retained checkpoint contained 36,864 accepted evaluations. No integral result
was produced. These observations establish persistence capacity, not numerical
agreement or performance parity.

The independent terminal record is
`output/diagnostics/native-named-prepared-fullgraph-4/independent-terminal-review.json`
(SHA256 `621a922005dc7e9ad7ded84e91f6dc50d2bdf5b8ddc350464cc0642c94221e2f`).
It rechecks all 71 frozen files, the retained outputs and absence of the eight
owned processes and four process groups. Its checkpoint progress is deliberately
kept distinct from the later live status.

## Existing ownership and reuse

The existing CLI QMC driver constructs an evaluator context for every selected
sector in every worker slot. `AcceptedReplay::contexts` calls the public
`KernelSet::restore_evaluation_context`; `WeightedEvaluationContext` owns a
`SectorKernel`. That kernel shares its portable program bytes but clones native
evaluator state, including exact and conditioning evaluators. Each context can
also retain four native multiprecision evaluator entries.

The existing public APIs already provide the required alternative:

- `QmcSession::worker_context` supplies a caller-owned point-generation worker.
- `KernelSet::restore_evaluation_context` restores a sector evaluator from its
  identity-bound accepted `ReplayState`.
- `WeightedEvaluationContext::merge_state` incorporates newly accepted maxima.
- The CLI's existing `submit_package` validates candidate state, submits the
  complete numerical package, and only then advances accepted replay history.

No new algebra, graph operation, evaluator, lattice implementation or public
library API is needed. The change belongs entirely to the CLI caller:
retain one current sector context per worker slot, drop it before constructing
a different sector's context, and restore its scientific history from the
accepted replay store. Existing Monte Carlo context management is outside this
change.

## Review boundaries and current status

Independent source review accepts the private `QmcSlot` implementation and
`AcceptedReplay::context` helper. Different-sector preparation drops the old
context before constructing its successor; failure leaves the slot empty.
Same-sector reuse merges accepted state. The existing native task order,
weighted evaluation callback, ordered package submission and checkpoint owner
remain unchanged. Monte Carlo retains its existing context map.

The four focused tests exercise restoration of a nonzero accepted envelope
after eviction, failed-prefix exclusion, full two-component mean/covariance and
diagnostic equality against the former eager caller under identical one- and
three-worker schedules, and partial-checkpoint continuation with eight workers.
The equivalence test explicitly requires native precision rescue and precision
above 53 bits. Existing controls cover rejected submissions, selected original
sector IDs, checkpoint identity and native lattice/refinement behavior.

The frozen source/build review is
`output/diagnostics/qmc-worker-context-lifetime-2/independent-source-review.json`
(SHA256 `8ffb3441f4dcb624b1b9412c240163d2b481031270c47c268dd1cfe168efa169`).
All 170 current and archived source bindings and 182 frozen files were checked.
The exact test binary is
`09a60e74eb3a9257e499256a1c2c1d3389926de23dd17fc45423e245d83cb308`.
Compilation and scoped Clippy passed. After the Pathfinder process released the
runtime slot, all 16 `driver::` controls passed with zero failures or ignored
tests. The bounded test process exited successfully in 3.332841759 seconds,
and its parent tool completed and reaped with exit zero. All 182 frozen
pre/postchecks passed. The final independent record is
`output/diagnostics/qmc-worker-context-lifetime-2/independent-validation.json`.
This accepts the caller-only context change without an additional workspace or
algebra-oracle gate.

The release build also exited successfully. Its twelve frozen files were
independently verified, including CLI binary SHA256
`43606467dc2146a3b7b4d2a04703182eac8c6419c8a56107f996aa0b5c36c9fa`.
All eight selected native and FastSecDec library artifacts remain byte-identical
to fullgraph4. The build record identifies the exact accepted working-tree
overlay over `c112d72`, rather than claiming it was already a committed revision.

Artifact, configuration, checkpoint, kernel and native QMC-checkpoint sources
are unchanged from `c3ec42a`. The retained fullgraph artifact and version-three
checkpoint therefore need no migration. Worker count remains excluded from
settings identity. Actual large-artifact resume has not yet run; it must use the
checkpoint's 36,864 saved evaluations rather than the later live status.

The implementation must preserve complete coefficient vectors and covariance,
precision policy, lattice design, package order and successful-submission rules.
It must not promise bitwise identity across arbitrary worker schedules: the
existing replay API explicitly allows observation order to affect precision
decisions.

Bounding live context ownership does not establish a measured memory reduction
or speedup. The round-robin scheduler revisits sectors, so eviction discards
native precision caches and can add repeated cloning and constant conversion.
Those costs stay visible in complete process timings; they are not silently
charged to the sample callback or extrapolated from the two-worker peak RSS.
Scientific test execution was serialized after the Pathfinder comparison.
The next capability observation is the already planned bounded resume using
the preserved artifact and checkpoint; its numerical result remains pending.
