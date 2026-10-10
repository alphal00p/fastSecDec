# Public nonlinear dynamic sampling controls

2026-10-10. The maintained native target is
`crates/fastsecdec/tests/contour_dynamic_sampling.rs`, with its fixture and
restoration helper in `contour_dynamic_sampling/common.rs`.

The unchanged cubic input is
`F=K*(1-2*x)*(1+y)*(1+z)`, with density `F^(-eps)/eps` on the unit cube.
Its full complex vector is pole `1+0i` and finite
`3-4*log(2)-log(K)+i*pi/2`. Native Symbolica differentiates the positive and
negative real primitives and verifies the mixed third derivative and complete
ray's cubic coefficient. The latter is harmful and nonzero at a declared
interior point, so this control exercises higher-degree envelopes. The test
uses both constructions, `K=1,L=1` and `K=40000,L=1e-5`, with `S=0.8,R=1`.
It makes no acceptance claim for the scaled default-cap case.

## Native reuse and execution

The public API, implementation and existing runtime tests were checked before
adding the caller loops. Generation uses `generate` and native Atom operations;
programs use `ProgramArchiveWriter`, `ProgramArchiveReader::select` and
`SelectedProgramReader::load_sector`. Native `KernelSet` binding, real contour
pilots and `KernelResultManifest::integration_problem_from_kernels` enforce
readiness. Sampling owns only the public work loop around `QmcSession`,
`HavanaSession` and `HavanaDiscreteSession`. Each native worker receives the
complete already-weighted vector from `WeightedEvaluationContext`; the test
neither constructs RNG streams nor implements an integration/statistics engine.
The existing worker sources and `runtime_{qmc,mc,mc_discrete}.rs` tests establish
the API's weighting and pilot/freeze conventions; the new executable gate checks
those conventions on actual generated contour evaluators.

Both recipes are encoded independently and their original native owners are
released. The requested recipe is selected before decoding, each actual sector
record is restored separately, and pointwise values are compared with its
resident counterpart. Sampling contexts outlive the reader and temporary
KernelSets. Always checking is exercised on an actual nonlinear production
point; changing to Pilot keeps mathematical identity and completed evidence.
Production then performs no optional causal checks. All four Laurent components
and the full native 4-by-4 covariance are retained and checked.

The declared native Kuo QMC work is 8192 points by eight shifts for None and
2048 by eight for Korobov2/3, seed 34723. Every component must have standard
error at most 0.005 and agree with its analytic value within eight standard
errors or absolute `2e-6`, whichever is larger. This is a scientific regression
criterion, not a claimed production accuracy or floating-point certificate.

Havana and discrete MC each train on 512 points by four batches, then freeze
into separate production streams. Production uses 2048 by 16 for K=1 and
8192 by 16 for K=40000, seeds 194731 and 782291 respectively. Pilot estimates
are unavailable and production counts restart at zero. Every component must
have standard error at most 0.03 and meet the same oracle criterion. The cubic
has one actual sector; its discrete test does not fabricate additional IDs.

A separate unequal-mass projective bubble supplies two actual primary sectors:
`U=x+y`, `F=(x+y)*(x+4*y)-11*x*y`, density
`U^(2*eps-2)*F^(-eps)/eps`. Native projective admission checks its homogeneity.
Its reference is HEPKit's existing Rust OneLOop `B0(11,1,4,1)` Expression backend,
using the same pole/finite normalization as `hepkit_one_loop.rs`. Both recipes
use cap 0.1 and actual discrete pilot/freeze execution, seed 817109, 32768
production draws and per-component standard-error ceiling 0.02. Both actual
sectors must receive samples, and shared full-vector covariance is preserved.

## Gate history and limits

The first target run passed the native primitive proof and two-sector OneLOop
control, but failed two independently declared uncertainty ceilings. With the
exploratory supplied vector `[1,433,1277]`, unperiodized QMC at 2048 by eight
had finite-real standard error 0.005209; increasing points to 8192 exposed
finite-imaginary error 0.007785. Refinement therefore did not give monotonic
error estimates. These failures are retained, not treated as zero or convergence.
The maintained regression uses the existing production Kuo catalogue rather
than retaining that exploratory vector. Seeds, caps and error ceilings are
unchanged. A bounded direct-rustc probe using the maintained helper and native
owners passes all twelve Kuo cases; the maximum component standard error is
0.003184 for unscaled None, 0.001373 for scaled None, and `8.20e-5` for transformed
cases. Source and output are in ignored `target/generation-agent-cubic-qmc/`.

The first scaled Havana allocation had error 0.04699 and failed its 0.03
ceiling. Increasing its production draws fourfold, with the original seeds,
caps and ceiling, passes all Havana/discrete cases; the maximum error is
0.02647. The refined three-of-four target run took 21.88 seconds, with only the
then-supplied-rule QMC gate failing. Raw initial/refined logs remain at
`target/contour-dynamic-sampling-{tests,refined-tests}.log`.

The final maintained target passed **4/4 in 30.66 seconds**, including all twelve
Kuo combinations and both actual Monte Carlo methods, with the original oracles
and uncertainty ceilings. The log is
`target/contour-full-family-sampling-tests.log`. Independent source review by the
foundation agent accepted the analytic normalization, native sampler weighting,
pilot/freeze separation, restored ownership and complete covariance controls.
No performance gain, per-mil acceptance, multiloop result, or actual WASM family
execution is claimed by this evidence.
