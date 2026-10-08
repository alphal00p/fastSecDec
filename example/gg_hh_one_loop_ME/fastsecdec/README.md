# Native FastSecDec reproduction

This computes the complete top-quark one-loop `g g -> h h` amplitude for incoming
`++` helicities at the shared [benchmark point](../point.json). There are two
triangle diagrams and six box diagrams. All are integrated, including their
Laurent poles, before forming the coherent complex amplitude.

From the repository root, with `SYMBOLICA_LICENSE` in the environment:

```sh
bash example/gg_hh_one_loop_ME/fastsecdec/run.sh
```

The script builds the native Rust CLI and exporter, generates portable SymJIT O2
kernels, integrates every diagram using 4096-point lattices and 32 independent
random shifts initially, and writes `result.json`. It uses one generation worker
and two integration workers. `FSD_WORKERS` can change integration concurrency;
`FSD_SKIP_BUILD=1`, `FASTSECDEC_BIN` and `GGHH_EXPORTER_BIN` select already-built
binaries. Large artifacts, checkpoints and streamed JSON logs stay in ignored
`work/` and `logs/` directories. No local Python module or Python algebra is used.

## Source construction

The tracked `inputs/` directory is the common source for this calculation and the
independent HEPKit one-loop reduction. To regenerate it into a **fresh** directory:

```sh
target/release/examples/gghh_one_loop_me /tmp/gghh-one-loop-inputs
```

[`exporter.rs`](exporter.rs) calls `Model::standard_model()` and the native HEPKit
diagram generator with one loop, QCD order 2, QED order 2 and the top/gluon/Higgs
particle selection. External labels are not symmetrized, so all labeled
permutations are explicit. The native zero-color filter removes three exactly
vanishing graphs, leaving eight diagrams. Native graph factors and closed-fermion
signs are retained without hand-assigned multiplicities. All other quark Yukawa
couplings and all widths are zero.

The exporter reuses the existing native double-box helpers for Idenso color
contraction, Linnet DOT serialization, exact external scalar products and generic
native `++` gluon wavefunctions. Each directory contains the original full native
diagram snapshot, the projected HEPKit DOT graph, model/card, exact point and CLI
cards. Polarization coefficients are the exact rational representation of the
native floating-point wavefunctions, including their phases; they are shared
unchanged with the analytic calculation.

The triangles retain the tree Higgs propagator in the graph. Their run cards
explicitly request `FamilyPreparationPolicy::SingleTerm`: native
`IntegralFamily::partial_fraction` and `sector` extract the exact scalar factor
`1/(s-MH^2)` before Schwinger parametrization. It multiplies the weighted numerator
once. This avoids mixing a loop-independent timelike denominator into the
Feynman-parameter polynomial, and leaves only the three top denominators to
sector-decompose. The family preparation report retains the original denominator
indices and the scalar prefactor. Existing library and CLI defaults are unchanged.

## Normalization and uncertainty

The projector is `delta_ab/8`; the reported amplitude satisfies
`M^{ab}_{++} = delta^{ab} A_{++}`. There is no initial-state spin or color average.
The fixed-helicity color-summed square is `8 |A_{++}|^2`. The result also supplies
the unnormalized delta-contracted native Laurent vector, obtained by multiplying
the projected vector by 8 and its covariance by 64.

Internal Lorentz algebra retains `D=4-2*eps`, so rational terms follow from the
native numerator; no separately generated R2 vertices are added. The normalized
native loop measure is `d^D k/(i*pi^(D/2))`. The cards use the OneLOop multiplier
`gamma(1-2*eps)/(gamma(1+eps)*gamma(1-eps)^2)` with `mu_squared=1` to match individual
diagram Laurent conventions. Restoring the physical loop measure contributes
`i/(4*pi)^(D/2)` to the native integral `J`, giving `iM`; division by `i` yields
the finite physical amplitude `A = sum(J_0)/(16*pi^2)` after pole cancellation.
No sign or phase is fitted to the reference.

Each diagram uses seed `78139 + 104729*index`. The authoritative native total
retains all shared-shift correlations across that diagram's sectors and all
correlations between Laurent coefficients. [`summarize.rs`](summarize.rs) reads
and validates the native saved results, then adds their full covariance matrices
for independently seeded diagrams using Numerica's compensated arithmetic.
The coherent finite amplitude must meet relative RMS error `1e-3`; all retained
poles must be compatible with zero within five standard errors plus a stated
floating-point allowance. Squared-amplitude uncertainty uses the full real/
imaginary covariance. `result.json` contains each diagram, both topology sums,
the complete Laurent sum, diagnostics, seeds and these checks.

## Checked result

The release CLI run on 2026-10-08 completed all eight diagrams and 30 sectors in
5.58 seconds, including sequential generation and integration: the reported
generation stages sum to 2.87 seconds and integrations to 2.31 seconds. Each
diagram reached its target in the first 4096 × 32 allocation. These are timings
on a shared host, not a portable performance promise. Executable hashes, worker
counts and the bounded monitor settings are in
[`run-provenance.json`](run-provenance.json). The debug-profile exporter/summary
only prepared the input and added saved results; every sector was generated and
sampled by the rebuilt release CLI.

The coherent result is `A++ = -0.005243873099405 ± 0.000000004569190`, with zero
imaginary part at this point and relative RMS uncertainty `8.71e-7`. The native
simple pole is `(-5.32 ± 6.51)e-8`, compatible with zero. The full amplitude agrees
with both native analytic reduction and MadLoop within 0.88 standard errors; see
the parent comparison report for all three methods.
