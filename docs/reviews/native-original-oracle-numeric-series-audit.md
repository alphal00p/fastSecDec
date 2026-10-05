# Native numeric-series alternatives for the original oracle

Source audit, 2026-10-05. No new executable, mathematical adapter, dependency
patch or scientific run was produced for this review. The original-expression
oracle is independent of the named candidate. Its first point-zero attempt
timed out after 180.167230052 seconds during the native width-seven expansion;
the separately approved 600-second attempt keeps the same exact-Atom algorithm.
Candidate generation/program construction do not substitute for this oracle.

## Findings

The pinned libraries do not expose a complete, directly interchangeable
`Series<MPFR>` evaluation route for this Laurent expression. Several useful
native pieces exist, but their domains differ:

| Native facility | Confirmed support | Limitation for this oracle |
| --- | --- | --- |
| `Series<F: Ring>` | Generic storage, native remainder metadata, addition, multiplication and coefficient access; Numerica supplies `FloatField<Float>` | Native series division, powers, logarithms and other transcendentals are specialized to `Series<AtomField>`. Atom series parsing also returns this specialization. |
| Evaluator ring evaluation | Native arithmetic, integer powers, fallible inversion and conditional instructions | `try_evaluate_in_ring` rejects noninteger powers and built-in/external functions. `map_to_ring` rejects evaluators with external functions. It cannot directly evaluate the regulator-dependent powers and Gamma expression into a numeric series. |
| Numerica `HyperDual` and Symbolica `Dualizer` | Native Taylor arithmetic over nonnegative component shapes; native evaluator vectorization supports regular special functions | No Laurent component shape or zero-center pole inversion. Exact removal of every intermediate singularity would need a separately proved native transformation first. |
| `Series::coefficient` | Distinguishes known zero from unknown remainder, including Laurent and fractional orders | Reads an already constructed native series; it does not bypass the slow construction. |
| Atom coefficient collection | Literal factor extraction or polynomial coefficient collection | Does not expand Gamma/regulated powers into a Laurent series; polynomial conversion is not a compact substitute for the native series engine. |
| Local `AtomField::custom_normalization` | A field-owned callback after native ring operations; the public `Series<AtomField> + &Atom` bridge preserves that field | Could numerically collapse constant coefficient expressions, but would change the oracle algorithm and potentially its zero/valuation decisions. No tested numeric-series oracle follows from API availability alone. |

## API, implementation and example checks

All Symbolica paths below are under
`DO_NOT_PUSH_FOR_REFERENCE_ONLY/worktrees/symbolica`; Numerica paths are under
`DO_NOT_PUSH_FOR_REFERENCE_ONLY/numerica`. This review concerns the existing
pinned and locally patched sources, not a claim about another release.

* `src/poly/series.rs:256` defines generic constructors and arithmetic;
  `:691` exposes `coefficient`, and `:717` maps coefficients within the same
  field type. Generic `SelfRing` support does not supply a complete numeric
  evaluation domain. Division at `:1153` and the transcendental implementation
  at `:1204` are `AtomField` specializations. No `Real`, `EvaluationDomain` or
  transcendental-trait implementation for generic `Series<F>` was found.
  Numerica `src/domains/float/field.rs:48,111,223` supplies the existing
  precision-carrying float field and its ring operations.
* `src/derivative.rs:376` constructs a default `AtomField` in ordinary Atom
  series expansion; private `series_impl` at `:429` also uses that type.
  The public addition bridge at `:726` reconstructs native series information
  from `self.get_field()` and delegates to that same private implementation.
  It retries until the native absolute remainder covers its input target.
  Thus a custom field can be supplied without copying Laurent arithmetic.
* `src/evaluate/evaluator.rs:473,580,632` contains the fallible ring evaluator,
  explicit unsupported-function/power cases, and coefficient ring mapping.
  Its documented finite-field example and
  `tests/evaluation.rs::evaluator_in_finite_field_ring` exercise polynomial,
  integer inverse and conditional arithmetic, not transcendental series.
