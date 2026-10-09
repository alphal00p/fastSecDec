# Contour foundation: independent mathematical and ecosystem audit

Date: 2026-10-09. This is the foundation review for
[CONTOUR_DEFORMATION_PLAN.md](../../CONTOUR_DEFORMATION_PLAN.md), not a claim
that either production contour mode or the Phase B acceptance suite is complete.
The implementation owner, runtime owner and this reviewer have separate files.

## Native ownership and executable reuse evidence

The source review used the workspace-selected Symbolica revision
`1deccb8538ccb91dc2c1e58fc0a2e900d2276bf4` and registry Numerica 3.0.1.
Reading another checkout alone is not proof of a capability in this build.

| Operation | Public API and source evidence | Executed evidence / decision |
| --- | --- | --- |
| Differentiation and polynomial series | Symbolica `AtomCore::derivative`, `series`; `src/derivative.rs`, `src/poly/series.rs` | Planning CAS probe rerun successfully. Use native derivatives and directional substitution/series; no derivative engine. |
| Symbolic determinant | Symbolica reexports Numerica `Matrix`; `Matrix<AtomField>::det` uses closed formulas through dimension three and Bareiss thereafter | Planning CAS probe rerun. Enable exact cancellation on division and disable statistical zero tests for expression-field determinants; test four-dimensional removable intermediate pivots. No determinant implementation is missing. |
| Real root finding | Symbolica `AtomCore::nsolve`, rational `UnivariatePolynomial::isolate_real_root_intervals` and `refine_root_interval`; `src/solve.rs`, `src/poly/univariate/roots.rs` | Planning CAS probe rerun. Generic root finding already exists. `nsolve` constructs value and derivative evaluators on every call; it is not yet an accepted per-sample hot-path design. |
| Implicit higher jets | Symbol derivative hook plus Symbolica `Dualizer`; `src/evaluate/dual.rs` | Planning IFT probe rerun through cubic Taylor order: `[2, 1/4, -1/64, 1/512]`, with one scalar root invocation. No replacement AD system. |
| Compiled complex callbacks | Native external-function domains, evaluator composition and SymJIT O2 | Planning callback probe rerun: complex jets agree; callback-bearing native program files were written. Fresh-process restoration must remain a production regression, not merely this probe's output claim. |
| Certified arithmetic | Numerica `RealBall`, `ComplexBall`; `src/domains/float/interval.rs`, including directed-rounding tests | Callback probe certifies the exact polynomial margin `11/16`. Certified scope is rational construction, negation, conjugation, addition, subtraction, multiplication, division, inversion and nonnegative integer powers. |
| Graphs and physical references | Existing `GraphIntegral` / native HEPKit family, `OneLoopMasters` and one-loop reduction | Extend the existing `hepkit_one_loop` and complete ggHH reference drivers; do not create another graph, routing, reduction or master-integral implementation. |

The three pre-existing ignored planning executables are
`target/contour_cas_probe`, `target/contour_ift_probe` and
`target/contour_callback_probe`. This review reran them. Their binary output is
supporting evidence, not a substitute for the production regression source.
An initial parallel run without a license hit Symbolica's restricted-thread
limit; rerunning the CAS probe with the user-provided environment license passed.
That environment failure was not a numerical failure.

A fresh independent ignored Rust source,
`target/contour-foundation-probe.rs`, was also compiled against the current
workspace dependency artifacts and executed successfully. It checks native
differentiation, a four-dimensional `AtomField` determinant, evaluation at a
removable zero Bareiss pivot, native `nsolve`, and certified rational-ball
arithmetic. Its determinant is exactly `(x^2-1)^2` and remains one at `x=0`.

**Certified-ball limitation:** The audited Numerica baseline explicitly does not
certify inherited transcendental operations. Its `RealBall::sqrt` must not
silently be used as a rigorous enclosure for sign-aware envelopes. The later
[narrow owner improvement](contour-certified-sqrt.md), independently tested and
published as PR 58, adds that specific finite nonnegative real-root capability;
it does not certify inherited complex/transcendental operations. A safe algebraic check of
a computed spectral majorant `mu` verifies `mu >= 0`, `mu-a >= 0` and
`(mu-a)^2 >= b_squared` using certified operations. For the planned dynamic
implementation, a certificate about a rounded numerical deformation would need
inequalities against the actual rounded direction and strength, rather than only
their idealized exact values. That stronger implementation-level certificate is
not delivered by the current fixed checker: it certifies the symbolic polynomial
map at the supplied floating inputs, not all rounded JIT intermediates. Native
error tracking is not ball arithmetic.

