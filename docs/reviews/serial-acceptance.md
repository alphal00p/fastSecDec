# Serial-mode acceptance matrix

This independent inventory maps the approved `SERIAL_MODE_PLAN.md` to concrete
source controls and validation evidence. The scientific, serial-execution,
consumer and build acceptance controls below pass. The coordinator owns final
publication and goal completion.

## Validation snapshot

The final full workspace run and focused rerun validate **638 distinct tests,
with 28 intentional ignores**. The reviewer counted the full run directly from
`output/serial-validation/workspace-final.log`: 637 passed, one failed and 28
ignored. Its sole failure was test cleanup in `family_preparation`, which
deleted the manifest before its helper resolved the immutable data filename.
The two-line cleanup-order correction passes both tests in `family-final.log`.
The already-passing repeated test is counted only once. All serial-generation,
serial-integration, runtime/exact, process-coordinate, refinement, covariance
and recovery controls pass; no failed scientific control is left outstanding.

Earlier findings are resolved in the full run: relative artifact publication
normalizes an empty parent to `.`, generation-specific data filenames are
observational, expensive corruption validation is explicit, resumed `run`
settings come from the current card, and CLI runtime-parameter parsing waits
for the native model to register its symbols and attributes. Native codecs
remain unchanged by that parsing correction.

The portable host suite records **72 passing tests**. The isolated Python
binding check including `python_stubgen` passes after an explicit `BTreeSet`
type annotation resolves consumer feature-dependent inference; this changes no
algorithm. The final strict workspace Clippy rerun in `clippy-final.log` passes,
as do formatting and diff checks. The optimized CLI build in `release-final.log`
finishes successfully in 3 min 03 s. No actual Wasm/browser execution or rebuilt
Python wheel is claimed for this milestone.

## Required scientific and execution gates

| Approved gate | Concrete coverage | Acceptance status |
| --- | --- | --- |
| All four generation/integration combinations use the same indexed format | CLI `serial_generation::normal_and_streamed_native_processes_preserve_the_complete_laurent_vector`; `serial_integration::both_artifact_generation_modes_support_all_serial_integrators`; `fixed_replica_growth_and_selective_complex_output_match_ordinary_integration`; native indexed writer/reader tests | Passed. Both generated artifact modes run through ordinary QMC/MC and all supported serial integrators, closing normal→normal, normal→serial, serial→normal and serial→serial. |
| Symbolic and numerical-dual, Taylor/IBP, complex Laurent vectors, runtime inputs, exact terms and symmetry | Seven native `generation::streaming` tests; heterogeneous local-output projection test; CLI generation parity with one/two workers; three `serial_exact` tests | Passed, including runtime symbols/mass constraints, exact fallback, HEPKit runtime rebinding, mixed exact/stochastic cancellation and exact-only no-sampling resume. |
| Distinct first coverage, minimum residence, retained winners and same-sector workers | Native tests `first_sweep_is_distinct_and_residence_is_minimum_then_same_sector_can_win`, `first_sweep_honors_residency_claims_between_replicas`, `minimum_residence_waits_for_a_concurrent_final_replica`, `elapsed_resident_can_yield_during_replacement_using_previous_evidence` | Passed. CLI interruption additionally observes accepted work while every resident is below a 60-second minimum, then cancels and resumes with changed residency settings. |
| Immediate complete-replica updates and valid global stopping | Native submit/observation tests; `asynchronous_refinement_can_stop_from_complete_evidence_without_global_barrier`; CLI status-driven interruption/resume | Passed. Accepted evidence and provisional progress stay separate. The explicit update-before-60-second-residence check confirms that publication does not wait for eviction. |
| Both refinement settings for serial and ordinary QMC/MC, including ordinary discrete MC | Native serial coordinate matrix loops QMC/MC × adaptive/nonadaptive × grow/append; native ordinary refinement tests; CLI `ordinary_refinement` and `serial_integration` | Passed. Ordinary CLI tests include QMC, adaptive QMC, MC, adaptive MC and discrete MC under both policies. Catalogue-limit shift growth has passing native-driver controls. |
| Actual duplicate-work regression across concurrent same-sector processes, reordered returns, retries, replacement and resized resume | `process::sampling_tests::actual_process_coordinates_remain_unique_across_replacement_refinement_and_resize`; native `serial::tests::matrix::actual_coordinate_matrix_survives_epochs_reverse_returns_reload_and_worker_resize` | Real OS-process fixture checks coordinate bits for QMC/MC and both policies, including two concurrent workers, retry after confirmed exit, reverse returns and one/three-worker restores. Coordinator reports this fixture passed. |
| Independent native RNG proof and reservation bounds | Independent seed/statistics review; `integration::streams` limit controls; serial identity/refinement counter tests; stale/duplicate-return controls; ordinary frontier integrity tests | Passed. Source/probe audit records period, jump spacing and draw bounds. Ordinary checkpoint integrity and exhaustion controls are included in the completed full run. |
| Full covariance, large cancellation, near-zero and missing/pilot-zero controls | Centered vector moments; complex Laurent covariance matrix; `cross_sector_cancellation_retains_sub_ulp_centered_means_and_saved_epochs`; tiny-variance and normalization-underflow tests | Passed. Regressions prevent rounded sector means or lost nonzero variance fabricating convergence. Identical tiny replicas remain valid; missing/pilot rows remain unavailable. |
| Interrupted writes, corrupt records, disk exhaustion and generation recovery | Native indexed corruption/range/footer/interrupted-write tests; CLI atomic publication and `/dev/full` tests; durable-sector-without-receipt recovery; actual killed-generation-coordinator resume | Passed, including relative-parent publication after correction. `/dev/full` injects real ENOSPC without deliberately exhausting the host disk. The separate family cleanup correction also passes its focused rerun. |
| Process ownership and aggregate RSS scaling | Process residency-slot and inherited-lock controls; selective-read instrumentation; independent four/eight-sector × one/four-worker RSS measurements | Measured source/ownership gate accepted within the documented CLI workload scope. Retain the upstream Symbolica global-resource limitation and 20 ms sampling caveat. |
| Bounded native ggHH after smaller analytic tests | Coordinator's `examples/gghh_double_box` generation/integration campaign | Coordinator reports completed serial numerical-dual/Taylor generation and all-30-sector QMC/MC checks through eps^0. Detailed bounded debug observations follow below; the two estimates agree within 0.76 combined standard deviations and correctly remain unconverged at their work limit. No three-loop run is required. |

