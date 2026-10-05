# Native individual-sample latency: bounded diagnostic

The subsequent independently reviewed execution passed both cases; see
[observed results and limitations](native-sample-latency-results.md). The
prespecified design below is retained as the preparation record.

This is source-only preparation following the approved eight-core baseline.
No new production profiler, sampling rule, estimator or default is proposed.
The measured baseline retains package-amortized worker means; it does not
measure individual-sample maxima. The diagnostic below supplies that separate
observable without relabelling earlier evidence.

Use the exact already-validated triangle and box artifacts from
`eight-core-preparation-20261005`, their frozen outer bytes, inner identity and
build evidence. A fresh frozen-CLI inspect validates the transport before an
ignored Rust caller loads the same inner native KernelSet, bound to the exact
inspected bytes. There is no independent artifact validator or symbolic
regeneration. Retain the outer/inner identities and actual linked rlib hashes.

The caller uses existing `QmcSession`, `QmcWorker::evaluate_weighted` and
`WeightedEvaluationContext::evaluate_weighted`, with the same Kuo33002,
Korobov3, N1024, R16, package1024, seed20261211 and eight numeric worker contexts.
Native QMC owns points, transforms, accumulation and uncertainty. A deterministic
batch/task assignment and ordered submission hold accepted replay history fixed
between instrumentation-off and instrumentation-on runs. The caller must merge
candidate replay maxima only after successful native submit.

For the instrumented run, place an `Instant` immediately before the weighted
kernel call and sample elapsed immediately after its return. Updating counters,
copying points, recording precision diagnostics and serialization happen after
the bracket. Thus this observable includes whole-vector evaluator work,
conditioning, MPFR rescue and extra weighted replay, but excludes lattice point
generation, Korobov transformation, native partial accumulation, task scheduling
and output serialization. One timing observation is one full output-vector
request, not one Laurent coefficient. The existing inclusive worker interval
remains separately available and must not be substituted for this bracket.

Each sector retains count, checked duration sum and observed maximum, with
maximum provenance: serialized native task, worker slot, attempt number, flat
point index, shift/lattice index, transformed point, weight, output vector and
native precision/replay report or error. Flat-index interpretation follows the
native plan's documented/source-checked shift-major layout; it is descriptive
metadata only and never regenerates a point. The actual randomized shifts are
retained in the native checkpoint. No per-sample input/output dump is required.

Attempted package timings are local until submission. On a callback or submit
failure, retain the measured prefix in a separately labelled rejected-attempt
collection, discard its candidate replay state, preserve accepted native
coverage and exit as failed. Accepted versus rejected counts must never be
combined into an accepted-sample mean. A separate explicit rejection control
injects a caller error after a known finite prefix, proves accepted state and
coverage unchanged, then retries from the accepted state through native APIs.
This control is excluded from latency summaries.

Before the paired runs, record an empty consecutive-clock-bracket control and
its raw duration distribution. Report count/min/median/mean/max overhead, with
no subtraction from observed sample maxima. The on/off proof compares complete
native vectors and covariance, common-shift means, effective design, coverage,
diagnostics and accepted replay state. Native checkpoints are restored/checked
through native APIs; elapsed worker costs are observational and need not match.
No estimates from the two runs are pooled. A mismatch remains a failed control,
not a reason to loosen numerical tolerance or discard an inconvenient run.

The reported maximum is the largest observed bracket for this finite sample
set, including scheduler interruptions; it is not a worst-case bound. Timer
overhead and instrumented throughput remain disclosed. Full-vector precision
tails and boundary rescues stay enabled. A later reference-side singleton or
batch probe has a different boundary and cannot silently become a comparable
individual-sample maximum.

Execution requires the coordinator's quiet runtime slot and independent review
of the ignored Rust source, controls, frozen input binding and wrapper. This
document does not authorize a runtime during the active external reference
attempt, and records no new latency measurement.