## Fixed contour and subtraction ordering

For real input parameters and a designated residual causal polynomial `F`, put
`w_i=x_i(1-x_i)`, `v_i=w_i partial_i F` and `z_i=x_i-i lambda v_i`.
Both endpoint faces of coordinate `i` remain fixed in that coordinate.
The direction does not require normalization and
`Im F(z) = -lambda A + O(lambda^3)`, with
`A=sum_i w_i (partial_i F)^2 >= 0`.

The scalar generation path currently tags U and F identically as singularity
factors. Explicit semantic identity is therefore necessary; position in a list,
an exponent, a symbol spelling or a numerical sign is not a sound substitute.
Keep polynomial admission and support discovery on the original real factors.
Do not send the deformed transcendental density back through that admission.
The designation itself is not a realness certificate: direct input such as
`F=1+i*x` must be rejected even when optional runtime causal checks are off.
Complex numerators and prefactors remain supported. Explicit complex F/U
coefficients can be rejected natively at generation; potentially complex
coefficient functions also need admission/binding treatment.

After the sector's monomial powers have been assembled, the smooth density is
pulled back by `z` and multiplied by

`det(dz/dx) product_i [1-i lambda (1-x_i) partial_i F]^(a_i+b_i eps)`.

This happens before Taylor or IBP subtraction and regulator expansion. The
analytic ratio avoids an endpoint `0/0`. Its real part is one for real data;
therefore its principal logarithm is continuous along the homotopy. Residual
causal F needs its own continued logarithm. Combining the logarithms of several
factors into the principal logarithm of their product can change the sheet.
The complex determinant is signed/oriented; taking its absolute value is wrong.

Local pySecDec source independently confirms this ordering:
`pySecDec/code_writer/make_package.py`, the blocks beginning with application of
the deformation to monomials and construction of `cal_I` (around lines
1091--1135), multiply both determinant and endpoint ratios before subtraction.
Its map construction (around lines 2028--2066) uses componentwise deformation
strengths and real F gradients. FastSecDec intentionally uses the approved
single scalar strength. There is no new pySecDec production dependency.

All boundary terms must restrict the same full-sector contour. Deleting a
coordinate and rebuilding a lower-dimensional deformation generally changes
the normal Jacobian factors and endpoint jets. This also applies to the dynamic
radius, structural counts and smoothing parameters. Numerical-dual generation
must keep every derivative introduced by the deformation and Jacobian; existing
finite-degree masks cannot blindly be reused for nonpolynomial expressions.

Korobov or Havana first supplies real cube coordinates. The complex pullback is
evaluated at those coordinates, and the real sampling Jacobian is multiplied
once. No inverse Korobov derivative is needed at an endpoint.

## Causal branches and numerical checks

The callback probe independently reproduced
`log(-1-i*x^2)` at `x=0` as `+i*pi` in both the ordinary eager and SymJIT paths.
The intended causal boundary value is `-i*pi`. A specific causal-log callback
with Symbolica derivative `1/z` is justified; implementing a replacement
logarithm or differentiator is not. The callback must preserve genuinely
positive imaginary arguments so validation can detect an incorrect contour.

A stationary point with `F != 0` is not automatically invalid: deformation can
vanish there while the integrand has a well-defined continued value. Conversely
`F=0` with `A=0` is an unresolved deformation point, not permission to return
zero and not enough evidence by itself to assert a physical Landau pinch.
Sign checks of a sampled final contour are not a proof that every intermediate
point of its homotopy avoided a zero. A zero complex Jacobian alone is also not
a pole-crossing diagnosis.

The numerical checking policy is observational and distinct from the
mathematical contour identity:

- `always`: check pilot and production;
- `pilot`: check independent pilot work, then remove causal checks from production;
- `off`: remove optional causal checks, retaining solver termination and nonfinite errors.

