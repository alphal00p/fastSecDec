# Native dual evaluation of the original oracle — independent review

This review began with source feasibility work following the retained 180- and
600-second exact-Atom oracle timeouts. Subsequent small-control results and
disconnected adapter reviews are recorded below. All three original-expression
oracle points and the candidate's three cold comparisons are now accepted for
this representative only. Production adoption and whole-graph gates remain
open. The named
candidate's coefficients and pole orders must not supply
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
remains mandatory. This interface acceptance does not accept actual runtime or
an actual-input numerical result.

The extracted helper subsequently passes its affected repeat under
`output/diagnostics/native-pole-dual-shared-small-1`: exit zero, no timeout,
7.688539304 seconds, peak child RSS 9,216 KiB and all 21 frozen files intact.
Independent comparison removes only timing fields and finds every remaining
JSON field identical to the accepted original, including all 40 rows, six
shapes/bounds, nine rejections and literal-zero evidence. The original source
remains unchanged. The shared helper SHA-256 is
`4be1d53b69ae54c28034bcc4a84680c9056ab63f3092a082985a77e4b6969c5f`.

The disconnected actual adapter implementation also passes source review for
compile-only preparation. Its native numeric export/import check preserves
both component values and precision for nonzero `Complex<Float>` atoms.
Native canonical numeric zero carries no stored Float precision; the record
labels that explicitly and never certifies symbolic zero. Failed stages retain
partial diagnostics, while a complete result requires all seven component
pairs, precision/realness agreement and immutable-input postflight. Actual
execution and the separate reader's transport checks remain subsequent gates.

The additive reader draft passes source review after one transport correction:
canonical-zero storage cannot accompany a nonzero coefficient row. It imports
all fourteen small native numeric exports to verify exact recorded values and
stored component precisions, or native numeric zero with no symbolic-zero
claim. The old Series-v1 remainder guard remains unchanged. The dual branch
instead validates the exact analytic/Taylor coverage, original proof and build
bindings, fourteen approximate exports and three native intermediates. Its
schema and export controls use accepted small-case values, including noninteger
Gamma coefficients, with explicitly synthetic provenance fixtures. Their
execution is a separate gate and cannot certify an actual captured vector.

That reader control now passes independently at
`output/diagnostics/native-dual-reader-small-1`: exit zero in 0.0284365 seconds,
no timeout, peak child RSS 3,072 KiB and all 27 frozen files intact. Four complete
real cases admit 28 rows; 31 rejection controls include the complex original,
negative requested maximum, false coverage, missing/changed bindings,
contradictory zero storage and numeric value/precision mismatches. Fourteen
noninteger Gamma-model exports retain exact native values and precision. This
accepts the diagnostic reader's admission and transport checks only.

The actual adapter's first compile failed before execution because Rust 1.97.1
could not read the frozen Rust 1.98.1 dependency. It remains retained. Fresh
build two uses the matching compiler and passes its 29-file immutable check;
the standalone source and shared helper are unchanged. Point-zero preparation
at `output/diagnostics/original-taylor-dual-point-0-attempt-1` passes all 78
frozen checks and binds only the original capture and completed Taylor proof.
The reviewed wrapper permits one 180-second process, five-second grace,
30-GiB address-space cap and CPU8, with no automatic later point or reader.
This closes concrete preflight, not the actual numerical outcome.

The first actual point-zero process then failed after 28.70488624 seconds
(exit one, no timeout, peak child RSS 589,848 KiB). Native factoring, exact
analytic admission, monomial reversal and dual evaluation completed, and all
seven recorded precision/realness comparisons passed. The first approximate
export rejected component precisions `[505, 511]` after evaluation requested at
512 bits. No final result or complete numeric export vector exists; the
independent outcome explicitly rejects oracle acceptance. All 78 input checks
remain intact.

