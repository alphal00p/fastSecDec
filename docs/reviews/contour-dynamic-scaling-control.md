# Nonlinear dynamic contour scaling control

2026-10-10. Bounded exploratory public-API probes on the current uncommitted
runtime implementation with public Symbolica/Numerica `516beb37` and SymJIT
`d74993ff`. These are analytic numerical controls, not physical multiloop
acceptance or a performance comparison. No registered production source changed.

## Exact input and reference

For positive, explicitly declared K, the source input is

\[
 I_K(\epsilon)=\frac1\epsilon\int_{[0,1]^3}
 [K(1-2x)(1+y)(1+z)-i0]^{-\epsilon}\,dx\,dy\,dz.
\]

There are no endpoint monomial powers, extra loop factors, measures or hidden
normalizations. The sole polynomial factor is causal, with exponent `-eps`;
the prefactor is `1/eps`. Generation directly receives the scaled F. The pole
coefficient is 1+0i, including the full complex deformed Jacobian. The finite
coefficient is

\[
 3-4\log 2-\log K+\frac{i\pi}{2}.
\]

At K=1 this is `0.22741127776021886 + 1.5707963267948966 i`.
At K=40000 it is `-10.369223455335854 + 1.5707963267948966 i`.
Since `(1+y)(1+z)>0`, the lower lip contributes +i*pi precisely on x>1/2.
Each y/z logarithmic integral contributes `1-2 log 2`; the real x contribution
is 1. Symbolica verifies derivatives of the separate positive/negative x
primitives and the y primitive exactly. The scale probe additionally checks
the exact native ratio `F_scaled/F=K` and primitive derivative shift by
`-log K`. No branch choice is inferred from a rounded sample.

For the existing native direction `v_j=x_j(1-x_j) ∂_j F_K`, Symbolica's exact
native ray series verifies

\[
 [t^3]\operatorname{Im}F_K(x-itv)=-2K v_xv_yv_z.
\]

This is harmful and nonzero at `[1/4,1/3,2/5]`: approximately .03484444444444444
for K=1 and `8.920177777777778e16` for K=40000. Both envelope constructions retain
two radius coefficients. Thus the controls exercise a nontrivial cubic ray
term and iterative implicit radius, beyond the structural quadratic shortcut.

## Public execution and predeclared protocol

Both constructions use symbolic generation, native Eager evaluation,
`horner_iterations=0`, the native full determinant, complete Laurent vectors,
and serialized/restored programs. Saved bytes roundtrip identically. Public
binding uses S=.8, R=1 and the declared L. No normalization of F, automatic cap
selection, private admission bypass, custom sampler or alternate numerical
implementation is used.

The initial probe fixes K=1,L=1. Before the follow-up run, K=40000 and the two
caps L=1 and L=1e-5 were declared. Each case receives three actual native certified
pilot points: `[.25,.25,.25]`, `[.75,.5,.25]`, `[.5,.5,.5]` in the source chart's
native coordinate order. Each pilot completes three checked arguments at 96 bits.
The public readiness gate passes before production. Production retains policy
Pilot and therefore performs zero certified checks; essential numerical failure
fences remain active. This does not claim pointwise production certification.

The native QMC settings are fixed at 512×8 then 2048×8 (points×shifts), seed 34723,
supplied rank-one lattice `[1,433,1277]`, native Korobov3 periodization,
package_points 128 and evaluation batches 64. The complete native covariance is
retained and printed for every result. No data from different work sizes, caps
or recipes are pooled. The two work sizes share their declared seed and are
not independent replications.

The scaled follow-up additionally records and compares every actual native
coordinate and weight bit-for-bit across both caps and constructions at each
allocation. All comparisons pass: 16,512 trace words at 512 points and 66,048 at 2048
(including batch metadata). The earlier unscaled probe uses the same declared
settings, but did not record this explicit cross-case trace. The trace-only
rerun preserves the initial scaled numerical outputs exactly.

