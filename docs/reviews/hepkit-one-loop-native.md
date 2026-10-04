# Native HEPKit one-loop cross-checks

The recurring reuse audit distinguishes three owners. `feynkit-tensor` supplies
covariant tensor reduction, `one-loop-reduce` supplies scalar one-loop reduction
through the native `IntegralFamily`, and `oneloop` supplies the scalar master
values. HEPKit's community bridge composes these existing libraries; FastSecDec
must not reproduce their master formulas or reduction algorithms.

The community reference lockfile selects:

- `oneloopmaster`: `a42a60aa5fe0b3ba0a5b9bb37a17c8465c06ba5a`.
- `one-loop-reduce`: `b53a70776a43bd14c6562c52a03bc4909568e473`.

Both were copied into isolated, ignored reference checkouts without changing the
original repositories. Both are now FastSecDec development dependencies, with
default features disabled, as the subsequent numerator-validation slice reuses
the native reducer. The dependency graph retains one
Symbolica kernel and SymJIT 2.26.0. No Python or Fortran library is introduced.

The API/source/probe checks are:

1. Community `src/oneloop.rs` delegates master evaluation to `oneloop` and
   reduction to `one-loop-reduce`; it does not place those capabilities in
   `feynkit-tensor`.
2. Native `oneloop/src/backend.rs::evaluate_with_backend` and
   `masters.rs::ScalarIntegral` accept squared invariants, squared masses, and
   the squared scale last, returning `[finite, simple pole, double pole]`.
   At the pinned revision, the generated Rust arithmetic backend is optional;
   tests select the provider's Rust expression interpreter explicitly.
3. An executable B0/C0/D0 probe verified ordering and normalization at the
   massless unit Euclidean point. Its log is retained locally at
   `output/hepkit-one-loop-probe.log`.

OneLOop's normalization differs from the default FastSecDec measure by
`(mu_squared)^eps * Gamma(1-2*eps) /
(Gamma(1+eps)*Gamma(1-eps)^2)`. The native triangle and box public normalization
functions add `pi^2/12` times the double pole to the finite coefficient; this
matches the documented Gamma-ratio conversion in original OneLOop source.
The cross-checks apply that exact symbolic multiplier to the **actual native
graph input** and compare its numerical Laurent vector directly with the
master API. They do not translate values using copied master formulas.

The new `tests/hepkit_one_loop.rs` covers massless and massive bubbles, a
scaleless bubble, triangles with one and three off-shell legs, and massless and
massive boxes at multiple Euclidean scales. Existing independent analytic tests
remain separate. These checks target coefficients through the finite term;
the master interface does not promise positive epsilon orders. Its own release
audit reports limitations at fixed binary64 precision and some degeneracies, so
it is an independent cross-check within its supported cases, not a replacement
for the independent analytic controls.

Validation passed: all three graph-to-master tests, covering eleven points,
finished in 8.53 seconds in the development test profile. The local log is
`output/hepkit-one-loop-tests.log`. This measures the regression workload, not
the project's performance acceptance benchmark.

The same audit exposed a separate CLI model-composition error. Cached derived
parameter/coupling values were read before exact inline overrides, so a model
with `a=2`, `b=2*a` cached as four and an inline `a=3` retained the stale four.
The permanent input-loading regression demonstrated the wrong integral
prefactor before the fix (`output/model-cache-reproducer.log`).

Native `Model::apply_parameter_card`, `apply_parameter_card_with`, and
`evaluation_request` establish that expression-backed internal parameters are
recomputed unless explicitly fixed by the card; couplings use their analytic
expressions. Native `ModelEvaluator` accepts and returns numeric `Complex<f64>`
values, which cannot preserve exact inline Symbolica expressions. The CLI's
existing thin Atom binding adapter therefore reads those native analytic
definitions, retains explicit internal restriction-card values and
expressionless constants, and overlays exact inline values last. It continues
to use native simultaneous substitution; no new master, reduction, numerical
model evaluator, or general algebra capability is added.

The exact input-loading regression now passes: an external override recomputes
both dependents, a symbolic pi value remains exact, an explicit internal
restriction survives unrelated external changes, and an inline internal value
supersedes that restriction. An expressionless internal constant is retained in
all four cases. The successful log is `output/model-cache-tests.log`.

The next numerator cross-check can use the existing
`oneloopreduce::reduce_family(&IntegralFamily, &[i32], &Atom)` directly. Its
public `Reduction::terms`, `MasterIntegral::arguments`, and
`OneLoopMasters::symbol_with_scale` retain the native family, exact dimension
dependence, and master argument conventions. The source limits numerator degree
to twenty and total positive propagator index to thirty-two. These are limits
of that validation provider, not new limits on FastSecDec's Gaussian path.

The community's finite-coefficient composition currently lives in its PyO3
bridge. It combines identical masters before series expansion and rejects
dimension-four coefficient poles that require unavailable positive epsilon
orders of the masters. Any Rust test adapter must preserve those checks and use
Symbolica's native series arithmetic. Merely evaluating a reduction coefficient
at dimension four would discard terms that multiply divergent master
coefficients. No numerator master comparison is claimed until that separate
API/probe/test slice is executed; its evidence is recorded separately in
`hepkit-numerator-reduction.md`.
