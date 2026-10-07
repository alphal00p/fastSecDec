# Numerical-dual subtraction and scientific controls

Scope: the explicit `numerical_dual` generation lane, reviewed on 2026-10-07.
The existing symbolic lane remains the default. This review covers the formal
endpoint recipe and independently checks its integration with the mapping,
retained-session, caller-dispatch and native evaluator boundaries. The native
composition extension and jet lowering have separate owner/reuse reviews.

## Native algebra reused

The existing `generation/coefficient_first` implementation already composes
Taylor subtraction and endpoint IBP, exact analytic boundary denominators,
Laurent poles from prefactors, relative series coverage, request limits and
cooperative cancellation. The new subtraction module supplies native opaque
functions `F(eps, t0, ...)` in place of actual regular bodies and calls that
implementation unchanged. It does not construct another subtraction recurrence,
symbolic series engine or numerical finite-difference algorithm.

A focused Symbolica probe established that its series of a native opaque
function produces native `der(...)` expressions with the expected factorials;
coordinate differentiation also produces mixed derivatives. The executable
and source are retained under ignored `output/deferred-sector-map-study/recipe`.
Native integer series depth includes that degree, so the probe explicitly
checked the reported absolute order rather than assuming an exclusive bound.

`subtraction/formal.rs` only decodes these native function/derivative nodes into
requests. A request identifies its source term, epsilon Taylor order, coordinate
multiindex and exact face (variable, zero or one). Its numerical value is a
**normalized multivariate Taylor coefficient**, matching HyperDual. The recipe
multiplies it by the product of epsilon and coordinate factorials wherever the
native node denotes a raw derivative. This distinction is required by IBP and
by mixed higher derivatives. Semantic tuple ordering gives deterministic request
IDs; source symbols are reserved before internal native symbols are allocated.

An opaque function cannot prove an unregulated boundary is zero. A singular
endpoint with zero epsilon slope returns `NeedsExactAdmission`; the pipeline
uses the established exact symbolic chart path. It never diagnoses convergence
or zero from sampled values. Signed infinity maps and nonanalytic epsilon
coefficients likewise retain explicit symbolic admission. Requested generation
mode and actual per-sector route are separate metadata.

## Independent boundary review

- Native monomial geometry and source-polynomial valuations remain exact.
  Polynomial residuals are shifted by their proven monomial valuations before
  inversion; evaluating `P(phi)/t^v` naively on a zero face is not used.
- Source-only native series bounds avoid expanding a large factored numerator's
  full sparse support. Conservative numerator valuations may produce additional
  subtraction/IBP terms. Equivalent integrals need not have equal pointwise IBP
  densities; tests and benchmarks must distinguish those two assertions.
- Native HyperDual/Dualizer lower all requested jets into scalar native IR.
  EvaluatorComposer connects existing native programs. Runtime has the usual
  eager/SymJIT evaluator, not a second FastSecDec IR interpreter or dual-number
  implementation. Runtime inputs and complete complex Laurent layouts remain
  explicit inputs/outputs of that evaluator.
- Retained generation advances one caller-requested unit at a time. Pauses keep
  completed work; errors expose no partial integral. Dispatch validates owner,
  stage and complete coverage, then restores deterministic chart order. The
  library creates no worker pool.
- Zero-dimensional projective charts use exact symbolic admission and are folded
  into the global exact vector. They cannot be emitted as stochastic sectors,
  which the integration API correctly rejects.
- Explicit coefficient inspection restores native expressions lazily. The Python
  selected coefficient getter follows that owner for deferred sectors; passive
  views retain the formal recipe rather than pretending its placeholders are a
  fully materialized integrand. Compilation and artifact serialization do not
  call the expensive materialization accessor.
- Worker progress callbacks only construct transient local snapshots for activity
  text. They do not add local job timings to the master totals. Dispatched dual
  preparation has a combined wall-time measurement; it should not be interpreted
  as a separately measured symbolic substitution phase.

The source-program cache retains native original/jet programs across chart
workers. Cache misses currently hold its mutex during compilation, preventing
duplicate programs but serializing those misses. Performance conclusions must
include evaluator construction, not merely the much smaller mapping phase.
Dense ancestor-closed jet shapes remain bounded and can still be expensive.
No exact zero is inferred from a floating-point evaluation.

## Verification

Four formal-recipe unit controls passed against exact native Symbolica algebra:

1. Taylor and IBP, two mixed singular axes, second coordinate derivatives,
   regulator-pole prefactors, mixed zero faces and IBP upper faces.
