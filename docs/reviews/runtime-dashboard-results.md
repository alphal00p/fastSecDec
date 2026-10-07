# Runtime stability and live dashboard acceptance

Validation on 2026-10-07, using the existing D05 s-channel ggHH artifact, native
SymJIT O2 at f64 and native eager evaluators for higher precision. The existing
artifact was deliberately retained to exercise the version-6 backward reader.
Temporary probe sources, PTY recordings, checkpoints and raw outputs remain
under ignored `output/`; no permanent tests or other examples were migrated.

## Build and numerical controls

- `cargo fmt --all --check`, `git diff --check`, and strict CLI Clippy passed.
- `cargo build --release --locked -p fastsecdec-cli` passed.
- `cargo check --locked -p fastsecdec --no-default-features --features portable`
  passed. The available native Rust 1.99 toolchain was used; `nix-shell` is not
  installed on this host.
- A separate portable-host executable passed eager generation, format-7 cold
  round trips, runtime bindings, native 106-bit and 3322-bit evaluation, cutoff
  and replay controls. A complete 4096-point QMC calculation returned the
  analytic complex coefficients `i` and `-i log(3)` with full covariance.
  This does not establish actual Wasm or Pyodide execution.
- Original endpoint power provenance passed all four physical/named-polynomial
  and Taylor/IBP combinations. Mixed powers 2 and 5/2 remained associated with
  their pre-IBP endpoints; historical global profiles were unchanged, and
  power-specific routing survived a cold binary round trip.
- A complex-target control used mean `3+4i`, RMS complex error 0.05 and a separate
  failing pole coefficient. Order-zero targeting succeeded while the historical
  all-component criterion failed; incomplete production could not converge.

The [stack implementation record](runtime-stability-stack.md), independent
[stability review](runtime-stability-review.md) and
[endpoint review](runtime-endpoint-profiles.md) cover distance equality and
neighboring floats, mixed-power normalized budgets, inherited thresholds,
transformed QMC coordinates, weighted maxima, explicit cutoff zeros and persistent
nonfinite failures. A review finding that incoming replay state could lower the
reference maximum was fixed: max 100 merged with max 50 still admits 60 at f64
under the 0.9 rule. Legacy version-5/6 bytes and identities remain unchanged.

The [live observation record](live-integration-observations.md) and independent
[numerical review](runtime-live-numerics.md) cover unequal worker prefixes,
implicit zero draws, sparse sectors, cancellation, common versus independent
QMC shifts, exact offsets and observation-order independence. Old/new validated
runs agreed exactly in means, full covariance and sector estimates across all
four exercised MC/QMC methods with one and eight workers. A historical complete
checkpoint resumed with no new samples under its original validated policy.

## Release dashboard checks

Actual CLI processes ran in 140-column PTYs. Keyboard sorting, order selection
and scrolling preceded cancellation. The following interrupt latencies include
worker shutdown and result/checkpoint reporting:

| Method | Workers | Interrupt | Latency | Terminal restored |
| --- | ---: | --- | ---: | --- |
| Discrete-sector Havana | 1 | Keyboard Ctrl-C | 0.167 s | Yes |
| Discrete-sector Havana | 8 | Keyboard Ctrl-C | 0.214 s | Yes |
| QMC | 8 | SIGINT | 0.079 s | Yes |
| Per-sector Havana | 1 | Keyboard Ctrl-C | 0.067 s | Yes |

Every assessed point had exactly one final precision class. All four runs had
zero numerical failures, zero deliberate cutoff points and zero final Arb points;
DoubleFloat was used when routing or weighted maxima required it. Aborted work
remained outside accepted statistical results. Operational counters included it.
The independent [display audit](runtime-dashboard-display.md) records 40–180
column layouts, rounding/extreme-value controls, cached keyboard redraws and
actual process CPU counter validation.

Cancellation polling does not collect full statistics: native observations,
operational reductions and cumulative prefix copies are all gated by the chosen
status interval. Complete tasks retain their final native prefix once before
worker slots are reused. Boundaries and terminal states publish immediately.

