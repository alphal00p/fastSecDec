# Physical generation with symbolic endpoint reduction

This audit concerns the user's corrected comparison: symbolic endpoint IBP in
both arms, with either a symbolic contour Jacobian or contour-only first-order
dual evaluation. The older [generation campaign](contour-gghh-1000-generation.md)
and [Jacobian timing appendix](contour-dual-jacobian.md) used NumericalDual
endpoint reduction and are not measurements of this comparison. The
[independent mathematical/API audit](contour-symbolic-endpoints.md) records the
new construction and its limits.

## Scope and immutable inputs

The input is the native D05 ggHH double box at 1000 GeV, incoming `++`,
`cos(theta)=4/5`, Higgs mass 125 GeV and top mass/Yukawa 172.5 GeV. All 19
runtime bindings, native graph, colour projection, numerator and loop measure
are unchanged. No diagram sum or spin/colour average is introduced. Native
source identity is
`d13a5cdc7aa156d2630b78ffc2738c732a17f7b26ea2b47f7e8b72bca934d414`.

Source qualification imports the original native StateMap and receipts. It
compares every common term's prefactor, powers, residual polynomials, exponents
and branch semantics, plus source/map identities and ordered coordinates.
NumericalDual-only trailing opaque preparation data is absent in the new
Symbolic preparation; it is not mathematical source data. The first baseline
attempt incorrectly compared that mode-specific payload and failed before
generation. Its source and failure are preserved under the ignored baseline
directory; the corrected comparison does not omit a common mathematical field.

The original geometry contains 30 six-dimensional source charts. Symbolic
reduction may produce a different residual-sector inventory, so generated
sector counts, dimensions and coordinate order are read from actual owners.
Source0 qualification is a complete selected-unit Laurent-vector comparison,
including exact offsets once, not an integral estimate or full-30 admission.

## Existing Symbolic-J baseline

The first feasibility run used immutable candidate4, source archive
`e6f32078e8ae5b0fee4423d260ea96053ecf0688fff46760e2a18c294b8451c3`,
native core SHA256
`3639e5e9d9b21954461d47df22290fd3c59a9daa6eb8326302c67aa7423a6888`,
and probe SHA256
`d52e74932c4cc30fd32b027ac43eab638f50cf788d226719f53a4e0154a39a39`.
The public Symbolica/Numerica owner revision is `74225696`.

Settings were Symbolic endpoint reduction, genuine IBP, Laurent order zero,
the existing coefficient-series strategy, fixed deformation and Symbolic J.
Compilation used one caller, release opt3/thin-LTO, SymJIT O2, Horner zero and
CPE cap 1000. No runtime point was used to simplify the parametric source.

| Observation | Result |
| --- | ---: |
| Native generation through symbolic coefficient expansion | 416.141 s |
| Compilation | 174.597 s |
| Native generation, save, checked values | 591.097 s |
| Original monitor elapsed | 591.773 s |
| Extension watchdog elapsed from original UTC start | 592.366 s |
| Peak sampled owned RSS across both monitoring intervals | 12,124,004,352 B |
| Saved selected-unit evaluator | 2,348,670 B |
| Fresh-process restore and checked values | 0.583 s; 21,520,384 B peak |

Symbolic coefficient expansion retried relative width one with width two.
Regular-series construction and lowering account for most generation time;
this run does not measure contour-only dualization. It produced one stochastic
residual sector with complete real/imaginary pole and finite components.

Fixed strength `1e-6` passed actual native contour validation/readiness and two
interior full-vector evaluations. Strength `1e-5` was refused with a certified
positive imaginary part of F along the homotopy. The refusal remains in the
record. A separate process restored the saved owner after the generator had
exited; all eight admitted scalar values matched exactly, with identical
layouts, points and refusal. No integration or variance claim follows.

The initial plan had a 600-second limit. Before completion, the parent explicitly
authorized 1800 seconds from the original start. A reviewed watchdog first
recorded readiness and PID/start-time identities, stopped only the old monitor,
and continued the same native process under the unchanged 100 GB cap. It resumed
the old monitor for reaping only after native work exited. Both evidence streams
and their RSS bridge are retained. Although work happened to finish before 600
seconds, acceptance is recorded under the already-active extension. No native
process was restarted. RSS polling sleeps were 50 ms, with additional `/proc`
scan time; these are sampled process peaks, not a guarantee of continuous maxima.

Raw files remain ignored under
`target/contour-gghh-double-box-1000-symbolic-endpoints-baseline/`.
`baseline-review.json` pins the build, original and extended execution records,
saved owner and full-vector reports. The independent raw review is
`target/foundation-symbolic-endpoints/candidate4-physical-baseline-review.json`.
Both monitors closed with exit zero, empty owned groups and no resource limit.

## Matched candidate6 qualification

The matched comparison is prepared against one corrected release, candidate6,
source archive
`2c80fc91e863cbfed38e06ccdefc6b25e7c04d2e2015d0d41c724ffeb08f39ed`.
Its release probe SHA256 is
`645dc6221dc2634c4f8f9ab719b673e34c260f235e364bf11798c6fe9df5a7c4`.
The earlier candidate5 snapshot is superseded: a genuine source-greater-than-zero
fresh-worker test exposed confusion between record-local and original source
indices. The corrected staged test passes while retaining native image/Jacobian
equality and job/source association checks. The candidate4 baseline above is
feasibility evidence; it does not substitute for either matched candidate6 arm.

The source0 protocol uses the same input, Symbolic IBP and compiler settings for
both Jacobian choices, 1800 seconds per arm and a combined 100,000,000,000-byte
owned-process limit. Fixed arms and the polynomial Symbolic-J baseline may run
concurrently. Complete native layouts, actual residual coordinate schemas,
admitted/refused cap sets and full vectors must agree before runtime cost is
compared. Fresh restore is a separate bounded process. The three-arm batch began
at `2026-10-10T10:36:19.77Z`, with original native PIDs 552671 (fixed Symbolic J),
552680 (fixed Dual J), and 552682 (polynomial Symbolic J). The reviewed combined
watchdog pins PID/start-time identities, includes its own memory and the thin
monitors, and leaves monitors alive for reaping before bounded failure cleanup.
Its inputs and observations remain under the ignored
`target/contour-gghh-double-box-1000-symbolic-endpoints-c6/` directory.

The Dual-J build must report surviving contour first-partial input slots greater
than zero. This is an actual compiler observation, not inference from a policy
tag. It is distinct from callback/sample counts and becomes unknown on restoration.
Exact-only contributions retain native symbolic semantic materialization.
Endpoint algebra never uses the full-density/endpoint Dualizer. Any unbound
higher derivatives remain native symbolic arithmetic. Opt-in Dual-J compilation
rejects active control-flow placement rather than eagerly executing conditional
image callbacks.

At this checkpoint, matched candidate6 physical results and runtime costs are
pending. No relative generation speed, sampling speed or amortization is claimed.
The full fixed/dynamic generation choice will follow these representative
measurements; full physical admission and the four final integration runs remain
separate gates.
