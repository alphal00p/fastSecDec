# Native symbolic Jacobian determinant construction

This optimization uses Symbolica's existing determinant, replacement and
evaluator APIs. It introduces no determinant algorithm or numerical determinant
callback in FastSecDec. The registered contour foundation gate passes all 71
tests, including the determinant controls below.

## API, source and executable checks

At public owner revision `7ec1be45`, native `Matrix::det` uses explicit formulas
through dimension three and Bareiss elimination thereafter. `AtomField` must
enable exact division cancellation; otherwise removable pivot denominators can
survive as false numerical poles. Source and tests expose no division-free
determinant alternative. Native `det_in_place` uses elimination and does not
remove that requirement.

The new path computes the native determinant of short independent matrix entries
once per dimension, then uses native simultaneous replacement to insert the
physical Jacobian entries. Three process-local caches cover dimensions four,
five and six. They contain no sector expressions, kinematics or evaluators;
retained storage is independent of sector count. Dimensions zero through three
and dimensions above six retain the existing native direct implementation.
The bounded range avoids factorial growth of generic determinant polynomials.
Higher-dimensional performance remains unverified.

Each template must pass native polynomial admission without negative powers.
The physical entries are substituted before the existing symbolic Taylor/IBP,
Laurent and numerical-dual construction. In particular, the result is an `Atom`
containing the complete coordinate dependence, not independent placeholder
variables. Exact singular Jacobians remain legitimate algebraic zeros.

An executable API probe checked a potential alias alternative. `AliasedAtom`
inherits symbolic `derivative` and `series` methods, but they operate on its
explicitly documented opaque root: `a²` with definition `a=x²` differentiates
to zero with respect to `x` until unfolded. There is no alias-aware override in
the inspected source. By contrast, its native evaluator followed by native
`Dualizer` preserves complete cubic chain-rule jets. The current implementation
therefore substitutes before differentiation; it does not retain opaque aliases
through symbolic subtraction.

## Bounded exploratory measurements

The isolated probe uses the exact archived `2L4P.b.K1` causal polynomial with
primary coordinate `x6=1`. It is a six-dimensional primary chart, not a complete
geometry sector or a profile of an active generation worker. Its physical
Jacobian entries contain 26,502 canonical-text bytes. Native direct determinant
construction did not finish within a 55-second limit. The short-entry template
took about 90 milliseconds and substitution about 20 milliseconds. Its generic
canonical text contains 77,639 bytes; the substituted expression has 1,666,829.
These text sizes are diagnostics, not aggregate RSS measurements.

| Native evaluator representation | Build | SymJIT O2 translation | Eager/sample | JIT batch/sample |
|---|---:|---:|---:|---:|
| Entry aliases retained in evaluator builder | 19 ms | 25 ms | 1.65 μs | 214 ns |
| Entries substituted before evaluator builder | 255 ms | 42 ms | 3.69 μs | 491 ns |
| Generic determinant as an uninlined native function | 10 ms | 58 ms | 6.65 μs | 1,110 ns |

The substituted path is the initial implementation because it preserves all
existing symbolic derivatives without a new mechanism. The first row is useful
future optimization evidence, not a current symbolic-generation capability.
Timing was obtained on a shared host and covers only the determinant. It does
not measure the complete density, convergence or accepted sector performance.

The probe verified native polynomial admission, a zero first pivot with exact
determinant minus one, three exact rational coordinate controls and 1,024 scalar
eager/JIT evaluations against native numerical matrix determinants. Batched JIT
values agree with the eager values at those same points. Production tests add
exact singular matrices, symbolic derivative and series controls, shape checks
and a six-dimensional gradient-map comparison. These registered controls pass
in `target/contour-shared-foundation-tests.log`; the runtime agent independently
reviewed bounded ownership, pivot cancellation and derivative preservation.
Larger-dimensional generation and end-to-end performance remain separate gates.

## Higher-dimensional API inventory (read-only)

This follow-up inspects public owner `7ec1be45` and does not change the current
four-to-six-dimensional optimization or its fallback. The required three-loop
case can need a larger Jacobian; its performance is not accepted by the smaller
primary-chart measurements.

