# K1 release pilot and paired variance smoke

2026-10-10. This is a bounded native runtime and matched-work result for the finite complex coefficient of `2L4P.b.K1`. All four finite pilots and all eight complete sampling allocations passed their operational gates. **No prescription reached the requested 0.1% relative joint uncertainty.** The best tested choice, polynomial `L=0.1`, has 14.32% and 13.89% relative joint uncertainty in the two runs. These results establish neither convergence nor full physical accuracy.

## Scope, normalization and native ownership

The unchanged HEPKit/LTD source has 186 six-dimensional stochastic sectors, 186 exact records and global layout orders `[0,0]`, components `[Real,Imag]`. Source identity is `d27e82184cdd7afc90bc12f27b8e6b6bfcd3786007326bb7f8e9c3a5c8b394d5`. Physical parameters are already substituted. The input includes the paper conversion `(4*pi)^(2*eps) * -1/(256*pi^4)`; no further phase or scale is applied.

The rounded ancillary reference is `(-1.0840618909337886 + 2.8682065140371712 i) * 10^-6`. Its uncertainty was not supplied. See [the native fixture provenance](contour-ltd-fixtures.md) and [the independent admission audit](contour-k1-runtime-admission-audit.md). Historical unconverged fixed estimates are not a new reference or timing baseline.

| Arm | Prescription |
|---|---|
| F001 | Fixed lambda=0.001 |
| F005 | Fixed lambda=0.005 |
| P01 | Polynomial dynamic S=0.8, L=0.01, R=1 |
| P1 | Polynomial dynamic S=0.8, L=0.1, R=1 |

No sign-aware full-integral run is included here. All prescriptions and allocations were frozen before sampling. Neither failures nor large errors caused a cap, seed or allocation change.

The caller uses native `ProgramArchiveReader`, parameter binding, `WeightedEvaluationContext`, `QmcSession` and `QmcWorker`. It loads one fixed and one polynomial sector owner at a time, rotates arm order by sector, actually rebinds and repeats the pilot for each arm, and creates an independent evaluation context for each seed. Two aggregate exact owners are loaded in total, one fixed and one polynomial. The appropriate owner is independently rebound and piloted for each arm, then its exact vector enters each seed session once. No readiness is borrowed from a discarded owner. No alternate sampler, CAS or covariance accumulator is used.

## Prior admission and artifact trust

The initial optional-validation map attempt remains incomplete evidence: 10 sectors in 120.434 s. Native load attribution separated optional metadata revalidation from saved-program restoration; no Horner/CPE reconstruction was found on load. A separately authorized trusted-artifact pass checked all 186 native source/geometry/F/U/face associations in 261.226 s, peak 401,584,128 bytes. It used the normal `validate=false` loader after integrity checks, retaining mandatory structural admission and explicit map comparisons. It did not repeat every optional semantic reconstruction.

Before trusting the legacy fixed data, the native catalogue matched its pinned historical manifest and all 372 original record BLAKE3 receipts passed a streaming check. The polynomial whole-file SHA was tied to the closed generation handoff. Both full archive hashes were checked again before pilots and sampling.

| Artifact | Manifest SHA256 | Data SHA256 |
|---|---|---|
| fixed | `58b61c00f4746aca2ab20cb6d1101fe09a577281535c41a29d9686673e66dcde` | `a1a2ab0c3ec274cf3b0382b9bbfbe10e20d58a6f0327d785399f09b90fbf7ae0` |
| polynomial | `dd35724144749a5f27a03a902bd5eb49abf0d0f08f411cc41b3fb0a1637d0f87` | `72c4ae498e5c52443b2bb7cc1eb06152a11de94e05f74f6f3c382ace3fc677f9` |

The complete pilot ran each of the 372 actual record owners and the fresh aggregate exact owner for each arm. Each source chart used 16 points: centre, all twelve individual coordinate faces with other axes at 0.5, all-0.01, all-0.99 and alternating 0.25/0.75. Each arm covered 186 charts and accepted 2,976 points; maximum certificate precision was 96 bits. Fixed arms reported 14,880 homotopy arguments, polynomial arms 2,976 bounded-homotopy requests: these are different work units. Aggregate exact owners had no surviving requests, so their zero pilot count is correct. Finite pilot coverage is not a whole-cube certificate.

Full pilot monitor: 718.760973 s, peak 1,153,024,000 bytes, under 900 s/3 GiB, all four arms passed. Concurrent K1* generation/private release compilation was recorded; this is admission timing, not an idle-host comparison. The historical admission audit is left unchanged; later results live here.

## Release provenance and cost preflight

