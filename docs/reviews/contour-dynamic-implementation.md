# Dynamic contour implementation research

2026-10-09. This note prepares the second delivery in
[CONTOUR_DEFORMATION_PLAN.md](../../CONTOUR_DEFORMATION_PLAN.md). It does not
enable dynamic production integration. The fixed-mode scientific and artifact
gates must be accepted first. The independent mathematical proof and its
smooth-spectral-bound correction are in
[contour-foundation.md](contour-foundation.md).

## Native algebra and executable probes

Public API/source inspection used Symbolica `AtomCore::derivative`, `series`,
`together`, and `cancel`; `Series::coefficient`; `SymbolBuilder` derivative
hooks and `EvaluationInfo`; native `Dualizer<HyperDual>`; and the numerical
root APIs. The focused source is
`target/contour_dynamic_probe/main.rs`, deliberately outside production code.
It links the same patched Symbolica dependency built for the fixed-mode gates.
The selected owner worktree revision is
`3db1607f5acd9669cde747aa048a3ca0c0fcb2e1`; the no-prepared-root API finding was
also checked directly in that revision's `src/solve.rs`, and in the selected
registry Numerica 3.0.1 source. Another local checkout alone is not the basis
for the finding.

The probe checks the following native operations:

1. Expand the Hessian of a factored two-variable degree-five polynomial along
   `x+t*d`, with independent direction symbols. Multiply the coefficient of
   `t^(k-2)` by `(k-2)!` to obtain `T_k`. Contract with two more direction
   vectors and compare exactly with `k!` times the coefficient of `t^k` in
   the independent full-polynomial ray. Check both `k=3` and `k=5`.
2. Extract the exact quadratic ray coefficient of a positive `U`.
3. Define a native scalar root callback for `a*r^2+b*r^4=1`, attach the
   implicit partial derivatives, and ask Symbolica's native Dualizer for
   cubic jets. Compare with derivatives of the explicit native algebraic
   expression, including both coordinate endpoints and the limit `b=0`.
4. Count actual root callback invocations: one scalar root for an entire
   requested jet, not one root per derivative.
5. Verify the sum-of-squares spectral-discriminant identity exactly.

The probe's native expansions are algebraic verification controls. Generation
should retain the factored coefficients and avoid expansion of the complete
integrand. No custom differentiator, determinant, polynomial collector,
algebraic simplifier, or general root solver is warranted by these operations.

The Hessian and implicit-jet controls were compiled and executed successfully
against the fixed-mode build on 2026-10-09. The implicit jet has one scalar
callback invocation at each of five tested coordinates, including zero and one.

## One sector recipe, with explicit symbolic dependencies

Keep the sector's full real coordinate list, designated residual F, residual
positive U factors, weights, gradients and Hessian. For the auxiliary series,
use new independent direction symbols and substitute `x -> x+t*d`
simultaneously. Differentiate only with respect to `t` during this step.
Substitute `d_i=w_i*partial_i F(x)` into the resulting coefficients afterwards.

This ordering separates two operations that must not be confused: the ray
expansion holds the direction fixed, while subsequent physical-coordinate
derivatives of the completed coefficient expression include derivatives of
the direction. The latter derivatives are essential in the radius gradient,
Jacobian, and every Taylor/IBP endpoint jet.

For the polynomial construction retain

`B_k = sum_(ij) w_i*w_j*T_k,ij^2/(k!)^2`

as a sum of weighted squares. This is an explicit nonnegativity certificate
on the real closed cube, including `A=0`; there is no division by `A`.
Each even U ray coefficient is the real coefficient of `U(x-i*t*v)`, so it
includes the sign `(-1)^(k/2)`. The baseline squares that coefficient, whereas
the tighter production envelope must retain its sign.

