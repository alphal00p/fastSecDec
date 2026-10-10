# Prepared scalar-root owner API

2026-10-09. Research and isolated Symbolica owner implementation for dynamic
contour evaluation. No FastSecDec production callback selects this code yet.
Consuming dependency changes remain coordinated separately.

The standalone owner change is published as
[Symbolica PR 56](https://github.com/symbolica-dev/symbolica/pull/56), based on
upstream `main` independently of the ball-domain PR. A fresh full owner library
build and the 8 scalar/failure plus 2 eager/JIT callback tests pass on that base.
It is authored by ValentinHirschi. GitHub denied formal reviewer assignment;
an explicit `@benruijl` review request is posted on the PR.

For consumer testing, the accepted ball, root and narrow tracked-hypot changes
are also available together on
[the consumer integration branch](https://github.com/ValentinHirschi/symbolica/tree/codex/contour-owner-integration)
at `7ec1be45ef92ae3b154e0d4ce754c0bdf3d9d0ca`, based on current upstream
`community` (`f4e7870`). It also retains existing PR 54 composition and the
reviewed PR 58 square-root enclosure. The first combined `eccd039` revision
omitted PR 54 and failed the actual FastSecDec build despite passing leaf owner
tests; this was corrected by cherry-picking its exact original commits.
Full Numerica, Graphica and Symbolica native libraries were rebuilt together on
the corrected head. Composition, ball, root, eager/JIT, tracked-hypot and
square-root controls pass; the last two also pass on the portable host backend.
This branch is a consumer source, while upstream review PRs remain independent
against `main`. Consuming FastSecDec checks are separate acceptance gates.

## Reuse evidence and boundary

The public API, source/tests and executable probes were checked in
[the dynamic implementation research](contour-dynamic-implementation.md) and
[the artifact design review](contour-dynamic-artifacts.md). The actual tested
Symbolica base is `3db1607f5acd9669cde747aa048a3ca0c0fcb2e1`; Numerica is 3.0.1.
`AtomCore::nsolve` constructs value/derivative evaluators on every invocation.
Native rational interval refinement also performs exact polynomial work.
Neither supplies a prepared callback with safeguarded floating refinement.

The isolated owner checkout is
`DO_NOT_PUSH_FOR_REFERENCE_ONLY/worktrees/symbolica-contour-root-solver`, branch
`codex/prepared-bracketed-root`. The change adds a public function under
`symbolica::solve`, using the same native `RealLike`/`Real` arithmetic and Newton
correction as the existing expression solver. It does not implement algebra,
differentiation, coefficient conversion, evaluator construction, a production
sampler or a parallel FastSecDec root solver.

The tested local owner head is
`2e47574f2505a003d138aac20d45693b74186b9f`, two commits above the ball-domain
commit, authored and committed as ValentinHirschi. It has not been published.

## Contract

`nsolve_bracketed(lower, upper, options, callback)` accepts an already prepared
callback returning `(f(x), f'(x))` in the same native scalar domain. Options
contain absolute/relative coordinate tolerance, iteration limit, optional
initial guess and explicit convergence policy. Bisection safeguards rejected
Newton steps. A native-domain half-endpoint midpoint avoids overflowing the
difference of large opposite endpoints.

The default policy is `Bracket`: terminate on the numerical bracket width or
an evaluated numerical zero. `NewtonOrBracket` additionally permits the existing
style of local correction test. This is explicitly a heuristic for general
functions, not a bound on distance to a root. A sharp but continuous derivative
regression demonstrates the distinction: the heuristic can return a poor
candidate with a large residual, while the strict policy refines the bracket
and reaches the root. The return record exposes the termination reason, root,
residual, ordinary sign bracket/endpoint values and actual callback/iteration
counts so callers can apply their appropriate mathematical contract.

The solver rejects invalid/nonfinite inputs, missing sign brackets, nonfinite
callback values, a zero derivative at a numerical root, iteration exhaustion
and unrepresentable refinement progress. Evaluation counters use checked
arithmetic. A zero derivative away from the root falls back to bisection.
There is no coefficient extraction to f64 and no solver-owned allocation for
plain scalar arithmetic; arbitrary-precision numeric operations retain their
native allocation behavior.

Error-tracked domains deserve a specific distinction: their comparisons and
`is_zero` concern numerical centers. Every successful result, including an
endpoint or initial-guess center zero, therefore applies the Newton correction
in the original domain before return. An uncertain coefficient cannot become
an allegedly exact root merely because the center residual vanished. This
preserves native arithmetic uncertainty, without claiming a certified enclosure
or a global bound on correlated coefficient effects. The numerical solver does
not replace Symbolica's independent implicit derivative hooks/higher jets.

## Contour-specific bounds belong to the contour adapter

For the approved nonnegative equation `H(u)=sum a_p*u^p`, `p>=2`, let `H(r)=1`.
Scaling each monomial proves these exact-arithmetic intervals:

- `H(x)>1`: `x/sqrt(H(x)) <= r <= x`.
- `0<H(x)<1`: `x <= r <= x/sqrt(H(x))`.

The contour adapter may exploit this structure to tighten its numerical bracket
without relying on the general Newton-correction heuristic. Similarly,
`1/sqrt(a_2)` is an upper bound useful for its initial guess when `a_2>=1`.
These facts do not belong in the generic owner solver. Bounds computed with
rounded arithmetic are not automatically certified; enabled validation must
separately enclose relevant endpoint signs/inequalities with native ball
arithmetic. Turning checks off must not suppress solver failures.

## Executed tests and review

The actual owner module and owner test file were compiled directly against the
selected Numerica rlibs, without competing for the shared Cargo build slot.
Both native GMP/MPFR and portable-host Malachite/Astro probes passed **8 tests**:

- f64, DoubleFloat and 192-bit Float retain their own root accuracy;
- prepared degree-six equations, positive/negative scaling from `1e-200` to
  `1e200`, callback-count accounting and retained bracket signs;
- roots near zero, near one and at endpoints, including a `1e-120` supplied
  guess and an independently reached `1e-20` root;
- zero-derivative fallback, nonfinite callbacks, invalid inputs, iteration
  exhaustion and a strictly unrepresentable requested tolerance;
- error-tracked f64, DoubleFloat and Float preserve nonzero coefficient
  uncertainty at endpoint/center roots and through ordinary iterations;
- the sharp-derivative counterexample distinguishing strict and heuristic
  termination.

Probe entry point: ignored `target/contour_root_solver_probe.rs`; owner tests:
`tests/prepared_bracketed_root.rs` in the isolated checkout. Portable-host
execution does not establish browser/WASM execution.

The complete isolated Symbolica library then compiled through a direct `rustc`
invocation using the existing exact native dependency fingerprints. Its public
`symbolica::solve` API passed all eight owner tests. Two additional owner tests
in `tests/prepared_bracketed_evaluator.rs` passed using prepared Symbolica
value/derivative evaluators: eager f64, DoubleFloat and 192-bit Float, and SymJIT
O2. Coefficients change between solves; symbolic differentiation, Horner/CPE,
coefficient mapping and JIT construction occur before refinement. Tiny positive
quadratic roots use a supplied scale-aware initial guess. The only library
build warning was the pre-existing unused export result in `src/atom.rs`.
At that owner-probe checkpoint, measured production runtime, consuming FastSecDec
integration and actual WASM execution remained pending. The consuming native
eager and SymJIT higher-jet controls subsequently passed, as recorded below;
actual WASM execution and final dynamic-production timing remain separate gates.

A subsequent 100,000-case prepared JIT microprobe exposed a rounded Newton
candidate one ULP outside a bracket that had already met tolerance. The initial
implementation incorrectly reported stagnation. The corrected owner accepts
the final native correction only when the union of bracket and candidate still
meets the coordinate tolerance, evaluates that candidate, and rebrackets by its
numerical sign. It does not clamp the value or strip tracked uncertainty. Both
equation signs are covered by the retained JIT regression; the eight native and
portable tests, two evaluator tests, complete owner metadata check and full
coefficient sweep pass after the correction. The runtime reviewer independently
checked this acceptance and rebracketing logic.

The narrow local microprobe measured roughly 320 ns/root and 9.1 average
iterations for strict refinement (maximum 56), versus 183 ns/root and 4.5
iterations for the explicit local heuristic (maximum 9), on varying positive
degree-six coefficients. Both had maximum residual below `9e-16`. These figures
use optimized probe code and prepared SymJIT O2 evaluators and exclude all
deformation construction, derivatives and validation. They are diagnostic data,
not production contour performance or permission to omit the contour-specific
residual/certificate checks required when selecting heuristic termination.

The runtime and generation agents independently reviewed the implementation.
Their requested strict/heuristic distinction and tracked center/endpoint controls
are included. No source-review blocker remains. The authorized publication is
recorded above; these owner controls do not establish Phase B completion.

## Consuming requested-vector controls, 2026-10-10

The combined FastSecDec generation-program filter passes ten tests, including
complete higher-endpoint Laurent vectors through native eager and actual SymJIT
evaluators. The tests retain the strict duplicate-root observation guard and
check the emitted request counts: two distinct requests for Taylor and three
for IBP in the selected control. Native higher jets and all Laurent outputs
share each required root within its actual coordinate/face request.

An initial IBP failure identified a diagnostic identity mismatch: equal symbolic
radii at two endpoint faces shared a union identity, while numerical-dual
subtraction evaluates separate face programs. The corrected native dual lowering
uses each actual face's subset identity; symbolic roots genuinely merged before
subtraction retain their union identity. This was not an evaluator CSE defect,
and the fix does not suppress duplicate detection or add a sampling cache. The
dedicated actual-face regression also passes. Dynamic production admission and
saved exact-vector binding are still separate, unfinished integration gates.
