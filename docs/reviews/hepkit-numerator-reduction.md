# Native HEPKit numerator reduction cross-checks

This validation slice reuses the same native `IntegralFamily` and contracted
scalar numerator consumed by FastSecDec's Gaussian parameterization. Its
independent reference is HEPKit's Rust `one-loop-reduce` library followed by
the Rust `oneloop` scalar-master provider. It adds no production reduction,
master formulas, graph conversion, or general algebra implementation.

## API, source, and executable probe

The community lockfile pins reducer revision
`b53a70776a43bd14c6562c52a03bc4909568e473` and master-provider revision
`a42a60aa5fe0b3ba0a5b9bb37a17c8465c06ba5a`. Both are development dependencies
with default features disabled. Root Cargo source patches redirect their
native graph/kinematics/master dependencies to the existing pinned checkouts.
The metadata in `output/native-reduction-metadata.json` was inspected: exactly
one package identity exists for Symbolica, Numerica, Graphica, FeynKit graph,
FeynKit kinematics, `oneloop`, and `one-loop-reduce`.

The three reuse checks establish the following:

1. Public `oneloopreduce::reduce_family(&IntegralFamily, &[i32], &Atom)` accepts
   the existing family directly. `Reduction::terms`,
   `OneLoopMasters::symbol_with_scale`, and `oneloop::master_arguments` preserve
   master identity and native squared-invariant/mass/scale ordering.
2. Reducer `src/shared_family.rs` validates symbolic dimension and numerator
   shape, uses the family's native partial fractions, and preserves dimension
   dependence through reduction. Community `src/oneloop.rs::native` shows how
   the bridge composes reduction coefficients with the finite master vector.
   This composition is exposed there through PyO3, so the Rust regression uses
   a small test-only adapter over those existing public APIs.
3. A focused Rust probe reduced the actual shipped triangle rank-one, box
   rank-one, box rank-two, and box rank-five graph inputs and evaluated their
   masters successfully. It took 2.50 seconds in the development test profile;
   local evidence is `output/hepkit-numerator-probe.log`. The probe did not
   substitute handwritten reference coefficients into a test.

The probe's native reference values, in `[finite, simple pole, double pole]`
order and OneLOop normalization, were:

| Shipped numerator point | Native reference |
| --- | --- |
| Triangle rank one | `[-0.4529206736008447, -0.6931471805599453, 0]` |
| Box rank one | `[9.869604401089358, 0, -4]` |
| Box rank two | `[-6, -3, -1.75]` |
| Box rank five | `[-20.881944444444443, -9.135416666666666, -1]` |

These are recorded probe observations; permanent comparisons obtain expected
values by calling the native libraries again.

## Composition and supported scope

The test adapter first combines identical native master symbols, then uses
Symbolica's `cancel` and native series at `D=4` through `(D-4)^2`. Conversion to
`D=4-2*eps` multiplies the first and second Taylor coefficients by `-2` and `4`.
Their convolution with the master vector retains contributions from divergent
masters to the finite coefficient. Evaluating all reduction coefficients at
`D=4` would lose those contributions.

The adapter follows the community bridge's limits: dimension-dependent master
kinematics, fractional coefficient powers, or coefficient poles at `D=4` are
rejected explicitly. Such poles require positive epsilon orders that the
master provider does not supply. Repeated master coefficients are combined
before this check so removable poles in their sum do not cause false rejection.
The reducer also has numerator-degree and positive-index work limits; these are
reference-provider limits and do not restrict FastSecDec's Gaussian path.

Native master evaluation selects `EvaluationBackend::Expression`, the provider's
Rust interpreter, independently of the FastSecDec generated SymJIT kernels.
The master-provider precision and degeneracy limitations remain as documented
in `hepkit-one-loop-native.md`.

For a complete integral comparison, the actual graph receives the exact
multiplier `mu_squared^eps * Gamma(1-2*eps) /
(Gamma(1+eps)*Gamma(1-eps)^2)` to match OneLOop's measure. Native Gaussian
parameterization, geometric sectors, subtraction, Laurent expansion, portable
O2 compilation, and randomized lattice integration then produce the actual
vector. The comparison checks all three Laurent orders and requires real
components for the Euclidean fixtures.

All seven such comparisons passed: the default four numerator fixtures, a
massive triangle at a different kinematic and renormalization scale, a massive
box rank-one point at another scale, and a massive box rank-two point. Every
reported standard error was below `1e-3 * max(1,abs(reference))`; the comparison
used eight standard errors plus a small binary64 reference allowance. The
rank-five finite estimate was `-20.88193116509553 +/- 0.000014209009359746426`
against native `-20.881944444444443`; all three rank-five coefficients differed
by about one standard error or less.

The numerator target took 320.60 seconds in the development test profile,
including generation, compilation, and fixed 2048-point/32-shift integration.
This is a meaningful complete regression, not a performance acceptance result.
Stage timings were added for the next run without weakening its scientific
comparison or sampling budget. Local evidence is
`output/native-numerator-weighted-tests.log`. The combined gate passed 41 tests,
including scalar masters, weighted replay controls, and private MC regressions.

A further complete comparison at a massive rank-two box point with zero
external Gram matrix passed. The native reference returned `[-1/3,0,0]`; the
generated finite result was `-0.3333330685600854 +/- 0.00000020102534118930533`,
with both pole coefficients exactly zero. This preserves Gram-degenerate
support rather than imposing a nonsingular-Gram restriction. The focused test
took 2.12 seconds, including native provider initialization; its Gaussian
parameterization took 0.007737 seconds and numerical integration 0.017144
seconds. These timings are diagnostic observations, not performance acceptance.
Evidence is `output/native-gram-tests.log`. The native reduction cross-checks
now cover eight complete parameter points across the five test functions.