2. Epsilon-dependent regular powers `(1+x+y)^(-1-eps)` through both strategies.
3. Explicit unregulated-face admission, cancellation and request limits.
4. Repeatable native roots/aliases and internal-symbol collision avoidance.

The scientific integration suite in `tests/numerical_dual.rs` additionally
covers analytic complex Laurent vectors, shifted singular polynomials, runtime
rebinding and cold binary loading, epsilon-pole fallback, mixed faces, exact
zero, zero-dimensional charts, cooperative pause/resume, reversed caller-thread
completion, and rejected missing/foreign completion sets. The 11 controls passed in 0.45 s against the current local native extension,
using the documented Apple Clang linker. The build/test log is retained at
`output/numerical-dual-study/scientific-tests-final.log`. This final run also
checks explicit native expression materialization against the symbolic lane and
retains `(1+x)^10000` compactly in both lanes.

The ignored CLI benchmark helper compares complete vectors at matched native
source charts (including representative permutations and multiplicities).
When conservative IBP representations differ, an explicit configuration requests
paired native QMC channels. Its difference uncertainty uses the retained full
cross-lane covariance, not an independence assumption or a new accumulator.
Raw vectors, full covariance and failure evidence are saved even if comparison
fails. Pure f64 timing is a separate, warmed, explicitly labelled benchmark
policy; it checks classifications and actual native matrix-call counters.
With `batch_rows=1`, the helper uses the actual `evaluate_weighted` scalar entry
point for both warmup and measurement, requiring zero matrix calls. Larger
batches use `evaluate_weighted_batch`. Reports name that entry point explicitly
and express per-invocation timing distributions per point; they do not equate a
one-row matrix with a scalar evaluator call.
The extended optimized control exercises both entry points and verifies their
native call accounting; it and the fresh-process control pass in 0.03 s in
`output/numerical-dual-study/benchmark-scalar-test.log`.
The helper itself passed a native control in 0.08 s: two equivalent source charts
merged into one symbolic kernel were compared with two deferred kernels, including
chart normalization, paired covariance and timing accounting. The log is
`output/numerical-dual-study/benchmark-helper-test.log`. Full ggHH measurements
are a separate acceptance task and are not inferred from this small control.

The first actual four-artifact invocation exposed a benchmark bootstrap defect:
parsing the point before native artifact restoration registered untyped symbols
that conflicted with saved real runtime inputs. The helper now loads all four
native contexts first, then parses and binds the point. Its report separates
artifact loading, shared point parsing and per-artifact binding, preserving the
sum of loading and binding without charging intervening work to any case. A
fresh-process regression with a real declared runtime input verifies restoration,
the bound analytic vector and this timing accounting. The optimized helper and
both focused controls pass; the regression log is
`output/numerical-dual-study/benchmark-runtime-order-test.log`.

No one-loop reference, reduction or integration algorithm is replaced here.
Existing one-loop master/reduction providers remain the independent reference
boundary; these controls isolate the endpoint algebra and native evaluator
composition required by the new lane.

## Follow-up independent wiring audit

A fresh read of `mapping.rs`, `valuation.rs`, `inspection.rs`, the generation
entry points and their artifact consumers confirmed the following details:

- Singular factors use exact source supports transformed by the integer sector
  matrix. A transformed support row must attain every component of the extracted
  minimum, preserving the constant-residual invariant. Regular numerator bounds
  use one native auxiliary scaling variable and a leading native Series, rather
  than the full mapped numerator or its expanded support.
- `ValuationCache` admits only polynomials in the source coordinates, with other
  indeterminates independent of those coordinates. Native `series_impl` returns
  coordinate-independent coefficients immediately. Its sampled-zero branch is
  only entered for negative coordinate-dependent powers; this admission excludes
  those. The initial `AtomField` disables statistical tests, and the returned
  field is checked again. Hidden exact coefficient cancellation can increase the
  true valuation, so an unresolved lower bound remains conservative. Runtime
  values are never used to infer a power or prove zero. Unsupported recognition,
  reserved-scale collisions and native Series errors return the safe bound zero.
- Negative sector matrices, nonanalytic epsilon coefficients and unregulated
  endpoint admission use the established symbolic chart route. A zero-dimensional
  chart always takes that route, and final assembly moves its exact coefficients
  into the global vector with `kernel_sector=None`. Positive-dimensional constant
  outputs can remain in a deferred sector; this changes variance/work, not the
  integral or covariance rules. The new lane deliberately does not perform the
  symbolic symmetry-merge step.
