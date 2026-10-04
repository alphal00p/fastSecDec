# Independent paired-measurement and status audit

This review reads the retained process outcomes and numerical reports. It does
not rerun Symbolica, change the estimator, or turn a timing observation into a
strict parity/convergence certificate.

All 28 prescribed processes under
`output/benchmarks/first-paired-20261004/` succeeded without timeout. Their
recorded intervals are disjoint and their commands select the same allowed CPU.
Every process reports unchanged executable, build evidence and hashed inputs.
There are exactly seven native/reference pairs for each case, seeds 20261201
through 20261207. The native binary and timer identities agree with the frozen
status-cadence build and independently tested process timer. The timer hashes
the outer `taskset` executable; the campaign separately retains the actual
FastSecDec/Python program identity, as required by its documented boundary.

Every native saved result has FullIntegral scope, production-complete totals,
all-real orders `[-2,-1,0]`, sixteen used/complete replicas in each sector and
131,072 used points per sector. Its effective design records Kuo33002,
Korobov3, packages of 1024, 8192 lattice points and the prescribed seed.
Triangle has two kernels and 262,144 evaluations; box has three and 393,216.
Every reference report independently records the same actual point count,
sixteen shifts, a complete iteration, full support and its complex O2 mode.
Its imaginary means/errors are zero. No absent imaginary row is filled into a
native real-only manifest, and no cross-order reference covariance is invented.

The observations file reproduces native total means, covariance, errors,
diagnostics and effective designs exactly. Finite means/errors, nonnegative
diagonal variances and `error² = covariance diagonal` are consistent. Retained
target errors remain nonzero where recorded. Rechecking the diagnostic residuals
against those same targets confirms the largest normalized residual is
1.959355997025996. This is a consistency check of the retained comparisons,
not a new independent target or a coverage proof from seven seeds.

Recomputed process medians agree with the author report:

| Case | Native wall seconds | Reference wall seconds | Median paired ratio | Native CPU seconds | Reference CPU seconds |
| --- | ---: | ---: | ---: | ---: | ---: |
| Triangle | 1.012243416 | 2.884075371 | 0.351000297746 | 1.005221 | 2.862605 |
| Box | 1.231616207 | 4.404233047 | 0.279604115145 | 1.223924 | 4.374802 |

One documentation discrepancy was found and sent to the author for correction:
the native precision maximum was not 256 bits. The seven triangle maxima are
`[320,320,320,320,320,384,320]`; all seven box maxima are 320 bits. Rescue counts
range from 25,587 to 25,633 and 35,990 to 36,036 respectively. All failure counters
are zero. The raw reports and observations already contained the correct
precision values; the discrepancy was limited to narrative text.

Compact independent evidence is retained in
`output/benchmarks/first-paired-20261004/independent-review.json`, bound to the
observations SHA-256. Different precision policies, available reference formula
caches, and different persistence work remain the reasons this is not the final
matched per-case performance acceptance.

## User status snapshot: timer and worker-cost checks

The status table's small generation numbers agree with all seven artifact
`generation_timings.total_seconds` values: medians are 9.728209 ms for triangle
and 15.371655 ms for box. These are internal generation totals including kernel
compilation, not whole command times.

The following ratios were independently recomputed from every named saved
result's native contribution records. Overall means use total accepted worker
seconds divided by total used points; the last column takes the largest
individual sector ratio. The small-case rows are medians of the seven run-level
ratios, not pooled observations with an invented uncertainty.

| Case | Mean worker microseconds per sample | Largest sector-average microseconds per sample |
| --- | ---: | ---: |
| Triangle | 3.760523857 | 7.406718842 |
| Box | 3.041249062 | 4.728726181 |
| Original off-shell triple box | 13.269168638 | 195.884888428, sector 474 |
| Original off-shell triple box, rank two | 3.753344790 | 78.283831543, sector 495 |
| Issue 1 | 0.294813556 | 0.364643311, sector 0 |
| Hard four-loop orthant | 25.944436070 | 71.703304932, sector 686 |

