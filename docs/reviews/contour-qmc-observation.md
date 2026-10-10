# QMC observation and cancellation audit

This review covers the narrow reporting and shutdown changes made after the
first 1000 GeV D05 fixed-QMC run exceeded its cancellation grace period. It does
not change the integrand, deformation, lattice, random streams, work packages,
acceptance of completed samples, or covariance formula.

## Observed failure and preserved evidence

The immutable candidate3 run used the native 74225696 owner graph and final seed
`202610101001`. Its monitor sent coordinator-only SIGINT at native integration
clock 300.082708 s. The process exceeded the 30 s shutdown grace and was killed
after 344.316159 s total wall time. The process group closed, a checkpoint was
retained, and no final result file was published. This is a failed shutdown,
not an accepted final five-minute row or a numerical integrand failure.

The original evidence remains in
`target/contour-d05-1000-runtime/candidate3-final-fixed/fixed-qmc/`. Copies of the
checkpoint, last status, failed monitor result, interrupt record, plan, artifact
manifest and previously accepted readiness result are frozen with SHA256 and
original modification times in
`target/contour-d05-1000-runtime/fixed-qmc-checkpoint-recovery/input-identity.json`.
The raw monitor reports 1,618,456,576 bytes peak aggregate RSS and an empty final
process group. No original evidence is overwritten by recovery.

## Independent source review

In `driver/execution/qmc/queue.rs`, the stopped outcome previously forced the
observation callback once for each remaining worker return. That callback can
reduce the full current statistics and save a checkpoint. Fifty completed or
cancelled returns therefore incurred repeated reporting work during shutdown.

The new condition stops calling that observer after cancellation or failure has
already been established. It still sets the worker stop flag, drains every
issued return, merges diagnostics, submits valid completed native packages,
preserves failures, and joins the scoped worker jobs. The enclosing QMC driver
still writes its final checkpoint and report after the drain. The independent
review found no change to sampling, successful-return admission, or the final
scientific state.

In `integration/qmc/results.rs`, common-shift selection now uses binary search
instead of a linear search through every sector's shift list. The prerequisite
is provided by the existing native owner:
`fastsecdec-qmc/src/statistics/accumulator.rs::shift_estimates` enumerates shifts
in increasing ID order and filters incomplete shifts without reordering them.
The same common rows remain in the same order, including gaps. Their arithmetic,
exact offsets, centering and full covariance calculation are unchanged. This
removes a quadratic lookup without adding an estimator or alternate numerical
implementation. No dependency defect or upstream patch is implicated.

The focused regressions cover eight completed returns held until cancellation,
with no repeated observation while draining, and common shift IDs 2, 3 and 7
submitted in reverse order with unmatched per-sector shifts. They check native
completion, covariance, exact-offset cancellation and checkpoint restoration.
Executable gate results are recorded below.

## Checkpoint-only recovery boundary

The ignored reporter derives the expected `IntegrationProblem` through the
public `KernelResultManifest::integration_problem` API, using the already
accepted current-owner readiness manifest and the original artifact accumulation
identity. It checks physical parameters, deformation, stability, artifact IDs
and final seed against the frozen checkpoint. It then calls only native
`QmcSession::restore`, `diagnostic_observation` and `design`.

There is no evaluator construction, `next_work` call, integration continuation,
or extra sample. Native restoration checks the complete configuration, every
sector's plan, canonical completed packages and corresponding timing records.
Native observation retains the common complete-shift total and covariance;
previous complete production and diagnostic evidence remain separate. Any
recovered estimate is the durable frontier of the failed shutdown, with no claim
that the full allocation completed or that its checkpoint exactly coincides with
the 300-second signal.

The reporter is linked to the immutable candidate3 release graph, before these
observation changes. Its source was independently reviewed by the runtime owner.
Build, input and output identities remain under the ignored recovery directory.
Execution passed in 1.620246 s with 814,743,552 bytes peak RSS and no new samples.
The native restore took 0.484832 s and its observation took 0.178768 s.

The recovered frontier contains 449,466,368 accepted points. Its estimate uses
3,657 complete common shifts across all 30 sectors; individual sectors have
3,657 or 3,658 completed shifts. The full allocation remains incomplete. The
finite coefficient is `69.122720669034 + 23.011511381359245 i`, with standard
errors `1.500870411959` and `1.364195583868`. Its covariance-of-mean entries are
`Var(Re)=2.252611993494`, `Var(Im)=1.861029591045` and
`Cov(Re,Im)=0.051172888191`; the trace is `4.113641584539032`. The full Laurent
vector, all cross-order covariance entries and previous completed round five
remain in the native report. These numbers describe the durable checkpoint of
the failed shutdown, not a replacement successful final run.

## Focused acceptance gates

The runtime owner completed all relevant gates on the four reviewed source/test
files. Their current SHA256 values independently match `source.json` in
`target/contour-d05-1000-runtime/qmc-stop-audit/`.

- CLI queue tests: **5 passed**, including the new deterministic eight-return
  cancellation control; `queue-tests.log`, 0.46 s test runtime.
- Native integration tests: **32 passed**, including the new common-shift
  covariance/restore control; `integration-tests.log`, 0.14 s. The separate
  one-test run in `core-qmc-tests.log` also passed and is a repeated subset.
- Strict workspace/all-target Clippy with `-D warnings`: passed in 20.88 s;
  `clippy.log`. Workspace formatting passed; `fmt.log`.

These are 37 distinct focused tests, with no ignored or failed tests in those
filters. The preceding 917-test workspace and 82-test portable gates apply to
the earlier source; they are not recounted as new-source runs. Source review,
the focused regressions and lint accept this narrow observation optimization.
They do not turn the failed historical run into a successful shutdown or accept
new physical benchmark numbers. The subsequent symbolic-endpoint clarification
also keeps those earlier NumericalDual measurements outside the newly required
construction comparison.