Positive residual U is a separate admission gate. For a native Feynman graph,
the first Symanzik polynomial has nonnegative support coefficients; sector
monomial extraction must retain a strictly positive constant coefficient in
the residual. This supplies a closed-cube certificate. For direct inputs with
arbitrary signed coefficient expressions, a caller's `Positive` tag is not a
certificate. Dynamic admission must either obtain a native sign certificate
at the bound real parameters or explicitly reject an unsupported positivity
case. It must not insert a positive floor. Runtime parameter changes invalidate
the certificate when they affect it.

Store fixed structural counts from the full sector recipe. A boundary
restriction or a coefficient evaluating to zero must not change counts,
dimension, cap, displacement scale, smoothing, or the selected radius
function. Every subtraction face restricts the same symbolic function.

In particular, `v=0` does not make the contour Jacobian the identity:
`partial_j v_i` can be nonzero at a stationary point. Preserve the full
generated Jacobian there. The unchanged causal lower-lip convention handles
stationary negative F; `F=0` at zero direction remains unresolved.

## Root callback and implicit differentiation contract

The polynomial construction can pass its nonnegative coefficient vector to a
canonical callback. With

`G(c,r)=sum_j c_j*r^(2*j)-1`,

the coefficient partial derivative is

`partial r/partial c_j = -r^(2*j)/partial_r G`.

Encode this relation in a native Symbolica derivative hook containing the same
root function. Then native higher differentiation automatically includes the
derivatives of every coefficient, both smooth caps, and all lower root
derivatives. The root solver's iteration decisions never appear in the
derivative graph. Native common-subexpression elimination shares the scalar
root between its derivatives and all Laurent outputs at the same coordinate
request. Distinct endpoint-face requests remain distinct roots.

Register the numeric callback for all supported native scalar domains,
including tracked f64/DoubleFloat/Float values. A callback that extracts
ordinary coefficient centers, solves a plain f64 root, and wraps that answer
as an exact tracked constant loses coefficient uncertainty. Preserve native
error propagation and precision rescue; symbolic implicit jets and numerical
error propagation solve different problems. Neither tracked arithmetic nor
the root solver's ordinary residual test replaces optional ball certification.

For `a*r^2+b*r^4=1`, use the cancellation-resistant closed form

`r=sqrt(2/(a+sqrt(a*a+4*b)))`, `a>0`, `b>=0`.

It remains finite at `b=0`. An independent exact Symbolica verification should
remain alongside the production specialization. In the dimensionless
`u=r/L` formulation, the coefficient of `u^2` is at least one, giving the
natural bracket `0<u<=1`. Scaling improves the root solve without changing
the approved strength convention. Preserve both `L` and `R` as mathematical
runtime inputs, not a hard minimum.

If the residual F is at most quadratic and every residual U at most linear,
all higher ray bounds vanish: `r=1/sqrt(1/L^2+norm(v)^2/R^2)` is exact.
If F is at most cubic and every U at most quadratic, the polynomial baseline
has only `r^2` and `r^4` and admits the specialized form above. Decide these
specializations from the actual mapped residuals, not the graph's loop count:
sector substitutions can raise their degree.

The sign-aware construction yields a polynomial in `u` after its smooth
spectral/scalar envelope coefficients have been evaluated: only its
coefficient construction has square roots. Collect and combine coefficients
with native polynomial APIs. The callback receives their numerical values;
it does not rebuild evaluators or invoke a symbolic solve at each sample.

### Missing hot-path API: narrow owner-library work, not a second solver

The public `AtomCore::nsolve` exists, and its implementation in
`symbolica/src/solve.rs` builds separate value and derivative evaluators on
every call. It uses Newton steps without a positive bracket. This is useful
for controls, but it does not supply the required allocation-free repeated
solve on an already compiled monotone equation.

The native rational univariate root APIs
`isolate_real_root_intervals` and `refine_root_interval` exist. The latter
repeats square-free factorization and operates on exact rational polynomial
coefficients. It is a correctness/reference route, not an accepted sampling
implementation. Source searches of Numerica found no reusable public
bracketed real callback solver.

