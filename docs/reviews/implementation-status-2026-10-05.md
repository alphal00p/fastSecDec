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
extension is published as PR 8. Its unchanged branch also passes exact Rust
1.89.0/Cargo 1.89.0 on Linux: 237 default, 240 serde and 217 alternative-backend
tests, independently checked against unchanged source and lockfile hashes.
This closes the Numerica Linux minimum-version gap; other platform and
performance gates remain open. The native production dependency graph excludes
Python and pySecDec; external reference execution is a development activity.

The latest complete workspace gate passes **370 tests**, with twenty-three
explicit probes ignored (`output/coefficient-first-workspace-tests-2.log`,
62 summaries, zero failures). Workspace formatting and all-target Clippy pass;
the latter finishes in 11.81 seconds. This includes the public `NativeNamed`
opt-in, exact unregulated fallback, caller limits/cancellation, conservative
conditioning, context/dispatch, weighted precision replay, worker cloning and
cold native artifacts. The CLI reuses native option types and reports one
exclusive coefficient-expansion duration. Its request JSON cadence preserves
the existing geometry and final-output behavior. Earlier failed test/lint and
presentation-regression evidence is retained. `Physical` remains the default.
The preceding 336-test gate covered cache/context dispatch, parallel geometry,
sector identity, small named/interleaved controls and rank-two reference
transport.
The separate ignored small named-program writer/reader were explicitly executed
and independently reviewed before this gate. The earlier
alias milestone passed 294 tests; its timing observations keep their own source
identities. Actual PTYs additionally verify monochrome output, wide/compact
resizing, key cancellation and terminal restoration.

The difficult representative now also passes ordinary public generation,
compilation, saving and three independent cold readers. The complete process
takes 108.356 seconds: 66.752 seconds for generation and 37.370 seconds for
compile/save. All seven orders `[-6,-5,-4,-3,-2,-1,0]` agree at all three
prescribed points, and all 24 weighted vectors pass, including twelve forced
1,024-bit replays. The actual chart permutation, new multiplicity one and
literal-zero exact offsets are verified. This closes the public representative
gate; it does not measure full-graph generation or convergence. See the
[results](native-named-public-actual-results.md) and
[independent audit](native-named-public-actual-independent.md).

The user's stopping rule is explicit: close first-phase capabilities and
scientific checks, establish representative performance parity with bounded
effort, then complete the active goal and stop. Further marginal tuning and
threshold-phase planning do not follow automatically.

The subsequent CLI preparation slice passes **38 focused tests**, formatting
and CLI all-target Clippy. It delegates to the existing native prepared-family
API, retains the original default and records preparation/fallback provenance.
Complete original/prepared `210 Gamma(eps)` controls exercise generation, cold
artifacts and integration through order one. See the
[CLI results](cli-family-preparation.md) and
[independent review](cli-family-preparation-independent.md).

The ten-parameter NativeNamed whole-graph trial was intentionally cancelled at
372 of 1,026 representatives after 1,006.900 seconds (peak RSS 13.43 GiB), with
no artifact or numerical result. This was a coordinator cancellation, not a
timeout. The next trial uses the existing exact repeated-propagator preparation
to represent the same integral with eight active parameters. The independent
external reference's original and projected generation attempts both hit their
30-GiB process-tree memory limits, after 530.288 and 431.801 seconds respectively.
All processes are reaped and frozen checks pass; neither attempt supplies a
reference vector. These are capability diagnostics, not new benchmark rows.

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
The hard four-loop full-orthant reference is now also Checked, retaining all
four provider orders. The three common native/reference coefficients have
maximum pull 1.629. The old native vector/covariance remain unchanged and its
comparison explicitly lacks order minus three; a separate complete native
generation proves zero through that order. Seven focused reference tests pass,
two probes ignored, with focused Clippy and formatting passing. This reference
has about 0.57% finite-part uncertainty and does not certify calibration or 1‰.
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

The subsequent CLI-only slice passes all 49 CLI tests (three explicit probes
ignored), formatting and CLI all-target Clippy. `generate` and `run` accept
`--geometry-workers`, defaulting to one independently of integration workers.
Real PTYs exercise parallel geometry resize/cancellation and colored/monochrome
completion, with terminal restoration and no artifact after cancellation.
These focused gates initially followed the 330-test baseline and are now also
included in the preceding 336-test and current 370-test workspace gates.
No generation speedup is claimed.

Still open: complete original on-shell triple-box generation and full-vector
validation; the remaining representative accuracy and matched-performance checks;
and final delivery review. The [bounded completion ledger](phase-one-remaining-gates.md)
identifies existing uncertainty evidence to reuse and avoids additional open-ended
calibration studies. Unavailable platforms remain explicitly unverified.
Cache adoption and the identified CLI color/terminal gaps are
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

The initial compact-coefficient composition was developed test-only. Eight small controls pass
using native Series, differentiation, face substitution and shared aliases,
with full coefficient comparisons. Separate bounded evaluator/precision and
cold-reload controls now pass 96 complete weighted-vector calls (768 real/imaginary
component checks), with independent unchanged-subtraction references stable at
512/1024 bits. Its first actual difficult-representative composition attempt
times out at 180.116 seconds without coefficients; that attempt did not close
the actual-input gate.
This test-only strategy is not the production default.

