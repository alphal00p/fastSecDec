# Numerical-dual double-box validation

This record compares the two generation modes on the current D05 s-channel
`examples/gghh_double_box` input. It is a measurement of one diagram on one host,
not a three-loop or cluster performance claim. The original symbolic mode remains
the default.

These initial measurements precede generation-scoped subtraction-formula
preparation and concurrent per-key evaluator caching. The subsequent
[formula-cache benchmark](numerical-dual-formula-benchmark.md) records that
follow-up with separate discovery, formula and sector-assembly timings. Keep the
two protocols distinct when interpreting the figures below.

## Protocol

The host is an Apple M3 Pro with 12 CPU cores and 36 GiB RAM. The optimized Rust
build uses rustc 1.99.0 and Apple's Clang linker. All four generation runs use
eight caller-owned workers, minimal numerator contraction, symbolic model and
kinematic inputs, epsilon order zero, SymJIT O2, ten Horner iterations and the
default 1000 common-pair rounds. Only `generation.mode` and
`generation.subtraction` differ between run cards. No other builds or benchmark
jobs run concurrently with a timed generation or evaluator measurement.

The numerical source owner is Symbolica community revision
`58652fabc2f736302a570deaaf8d517679f7fe6e` plus the independently reviewed local
composition commits `9b82a0bacaed334616ed9066db57b78206418ec4` and
`1deccb8538ccb91dc2c1e58fc0a2e900d2276bf4`. Numerica and Graphica remain registry
3.0.1. These measurements used an explicit local Symbolica override. The exact
tested revision has since been published in
[Symbolica PR #54](https://github.com/symbolica-dev/symbolica/pull/54), targeting
`community`, and pinned from the public `ValentinHirschi/symbolica` fork in all
three FastSecDec consumer workspaces. No implementation change or new timing run
is implied by replacing the source path with that public pin.
The timed runs precede integration with the concurrent runtime complex-branch
repair `2092eaa`. Their ggHH artifacts already use complex arithmetic. The
combined-source publication checks exercise that repair separately; the timings
below remain the recorded runs, not a fresh performance measurement after rebasing.

Run cards, generated pairs, logs and full numerical reports are retained under
the ignored `output/numerical-dual-study/bench/` directory. The permanent ignored
CLI test `numerical_dual_benchmark::compare_saved_numerical_dual_artifacts` reads
`FASTSECDEC_DUAL_BENCHMARK_CONFIG`, a JSON file declaring four named artifact
basenames, the two reference/candidate pairs, the runtime parameter card and the
report path. This keeps large generated artifacts and raw measurements out of Git.

The comparison restores artifact symbol attributes before parsing runtime values,
then binds `examples/gghh_double_box/point.toml`. It matches native source-chart
geometry and validates and applies each lane's representative permutation,
divides representative contributions by their chart multiplicity, and adds
exact offsets once. All real and imaginary
Laurent components are retained, including poles. Taylor and IBP are compared
between generation modes separately; different subtraction strategies need not
have pointwise-equal densities.

The primary pointwise check uses 16 deterministic native Havana points per chart
in `[0.15,0.85]` and three equal-coordinate points at 0.01, 0.2 and 0.8. The seed
is 4917. Its componentwise tolerance is `1e-10 + 1e-8*max(abs(a),abs(b))`.
An explicit alternate comparison can use paired native QMC channels with the
complete cross-lane covariance when conservative factorizations produce different
but analytically equivalent IBP densities; it does not loosen the pointwise test.

Warm evaluator measurements use native f64 timing spans inside the existing
weighted evaluation context. Four batches warm each sector, followed by 64
repetitions of a fixed 256-point block in `[0.2,0.8]`. Max-weight escalation is disabled only for
this diagnostic, and tiny positive distance thresholds admit interior points
to f64. The test requires every measured point and evaluator call to remain f64.
It records each sector's average, median-batch and 95th-percentile-batch cost,
separately from worker wall time and cold loading/binding. These are evaluator
costs per sample, not full integration throughput or convergence timings.

## Generation measurements

These are single runs; timing variability has not been estimated. Recorded total
generation time includes input, parameterization, map/endpoint work, evaluator
construction and host compilation. `/usr/bin/time -l` additionally covers CLI
startup, final artifact writing and exit.

| Subtraction | Symbolic total | Numerical-dual total | Observed speedup |
|---|---:|---:|---:|
| Taylor | 115.25 s | 23.34 s | 4.94× |
| Integrate by parts | 118.58 s | 32.28 s | 3.67× |

All four runs retain 30 source charts and 30 numerical sectors. Every dual chart
uses the dual route, with no symbolic fallback. All outputs retain epsilon
orders −1 and 0, each with real and imaginary components. The numerical-dual
Taylor and IBP binary payloads are byte-identical for this diagram: after its
monomial extraction no higher endpoint power needs IBP lowering. Nontrivial
higher-power IBP is covered by the separate analytic controls.

For Taylor, the saved binary is 2,477,048 bytes in symbolic mode and 17,438,972
bytes in numerical-dual mode. Saving the new generation route is therefore not
an artifact-size optimization. Both artifact members contain no absolute host
paths. The OS reports peak resident sets of 3.81 GB symbolic and 2.90 GB dual;
its separate physical-footprint counter reports 1.15 GB and 1.42 GB respectively.
Those macOS counters measure different things and must not be interchanged.

### Where generation time goes

The saved phase timings are elapsed stage times for the eight-worker run, not
summed CPU time. The dual route records these costs:

| Stage | Taylor | Share | IBP | Share |
|---|---:|---:|---:|---:|
| Input | 0.123 s | 0.5% | 0.090 s | 0.3% |
| Parameterization | 1.652 s | 7.1% | 1.737 s | 5.4% |
| Formal chart preparation | 10.468 s | 44.8% | 11.636 s | 36.1% |
| Evaluator construction, optimization and backend compilation | 11.076 s | 47.5% | 18.783 s | 58.2% |
| Domain, geometry and unassigned coordinator time | 0.024 s | 0.1% | 0.031 s | 0.1% |

In dispatched generation, the existing `coefficient_expansion_seconds` field
wraps the whole dual chart-preparation call. It includes conservative polynomial
valuations, map/endpoint metadata, formal Taylor or IBP recipes, epsilon expansion
and conditioning profiles. Its name does not imply that all of this time is
epsilon algebra. Similarly, a zero `mapping_seconds` here means there is no
separate full symbolic map-substitution stage, not that coordinate preparation
has no cost. The density-symmetry stage is skipped.

`compilation_seconds` includes building and optimizing source evaluators,
lowering native dual jets to scalar instructions, composing and simplifying
the selected outputs, and SymJIT O2 compilation and initialization. The present
instrumentation does not split those substeps, so it cannot identify SymJIT or
Horner/CPE optimization alone as the largest remaining cost. The derivative
programs are built during generation and executed as ordinary native evaluators;
there is no new runtime interpreter for dual recipes.

Parameterization still performs the original numerator/Gaussian algebra. The
new lane avoids the large per-sector expression substitution and differentiation,
while retaining exact support/valuation work and smaller formal subtraction
algebra. Most remaining time is therefore chart preparation plus construction
of the larger numerical programs. More detailed profiling is needed before
choosing the next optimization; these single runs do not isolate cache effects.
Chart and evaluator jobs use the caller's worker pool. In these original runs,
shared valuation and source-program caches serialized construction on cache
misses; their waiting time was not separately measured, so these stage totals
do not establish the serial fraction. The follow-up replaces the source/jet
cache's construction lock with per-key native owners; the valuation cache is
unchanged.

## Numerical parity and runtime cost

Both pairs pass the complete Laurent-vector pointwise check. The largest absolute
difference is `1.4551915228366852e-11`; the largest relative difference, using
`max(abs(a),abs(b),1e-10)` as denominator, is `3.126308197951741e-15`. These maxima
are over the chart-normalized full sums, not individual-chart error maxima. No
paired statistical fallback was needed. Both symbolic Taylor/IBP payloads are
also byte-identical; the variation between their construction timings is not a
difference between final executed kernels.

| Subtraction | Symbolic f64/sample | Numerical-dual f64/sample | Observed slowdown |
|---|---:|---:|---:|
| Taylor, batch 256 | 1.954 μs | 23.124 μs | 11.84× |
| IBP, batch 256 | 1.950 μs | 23.284 μs | 11.94× |

Each average uses 491,520 measured sample evaluations over 30 sectors, from
exactly 1920 native matrix calls. No measured evaluator call used DoubleFloat,
Arb or the conditioning evaluator. Per-sector Taylor means span 1.816–2.201 μs
for symbolic programs and 22.288–26.407 μs for dual programs. The slowest recorded
averages are sector 1 symbolic and sector 13 dual. The corresponding IBP ranges
are 1.813–2.187 μs and 22.555–23.487 μs. Identical payloads across the two
subtraction settings make their small runtime differences measurement variation.

Every evaluator retains 21 runtime inputs (15 Gram products and six model inputs),
plus six integration coordinates. Sector 0 illustrates the increased numerical
work: its symbolic program has 2287 additions, 3615 multiplications, 12 inversions
and five function calls; its dual program has 20051 additions, 13198
multiplications, 15 inversions and eleven function calls. Cold loading and binding
took about 0.50 s per symbolic artifact and 2.49–2.50 s per dual artifact.

This option exchanges generation work for a larger numerical program. For this
double box, faster generation comes with roughly twelve times the primary f64
cost in the configured batches. The default remains symbolic. Actual integration
throughput also depends on sampling, precision routing and convergence; none of
those costs is included in this evaluator-only ratio.

The separate true-scalar check also passes full-vector parity. It uses
`batch_rows=1` to call `evaluate_weighted` directly, with 4096 measured calls at
one seeded interior point per sector (122,880 per artifact) and no matrix
invocations. Taylor costs are
3.138 μs symbolic versus 33.426 μs dual (10.65×); IBP costs are 3.210 μs versus
32.514 μs (10.13×). All calls remain f64. The 256-point batches improve both
representations relative to these scalar calls; a one-row matrix call was not
substituted for this measurement.

## Development evidence and remaining boundaries

Two earlier construction attempts were rejected during development. Expanding
regular numerator supports stalled before geometry; the replacement uses native
one-scale leading Series to obtain conservative valuations. A subsequent attempt
reached evaluator construction quickly but retained unused jet instructions and
duplicate constants, taking over two minutes without a completed sector and
reaching a sampled 10.5 GiB RSS. Native output-liveness pruning, constant
deduplication and the existing instruction-CSE pass resolve that observed case.
These failed attempts are not included as successful measurements.

The scientific controls, exactness boundary fixes and independent owner reviews
are recorded in [subtraction](numerical-dual-subtraction.md),
[native composition](numerical-dual-native-composition.md),
[evaluator construction](numerical-dual-evaluator.md) and
[options and persistence](numerical-dual-options.md). Portable host tests are
separate evidence from actual Pyodide/Wasm execution. No three-loop generation
or rebuilt Python-wheel runtime validation is claimed here.

Final coordinated gates pass 515 native workspace tests (26 explicit ignored
diagnostics), 71 portable host tests, strict workspace all-target Clippy, isolated
binding Clippy with stub generation enabled, and root/leaf formatting. The 12 new
shared numerical-dual controls run under both arithmetic backends. The full
double-box parity/performance test is explicitly invoked in addition to the
ordinary ignored-test gate. Those original measurements used the documented
local dependency override; public-source validation is recorded separately.

After integrating the concurrent complex-branch repair, the public-source gates
pass 527 native tests (26 intentionally ignored) and 72 portable host tests,
including 13 shared numerical-dual controls. Strict all-target workspace Clippy,
binding/stub Clippy and the optimized CLI build pass. The core's 359 tests passed
in the workspace run; the remaining packages passed separately after updating
three test fixtures that assumed real-only layouts. Those fixtures now verify
explicit real/imaginary channels, analytic values and complete covariance rather
than dropping the new zero imaginary components. No production numerical guard
was weakened. This completes source delivery validation, not a new timing run or
an installed-wheel/browser runtime validation.
