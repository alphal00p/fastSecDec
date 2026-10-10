# Scalar contour benchmark design

Status: inventory and execution design, 2026-10-10. No scalar benchmark has
been launched. The physical D05 campaign finishes first, as required by the
[current plan](../../CONTOUR_DEFORMATION_PLAN.md#scalar-benchmark-extension-before-the-stopping-checkpoint).
The root review accepts the four-case direction and bounded protocol below;
native input/reference admission and a maintained driver remain to be implemented.

## Four cases and independent references

All inputs use unit propagator powers, scalar numerator one, and
`D = 4 - 2*eps`. Generate through finite order zero and retain **every** emitted
Laurent coefficient, including cancelling or analytically zero poles.
Dimensions below are the original projective dimensions; record the actual
residual dimensions after symbolic endpoint reduction separately.

| Case | Physical point | Loops / projective dimension | Existing native input and reference |
| --- | --- | --- | --- |
| Massive triangle | `C0(0,0,5;1,1,1)`, `mu_squared=1` | 1 / 2 | Existing HEPKit graph and physical OneLOop comparison |
| Massive box | `D0(0,0,0,0,5,-1;1,1,1,1)`, `mu_squared=1` | 1 / 3 | Existing HEPKit graph and physical OneLOop comparison |
| Massless scalar sunrise | `p_squared=1`, three connected massless propagators | 2 / 2 | Existing native sunset graph with `with_numerator(1)`; analytically continued gamma-function identity |
| Mixed-mass kite | `p_squared=3/1000`, `m_squared=1`; three massive and two massless propagators | 2 / 4 | New small native input using existing family/graph APIs; published zero-threshold small-momentum reference, subject to normalization admission |

The triangle and box are already covered at these exact points by
`physical_native_triangle_and_box_contours_match_hepkit_in_both_generation_modes`
in [hepkit_one_loop.rs](../../crates/fastsecdec/tests/hepkit_one_loop.rs).
Reuse its native `GraphIntegral`, model, Gram products and OneLOop
`EvaluationBackend::Expression` reference. The reference call is independent
of the generated SymJIT evaluator. The test's historical NumericalDual endpoint
arm is not part of this campaign. The existing normalized measure must retain
the test's explicit factor

\[
(\mu^2)^\epsilon
\frac{\Gamma(1-2\epsilon)}{\Gamma(1+\epsilon)\Gamma(1-\epsilon)^2}.
\]

Use the native provider's complete `[finite, pole, double-pole]` result with an
explicit map into the generated real/imaginary Laurent layout. These are
above the equal-mass two-particle threshold and probe interior causal
surfaces. OneLOop's scope is described in
[van Hameren, *OneLOop*](https://arxiv.org/abs/1007.4716).

The scalar sunrise reuses
[the existing graph](../../examples/graphs/sunset_2loop_numerator.dot), removes
the numerator through native `FeynmanDiagram::with_numerator(Atom::one())`,
and keeps its native routing. Its propagators are
`k1^2`, `k2^2`, `(k1+k2+p)^2`; this is a connected integral with a nonzero
external scale. For the normalized loop measure and positive-propagator
convention, use

\[
I(D,s)=-(-s-i0)^{D-3}
\frac{\Gamma(3-D)\Gamma(D/2-1)^3}{\Gamma(3D/2-3)}.
\]

The existing [sunset audit](coupled-sunset-numerator.md) establishes the
spacelike scalar sign before Laurent continuation by composing
[Grozin's Eq. (4.3)](https://www-library.desy.de/preparch/desy/proc/ali/proc/grozin_andrey/grozin_andrey.pdf).
At the proposed timelike point evaluate the phase with
`log(-s-i0)=log(s)-i*pi`, then obtain Laurent coefficients using native
Symbolica series. Do not reuse the numerator reference, drop UV poles, or
declare the integral scaleless. Here `F=-s*x0*x1*x2` has no interior zero on the
positive simplex: this is a branch/endpoint control, explicitly **not** a
second interior-threshold stress test. Monomial extraction may make some
contour work inactive. Record that degeneracy and actual retained derivative
slots instead of claiming all six evaluators perform different work.

The kite denominators, in order, are

```text
k1^2-m^2, (k1+p)^2-m^2, (k1-k2)^2-m^2, k2^2, (k2+p)^2.
```

The [upstream pySecDec input](https://github.com/gudrunhe/secdec/blob/master/examples/bubble2L_largem_ebr/generate_bubble2L_full.py)
identifies this connected five-line self-energy as the zero-threshold example
of [Fleischer, Smirnov and Tarasov](https://arxiv.org/abs/hep-ph/9605392).
Its explicit additional multiplier is `-1` to match that paper's measure.
Use the same explicit multiplier, not a fitted sign. The
[upstream reference script](https://github.com/gudrunhe/secdec/blob/master/examples/bubble2L_largem_ebr/integrate_bubble2L_full.py)
quotes the finite value
`4.403658192582334 + 1.5704037847169694*i` at the proposed point.
This printed value is a cross-check; compute the reference natively from the
documented convergent series and retain its truncation bound. No pySecDec,
SymPy or Mathematica execution is required.

The local read-only reference checkout is
`2b3287ecd59436147350ae630a6fdd19eaba9097`. Before admitting the kite, verify
the exact five denominators, connectivity, native U/F and common gamma/measure
prefactor against that input, using HEPKit objects and Symbolica equality.
In particular the stated `-1` multiplier cancels the five-propagator Gaussian
sign: the scalar density has prefactor `Gamma(1+2*eps)`,
`U^(-1+3*eps)` and `(F-i0)^(-1-2*eps)`. Check this identity rather than
transferring a numeric normalization from another topology. Keep
`p_squared=3/1000` exact. The small timelike point crosses the massless cut;
it is not the massive threshold at order-one momentum.

For a finite sum through `n=N` in the published series, both coefficient
ratios decrease and the expansion parameter has magnitude `q=3/4000`.
A conservative absolute remainder bound is
`(3 + abs(log(s/m^2)-i*pi))/(2*m^2) * q^(N+1)/(1-q)`.
Native evaluation through `N=8` therefore suffices well beyond binary64
accuracy; the independent implementation must check that bound and its
branch. Do not use the reference for accuracy acceptance until these native
normalization checks pass. Failure within the preparation budget leaves an
explicit missing/censored case; it does not trigger an open-ended reference
implementation or an unreviewed replacement fixture.

An existing unequal-mass physical bubble, `B0(11;1,4)` at `mu_squared=1`, can
serve as a short preparation calibration if needed. It is already checked in
[contour_dynamic_sampling.rs](../../crates/fastsecdec/tests/contour_dynamic_sampling.rs).
It is not a fifth full benchmark. An elliptic massive sunrise and another
large ladder/LTD example are outside this compact suite.

## Six constructions, one endpoint contract

For each case compare `fixed-v1`, `dynamic-polynomial-v1` and
`dynamic-sign-aware-v1`, each with `contour_jacobian = symbolic` and `dual`.
All six use `GenerationMode::Symbolic`, `IntegrateByParts`, finite order zero,
SymJIT O2, Horner iterations zero, CPE rounds 1000, one compiler core and
coefficient-series initial relative width two. The existing absolute series
coverage gate is unchanged. Preserve deform-before-IBP ordering, exact offsets
and all symbolic derivatives of the determinant and local strength.

The Dual choice must report its actual surviving degree-one contour-image
slots on construction; no full-density endpoint Dualizer is allowed.
Restored statistics may correctly report `None`. An inactive or entirely
exact case may report zero slots, which must appear as such in the results.
An unsupported native control-flow or layout combination is reported as
unsupported, without silently changing the endpoint mode or J policy.

Use one fixed prescription and one cap for each dynamic construction, common
between that prescription's two J choices. A small predeclared admission-only
ladder is `L=[0.1,0.03,0.01,0.003]`, with dynamic `S=0.8`, `R=1`.
Choose the largest common entry admitted by all six arms of a case at the
separate native Pilot16 points. Move down the ladder only for a recorded
causal-admission refusal, not after inspecting production variance or the
reference. Native failures of a different class remain failures. If no common
entry is admitted, retain that result without automatic retuning. This is a
controlled prescription comparison, not a claim that each method's optimal
variance has been found.

Use the actual native validation obligations and aggregate exact owner.
Pilot values never enter production. A centre plus fifteen deterministic
uniform points does not imply exhaustive face coverage; retain and report the
native face obligations separately. Stability settings are identical in all
six arms and recorded explicitly. Production optional causal checks and
diagnostics are disabled after the admitted pilot, while mandatory solver and
precision-rescue checks remain active. Record all DD/Float rescues and failures.

## Matched work and convergence

There are 24 generated arms and **48 independent-run-seed executions**:
two seeds per arm. Each seed execution runs native lattice epochs
`1024`, `2048`, `4096`, each with eight shifts, Kuo33002, Korobov3 and
`package_points=1024`. Each epoch has its own `QmcSession::democratic` and
native complete estimate. Never pool values, covariance or elapsed cost from
different lattice sizes into one estimate. The three native epochs within a
seed are not three independent replications.

Use proposed production seeds `202610103001 + 100*case_index` and
`202610103002 + 100*case_index`, with case indices zero through three. Pilot
seed `202610103901 + case_index` is separate. Confirm their absence from
prior run inventories before freezing the execution plan. The physical D05
final seeds `202610102001` through `202610102004` remain untouched.

Symbolic/Dual J arms for a fixed prescription must have matching native
source mapping, actual residual coordinate order/dimensions and full Laurent
layout. Hash the actual weighted points, including sector, shift, index,
coordinate bits and weight bits, as the maintained variance example already
does. A shape or coordinate mismatch forbids a matched-point claim.
If different prescriptions also share the points, report a **paired**
comparison; do not pool them as independent replicas. Rotate the six-arm
execution order between the two seeds. Mathematical content IDs may coincide
for degenerate cases, so identity inequality is not itself a requirement.

Retain the native full complex estimate, exact offsets once, all covariance
entries, accepted work and completion status. Report finite-part covariance
trace, error to the independent reference, standard errors of each component,
and the two-seed spread. Show convergence against both complete accepted work
and measured sampling time. Two seeds and three lattice sizes give an
illustrative rate, not a calibrated error-coverage or asymptotic-rate proof.
Do not convert numerical failures, incomplete coverage or absent estimates to
zero. Pole cancellations and zero reference components need absolute errors,
not division by a zero reference.

Use a single caller-owned sampling worker per arm for the first comparison,
reusing the existing maintained example's execution pattern. Up to two arms
may run concurrently on disjoint physical cores. These measurements compare
the same workload and concurrency policy; they are not a repetition of the
50-worker D05 throughput experiment. A separate MC grid is unnecessary unless
a specific QMC/reference discrepancy motivates an explicitly bounded check.

## Budget and timing boundaries

The approved design direction has the following hard initial ceilings. They
are feasibility limits, not predicted runtimes or permission to escalate a
censored case. Test the kite's native input/reference admission and first
generation arm early before spending its remaining five arms.

| Stage | Maximum count | Per-task limit | Total active-task allowance |
| --- | ---: | ---: | ---: |
| Native input/reference admission, including optional bubble calibration | 1 | 300 s | 300 s |
| Generation and compiled artifact serialization | 24 | 120 s | 2880 s |
| Restore, bind and separate causal pilot | 48 | 15 s | 720 s |
| Production, all three lattice epochs within one seed execution | 48 | 20 s | 960 s |
| **Total** | | | **4860 s = 81 min** |

Use a 90-minute campaign wall guard, at most two task processes concurrently,
and the user-approved 100 GB aggregate campaign RSS bound.
The 81-minute sum excludes compiling the maintained driver and bounded
termination grace; both must be reported separately. A short first-arm setup
probe consumes its corresponding allowance rather than creating another
unbounded campaign. Censored tasks retain partial native evidence and do not
automatically resume. Physical final runs must already be closed before this
campaign launches.

Measure generation, serialization, fresh-process restoration, cache outcome,
binding/pilot, and sampling separately. Sampling includes native worker
context creation and precision-cache warm-up. Retain the coordinate-hash cost
as a separately measured component and state whether displayed time includes
it. Peak memory is aggregate owned-process RSS measured by the bounded
monitor; do not add unrelated stage peaks or infer allocator release from
logical drops. Record concurrency and affinity. No idle-host guarantee is
claimed. Final estimates use only native complete replicas; deadline/drain
overshoot, incomplete allocations and missing output are explicit.

## Native reuse and reproducibility work

Extend or factor the existing maintained
[contour_variance example](../../crates/fastsecdec/examples/contour_variance.rs)
with a small scalar-fixture adapter and named generation/production actions.
Its native `QmcSession`, `QmcWorker`, `KernelResultManifest`, complete-vector
statistics, weighted evaluation and coordinate hashing are already the right
owners. Native
[QMC refinement](../../crates/fastsecdec/src/integration/qmc/refinement.rs)
extends shifts only; changing the lattice size therefore starts a distinct
native epoch. No extra sampler, estimator, graph parser, algebra engine or
library-owned worker pool is needed. Existing OneLOop masters remain a
reference dependency of the example/tests, not the ordinary core.

Track the small graph/model/run cards, native reference definitions, driver,
and curated numeric summary. Raw logs, artifacts and binaries stay ignored.
The new cards/driver actions below are proposed paths, **not existing runnable
deliverables**. The ordinary CLI generation syntax is already supported:

```sh
fastsecdec generate examples/contour/scalar_benchmarks/triangle.toml \
  --recipe dynamic-polynomial-v1 --contour-jacobian dual \
  --serial --workers 1 --output target/contour-scalar/triangle-poly-dual.fsd
```

The common generation card fragment is:

```toml
[generation]
mode = "symbolic"
subtraction = "integrate_by_parts"
order = 0
[generation.coefficient_expansion]
method = "coefficient_series"
initial_relative_width = 2
[generation.evaluator]
backend = "symjit"
horner_iterations = 0
cpe_rounds = 1000
cores = 1
direct_translation = true
```

A proposed maintained `contour_scalar_benchmark` example should accept the
fixture/card, recipe, J policy, seed, lattice sizes, shifts and output path;
print its complete effective settings and separate prepare/generate/sample
actions. Its final exact command line belongs in the execution manifest after
implementation, not as a fabricated existing interface here. Build it through
`nix-shell`/Cargo with a private target directory and record source, lockfile,
native-owner and binary identities. An isolated probe can reuse the verified
optimized core described by
`target/symjit-loader-profile/current-cache/core-build-command.json`
(source archive `d56a44a844d205efb05d11947d477319219a15045a3310dc7d2baa94c0f12a54`),
but the maintained Cargo example is the reproducible delivery. Preserve the
user's `target/release/fastsecdec` and all physical campaign executables.

Remaining admission work is bounded: implement the scalar adapter/cards,
verify the kite's native normalization and series remainder, check the
timelike sunrise reference and inactive-deformation behavior, exercise all
six construction choices, and independently review the final seed/clock/
covariance manifest before execution. This design makes no new scientific or
performance acceptance claim.