Pilot samples never enter estimates and never advance production random
streams. Changes to kinematics or mathematical contour settings invalidate pilot
evidence; changing validation policy alone does not invalidate accepted sample
statistics. Reports must not call a finite pilot a global certificate.
Certified checks that cannot resolve a sign require precision escalation or an
explicit error, not sample rejection or an undifferentiated emergency shrink.

## Independent dynamic-bound derivation

This is research evidence for the second milestone. No dynamic production
implementation is accepted before the fixed milestone passes.

At each real point let `q=sqrt(W) grad F`, so `A=q^T q` and `v=sqrt(W) q`.
The auxiliary ray expansion holds `v` fixed. For odd `k >= 3`, write

`T_k=D^k F[v^(k-2), ., .]`, `M_k=sqrt(W) T_k sqrt(W)`.

Then `D^k F[v^k]=q^T M_k q`, hence

`|D^k F[v^k]/k!| <= A ||M_k||_F/k! = A sqrt(B_k)`.

The smooth polynomial `B_k` in the approved plan is therefore correct even
where `A` tends to zero. Cauchy--Schwarz yields

`sum_k sqrt(B_k) t^(k-1) <= sqrt(m_F sum_k B_k t^(2k-2))`.

Every term of H is nonnegative, and the cap term is strictly positive at its
positive root. Consequently the higher odd terms are strictly smaller than
`t A` for `0<t<=S r`, giving `Im F<0` wherever `A>0`. For each strictly positive
U, the same argument applied to `d_k/U` gives `Re U(z)>0`. At `A=0`, the weighted
gradient direction is zero; the separate stationary-point branch rules apply.

`H(0)=0`, its cap term grows without bound, and `H_r>0` for `r>0`. Thus its
positive root is unique. Since all powers of r are at least two,
`r H_r >= 2 H=2` at the root. Native implicit differentiation is well-conditioned
in this relative sense; it does not by itself establish floating-point accuracy
of every coefficient or prohibit physical ill-conditioning.

The full x derivative differentiates the direction as well as polynomial
derivatives and caps. Holding v fixed is only an auxiliary ray-expansion step.
The Jacobian must include the rank-one term `-i v_i partial_j lambda` and all
its further derivatives required by endpoint subtraction.

For the sign-aware majorant use the fixed full-sector dimension n and

`a=tr(R)/n`,
`b_squared=(n-1)/n * (tr(R^2)-tr(R)^2/n)`.

The standard symmetric-matrix bound is `lambda_max(R)<=a+sqrt(b_squared)`.
The square root is not smooth at scalar-matrix degeneracy. Smoothing only the
subsequent positive part is insufficient. The coordinated production formula
therefore smooths both operations with the fixed positive delta:

`t=a+sqrt(b_squared+delta^2)`,
`mu=(t+sqrt(t^2+delta^2))/2`, with `delta=1e-3` initially.

This is smooth and bounds both zero and the largest eigenvalue. For negative t,
evaluate the equivalent rationalized form to avoid cancellation. Weighted traces
and squared norms are polynomial identities; explicit square roots of W and
an eigensolver are unnecessary. All derivatives include this smoothing.
The scalar U bounds need the corresponding smooth positive upper envelope.

The structural term counts, dimension, cap and delta cannot change on a face
or when a coefficient happens to vanish at a sample. Once these choices are
fixed, the resulting radius is smooth. Exact-arithmetic causal validity does
not become a global floating-point certificate when runtime validation is off.

## Scientific gates and current limitations

Useful first independent controls avoid graph-normalization ambiguity:

1. With `F=1-5x(1-x)` and `beta=sqrt(1/5)`,
   `-integral_0^1 log(F-i0) dx = 2-beta log((1+beta)/(1-beta))+i*pi*beta`.
   Evaluate the stationary negative-real midpoint explicitly.
2. With `a=1/4`,
   `integral_0^1 x^(-1-eps)/(a-x-i0) dx`
   has pole `-4/eps` and finite part `-4 log(3)+4 i*pi`.
   This checks endpoint ratios, causal continuation and Taylor/IBP agreement.
3. Compare a generated massive above-threshold bubble against the existing
   HEPKit OneLOop provider, retaining the documented gamma/measure multiplier.
4. Exercise a four-dimensional symbolic determinant at removable intermediate
   pivots; zero-dimensional exact contributions; lower/upper faces; fresh-process
   callback restoration; and fully complex vector layouts.

