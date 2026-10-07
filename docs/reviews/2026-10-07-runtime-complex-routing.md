# Independent runtime complex-routing review (2026-10-07)

## Scope and present acceptance

This review covers the runtime real/complex selection patch in the isolated
`fastsecdec-runtime-complex-20261007` checkout. The reviewer did not implement
the numerical patch or run a competing build. Final source review accepts the
repair after the scoped native controls, strict Clippy and matched numerical
checks on the complete release Community wheel pass. The tested wheel is based
on Community `80e23e6a`, RustFlow `21edf80` and the final local FastSecDec source.
This establishes the native repair's deployment gate, not a claim that the live
server has already been replaced. It does not establish actual Wasm execution,
high-statistics convergence, a complete physical amplitude, or a new causal
continuation policy.

The earlier notebook audit's literal-all generation policy explicitly disabled
native scattering filters. That historical description does not certify those
graphs as suitable QMC inputs. The present Community repair restores native
loop-1PI, self-energy, tadpole and zero-snail filters, rejects zero-flow edges,
and opts into native zero-color filtering. No graph/parser implementation is
added in the notebook. The generator's default tadpole filter vetoes attachments
through both massive and massless particles; its default self-energy filter
also vetoes both masses. Its default zero-snail filter vetoes massless
attachments. These are native topology policies, not a notebook scalelessness
or cancellation proof.

## Reuse and proof boundary

The former sufficient test only inspected literal imaginary coefficients. Real
literals do not imply a real value for `sqrt(p)` or `log(p)` at arbitrary finite
real runtime `p`. The patch instead uses public native `AtomCore::replace_map`
and `AtomCore::is_real().is_true()`; arithmetic and branch evaluation stay in
Symbolica's existing `ExpressionEvaluator` owners. A real final result alone
does not justify the real backend: `abs(sqrt(-p))` still needs a complex square
root for positive `p`. The final admission proof visits every actual mapped
symbolic operation and requires each to be natively real. It prunes only
already-certified input, alias and immutable-constant proxy leaves; it neither
interprets instructions nor rebuilds an algebra implementation.

Declared inputs receive distinct native function proxies. Runtime parameters
receive only the `Real` attribute. Coordinates receive `Positive` for their
interior domain: existing scalar and batch admission validate the finite closed
unit cube. Endpoint controls retain `sqrt(0) = 0` and native nonfinite errors
for `log(0)` and actual poles; positivity cannot erase endpoint failures.
Native Symbolica requires a nonnegative square-root argument and a positive logarithm argument
to prove their values real; fractional powers require the corresponding native
domain proof. Inconclusive results therefore select complex arithmetic. Distinct
proxies prevent a substitution from accidentally cancelling independent inputs
such as `x-p`. The existing builder rejects duplicate coordinates/runtime
parameters and their overlap before constructing supported programs.

All native alias handles are initially replaced by opaque proxies without their
original attributes. A fixed-point pass upgrades an alias to a distinct real
proxy only when Symbolica proves every operation in its body real under the already established
input/alias assumptions, propagating positivity only when the native body query
also proves it. This prevents a `Real` attribute on an opaque handle
from concealing a complex body or an indirect dependency. Unused complex aliases
do not force a real root complex. The compact alias graph is retained; no
`AliasedAtom::into_inner()` expansion, alternate CAS, numerical realness sampling,
or FastSecDec expression interpreter is introduced.

Two narrow immutable native facts fill missing attributes in Symbolica's
current query. The canonical Euler-Mascheroni constant is positive; its owner
constructs `Constant::Euler` with an exact zero imaginary part. Canonical
`polygamma(n,q)` is real only when `n` is an exact nonnegative integer and `q`
an exact positive rational. No sign is assigned to polygamma, particularly
digamma. Runtime arguments, fractional orders and complex arguments are not
admitted by this fact. Each proxy wraps the original constant identity so
unrelated constants cannot cancel. No callback is evaluated to infer realness,
and no floating-point imaginary-zero comparison or general callback return type
is used as proof. Special-function evaluation remains entirely native.

Direct, cooperative and caller-dispatched compilation share the same selection
helper. Exact endpoint offsets use the same native proof under runtime inputs.
The per-coefficient realness facts used for imaginary-component stability checks
also use this stronger proof. An always-real result with complex intermediates
conservatively receives no imaginary-zero certificate; that can forgo a
stability optimization but cannot justify discarding an imaginary value.
A future, separate final-output proof must never weaken real-backend admission.
The patch does not change coefficient expressions,
transformations, QMC sampling, Laurent assembly, precision rescue thresholds,
or caller-owned execution.

## Regression coverage reviewed

The native controls exercise:

- Square roots and logarithms at positive and negative runtime values against
  direct native complex evaluation, through direct, stepped and dispatched
  compilation, batches and serialized reload.
- The real-valued `abs(sqrt(-p))` control through those same paths, ensuring
  complex intermediate operations are preserved even when the final imaginary
  component is zero.
- The analytic integral `sqrt(p)/eps`, whose branch is entirely an exact endpoint
  coefficient, rebound across the cut with its complete Laurent vector.
- Transitive aliases with misleading `Real` handle attributes, unused complex
  definitions, distinct inputs, and unchanged real polynomial routing.
- A genuine pole at a zero denominator remaining a `KernelError::NonFinite`,
  rather than being admitted as a zero result.