| Existing native facility | Verified behavior and implication |
|---|---|
| `Matrix<F: Ring>::det` | Explicit formulas through dimension three; native Bareiss with exact `try_div` thereafter (`numerica/tensors/matrix.rs:1256`). It is not a division-free expression-DAG constructor. |
| `Matrix<F: Field>::det_in_place` | Uses partial row reduction and multiplies pivots (`matrix.rs:1843`); changing to this method does not eliminate removable pivot denominators. |
| `SparseMatrix::det` | Uses the sparse row reducer/LU factors (`tensors/sparse.rs:1078`); useful only after measuring actual structural sparsity, and still an elimination route. |
| Native characteristic-polynomial/division-free matrix API | No public characteristic-polynomial, Berkowitz or other division-free matrix determinant entry point was found in Numerica/Symbolica source, examples or tests. The division-free formulas found in `poly/resultant.rs` are specialized low-degree polynomial resultants, not a general matrix operation. |
| `AliasedAtom` symbolic derivatives/series | The type deliberately exposes an opaque root. `AtomCore::derivative` delegates to that root (`atom/core.rs:688`) and there is no alias-aware override. The prior executable probe establishes the lost chain rule if aliases are treated as independent symbols. Unfolding before differentiation is supported and correct. |
| `FunctionMap` and builder named functions | `add_function[_with_options]`, tagged functions and `InliningPolicy::{Always,Never}` are public native APIs. Definitions live in the evaluator builder; adding them does not itself register a global symbolic derivative hook. Aliases always inline in their caller's scope. |
| Native evaluator dualization | Inlined function bodies/aliases become ordinary native instructions and can be dualized. At `evaluate/dual.rs:129`, `vectorize` explicitly rejects a retained sub-evaluator body with “Sub-evaluator ... cannot be vectorized”. This is a source-confirmed limitation; a new focused named-function reproduction is still pending. |
| Symbolic function derivative/series hooks | `SymbolBuilder::with_derivative_function` and `with_series_function`, plus `symbol!(..., der=...)`, are existing owner extension points. Native `Dualizer` generates higher components from registered symbolic derivatives, with owner tests for mixed/higher derivatives and callback reuse. A derivative hook can use Symbolica-derived bodies; no local AD implementation is justified. |

The existing choices therefore include native direct determinant construction,
a bounded larger generic template followed by substitution, or an inlined
native function/alias evaluator followed by native dualization. Symbolic
subtraction still needs the complete dependence or an explicitly registered
native derivative/series hook. Retaining a `Never` sub-evaluator alone does not
satisfy that requirement. A callback that computes numerical determinants would
also change the approved optimized symbolic-Jacobian design and is not proposed
as a shortcut.

The likely owner gap worth investigating is an efficient **division-free native
matrix expression program** for larger symbolic Jacobians, and, separately,
vectorization of retained evaluator function bodies. The API/source inventory
establishes their absence in this revision, but does not yet establish that a
new owner patch is necessary or faster than supported routes. No new algorithm,
owner patch or PR was made in this review.

### Small executable follow-up plan

Before requesting an owner improvement:

1. Build a two-argument native named-function control and compare explicit
   symbolic derivatives/series with inlined evaluator `Dualizer` output through
   mixed cubic jets. Attempt `InliningPolicy::Never` on the same body and record
   the actual rejection. Repeat using a native derivative hook whose body is
   computed by `AtomCore::derivative`, including a subtraction face.
2. Bound generic seven- and eight-dimensional native-template trials by wall
   time and aggregate RSS. Compare dense and genuinely sparse exact controls;
   preserve exact singular determinants and nonzero determinants with vanishing
   Bareiss pivots. Do not turn a timed-out trial into a zero or relax cancellation.
3. Obtain one actual mapped higher-dimensional sector through existing native
   staging APIs, then compare native direct, template/substitution and supported
   inlined function-map construction. Measure generation, expression/evaluator
   size, sampling cost and full required jets; a primary chart is insufficient.
4. Only if those native routes fail the bounded performance requirement, reduce
   the failing native matrix/program case to an owner-library reproduction and
   discuss a narrow owner API extension. A new determinant or AD algorithm must
   not be implemented in FastSecDec.

These are planned probes, not executed higher-dimensional performance results.
The shared Cargo target and implementation sources were unchanged by this
inventory (apart from the separately requested module-order formatting fix).
