# Bounded double-box lattice follow-up

The latest single current-source-equivalent Korobov2 observation reaches
**0.714‰** in **172.715 seconds** on eight workers. Its full result and limits
appear in the final section; the historical records below remain unchanged.

The native finite coefficient reaches **0.393‰ reported relative standard error**
at 16,384 HKKN points and sixteen shifts on eight workers. That level takes
437.039 seconds; both tested HKKN levels together take 664.554 seconds. All five
real reference comparisons pass. This is one observed path to the requested
precision, not matched Pathfinder parity or an uncertainty-calibration result.

The retained double-box generation already has three completed pairs: median
4.219 seconds native and 35.839 seconds Pathfinder. The next useful investigation
concerns convergence, using the existing artifact rather than another generation
or coefficient-representation experiment.

Two native HKKN alpha-three observations reuse the frozen `8ecc406` executable
and all 102 complete-vector kernels. The first changes only the published lattice
catalogue relative to the retained 8,192-point Kuo row: sixteen shifts, seed
20561302, Korobov3, 1,024-point packages, eight workers and ordinary precision
rescue remain fixed. The second doubles the lattice size with every other
setting fixed. All rows retain five real coefficients through epsilon zero.
These are historical-build
observations, not benchmarks of current main or a matched Pathfinder comparison.

| Quantity | Kuo33002, N8192 | HKKN, N8192 | HKKN, N16384 |
|---|---:|---:|---:|
| Complete process wall time | 209.507 s | 227.514 s | 437.039 s |
| Accepted evaluations | 13,369,344 | 13,369,344 | 26,738,688 |
| Finite coefficient | −14.240031019 | −14.835574600 | −14.854766319 |
| Finite standard error | 0.477426162 | 0.019628418 | 0.005837250 |
| Finite relative standard error | 3.3527045% | 0.1323064% | 0.0392955% |
| Mean accepted worker time per sample | 124.028 µs | 134.478 µs | 128.585 µs |
| Largest sector mean per sample | 798.727 µs | 854.966 µs | 810.789 µs |
| Evaluation failures | 0 | 0 | 0 |

The worker interval includes lattice generation, transformation, complete-vector
evaluation, rescue and accumulation. Its largest sector average is not an
individual-sample maximum. Compilation on separate CPUs and unrelated host
workloads were not excluded, so these single elapsed times do not establish a
timing regression or improvement.

The effective HKKN generator is `[1,4533,7821,7711,5611,5265]` modulo 8192 in every
sector. It matches Numerica's existing attributed catalogue. Exact integer
arithmetic finds the short frequency `(1,1,1,1,2,0)` in the original Kuo dual
lattice throughout the tested 1,024–8,192 sizes, and outside this HKKN lattice.
The result is consistent with the earlier
[constant-integrand and physical controls](six-line-qmc-convergence.md).
It does not imply invalid Kuo sampling or justify changing the universal default.

All five real coefficients agree with the existing independent reference within
1.418 and 1.098 combined standard errors at the two respective HKKN levels. The
complete covariance is retained. The final run records 12,231,536 rescues,
maximum precision 576 bits, 116 weighted checks, 64 additional replays and no
failures. Its finite relative error is **0.393‰**, meeting the requested **1‰**
reported-error target. Reference compatibility is not a
calibration certificate for the estimated uncertainty.

The frozen data reader admits the previously unreviewed Kuo 8,192-point row
without resampling. For HKKN, the ordinary native `show-result` command validates
a separate result copy containing only the existing reference and its explicitly
recorded comparison context. Every other field and numerical float bit remains
unchanged; the original result is retained. No previous comparison outcome was
transplanted. The first attachment attempt rejected an enum spelling before
creating a copy; that error and the narrow correction are preserved.

Evidence is retained in
`output/diagnostics/bounded-double-box-followup-{1,2}/{REPORT.md,summary.json}`.
Independent review checked both sets of principal file bindings, native
comparisons, effective catalogues, accepted work and all 103 total/sector
covariance matrices in each run. Both processes exited successfully under their
300+5-second and 600+5-second bounds, with a 30-GiB address-space limit, and were
reaped. The cumulative 664.554-second figure includes both complete HKKN
executions; no earlier samples were reused. It excludes the old Kuo ladder and
generation, and is not a timing median. Numerical work stops here. No production
default, mathematical source, prepared artifact or historical benchmark output
changes. Subsequent current-build validation and a Pathfinder counterpart are
recorded below; representative timing parity remains open.

## Current release and Pathfinder counterpart

The published-source release now generates the original scalar double box in
4.441 seconds and passes the normal artifact reader. All 102 six-dimensional
kernels, orders −4 through zero and the entire native kernel envelope equal the
retained artifact. Its new outer identity correctly records the changed
dependency provenance. Independent review checked the executable, all 4,039
source bindings, unchanged physics and strict loading.

