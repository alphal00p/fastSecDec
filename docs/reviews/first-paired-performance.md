# First triangle/box paired performance execution

This records section B of the
[prespecified campaign](convergence-campaign-design.md). The four smoke rows
passed before the separate paired block started. The ignored
`output/probes/first_paired.sh` delegates every
calculation to the existing production CLI or the frozen Pathfinder CLI. It
implements no integrator, coefficient algebra or estimator.

The native executable is the frozen `fastsecdec-eb7e5d8` build from the status
campaign, SHA-256
`b9bcec8797df82f1a124dc607c2f6a33eb064205257bd80b36ebb07321860851`.
Its complete build/dependency evidence is retained under
`output/diagnostics/status-cadence/`. The reference revision remains
`582d8c7f6dde9bf750750d4c2a2d85a94ce940cd`, with the already verified complex O2
path. Backend versions and precision policies differ as recorded in the campaign
protocol; these are explicitly versioned programs, not identical generated code.

The runner accepts a new absolute output directory and the independently tested
process timer. Each timed process uses one identical allowed Linux CPU, a
180-second deadline, a five-second interrupt grace, exact argument capture,
input/source/build hashes and whitelisted numerical environment settings. It
retains every outcome, checks the expected row count, refuses an existing
campaign directory and does not overwrite failed attempts.

The first stage has four fresh-process smoke rows: native/reference triangle,
then reference/native box, using 1024 points, eight shifts and seed 20261200.
The separate paired stage has 28 rows: seven alternating native/reference pairs
for each input, 8192 points, 16 shifts and seeds 20261201 through 20261207. It
requires a coordinator-reviewed smoke record bound to the exact native binary,
reference revision and runner hashes. Equal integer seeds do not establish
shared shifts between Havana and QMCPy. Statistical checks use the independently
established analytic/master/frozen-target values and preserve each uncertainty;
timing pairs control execution order, not cross-program sample covariance.

The reference command explicitly selects complex SymJIT O2, full support,
democratic correlated sectors, QMCPy linear lattice, Korobov3, one worker,
1024-point batches and one complete round. Its point cap is disabled. Native
settings select QMC/Kuo33002, one worker and zero stopping tolerances, retaining
the entire requested allocation. Before the smoke gate is accepted, inspect
both programs' actual settings, retained orders, generating vector, shifts,
periodization, package sizes, completed work and numerical failure counts.
Native Korobov3 and package size 1024 currently come from checked defaults, not
explicit flags. Missing imaginary or lower-order native rows remain missing.

The independent reference/performance agent reviewed the shell orchestration
against the fixed protocol and found no blocking mismatch. The reference's
supplied formula caches remain
available, so this is not cold generation. Native artifact/checkpoint/result
persistence and reference formula/result persistence differ; report those costs
and do not call this first mode a matched durability comparison. Timing ratios
alone cannot close the strict accuracy/precision or per-case performance gates.

## Executed smoke admission

All four fresh processes succeeded. Triangle used exactly 16,384 kernel
evaluations in each program, and box 24,576. Both programs retained all three
orders `[-2,-1,0]`. Native saved results include full covariance and zero
evaluation failures. Reference imaginary coefficients and their errors are
zero; native real-only manifests have no imaginary rows. The reference format
does not supply cross-order covariance, which remains unknown.

Actual native settings confirm Korobov3, packages of 1024, eight complete
shifts, and Kuo33002. A metadata-only query in the existing reference environment
confirms QMCPy's vector begins `[1,182667,213731]`; modulo 1024 this is
`[1,395,739]`, matching the native checkpoints in the corresponding dimensions.
This checks the lattice, not identity of random shifts between the programs.

Every retained real coefficient is below the prespecified five-combined-SE
investigation threshold against the frozen analytic/master-backed targets,
including their nonzero reported errors. The largest normalized residual is
3.174 for the native box leading coefficient. Eight shifts do not establish
uncertainty coverage. All means, errors, native covariance, effective settings,
counts, and precision diagnostics remain in the accepted smoke record.

