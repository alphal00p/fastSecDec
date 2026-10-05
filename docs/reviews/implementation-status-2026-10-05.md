# Implementation and measured performance, 2026-10-05

This is a progress report against `FIRST_PHASE_PLAN.md`, not phase-one
acceptance. The convergence target is the **largest signed requested epsilon
power**: epsilon zero in the small cases below and epsilon two for Issue 1.
Reported relative standard error is not a certified bound on true error.

## Plan coverage

Implemented and audited: Rust-only HEPKit/FeynKit/Linnet input, graph and
polynomial-numerator conversion; exact sector geometry and symmetry; endpoint
subtraction and complete Laurent-vector direct evaluators; portable SymJIT O2,
conditioning and native MPFR rescue; caller-driven MC/QMC, covariance,
checkpoint/resume, typed status, results and the CLI dashboard. Numerica's QMC
extension is published as PR 8. The native production dependency graph excludes
Python and pySecDec; external reference execution is a development activity.

The latest complete workspace gate passes **330 tests**, with twenty-two explicit
probes ignored (`output/dispatch-named-rank2-workspace-tests.log`, 59 summaries,
zero failures). This includes cache/context dispatch, parallel-geometry,
sector-identity, small named-coefficient and rank-two reference controls.
Formatting and all-target Clippy pass; the latter finishes in 3.14 seconds.
The separate ignored small named-program writer/reader were explicitly executed
and independently reviewed before this gate. The earlier
alias milestone passed 294 tests; its timing observations keep their own source
identities. Actual PTYs additionally verify monochrome output, wide/compact
resizing, key cancellation and terminal restoration.

All 24 run cards and 17 modern native DOT fixtures load. Independent controls
cover twelve scalar one-loop and eight numerator points; the coupled sunset,
six massive families, double box and Issue 1 have further independent evidence.
The off-shell scalar triple box subsequently gains a checked independent full
`[-3,-2,-1,0]` reference: separate native original/projected comparisons retain
their full covariance and have maximum pulls below 2.816/2.023. Five focused
reference tests pass, with two recorders ignored. The finite reference value
`2.283413488400494 +/- 0.05718537938081278` has about 2.5% relative uncertainty;
this closes a reference gate, not calibration, performance or the 1‰ target.
The off-shell rank-two reference now also passes independent source, full-vector
and native transport checks. Its finite value is
`0.6263784670955248 +/- 0.01044026507101012` (about 1.67% relative SE), with
maximum original/projected comparison pulls 1.703/1.727. Both native covariance
matrices remain unchanged and the observations are compared separately. Six
focused reference tests pass; the fixture does not certify calibration or 1‰.
The historical matrix has 100 Covered, 81 intentionally Retired and one Partial
row out of 182. This matrix does not replace complete difficult-example gates.

The compact production alias pipeline and version-three native evaluator-IR
artifacts are committed at `6332676`; the cache core at `2db942b`. Native
`AliasedAtom` roots and images stay compact, and O2/conditioning/MPFR share the
same exact native program. The difficult representative passes eighteen
original-expression oracle comparisons and seventy-two fresh/decoded weighted
component checks. This is representative-level evidence. All performance
tables below retain their older frozen build identities and **do not measure
the newly committed alias pipeline or cache**.

Still open: complete original on-shell triple-box generation and full-vector
validation; the independent hard-orthant reference;
difficult-case convergence and error calibration; CLI adoption of
caller-owned chart/cone dispatch; matched performance and
platform gates. Cache adoption and the identified CLI color/terminal gaps are
now covered by the gates above. General
affine upper-cube endpoint charts remain explicitly unsupported. Future Python
bindings and phase-two contour/GCAD algorithms are outside this phase.

The additive sector content-ID API passes four focused tests,
independent review, formatting and library Clippy. It hashes the current native
representation and associated semantics without changing parent artifact or
checkpoint identities. It does not establish mathematical equivalence or a
machine-code cache. The native parallel geometry API also passes its focused
38-test sector gate and independent review. Callers own all workers; canonical
merge preserves exact maps, failures, cancellation and resource limits. The main
generation context now has an additive dispatched route through the same native
cache and symbolic pipeline. Its focused 41 sector and eight context tests pass,
including caller-owned threads, complete analytic vectors, cache reuse and
rejection/cancellation boundaries. Existing methods retain their serial route.