A fresh card changes only `integration.package_points` to 16,384, with input
paths made absolute. Ordinary generation takes 3.793 seconds and again produces
the identical kernel envelope. The ensuing current-build integration and the
separately admitted Pathfinder default-catalogue observation give:

| Quantity | Current FastSecDec | FastSecDecPathFinder |
|---|---:|---:|
| Lattice / actual points per shift | HKKN / 16,384 | Default prime / 17,807 |
| Shifts / workers | 16 / 8 | 16 / 8 |
| Complete integration/result process | 479.451 s | 275.506 s |
| Accepted evaluations | 26,738,688 | 27,351,552 |
| Finite coefficient | −14.854766319 | −14.870171266 |
| Reported finite relative error | 0.393‰ | 0.577‰ |
| Mean time per sample, respective interval | 142.312 µs | 58.675 µs |
| Largest sector mean, respective interval | 969.263 µs | 442.739 µs |
| Individual-sample maximum | Unmeasured | Unmeasured |

The native sample interval includes point generation, transformation, complete
vector evaluation, rescue and accumulation. The Pathfinder interval above is its
evaluator-plus-Python bucket; global lattice/transform and prefactor work lie
outside it. Its evaluator-only mean and largest sector mean are 50.137 and
426.772 microseconds. These are unlike intervals, not scalar-JIT speed ratios.
Native uncertainty uses the joint physical vector covariance; Pathfinder reports
marginal errors with L1 prefactor propagation. Both pass all five available real
reference checks. The reference has no imaginary rows; these are not invented.

The larger native packages preserve every actual point and shift. All sector
estimates, total means/errors/covariance and precision counters are exactly equal
to the retained 1,024-point-package result, including 12,231,536 rescues, maximum
576 bits, 116 weighted checks, 64 additional replays and zero failures. This
closes current-build numerical validation for the observation. It shows no
timing improvement over the historical 437.039-second run; changed executables
and host activity prevent attributing the difference solely to package size.
The default remains unchanged. A rejected data-only partial-count assumption and
its correction are retained; no scientific execution was repeated.

Pathfinder reuses its existing bundle and normal precision policy, requests
16,384 points and selects prime 17,807 with vector
`[1,6801,7999,5312,2438,2316]`. It uses the existing table only; no FORM or pySecDec
generation runs. Its single observation passes independent data review. Neither
row is a timing median or an identical-rule comparison. Double-box performance
parity remains unmet, and the bounded package experiment ends here.

Evidence is in `output/diagnostics/current-native-release-build-1`,
`current-double-box-generation-1`, `current-double-box-package16384-1` and
`bounded-double-box-prime-counterpart-1`. The independent current-result audit
accepts all 1,632 complete packages, 103 total/sector covariance matrices,
unchanged native plans and the reference-only transport. Every scientific
process completed within its existing bound and was reaped.

## One current Korobov2 observation

The existing published Korobov2 transform reaches the requested finite-part
reported precision in one HKKN alpha-three allocation, N8192/R16,
seed20561302, eight workers and 1,024-point packages. The graph, kinematic point,
complete five-real-component vector, precision/rescue policy and reference
convention are unchanged. This is one different-transform observation, not a
matched-transform Pathfinder comparison or a calibrated uncertainty claim.

| Quantity | Current FastSecDec, Korobov2 | Retained Pathfinder, Korobov3 |
|---|---:|---:|
| Lattice / actual points per shift | HKKN / 8,192 | Default prime / 17,807 |
| Shifts / workers | 16 / 8 | 16 / 8 |
| Complete integration/result process | 172.715 s | 275.506 s |
| Accepted evaluations | 13,369,344 | 27,351,552 |
| Finite coefficient | −14.861132552 | −14.870171266 |
| Reported finite relative error | 0.714‰ | 0.577‰ |
| Mean worker/bucket time per sample | 102.060 µs | 58.675 µs |
| Largest sector mean per sample | 623.643 µs | 442.739 µs |
| Individual-sample maximum | Unmeasured | Unmeasured |

The native elapsed integration interval is 172.333 seconds, with 0.353 seconds
for artifact loading; the table uses the complete 172.715-second process.
Sample intervals remain different: native worker time includes points,
transformation, complete-vector evaluation, rescue and accumulation, while the
Pathfinder bucket excludes its global lattice/transform and prefactor work.
The time-to-reported-precision observation improves, but these rows do not
establish matched parity or an individual-sample latency comparison. No default
transform or precision setting changes.

Fresh generation by the accepted archived optimized a3/f6 executable takes
2.443152 seconds excluding compilation plus 0.367073 seconds for SymJIT O2
compilation, or 2.810225 seconds total native generation and 2.825224 seconds
process wall. The strict current reader accepts the new artifact. Every native
numerical payload field and every legacy metadata field equals the old scalar
artifact exactly after removing only the newly added per-chart
`pre_subtraction` records. The new content ID and updated Numerica provenance
are retained, without pretending the complete serialized envelope is unchanged.
The archived binary's accepted equivalent Option-spelling difference from
published a3 is explicitly bound in its source-equivalence receipt.