A subsequent diagnostic identifies the expensive stage: native composition
reaches its required remainder in about 19.6 seconds, then mixed derivatives
grow to hundreds of megabytes before boundary substitution. The unchanged
180-second limit again expires without a coefficient vector. A test-only
resolver now applies each admitted constant face after its own required native
derivatives, with the original route retained for general arguments. Ten small
controls pass in 0.192 seconds, including exact mixed derivatives, fallback and
cache isolation, complete Taylor/IBP vectors and unsupported cases. Its native
program controls also pass all 96 weighted-vector calls and 768 component
checks across fresh, cloned, decoded and cold-reader kernels, with precision
rescue up to 320 bits. The captured actual representative then generates in
54.875504 seconds under the same 180-second limit, retaining all formal orders
minus six through zero and native remainder one. Its complete evaluator builds
in a separate 43.313581-second process and retains 14.58 MB of native exact IR.
The first independent original-expression oracle times out in native series
at 180.167 seconds without a coefficient vector.
A fresh unchanged-math point-zero attempt also times out, at 600.315 seconds,
without returning the full series. Both failures are retained; that Series
chain stops. Native factor extraction and high-precision automatic
differentiation subsequently pass a disconnected small gate: six cases and
forty signed coefficient rows agree with original-expression native Series at
512/1024 bits. Relative-only tiny nonzero controls, complex output, negative
requested maxima, nine typed rejections and a separate literal-zero control
also pass independent review. Following a separately tested numeric-transport
correction, the actual adapter completes all three prescribed points and gains
independent acceptance. The cold candidate reader then passes all 21 signed
coefficient/point comparisons and 24 weighted full-vector calls, with rescue at
256/384 bits and all immutable checks passing. Its analyticity/Taylor coverage
has a distinct diagnostic format and cannot be reported as a native Series
remainder. This closes the captured representative's experimental agreement gate;
the public representative subsequently passes the gate above. The full graph
remains open. No benchmark row below changes.

The first production adoption slice now extracts shared native endpoint
admission and checks cancellation-degree overflow. Its independently reviewed
focused gate passes **50 tests**, with two explicit probes ignored; package
formatting and all-target Clippy pass. Existing Taylor/IBP schedules, exact
pruning, error precedence and defaults are preserved. This is a focused gate;
that extraction preceded the complete 370-test public-adoption gate above. Separately,
the native symbol-hygiene probe rejects all 17 foreign metadata/hook conflicts
without executing callbacks and verifies occupied-input names, stable repeated
names and independent simultaneously live alias maps. The native API/probe
prerequisite and production allocator/private/public small-case gates are now
closed. Public reconstruction of the actual captured representative also
passes the gate above; whole-graph validation remains open. See the
[endpoint audit](native-endpoint-admission-independent.md) and
[symbol-hygiene audit](native-symbol-hygiene-independent.md), and the
[public integration record](native-named-public-integration.md).

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
| Original on-shell triple box | Timed out at 1805.689 s; no artifact | Not measured | Not available | Not measured | Not measured |

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

The subsequent named-coefficient experiment uses those same captured mapped
terms directly, without replaying the giant physical Taylor subtraction.
Its independently reviewed, frozen generation-only process times out after
180.116318 seconds, with peak RSS 1,831,116 KiB and no generated coefficient
files, final native remainder, evaluator or numerical result. All 89 immutable
hash checks pass and the child is reaped. The last recorded stage in this
uninstrumented attempt is native named Taylor composition. Program/oracle stages
do not start after this failure.
The small controls remain valid, but they do not establish feasibility on the
actual representative. See the [actual-target protocol and outcome](native-named-actual-proposal.md).
Guarded reference compilation overlaps on other CPUs, so this is a bounded
capability diagnostic, not a matched generation benchmark.

The separately frozen instrumented attempt also times out, after 180.111590
seconds, with peak RSS 1,736,912 KiB and all 103 input checks passing. Native
composition reaches absolute remainder one at width seven by 19.567 seconds;
lowering starts at 19.735 seconds. Completed native face substitutions occupy
142.895 seconds, including one 62.343-second substitution on a 367 MB mixed
partial that becomes only 33 KB after substitution. These are inclusive traced
intervals, not a disjoint CPU budget or a benchmark. No complete vector,
evaluator or original-expression oracle result exists. The
[phase attribution](native-named-phase-attribution.md) and
[independent audit](native-named-phase-attribution-independent.md) motivate the
[restricted native resolver experiment](native-interleaved-face-proposal.md).