The predeclared disagreement screen was eight estimated standard errors or an
absolute 2e-6, whichever larger. It is an exploratory screen, not a claim of
convergence. Every coefficient below remains visible even when its mean is
far from the exact value. Entries show mean ± one native standard error;
cross-component covariance is in the hash-identified run logs.

## Complete uncertainty tables

### K=1

| Construction | L | Points × shifts | Pole real | Pole imaginary | Finite real | Finite imaginary |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Polynomial | 1 | 512 × 8 | 1.000116923 ± 0.00013375 | 9.5131136e-05 ± 0.00010067 | 0.2272546868 ± 0.00027683 | 1.570867232 ± 0.00050891 |
| Polynomial | 1 | 2048 × 8 | 1.000152798 ± 0.00013128 | 5.318637287e-05 ± 5.6987e-05 | 0.2272455066 ± 0.00025066 | 1.571064635 ± 0.00023083 |
| SignAware | 1 | 512 × 8 | 1.000118027 ± 0.00013423 | 9.508838185e-05 ± 0.0001005 | 0.2272539358 ± 0.00027723 | 1.570869701 ± 0.00050914 |
| SignAware | 1 | 2048 × 8 | 1.000153818 ± 0.00013214 | 5.321769221e-05 ± 5.7327e-05 | 0.2272447008 ± 0.00025214 | 1.571066801 ± 0.00023263 |

All unscaled components lie within 1.18 estimated standard errors of the exact
reference. This supports the nonlinear three-dimensional Jacobian/implicit-root path for this
control, without establishing monotonic convergence or other physical inputs.

### K=40000

| Construction | L | Points × shifts | Pole real | Pole imaginary | Finite real | Finite imaginary |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Polynomial | 1 | 512 × 8 | 0.9760730165 ± 0.027185 | 0.006309680055 ± 0.0069092 | -10.13010948 ± 0.30452 | 1.463049127 ± 0.11815 |
| Polynomial | 1 | 2048 × 8 | 0.9965597124 ± 0.016781 | -1.830184511 ± 2.9092 | -6.046090448 ± 6.6732 | 21.53380608 ± 33.634 |
| Polynomial | 0.00001 | 512 × 8 | 0.9999857177 ± 3.3709e-05 | 4.00939525e-05 ± 0.00011007 | -10.36910881 ± 0.00041535 | 1.570234544 ± 0.0014354 |
| Polynomial | 0.00001 | 2048 × 8 | 0.9999973578 ± 3.6485e-06 | 1.418369378e-07 ± 4.0615e-07 | -10.36920938 ± 2.9949e-05 | 1.57077197 ± 2.4987e-05 |
| SignAware | 1 | 512 × 8 | 0.9756399376 ± 0.027548 | 0.006619116578 ± 0.0069733 | -10.12607207 ± 0.30849 | 1.458919435 ± 0.11928 |
| SignAware | 1 | 2048 × 8 | 0.9962524231 ± 0.016944 | -1.836082755 ± 2.9168 | -6.029739515 ± 6.6893 | 21.59958941 ± 33.72 |
| SignAware | 0.00001 | 512 × 8 | 0.9999857143 ± 3.3711e-05 | 4.009398256e-05 ± 0.00011007 | -10.36910878 ± 0.00041537 | 1.570234534 ± 0.0014355 |
| SignAware | 0.00001 | 2048 × 8 | 0.9999973543 ± 3.6511e-06 | 1.421425651e-07 ± 4.0424e-07 | -10.36920935 ± 2.9965e-05 | 1.570771956 ± 2.4987e-05 |

The default-cap scaled estimates are **unconverged**. At L=1, increasing the
allocation exposes much larger imaginary-pole and finite-vector fluctuations;
the 2048-point polynomial finite estimate is about −6.05+21.53i with standard
errors 6.67 and 33.63. Statistical compatibility from these broad errors is not
numerical accuracy. The sign-aware result exhibits the same behavior.