The export assertion was too strict. Native Numerica `Float` operations loosely
track result precision: addition/subtraction can reduce it after cancellation,
multiplication/division inherit the lower input precision, and `get_precision`
returns this current precision. The frozen `multiprecision.rs` source and its
`zero_does_not_restore_lost_precision` control confirm the behavior; the source
SHA-256 is `ed49b16eff4521a7fadd61bf623c4753a6d60522eb0b8dee9856814d892464d1`.
The required correction separates requested evaluator precision from actual
component precision and preserves actual values/precision through native
export/import. The reader must parse each component at its recorded actual
precision. Resetting precision to the requested value would erase native
information and is not accepted. This is transport metadata, not a new error
bound, arithmetic algorithm or tolerance. A focused dynamic-precision control
and fresh bounded actual attempt remain required.

Transport text also needs the native round-trip formatter. `Float::Display`
explicitly prints only significant digits and warns that its output may not
reconstruct the stored value. The existing public
`Float::as_raw().to_string_radix(10, None)` API supplies enough digits for native
MPFR round-trip parsing at the recorded actual precision. The correction must
check that exact round trip rather than treating display text as a lossless
serialization or upgrading its precision during import.

The corrected writer's six-case control now passes independently at
`output/diagnostics/native-float-export-control-1`: exit zero in 0.009625888
seconds and all 19 frozen files intact. Requested 512/1024-bit arithmetic
produces cancellation components at 483/995 bits and square-root components
at 513/1025 bits; native zero retains its separate storage semantics. Exact
native decimal parsing and Atom export/import preserve every observed value
and component precision. The accepted shared I/O source has SHA-256
`3251cab5482b8e962a38fefd0ee11b4bbdbd2278cd19ccab4da6626f9f6ffb52`.
This closes the writer transport correction, while a fresh actual-input
outcome remains separate. The revised reader control also passes at
`output/diagnostics/native-dual-reader-small-2`: exit zero in 0.034905062 seconds
and all 24 hashes intact. The four real cases/28 rows and 31 prior rejections
remain, with two additional native arithmetic transport cases. They preserve
cancellation precision 446/958 and square-root precision 513/1025 against
requests of 512/1024 bits, and reject substitution of nominal precision for
actual precision. The legacy Series-v1 reader is unchanged; no actual captured
coefficient agreement follows from these small transport controls.

Corrected actual point zero now passes independently under
`output/diagnostics/original-taylor-dual-point-0-attempt-2`: exit zero in
28.316419658 seconds, peak child RSS 589,832 KiB, all 32 build and 78 attempt
hashes intact. All original source hashes, the exact completed Taylor proof
and all 17 output artifacts were independently rehashed. Seven orders from
−6 through zero pass the requested 512/1024-bit evaluations and realness checks;
fourteen native numeric exports preserve actual component precision. Real
components retain 501–505 and 1013–1017 bits respectively; imaginary components
retain 511 and 1023. None is padded to the nominal request.

The accepted result BLAKE3 is
`d448310303c4a15a5d8915e583e2237ba18b21f3fe9977a7b5ba365802ab0b36`.
Its independent audit binds the absolute result path, point index and digest
for explicit reader selection. This is a complete approximate oracle at one
exact point, based on original-source pole coverage and native analytic Taylor
evaluation. It is not a Series remainder certificate, candidate agreement,
symbolic-zero proof or whole-integral validation. The failed earlier attempts
remain unchanged. This point-zero acceptance was followed by the two outcomes
below; the candidate comparisons were a separate subsequent gate.

Points one and two also pass independently with the same frozen build, native
operations, tolerances and bounds. Point one uses the exact near-boundary tuple
`[1/10^20,37/100,37/100,37/100,37/100,37/100,37/100,37/100,37/100]`;
point two alternates `1/10^8` and `73/100`, starting and ending with `1/10^8`.
Each retains all seven orders, fourteen native exports and seventeen native
output artifacts. Their original source/proof bindings and all 32 build plus
78 attempt hashes were independently checked. Actual real-component precision
reaches 455/967 bits at point one and 479/991 bits at point two, with all
requested two-precision agreement and realness checks passing unchanged.