`integration/worker.rs:127` starts the QMC worker timer immediately before the
point loop. It includes native lattice point generation, periodization, the
complete-vector evaluation callback and its diagnostic accounting, rescue, and
native partial accumulation. It excludes partial construction/buffer resizing
before the timer, evaluator/context construction, coordinator submission,
status serialization and checkpoint I/O. Thus these ratios are accepted worker
loop costs, not scalar-JIT-only latency or maximum individual-sample latency.
The status snapshot correctly labels the slowest column as a sector average.

The large saved-result elapsed values also agree: 108.146392472 s scalar
off-shell, 35.235925792 s rank two, 1.043265195 s issue 1, and 95.912381737 s hard
orthant. Artifact load durations are separate fields. Those numerical/reporting
phases include coordinator work; summing two workers' times is not their wall
time. The large whole-generation process times and the small internal generation
medians remain deliberately different timer boundaries. No eight-core or
time-to-one-per-mille result is established by these fixed-work observations.

## Pathfinder side-by-side timer boundaries

The user's follow-up requests side-by-side generation and sector cost, so the
same fourteen reference results were traced back to their timer owners. Their
reported medians are:

| Recorded quantity | Triangle | Box |
| --- | ---: | ---: |
| FastSecDec generation elapsed, ms | 9.728209 | 15.371655 |
| Pathfinder generation record sum, ms | 319.812990 | 332.533938 |
| FastSecDec mean accepted worker cost, us/sample | 3.760524 | 3.041249 |
| FastSecDec largest sector-average worker cost, us/sample | 7.406719 | 4.728726 |
| Pathfinder evaluator-only mean, us/sample | 3.239014 | 3.835892 |
| Pathfinder largest evaluator-only sector average, us/sample | 5.217707 | 4.934313 |
| Pathfinder evaluator-plus-Python mean, us/sample | 5.408920 | 6.915875 |
| Pathfinder largest evaluator-plus-Python sector average, us/sample | 7.985160 | 8.529100 |

These rows deliberately name different recorded boundaries. They must not be
turned into an isolated generation or evaluation speedup ratio:

- Native generation is one elapsed interval from before input loading through
  compilation and artifact construction, sampled before artifact file writing
  (`fastsecdec-cli/src/generate.rs`). Pathfinder's
  `summary.generation_timings.total` is a **sum of named records**, not one
  enclosing stopwatch (`src/generation_timing.py:80`). In particular its sector
  decomposition timer encloses the separately recorded symmetry-squashing
  timer (`pysecdec_bridge.py:837,971`), so that component is counted twice in the
  sum. Reference formula-cache hits and the existing different startup/build
  boundaries are also retained. The recorded totals are useful status data but
  are not a matched cold-generation measurement.
- Reference `avg_eval_us_per_sample` divides worker evaluator-call time only
  (`integrator.py:1541`). It excludes Python control/array work. Adding the
  same sector's `python_seconds` includes coordinate-array setup, precision
  dispatch/rescue orchestration, component masking, Jacobian-weight
  multiplication and local per-shift reduction under
  `_evaluate_qmc_batch`'s hot timer (`integrator.py:564–845`). It still excludes
  separately charged lattice-generation/periodization time and coordinator
  aggregation.
- In these one-worker records, every sector's `havana_seconds` is zero; shared
  lattice generation and Korobov transformation are charged to global
  integrator time while batches are prepared. The global integrator bucket alone
  has medians 2.014926 us/evaluation for triangle and 1.493423 for box. It cannot
  be assigned back to individual sectors or mistaken for total integration
  wall time. Reference global physical-prefactor convolution is applied to
  statistics outside this sector evaluation timer, whereas native kernels
  already represent the complete physical coefficients.
- Native accepted worker cost includes its lattice/transform/sample accumulation
  and physical complete-vector kernel work. Reference precision policies and
  evaluator outputs also differ. Neither program recorded individual-sample
  latency maxima; the largest sector-average is explicitly a different
  statistic. A matched cost profile must add aligned native instrumentation,
  not relabel these existing buckets.

Every derived run value and the medians are preserved in
`output/benchmarks/first-paired-20261004/independent-timer-boundaries.json`.
The same-CPU complete-process measurements remain the cleanest recorded
end-to-end comparison, with the previously disclosed cache, durability and
precision differences. No recorded eight-core time-to-one-per-mille observation
exists for either program in this campaign.
