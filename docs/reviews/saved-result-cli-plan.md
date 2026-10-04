# Saved numerical results: CLI implementation plan

This plan was prepared by reading the CLI while the cache/catalogue milestone
was frozen. It proposes no source change during that gate. The native result
owner is described in [the library contract](saved-result-contract-proposal.md);
the CLI will use that owner's validators, comparison, scope and export methods.

## Commands and ownership

Add `--save-result FILE` to the existing `run` and `integrate` commands. Keep
`--output`'s current meaning of a portable symbolic artifact. The result path is
caller-side output steering, excluded from numerical settings and checkpoint
identity. Write the native versioned result envelope atomically after collecting
the final accepted numerical state. Preserve the existing final stdout report.

Add `show-result FILE`, which dispatches directly to `read_result`. It must not
read source cards, load artifacts, initialize a dashboard or construct evaluators.
Plain output shows scope, stop/coverage status, authoritative total, separate
exact offset, selected native sampling design, sector contributions and any
library-derived comparison. Shared-replica marginal errors are labelled; they
are never summed to reconstruct total uncertainty.

Sector viewing should support deterministic ID order and magnitude/error order
for an explicit Laurent `CoefficientKey`. Unknown keys are errors, unavailable
estimates sort last, and IDs break ties. Coordinate a native ordering helper
with the result owner so later HEPKit bindings use the same behavior. Sorting
does not mutate the saved numerical payload. JSON viewing retains the original
typed result and identifies any derived comparison/order as a view; only
`encode_result` writes the persistence envelope.

Add `export-reference FILE --source estimate|stored --output FILE`, requiring an
explicit selection. Delegate to the native result extraction API, then write
the existing native reference envelope. There is no fallback between sources,
automatic validation upgrade or trust flag. Ordinary `--reference` loading
continues to accept reference documents, rather than guessing inside result
documents.

The proposed file changes are:

- `src/main.rs`: command/argument dispatch and explicit nonzero exit after a
  displayed numerical failure; no result validation or conversion implementation.
- `src/results/mod.rs`, with small `storage.rs`/`display.rs` helpers if warranted:
  assemble native result metadata, call native readers/writers/export/view APIs,
  perform atomic file writes and render the numerical view.
- `src/driver/report.rs` and `driver/execution/{qmc,mc}.rs`: retain the existing
  session's native `ContributionReport` when finalizing, together with the typed
  stop reason and effective `QmcDesign`. Collect once after the final accepted
  work package; keep covariance/statistics entirely in the session owner.
- `src/reference.rs`: expose the original `ReferenceResult` and its original
  `ComparisonContext` as a native `StoredReference`, without reconstructing
  either object from displayed comparison rows. Retain file path/digest as
  descriptive provenance without changing the reference's evidence.
- `tests/results.rs` and focused driver tests: exercise the transport and process
  boundaries below. Update CLI documentation with concrete save/view/export
  commands and the distinction between a result, artifact and checkpoint.

## Identity and exact-contribution policy

`IntegrationReport.content_id` and the current session/checkpoint problem ID
are the outer CLI artifact identity. Existing reference comparisons correctly
use the inner `KernelSet::content_id`. The saved native manifest must use the
inner kernel identity, its complete ordered coefficient layout, all sector IDs
and dimensions, and the complete exact coefficient vector. Preserve the outer
artifact identity in provenance; do not change checkpoint identity semantics.

The current CLI integrates every kernel sector and includes all exact offsets,
so it constructs `ResultScope::FullIntegral`. The native selected-sector scope
and explicit `IncludeAll`/`ExcludeAll` exact policy remain visible on imported
records. This slice does not add partial-sector integration steering. Selected
records cannot become full-integral references or comparisons by completing
their own allocation; native scope checks decide what can be exported/compared.

Save the actual final native `QmcDesign`, not the source card's initial counts.
Keep native `ReferenceValidation::Unverified` for newly computed CLI results;
successful allocation or meeting tolerance is not independent evidence. Preserve
any original stored reference and context even when no estimate is available.

## Failure and cancellation outcomes

The current evaluation-failure branches save accepted checkpoint state and then
return an error, losing the final contribution view. The next implementation
must preserve that state in a typed driver outcome. A minimal path is to finalize
the accepted session with `StoppingReason::NumericalFailure(message)`, retain
its native contribution report and diagnostics, and return that report to the
CLI. The CLI saves/displays it and explicitly exits nonzero. No numerical error
may become a successful command exit.

Only completed, accepted packages enter the saved numerical report, just as for
checkpoints and replay maxima. Failed package prefixes remain excluded. An MC
pilot failure retains pilot status and the existing restart-required limitation.
Setup, malformed-input, native validation and filesystem errors remain ordinary
errors; they do not fabricate a numerical record. Cancelled and failed records
may be viewed, but estimate-reference extraction is governed by the native
stop/production/coverage checks. A completed work-limit result can retain a valid
estimate without claiming that the requested tolerance was reached.

## Focused acceptance tests

1. Save a small actual QMC run, then remove its card, artifact and checkpoint.
   A fresh `show-result` process still reads the complete vector, covariance,
   exact offset, accepted coverage, diagnostics and effective catalogue design.
   Result commands are routed before any symbolic loader; malformed versions or
   numeric layouts yield one JSON error document.
2. Save a run whose original stored reference value differs visibly from its
   estimate. After removing the reference file, explicit exports select the
   correct original object. Missing selected sources reject without fallback;
   stored uncertainty/evidence/context survive unchanged. Estimate export stays
   unverified, with `StandardError(0)` preserved as a standard error if present.
3. Retain a cancelled partial run and a controlled numerical-failure run. Both
   remain viewable, preserve accepted work and typed stop reasons, and reject
   estimate-reference export. Numerical failure emits one final JSON report and
   a nonzero exit; no failed sample prefix enters covariance/replay state.
4. Use pure native numerical fixtures for selected scope, pilot-only, exact-only,
   sparse coefficient layout and missing marginal estimates. Verify qualified
   display, explicit exact policy, stable sorting and scope-aware export errors,
   without graph construction, JIT compilation or invented missing zeros.
5. Save a resumed QMC report with an explicit catalogue and changed workers;
   native allocation/settings and numerical data match the session's report.
   Saving does not alter sampling, tolerance, stopping, checkpoint identity or
   the original reference comparison semantics.

Native round-trip, structural validation and statistical invariants belong in
the library's own tests. CLI tests should cover the real process/file/exit
boundaries rather than duplicate those validators. No new integration estimator,
CAS operation, kernel serialization or model conversion is part of this slice.