* `src/evaluate/dual.rs:91` documents native vectorization; `:127` rejects
  retained sub-evaluator bodies. Its
  `automatic_mixed_derivatives_and_multiple_arguments` test includes Gamma,
  polygamma and other special functions, comparing Taylor components against
  native symbolic derivatives at regular points. This is useful existing
  coverage, but it does not handle Gamma or rational poles at regulator zero.
  Numerica `src/domains/dual.rs:1324` accepts ancestor-closed nonnegative
  shapes, and inversion at `:1975` begins with the scalar component inverse.
  Merely multiplying a retained singular expression by an overall epsilon
  power does not make each intermediate operation regular.
* `examples/series.rs` uses ordinary Atom series for `(1-cos(x))/sin(x)`;
  `examples/dual.rs` uses native dual arithmetic at nonzero scalar values.
  The series module's coefficient tests explicitly cover unknown versus zero,
  poles, fractional powers and native remainder bounds. No numeric-coefficient
  transcendental-series example or custom-field numeric-series regression was
  found in the inspected examples/tests.
* Atom `coefficient` is documented as literal extraction in
  `src/atom/core.rs:389`; `src/collect.rs:154` implements `coefficient_list`
  through native polynomial conversion. Neither is an analytic series oracle.

## Narrow research option, not a ready oracle

The least invasive possible follow-up would test the existing local
`AtomField` hook on small original expressions. The hook would use native
high-precision evaluation only for regulator-independent coefficient Atoms;
all series products, powers, Gamma derivatives, term ordering and remainder
propagation would remain native. It would not read candidate coefficients.
`src/domains/atom.rs:18,48,80` confirms local cloneable callback ownership;
it requires no process-global function registry or new series container.

This option has unresolved correctness and cost issues. The callback receives
an already formed result, so it cannot guarantee avoidance of every large
temporary. Constant-series construction can retain an Atom without invoking
the field hook immediately. The callback returns a boolean replacement flag,
not a fallible result, so evaluation failure must remain distinguishable from
successful numeric collapse. Exact rational exponents and other structural
data must not silently become floating-point approximations. Native
`AtomCore::to_float` is available (`src/atom/core.rs:1083`), but is not itself
a success certificate: its implementation can leave unsupported functions
symbolic or produce indeterminate coefficients.

More importantly, native series truncation calls the field's zero predicate
(`src/poly/series.rs:645`), and the negative-power branch can enable a native
statistical zero check (`src/derivative.rs:570`). Rounded cancellation before
these decisions can change leading order, required depth or inversion behavior.
Native remainder metadata would then describe the computed approximate
coefficient operations, not independently certify the exact original Laurent
valuation. No tolerance-based pruning, pole trimming or inferred exact zero is
acceptable as a workaround.

Before any actual-input proposal, a separate bounded small control would have
to preserve nonempty full signed-order unions and strict native remainder
coverage; compare against the existing exact-Atom series followed by MPFR;
exercise exact leading cancellation, deliberately tiny nonzero leading
coefficients, rational and Gamma poles, negative requested maxima, complex
constants and evaluation failures; and repeat independent 512/1024-bit runs.
Such a control could establish numerical agreement for those examples, not an
exact all-domain valuation theorem. It has not been implemented or authorized
for execution by this source review. The unchanged exact-Atom oracle remains
the active verification path.

## Captured-input alternative: exact pole extraction before native duals

The original mapped record offers a more specific route than approximate
coefficient normalization. The already source-bound
`output/diagnostics/native-ibp-prepare-1/prepared/progress.json` records one
mapped term, common prefactor `Gamma(4+3*eps)`, and endpoint powers
`[0,-3-3*eps,-3-3*eps,-3-3*eps,0,-2-2*eps,0,-3-3*eps,-1-eps]`.
Its regular part is `U^(2+4*eps)*F^(-4-3*eps)` with the captured positive
polynomial residuals; the prescribed exact interior points are positive.
The completed Taylor identity in that record binds the original 872-piece
density. The later IBP failure contributes no mathematical certificate.

The unchanged Taylor construction in
`crates/fastsecdec/src/generation/subtraction.rs:74–169` provides a conservative
per-branch pole bound six, independently of the observed global leading order:

* Exactly six axes require subtraction. Each selected boundary branch adds
  one denominator `a+degree+1+b*eps` at that axis, with nonzero rational slope.
  This affine denominator has at most one epsilon-zero pole; the other degrees
  have nonzero constant term. There is no product over all degrees in a branch.
