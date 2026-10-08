# HEPKit analytic one-loop reference

This method reduces the complete shared top-loop `g g -> h h` catalogue to
HEPKit's native scalar master basis. It uses the same labeled diagrams, exact
external Gram products, incoming `++` polarization vectors, parameter card and
`delta_ab/8` color projector as the fastSecDec calculation. It includes both
the Higgs-mediated triangle diagrams and the crossed/oriented box diagrams.
The native graph retains each generated overall factor and the triangle's
tree Higgs propagator.

The implementation is [`reference.rs`](reference.rs), with a thin Cargo entry
point in
[`gghh_one_loop_reference.rs`](../../../crates/fastsecdec/examples/gghh_one_loop_reference.rs).
It composes existing HEPKit owners: `GraphIntegral`/`IntegralFamily`, native
tensor contraction, `oneloopreduce::reduce_family`, `OneLoopMasters`, and
`oneloop::evaluate_with_backend`. There is no copied reduction recurrence,
Gram solver or scalar-master formula.

From the repository root, after producing the shared fastSecDec input catalogue:

```bash
nix-shell --run 'cargo build --locked -j 4 -p fastsecdec --example gghh_one_loop_reference'
./target/debug/examples/gghh_one_loop_reference \
  example/gg_hh_one_loop_ME/fastsecdec/inputs \
  example/gg_hh_one_loop_ME/hepkit/result
```

[`run.sh`](run.sh) builds and runs this command. The input catalogue is shared
with fastSecDec: this reference independently reduces and evaluates those
diagrams, rather than generating a second catalogue. A copy of the source
manifest is saved as `input-manifest.json` with every result.

The user-provided `SYMBOLICA_LICENSE` must be available to the process. Add
`--ward1` or `--ward2` and choose a different output directory to replace the
corresponding incoming gluon polarization by its momentum, before contraction.

Each diagram output retains its scalar numerator, exact dimension-dependent
master coefficients and complete numerical Laurent vector. `amplitude.masters.txt`
is their summed analytic expression in native `A0`, `B0`, `C0`, and `D0`
functions, with the exact benchmark kinematics substituted. This is a
point-specialized analytic master expression, rather than a formula valid for
arbitrary Mandelstam variables. `result.json` also records the physical finite
color coefficient and its fixed-helicity color sum and average.

Reduction coefficients remain functions of `D` until they are expanded about
`D=4-2*eps`; coefficients multiplying ultraviolet poles therefore retain their
finite rational terms. Identical masters are combined before expansion.
The master provider returns `[finite, simple pole, double pole]` at scale
`mu^2=1`. Its normalization relative to the fastSecDec normalized Minkowski
measure is `Gamma(1-2*eps)/(Gamma(1+eps)*Gamma(1-eps)^2)`; the common fastSecDec
cards use that same multiplier. For the ultraviolet-finite full amplitude,
the finite value is independent of this multiplier. No color or spin average
is included in the amplitude. The finite native color coefficient is divided
by `16*pi^2` to recover the amplitude with the conventional loop measure;
the fixed-helicity color sum is `8*abs(A_++)^2`.

The native generator retains eight nonzero diagrams: two triangle orientations
and six labeled boxes. Its native color filter removes three identically
zero-color diagrams; no surviving graph is selected by hand. All retained
graphs have native automorphism order one and closed-fermion-loop sign minus
one. Their projected overall factor is `-1/8`; the color contraction and the
couplings remain in the numerator. The common manifest records these weights.

The completed benchmark gives:

| Contribution | Physical color coefficient `A_++` |
| --- | ---: |
| Two triangles | `+0.007453970457652831` |
| Six boxes | `-0.012697847572064918` |
| Coherent full sum | `-0.005243877114412127 - 2.47e-16 i` |

The fixed-helicity color sum is `2.1998597752844208e-4`; its color average is
`3.4372808988819075e-6`. No initial helicity average or identical-Higgs phase-space
factor is included. The directly computed total agrees with the independent
MadLoop result to about `4e-16` in the amplitude, without changing a phase or
fitting a normalization. The full analytic reference took 7.55 seconds in the
debug example executable; this is a validation timing, not a release benchmark.

The native simple pole cancels to `8.88e-16`, and the double pole is zero.
Replacing either incoming polarization by its momentum independently gives a
physical finite residual of magnitude `2.94e-14`. The corresponding native
Ward residual is `4.64e-12` against a sum of absolute diagram contributions of
`192.98`, a relative cancellation of about `2.4e-14`. Both Ward pole checks pass.
These are numerical checks of the exact native master expressions; they are
not claims of symbolic reduction of the whole Ward sum to zero.

The complete outputs are in [`result/`](result/), [`ward1/`](ward1/), and
[`ward2/`](ward2/). In particular, [`result/result.json`](result/result.json)
retains the numerical values and checks, and
[`result/amplitude.masters.txt`](result/amplitude.masters.txt) retains the full
analytic master expression. Each method directory also includes per-diagram
master expressions and coefficient data, including their source graph digests.

The independent API/source/probe audit found no missing reduction functionality:
the shared-family adapter factors loop-independent tree denominators, retains
native auxiliary-vector Gram products through zero-power reference lines, and
reduces the contracted numerator with symbolic `D`. This actual eight-graph
probe and both Ward variants use those public owners directly. The only local
composition is the same dimension-series/master convolution already used by
the HEPKit bridge and fastSecDec's one-loop reference tests.

The independent review of the numerical example's result adapter also checked
the public APIs before accepting its small amount of bookkeeping. Native
`VectorEstimate` exposes validation, accuracy checks and complete covariance,
but no public operation for adding independent estimates with different
Laurent layouts. Numerica's scalar `StatisticsAccumulator::merge_samples`
pools raw samples; it cannot replace addition of independent diagram integrals
while preserving their Laurent covariance. The adapter therefore maps the
native layouts and adds their complete matrices using existing Numerica
`DoubleFloat` arithmetic. Its actual eight-result execution was checked by
independently recomputing every summed mean and covariance entry: all entries
agreed exactly at binary64 output precision. The color-square uncertainty also
agreed, including its real/imaginary covariance term.
