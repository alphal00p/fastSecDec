# Independent review: complete one-loop top contribution to gg → hh

Date: 2026-10-08. Reviewer: the MadLoop reference subagent, independent of the
native exporter and HEPKit one-loop reduction implementation. This review covers
the shared point, native graph catalogue, analytic reference, independently
generated MadLoop result and completed numerical FastSecDec sampling.

## Scope and catalogue

The benchmark is the full **top-quark** one-loop contribution at incoming `++`
helicity. All other quark Yukawas are zero. It includes the Higgs-exchange
triangle with the Standard-Model Higgs cubic coupling, both orientations of the
triangle, and six labeled boxes. It does not include a massive-bottom loop or
claim to be the all-quark Standard-Model result.

Reviewed `example/gg_hh_one_loop_ME/fastsecdec/exporter.rs`, the native helpers
under `crates/fastsecdec/examples/gghh_double_box/`, all eight saved parameter and
exact point files, the native manifest, `hepkit/reference.rs`, both method
READMEs, the common point and the actual MadLoop-generated color/loop sources.
The native generator uses `Model::standard_model`, one loop, exact QCD order 2
and QED order 2, labeled initial/final legs and native zero-color filtering.
Its two triangles and six boxes agree with the independent MadLoop generation.
Topology checks require `[top,top,top,Higgs]` or four top propagators. No extra
hand-counted permutation or symmetry factor is introduced.

## Parameters and external states

Every exported card agrees with the common point for `MT=ymt=172.5`, `MH=125`,
`aS=0.118`, `Gf=1.16639e-5`, `aEWM1=132.507`, `MZ=91.188` and zero widths.
The MadLoop card is checked against the same point by its reproduction script.
Both model sources use `v=2 MW sin(theta_W)/e`, with `MW` derived from the same
GF input scheme; `lambda=MH²/(2v²)`, `yt=sqrt(2) ymt/v` and the Higgs cubic
vertex `-6 i lambda v`. The strong coupling is `sqrt(4 pi alpha_s)` in both.
There is no hard-coded vev=246 substitution. Native unused bottom mass and tau
Yukawa defaults are immaterial because those particles are excluded from this
top-only graph catalogue; MadLoop's `no_b_mass` restriction removes bottom
Yukawa diagrams.

The common physical momenta satisfy conservation, massless incoming gluon
conditions and Higgs mass 125. Native momenta retain exact `sqrt(11)` spatial
components. The actual saved native wavefunctions and directly evaluated
MadLoop `VXXXXX` wavefunctions have matching signs and phases:

```text
eps1 = (0, -1/sqrt(2), -i/sqrt(2), 0)
eps2 = (0, -1/sqrt(2), +i/sqrt(2), 0)
```

Their component difference is at most `1.11e-16`, the rounding difference
between native constructions of `1/sqrt(2)`. The native Gram checks enforce
null polarizations, transversality and their normalization. MadLoop takes its
verified native `(1,1,0,0)` helicity row directly; no helicity sum is inferred
from an unpolarized result.

## Normalization, rational terms and owner reuse

The native projection contracts `delta_ab` using existing graph/color tensor
primitives and divides by eight, producing `M_ab=delta_ab A`. It keeps native
diagram overall factors and the triangle Higgs propagator. The normalized
native finite result is converted with `1/(16*pi²)`. MadLoop's generated
`CT_interface.f` already supplies that factor, while its sole color flow is
`Tr(Ta Tb)=delta_ab/2`; consequently `A=JAMPL/2` directly. No complex phase or
scale was fitted to the native result.

MadLoop's generated `IDEN=512` and fixed-helicity `HELAVGFACTOR=4` imply that
its squared result multiplied by 128 is the color sum without initial spin or
color averaging and without an identical-Higgs phase-space factor. This
independently equals `8|A|²`. The color-averaged fixed-helicity result is
`|A|²/8`.

