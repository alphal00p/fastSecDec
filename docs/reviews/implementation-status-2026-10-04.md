# Implementation and performance status, 2026-10-04

This snapshot answers the requested capability, generation, eight-core
one-per-mille convergence, and per-sector sample-cost status. It is a progress
record, not phase-one acceptance. The governing plan remains
[`FIRST_PHASE_PLAN.md`](../../FIRST_PHASE_PLAN.md).

The user clarified the convergence target: **the largest signed requested
epsilon power**, usually the finite coefficient, rather than the most singular
pole. The tables retain their original measurements; no eight-core crossing
measurement has yet been recorded under either interpretation.

## Capability coverage

Implemented: native HEPKit/FeynKit/Linnet input and graph conventions; scalar
and polynomial numerator parameterization; exact native sector geometry and
symmetry; endpoint subtraction and complete direct Laurent evaluators; portable
SymJIT O2 and native high-precision rescue; caller-owned MC/QMC workers and
integration; checkpoint/resume, numerical-only saved results, typed diagnostics,
qualified sector selection, and the CLI dashboard. Numerica's QMC extension is
published as upstream PR #8. Production remains Rust-only without pySecDec.

All 24 run cards and 17 native DOT fixtures load. Native one-loop controls cover
twelve scalar and eight numerator points; the coupled two-loop numerator has
independent density and Laurent-vector controls. All six massive multiloop
fixtures have independent references and a completed 72-row holdout. Off-shell
triple-box scalar/rank-two and both difficult orthant cases generate and integrate
complete vectors, with their scientific certification still pending.
The double-box full vector now has an independently audited pySecDec reference;
the retained 64-shift native result agrees within 1.27 combined standard errors
at all five orders. Its uncertainty calibration remains open.

Remaining coverage: complete on-shell triple-box generation; independent
difficult triple-box/orthant vectors; difficult-case
convergence/calibration; final matched performance and platform gates. The latest
combined workspace gate passes **281 tests**, with seventeen explicit probes
ignored. Formatting and all-target Clippy pass. It includes the CLI, family/input,
experimental Series-first and Laurent capture/depth regressions; the separately
executed expensive scientific campaigns retain their own evidence.

The additional regression covers a confirmed native Symbolica defect: an
underscore-suffixed regulator could make Gamma's Laurent series silently empty.
A minimal literal-substitution patch passes its native regression and the full
FastSecDec gate. The first two formal-function proof attempts are rejected as
empty-vector evidence; the corrected six-case proof passes. This dependency fix
does not close the on-shell generation or scientific-reference gaps.

The on-shell bottleneck is Laurent expression growth. The first Series-first
experiment completed one difficult representative but took about 83 s and
5.37 GiB, worse than the earlier approximately 42 s / 1.22 GiB ordering. It stays
test-only. The small proof using native formal derivatives and shared FunctionMap
bodies passes complete-vector O2, MPFR and portable-reload checks under both
inlining policies. Its first actual-representative attempt is a negative result:
preparation takes 144.909 seconds and leaves a 17.1 MB expression, followed by a
180-second native-series timeout without coefficients. Public native derivative
callbacks are the next bounded proof, to preserve early polynomial zero pruning.
That oracle now supplies all six coefficients at all three prescribed exact
points, including two near-boundary points; native 512-/1024-bit agreement and
an exact absolute-/relative-series control pass. This is representative-level
evidence, not an integral-level convergence result.

## Generation and fixed-work measurements, alongside Pathfinder

The backend is Symbolica 3.0.1 / SymJIT 2.26.4, portable O2. Triangle/box figures
are seven-run medians of the internal generation timer including compilation;
the larger figures are single bounded whole generation processes including
artifact output. Those different boundaries are deliberate labels, not a
cross-case benchmark. Pathfinder's two small generation entries are its reported
sum of timing records, including a nested symmetry interval counted inside its
decomposition interval; they are not an equivalent elapsed stopwatch. Formula
caches were available. No generation speedup ratio is inferred. Projected
families are exact native-family diagnostic
callers; automatic CLI projection is not enabled.
These measurements retain their frozen binaries from before the fourth local
Symbolica patch; they have not been relabelled as measurements of the new build.

| Case | Native kernels | FastSecDec generation | Pathfinder generation |
| --- | ---: | ---: | ---: |
| Triangle | 2 | 9.73 ms, internal median | 319.81 ms, reported record-sum median |
| Box | 3 | 15.37 ms, internal median | 332.53 ms, reported record-sum median |
| Off-shell triple box | 1182 | 51.705 s original; 18.373 s projected | Not measured here |
| Off-shell triple box, rank two | 1182 | 55.406 s original; 16.411 s projected | Not measured here |
| Issue 1 | 328 | 3.463 s | Not measured here |
| Hard four-loop orthant | 699 | 36.755 s | Not measured here |
| On-shell triple box | No completed artifact | Original and projected trials exceeded approximately 300 s | Not measured here |

At identical actual point counts and one worker, the seven-pair whole-command
medians are **1.012 s native / 2.884 s Pathfinder** for triangle (262,144
evaluations), and **1.232 s / 4.404 s** for box (393,216). These share the process
timer boundary, but backend/precision/persistence differences remain. Native
uses Symbolica 3.0.1 / SymJIT 2.26.4; the frozen Pathfinder environment uses
Symbolica 2.1.0 / SymJIT 2.18.6, with complex O2 and its own precision policy.