Production-only JSON controls received three distinct live means at approximately
1.0-second intervals before a single production batch had been admitted. The
one-worker previews covered 46,445, 93,860 and 140,555 draws; eight-worker previews
covered 342,230, 681,391 and 1,025,974 draws. Accepted production counts remained
zero throughout, establishing mid-batch publication independently of acceptance.
Status JSON contained no ANSI output.

Runtime-overlay controls also passed on the final executable without sampling.
Scalar artifact → overlay → CLI precedence and canonical parameter alias
precedence were verified in saved settings. Duplicate aliases within an overlay
or point file were rejected before any checkpoint/result write. Policy and
complex target provenance retained all four Laurent components and covariance.

## Complete ggHH allocations

One/eight-worker means, complete covariance and every sector estimate agreed
exactly for each fixed allocation. MC used 4096 production points × 8 batches
plus 98,304 pilot points. QMC used 1024 points × 4 shifts × 30 sectors. Operational
fractions include pilots; no artificial target precision fraction was imposed.

| Method | Workers | Wall | Aggregate worker elapsed | Process CPU | f64 | DoubleFloat | Arb<1000> | Unstable |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Havana | 1 | 2.892 s | 2.860 s | 2.877 s | 98.9403% | 1.0597% | 0.0% | 0.0% |
| Havana | 8 | 0.582 s | 4.069 s | 3.848 s | 98.9403% | 1.0597% | 0.0% | 0.0% |
| QMC | 1 | 2.278 s | 2.198 s | 2.273 s | 98.6051% | 1.3949% | 0.0% | 0.0% |
| QMC | 8 | 0.360 s | 2.578 s | 2.663 s | 98.6051% | 1.3949% | 0.0% | 0.0% |

Havana reached the requested 1% order-zero target with measured relative error
0.3553%. The bounded QMC allocation reached its work limit with relative error
1.1952%, above the requested 0.1%; it was correctly not reported as converged.
These small controls establish runtime/statistical behavior, not a scientific
accuracy guarantee for the README's larger refinements.

| Method | Workers | Integrator overhead | Integrand overhead | Evaluator calls |
| --- | ---: | ---: | ---: | ---: |
| Havana | 1 | 3.66% | 2.20% | 94.14% |
| Havana | 8 | 4.04% | 18.72% | 77.24% |
| QMC | 1 | 1.32% | 9.86% | 88.82% |
| QMC | 8 | 1.68% | 12.73% | 85.59% |

These are instrumented elapsed-span fractions for this short allocation, not
OS CPU fractions or a scaling benchmark. Worker-local evaluator preparation is
included, so small allocations can have substantial integrand setup cost.
Changing the stability mode, cutoff, or accuracy target on resume was rejected
by the checkpoint settings compatibility check before new work. An unchanged
distance-policy checkpoint resumed with a different worker count, reproduced
the complete estimate exactly, and performed zero new evaluations.

## Measurement overhead

Thirty rotated old/new executable trials used a deliberately cheap six-sector
fixture under the same validated numerical policy. Every trial preserved exact
means, full covariance and sector estimates. On longer allocations, added
instrumentation cost approximately 0.130 µs of process CPU per point with one
worker and 0.272 µs with eight workers. This is material for this tiny evaluator:
quiet elapsed time rose 71% and 112%, respectively. The added work includes
native-call clocks and worker bookkeeping; the experiment does not isolate their
individual costs.

The incremental 1 Hz status-JSON observation cost was 0.58% in the stable
single-worker comparison; this benchmark did not measure TTY drawing cost. Eight-worker run-to-run variation was too large to resolve a small
refresh penalty, so no speedup or precise percentage is claimed there. These
figures are not ggHH throughput predictions. The
[observation review](live-integration-observations.md) records allocation sizes,
repetitions, process timing, binary/input identity checks and the remaining
per-point bookkeeping cost. Timing and dashboard instrumentation are not free.

A final presentation-only follow-up widened integration report labels. Native
controls retained byte-identical generation tables at widths 1–140, including
color; the final release was rebuilt after this layout change. Numerical and
observation code remained unchanged from the timed benchmark.
