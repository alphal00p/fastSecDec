# Native dual evaluation of the original oracle — independent source review

This is source-only feasibility work following the retained 180- and 600-second
exact-Atom oracle timeouts. No alternative adapter, symbolic execution or
dependency patch was introduced by this review. The named candidate's
coefficients and pole orders must not supply
an independent oracle's input or admission certificate. The corresponding
[native API survey](native-original-oracle-numeric-series-audit.md) remains
explicit about its unexecuted scope.

## Existing native owners

Symbolica `src/evaluate/dual.rs:538` implements `Dualizer` over exact
`Complex<Rational>` evaluator coefficients. It generates nonnegative Taylor
components using native symbolic derivatives and the native dual shape;
factorial conventions are handled inside this implementation. The resulting
ordinary evaluator can use native MPFR coefficients/evaluation. This differs
from assuming that every external function directly registers an evaluator for
`HyperDual<Float>`.

The existing `automatic_mixed_derivatives_and_multiple_arguments` test includes
Gamma and polygamma at the regular center 0.4, as well as other special
functions. Gamma and polygamma register real and complex MPFR evaluators in
`src/transcendental.rs:400` and `:489`. Missing symbolic derivatives produce a
typed vectorization error; retained sub-evaluator bodies are rejected by
`ExpressionEvaluator::vectorize` at `src/evaluate/dual.rs:127`. These are existing
contracts, not grounds to invent a second special-function implementation.

Dual evaluation does not provide Laurent inversion. Numerica
`src/domains/dual.rs:1975` starts inversion with the scalar component's inverse.
Symbolica's vectorized inverse does the same at `src/evaluate/dual.rs:960`;
logarithms and general powers also require a regular scalar base. An expression
that becomes analytic only after cancelling separately evaluated poles remains
unsuitable for this graph. Finiteness of final outputs is not proof that such
intermediate operations were valid.

## Gamma regularization is already native, but not automatic in Dualizer

`Symbol::get_series_function()` is public (`src/atom.rs:1712`). Gamma's native
hook (`src/transcendental.rs:379`) returns singular and regularized factors using
its existing recurrence at a nonpositive integer center. Ordinary native Series
invokes this hook; Dualizer instead differentiates Gamma into Gamma/polygamma
at the evaluation center. Thus an unresolved Gamma pole is not handled merely
by requesting more dual components.

The hook can be considered for exactly represented finite polynomial arguments,
but its reconstruction via `Series::to_atom` must equal the original argument:
feeding a truncated general argument would silently change the expression.
Nor is the name "regularized" by itself a certificate of a safe evaluator
graph. The polygamma hook's product can still contain cancelling inverse factors
before exact normalization. No custom Gamma recurrence is needed or proposed.

For the actual captured original density the retained mapped prefactor is
Gamma(4+3 epsilon), regular at zero. Its U/F bases and prescribed coordinates
are positive; this should avoid any need for a Gamma-pole transformation in
this particular diagnostic. Actual imported-expression admission remains
necessary rather than inferred from a convenient nearby example.

## Independent pole bound and structural admission

The original Taylor source appends at most one affine endpoint denominator per
singular coordinate in each branch (`generation/subtraction.rs`). The captured
mapped powers have six such axes. Coordinate differentiation changes regular
polynomial/residual factors and their exponents, not the regulator, and the
common Gamma factor is regular. This supports a conservative branch pole bound
of six from the original source construction, independently of the global
native leading order or the candidate's output.

The bound alone does not complete an oracle. A prospective exact native factor
collection and multiplication by epsilon^6 must preserve the original
expression and remove all intermediate pole operations. Admission must reject
unrecognized structure and require, at minimum:

* no remaining integration coordinates after exact prescribed substitution;
* every inverse base nonzero at epsilon zero, using exact native facts;
* positive exact bases at zero for real regulator-dependent powers/logarithms;
* regular Gamma/polygamma centers and supported native function derivatives;
* no remaining epsilon pole hidden inside a sum, opaque function or denominator.

If native factoring cannot establish this within the diagnostic's bounds, it is
an unavailable route. Global cancellation, sampled finiteness or an approximate
leading zero cannot replace this admission. No general custom valuation engine
or rational-function simplifier is proposed.

With a proved pole bound B and analytic regularized expression, native Taylor
components through B plus the requested highest order cover the corresponding
full Laurent layout. Retain every order from −B through that maximum, including
zeros, and compare the entire union. This analyticity/Taylor argument is distinct
from a native Laurent remainder certificate; do not label a dual component
shape as a `Series::absolute_order` result.

## Proof required before any actual-input run