All 102 six-dimensional sectors complete all sixteen shifts. The full vector,
all 103 total/sector covariance matrices, their standard-error diagonals and
unchanged effective lattice are checked. Every available reference component
passes, with maximum absolute pull 1.146913. The run records 6,219,701 rescues,
448-bit maximum precision, 115 weighted checks, no additional replays and zero
failures. The finite estimate is −14.861132551599596 ±0.010604567935994033.
Near-zero pole contributions retain their measured errors; none is declared
symbolically zero.

The first input-only prefix rejected an old bare `ReferenceResult` before any
algebra or sampling. The documented version-one envelope then wrapped those
original reference bytes unchanged. That failure remains in
`output/diagnostics/bounded-double-box-korobov2-2/`; it was conservatively charged
two seconds, leaving 398 seconds of the original 400-second cumulative cap for
the sole scientific allocation. There was no numerical retry or convergence
ladder. All owned processes are gone and all 27 bound archived inputs remain
unchanged. The sampled RSS guard was 12 GB, with a separate 14 GB per-process
address-space backstop.

Current evidence is in
`output/diagnostics/bounded-double-box-korobov2-3/assessment.json`, with the
ordinary saved result/checkpoint, strict-reader output, complete stage timings,
transport repair and unconditional postflight retained alongside it. The
historical Korobov3 observations and failure prefixes have not been overwritten.

Independent retained-data review accepts the current native Korobov2 result in
`bounded-double-box-korobov2-3/independent-review.json`.

A subsequent Pathfinder counterpart uses the same Korobov2 transform, requested
8,192 points, sixteen shifts, seed20561302 and eight workers. Its ordinary prime
catalogue selects 8,311 points with vector `[1,3068,1811,1128,1964,516]`:
12,765,696 evaluations across all 96 groups. It completes in **42.756 seconds**,
with finite coefficient −14.8603884244123 ±0.07521661845352776, or **5.062‰**.
All five available reference checks pass, but this allocation does **not** reach
1‰. Its faster elapsed time is not a time-to-target measurement, and no target
time is extrapolated.

The reference evaluator-plus-Python mean is 19.775 µs and its largest sector
mean is 130.661 µs, still using the narrower bucket boundary above. It retains
90.19% ordinary evaluations, 8.82% at 32 decimal digits, 0.78% at 100 digits and
0.21% at 1,000 digits. No precision policy changes. The existing bundle's fused
raw-coordinate kernels were prepared with Korobov3; the normal correlated route
applies the requested Korobov2 map before dispatching its transformed-coordinate
component evaluator, so those stored fused kernels are bypassed. Source and
runtime request/group checks retain this qualification; no regeneration or
reference-source patch occurs. Full returned real/imaginary vectors and all
active shift counts are retained, without inventing unavailable reference
covariance. Evidence is in
`output/diagnostics/bounded-double-box-korobov2-reference-1/`; all processes are
reaped and the original source, read caches and prepared bundle are unchanged.

The one final larger Korobov2 reference allocation requests16,384 points and
selects prime17,807, keeping every other setting fixed. It completes all
27,351,552 evaluations in **91.584 seconds**, with finite coefficient
−14.848140000889918 ±0.0053784378654040295, or **0.362‰** reported relative error.
All five available reference checks pass (largest absolute pull1.261642). Its
mean evaluator-plus-Python bucket is19.900µs and the largest sector mean is
135.109µs; these narrower intervals still cannot replace the API-call
measurements in [the sample-latency report](current-sample-latency-results.md).

| Best completed Korobov2 target row | FastSecDec | Pathfinder |
|---|---:|---:|
| Actual points / shifts | 8,192 / 16 | 17,807 / 16 |
| Complete integration/result process | 172.715 s | 91.584 s |
| Finite reported relative error | 0.714‰ | 0.362‰ |
| Accepted evaluations | 13,369,344 | 27,351,552 |

This comparison leaves a real convergence-time gap: Pathfinder reaches a
smaller reported uncertainty sooner in its completed row. The earlier slower
Korobov3 reference is not selected as the final comparator. Different published
rules and uncertainty propagation remain explicit, and neither row is a median
or an uncertainty-calibration result. The final reference bracket, source and
complete physical-vector admission are retained in
`output/diagnostics/bounded-double-box-korobov2-reference-2/`; all processes are
reaped and the frozen bundle/read caches remain unchanged. The reference ladder
stops after this bounded bracket, with no extrapolation to further allocations.
Independent review in the final counterpart directory (`independent-review.json`)
accepts all 96 groups, complete vectors and available reference checks, verifies
the 375 prepared-program bindings, and confirms that the native target-time gap
remains open.