That restricted candidate subsequently completes the same captured
representative in **54.875504 seconds**, with peak RSS **980,816 KiB**. All 101
frozen source/input checks pass; the successful child is reaped. It exports
formal orders `[-6,-5,-4,-3,-2,-1,0]`, 2,523 aliases, about 6.98 MB of roots and
89.62 MB of unique bodies, with native absolute remainder one at relative width
seven. These formal lower orders still require independent comparison against
zero where absent from the original oracle. Multiplicity four is not applied.
The original resolver remains the default. This closes one bounded generation
experiment, not numerical correctness, complete on-shell generation or a matched
speedup. See the [candidate results](native-interleaved-face-results.md) and
[independent review](native-interleaved-face-independent.md).

The separate actual native program stage also passes, retaining all seven
orders in 14,579,188 bytes of validated exact IR. The process takes 43.313581
seconds, including 3.712188 seconds for native exact program construction and
27.017606 seconds for production backend construction, with peak RSS
1,337,068 KiB. All 5,202 immutable checks pass. This compiles previously generated
coefficients; it does not include their generation or certify their values.
At that stage, the three independent original-expression oracles and complete
cold-reader comparisons remained required.

The first original-expression point fails at both separately declared limits:
180.167230 and 600.315441 seconds. Exact binding leaves a 22.69 MB Atom; native
relative width one returns leading order minus six and remainder minus five,
but the width-seven request never completes within either limit. The longer
attempt peaks at 3,617,212 KiB; its inputs remain unchanged and the child is
reaped. No complete oracle or dependent comparison was accepted from those
Series attempts. See the
[original-oracle record](native-original-taylor-point-first-oracle.md). This
failure initially limited validation of the candidate; it is not a timed production
integration or a coefficient disagreement.

The alternative native-dual adapter subsequently completes its first actual
point's mathematical stages, including exact pole removal and all seven
two-precision/realness checks, but exits after 28.704886 seconds during numeric
export. The adapter incorrectly expected output precision to equal requested
precision; native Float arithmetic intentionally tracks precision dynamically.
All 78 input and 29 build postchecks pass, but no complete oracle or accepted
numeric export exists. A narrow transport correction records the actual
component precisions separately, using native round-trip decimal formatting
without padding precision or changing arithmetic/tolerances. The corrected
writer's six cancellation/growth/zero cases and the reader's full-vector,
malformed-record and dynamic-precision controls now pass independent review.
The fresh point-zero attempt then succeeds in 28.316420 seconds, with peak RSS
589,832 KiB. Independent review accepts all seven orders, fourteen native
numeric exports, the original-expression proof and all 32 build/78 input
checks. Requested 512/1024-bit evaluations retain their actual real-component
precision ranges of 501–505/1013–1017 bits. The two remaining prescribed points
also succeed and pass independent review: 32.789804 and 31.645153 seconds, with
the complete seven-order vectors and fourteen native exports each. Actual
real-component precisions remain at least 455/967 and 479/991 bits, respectively.
All three original-expression oracles are accepted. The subsequent cold reader
also passes independent review: all 21 coefficient/point comparisons over the
complete `[-6,-5,-4,-3,-2,-1,0]` union, and 24 weighted cold/worker-clone vectors
(168 real components), including twelve forced replay vectors. All 24 calls
use precision rescue: 256 bits at the first two points and 384 at the third.
The reader processes take 60.168910, 61.542493 and 64.462582 seconds, with no
timeouts and all 5,417 immutable checks passing. This closes the captured
representative's candidate agreement. Public production integration and the
full original graph remain separate gates. The shared-helper repeat also passed
with every non-timing field equal to the original small gate. These diagnostics
do not change any benchmark row above.

### Hard four-loop independent reference

The external reference generated all 2,676 ordinary sectors in 243.998 seconds
and compiled its complete library in 843.013 seconds. Its numerical phase then
timed out after 750.953 seconds, after beginning the finite coefficient but
before returning the complete raw tuple. These external pySecDec timings are
not Pathfinder direct-kernel performance measurements. The failed attempt
remains unaccepted; partial coefficient observations are not pooled or promoted.

An independently reviewed fresh numerical-only attempt reuses the exact
compiled package, with the same caller, seed, full-sector sum, lattice and
transform. It returns the complete physical tuple within its predeclared
1,200-second limit, without regeneration or recompilation. All four external
orders, including minus three, are retained. Independent source, normalization,
original-tuple, exact-bit and versioned-transport audit passes. The checked
fixture is `examples/references/four_loop_hard.json`, and seven focused reference
tests pass with two explicit probes ignored. The
finite coefficient is `-150.88953098736903 +/- 0.8585945824027943`, about 0.57%
reported relative SE. This fixed-work reference does not reach 1‰.
See the [continuation protocol](hard-four-loop-numeric-continuation.md).

The old native result begins at order minus two and remains unchanged, including
its full covariance. A separate native generation requested through minus three
now proves exact zero there, with no numerical kernels, all 2,760 charts and
699 representatives accounted for, and matching portable geometry. Its process
takes 67.388 seconds; its recorded generation/validation span is 65.187 seconds,
with the native Complete event at 64.698 seconds. This is not a new finite-part
performance measurement. The original comparison
still explicitly reports the missing estimate; the symbolic zero proof is
separate evidence. See the [omitted-order audit](hard-four-loop-lower-order-scope.md).
