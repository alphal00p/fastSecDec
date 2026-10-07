# Batched runtime validation

Date: 2026-10-07. This review covers configurable evaluator batching and its
integration with the existing runtime. The independent native API, numerical
and reuse review is [batched-evaluation-reuse.md](batched-evaluation-reuse.md),
and worker evidence is [batched-integration-workers.md](batched-integration-workers.md).
Other examples and permanent test/gate migration remain deferred.

## Configuration and ownership

`evaluation_batch_size` defaults to 256; runtime TOML overlays and explicit
`--evaluation-batch-size` / `--batch-size` flags obey the existing precedence.
Zero is rejected. Saved result provenance records the effective size; checkpoint
scientific identity excludes it, just as it excludes worker count. The CLI owns
parallel execution. SymJIT O2 matrix calls explicitly disable internal threading.
Eager and multiprecision evaluators retain native scalar fallback inside the
bounded batch adapter; no SIMD claim is made for those backends.

The focused CLI configuration probe passed default, overlay, explicit override,
alias, invalid-zero and checkpoint-identity controls. Existing artifacts load
without regeneration or byte changes. All three methods resumed pre-batching
completed checkpoints with batch size 13 without evaluating new points.

## Scientific comparisons

The actual 30-sector ggHH optimized artifact was evaluated with seed 20261008,
runtime parameters from the example, one and eight workers, and evaluator
batches 1, 7 and 256. Comparisons used a preserved pre-batching release binary.
QMC used 1024 points × 2 shifts; ordinary and discrete Havana used 2048 points ×
8 batches, with their normal pilot work. Every full estimate, full covariance,
sector estimate and stopping reason passed the comparison. The complete total
means, errors and covariance were bit-identical on this host; this is evidence
for these cases, not a cross-platform SIMD guarantee. All final classification,
rescue, replay/check and failure counters matched the scalar baseline exactly.

| Method | Assessed points including pilots | f64 | DoubleFloat | Arb<1000> | Unstable | Native matrix calls at batch 256 |
|---|---:|---:|---:|---:|---:|---:|
| QMC | 61,440 | 98.593750% | 1.406250% | 0% | 0% | 240 |
| Discrete Havana | 114,688 | 98.788888% | 1.211112% | 0% | 0% | 13,264 |
| Ordinary Havana | 491,520 | 99.549154% | 0.450846% | 0% | 0% | 1,920 |

These counts/fractions agree for one and eight workers. Discrete Havana splits
each sampled chunk among its sampled sectors, so individual matrix calls often
contain fewer than 256 points. All nonzero classifications are final-point
classes; speculative primary work remains separately counted as evaluator work.

A real partial QMC checkpoint (3,072 accepted points) and a real partial discrete
Havana checkpoint (32,768 accepted production points) were resumed with worker
count changing 1 → 8 and batch size changing 7 → 256. Their final full means and
covariance agreed with uninterrupted runs. No partial statistical package was
admitted. The independent kernel probe also restores precision/maxima state
mid-stream while changing chunk sizes.

## Runtime and terminal controls

Native real/complex probes cover mixed f64, DoubleFloat, Arb and cutoff rows,
zero weights, runtime inputs, native matrix tails, invalid inputs, persistent
numerical failures, completed error prefixes, maximum-weight chronology,
validated replay, empty/shape rejection and cancellation. Scalar and batched
outputs, precision decisions and final replay states agree. Matrix timing shares
sum exactly to the native measured span. Cutoff-only batches perform no evaluator
work; cancellation does not become a numerical failure.

The portable host probe exercises actual eager real and complex evaluators over
263 rows, two runtime bindings and chunk sizes 1, 7 and 256. All four classes,
weighted decisions, final states and numerical outputs agree with scalar eager
execution; matrix counters correctly remain zero. Cold-loaded artifact bytes
remain unchanged. This establishes host eager compatibility, not actual
Wasm/Pyodide execution.

Actual terminal runs exercised mouse/keyboard interaction and interruption in
discrete Havana with one/eight workers, QMC with eight workers, and ordinary
Havana with eight workers. Cancellation completed in 0.095–0.269 seconds in
these checks. Raw mode, cursor, alternate screen and every enabled mouse mode
were restored, with zero recorded numerical failures.

## Measured batching cost

After the build/probe jobs finished, two ABBA sweeps used the same optimized
artifact and seed. QMC used 4096 × 4 shifts (491,520 assessed points); discrete
Havana used 32768 × 8 batches (360,448 points including pilots). Measurements
are short host observations, not a throughput guarantee. Integration wall time
excludes cold artifact loading; f64 time is amortized across all evaluated rows.

In the same-binary batch-size sweep, mean integration time at size 256 was
1.52–1.65 times faster than size 1 across both methods and worker counts.
The separate pre-change scalar-binary comparison recorded these ranges:

| Method | Workers | Previous scalar integration time | Batch 256 integration time | Previous f64 µs/point | Batch 256 f64 µs/point |
|---|---:|---:|---:|---:|---:|
| QMC | 1 | 2.645–2.725 s | 1.819–1.872 s | 3.707–3.717 | 2.105–2.168 |
| QMC | 8 | 0.429–0.465 s | 0.411–0.482 s | 3.796–4.499 | 2.716–3.330 |
| Discrete Havana | 1 | 6.473–6.928 s | 4.683–4.722 s | 16.692–17.778 | 11.484–11.607 |
| Discrete Havana | 8 | 0.587–1.133 s | 0.744–0.767 s | 9.818–17.834 | 11.050–11.435 |

The eight-worker comparisons overlap and show substantial run-to-run variation;
they do not establish an end-to-end speedup over the previous scalar binary at
these small work sizes. Actual matrix counters establish native batch dispatch:
QMC uses 1,920 matrix invocations for 491,520 primary points; discrete Havana uses
41,775 for 360,448. Performance remains dependent on sector occupancy, runtime
precision routing and host workload. All benchmark runs reported zero failures
and identical precision-class counts within each design.

## Build and coverage limits

Release build, native/portable compilation, strict workspace library/binary
Clippy, formatting and diff checks pass. All 29 existing scientific worker
controls pass unchanged (8 ordinary MC, 5 discrete MC, 16 QMC), run against the
fresh native release library. Independent worker probes compare native sample
and weight streams, adaptation, live prefixes, covariance, transformed QMC
points and complete-work admission across five chunk sizes.

The broader all-target Clippy check remains blocked by pre-existing unmigrated
test initializers: missing `SectorProgram.runtime_parameters`,
`PrecisionReport.class/timings`, and `SavedIntegrationResult.requested_accuracy_target`.
Those fields and stale fixtures already exist in the previous committed state.
The user explicitly deferred permanent test/gate migration; no tests or other
examples were rewritten to bypass that boundary. No complete workspace-suite
pass or actual Wasm execution is claimed.

Reproducible scratch scripts and raw observations remain ignored under
`output/batched-evaluation/` and `output/batched-evaluation-review/`; no benchmark
output, build products or local paths enter portable artifacts.