- Source exponent arithmetic uses native arbitrary-size integers before bounded
  conversion. Mapping powers include the positive measure density and each
  factor's exact exponent. Inspection performs the same shift before taking
  normalized epsilon/coordinate derivatives and substituting faces. Its expensive
  materialization stays behind the explicit cached accessor; the scientific test
  checks its exact equality to the existing symbolic result.
- One-shot, dispatched and retained-session routes use the same `prepare`/`finish`
  functions and generation-owned native caches. One-shot cancellation can stop
  at events and formal request polls. A retained session instead observes the
  request and finishes the current atomic unit before pausing; successful units
  are retained, while an error makes the session terminal and exposes no result.
  Native Series/evaluator calls themselves are not preempted. The caller must
  provide cancellation to its own dispatch executor; no library thread is added.
- Saved evaluators still use the ordinary native scalar-IR binary owner. The
  new requested/observed mode fields are optional human-generation records and
  remain outside mathematical identity. Historical omission is preserved by
  serde rather than inventing prior settings. Existing cold binary replay tests
  exercise rebinding and full vectors for the new evaluator programs.

Two actionable **reporting** findings were sent to the pipeline owner:

1. Serial/retained-session full-expression fallback emitted the internal
   subtraction timing and then a combined coefficient timing that included the
   same subtraction. The dispatched CLI path does not aggregate worker-local
   timings and was unaffected. The owner corrected the nested timing events.
2. The `source_chart_modes` record described all original charts but its builder
   filtered out charts without a kernel index, including exact zero-dimensional
   fallback charts. The artifact owner now retains those charts with their known
   symbolic route and tests both requested modes. The corrected source was
   independently reviewed; identity and numerical results are unchanged.

A further exactness audit found a **pre-existing sampled-zero boundary** in
shared support extraction and the physical sparse monomial-shift collector. The
new lazy inspection collector initially reused the same default. Native
`to_polynomial_in_vars` uses `AtomField::new()`, whose statistical zero tests can
turn an inconclusive numerical result into an absent polynomial coefficient.
A focused native probe demonstrated the issue: the exactly nonzero coefficient
`(1+a)^2-a^2-2*a-1+1/10^30` is lost by the default collector, removing an `x^2`
support term from `c*x^2+x^3`. The configurable native collector with
`statistical_zero_test=false` retains both terms and shifts them exactly.

With coordinated ownership, shared `parametric::polynomial_support` and the
physical signed sparse collector now use the existing native configurable-field
API; the pipeline owner made the same correction in lazy inspection. This
changes zero-proof policy, not the algebra engine. The new support and signed
infinity-map regressions exercise a tiny exact residual hidden by cancellation.
The executable proof is retained in
`output/numerical-dual-study/exact-boundary-probe.log`. No runtime parameter
sampling or tolerance is used for these exact structural decisions.

The exact-support regression passed, as did the signed infinity-map regression
(`exact-support-test.log` and `exact-mapping-test.log` in the same ignored evidence
directory). Owned source passes `rustfmt --check` and whitespace checks. Strict
core Clippy, including the numerical-dual and parametric integration tests,
passed in 23.21 s; strict CLI Clippy including the benchmark helper passed in
20.66 s. Their logs are `core-clippy.log` and `benchmark-clippy.log`. Builds used
the coordinated native dependency overlay and Apple Clang.

The root reviewer subsequently found a fourth evaluator-construction entry point
that this initial wiring audit had missed: direct
`GeneratedIntegral::to_kernel_bytes_with_settings` serialization. The artifact
owner corrected it to call the same `program::build_sector` owner as compilation,
and to derive the real/complex layout from the original deferred factors as well
as the formal coefficients. It now preserves the same cancellation terms and
endpoint profiles. Independent source review accepts this bounded correction;
the artifact owner's executable regression passes real and complex source
polynomials with both Taylor and IBP. In particular, the imaginary coefficient
is hidden inside the original polynomial, where formal recipe inspection alone
cannot detect it. The tests compare analytic complete Laurent vectors and exact
serialized bytes against eager compilation, without constructing JIT evaluators.
Portable execution of that additional control is recorded by its owner.

These checks establish the covered algebra, ownership and artifact paths. They
do not establish the performance of the actual ggHH double box: complete
generation and matched evaluator measurements are recorded separately in the
[completed benchmark](numerical-dual-benchmark.md).
