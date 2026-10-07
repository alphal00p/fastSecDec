# Accepted-estimate dashboard audit

Date: 2026-10-07. Read-only numerical investigation; no sampler, estimator,
admission, checkpoint or stopping policy changed.

## Finding

The dashboard reads the native `ContributionReport::total` correctly. This is
copied from `IntegrationSnapshot::estimate`, without extra filtering. The
generic word “unavailable” obscures several intentional states:

| Method/stage | Evidence required for a native accepted mean and error |
| --- | --- |
| Pilot, any adaptive method | No production estimate: pilot work only trains the grid/allocation. |
| Discrete Havana production | At least two complete independent global batches. |
| Ordinary Havana production | At least two complete independent batches for every selected sector. |
| Democratic QMC production | At least two complete shifted lattices with the same shift IDs across every selected sector. |
| Adaptive QMC production | At least two complete independent shifted lattices per selected sector. |
| Exact-only problem | Exact total is immediately available without stochastic work. |

These requirements come from `integration/mc/session/results.rs`,
`integration/mc_discrete/session/observation.rs`, `integration/qmc/results.rs`
and the native `QmcEstimate::from_shift_means` reducer. The failure-safe
`integration/observation.rs::observed_estimate` preserves the reason as
`PilotOnly`, `WaitingForCoverage` or `StatisticalFailure`, rather than inventing
a result. Statistical failures must remain distinguishable from ordinary
waiting states.

The current CLI admits completed MC tasks in deterministic order after a whole
worker wave returns. With eight workers and eight global production batches,
there is one wave: accepted counts stay zero while every worker is inside its
batch, even though the independent point-level preview grows. At wave completion
the whole allocation becomes accepted at once. The dashboard must not turn
those prefixes into accepted results to make this column populate earlier.

Accepted complete batches or complete lattices can provide a native estimate
before the full planned allocation finishes. Such estimates explicitly carry
`production_complete=false`; the accuracy API still requires completed
production before convergence. “Accepted” therefore means accepted complete
work units, not necessarily a finished production allocation.

## Existing executable evidence

Read-only analysis of the previously validated release JSON status streams
confirms the source-level explanation:

- `output/live-integration/overhead-final-release-long/new-1hz-w1-r1.status.jsonl`:
  at 1.041 s, 3,145,728 of 8,388,608 production points were accepted and the
  accepted estimate was available with `production_complete=false`; at 2.622 s
  all points were accepted and the flag became true.
- `output/live-integration/overhead-final-release-long/new-1hz-w8-r1.status.jsonl`:
  at 1.029–4.096 s, the preview grew from 13,613,717 to 56,556,091 points while
  accepted points remained zero. At 4.940 s, all 67,108,864 points were accepted
  and the complete estimate appeared. This is the expected single-wave case.
- `output/runtime-dashboard/final-status-discrete-w8.jsonl`: ggHH completed its
  three pilot passes, then entered production at 0.514 s. At 3.581 s there were
  1,025,974 preview points but no completed production batch; cancellation
  discarded those prefixes. `WaitingForCoverage` was correct throughout.

The earlier old/new numerical control established identical accepted means,
full covariance and sector estimates for these fixed work allocations. This
audit reused their existing output; it did not rerun integration or manufacture
new native observations.

## Refinement and smallest presentation correction

After a completed allocation misses its target, the CLI constructs a fresh
session for the next refinement round. Discrete/adaptive MC starts a new pilot;
QMC constructs a larger fresh design. The preceding estimate is intentionally
not an input to the next round's estimator. The dashboard currently replaces
its cached observation immediately, so the completed value can be visible only
briefly before the new round has no accepted production coverage again.

The smallest correction is to display the existing native reason: training
pilot, waiting for completed production batches/lattices, or statistical
failure. An optional CLI-only historical display may retain the last native
estimate with `production_complete=true` and `stage=Production`, explicitly
labeled “Previous completed allocation”. It must retain its method, coefficient
layout and result scope; clear it when integration begins; and never present it
as current accepted work, merge it with the new preview, use it for convergence,
or insert it into checkpoints. A current partial-allocation accepted estimate
must remain labeled as current evidence rather than being silently promoted to
the historical completed-allocation slot.

No native numerical defect was found. The UI owner was given these findings and
the cache provenance contract.

## Independent presentation review

The implemented CLI `CompletedAllocation` record is accepted. It stores only a
native Production estimate with `production_complete=true`, resets at
`begin_integration`, and checks method, result scope, ordered sector IDs,
coefficient orders and components before reuse. A current accepted estimate,
including one from a partially completed allocation, takes precedence. Exact
results use their current exact total. A current statistical failure or
provisional numerical-range failure suppresses historical substitution.

The native observation is cloned into the display cache without mutation; the
historical value never enters JSON numerical observations, session state,
checkpoint storage or the accuracy API. “Previous allocation” is explicitly
separate from the current preview. Pilot and insufficient batch/shift coverage
have distinct waiting labels. Each MC/QMC driver already clears its provisional
view at a fresh round boundary before publishing the next native observation,
so retaining the historical column cannot leave a stale prior-round preview.

The focused native TestBackend probe in `output/dashboard-accepted/probe.txt`
passed complete-only caching, current partial-total precedence, compatibility
isolation, failed-range suppression and observation immutability. Captures at
80, 120 and 160 columns cover pilot, waiting batches, waiting shifts, previous
allocation and failure displays. Source inspection independently confirmed the
same conditions; no numerical-core changes were needed.
