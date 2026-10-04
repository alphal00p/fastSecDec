# Boundary growth and retry proposal

This is a design proposal, not an implemented feature. It addresses the three
remaining Pathfinder boundary regressions at `tests/test_boundary_test.py:63`,
`:109` and `:209`, without adding an asymptotic algebra system or integration
schedule. The existing raw boundary evaluator remains the only numerical
sampling path.

## Reference behavior and limits

The original compares consecutive requested distances within each face pattern.
Its effective growth is `max(0, log(value_near/value_far) /
log(distance_far/distance_near))`, tested against tolerance times the number of
approached coordinates, plus a `1e-6` numerical slack. Training-observable growth
is recorded but does not decide failure. Failed sectors are rescanned at the
original distances multiplied by each configured scale. Its final sector map
replaces earlier results with the latest attempt, although retry summaries are
kept separately.

Two details should be improved in the native version. A maximum across Laurent
components can hide growth in a smaller component behind a large constant; use
each physical numerical component separately. The original artificial `1e-300`
floor and binary64 ratios can invent a finite exponent at a zero or overflow
despite finite input samples; use explicit zero states and native logarithms.
Also use actual endpoint distances from sampled coordinates, particularly near
the rounded upper endpoint. None of these finite samples proves integrability.

## Existing capability audit

The current public `diagnostics::boundaries` already owns bounded subset/side
enumeration, strictly representable interior coordinates, precision rescue,
budget accounting, cancellation and completed-row retention. Its
`BoundaryProbe` currently keeps only the maximum absolute coefficient, which is
insufficient for the componentwise diagnostic. `KernelSet::orders/components`
already provide the numerical layout; there is no reason to parse expressions or
run a second evaluator.

No matching two-distance boundary diagnostic was found in the checked Numerica
numerical-integration code or HEPKit/Symbolica-community integration wrappers.
The broader FeynKit tree does contain GammaLoop's public
`utils::fitting::log_log_slope_constant_dropped` and private IR `fit_power_law`
(`crates/gammalooprs/src/utils/fitting.rs:82` and
`src/integrands/process/ir.rs:2286`). Their source and tests require at least
three geometrically spaced samples, transform adjacent **value differences** to
remove a constant, and perform linear regression. That operation is useful for
GammaLoop's IR profiles, but differs from the required adjacent-pair growth of
the actual physical values and rejects the constant/zero cases treated here.
No part of that fitter should be copied or reimplemented; a future multi-point
fit feature should revisit its native ownership rather than grow another fitter.

Numerica owns the required arithmetic: public `domains::float::Real`
provides `log`, and `Float::with_val`/`RealLike::to_f64` provide explicit-precision
conversion. The MPFR-backed `Float` implementation calls its native logarithm;
it does not need Symbolica Atom evaluation or a workspace. Existing native
floating-point tests exercise these operations. A focused extreme-range test in
the new diagnostic suite will verify the composed log difference before it is
used for scientific classification. No least-squares fitter is needed for a
two-point diagnostic, and no new logarithm, compensated-sum or uncertainty
implementation is justified.

## Proposed public surface

Keep `boundaries(kernels, &BoundaryOptions, callback)` as the raw one-attempt
sampler. Add successful complete `values: Option<Vec<f64>>` to each probe and
the existing order/component layout to its report. Failed evaluations have no
invented value vector. Old serialized rows without vectors may still be read,
but growth analysis reports their missing observations explicitly. No training
weight or adaptive-grid observable enters this physical-data interface.

Add two entry points, re-exported from `fastsecdec::diagnostics`:

```rust
pub fn analyze_boundary_growth(
    samples: &BoundaryReport,
    options: &BoundaryGrowthOptions,
) -> Result<BoundaryGrowthReport, DiagnosticError>;

pub fn scan_boundaries(
    kernels: &mut KernelSet,
    options: &BoundaryScanOptions,
    progress: impl FnMut(&BoundaryScanProgress) -> ControlFlow<()>,
) -> Result<BoundaryScanReport, DiagnosticError>;
```