First require small exact comparisons against the unchanged native Laurent
route: rational poles of different orders, exact leading cancellation, tiny
nonzero leading coefficients, regular and singular Gamma controls, positive-base
regulated powers, negative requested maxima, complex constants, and deliberately
unremoved intermediate poles. Exercise native missing-derivative/body rejection.
The complete nonempty vectors must agree at independently repeated 512/1024-bit
precision; a failed admission must remain a failure. Use native dual component
semantics without caller factorials, coefficient convolution or tolerance-based
pole trimming. Only a subsequently reviewed source-bound original-expression
adaptation could become another oracle. None is accepted by this source review.

## Disconnected small source review

The subsequent `output/probes/native_pole_dual_small.rs` draft is accepted at
source level only. Six original branch models include an independently known
polynomial (also with a negative requested maximum), the actual regular-Gamma
and positive-power shape, complex logarithmic terms, exact leading cancellation
and a tiny nonzero leading coefficient. Native factor collection, monomial
reversal, structural admission and scalar-first dual components precede
512/1024-bit comparison to ordinary native Series of each original model.
Seven grammar rejections, two source-bound failures and exact literal zero are
explicit controls. No candidate or actual-capture file enters these models.

Independent review found and fixed a nonvacuity gap before execution: the usual
unit-floor tolerance could have accepted a zero in place of the roughly
10^−80 leading coefficient. That row now asserts exact and both-precision
nonzero values and compares relative to the tiny reference without a unit
floor. The complex control separately requires a nonzero imaginary coefficient.
Compilation and concrete frozen execution remain separate gates; source
acceptance does not establish an actual-input oracle.

Concrete preflight passes all 19 immutable files under
`output/diagnostics/native-pole-dual-small-1`. The compiled source has SHA-256
`386ce084e431154e0281eac4d1e7ecf51c0049dd94c5478001bf30b42f2db2c4`,
and the probe has SHA-256
`392ed5e30ab652b964f36ffb12eeab02154a4ada89beabac91e56a974317560d`.
Its fresh-output wrapper allows one 180-second process with five-second grace,
30 GiB address-space cap and CPU8 affinity, verifies frozen inputs before and
after, and has no follow-on stage. The linked native snapshot matches the
original oracle's frozen dependency source/build. Execution outcome remains
separate from this preflight.

The frozen small execution now passes independently: exit zero, no timeout,
7.721823169 seconds and peak child RSS 9,216 KiB. All 19 immutable postchecks
pass. Six cases retain 40 full signed-order rows, with native dual and original
Series agreement independently at 512/1024 bits and between precisions. The
known polynomial checks Taylor rather than raw-derivative normalization; exact
leading cancellation, relative agreement for the 10^−80 nonzero leading term,
nonzero complex output and the negative requested maximum all pass. Seven
analytic-grammar rejections, two source-bound rejections and the separate
literal-zero path pass. `independent-review.json` records these counts and
explicitly leaves actual-capture oracle and production acceptance false.

A prospective actual adapter must retain its distinct analytic/Taylor coverage
record. It cannot set `native_absolute_order` from a dual component count to
satisfy the existing original-Series reader. That reader requires either an
actual native Series certificate or a separately reviewed format and coverage
admission for this different native route. Complete original-input identity,
per-branch bound, intermediate-domain checks and the full signed-order union
remain required before any actual comparison.

The subsequent [actual adapter protocol](native-dual-original-adapter.md)
passes independent source/interface review. It chooses the separate format
`native-original-taylor-dual-oracle`, version one, and rejects a fabricated
`native_absolute_order`. Its coverage names the original-source bound, singular
axes, monomial power, exact intermediate admission and complete native Taylor
shape; all seven component/order pairs remain explicit. The old Series-v1
reader branch must remain intact. Approximate MPFR exports retain precision and
digest labels, and every original input/proof/exact-point/multiplicity check
remains mandatory. Shared small-helper extraction needs its affected control
rerun. This accepts the bounded diagnostic interface, not an implementation,
actual runtime or numerical result.

Reviewed source SHA-256 values are
`a6da1267821bb28f8c8f7eda8584c73f574308f93592ef74409d74b86519e040`
for Symbolica `evaluate/dual.rs`,
`99786b82b110ec42607878c3e5a97c9580e5604b7aaab70b7c9c11f069db841c`
for its `transcendental.rs`, and
`daad6e2cdca234b2691ab4d0a33fae1313f65dbe34763a7cc4e91333fbc499d5`
for Numerica `domains/dual.rs`. These identify the inspected local native
sources; they are not execution evidence for the proposed route.