At the predeclared L=1e-5, all components are within one estimated standard
error, and the finer finite errors are about 3.0e-5 and2.5e-5. No pilot, callback
or nonfinite error occurred in any case. Pilot-only validation limits the
interpretation of that statement as described above.

Multiplying F scales the direction and its derivatives, so the same numerical
cap can produce very different sampling structure. The observed deterioration
and its cap sensitivity support a sharp-Jacobian/high-variance hypothesis.
They do not by themselves locate the large-contribution regions, prove the
hypothesis, or establish a universally suitable cap. No default setting is
changed by this probe.

The separate [physical 400 GeV review](contour-dynamic-physical-readiness.md)
records a materially inconsistent default-cap FK05 box vector despite passing
causal checks. Its pole/finite discrepancy is far larger than its reported
sampling errors. **That failure remains unresolved.** The analytically controlled
scaled-cube variance, and the box's smaller-cap agreement, cannot excuse it or
prove default-cap physical correctness. The next physical acceptance still
needs independent reference agreement and adequate sampling of its actual
multidimensional density.

## Bounded evidence and reproduction

Both probes exit 0. The unscaled run takes 1.571 s with peak RSS 21,520 KiB; the scaled
trace run takes 2.924 s with peak RSS 21,520 KiB. Each launcher enforces 180 s runtime
and 3 GiB address-space limits. These acceptance durations include generation and
restoration and are not comparative performance measurements. Symbolica reports
an invalid offline license signature and uses its restricted one-thread mode;
these serial probes complete under that mode.

The ignored sources are `target/contour-nonlinear-cube-probe.rs` and
`target/contour-scaled-cube-probe.rs`. Their adjacent `*-run.py` launchers record
wall time, exit status and process resource usage in `*-result.json`.
Compilation used the existing 516 native rlibs with direct `rustc --edition=2024
-O`, `CARGO_CRATE_NAME` set and `-L dependency=target/debug/deps`; it acquired no
shared Cargo lock. The exact rlib paths/hashes below identify the tested snapshot
independently of the still-uncommitted workspace. Build/stdout/stderr logs and
the initial scaled run are retained under those target prefixes. These ignored
files are not publication artifacts; the formulas, settings, table and hashes
in this review are the retained evidence.

SHA256 identities:

| File | SHA256 |
| --- | --- |
| `target/contour-nonlinear-cube-probe.rs` | `975d56724207e1957c1a9e972ea823b0e333d1ba556425082119de32f4cef575` |
| `target/contour-nonlinear-cube-probe.log` | `1dc125c31f5ee8b8e6a341430db54b670dfcaa00e3b69d7a9b6b65240811b356` |
| `target/contour-nonlinear-cube-result.json` | `b99c48af9c7f5462441d7ed1a5e381d1c1883135d11588d8cec830c2007211a8` |
| `target/contour-scaled-cube-probe.rs` | `67c23cf754d4160bd6421012d26d2ac4052d60da18115026f7e55d04debf7a7e` |
| `target/contour-scaled-cube-probe.log` | `a17039b223dd97a1880a979f89432bf4bdf55c5c24f8fabf594adf42a00d2c06` |
| `target/contour-scaled-cube-result.json` | `0bb1a50ed65586ea49c05337b80c8130fe7766c4e9130f5a4086555eee85874d` |
| `target/debug/deps/libfastsecdec-88a3f23976bd40a9.rlib` | `1710f929f7d08b75365aed28f66ccf320ff72265f99b302ffc00ac637d3291aa` |
| `target/debug/deps/libsymbolica-0a72932a9eb0c783.rlib` | `d8d7eb3ba5170a9cfc5d4c6015eafd78f58e240998c5af8733568c74d7106dfc` |

No registered regression or new variance protocol was introduced here. Promoting
this bounded control and expanding its independent-seed evidence remain separate
follow-up work; aggregate diagnostics implementation is still pending.
