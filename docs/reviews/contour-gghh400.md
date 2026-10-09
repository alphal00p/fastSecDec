# Fixed contour: complete one-loop ggHH above threshold

2026-10-09. This is a fixed-mode scientific gate, not completion of Phase B
or a dynamic performance comparison. Compact numerical evidence and executable
hashes are in
[the reproduction](../../example/gg_hh_one_loop_ME/threshold/README.md).

## Shared physical input and independent calculations

The existing native generator exports its complete two-triangle/six-box top-loop
catalogue at `sqrt(s)=400 GeV`, `mH=125 GeV`, `mt=172.5 GeV`, and `cos(theta)=4/5`.
`Point::with_sqrt_s` uses native exact arithmetic and `FourMomentum`; numerical
incoming helicities still come from the shared HEPKit/GammaLoop implementation.
The 300 GeV entry point remains the default. Every diagram must agree on the
exported physical point and external labels.

The same native point supplies the reference JSON. HEPKit performs its native
one-loop reduction and OneLOop evaluation; MadLoop independently generates the
eight diagrams and its rational terms in a private installation. No new reduction,
master formula, graph parser or polarization implementation was added.

HEPKit and MadLoop agree to `3.90e-15` relative precision. Their common amplitude
is approximately `-0.0194295209824 - 0.0117460318328 i`, with the established
`M_ab = delta_ab A++` colour convention and physical loop factor. Both native
Ward substitutions vanish and the reference Laurent poles cancel. MadLoop's
private generation/build/evaluation took 80.98 seconds within its ten-minute,
15-GiB limit; this is reference execution evidence, not a timing comparison.

## Numerical contour result

FastSecDec used fixed lambda `1e-6`, an explicit checked pilot, unchecked
production, eight caller-owned CLI workers and SymJIT O2 kernels. Recorded
production causal-check counters are zero. The retained result is

`A++ = -0.0194290118804 - 0.0117485831701 i`,

with joint real/imaginary standard error `6.72874e-6` and relative uncertainty
`2.96355e-4` (0.0296%). It lies 0.38665 standard errors from both references.
The full Laurent vector, covariance, diagram seeds and pole checks are retained;
the squared amplitude is derived with the existing covariance propagation.

The initial six-round attempt imposed a per-diagram relative target `2e-5`.
Four boxes exhausted that limit, and the combined uncertainty was `0.1045%`,
slightly above the requested `0.1%`. Those boxes were rerun as separate sessions
with up to eight rounds and target `1e-4`. All four reached that target. Only
their replacement estimates enter the accepted sum; the initial estimates are
never pooled with them. The four already completed simple diagrams retain
their original estimates. The revised reproduction script uses the accepted
`1e-4`/eight-round settings from the outset.

This gate used a debug-profile driver, preserving the existing release binary
and unrelated host workloads. Its integration kernels used the normal SymJIT
O2 backend. The working tree required the documented Symbolica ball-domain
and SymJIT complex-callback patches. No optimized driver timing or dynamic
variance improvement is claimed.

## Independent review

The runtime agent reviewed the native point generalization, exporter and
script. It confirmed the unchanged 300 GeV default, exact mass-shell and
momentum-conservation construction, shared point provenance, fresh output
directories, distinct diagram seeds and unchanged normalization. The existing
native comparison program also passes against the compact retained records.

The next corresponding gates are fixed/dynamic strength agreement, matched
production coordinates with repeated independent seeds, full covariance-based
variance comparisons, and actual runtime cost. The physical double box and
required LTD two-/three-loop examples remain separate outstanding gates.