Fixed-strength variation and dynamic/fixed agreement must compare complete
complex Laurent vectors, including covariance and cancelling poles. Required
physical ggHH and the approved two-/three-loop LTD fixtures remain later gates;
no claim that they ran follows from these foundation probes.

Before accepting the foundation implementation, independently review causal
metadata persistence, symmetry compatibility, endpoint jet masks, runtime
parameter binding, the check-free hot path, exact contributions and fresh-process
restore. All are active integration requirements, not completed by this document.

### Early implementation review

The mapped fixed-density implementation matches the ordering above and uses
native `Matrix<AtomField>::det` with exact division cancellation. Its causal-log
callback delegates numerical logarithms and higher symbolic derivatives to
the existing owners. The first iteration explicitly rejected numerical-dual
contour generation rather than silently changing generation strategy. The owner
has since implemented contour jets with existing native `SourcePrograms::jets`,
`HyperDual`, `Dualizer` and `EvaluatorComposer`, keeping meromorphic prefactors
outside source jets. Its complete independent regression review remains pending.

Two review findings were communicated and addressed in source before milestone
acceptance: contour factors require an existing native symbolic realness proof,
and the new `BranchPolicy` variant must be appended after the old variants.
The latter matters because legacy assessment records use positional bincode
enum discriminants: prepending a variant silently reinterprets old policies,
even if old chart field layouts have a compatibility wrapper. A legacy binary
control remains necessary. These are source-review findings; the production
test suite was still running at this review update.

## Primary literature