The native analytic implementation reuses `GraphIntegral`, native contraction,
`oneloopreduce::reduce_family`, `OneLoopMasters` and
`oneloop::evaluate_with_backend`. It combines identical masters and retains
dimension-dependent reduction coefficients through the epsilon expansion,
including coefficients multiplying ultraviolet poles. It does not copy a
master formula, implement tensor reduction, or add independent R2 terms.
MadLoop includes its native two R2 contributions; both calculations therefore
retain finite rational terms by their respective native methods.

The native analytic and numerical cards use the same OneLOop gamma factor at
`mu²=1`. MadLoop uses `mu_R=300`; the full amplitude's ultraviolet and infrared
poles cancel, so this scale difference does not alter the finite amplitude.
Per-diagram finite values are not claimed to agree across different scale
conventions. The full triangle and box partitions agree independently.

## Executed checks

| Quantity | HEPKit one-loop reduction | MadLoop |
| --- | ---: | ---: |
| Real `A_++` | -0.005243877114412127 | -0.005243877114411830 |
| Imaginary `A_++` | -2.4664e-16 | 1.4315e-33 |
| Triangle real amplitude | 0.007453970457652831 | 0.007453970457652831 |
| Box real amplitude | -0.012697847572064918 | -0.012697847572064663 |
| Color sum `8|A_++|²` | 0.00021998597752844208 | 0.00021998597752841711 |

The complex-amplitude relative difference is `7.37e-14`. The native coherent
single pole is at floating-point roundoff and its double pole is zero. MadLoop
has a `1.73e-18` single-pole remainder and zero double pole. Its requested
relative stability is `1e-12`, with successful quadruple precision rescue.
MadLoop's printed zero stability is rounded agreement between its checks,
not an exact-error claim.
The final clean MadLoop process-generation/build/evaluation script passed in
81.94 seconds with sampled peak process-group RSS 1.16 GiB, within the requested
10-minute/15-GiB ceiling. A separate `--reuse` rebuild/evaluation also passed.

Both native Ward substitutions were independently inspected. Replacing the
first or second gluon polarization before contraction gives finite physical
remainders `-2.94e-14` and `+2.94e-14`, respectively. The corresponding native
finite remainder is `4.64e-12` against sum-of-absolute-diagram scale `192.98`;
the single-pole remainder is `4.55e-13`. Both pass the recorded scale-aware
Ward and pole checks.

## Numerical sector integration

Reviewed `fastsecdec/summarize.rs` and the completed eight-diagram
`fastsecdec/result.json`. The summarizer reads validated native result objects,
requires full-integral target-reaching production estimates, and checks that
each recorded QMC seed equals its distinct manifest seed. It forms the union
of Laurent orders/components and adds the full covariance matrices for the
independently seeded diagrams using native Numerica `DoubleFloat` sums. Each
diagram's sector correlations remain in its authoritative native total.
The triangle's loop-independent Higgs propagator is extracted once through
existing native family preparation with explicit `SingleTerm` policy.

The actual run contains 30 sectors and 3,932,160 sector sample evaluations,
using 4096-point lattices, 32 random shifts and the Korobov3 transform. All eight
diagnostic summaries report zero failures and zero unstable points. Native
precision rescue is retained and reported. The complete physical amplitude is

```text
A_++ = -0.005243873099404801 + 0 i
RMS standard error = 4.569190345257295e-9
relative standard error = 8.71338847954943e-7
```

It differs from the independent MadLoop result by `0.879` standard errors.
Each of the eight individual finite coefficients agrees with its native
analytic reference within `1.92` standard errors. The retained real simple
pole is `-5.3153e-8 ± 6.5133e-8` in native normalization and is compatible with
zero. Imaginary components vanish for these real, below-threshold inputs.

The color sum is `0.0002199856406612905 ± 3.833640694008779e-10`. Its stated
uncertainty is first-order propagation with the complete real/imaginary
covariance; this is appropriate at the achieved relative amplitude error.
The result therefore exceeds the required one-per-mil finite-amplitude target
and passes the graph, normalization, full Laurent cancellation, independent
reference and numerical-uncertainty acceptance checks at this benchmark point.