The focused repeated-root probe measured approximately 50.7 microseconds per
native `nsolve` call for the positive root of `r^2+2*r^4+3*r^6=1` (64 calls).
Evaluating an already prepared native H/Hr program took 44.4 nanoseconds per
call (8192 calls). The latter measures a primitive, not a complete alternative
root solve; these are local research timings rather than production sampling
claims. Combined with the source inspection, they justify pursuing reuse of
prepared evaluators before accepting a generic per-sample solve.

For general-degree production work, propose a small Symbolica/Numerica
owner API accepting an already prepared value-and-derivative callback,
known bracket, tolerance and iteration limit. Return a termination status
and an enclosure suitable for optional certified validation. FastSecDec
should own the contour-specific nonnegative coefficients and mathematical
admission, while the owner library owns the generic numerical refinement.
This note does not claim such an owner API has already been implemented.

## Smooth sign-aware coefficients

Use weighted trace and squared-norm identities for the symmetric matrix
`R_k`; never materialize square roots of `W` or run an eigensolver. The fixed
positive smoothing parameter must regularize both the discriminant square
root and the positive part. Regularizing only the positive part does not
make the spectral bound differentiable when the matrix is scalar.

For `a=trace(R)/n` and
`b2=(n-1)/n*(trace(R^2)-trace(R)^2/n)`, use

`t=a+sqrt(b2+delta^2)` and
`mu=(t+sqrt(t*t+delta^2))/2`.

For numerical evaluation, express the same discriminant as

`b2=(n-1)/n^2*sum_(i<j)(R_ii-R_jj)^2`
`    +2*(n-1)/n*sum_(i<j)R_ij^2`.

Here `R_ii` is a weighted diagonal Hessian coefficient, and `R_ij^2` uses
`w_i*w_j*T_ij^2` times the squared scale. This exact trace identity avoids
subtracting two nearly equal positive norms, and it needs no explicit
square roots of the weights. Native Symbolica should verify the identity
and construct the factored sum.

For negative `t`, evaluate the equivalent rationalized expression
`delta^2/(2*(sqrt(t*t+delta^2)-t))` to avoid cancellation. This numerical
branch evaluates the same smooth mathematical function. Its derivative hook
must differentiate that function rather than differentiating a branch test.
Substitute the full-sector dimension even on a face; handle the empty-sector
case as an exact contribution with no deformation direction.

Numerica currently certifies algebraic ball operations, not inherited
transcendentals. A computed spectral bound must be certified by polynomial
inequalities such as `mu>=0`, `mu-a>=0` and `(mu-a)^2>=b2`; calling an ordinary
ball square root alone does not certify it. With validation off, do not
perform these certification operations per sample. Essential solver
termination and nonfinite errors remain active.

## Required generation and artifact changes after fixed acceptance

A fixed-mode evaluator has a constant runtime lambda input. Substituting a
computed `lambda(x)` into that input at evaluation time is insufficient:
its Jacobian lacks `v_i*partial_j lambda`, and its subtraction jets omit all
local-strength derivatives.

Generate the dynamic density from a symbolic strength expression carrying
the implicit root hook, then use the existing fixed-mode symbolic and native
dual lowering. Keep the mathematical construction shared, but persist
distinct preoptimized evaluator programs for different contour recipes.
Runtime recipe/strength/cap rebinding selects and binds a saved program; it
must not repeat symbolic subtraction, Horner optimization or CPE. The
undeformed path keeps its existing zero contour overhead.

The recipe identity includes the construction version, structural powers,
dimension and smoothing; mathematical runtime settings include S, L and R.
Optional validation policy and pilot evidence stay separate. Preserve the
native callback registration-before-decode discipline and the fixed-mode
regression for complex SymJIT batch callback lanes. Share roots within one
coordinate request, never through an unbounded cross-sample cache.

Future acceptance must exercise mixed endpoint derivatives, symbolic versus
native dual jets, cancellation controls, complete complex Laurent vectors,
all validation policies with identical production sampling, fresh-process
restoration and all ordinary/serial combinations. Compare actual sampling
cost and convergence against fixed mode; a larger radius by itself is not
a performance result.