The direction/magnitude separation is motivated by the
[LTD construction, sections 3 and 6](https://arxiv.org/html/1912.09291v2).
Its momentum-space bounds are not a proof of the new parameter-space H.
The local-strength derivative contribution to the Jacobian is independently
discussed in [arXiv:2112.09145v2](https://arxiv.org/html/2112.09145v2).
The [pySecDec FAQ](https://secdec.readthedocs.io/en/stable/faq.html) describes
practical deformation-strength/sign-check behavior. Source inspection above
establishes the precise subtraction placement needed here.

## Compiled certified-ball evaluator boundary

A follow-up runtime probe exposed a genuine, narrower owner gap. Numerica's
balls implement the arithmetic required by Symbolica's eager instruction
interpreter, but the selected Symbolica revision does not implement
`EvaluationDomain` for either `RealBall` or `ComplexBall`.

The public `ExpressionEvaluator::map_coeff_with_prec` therefore fails at compile
time with `E0277` for both domains. Its coefficient mapper is precisely the
needed existing operation: exact rational coefficients can be enclosed at a
chosen precision without reconstructing or expanding polynomial expressions.
`ExpressionEvaluator::map_to_ring` is not an overlooked workaround: the existing
`ConvertToRing` implementations also omit the ball fields. The polynomial
`evaluate` API remains available, but requiring a fully expanded sparse
polynomial here would undo the retained factored evaluator design.

The focused compiler evidence is `target/contour-ball-domain-before.log`, from
`target/contour-ball-domain-probe.rs`. Source inspection covered the pinned
`evaluate/{domain,evaluator,external,instruction}.rs` and `coefficient.rs`.
An isolated checkout at
`DO_NOT_PUSH_FOR_REFERENCE_ONLY/worktrees/symbolica-contour-balls` contains the
[small owner patch](../dependency-patches/symbolica-ball-evaluation-domain.patch).
The dirty shared `/common/dev/symbolica` tree and Cargo's immutable Git checkout
were not modified. No workspace dependency revision is changed by this review.

The patch adds the two variable-precision domain implementations. It intentionally
leaves conversion from an already rounded `Complex<Float>` unsupported: a zero
radius would incorrectly pretend to enclose an unknown exact mathematical
constant. Directly registered ball callbacks remain possible; no fallback to a
floating-point callback or uncertified transcendental function is introduced.

This domain declaration alone is not a certificate for arbitrary evaluator
programs. FastSecDec must inspect the native exported instructions, nested
alias bodies and `constant_functions` before mapping its validation program.
Only the documented certified arithmetic subset is allowed. Looking only at
the operation count misses external constant placeholders. The eager interpreter
handles integer powers with native `pow`/`inv`, so polynomial validation does
not secretly use a noninteger-power or logarithm operation.

The isolated patched library compiled successfully using the exact existing
workspace dependency artifacts, without acquiring the shared Cargo target lock.
The standalone mapping probe passes. All **three owner integration tests pass**:
real/complex polynomial enclosures, an exactly known nonzero-imaginary causal-ray
value, and refusal of rounded constant conversion / ordinary-float callback
fallback. The actual owner test source is `tests/ball_evaluation_domains.rs`;
it was compiled with `rustc --test` against the isolated patched library. This
is focused validation, not a full Symbolica test-suite run.

Publication: [Symbolica PR 55](https://github.com/symbolica-dev/symbolica/pull/55)
contains this change independently on upstream `main`. Its full owner library
and three focused tests were rebuilt successfully on that base. The author is
ValentinHirschi. GitHub denied the formal reviewer assignment; an explicit
`@benruijl` review request is posted on the PR.


## SymJIT complex-callback SIMD boundary

A fresh full-bubble experiment exposed wrong batched answers despite correct
scalar answers. The independent probe first separated callback-only, complete
`-det(J)*causal_log(F(z))`, builtin-log and polynomial expressions. Only the
callback-bearing batch paths failed: one to three rows matched eager evaluation,
while four and more rows failed by order-one amounts. Scalar calls, builtin log
and polynomial batches agreed to approximately machine precision. Thus artifact
restoration, numerical statistics and contour construction were not the cause.

A [standalone owner-only reproduction](../../mre/symjit-complex-callback-lanes/README.md)
removes even Symbolica. A two-argument scalar-complex callback with exact dyadic
inputs fails in the first four-row batch. Source review identifies a native
layout mismatch: `Complex<NativeSimd>` stores all real lanes followed by all
imaginary lanes. Reinterpreting it as a slice of scalar `Complex<f64>` pairs
adjacent real lanes, assigning each callback the wrong complex sample.

The latest-release check used the [official sparse index](https://index.crates.io/sy/mj/symjit)
and the exact published 2.27.0 archive. That version was published 2026-10-08,
newer than the initially selected 2.26.4. Both releases fail the same independent
probe; upgrading alone is insufficient. The 2.27.0 archive digest is
`f06329022a5123536f7fd50eb01b3e885dfafe3983f90a0d7b17408f4d0469ea`.
Upstream GitHub `v227` at `edb0db3e0ea4f5c1475d2e56e5d2b2ff4f8fd9e9`
still contains the same adapter defect.

The [small native owner fix](../dependency-patches/symjit-complex-callback-lanes.patch)
gathers matching real/imaginary lanes into each scalar callback and scatters its
result directly into the native split layout, avoiding a subsequent shuffle.
No FastSecDec scalar fallback or alternate evaluator is introduced. The owner
regression covers direct/ordinary translation, threaded/nonthreaded execution,
and rows 1, 2, 3, 4, 5, 8, 17, 255, 256 and 257. All cases pass. The full compiled
owner unit suite reports **2148 passed, zero failed, one ignored** on x86_64.
AArch64 and RISC-V execution have not been tested here.

The isolated owner commit is `24017d4f84010716263d3cbc0e7aba2d48b63fe6`,
authored as ValentinHirschi and not published. The upstream GitHub tree has a
Python/cdylib package layout; the published Rust package wraps the same native
files in `src/symjit`. The validation build preserves the released Rust layout
at ignored `target/symjit-227-contour`, changing only the native adapter/test.
Coordinated dependency selection and upstream publication remain separate work.


The final end-to-end probe rebuilt isolated Symbolica against the corrected
2.27.0 owner crate. Both causal-log-only and complete
`-det(J)*causal_log(F(z))` batches now agree with native eager/scalar evaluation
to **1.56e-15 or better** for every tested batch size through 257. The previous
order-one disagreement disappears; builtin-log and polynomial controls retain
their previous agreement. This is independent evaluator evidence in addition
to the exact owner-only test, not yet the complete physical integration gate.

## Dynamic callback and shared-map source review

The internal dynamic callback foundation reuses a saved native evaluator for
the coefficient-only radius equation and its derivative. Its tagged callback
resolves an already retained helper, with weak global routing and independently
cloned mutable workspaces. Native `ExternalFunctionContainer::clone` clones the
captured callable, so the explicit workspace clone creates independent scratch
rather than sharing a sampling mutex. Restoration must admit the helper before
mapping its parent evaluator. No callback constructs or optimizes an evaluator
at a sample point.

The implicit derivative hook differentiates the smooth mathematical root rather
than solver decisions. Keeping the accepted real root centre while retaining
the imaginary Newton correction preserves uncertainty in zero-centred complex
coefficients. This is native local error propagation, not a certified enclosure
or a claim about globally correlated coefficient errors. The reviewed regression
sources cover imaginary uncertainty, higher jets, concurrent clones, owner
lifetimes and fresh-process restoration; their execution is a separate runtime
gate.

The shared map differentiates the full supplied local strength, including its
gradient in the Jacobian. Chart preparation collects positive residuals from
every term before constructing one map. Its face controls compare restriction
of that full map against the incorrect operation of rebuilding a lower-
dimensional radius. The stationary-gradient control correctly demonstrates that
a displacement cap alone does not bound the Jacobian. The shared map remains
private, and future dynamic recipes must supply their own validated descriptor;
geometric metadata alone must not grant fixed-v9 capability.

One additional owner limitation is independently verified. Numerica
3.0.1's native `Real::hypot` enters its scaled branch through `real_cmp`.
`ErrorPropagatingFloat` does not delegate that operation, so tracked `hypot` and
`Complex<ErrorPropagatingFloat>::norm` use an overflowing square-and-square-root
fallback. A focused primitive-only probe with `1e200` and `1e-3` produces a finite
plain-f64 result and nonfinite tracked results. No existing alternative scaled,
tracking-preserving norm was found in the public API or source. A
[small isolated owner correction](contour-tracked-norm.md) supplies a tracked
`hypot` override using the existing stable centre primitive and native local
uncertainty propagation. Broader scalar-guard forwarding was rejected after
independent tests exposed activated complex shortcuts losing uncertainty.
Native and portable focused tests and existing native API/complex regressions pass.
Consumption remains coordinated separately. A Hermitian complex norm is not a
replacement for the holomorphic square root required by the smooth spectral
envelope.

The follow-up native-v10 descriptor review confirms that saved helper ownership
is restored before numerical callbacks and that the legacy-v9 layout stays
separate. Indexed receipts cross-check explicit descriptor presence and local
source/check indices. Dynamic production remains deliberately rejected. Before
that gate is lifted, the descriptor must also be bound to the actual retained
chart metadata: source identity, dimension, positive factors and required helper
association. Internal descriptor consistency alone does not establish that
association, and selecting charts must not silently omit a required dynamic
descriptor. This is a pending dynamic-admission gate, not a failure in the
currently accepted fixed recipe.

## Public portable application gate

The corrected public Symbolica/Numerica consumer revision `7ec1be4` passed an
actual FastSecDec application build with the portable Malachite/Astro backend.
The test uses public APIs in `tests/portable-kernel/tests/contour_fixed.rs` and
checks both symbolic and numerical-dual generation of the above-threshold
bubble, native artifact serialization/restoration, a certified caller-owned
pilot, checked production at stationary negative `F`, and unchanged values and
mathematical identity after switching validation off. Deterministic quadrature
agrees with all four analytic Laurent components, including the vanishing
imaginary UV residue. The independent generation reviewer requested that final
cancellation assertion; it is included.

The new contour control passes **1/1** in 0.16 seconds. Existing fresh-process
artifact controls pass **2/2**, and complete-complex/covariance/precision-rescue
controls pass **4/4**. This gate uses ordinary locked Cargo with public owners,
registry Graphica 3.0.1 and Feynkit `259df879`; no local dependency override is
active.

The subsequent full portable consumer suite also passed after the coordinated
Feynkit update to `8e3a643f388b45939d6573a648ef3a509086835e`: **73 tests**, zero
failures or ignored tests, with ordinary locked Cargo and no local overrides.
This includes the contour control, symbolic/numerical-dual generation, complete
complex coefficient and covariance controls, QMC/Havana execution, checkpoints,
replay and fresh-process artifacts. The complete test build took 4 minutes.
Portable-host execution does not establish actual browser/WASM execution.