| Point | Retained attempt under `output/diagnostics/` | Process wall seconds | Peak child RSS KiB |
| --- | --- | --- | --- |
| 1 | `original-taylor-dual-point-1-attempt-1` | 32.789803612 | 610,396 |
| 2 | `original-taylor-dual-point-2-attempt-1` | 31.645152999 | 598,164 |

Both processes exited zero, were reaped and did not time out. Their result
BLAKE3 values are respectively
`1c948f789b73161f8e0ccd00b52a4eda296ee65201cfc617612a933ae38e3988`
and `47afbfe4b1f7e4ce2062c4271d096d8bc020c028dbaf1f6c64c495a262513548`.
Each independent audit exposes the accepted point index, absolute result path
and exact digest for the explicit reader selection. No old failed attempt is
eligible for that selection. These three complete original-input oracles close
the independent input side of the comparison gate. By themselves they do not
establish agreement of the candidate, whole-integral correctness or matched
performance.

## Complete cold candidate comparisons

The explicit reader selection binds point zero attempt two, point one attempt
one and point two attempt one. The rebuilt additive reader preserves the old
Series-v1 branch and admits the distinct analytic-Taylor coverage and native
approximate exports. Its compiled source archive matches the independently
reviewed reader and transport controls; the temporary library wiring was
restored byte for byte after compilation.

The three separately started cold readers under
`output/diagnostics/native-named-actual-readers-dual-1` all exit zero without
timeouts. Independent preflight and postflight verify all 5,417 frozen files;
the 5,411 configuration input BLAKE3 bindings were also rehashed. Final
`result/result.json` records, rather than intentionally earlier progress
snapshots, provide the acceptance evidence.

| Point | Process wall seconds | Peak child RSS KiB | Weighted evaluation precision |
| --- | --- | --- | --- |
| 0 | 60.168910078 | 876,748 | 256 bits |
| 1 | 61.542492682 | 860,104 | 256 bits |
| 2 | 64.462581823 | 885,408 | 384 bits |

All 21 exact-coordinate coefficient comparisons cover the full signed union
`[-6,-5,-4,-3,-2,-1,0]`, including the additional leading order, against the
independent original-expression oracles. Candidate native MPFR evaluation
agrees at requests of 512 and 1024 bits and with the original oracle under
the existing scaled `1e-70` rule. No missing coefficient was padded or accepted
as a sampled symbolic zero.

At the identical rounded coordinates, the cold `SectorKernel` and its native
worker clone each pass weights one and `1e40`, with adaptive and forced replay:
24 complete vectors and 168 real output components. The rounded native MPFR
reference independently agrees at both precision requests; weighted outputs
meet the existing `2e-12 * max(abs(expected), 1)` criterion. All 24 vectors
receive checks and rescue, including all 12 forced replay vectors. This
preserves the conservative cold unknown-real/zero facts and does not reduce the
precision policy to improve the result.

The native IR remains 14,579,188 bytes with BLAKE3
`f9ff07d37d3b82cb8c3b1aa234c4660ebe80088fd35ea36eb5b5fd2ea58d7362`.
The aggregate independent review has SHA-256
`a2600da64d07802684693f22a01dad9f0ef7854c71d96a06faffc706771de54e`.
This closes complete numerical agreement and cold evaluator/weighted replay
for one captured Taylor representative, without its chart multiplicity.
Public generation integration, status/limits, full original graph completion,
integration convergence and matched performance remain separate gates.

Reviewed source SHA-256 values are
`a6da1267821bb28f8c8f7eda8584c73f574308f93592ef74409d74b86519e040`
for Symbolica `evaluate/dual.rs`,
`99786b82b110ec42607878c3e5a97c9580e5604b7aaab70b7c9c11f069db841c`
for its `transcendental.rs`, and
`daad6e2cdca234b2691ab4d0a33fae1313f65dbe34763a7cc4e91333fbc499d5`
for Numerica `domains/dual.rs`. These identify the inspected local native
sources; they are not execution evidence for the proposed route.
