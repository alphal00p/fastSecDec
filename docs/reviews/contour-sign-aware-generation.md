# Sign-aware native generation

This increment selects the approved sign-aware envelope through the existing
`ProgramRecipe::DynamicSignAwareV1` native interface. It shares the polynomial
recipe's full-sector envelope, prepared root helper, local-strength map,
Symbolica Jacobian, subtraction and Laurent pipeline. It does not enable
dynamic production admission, which still requires the independent certified
runtime checker.

Each `SmoothPositivePart` is lowered through the previously tested native
`smooth_positive_v1` callback before Symbolica collects the root coefficients.
Its stable numerical branches implement the same analytic function and its
Symbolica derivative hooks supply the complete higher jets. No solver,
polynomial convolution, differentiation or serialization is added here. The
retained descriptor keeps the helper alive throughout generation and detached
compilation, and every subtraction face restricts the same full-sector function.

The checker source now receives the selected recipe and an unmodified copy of
the full-strength Atom. Diagnostic tagging is intentionally outside the
mathematical density used by symmetry and subtraction. Native staging schema 3
rejects schemas 1 and 2 before importing their native payloads; their older
checker source layouts cannot be reinterpreted. Finished fixed v9 artifact
compatibility is unchanged.

## Exact cubic control

The direct-cube control is

\[
I(\epsilon)=\int_0^1
\frac{x^{-1-\epsilon}}{(1/4-x)(1+x^2)-i0}\,dx.
\]

It also declares the positive residual \(U=1+x^2\) at exponent zero, retaining
the branch bound without changing the density. The cubic F term and the even U
term are both structurally present; the descriptor must retain odd order 3,
even order 2, full dimension 1 and two dense root coefficients even on faces.

The expected Laurent vector follows from the native exact decomposition

\[
\frac1{x(1/4-x)(1+x^2)}
=\frac4x+\frac{64}{17(1/4-x)}
 +\frac{-4x+16}{17(1+x^2)}.
\]

Consequently the pole is \(-4/\epsilon\), and the finite coefficient is

\[
-\frac{64}{17}\log3-\frac2{17}\log2+\frac{4\pi}{17}
+i\frac{64\pi}{17}.
\]

The test verifies the decomposition with Symbolica's `apart`, `together` and
`cancel`, rather than introducing a reference algebra helper. Public API/source
evidence is Symbolica 7ec1be4 `src/collect.rs` (`apart`, line 404) and its native
`apart` unit test (line 1853). The focused FastSecDec test is the executable API
probe. Deterministic midpoint controls use identical coordinates for fixed,
polynomial and sign-aware recipes, in symbolic and numerical-dual generation
with Taylor and IBP subtraction. The complete complex pole and finite part,
including exact offsets, are checked.

The shared-preparation controls now include all four recipes and this cubic F/U
source. They check one geometry/extraction, declaration retention before term
cancellation, and equality of complete Laurent vectors between selected and
shared generation. This does not claim a variance benefit or measured memory
reduction; those require the later runtime and repeated-run gates.

## Acceptance status

Focused native gates passed against the public owner dependencies:

- Generation: 3 tests, including the exact partial-fraction reference and all
  twelve fixed/polynomial/sign-aware mode/strategy combinations (1.60 seconds).
- Streaming: 16 tests, including four shared recipes, old staging rejection,
  and fresh-process compilation/evaluation of polynomial and sign-aware sources
  in both generation modes (0.88 seconds).
- Native recipe/schema/factor association: 10 tests (0.16 seconds).
- Resident native publication: 1 test (0.05 seconds).

The fresh-worker gate caught an omitted Symbolica state export for the new
full-strength and independent coefficient-combiner symbols. The codec now
exports every retained source expression and ordered input before serializing
the native payload; the fresh-process gate passes after that correction.

The first cubic quadrature used materialized coefficient expressions for both
generation modes. That historical gate checked numerical-dual formula
mathematics. The strengthened gate now uses the same `numerical_dual::native::build`
path as kernel compilation, and one complete Laurent-vector evaluator per sector.
It asserts that the requested deferred mode survives and the materialized
coefficient cache remains empty. Native Complex<DoubleFloat> arithmetic resolves
the legitimate cancellation in repeated endpoint subtraction.

The strengthened generation filter passed **5/5 tests in 18.25 seconds**. Besides
the cubic control above, it tests \(x^{-3-\epsilon}/F\): the pole is
\(-60/\epsilon\), and the finite term is

\[
-18-\frac{1024}{17}\log3+\frac2{17}\log2-\frac{4\pi}{17}
+i\frac{1024\pi}{17}.
\]

Symbolica verifies its exact partial fractions and differentiates the causal
primitive; native endpoint evaluation checks the finite reference including
the lower-lip sign. Both generation modes and Taylor/IBP then agree with the
complete vector. The deferred requests retain at least second endpoint
derivatives, and metadata records the correct zero/one faces. Higher local
strength derivatives therefore run through native jets and the implicit root
hooks, without differentiating solver decisions.

The complete core-library gate passed **297 tests with 16 ignored**, no failures,
in 23.83 seconds. Final Clippy/portable checks remain coordinated separately.
These are generation and ownership results, not dynamic production admission
or a demonstrated variance improvement.

## Independent review

The foundation reviewer independently checked the strengthened cubic and
repeated-endpoint controls. The exact decomposition, causal primitive derivative
and explicit boundary phase supply references independent of the quadrature;
the deferred branch executes the actual native vector/jet path without
materialization. No scientific control defect was found.

A separate source review checked the registered schema-3 checker DTO. Its
canonical F/U identities are compared with retained contour metadata; its
namespace is recomputed from the ordered mathematical recipe rather than a
mutable chart index. Full-strength callback arity/helper association and the
independent coefficient input/output layout are checked. The combiner reuses
the native envelope construction. No source/staging ownership or ecosystem-reuse
blocker was found. This structural admission is not the later certified
per-evaluation causal check or saved-checker runtime admission.
