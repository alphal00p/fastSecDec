# Bounded double-box lattice follow-up

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