- Coordinate endpoints and immutable special-function domains, including
  rejection of tiny imaginary arguments and retention of the Gamma Laurent
  vector's real layout.

These are scientifically meaningful dispatch controls: the exact pole integral
has an analytic expectation, and the branch cases compare both signs with an
existing native complex evaluator. The reviewer inspected execution logs:

| Gate | Observed result | Evidence |
| --- | --- | --- |
| New routing and binary-artifact controls | 9 passed | `/tmp/fastsecdec-runtime-scoped-final.log` |
| Existing complex kernel, Gamma regulator, artifact and native alias controls | 19 passed, unfiltered, after all-operation strengthening | `/tmp/fastsecdec-runtime-existing-final3.log` |
| Final kernel-unit scope, including nested non-inlined branches and Real-attributed alias intermediates | 38 passed, 8 existing ignored diagnostics | `/tmp/fastsecdec-runtime-kernel-complete.log` |
| Restored 1PI one-/two-loop notebook QMC comparison on the old wheel | 1 passed in 161.83 s | `/tmp/gghh-qmc-regression-before.log` |
| Triangle fixed/runtime comparison on the first experimental patched wheel | 1 passed in 9.36 s | `/tmp/gghh-triangle-regression-after.log` |
| Focused notebook checks | 8 passed | `/tmp/gghh-runtime-focused-checks.log` |
| Strict native library Clippy | Passed | `/tmp/fastsecdec-runtime-clippy-final.log` |
| Final complete release wheel: matched QMC, cold reload, rebinding and notebook checks | 10 passed; 266 unrelated tests deselected; 159.42 s | `/tmp/gghh-runtime-verified-wheel-checks.log` |
| Installed shared-package namespace/type checks | 2 passed | `/tmp/community-runtime-shared-package-checks.log` |

The filtered integration-suite blocks in the first log are not counted as
executed tests. The final kernel-unit run includes the tenth new control: a
logarithm behind two non-inlined native bodies is detected, while a sine body
is admitted. The final unit and existing integration runs also include the
all-operation strengthening. The final 38-test run covers eleven new controls,
including a separate Real-attributed alias chain `a=abs(b), b=sqrt(-p)` that
must use complex arithmetic while returning the real value at both signs of
the parameter. The final complete release wheel also passes the numerical gate
described below. The earlier successful unit run preceded the final proof-domain
compatibility extensions and is not substituted for the final 38-test result.

## Historical artifacts

The original binary loader retained historical component layouts for sampled
native programs. Strengthening only exact-offset validation would leave an old
incorrect real layout usable for a branch-sensitive residual coefficient.
Affected artifacts must be regenerated or rejected; an incorrect layout must
not silently discard a possible imaginary component.

The patch implements a version-8 producer boundary and a conservative gate for
legacy real layouts, using Symbolica's public
`ExpressionEvaluator::export_instructions()`. A narrow inspection of existing
native instructions and nested exported evaluators is an admission check, not
an implementation of evaluation or algebra. Binary version 5-7 and historical
native JSON real layouts containing square-root, logarithm or fractional-power
instructions are rejected with a regeneration error; nested exported bodies
are inspected as well. The new binary fixture test checks safe polynomial and
complex layouts and unsafe real branch layouts, while the existing legacy JSON
and version 1/2 expression fixtures pass.
Safe newly proved real artifacts must not be rejected merely because they
contain a branch-capable instruction. New version-8 artifacts bypass the
historical branch-presence gate and retain their native expression proof;
exact offsets remain checked for every real layout. This is a conservative
compatibility boundary, so some historically safe real branch artifacts require
regeneration. It is not an untrusted-IR validator or a proof of arbitrary
external callback semantics.

## Passed native numerical integration gate

The reviewer inspected the QMC regression source and the final execution log.
The repaired runtime-parametric notebook is compared against the fixed-algebra
preparation in `gghh.py` using the **same native graph, kinematic point, model
inputs, QMC seed and allocation**, preserving the complete Laurent vector.
The point is sqrt(s)=300 GeV, mH=125 GeV, mt=172.5 GeV and cos(theta)=0.8;
QMC uses 1024 points, two shifts and seed 1. Equality is checked coefficient by
coefficient with relative and absolute tolerance 1e-8. Both vectors must be
finite and nonzero.

The contributing one-loop top box and two-loop double box come from the restored
native 1PI catalogue. A separately requested reducible Higgs-exchange triangle
tests the original runtime-branch failure without admitting that topology into
the notebook catalogue. The double box is selected by the older notebook's
native topology predicate rather than a historical diagram label. The kernels
are serialized and reloaded before runtime binding; changing the one-loop point
to sqrt(s)=320 GeV and cos(theta)=0.4 must change the numerical vector.

These QMC comparisons, cold reload, runtime rebinding and eight focused notebook
checks all pass on the final rebuilt complete wheel: ten tests in 159.42 seconds.
The additional two installed namespace/type checks pass as well. The native
numerical repair gate is therefore **passed**, and the independent review finds
no remaining source blocker for deploying that tested wheel.

A nonzero short-run QMC result alone is insufficient evidence of phase or
normalization agreement. This repair enables native complex coefficient
arithmetic; it does not establish a new Feynman causal prescription or authorize
general threshold continuation. Actual portable/Wasm deployment requires its
own execution evidence.