The original larger native fixed-work numerical/reporting phases are 108.146 s
and 35.236 s for the scalar/rank-two triple boxes (9,682,944 evaluations each),
1.043 s for issue 1 (2,686,976), and 95.912 s for the hard orthant (5,726,208),
all with two workers. No corresponding complete Pathfinder timing is claimed.

The two small whole-run timings include generation; the larger numerical-phase
timings exclude artifact loading. The larger runs predate the CLI observation
cadence improvement and include frequent reporting/checkpoint costs. The separate
rank-two cadence experiment lowered median integration time from 35.183 s to
28.135 s while preserving the complete numerical state. The timings above are
not eight-core measurements or time-to-accuracy claims.

## One-per-mille convergence on eight cores

**Not measured yet for either program.** No recorded experiment establishes eight-core elapsed time
to `standard_error / abs(coefficient_at_highest_requested_order) <= 0.001`. Existing hard-case
trials use two workers and stop at their fixed allocations without convergence;
the holdout uses one worker. Do not extrapolate them by dividing elapsed time by
four or eight. The triangle/box runs exceed the requested precision at their
fixed budgets, but do not establish the earliest target-crossing time.

The six-case holdout completes 223,838,208 evaluations without failures or
five-standard-error reference flags. At 8192 points HKKN's estimated error is
lower for all six fixtures; at 1024 it is higher for four. This supports a
measured lattice/transform campaign, not an automatic default change.

Relative error to zero is undefined. If the selected highest-order coefficient
is known to vanish, use an explicit absolute criterion without switching
coefficient. Known cancellations in lower orders retain separate absolute
checks, while the highest requested order remains the convergence target.
The [approved baseline protocol](eight-core-accuracy-protocol.md) uses
eight distinct available physical cores, records observed target
crossing and full-vector scientific checks, and separate generation/loading from
integration. Estimated one-per-mille error alone is not independent certification.

## Current per-sector sample-cost evidence

Saved native contribution records already retain accepted points and worker time
for every sector. Their ratio includes lattice generation, periodization,
complete-vector evaluation, precision rescue and native sample accumulation.
It excludes coordinator status/checkpoint work and is not isolated scalar-JIT
latency. Pathfinder's available per-sector bucket combines its evaluator and
Python work, including rescue, weighting and within-shift reduction. It excludes
global lattice/transform/integrator work and later accumulation/prefactor
convolution. Thus these columns retain distinct timing boundaries and cannot
justify a per-sample speedup ratio. The two maximum columns below are each the
**largest sector-average cost**, not the
maximum latency of an individual sample. Individual sample maxima were not
instrumented in these runs.

| Case | Workers | Native average µs/sample | Pathfinder sector average µs/sample | Native slowest-sector mean µs/sample | Pathfinder slowest-sector mean µs/sample |
| --- | ---: | ---: | ---: | ---: | ---: |
| Triangle, medians over seven runs | 1 | 3.761 | 5.409 | 7.407 | 7.985 |
| Box, medians over seven runs | 1 | 3.041 | 6.916 | 4.729 | 8.529 |
| Off-shell triple box, original | 2 | 13.269 | Not measured | 195.885, sector 474 | Not measured |
| Off-shell triple box rank two, original | 2 | 3.753 | Not measured | 78.284, sector 495 | Not measured |
| Issue 1 | 2 | 0.295 | Not measured | 0.365, sector 0 | Not measured |
| Hard four-loop orthant | 2 | 25.944 | Not measured | 71.703, sector 686 | Not measured |

Pathfinder's separately charged global integrator work adds a median 2.015
µs/sample for triangle and 1.493 µs/sample for box; its sector bucket does not
assign that work to individual sectors. Evaluator-only Pathfinder averages are
3.239 and 3.836 µs/sample, respectively, and likewise are not comparable to the
broader native worker timer. Neither program's individual-sample maxima are
available. The complete 3461-row extraction is retained as
`output/diagnostics/status-comparison-20261004/sector-costs.csv`, with the source
result path and timing boundary in each row.

All large rows use 1024 points times eight shifts per sector; the small rows use
8192 times sixteen. Sector IDs belong to their retained artifacts and cannot be
matched to another decomposition by number. Later profiling will distinguish
ordinary/rescued samples, actual per-sample tails, and point-generation/transform
cost, while measuring instrumentation overhead. The broad optimization campaign
follows capability coverage; the on-shell generation fix is needed to reach it.

Source evidence: [paired small cases](first-paired-performance.md),
[off-shell cases](triple-box-offshell-diagnostics.md),
[native projection](native-family-projection-diagnostics.md),
[hard orthant](hard-four-loop-diagnostics.md),
[on-shell and issue 1](onshell-triple-box-and-issue-one.md),
[holdout](massive-holdout-stage-a.md), and
[cadence comparison](cli-status-performance-attribution.md). Exact per-sector
means are derived from each named native saved result's `worker_seconds` and
`used_points`; no independent numerical estimator is introduced.
The [independent comparison review](first-paired-performance-independent.md)
verifies both programs' timing definitions and the side-by-side values.