The next compact-coefficient composition is test-only. Eight small controls pass
using native Series, differentiation, face substitution and shared aliases,
with full coefficient comparisons. Separate bounded evaluator/precision and
cold-reload controls now pass 96 complete weighted-vector calls (768 real/imaginary
component checks), with independent unchanged-subtraction references stable at
512/1024 bits. The actual difficult representative remains a pending gate;
this test-only strategy is not the production default.

## Small-case generation and eight-core accuracy

| Metric | Triangle FastSecDec | Triangle Pathfinder | Box FastSecDec | Box Pathfinder |
| --- | ---: | ---: | ---: | ---: |
| Fresh generation process, one observation | 0.034402 s | 0.942928 s | 0.019430 s | 0.869645 s |
| Integration to first observed allocation below 1 per mille, eight cores | 0.032969 s | Instance-limit abort | 0.041599 s | Instance-limit abort |
| Full integration process at that allocation | 0.043068 s | Unavailable | 0.054016 s | Unavailable |
| Artifact load/O2, separate median | 0.004714 s | Unavailable | 0.006712 s | Unavailable |
| Pooled worker mean per sample, eight cores | 3.884 us | Unavailable | 3.841 us | Unavailable |
| Slowest sector's worker mean per sample, eight cores | 7.600 us | Unavailable | 5.886 us | Unavailable |

The accuracy rows are seven prescribed-seed medians on eight distinct physical
cores, using Kuo33002/Korobov3, 1024 points and sixteen shifts per sector.
All fourteen runs meet the epsilon-zero criterion at the first complete tested
allocation: 32,768 triangle or 49,152 box full-vector evaluations. Earlier
crossings and minimal required work are unmeasured. Finite-part relative SE
ranges are 4.46e-9–6.06e-9 and 5.95e-7–9.50e-7 respectively; all runs retain the
complete `[-2,-1,0]` vector/covariance and have no evaluation failures.

Native generation compiles O2 and saves canonical expressions; loading compiles
those expressions again. The reference saves evaluators for lazy loading and
had formula caches available. The native
baseline uses Symbolica 3.0.1 with four local fixes and SymJIT 2.26.4; the frozen
Pathfinder environment uses Symbolica 2.1.0/SymJIT 2.18.6. Consequently even the
whole-process generation rows do not establish matched speedup. Pathfinder's
eight-worker runs abort at Symbolica's concurrent-instance license check; no
eight-core reference time is inferred from lower-worker results.

The broad native worker timer includes point generation, transformation,
complete-vector evaluation/rescue and accumulation in accepted packages. A
slowest-sector mean is not the maximum time of an individual sample. See the
[eight-core results](eight-core-native-results.md) and
[smoke/generation evidence](eight-core-smoke-results.md).

## Sample costs and individual maxima

For an actually measured side-by-side comparison, the older seven-pair campaign
has one worker, 8192 points and sixteen shifts per sector:

| Case | FastSecDec pooled worker mean, us/sample | Pathfinder sector-bucket mean, us/sample | FastSecDec slowest sector mean, us/sample | Pathfinder slowest sector mean, us/sample |
| --- | ---: | ---: | ---: | ---: |
| Triangle | 3.761 | 5.409 | 7.407 | 7.985 |
| Box | 3.041 | 6.916 | 4.729 | 8.529 |

These clocks have different boundaries. Pathfinder additionally charges global
integrator work of 2.015/1.493 us per sample, which is not assigned to sectors.
Its evaluator-only means are 3.239/3.836 us per sample. Neither side's bucket is
an interchangeable arithmetic-only measurement. See the
[independent paired review](first-paired-performance-independent.md).

A separate eight-core, single-seed native diagnostic times each weighted
complete-vector kernel call, including conditioning/rescue/replay and possible
OS interruptions, excluding point generation, transformation and accumulation:

