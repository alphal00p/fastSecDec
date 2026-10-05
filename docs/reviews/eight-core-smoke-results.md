# Eight-physical-core finite-target smoke results

The approved runner completed preparation and all four labelled smoke processes
on 2026-10-05. Both FastSecDec cases passed; both frozen Pathfinder integrations
aborted before producing a physical result because of the installed Symbolica
instance limit. The paired smoke gate is **not accepted**, and no seven-seed
paired campaign has run. Smoke seed `20261210` is excluded from timing summaries.

The full retained directories are
`output/benchmarks/eight-core-preparation-20261005` and
`output/benchmarks/eight-core-smoke-20261005`. They include exact argv, native
build evidence, source/artifact hashes checked before and after, timer records,
stdout/stderr, all successful native checkpoints/results, and failed process
outcomes. The pure-data reader's direct Rust build and four previous-result
extraction controls are recorded in
`output/diagnostics/eight-core-reader-validation`; those cached one-worker
records are transport controls, not new eight-core measurements.

## Observed native results

Both cases requested all orders `[-2,-1,0]`; the target is the largest signed
order, **eps^0**. Each native row used the first prescribed allocation,
`N=1024`, `R=16`, Kuo33002, Korobov3, package size 1024, eight worker threads,
and shared shift indices across all sectors. CPU affinity 0–7 contains eight
distinct physical cores; the retained topology check verifies they are allowed
and records socket/core/SMT siblings. No statistical stopping or sampling rule
was changed to obtain the result.

| Case | Finite coefficient | Standard error | Relative SE | Reported integration elapsed | Separate artifact load/JIT | Complete process |
|---|---:|---:|---:|---:|---:|---:|
| Triangle | 0.6558780721990614 | 2.8856239473e-9 | 4.3996347334e-9 | 0.041874569 s | 0.005482508 s | 0.053248097 s |
| Box | -12.49312230872735 | 8.1169400213e-6 | 6.4971268357e-7 | 0.049499684 s | 0.006639842 s | 0.061733734 s |

The target was met at the **first tested complete allocation**. This is neither
a minimal sample count nor an interpolated crossing time. Triangle retained
32,768 evaluations across two sectors; box retained 49,152 across three. Every
sector has all 16 complete shifts and 16,384 accepted points. Complete vectors,
cross-coefficient covariance and native shared-shift covariance are retained.
All coefficient residual checks against the frozen historical target passed;
that target remains `Unverified`, with its errors and comparison eligibility
unchanged. These checks supplement, rather than replace, prior independent
native master/analytic validation.

Native precision diagnostics were active: triangle recorded 3,207 rescues,
4,464 conditioning checks and 9 additional weighted replays; box recorded 4,516,
7,671 and 10 respectively. Both reached 320 bits and recorded zero evaluation
failures. The reported integration interval starts after artifact loading and
pool/problem setup; it includes worker scheduling, evaluation, accepted-state
processing, observations and checkpoint I/O. Complete process wall time is the
common end-to-end boundary. These are smoke observations, not repeated timing
statistics or individual-sample latency measurements.

## Preparation and unresolved reference integration

All four fresh generation processes exited zero with unchanged inputs.
Triangle native generation reported 0.026132377 s (process 0.034402084 s), and box
reported 0.014452948 s (process 0.019429895 s). Pathfinder triangle reported
0.335114852 s (process 0.942928249 s); box reported 0.328578610 s
(process 0.869644876 s).
The reference reported generation total sums overlapping named records
(including symmetry nested within sector decomposition), as documented in the
earlier paired audit. It is retained with that qualification rather than treated
as an independent elapsed timer. The generation process wall boundary is clean.
Native generation builds O2 and persists canonical expressions, so integration
loads and compiles O2 again. Pathfinder generation persists hot/precision
evaluator payloads whose lazy loading can occur inside its integration interval.
These differing readiness/load boundaries prevent treating their reported
integration fields as identical pure evaluator-loop timings.

Both reference numerical processes used the prescribed eight workers and
CPU 0–7. They terminated with signal 6, after 1.119443074 s for triangle and
1.114424386 s for box, without a physical result. Their stdout repeatedly states
that another unlicensed Symbolica instance is already running. The frozen
source confirms QMC uses `ProcessPoolExecutor` with `fork` and
`_init_worker_from_parent` (`src/integrator.py:3041` and 317), creating a sector
processor in each worker. Inherited evaluator payloads can load lazily in those
processes (`src/integrand.py:322`). The process evidence establishes a runtime
instance-limit failure, not an integrand, convergence or numerical accuracy
failure. No licensing settings were changed, and no reduced-worker measurement
was substituted for the eight-core request. The owned process groups were
reaped; failed rows remain part of campaign evidence.

The frozen reference JSON also contains Python NaN tokens in some diagnostic
error fields. A separately reviewed shell/jq transport tags numeric
NaN/±Infinity, preserving quoted strings, null and finite values, original bytes
and SHA-256. This conversion occurs after timing. The Rust reader still rejects
nonfinite/tagged physical means or standard errors. The capability controls,
jq executable/version/hash and transport source are retained; no unknown error
is replaced with zero or exactness.