The exact orchestration SHA-256 is
`c6a252485ae84476a7c35ea20360a36f7e771e332f7e9629c5e0d83ff695fb60`;
the independently tested process timer is
`5ff6d1ce5a8363c872a5a9f33139a1d7ece6f72770683aa9dc83300ca1af9ba9`.
The accepted record binds those identities, the native executable and reference
revision. Evidence is retained in
`output/benchmarks/first-paired-smoke-20261004/`, including
`smoke-acceptance.json` and the reference lattice query. The one-off smoke times
are operational checks, not performance acceptance.

## Executed seven-pair block

All 28 processes succeeded, with no timeout, incomplete allocation or native
evaluation failure. The complete real vector remains `[-2,-1,0]` in every row.
Each triangle run accepted 262,144 physical kernel evaluations and each box run
393,216 in both programs. Actual reference diagnostics confirm 8192 lattice
points, sixteen shifts, full support, and one complete iteration; native saved
results confirm all allocations and complete production. All diagnostic
comparisons against the frozen targets remain below the prespecified threshold;
the largest normalized residual is 1.960. Native saved validation stays
`unverified`: this report does not silently attach a new certified reference.

| Case | Seed | Native process seconds | Reference process seconds | Native/reference |
| --- | ---: | ---: | ---: | ---: |
| triangle | 20261201 | 1.030043 | 3.013233 | 0.3418 |
| triangle | 20261202 | 1.012847 | 2.885600 | 0.3510 |
| triangle | 20261203 | 1.012243 | 2.884075 | 0.3510 |
| triangle | 20261204 | 1.012410 | 2.876754 | 0.3519 |
| triangle | 20261205 | 1.010422 | 2.922206 | 0.3458 |
| triangle | 20261206 | 1.011119 | 2.869545 | 0.3524 |
| triangle | 20261207 | 1.010353 | 2.869012 | 0.3522 |
| box | 20261201 | 1.233931 | 4.257669 | 0.2898 |
| box | 20261202 | 1.230588 | 4.243836 | 0.2900 |
| box | 20261203 | 1.231442 | 4.404233 | 0.2796 |
| box | 20261204 | 1.230714 | 4.471541 | 0.2752 |
| box | 20261205 | 1.231616 | 4.445630 | 0.2770 |
| box | 20261206 | 1.233924 | 4.508122 | 0.2737 |
| box | 20261207 | 1.238865 | 4.332146 | 0.2860 |

Triangle median process times are **1.012243 s native / 2.884075 s reference**;
the median paired ratio is 0.3510. Box medians are **1.231616 s / 4.404233 s**,
with median paired ratio 0.2796. Median CPU times are 1.005221 / 2.862605 s for
triangle and 1.223924 / 4.374802 s for box. All rows ran serially on the same
allowed CPU without concurrent project compilation or scientific workloads.

These are useful end-to-end observations of explicitly versioned commands. They
do not isolate evaluator throughput, cold generation, or equal persistence
costs, and they retain different precision policies. Native full-vector rescues
range from 25,587 to 25,633 for triangle and 35,990 to 36,036 for box; the retained
records reach 320–384 bits for triangle and 320 bits for box. Reference
precision-tier counts remain in every result;
its 32/100/1000-decimal policy is not equated to native conditioning/rescue.
The next prepared-artifact mode must align durability and verify effective
settings before the strict per-case performance gate can close.

Evidence is under `output/benchmarks/first-paired-20261004/`: all invocations,
process outcomes, stdout/stderr, complete reference results, native artifacts,
checkpoints and saved results, plus `observations.json`. The latter retains every
vector and native covariance, all target uncertainties and paired observations.
No row was dropped or replaced. The independently tested timer measures process
completion with blocking native waits; its peak RSS is the child usage statistic,
not a simultaneous process-tree memory sum.

The [independent evidence review](first-paired-performance-independent.md)
checks all 28 outcomes and serial intervals, complete allocations, vectors,
native covariance and settings, target residuals and timing summaries. It also
verifies the status report's worker-cost ratios and their timer boundaries.
Its precision-policy narrative finding is corrected above; no performance
acceptance or eight-core convergence claim is inferred from these records.