## Bounded double-box observations

The coordinator reports these completed runs from its fixed `debug-hardlink-v2`
executable snapshot. The original `gghh_double_box` graph, numerator and
kinematics were retained; generation selected numerical-dual/Taylor through
eps^0 with two processes. The immutable result contains 30 sectors, four shared
formulas and 26 reused uses. Generation took 309.21 s wall time with observed
peak aggregate RSS 492.5 MiB, parent peak 24.98 MiB and at most two children.

Both integration controls use this same artifact, all 30 sectors, two workers,
two complete 1,024-point replicas per sector and 61,440 evaluations each.

| Method | Wall time | eps^0 coefficient | Observed aggregate peak | Parent peak | Stop |
| --- | --- | --- | --- | --- | --- |
| Serial QMC | 13.09 s | i × (355.1494 ± 1.6991) | 154 MiB | 43 MiB | Work limit, unconverged |
| Serial Havana MC | 26.37 s | i × (358.3537 ± 3.8700) | 182 MiB | 69 MiB | Work limit, unconverged |

Their difference is 0.76 combined standard deviations. These are bounded
scientific/execution controls on a debug build, not optimized performance or
one-per-mil convergence claims. The independent analytic sector-count RSS
matrix remains the direct evidence for residency scaling; this larger example
establishes that native generation and both integration paths complete within
the bounded workload.

## Additional interface and persistence checks

The following existing controls complete the mapping beyond the final checklist
in the approved plan:

- `config::tests` preserves omitted `max_rounds` through artifact/overlay
  serialization, ordinary default-one behavior and serial-unlimited provenance;
  validates residence times and rejects serial discrete MC with per-sector
  Havana guidance.
- `artifact::persistence_tests` preserves supported monolithic ordinary readers,
  scientific identity independent of scheduling, explicit saved-representation
  identity, immutable data publication and prior-artifact usability on failure.
- Native indexed tests assert saved program-byte identity, heterogeneous
  real/complex local output scattering, and no reads of unselected sector bytes
  during selected-sector or exact-only setup. CLI inspection tests preserve
  global IDs for a selected nonzero sector and metadata-only defaults.
- Native serial checkpoint tests and the CLI transaction test preserve accepted
  statistics, replay state, diagnostics and reserved streams together. Worker
  count, residence and evaluation batch size may change on resume; mathematical
  and statistical settings may not. The process fixture rejects old run/lease
  returns after restoration.
- Serial budget tests count production/refinement allocations, not visits.
  Zero-tolerance analytic CLI/RSS controls finish with `work limit` and
  `converged=false`, retaining valid estimates.
- New `serial_exact` tests exercise `run --serial`, card-selected serial
  integration after ordinary generation in a disposable setup process, and
  card-selected serial generation/integration. They use HEPKit's shipped bubble
  to check runtime rebindings for exact and sampled cases; a separate mixed
  exact/stochastic cancellation checks that offsets enter exactly once; a
  complex exact-only case checks zero sampling and changed-worker resume.
- Native API and ecosystem review confirms caller-owned synchronous jobs,
  Symbolica/evaluator codecs, HEPKit graph admission, Numerica sampling and
  independent snapshot presentation. No Python dependency enters the default
  core/CLI. See [native API review](serial-native-api-independent.md).

## Final delivery

1. Retain the bounded double-box outcome and independent analytic RSS evidence
   with their measurement and debug-build limitations.
2. Attach final validation to the independent reviews/reuse audit, commit and
   push the validated milestone on `main`, and only then consider completion
   of the active goal against every required gate.

The separate [seed/statistics review](serial-seed-statistics-independent.md)
and [artifact/memory review](serial-artifact-memory-independent.md) retain their
more detailed proofs, source findings and measurement limits. Final delivery
updates must retain the distinction between completed scientific controls,
targeted cleanup verification and publication.