* Subsequent coordinate differentiation does not differentiate epsilon or the
  already introduced coordinate-independent endpoint denominators. Native
  derivatives of the regular polynomial powers add factors polynomial in the
  affine exponents and reciprocal residual polynomials, not new epsilon poles.
* The common Gamma center is four. Positive residuals at the admitted faces
  and positive interior coordinates make their regulated powers regular near
  epsilon zero. Addition of branch terms cannot create a worse pole.

HEP's independent source review agrees with this restricted argument. This
does not certify an arbitrary graph, IBP input, moving singularity or unknown
function. A future executable control would still replay the original mapped
powers, residual/domain admission and completed density identity before using
the bound. It must never obtain the bound from candidate coefficients.

Two existing native operations can express a possible transformation without
implementing Laurent arithmetic. `AtomCore::collect_factors` delegates to
`src/collect.rs:1266–1414`: it recursively finds common integer powers in nested
sums/products, including negative powers absent from some summands. It does
not invoke polynomial conversion or blanket expansion and treats functions as
opaque factors. Native multiplication normalization
(`src/normalize.rs:311,672,958`) combines equal-base powers and distributes an
integer power over a product. Thus exact affine zero-center denominators such
as `(-3*eps)^-1` can expose literal epsilon factors, and multiplying the
collected expression by `eps^6` can cancel them natively. The existing
`collect_denominator_factors` and `collect_factors` tests exercise this owner
behavior, but the actual large input has not been transformed or measured.

`map_terms_single_core` is also public, but only maps a top-level `Add`
(`src/streaming.rs:656`). It does not by itself reach terms grouped under the
common Gamma prefactor by subtraction's final assembly at `:178–195`.
Multiplying `eps^6` onto that outer product without extracting nested pole
factors could leave `0*infinity` intermediates. It is not sufficient.

After exact native factoring, any future dual input needs a fail-closed
structural admission check over the entire resulting operation graph, using
native literal epsilon-zero substitution and exact number predicates:
all inverse bases must be nonzero at zero, all noninteger or
regulator-dependent power bases must be positive there, Gamma arguments must
be strictly positive, and no remaining coordinate or unrecognized function
may occur. Inspecting only the whole expression's finite value would miss
unsafe intermediates. This is an input-domain check, not a replacement series
algorithm or a numerical zero test. Native factor collection supplies the
algebraic identity; its factored output need not be structurally identical to
the uncollected original, so a giant expansion merely to assert Atom equality
is inappropriate. Input hashes, native transformation identity and the
reversible explicit monomial cancellation must be distinguished in evidence.

If these conditions can be established in bounded small controls, native
`Dualizer` with component shape zero through six could evaluate the analytic
expression at zero and return the original orders minus six through zero by
the exact monomial identity. Its existing regular-Gamma test is encouraging;
the actual seventh-component/Gamma/MPFR route still requires execution proof.
If that native bridge rejects the regular Gamma factor, a separate native
series of this single affine Gamma factor is a possible source-owned fallback,
with absolute remainder strictly above six. No custom factorial, convolution,
interpolation or candidate-derived data would be justified. This section is
source research only; no transformation, alternate oracle, build or run has
been performed.

## Source identity

SHA-256 at review time:

| Source | SHA-256 |
| --- | --- |
| Symbolica `src/poly/series.rs` | `1e9b1237e32c882445a92db70beb2283333274a65399ea13bf7c29d97ad87516` |
| Symbolica `src/derivative.rs` | `c194b04a3f0ddb94a65a39bb74e309798009398d1010f0bd5188eea8c5c070f6` |
| Symbolica `src/domains/atom.rs` | `dfc6d0b597417383eea5569ed29bb28d686d2bc2ffd29d564cbc0b7d2ba1d66a` |
| Symbolica `src/evaluate/evaluator.rs` | `fbb0086857ea53a71e64060af9c99d1df211166c2ee5212e64bcf214589aea5e` |
| Symbolica `src/evaluate/dual.rs` | `a6da1267821bb28f8c8f7eda8584c73f574308f93592ef74409d74b86519e040` |
| Numerica `src/domains/float/field.rs` | `658d6ef4c43b3fdb4d9b4bd2ef4d8f703e31d6155eb0418e2b102a196e81cc15` |
| Numerica `src/domains/dual.rs` | `daad6e2cdca234b2691ab4d0a33fae1313f65dbe34763a7cc4e91333fbc499d5` |