The pure analysis function lets HEPKit inspect already collected native data.
The scan wrapper reuses the raw sampler internally with explicit selected sector
IDs, distance scale and remaining global budget; it does not create workers.

- `BoundaryGrowthOptions { max_power_per_axis: f64, numerical_slack: f64 }`,
  defaults `0.5` and `1e-6`. Both must be finite and nonnegative. A growth-enabled
  scan needs at least two distinct distances. Thresholds and measured powers
  remain in the report, so these policy choices are visible.
- `BoundaryScanOptions { sampling: BoundaryOptions,
  growth: BoundaryGrowthOptions, retry_scales: Vec<f64> }`. Default retry list is
  empty. Scales are distinct, strictly decreasing and inside `(0,1)`; each is
  relative to the **original** distances, never cumulative. Validate every
  prospective coordinate before starting, including upper-endpoint rounding.
  Invalid scales are typed request errors, not silently skipped attempts.
- `BoundaryGrowthPair` identifies the sector, typed endpoint pattern,
  order/component index, and the two probe indices. It records the actual
  endpoint-distance log ratio, observed magnitudes, effective power, threshold
  and typed assessment. Zero-to-nonzero emergence and failed/missing evaluations
  are explicitly inconclusive; they do not produce infinity or an invented
  finite power. Both-zero and decreasing magnitudes have zero growth.
  The estimate enum distinguishes `Power(f64)`, `BothZero`, `DecreasesToZero`,
  `EmergesFromZero`, and `Unavailable(cause)`; causes distinguish missing values,
  failed evaluation, invalid/collapsed distances and lost numerical range.
- `BoundaryGrowthReport` contains the pair rows and per-sector assessments,
  including incomplete/missing pairs. Matching is by the typed sector/face key,
  not a formatted label. It compares adjacent requested distances after sorting,
  and does not bridge a failed/missing intermediate sample.
- `BoundaryScanAttempt` retains its scale, selected sectors, complete raw report
  and growth report. `BoundaryScanReport` retains **all** attempts, aggregate
  evaluation diagnostics, global probe counts, stop reason, and latest
  per-sector assessment. It exposes whether earlier attempts were flagged,
  separately from the latest assessment. A later acceptable scan never erases
  an earlier failure or turns the overall report into an integrability proof.
- `BoundaryScanProgress` adds attempt index/scale to the existing bounded raw
  progress event. Immediate and mid-attempt cancellation preserve all completed
  rows. Existing `DiagnosticStop` still describes execution, independently of
  scientific growth flags.

Use small modules for sampling, growth analysis, retry orchestration and their
types. Existing benchmark behavior and public callback contracts need not change.
The CLI can later expose only a tolerance and retry scales; richer component/
pair data remain in the reusable library report and JSON/status stream.

## Numerical and coverage semantics

For a face approaching `k` endpoints, let `delta_far,j` and `delta_near,j` be the
actual distances to the specified lower/upper endpoints. Compute

```text
log_distance_ratio = sum_j(log(delta_far,j) - log(delta_near,j)) / k
power = max(0, (log(abs(value_near)) - log(abs(value_far))) / log_distance_ratio)
threshold = k * max_power_per_axis + numerical_slack
```

Evaluate these few arithmetic operations with native Numerica `Float` at 128
bits, then check the reported binary64 result for finiteness. Log differences
avoid forming either potentially overflowing ratio. The geometric-mean endpoint
distance makes the result account for lower/upper rounding while retaining the
reference's codimension convention. Require every approached actual distance to
decrease strictly; collapsed or invalid distance pairs are inconclusive.

Retain real and imaginary components separately, as in the native kernel and
covariance layouts. Growth is clipped at zero only for the classification;
observed magnitudes and distances remain available for interpretation. Exact-zero
components need no magnitude floor. A zero farther observation followed by a
nonzero nearer observation cannot establish a power law and is inconclusive.

