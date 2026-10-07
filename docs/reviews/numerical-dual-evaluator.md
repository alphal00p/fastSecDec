# Numerical-dual evaluator construction

This is the implementation record for the native evaluator boundary. Independent
review of the prerequisite is in `numerical-dual-native-composition.md`; production
scientific parity is exercised by `crates/fastsecdec/tests/numerical_dual.rs`.
Whole-graph performance must be measured separately.

## Native owners and composition

Pinned Symbolica's public `ExpressionEvaluator::vectorize` and `Dualizer` lower
Numerica `HyperDual` arithmetic to ordinary exact scalar evaluator instructions.
The components are normalized multivariate Taylor coefficients. This is the
same evaluator owner consumed by native eager arithmetic, SymJIT, DoubleFloat
and arbitrary precision; FastSecDec does not introduce a jet interpreter.

The pinned owner offered `merge`, which concatenates outputs of two programs
with the same input layout. It did not expose sequential input/output
composition or output projection. `EvalTree` also has no public construction
boundary that can reuse already compiled programs. Re-evaluating the original
polynomial in an Atom ring would reconstruct mapped symbolic expressions and
would defeat the requested deferred map.

The isolated Symbolica change therefore exposes `EvaluatorComposer`: append an
existing evaluator with bindings to existing input/result slots, then finish
with selected outputs. It reuses `inline_vector_components`, already used by
native automatic function dualization. The former vectorizer's scalar lowering
is shared, preserving callback constant ownership, external functions, branch
labels and optimization. No dependency cache was edited. The initial local
owner commit is `9b82a0bacaed334616ed9066db57b78206418ec4`. The reviewed composition
optimization is the following commit
`1deccb8538ccb91dc2c1e58fc0a2e900d2276bf4`. Both are now published through
[Symbolica PR #54](https://github.com/symbolica-dev/symbolica/pull/54), targeting
`community`; FastSecDec pins the exact tested revision from the public fork.

## Construction, not evaluation-time dispatch

`generation/numerical_dual/native.rs` builds one exact scalar program for a
sector. The caller-owned `SourcePrograms` cache compiles each unmapped original
polynomial for its exact input schema and optimizer settings once. It also
retains native jet lowering by shape and exact known-zero component mask.

For each formal request, FastSecDec:

1. Compiles the small monomial map, independently of the original polynomial.
2. Seeds native jets at the formal coordinates (`Variable`, `Zero`, or `One`),
   with the regulator centered at zero and model/kinematic inputs constant.
3. Composes those mapped jets into the original polynomial's native lowered IR.
4. Selects coefficients shifted by the known monomial valuation on axes fixed
   at zero. It divides only by the remaining monomial on axes with nonzero
   expansion centers.
5. Uses native evaluator/dualizer arithmetic for the regular factor product,
   including regulator-dependent exponents, and selects the requested
   normalized coefficient.
6. Composes all requested values into the existing formal subtraction recipe's
   complete Laurent vector.

No original polynomial Atom is subjected to the sector map. The small scalar
recipe continues to use the existing native aliases and optimizer. Final
artifacts contain ordinary scalar IR and use the existing eager/compiled,
precision, worker-clone, batch and persistence boundaries.

The valuation shift has no additional factorial: if `P(t)=t^v R(t)`, the
normalized coefficient of order `a` in `R` is the coefficient of order `a+v`
in `P` at the zero face. The subtraction recipe separately compensates for its
formal derivative convention. For partial faces, only the axes fixed at zero
are shifted; dividing the other axes as ordinary unit jets preserves their
mixed derivatives and the complete interior function.

Repeated requests share composed slots for identical maps/faces/shapes and
identical polynomial/valuation/depth/face combinations, independently of the
factor exponent. Identical complete regular requests also share slots. This
avoids appending shared U/F work for every Gaussian term.

Known-zero hints supplied to native `Dualizer` are exact, not sampled. A
monomial map's Taylor component can be nonzero only at its exact exponent on
an axis whose center is zero, and at orders no larger than its exponent on
other axes. Runtime inputs have only their scalar component; the regulator
has only its first Taylor component. No numerical cancellation is promoted to
an exact zero or an endpoint convergence proof.

## Evidence and bounded limitations

The ignored focused native composition probe passes input admission, fanout,
identity passthrough, repeated/selected outputs, native branches and special
function constants through eager and SymJIT O2. Its direct example composes
`x=t, y=t*u` into the unmapped `P=x^2+x*y+x^3`; shifting the known `t^2`
valuation yields the residual face value and first coefficients `[3,1,1]`
at `u=2`.

The preexisting focused native jet controls were rerun against the modified
owner and pass eager, SymJIT O2, DoubleFloat106 and Float3322. They check mixed
regulator/coordinate coefficients of `(3+t)^(-1-eps)` and native Gamma
coefficient generation. Logs are retained under the ignored
`output/numerical-dual-study/` directory. They do not establish browser runtime
performance or high-precision Gamma throughput.

Ordinary Taylor jets cannot invert a zero constant term. Directly evaluating
the unfactored singular quotient at the face is demonstrably nonfinite; the
known valuation shift must happen first. Likewise, a negative exponent in a
sector map describes an infinite source-coordinate center and is not an
ordinary Taylor jet. Such maps must use the explicitly recorded symbolic
route or fail admission; they must not be silently assigned zero.

After integration with the concurrent runtime branch repair, actual deferred
sectors conservatively use native complex arithmetic. Formal request symbols do
not prove that their hidden factors or derivative intermediates are real. The
same policy is applied in ordinary, dispatched, cooperative and direct-to-bytes
construction; symbolic fallbacks and exact offsets keep the upstream realness
proof. Real dual results therefore retain zero imaginary output components.
This preserves branches without expanding the deferred expressions. The native
codec remains upstream v8, including its historical real-branch admission guards.

The dense native HyperDual multiplication table has quadratic storage/work in
the number of components. This first adapter reports a resource error above
4096 requested components, including valuation shifts, instead of risking an
allocation failure. It does not invent a sparse jet algebra.

The first whole-graph construction reached compilation rapidly but exposed a
native composition bottleneck: final common-pair optimization retained all
unselected jet instructions and repeated constants. The coordinating benchmark
was stopped after more than two minutes with no completed sector and a sampled
10.5 GiB peak. This is performance evidence, not a successful ggHH validation.

The native composer now slices straight-line instructions backward from selected
outputs, retaining only their exact dependency graph and live callback constants.
It deliberately skips this pruning for programs containing control flow. It then
reuses native callback-aware constant identity and the existing common-instruction
pass before the configured common-pair rounds. The existing Dualizer path keeps
its original optimization sequence; the symbolic baseline is unaffected. This
is instruction dependency analysis in Symbolica's owner, not a FastSecDec algebra
or evaluator implementation.

Three permanent native composition controls pass against the consumer's registry
Numerica/Graphica owner. They cover projected mixed jets and dead Gamma constants,
repeated callback/literal constant distinction, identical native operation counts
after appending a duplicate program, and nested branches through eager and SymJIT
O2 with the configured 1000 common-pair cap. Empty outputs, direct input outputs,
large negative rationals and rejected-append atomicity remain covered. Independent native review accepts the optimization and the final composer-only
abort guard. It separately exercises projection across native Pow, Powf, built-in
and external callbacks, different common-pair limits, and inactive log(0) branches.
The expanded FastSecDec scientific suite passes 12 controls, including both
subtraction methods, native eager/SymJIT, mixed faces, runtime binding, binary
replay and exact lazy materialization. The final portable consumer with registry Numerica/Graphica also passes the
expanded composition probe and all 12 scientific controls; isolated Python
binding/stub-feature checks pass. The completed [double-box benchmark](numerical-dual-benchmark.md)
records full-vector parity, faster observed generation and substantially slower
per-sample evaluation. These host checks do not establish browser execution or
a general generation speedup.

## Conservative numerator valuation

For regular numerator factors, expanding a multivariate support can itself be
prohibitive. The native valuation helper instead substitutes one auxiliary scale
into the original polynomial, weighted by a map column, and asks native Series
for one leading term. It caches original-polynomial/weight pairs. This does not
substitute a complete sector map or enumerate the polynomial's support.

The public AtomCore::series route uses AtomField with statistical zero testing
disabled; the adapter admits only nonnegative source-polynomial powers and checks
that this remains disabled. It never expands or numerically proves the leading
coefficient nonzero. An unresolved cancellation therefore returns a lower bound,
which is safe for exact monomial division; native errors or unsuitable expressions
fall back to the conservative bound zero. Signed maps retain the separate fallback.

The focused owner probe finds weighted degree 2000 for
`(x+y)^1000*(x+z)^1000` with weights `(1,2,3)` in about 1.4 ms. An intentionally
uncancelled symbolic zero coefficient returns bound 2 where the true degree is 3,
confirming the conservative direction. These small controls do not establish the
whole ggHH construction speedup; the separate coordinated benchmark above
provides that measured evidence.
