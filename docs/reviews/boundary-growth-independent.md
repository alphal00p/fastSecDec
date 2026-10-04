# Independent HEPKit and API review of boundary growth

Reviewer: the native HEPKit/CLI integration agent, separate from the Numerica
agent implementing the diagnostic. The initial review covered the disconnected
`diagnostics/growth/{types,analysis,scan}.rs` draft and
`boundary-growth-proposal.md`. Source review was completed before the raw adapter
was connected; the later execution evidence is distinguished below. A second
independent numerical review is recorded in `boundary-growth-secondary.md`.

## Native reuse

The reviewer independently inspected
`feynkit/crates/gammalooprs/src/utils/fitting.rs`. Its public
`log_log_slope_constant_dropped` requires at least three samples, checks geometric
spacing, takes logarithms of adjacent **value differences**, and fits a linear
regression. Constant and zero adjacent differences are rejected. It is therefore
a different scientific operation from the requested adjacent-pair growth of the
actual physical coefficient. Neither copying nor reimplementing that fitter is
justified here. Native Numerica `Float`/`Real` logarithms and explicit-precision
arithmetic provide the required numerical operations.

The existing raw boundary sampler owns evaluations, endpoint enumeration,
progress/cancellation, precision diagnostics and bounded coverage. The wrapper
reuses it with selected native kernel indices, a distance scale and the remaining
shared budget. No new evaluator, parallel worker pool, graph representation,
symbolic asymptotics or algebra implementation is introduced.

## Scientific and public-data contracts

No actionable scientific or reuse defect was found in the reviewed design:

- Physical components use the existing Laurent-order/real-imaginary layout. A
  large constant cannot hide another component's growth. Training observables do
  not enter classification.
- The ratio uses actual lower/upper endpoint distances, requires each approached
  coordinate to get nearer and keeps spectator coordinates fixed. Native
  logarithmic differences avoid overflowing a ratio of finite values.
- Exact zeros, zero-to-nonzero emergence, failed evaluations, missing values,
  missing probes and invalid distances have distinct outcomes. An absent
  intermediate distance is never bridged to invent a slope.
- Codimension-dependent thresholds remain visible alongside observations.
  Flagging or passing is a sampled policy assessment, not a proof of divergence
  or integrability.
- Retries scale the original distances and share one global probe budget. Every
  attempt remains in the report alongside the latest per-sector assessment.
  Execution stop reasons remain separate from scientific growth flags.
- The typed analysis belongs to the reusable library. Internal callers need no
  string/JSON round trip or CLI-specific state.

The reviewer raised one legacy-format clarification: old serialized reports lack
both coefficient layout and sector metadata, not merely each probe's value
vector. Serde readability cannot reconstruct that missing scientific layout.
With valid caller-supplied layout/sector metadata, old rows can report
`MissingValues`; without it, analysis must reject the report. The connected API
documentation and tests explicitly preserve this distinction.

The review requested focused coverage for failed intermediate observations,
immediate cancellation retaining selected-sector metadata, and a retry exhausting
its budget before reaching later selected sectors. Untouched sectors must remain
inconclusive, with earlier concerns and completed rows retained.

## Connected source and execution evidence

After milestone `a06f01e`, the reviewer inspected the connected adapter in
`diagnostics/boundary.rs` and the report additions in `diagnostics/types.rs`.
`boundaries` delegates to the same original sampling loop. All selected sectors
and the native component layout are recorded before the initial cancellation
callback. Successful complete value vectors are copied, while failed vectors
remain absent. All requested retry coordinates are checked before sampling.
Cancellation unwinding does not call the caller again. No extra symbolic or
numerical evaluation engine appeared during connection.

The author's coordinated run passed **11 boundary-growth tests** in 0.03 s and
**5 existing diagnostic tests** in 0.04 s. The reviewer inspected the test source
and logs `output/boundary-growth-tests.log` and
`output/diagnostics-growth-adapter-tests.log`. The cases cover native MPFR
extreme-range logarithms, physical-component masking, actual upper distances,
known codimension behavior, missing/failed intermediate probes, old values and
missing layouts, collapsed distances, retry crossover, untouched selected sectors
after budget exhaustion, immediate and mid-retry cancellation, invalid scales
before evaluation, and an all-exact integral's `NotApplicable` outcome.

Those results close the focused execution requests above. They establish the
sampled diagnostic contracts, not integrability certification, extrapolation,
phase-two boundary handling or general power-law fitting. CLI presentation is a
subsequent thin adapter over the same typed reports and callbacks.

## CLI adapter gate

The CLI now calls `scan_boundaries` directly. The growth tolerance defaults to
0.5, retry scales default to empty, and progress is serialized from the public
`BoundaryScanProgress` type. The library's `Display` produces a compact attempt
table; terminal colors belong to the CLI and are disabled by `--plain`/`NO_COLOR`.
No numerical policy or new classifier was implemented in presentation code.

Three new CLI process tests passed in 0.16 s: a growth flag keeps exit success
while retries preserve typed history, a genuine numerical range failure emits
one full JSON report and a nonzero exit, and SIGINT retains incomplete coverage
and selected-sector metadata. Four existing CLI process tests also passed in
3.79 s, including all shipped input cards and fresh-process artifact loading.
Logs are `output/boundary-cli-tests.log` and
`output/boundary-cli-existing-tests.log`. An actual six-line artifact preview
(`output/boundary-cli-preview.txt`) completed 120 probes with no failures and
rendered the intended readable table. None of these sampling checks replaces
the independent integral-value or convergence tests.

Pathfinder's independent public-data review found that displaying an imported
attempt index of `usize::MAX` could overflow the human-facing one-based index.
The presentation now uses checked addition and prints `invalid-attempt` for
that malformed value. A pure-data JSON import/format regression passed in
`output/boundary-display-import-test.log`. This changes no scanner state,
numerical assessment or report contents.
