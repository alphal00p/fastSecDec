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

The matched comparison uses one corrected release, candidate6,
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

## Fixed source0 result and runtime cost

Both fixed arms completed with exit zero, empty process groups and no resource
limit. Each produced one six-dimensional residual with ordered coordinates
`t0` through `t5`, Symbolic endpoint mode, and complete pole/finite real/imaginary
components. The native exact owner was included once in the pointwise comparison.

| Observation | Symbolic J | Contour-only Dual J |
| --- | ---: | ---: |
| Generation through symbolic coefficient expansion | 378.423 s | 368.668 s |
| Compilation | 180.839 s | 197.559 s |
| Native generation, save and checked values | 559.597 s | 566.549 s |
| Monitor elapsed | 559.897 s | 566.864 s |
| Sampled owned RSS peak | 12,010,889,216 B | 12,014,661,632 B |
| Mapped source record | 365,538,755 B | 368,942,496 B |
| Whole saved owner | 2,348,670 B | 745,241 B |
| Exact evaluator IR | 108,301 B | 136,831 B |
| SymJIT IR | 156,211 B | 202,835 B |
| Surviving first-image partial input slots | Inapplicable | 6 |

The smaller Dual whole-owner file does not mean a smaller compiled evaluator:
both evaluator representations are larger in that arm. These are single
concurrent generation observations, with no established generation speedup or
amortization benefit.

Both arms admitted fixed strength `1e-6` and refused `1e-5` with the same certified
positive-imaginary-F diagnosis. Separate fresh processes restored each saved
owner after generation exited. All eight scalar values matched their own
generated owner exactly. Across Jacobian choices the maximum absolute difference
was `5.684341886080802e-14`, and maximum scaled difference was
`3.4312339600082543e-15`, below the unchanged `2e-10` comparison tolerance.
Native layouts, coordinate schemas, points and admitted/refused cap sets matched.
Restored build observations are unknown, as intended. The hash-pinned records
are `parity-fixed.json` and `cost-handoff-fixed.json` in the candidate6 directory.

The subsequent native Kuo `4096 × 2` cost comparison used the same 8,192 actual
coordinates and weights, verified by their digests, at fixed strength `1e-6`.
Both owners completed Pilot16/readiness, including the actual source's exact
obligations. Each accepted 8,165 points in f64 and rescued 27 in double-double;
neither used arbitrary precision or optional production causal checks.

| Native weighted evaluation cost | Symbolic J | Contour-only Dual J |
| --- | ---: | ---: |
| Evaluation elapsed, including precision retries | 0.119918 s | 0.157260 s |
| Mean per assigned sample | 14.638426 µs | 19.196787 µs |

The observed Dual/Symbolic cost ratio is `1.311397`. This mean includes first-use
work inside evaluation and all precision attempts; it excludes restore, binding,
pilot, context construction and coordinate hashing. It is a source0 measurement,
not an integral estimate, all-sector cost or 50-worker result. The preceding
`32 × 2` run is retained as cold feasibility only. The larger cost monitor closed
in 3.396 s with 39,952,384 B sampled peak. Its complete native metrics and hashes
are in
`target/contour-d05-1000-runtime/symbolic-endpoints/cost-fixed-matched_4096x2/cost-120s/matched-cost-review.json`.

## Full fixed generation in progress

The source0 evidence supports choosing Symbolic J for the complete fixed
artifact. A fresh native serial journal began at
`2026-10-10T10:53:11.886830Z`, using six workers, a 7,200-second allocation and a
100,000,000,000-byte combined memory guard. The coordinator is PID/PGID 1514885;
the immutable candidate6 CLI SHA256 is
`e167fc2d3807ea16e9a83e2e439dd12e23862b8dde4c68fe120d69ea89191544`.
Settings remain Symbolic endpoint IBP, Symbolic J, coefficient series starting
at the existing width-one default, SymJIT O2, Horner zero and CPE cap 1000.
Source0 receipts were not grafted into the fresh journal.

The monitor accounts for the concurrently active, PID/start-time-pinned
polynomial source0 probe and its monitors, but can terminate only this full
generation's owned group. Own-group and combined RSS are recorded separately.
The native geometry still contains 30 source charts; eventual residual counts
and dimensions will come from the generated catalogue. Raw plans, authorization,
status and resource evidence remain under
`target/contour-gghh-double-box-1000-symbolic-endpoints-full-fixed-c6/`.
Full publication, physical admission and the four final integration runs remain
pending.

## Polynomial source0 qualification

The same candidate6 Symbolic-endpoint/Symbolic-J probe completed its polynomial
dynamic source0 unit in 1,511.568 monitored seconds, with a sampled peak of
28,441,194,496 B, exit zero and an empty process group. Generation through
symbolic coefficients took 1,004.993 s; compilation took 504.993 s. The first
relative-width attempt was discarded before width two satisfied coverage.
The result contains one six-dimensional residual, with exact offsets accounted
for once and the same complete four-component Laurent layout.

