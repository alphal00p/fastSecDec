# Independent CLI runtime review

Date: 2026-10-04. Reviewer: the Numerica/runtime agent, separately from the CLI
author. Scope: `driver.rs`, input provenance, artifact validation, cancellation,
refinement, and numerical diagnostics. CLI implementation changes were made by
its author; the reviewer supplied the shared typed diagnostics structure.

## Findings addressed during review

1. **Adaptive production must grow after the lattice cap.** The first driver
   doubled only pilot shifts above the bundled `2^20`-point limit; its fixed
   production budget could leave the frozen production allocation unchanged.
   `adaptive_budget` now scales both production seconds and minimum production
   shifts with the post-cap refinement factor. All production sectors remain
   covered, while pilot observations remain excluded from the estimator.
2. **Source identity must survive changing the working directory.** Recorded
   relative paths could make a legitimate `run --resume` fail even when the
   caller supplied the same absolute run-card path. Input loading now records
   canonical source paths. Byte hashes still reject changed cards, DOT/model
   data, parameter cards, and direct polynomial files.
3. **Numerical precision reports were discarded.** Workers now call
   `evaluate_with_diagnostics` and aggregate the shared
   `status::EvaluationDiagnostics` per package. Snapshots optionally include
   evaluation/check/rescue/failure counters and the maximum precision used;
   CLI checkpoints retain these cumulative counters. Boundary probes retain
   their individual conditioning/rescue/precision result. These diagnostics
   do not alter the worker closure API or enter statistical observations.

The root review had already identified and the author fixed: preserving the
refinement round in a checkpoint; permitting only worker-count changes on
resume; preserving an available partial estimate when cancelled; and avoiding
the unsupported Havana pilot checkpoint path. A cancelled MC pilot now states
that the pilot must restart, while frozen production is resumable.

## Contract checks

- Every submitted result is reduced by the library's canonical package/batch
  identities. Worker count affects execution, not package boundaries. The CLI
  neither merges incomplete replica means nor computes a separate error bar.
- Democratic QMC requires common completed shifts across every stochastic
  sector. Adaptive production uses disjoint frozen streams and excludes pilot
  samples. MC production keeps its grid frozen and estimates covariance from
  independent equal-sized batches. The public runtime test suites separately
  cover these mathematical contracts.
- Accuracy stopping delegates to `VectorEstimate::meets`, which requires full
  planned production coverage. A partial estimate can be displayed on
  cancellation but is not labelled converged. Whole-exact problems have valid
  zero uncertainty immediately.
- Each refinement round constructs a fresh session. Earlier estimates are not
  averaged into the new result. A completed checkpoint resumes at its recorded
  round without repeating that completed allocation.
- A checkpoint binds the complete artifact identity and integration settings;
  the nested library checkpoint additionally validates complete generated
  coefficient/component identity, rule/stream allocation, and accepted work
  layout. In-flight work is reissued by the library and already accepted work
  cannot be accepted twice.
- Portable artifacts bind source provenance, dependency revisions/patch state,
  precision policy, and canonical kernel expressions. Loading validates both
  artifact and kernel identity before evaluation. `integrate` intentionally
  consumes the saved artifact; `run --resume` additionally checks current
  source files. Worker overrides do not change the mathematical artifact.
- SIGINT and terminal cancellation are observed at package/batch boundaries.
  A second SIGINT may terminate immediately. No strict latency during one
  kernel call, native symbolic operation, or compilation is promised.

## Validation and remaining boundaries

The author maintains CLI unit tests for capped lattice growth, adaptive budget
growth, completed resume without repeated work, partial cancellation,
nonresumable MC pilot cancellation, and changed-setting rejection. Process
tests cover JSON output, portable integration/resume, and SIGINT checkpointing.
The final shared workspace gate records execution after the changes above.

The reviewer also added `tests/complex_kernel.rs`, which exposed a separate
scientific artifact roundtrip defect: a native complex coefficient could lose
parentheses in its canonical text and change `(2+3i)*x` into `2+3i*x`. Its fix
and regression belong to the kernel/Symbolica work; this CLI identity review
does not treat hashing an ambiguous textual representation as sufficient.

Adaptive cost estimates are measured worker time, so allocation recommendations
can differ with machine load or worker count before production is frozen.
Checkpointed frozen production remains reproducible. Cumulative numerical
diagnostics may include pilot work and prior refinement rounds; snapshot point
counts describe the current session. Timings and finite boundary probes are
diagnostics, not a proof of numerical conditioning or performance parity.
