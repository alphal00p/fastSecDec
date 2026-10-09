# Smooth positive-part callback foundation

Status: independently reviewed and registered after the recipe-ownership
checkpoint. All eleven callback controls pass in the registered contour gate
(71 tests total). This foundation does not constitute production
dynamic-contour admission.

## Native reuse and numerical construction

The public `Real::hypot` API, Numerica's implementations in
`lib/numerica/src/domains/float.rs` and `float/error.rs`, and focused executable
probes were checked against public owner revision `7ec1be45`. The owner provides
the required stable real norm, including its reviewed tracked-error override.
Symbolica's public `AtomCore`/function APIs and source contain no symbolic
`hypot` or smooth-positive builtin. Its existing `EvaluationInfo` callbacks,
derivative hooks, evaluator construction, native dualization and codecs provide
the necessary extension points. No replacement norm, derivative system, root
solver or expression serializer is introduced.

For positive regularity delta, the callback evaluates
`mu = (t + sqrt(t*t + delta*delta))/2`. The nonnegative branch uses
`t/2 + h/2`. The negative branch uses
`(delta/2) * ((delta/2)/(h/2 - t/2))`, avoiding both cancellation and avoidable
intermediate overflow. These are two numerical evaluations of the same smooth
function. Symbolica receives the exact hooks `mu_t = mu/h`,
`mu_delta = delta/(2*h)`, `h_t = t/h`, and `h_delta = delta/h`; its native
higher-derivative machinery differentiates the mathematical function.

The complex callback uses the analytic square root, preserving holomorphic
continuation near the physical real domain. It never substitutes a Hermitian
norm. The real fast path is admitted only when both imaginary components are
exactly zero, including zero tracked absolute error. Native tracked
`is_fully_zero` alone only tests the centre and is insufficient for that choice.
No global numerical ordering, branch or zero predicate was changed.

Invalid regularity, nonfinite values and a nonpositive/underflowed envelope
produce a native NaN and preserve the first failure in the existing
caller-owned evaluation attempt. Uncertain complex fixed-precision arithmetic
can still overflow its native intermediate squares; that limitation is exposed
for the existing precision-rescue path, rather than erasing imaginary errors or
silently replacing the analytic function. Plain f64 derivatives can underflow
below their representable range; the native multiprecision values remain
available and are explicitly tested. This callback is not a numerical
certificate; optional certified envelope checks remain a separate delivery.

## Isolated tests and independent review

Eleven tests passed in a direct native Rust harness linked to the public owner
library. The harness includes the actual new callback and test sources and a
copy of the existing narrow `Number` adapter to avoid changing the active
workspace while its checkpoint tests run. The same controls now pass in the
registered native core (`target/contour-shared-foundation-tests.log`). They cover:

- Both signs, zero, very small inputs, and values near `1e308`, compared with
  native 192-bit arithmetic.
- Imaginary uncertainty in either argument, real tracked error, and native
  f64, double-double and multiprecision evaluation.
- Nonreal analytic continuation, native cubic jets with variable regularity,
  and exact callback derivative normalization at extreme scales.
- `dmu/ddelta` near `5e-312`, and a positive multiprecision `dmu/dt` whose
  f64 conversion underflows for large negative t.
- Fresh-process evaluator restoration, complete complex SymJIT O2 batches,
  explicit failures and preservation of the first attempt failure.

The runtime agent independently reviewed the formulas, native arithmetic reuse,
uncertainty guards and tests, and requested the additional imaginary-regularity
control, which passes. The root review requested the small-regularity extreme
derivatives, which also pass. No scientific or ecosystem blocker was found in
this foundation; recipe lowering, certified checking, production admission and
sampling-performance measurements are not claimed here.