| Case / original sector | Native mean, us/sample | Native observed individual maximum, us/sample | Pathfinder mean / individual maximum at this boundary |
| --- | ---: | ---: | --- |
| Triangle / 0 | 0.141 | 182.151 | Not measured |
| Triangle / 1 | 8.380 | 3065.290 | Not measured |
| Box / 0 | 7.093 | 6167.589 | Not measured |
| Box / 1 | 0.138 | 198.040 | Not measured |
| Box / 2 | 5.843 | 5286.366 | Not measured |

Each row contains 16,384 samples. Pooled means are **4.261 us** for triangle and
**4.358 us** for box; observed maxima are **3.065 ms** and **6.168 ms**.
All maximum samples used precision rescue, but their entire elapsed latency
cannot be attributed to arithmetic. These are finite observations, not
worst-case bounds. Instrumented/uninstrumented vectors, covariance and accepted
replay state agree exactly. The median empty clock bracket is 30 ns, retained
without subtraction. See [the sample-latency record](native-sample-latency-results.md).

## Larger cases

These are older single native generation observations and fixed-work sample
costs with **two workers**, 1024 points and eight shifts. The off-shell sample
costs belong to the original graph campaigns; only the generation column also
lists projected-family measurements. These rows do not supply an
eight-core time to one-per-mille accuracy or individual-sample maxima.

| Case | Native generation process | Pathfinder generation | Native pooled / slowest-sector mean, us/sample | Pathfinder matching costs | Eight-core last-order time to 1 per mille, either program |
| --- | ---: | --- | ---: | --- | --- |
| Off-shell triple box, scalar | 51.705 s original; 18.373 s native projected family | Not measured | 13.269 / 195.885 | Not measured | Not measured |
| Off-shell triple box, rank two | 55.406 s original; 16.411 s native projected family | Not measured | 3.753 / 78.284 | Not measured | Not measured |
| Issue 1 | 3.463 s | Not measured | 0.295 / 0.365 | Not measured | Not measured |
| Hard four-loop full orthant | 36.755 s | Not measured | 25.944 / 71.703 | Not measured | Not measured |
| Original on-shell triple box | No accepted complete generation | Not measured | Not available | Not measured | Not measured |

All larger numerical observations preserve full vectors, not a selected pole
or favorable sector. The separate checked Issue 1 reference reaches epsilon-two
relative SE about 0.001285, still above the requested 0.001. General estimator
calibration remains open even for a checked reference. No two-worker timing is
scaled to pretend to be an eight-core result. See the source-linked
[earlier detailed snapshot](implementation-status-2026-10-04.md), and the
[remaining-gates audit](phase-one-remaining-gates.md).

### New on-shell capability trial

The release build from `6332676` did not complete the original on-shell triple
box within its predeclared 1,800-second generation limit. It completed 80 of
1,026 representative Laurent stages, then reached the deadline during
representative 81 (872 endpoint terms). The original five-second grace was
followed by SIGKILL; the process was reaped after 1,805.689 seconds, with peak
RSS 7,085,428 KiB. No kernel artifact or numerical result was produced, and
inspection/integration did not start. All 26 frozen-input checks passed.

The physical card, ten-parameter graph, requested order zero and complete-vector
scope were unchanged. Source and native dependency archives bind the frozen
release binary. This bounded capability result does not replace the older
benchmark tables or establish performance parity; guarded external compilation
overlapped on other CPUs. See the [full terminal evidence and limits](native-alias-production-results.md)
and [source-only attribution](native-series-fullgraph-attribution.md). The
on-shell coverage gate remains open.

A subsequent preparation-only comparison replayed the actual failed
representative's mapped input and proved exact identity with its 872-term
Taylor expression in 32.382 seconds. The existing IBP alternative then failed
to finish within the unchanged 180-second total bound (180.483 s observed,
6,020,136 KiB peak RSS). It produced no IBP expression or evaluator. All 52
frozen input checks passed and the child was reaped. This is a separate
development probe, not an end-to-end generation time; see
[the IBP protocol and terminal record](native-triplebox-ibp-proposal.md).