The saved whole owner is 5,330,684 B; its exact evaluator IR is 294,022 B and
SymJIT IR is 524,646 B. Both tested dynamic caps, `1e-6` and `1e-5`, passed native
validation at `S=0.8`, `R=1`. A separate restore process completed in 1.179 s
with 61,837,312 B peak; all 16 scalar values matched the generated owner exactly.
`polynomial-symbolic-restore-review.json` pins these source0 reports and saved
bytes. This is pointwise qualification, not complete-chart admission or an
integral estimate.

The matched polynomial contour-only Dual-J source0 began at
`2026-10-10T10:59:43.008343Z`, native PID/PGID 1748578, with a 3,600-second
allocation. It retained candidate6, Symbolic endpoints and the same width-one
starting policy. It completed with exit zero, an empty process group and no
resource limit. Its guard counted the concurrent fixed generation separately;
the sampled combined peak was 49,430,777,856 B.

| Polynomial source0 observation | Symbolic J | Contour-only Dual J |
| --- | ---: | ---: |
| Generation through symbolic coefficient expansion | 1,004.993 s | 506.307 s |
| Compilation | 504.993 s | 287.072 s |
| Monitor elapsed | 1,511.568 s | 794.947 s |
| Sampled owned RSS peak | 28,441,194,496 B | 16,727,838,720 B |
| Whole saved owner | 5,330,684 B | 1,676,683 B |
| Exact evaluator IR | 294,022 B | 372,004 B |
| SymJIT IR | 524,646 B | 519,988 B |
| Surviving first-image partial input slots | Inapplicable | 11 |

Both fresh restores reproduce their own 16 scalar components exactly. The
cross-Jacobian maximum absolute difference is `1.1368683772161603e-13`, and the
maximum scaled difference is `6.0126526211687044e-15`, below the unchanged
`2e-10` tolerance. Coordinate schemas, layouts and admitted caps agree.
`parity-polynomial.json` and `cost-handoff-polynomial.json` retain the pinned
native owners, generation reports and independent restore evidence. The lower
generation time and memory are observations from one source0 comparison, not
an all-chart scaling result. Its exact evaluator IR is larger despite the
smaller whole saved owner.

Native Kuo `4096 × 2` sampling at `S=0.8`, `L=1e-6`, `R=1` used identical
8,192 coordinate/weight pairs in both arms. Each owner passed Pilot16/readiness,
accepted 8,164 points in f64 and rescued 28 in double-double. Neither used
arbitrary precision or optional production causal checks.

| Native weighted evaluation cost | Symbolic J | Contour-only Dual J |
| --- | ---: | ---: |
| Evaluation elapsed, including precision retries | 0.308934 s | 0.245677 s |
| Mean per assigned sample | 37.711716 µs | 29.989820 µs |

The observed Dual/Symbolic cost ratio is `0.795239`. The timing scope is the
same as the fixed comparison: all native evaluation attempts and first-use work
are included; restore, binding, pilot, context construction and coordinate
hashing are excluded. The monitor closed in 2.839 s at 67,579,904 B peak.
The raw native timings, matching digests and precision counts are pinned by
`target/contour-d05-1000-runtime/symbolic-endpoints/cost-polynomial-matched_4096x2/cost-120s/matched-cost-review.json`.
This is source0 cost evidence, not an integral or final 50-worker result.

## Full polynomial generation selection

The source0 generation, parity and cost evidence supports selecting contour-only
Dual J for polynomial deformation; fixed continues with Symbolic J. A fresh
candidate7 journal began at `2026-10-10T11:20:38.041522Z`, PID/PGID 2862610,
with two serial workers, a 7,200-second allocation and the combined
100,000,000,000-byte guard. Symbolic endpoint IBP, SymJIT O2, Horner zero and
CPE cap 1000 remain unchanged. Native coefficient expansion starts at relative
width two, with the existing absolute-coverage check and retry logic.
The candidate7 CLI SHA256 is
`7b4563df8392e884267a9aed2a85fa9a37d04a4ede0142b31641af9ffdda9243`;
its source archive SHA256 is
`38222a65b58e6dcf9949d863771146808c74941c2b2f6e217f12c19281e744ab`.

The earlier candidate7 full Symbolic-J attempt was deliberately cancelled
through the native coordinator after the selection. Its mapped records, native
cancellation status and resource observations remain intact; it produced no
completed stochastic unit before cancellation. The new journal imports none
of its receipts. Both attempts and the continuing full fixed campaign remain
under their separate ignored directories. Full polynomial publication and
all-chart physical admission are pending.

## Definition registration follow-up

A bounded compiler profile motivated removing unused owned copies of compact
function arguments during native FunctionMap registration. Native traversal,
derivative bodies and error checks remain unchanged. The old/new focused probe
produced identical exact evaluator bytes; its small timing difference does not
establish a physical compilation speedup. Sixteen focused native controls and
independent source/test review passed. The API, source, probe and verification
record is [the definition registration audit](contour-definition-registration.md).
This change is separate from the immutable binaries used in the measurements
and continuing generation campaigns above.