The private native core is exact commit `33030058d2cd0587dd718ef3e7dc9c56ba6952f8`, verified from 1,405 archived source files and its lock/owner graph, features `[native]`, opt-level 3. The standalone driver links only that private release graph with `-C opt-level=3 -C lto=thin`; no installed/user binary was replaced. The data encode saved native evaluators; normal restoration still prepares numeric backends rather than claiming cached machine-code loading.

- Final driver source SHA256: `7f8854521e5cf6d13e26e4e4249752127a6a1f058d765084a441b6d0847aca52`.
- Final driver executable SHA256: `b37038f5cc6ed1cb0f601cc8c431815dd3597fda3a81362384dba1fb43c941b6`.
- Private core rlib SHA256: `ae51321d6d1f6769c5a97a3316c7514c556828217d2f99b4892b19a62460efa4`.
- Release verification record SHA256: `763b5c769130ea3c3d1681ab991fb8885dd3cf59ebfeede3e1e7551304011253`.
- Complete eight-run JSON SHA256: `bfc5163e85514b0c258bcd8cc13fdd217b9b66d5468ff1ae2bce0c6de845c4b0`.

The first linked paired source remains separately preserved (`49710f47c5b731bff5ab877deab929caa9fc80897a01dfcf93ed0ed4958649c3`). The final delta changes only failure reporting: native `BatchEvaluationError.completed.len()` supplies the failing row; the error retains arm, seed, sector, native work identity, shift/lattice index, coordinate/weight values and exact f64 bits. Independent review verified this against the native row and shift-major contracts. Successful arithmetic, sample allocation and submission are unchanged.

A separately reviewed, release-linked source-0 probe sampled all four arms and two seeds, 8,192 points each. It passed all coordinate/weight comparisons and showed zero optional production checks after Pilot admission. Its 120 s/3 GiB monitor closed in 5.973499 s, peak 79,925,248 bytes: 4.485768 s archive hashing, 0.593758 s sector loading, 0.369201 s binding/pilot, and 0.283476 s sampling including 0.010113 s coordinate hashing. A source-0-only extrapolation suggested roughly 232–277 s for the full smoke; it was feasibility evidence, not a bound or performance promise. An unrelated host compilation was observed and left untouched.

## Frozen sampling and complete statistics

Each arm/seed used 1,024 Kuo lattice points per shift, eight shifts, Korobov3 periodization, package size 1,024 and batch size 256. Seeds were 34723 and 92711. Each complete run has 1,523,712 stochastic points across all 186 sectors; all eight together have 12,189,696 points. No extra seeds, points or continuation were added.

The native democratic session estimates covariance from complete common-shift sums across sectors. It retains cross-sector and real/imaginary covariance; it does not sum independent-sector variances. Exact offsets enter the mean once and never covariance. Native contributions and complete shift vectors are saved. Different prescriptions remain separate estimates.

The driver hashes actual transformed coordinates, weights, sector IDs and work ranges, then compares every arm for each same seed and sector. All comparisons passed. Every production context reported zero optional contour checks, as required by Pilot semantics. Essential numerical failure handling remained active.

The 900 s/4 GiB monitor closed successfully in 218.333458 s, peak 839,757,824 bytes. Its process group was verified empty. All eight native allocations completed; **zero arms failed and zero sampled points were dropped**. Other authorized source/template work and unrelated host compilation were permitted concurrently, so this is not a controlled idle-host or eight-worker speed measurement.

Means and component sampling standard errors below are in units of `10^-6`. Relative joint SE is `sqrt(trace(C))/abs(reference)` and is shown as a dimensionless ratio; the requested target is 0.001. This reference-normalized diagnostic differs from the native stopping rule, which uses the norm of the current estimated complex mean.

| Arm | Seed | Re mean ± SE | Im mean ± SE | Relative joint SE |
|---|---:|---:|---:|---:|---:|
| F001 | 34723 | 1277.1455 ± 3184.593 | 4807.1865 ± 2577.31 | 1336.1165 |
| F001 | 92711 | -4623.6965 ± 4943.511 | 6706.0175 ± 4567.012 | 2194.9467 |
| F005 | 34723 | 115.7729 ± 116.91792 | -23.573183 ± 64.401828 | 43.532803 |
| F005 | 92711 | 95.552391 ± 117.3639 | 7.9067863 ± 83.707048 | 47.014235 |
| P01 | 34723 | 27.960976 ± 25.073257 | -3.97015 ± 19.682422 | 10.395741 |
| P01 | 92711 | 38.675841 ± 27.701621 | 14.97213 ± 33.705677 | 14.22871 |
| P1 | 34723 | -0.85913754 ± 0.21639196 | 2.205758 ± 0.38216742 | 0.1432304 |
| P1 | 92711 | -0.87433895 ± 0.27427968 | 3.0120229 ± 0.32577102 | 0.13888668 |