Only sectors flagged or inconclusive after a **complete** attempt are retried.
All samples, including retries, consume `sampling.max_probes`; the budget never
resets. Cancellation or exhausted coverage ends scanning instead of certifying
the partially scanned latest attempt. A passing finite family cannot compensate
for an unavailable family or component. An integral with no stochastic sectors
gets an explicit not-applicable assessment rather than a fictitious successful
growth fit.

## Meaningful validation before connecting the CLI

1. Generate an integrable `x^(-3/4)` density retaining a second spectator axis.
   At the same distances its active one-axis approach has power `3/4` and exceeds
   tolerance `1/2`; its two-axis approach has the same power and a threshold of
   one. This directly checks the original codimension rule with a known law.
2. Use the complete vector `[10^12, x^(-3/4)]` to show that growth of a small
   Laurent component is detected even when the maximum vector norm is constant.
   There is no training observable in the classification API, so a training-only
   growth signal cannot change the result.
3. Check bounded, logarithmic and zero components, including a zero-to-nonzero
   crossing reported as inconclusive. Use upper/lower mixed faces to verify the
   actual sampled endpoint distances, rather than nominal decimal labels.
4. Use the bounded function `(x + 10^(-7))^(-3/4)` at original distances
   `10^(-4),10^(-6)`: its measured growth exceeds `1/2`. At scale `10^(-2)` the
   distances are `10^(-6),10^(-8)` and growth falls below `1/2`. Preserve both
   attempts, the original flag and the latest acceptable observation. Verify
   retry selection from sector assessments and that scales are not cumulative.
5. Cancel within a retry and exhaust the shared budget during a retry. Earlier
   failures, completed rows and total evaluation counters must survive, with no
   false complete/acceptable status.
6. Check the numerical analysis with finite magnitudes `10^(-300),10^300` and
   known distance ratio, which would overflow a naive value ratio. Also reject
   malformed layouts/nonfinite values and unrepresentable retry coordinates
   before evaluating a kernel. These are numerical/data contract tests, not
   snapshots of formatting or private helper calls.

The proposal intentionally does not implement general fits, extrapolation,
automated integrability certification, training diagnostics, contour handling or
symbolic asymptotics. Any additional primitive discovered upstream should be
used instead of duplicating it during implementation.

## Connected implementation and first validation

The library exports `analyze_boundary_growth` and `scan_boundaries` from
`diagnostics`. The raw `boundaries` signature remains unchanged. Raw reports now
carry full physical vectors, the Laurent order/component layout, the applied
scale and every selected sector's dimension, including sectors never reached
before cancellation or budget exhaustion. Serde defaults keep older raw JSON
readable; a report missing its layout/sector metadata cannot be analyzed until a
caller supplies that metadata. Old rows without vectors then receive the typed
`MissingValues` outcome instead of reconstructing components from their norm.

The focused connected suite passed **11 tests** in
`output/boundary-growth-tests.log`; the existing diagnostic suite passed **5
tests** in `output/diagnostics-growth-adapter-tests.log`. This includes native
O2 kernels for the two-component masked-growth fixture, the integrable fractional
power with codimension-dependent policy, the bounded crossover resolved by a
scaled retry, two non-equivalent sectors with a retry budget ending before the
second sector's first observation, immediate and mid-retry cancellation, and an
all-exact problem. Six pure numerical tests additionally cover actual rounded
upper distances, real/imaginary layout, extreme finite value ranges, zero and
emergence states, missing/failed intermediate observations, invalid actual
separation and unavailable legacy metadata. Failed intermediate observations
produce adjacent unavailable pairs, never a fitted bridge.

Independent source/public-contract reviews preceded connection; the connected raw
adapter and executable evidence are now with both reviewers. No CLI retry controls
or default policy changes are included in this library slice.
