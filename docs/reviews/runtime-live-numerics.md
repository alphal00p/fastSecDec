# Independent live integration numerics review

Date: 2026-10-07. Reviewer: parameters agent, independently of the integration author.
Scope: `integration/mc_live.rs`, `accuracy.rs`, `qmc/results/live.rs`, the MC worker factoring and the CLI observation/admission boundaries. Accepted with no blocking scientific finding.

## Native ownership and scalar previews

The MC adapter retains Numerica's `StatisticsAccumulator<DoubleFloat>` and public `get_live_estimate`; the pinned native implementation is Symbolica revision `58652fabc2f736302a570deaaf8d517679f7fe6e`, `lib/numerica/src/numerical_integration.rs:148,209,273`. It centers samples at the first observed value to protect small fluctuations around large means. Factoring the former worker-local centered accumulator into this adapter leaves complete-batch means and native training unchanged. Native grids still own sampling, importance weights, RNG and adaptation. The observation callback runs after a completed point and cannot submit an incomplete native return.

The pooling adapter is needed because native accumulators do not expose a sparse-sector, across-task prefix view with implicit unselected zeros. It combines native scalar standard errors rather than introducing a second integration estimator. If selected observations total `k` among `N` global discrete draws, with selected mean `mu`, its marginal standard-error squared is `(SSE_selected + k*(N-k)/N*mu^2)/(N*(N-1))`. The implementation obtains the selected sum of squared deviations from within-task native errors and between-task centered means. The additional term accounts for unselected zero draws without inserting them one at a time. A sector with no selected observations remains explicitly unavailable; one observation has no invented uncertainty. Discrete total uncertainty comes from the full point-vector component samples, not an independence assumption between mutually exclusive sectors. These are scalar live errors; accepted complete-batch estimates retain the full covariance.

The caller is responsible for pooling one immutable proposal epoch. The CLI keeps current prefix snapshots by task identity, replaces cumulative updates, and resets them between native phases. Operational counters are separate and can include discarded work; previews do not become convergence evidence or checkpoint statistics.

## QMC coverage and accuracy

`QmcAccumulator` retains complete validated packages in a `BTreeMap` keyed by canonical point start and reduces them in that order. Native overlap and duplicate validation and complete shift coverage are reused. A live sector mean needs one complete gap-free shifted lattice, and its error needs two shifts. Democratic total previews use common shift identities across all sectors; two or more common shifts reuse the authoritative centered total estimate. Exact offsets are added once. Pending-return observations operate on a cloned statistics session and therefore cannot change accepted coverage, replay state or checkpoints.

Selected Laurent-order stopping uses the magnitude of the full complex total and `sqrt(C_RR + C_II)`, which is the RMS Euclidean error of that complex coefficient. Cross-covariance is retained in results; it does not enter the trace. Every coefficient remains present, and incomplete production cannot satisfy the selected stopping target. The existing all-component target remains available.

## Independent evidence

A separate public-API Rust probe, retained only under ignored `output/runtime-dashboard/live_independent_probe.rs`, passed:

- Native dense `StatisticsAccumulator<DoubleFloat>` comparisons for unequal prefixes of 1, 4 and 17 draws, two correlated complex components, means of order `1e14`, sparse implicit zeros, and reversed snapshot order.
- Exact offsets added once and no change to the accepted MC checkpoint from prefix observations.
- A common single QMC shift gives a mean with unavailable error; duplicate pending returns reject without mutation.
- Reversed complete QMC returns give the same preview and accepted estimate; a negative real/imaginary covariance survives in the accepted full matrix.

The execution log is `output/runtime-dashboard/live-independent-probe.txt`. The author's separate native observation probe and old/new validated-policy controls supplement this evidence; they are not represented here as independent executions. No permanent test files or other examples were changed. Statistical previews remain observations, not certified error bounds, and this review does not claim bitwise policy invariance when caller task ordering or precision routing changes.

## CLI policy, target and result provenance audit

The settings path starts from serialized artifact defaults, restores the explicit validated policy for historical checkpoints lacking a stability field, applies only supplied TOML overlay keys, and then applies explicit CLI controls. Nested objects merge and arrays replace; unknown fields reject. Both integration entry points bind the physical parameters and install the effective stability settings before constructing the integration problem. Target-order presence, positive worker/round limits and the numerical settings are validated before worker execution. Checkpoint identity compares canonical settings except worker count, and replay identity independently admits the physical point and stability policy.

Result assembly records the effective caller settings, operational measurements, actual process CPU time, accuracy target and native complete contribution report. `SavedIntegrationResult::validate` checks that a target-reached claim is supported by the selected native complete estimate. Human output calls incomplete coverage unavailable and keeps accepted coefficients separate from live previews. Full Laurent coefficients and covariance remain in the saved result even when stopping targets one order.

A bounded input finding was sent to the root owner: raw string keys in a TOML parameter overlay can alias the same native Symbol; later collection into a Symbol-keyed map silently drops one, and an explicit CLI override can lose to a differently spelled alias. The suggested correction canonicalizes effective parameter keys before explicit overrides and rejects duplicate aliases within a supplied layer. The owner fixed this by canonicalizing artifact, overlay and point layers before merges, and canonicalizing each explicit flag before insertion. The ordinary fully-qualified ggHH point was unaffected.

The final caller cadence adjustment was also reviewed: MC prefix collection is lazy behind the shared observation gate, with one explicit capture at completed-wave handoff before worker slots are reused. Completed snapshots stay in the observational ledger; aborted prefixes are subsequently removed without native admission. Operational reductions are cadence-gated, and the final report explicitly flushes them. Cheap cancellation/activity polling remains independent of expensive statistic reduction. This changes observation work scheduling only.


The alias fix passed an actual final-release CLI probe using an explicit empty sector selection, so no numerical samples were taken. A complete numeric overlay with the unqualified `p0p1` key was overridden by the explicit qualified `feynkit_graph::p0p1=46000`, leaving exactly one canonical saved key. CLI points, shifts, seed, workers, tolerance, round limit and target order all overrode their overlay values. The saved result retained the effective distance policy, selected accuracy target, all four Laurent components and the full 4-by-4 covariance. Duplicate aliases within either the overlay or point file rejected before any result/checkpoint output was written. Log: `output/runtime-dashboard/alias-review.txt`; the final saved record and rejection messages are alongside it. The review is closed with no unresolved finding.