All componentwise `abs(mean-reference) <= 5*samplingSE + 1e-14` diagnostics pass. The declared absolute floor is not a reference uncertainty. Passing with large errors is not accuracy acceptance; notably the fixed and L=0.01 estimates remain severely underresolved.

Complete native covariance of the mean follows in unscaled integral units. The omitted lower off-diagonal equals `C_RI` in every raw report.

| Arm | Seed | C_RR | C_RI | C_II |
|---|---:|---:|---:|---:|
| F001 | 34723 | 1.0141632504e-05 | 1.0644807108e-06 | 6.6425267940e-06 |
| F001 | 92711 | 2.4438300950e-05 | -5.2771132853e-06 | 2.0857599016e-05 |
| F005 | 34723 | 1.3669799497e-08 | -2.8041395252e-10 | 4.1475954431e-09 |
| F005 | 92711 | 1.3774285582e-08 | -2.0188398638e-09 | 7.0068698701e-09 |
| P01 | 34723 | 6.2866822219e-10 | -1.2459603195e-10 | 3.8739772111e-10 |
| P01 | 92711 | 7.6737980350e-10 | -3.5105236402e-10 | 1.1360726387e-09 |
| P1 | 34723 | 4.6825479757e-14 | -5.0406163495e-14 | 1.4605193855e-13 |
| P1 | 92711 | 7.5229344259e-14 | -6.6583687259e-14 | 1.0612675864e-13 |

## Observed variance and cost limitations

The scalar variance diagnostic is the trace of this full covariance. Against fixed 0.005, polynomial L=0.01 has observed variance reductions 17.54 and 10.92 in the two matched pairs; L=0.1 has reductions 92,377 and 114,588. These are noisy eight-shift estimates from only two seeds, not a general variance guarantee. Full values and comparisons against both fixed strengths are retained in `audit-result.json`.

| Arm | Seed | Sample seconds incl. hashes | Coordinate-hash seconds | Sample seconds excl. hashes |
|---|---:|---:|---:|---:|
| F001 | 34723 | 2.042292 | 0.235394 | 1.806898 |
| F001 | 92711 | 2.064030 | 0.234561 | 1.829469 |
| F005 | 34723 | 2.049081 | 0.234998 | 1.814082 |
| F005 | 92711 | 2.077431 | 0.234745 | 1.842686 |
| P01 | 34723 | 9.048489 | 0.235385 | 8.813104 |
| P01 | 92711 | 9.243572 | 0.235060 | 9.008512 |
| P1 | 34723 | 9.455877 | 0.235373 | 9.220504 |
| P1 | 92711 | 9.556070 | 0.235220 | 9.320851 |

Across the one-load-per-sector loop, fixed restoration totals 6.615596 s and polynomial restoration 98.057043 s. Sampling timers exclude restore, bind, pilot and context creation. Exact initialization, archive hashing and output are included in the outer monitor. No per-arm total-time speedup is inferred by assigning shared load costs arbitrarily.

Polynomial samples were about 4.86–5.08 times slower than fixed 0.005 after subtracting coordinate-hash clocks. The observed variance reduction is larger in these two pairs, but neither a universal time-to-precision improvement nor parallel speedup is established. No covariance estimates are pooled across prescriptions.

## Remaining scientific gates and reproducibility

Required follow-up includes independent-seed/refinement convergence evidence for the promising L=0.1 setting, reaching the 0.1% target, reference uncertainty/independent reference budgeting, and whole-integral sign-aware and massive K1* comparisons. The current finite pilot and broad five-SE diagnostics cannot replace those gates. No default-L=1 or other problem/kinematics conclusion follows from these explicitly chosen dimensionless caps.

All raw artifacts and programs remain ignored. The accepted map/pilot results are under `target/contour-ltd-k1-runtime/maps-trusted/` and `full-pilot/`; release source-0 evidence is `release-source-cost/`; the final reviewed driver, source/build identities, exact commands, authorized limits, point hashes, all eight reports, native covariance/shift vectors and independent process-group monitor are in `paired-sector-final/`. Legacy fixed input is `target/contour-ltd-template-k1/integral.fsd.json`; polynomial input is `target/generation-agent-ltd-compact-k1/polynomial.fsd.json`. The runner verifies full-admission and map provenance plus both complete data hashes before invocation, refuses stale attempt evidence, and uses the separately verified private release graph under `target/contour-private-release-3303005/`.

This slice changes no production source. Independent ecosystem reuse and failure-boundary review was performed by the foundation agent before launch. Its final result audit also passed: all eight raw reports match the summary, native complete-shift vectors reproduce every mean and covariance entry, and all coordinate/weight comparisons and zero-production-check claims match the raw evidence (`paired-sector-final/foundation-result-audit.json`). No dependency bug or missing native numerical operation was found by this runtime experiment.
