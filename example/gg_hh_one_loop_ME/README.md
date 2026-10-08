# Complete one-loop top contribution to gg → hh, incoming ++

This example compares the coherent sum of all top-quark one-loop diagrams,
including the Higgs-exchange triangles, the boxes and their interference. It
uses the Standard Model with the other quark Yukawa couplings set to zero.
It is not the additional massive-bottom contribution, which has an open
physical cut at this point.

The three reproduction directories are:

- [`fastsecdec/`](fastsecdec/): native HEPKit diagram generation and inputs,
  FastSecDec sector generation and numerical Laurent-vector integration.
- [`hepkit/`](hepkit/): native one-loop reduction to analytic scalar-master
  expressions, followed by the existing native OneLOop evaluation.
- [`madloop/`](madloop/): an independent MadGraph/MadLoop process, parameter
  card and fixed-helicity driver.

All use the point in [`point.json`](point.json):
√s = 300 GeV, mH = 125 GeV, mt = ymt = 172.5 GeV, cosθ = 4/5,
αs = 0.118 and GF = 1.16639 × 10⁻⁵ GeV⁻². The Higgs self-coupling and
top Yukawa are derived from the model, including its GF-derived vacuum
expectation value. Internal widths are zero.

Physical momenta, in `(E, px, py, pz)` order and GeV, are

```text
p1 = (150,             0, 0,            150)   incoming gluon
p2 = (150,             0, 0,           -150)   incoming gluon
p3 = (150,  15*sqrt(11), 0,  20*sqrt(11))     outgoing Higgs
p4 = (150, -15*sqrt(11), 0, -20*sqrt(11))     outgoing Higgs
```

The metric is `(+---)`. Both incoming gluons have helicity +1; the Higgs
states have spin zero. This point is below the top-pair threshold.

## Normalization

Define the colour coefficient by `M_ab(++ → hh) = δ_ab A_++`. The native
inputs project with `δ_ab/8`. The colour-summed fixed-helicity squared
matrix element is `8 |A_++|²`, and its colour average is `|A_++|²/8`.
There is no initial-helicity average and no identical-Higgs phase-space
factor in these matrix elements.

## Three-method result

| Method | Real colour coefficient A_++ | Colour sum 8\|A_++\|² |
| --- | ---: | ---: |
| FastSecDec QMC | −0.00524387310 ± 0.00000000457 | (2.19985641 ± 0.00000383) × 10⁻⁴ |
| HEPKit analytic reduction + scalar masters | −0.005243877114412127 | 2.199859775284421 × 10⁻⁴ |
| MadLoop | −0.005243877114411830 | 2.199859775284171 × 10⁻⁴ |

FastSecDec uncertainties are one sampling standard error. Its complex-amplitude
relative RMS uncertainty is `8.71e-7`, and the difference from either reference
is `0.879` standard errors. The analytic and MadLoop complex results differ by
`3.86e-16` in absolute value. The imaginary amplitude is zero within numerical
precision at this below-threshold point.

The [analytic master expression](hepkit/result/amplitude.masters.txt) and
[coefficient/master values](hepkit/result/result.json) are retained explicitly,
along with the eight individual diagram expressions. They specialize the exact
kinematics to this benchmark point while retaining the dimension dependence
needed to assemble the finite term.

The coherent native single pole is compatible with zero. Both analytic Ward
checks pass: replacing either incoming polarization by its momentum leaves a
physical finite remainder of approximately `3e-14`. Every numerically integrated
diagram agrees with its analytic Laurent coefficients within `1.92` standard
errors. All eight contributions are retained; no diagram or pole is dropped
because of a numerical cancellation.

[`comparison.json`](comparison.json) records the executable acceptance checks.
To verify the retained results without regenerating or integrating anything:

```bash
nix-shell --run 'cargo run --locked -p fastsecdec --example gghh_one_loop_compare'
```

The comparison fails if a required result is absent, a normalization disagrees,
the references disagree, the numerical discrepancy exceeds five standard errors
plus the stated reference allowance, the one-per-mil amplitude target is missed,
or a pole/Ward check fails.

## Loop-measure convention

The native normalized loop measure is `d^D k/(i π^(D/2))`. At the finite
order, its conversion to `d^4 k/(2π)^4` contributes `i/(16π²)` to the
Feynman graphs for `iM`; therefore the finite colour coefficient is the
native sum divided by `16π²`. The reference uses OneLOop's measure factor
`Γ(1−2ε)/(Γ(1+ε) Γ(1−ε)²)` at μ² = 1 GeV², and FastSecDec includes
the same factor before Laurent expansion.

Dimension-dependent numerator and reduction coefficients are retained until
the finite coefficient is assembled. MadLoop supplies its rational R2
terms separately; the dimension-dependent native calculation already
includes them. No additional R2 contribution is added to the native result.

## Reproduction

Use the repository's documented licensed `nix-shell` Rust environment.
Each method directory contains its own reproduction instructions and source
inputs. Generated executables, native evaluator caches and MadGraph build
trees are excluded from version control; the scripts recreate them.

Run the native numerical calculation and analytic checks from the repository
root with:

```bash
bash example/gg_hh_one_loop_ME/fastsecdec/run.sh
bash example/gg_hh_one_loop_ME/hepkit/run.sh
bash example/gg_hh_one_loop_ME/hepkit/run.sh example/gg_hh_one_loop_ME/hepkit/ward1 --ward1
bash example/gg_hh_one_loop_ME/hepkit/run.sh example/gg_hh_one_loop_ME/hepkit/ward2 --ward2
```

Then follow the [MadLoop reproduction instructions](madloop/README.md), which
use a private copy of `/common/dev/MG5_aMC_v3_7_0` and preserve the shared
installation. Run the comparison command above after all three methods finish.
The MadLoop harness caps its run at ten minutes and 15 GiB; the recorded clean
generation/build/evaluation completed in about 82 seconds and 1.16 GiB.

The HEPKit analytic reference deliberately uses the same native diagram
inputs as FastSecDec. This tests the integration independently of the
numerical sector kernels. MadLoop additionally provides independent diagram
generation and matrix-element evaluation.

The [independent review](../../docs/reviews/gghh-one-loop-me-independent.md)
documents completeness, couplings, phases, covariance, normalization and the
per-diagram checks. The triangle cards explicitly enable native `SingleTerm`
family preparation to factor out the tree Higgs propagator; existing generation
defaults are unchanged.

The completed release FastSecDec run generated and integrated all eight
diagrams in about 5.58 seconds on this host (one generation worker, two sampling
workers). See its [run provenance](fastsecdec/run-provenance.json) for the
executable identity and measurement boundaries; this is a single observation.

Delivery checks passed: 640 workspace tests, zero failures, 28 intentional
ignores, strict Clippy across all workspace targets, formatting and diff checks.
All three method reproductions and the saved-result comparison were executed.
