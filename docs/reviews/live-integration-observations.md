# Live integration observations, accuracy targets and operational timing

Implementation/review slice: caller-driven integration core and CLI execution,
2026-10-07. Other examples and permanent tests/gates remain deliberately
unmodified. Ignored focused evidence is under `output/live-integration/`.

## Native ownership and statistical boundaries

`integration/mc_live.rs` adapts Numerica's existing
`StatisticsAccumulator<DoubleFloat>::get_live_estimate`. It retains the first
sample as an origin and accumulates centered differences, reusing the same
native arithmetic and final batch-mean operations already used by Havana.
There is no new covariance engine, RNG, sampling grid, adaptation algorithm,
thread pool, or integration loop in the library. `HavanaWorker` and
`HavanaDiscreteWorker` expose borrowed per-point views; the caller chooses when
to take an owned prefix snapshot. A native failed/cancelled evaluation does not
produce an admissible return.

The CLI remains the executor. Prefix ledgers replace each `(sector,batch)`
ordinary-MC or `(batch)` discrete-MC entry, rather than adding successive
snapshots. A ledger contains one immutable sampling epoch only and is cleared
between pilots, production and refinement rounds. It includes completed work in
that epoch and current prefixes, so accepted batches are not added a second
time. Resumed partial epochs are explicitly labelled `SinceResume`: the native
checkpoint does not contain point-level moments for previously accepted
batches. After the restored epoch completes, subsequent fresh epochs use
`CurrentIteration`.

Accepted means, full Laurent covariance, sector contributions, adaptation,
checkpoint records and stopping evidence still come exclusively from native
complete returns. Pilot observations are labelled pilot observations. Cancellation
and failure discard prefixes from the ledger and from replay-state admission;
final reports clear the provisional layer. Ordinary MC now uses the same bounded
coordinator/worker separation as discrete MC: original slot assignment and
ordered return admission are retained, while cancellation is polled at most
50 ms apart between coordinator operations. A native point evaluation itself
is not preempted.

Native worker snapshots use the configured observation interval (default one
second), plus end-of-task publication. The coordinator uses the same dashboard
cadence for scalar pooling and native accepted observations. A five-second
interval therefore does not incur a hidden ten-Hz native snapshot reduction.
Prefix collection from worker slots is also lazy: ordinary 50 ms polls pass a
collector handle, and only a due observation invokes it. A final collection
once per completed wave retains each finished task before slot reuse. Operational
metric merges also occur inside that same due gate, or explicitly for the final
report; ordinary cancellation polls only copy small activity counters. Keyboard
interaction redraws cached observations and does not resample them.

## MC scalar preview formulas

For each selected sector, let `N` be all discrete draws in the observed prefixes,
`k` the number selecting that sector, and `m` its conditional mean of the final
importance-weighted values. Native per-prefix means and standard errors provide
within-prefix centered sums of squares; unequal prefixes are pooled with their
sample counts and between-prefix centered mean differences. Numerica
DoubleFloat and its native `hypot` combine normalized error terms.

The marginal preview uses implicit zeros for the other `N-k` draws:

```
mean = (k/N) m
SE² = selected_SSE / [N(N-1)] + k(N-k)m² / [N²(N-1)]
```

This includes selection uncertainty and does not allocate zero-valued samples.
Unvisited sectors report `NotSampled`, with no numerical mean/error; they are not
claimed to be known zero. One ordinary sample has a mean but no error. A single
selected draw among multiple discrete draws may have an empirical marginal
error because the other draws are known zeros. The global discrete total has
its own native accumulator, so it does not assume independent sector
marginals. Ordinary MC sums independently sampled sector means/errors. Exact
contributions enter the total once and do not enter sector rows. Preview range
failures remain unavailable observations, not zero-valued results.

These are scalar preview errors only. They never replace native accepted vector
covariance or justify convergence. The preview API documents its single-epoch
caller obligation.

## QMC observations

`QmcSession::live_observation` uses the existing QMC accumulator's complete
shift IDs and shift estimates. A gap in a shifted lattice excludes the entire
lattice. One complete lattice yields a mean only; errors require at least two
independent shifts. Sector rows use their own complete coverage. Democratic
totals use common shift IDs across every sector and reuse the authoritative
centered full-vector reduction when two or more common shifts exist. Adaptive
sectors retain independent streams and marginal errors. Exact offsets are added
once.

`live_observation_with_pending` accepts completed caller-held returns through
native submission on a temporary statistics-only session clone. It therefore
reuses all native task identity, interval, overlap and numeric validation without
mutating accepted coverage or checkpoints. The existing CLI QMC queue already
admits completed returns on arrival; its native canonical accumulation order is
unchanged, so the CLI does not introduce an extra holding queue solely for the
preview. Partially evaluated lattices never become visible means.

## Accuracy and timing contracts

`AccuracyTarget` defaults to historical `AllComponents` stopping. A selected
Laurent order uses the complete total complex magnitude and RMS vector error
`sqrt(C_RR+C_II)`. Cross-component covariance is retained in the result, and all
other Laurent orders remain present. A target order absent from the kernel
layout fails before the worker pool is built. Only a complete production
allocation may pass `assess_accuracy`, even if a preview appears precise.
The explicit accuracy target is persisted in native saved results and checkpoint
settings. Historical checkpoints missing stability retain `validated`, and
missing accuracy targets normalize to `AllComponents` before strict comparison.

Operational metrics are separate from accepted statistical cost records. They
include pilots, abandoned prefixes, and failed evaluator calls in this invocation.
`worker_seconds` sums active worker elapsed spans, not OS CPU time.
`integrand_seconds` is inclusive worker evaluator/context preparation plus the
weighted evaluation wrapper; `evaluator_seconds` is the nested native evaluator
call time. Thus exclusive wrapper effort is their difference. Coordinator
context preparation and active scheduling/admission/observation/checkpoint work
have separate spans; receive waits are excluded. The display's measured effort
denominator is worker elapsed plus those active coordinator spans. Process CPU
is independently read from the OS and force-refreshed at the final driver
boundary. No wall-minus-workers inference is used.

Per-sector native point spans remain separate from whole-task unassigned setup
and reduction overhead; they are not expected to sum to global worker elapsed.
Native evaluator timings and final classification counters are attributed even
when a result is rejected. Largest sector contributions use the final finite
importance-weighted complex coefficient magnitude, grouped by Laurent order,
for this invocation. Exact offsets and superseded lower-precision attempts do
not inflate these maxima.

## Focused evidence

- `native-observation-probe.rs`: discrete unequal 7/13-point prefixes matched an
  explicit 20-draw calculation for the total and both sparse-zero marginals;
  duplicate ledger entries were rejected; exact offset was added once and
  checkpoint bytes stayed unchanged.
- The same probe checked ordinary 1/9-point prefixes near a `1e15` origin,
  preserving the small standard error; a single point exposed no error.
- QMC gap, single-shift and reversed pending-return cases passed. Pending
  previews left checkpoint bytes unchanged and matched the eventually admitted
  complete estimate exactly. An absent target order failed, a selected complex
  target passed while an unrelated order failed, and incomplete production
  could not stop.
- `driver-probe.py`: preserved prior binary versus new validated-policy CLI
  controls matched the entire mean/covariance and every sector estimate exactly
  for MC, adaptive MC, discrete MC and QMC at one and eight workers. The fixture
  has six stochastic sectors and a four-scalar complex Laurent vector.
  Adaptive QMC's time-based allocation was not claimed bit-identical.
- Historical completed checkpoints with neither stability nor accuracy target
  resumed with identical estimates and zero new evaluations, for one/eight
  workers.
- Long ordinary/discrete MC batches with one/eight workers exposed multiple
  live per-sector means/errors while accepted points remained zero. SIGINT
  latency was 39–76 ms on this host. Final reports omitted cancelled prefixes;
  operational counters retained 189,005–1,334,773 attempted points across these
  controls. Actual counts depend on scheduling and are not a speed claim.
- Five-second observation cadence during a 0.4-second cancelled ordinary batch
  produced no native prefix snapshot; interrupt response remained bounded.
- Global operational spans satisfied worker >= inclusive integrand >= evaluator
  in the completed and cancelled controls; sector evaluation counters summed to
  global counters. OS CPU sampling and rendering have separate owner probes.

No one-loop master or reduction implementation changed. Existing native master
providers, analytic-reference contracts, graph and algebra owners remain intact.
This milestone does not claim new numerical reference algorithms or automatic
threshold regularisation. Native/eager portability and the final workspace
checks are coordinated by the root reviewer; permanent test migration remains
explicitly deferred by the user.

Independent review by the parameter/kernel owner also passed a separate public
native probe (`output/runtime-dashboard/live-independent-probe.txt` and
`live_independent_probe.rs`): unequal 1/4/17-point MC prefixes around `1e14`,
correlated complex outputs and sparse implicit zeros matched a dense native
`StatisticsAccumulator<DoubleFloat>` control; reversing prefix order preserved
the estimate and exact offsets entered once. QMC common-one-shift previews were
mean-only, duplicate pending returns were rejected, reversed complete returns
matched accepted estimates, and negative full cross-covariance remained intact.
The reviewer found no blocking statistical or ownership issue.

The independent reviewer also accepted the final lazy-collection cadence
refactor: prefix cloning occurs only when due or before completed-wave slot
reuse, operational reductions occur behind the same gate, and cancellation
prefix discard remains separate from accepted result/replay admission.

## Cheap-kernel instrumentation and observation cost

After final release freeze, an additional bounded, sequential benchmark compared
`output/runtime-dashboard/fastsecdec-before` with the new release using explicit
`validated` routing. This isolates the instrumentation/driver change from the
new default distance policy. The existing cheap six-sector complex fixture,
seed 73, eight global production batches and deterministic 2,048-point pilot
were identical within each worker count. No production checkpoint or user result
was overwritten. Three short repetitions and two longer repetitions used rotated
variant order; all 30 trials matched the entire accepted mean/covariance and
every sector estimate exactly. Input and binary hashes were checked before and
after. The ignored script, individual reports/status streams, load averages,
wall/child-CPU measurements and summaries are in:

- `output/live-integration/overhead-benchmark.py`
- `output/live-integration/overhead-final-release/`
- `output/live-integration/overhead-final-release-long/`

The longer comparisons, where one-Hz status-JSON observations actually occurred,
gave these median driver elapsed times. These redirected runs did not render a
TTY dashboard:

| Workers | Production draws | Prior, quiet | New, quiet | New, 1 Hz |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 8,388,608 | 1.524 s | 2.611 s | 2.626 s |
| 8 | 67,108,864 | 2.566 s | 5.445 s | 4.996 s |

The work budget scales with worker count so both rows exercise observations;
these rows are not a fixed-work parallel scaling comparison. Quiet means a
one-hour interval with only forced boundary/final reports. The one-worker quiet
instrumentation increase was 71.3%, or approximately **0.130 microseconds of
additional process CPU per attempted point**. The eight-worker increase was
112.2% in driver elapsed and approximately **0.272 microseconds of additional
process CPU per point**. The shorter three-repetition controls showed 69.7% and
86.8% driver increases, respectively. This is a material percentage on a
purposefully inexpensive evaluator, not a claim about ggHH cost.

The longer one-worker one-Hz increment was 0.58% (two intermediate production
observations). The eight-worker runs had appreciable scheduling/host variation:
quiet times were 5.245–5.646 s, while one-Hz runs were 4.952–5.041 s and the prior
binary ranged 2.374–2.758 s. Their negative median one-Hz difference is **not**
evidence that observations improve performance; no small incremental refresh
cost is resolved by that sample. The short one-worker runs were below one
second, so their equal quiet/one-Hz times provide no observation-cost evidence.

Source review identifies potential future reductions in CLI bookkeeping:
`WorkerMeter::record` and native discrete point-time attribution each acquire an
uncontended worker-local mutex per point; operational global/sector diagnostic
merges coexist with the separate historical diagnostic accumulator, and final
complex maxima are updated by Laurent order. These operations are separate from
the native actual-call and wrapper clocks, which also have a cost. The benchmark
measures their combined effect; it does not assign the measured overhead to a
particular mutex or timer without a dedicated profile. Native moments and
operational aggregation on ordinary cancellation polls were removed before the
measurement. No further numerical or ownership rewrite was made merely to meet
an arbitrary percentage target. The library remains caller-driven and accepted
scientific results remain unchanged.

## Mouse capture and cached interaction follow-up

The CLI uses Crossterm 0.29's existing `EnableMouseCapture`,
`DisableMouseCapture` and typed `Event::Mouse` APIs. The owner implementation
emits normal/button/motion/RXVT/SGR capture modes and disables the inverse modes
in reverse order; FastSecDec adds no mouse escape parser. Capture ownership is
recorded before the compound terminal entry command, so partial entry failures
are eligible for cleanup. `enter_terminal` immediately restores on an entry
error. Capture disable is attempted independently before leaving the alternate
screen; a failed disable write cannot suppress the remaining cleanup attempts.
Existing stderr-before-mode lock order, draw active-state guard, raw-mode
ownership, atomics-only signal callbacks and native signal-thread cleanup remain
intact. The terminal constructor never enters capture in plain/JSON/non-TTY
modes.

`Dashboard::cancelled` forwards typed mouse events only to the cached integration
view, then redraws cached data if presentation state changed. It does not call
an integration observation, statistics reducer, sampler or evaluator. Worker
activity is now copied into `Cached` alongside its accepted/provisional
observation: interaction cannot combine newer in-flight counters with an older
accepted-work snapshot.

The ignored `output/mouse-terminal/terminal_mouse_probe.rs` and `mouse_pty.py`
probe the production terminal owner directly. Eleven PTY cases passed: normal
return, ordinary error, panic-hook cleanup, pre-existing raw-mode ownership,
plain/JSON capture exclusion, native SGR left-click/wheel decoding, second
keyboard interrupt, second SIGINT and SIGTERM while the caller is blocked, and
broken-pipe terminal-entry failure. Capturing cases emitted one enable and one
disable, with capture disabled before leaving the alternate screen; termios and
cursor state were restored. Forced-interrupt cleanup completed in 10–13 ms.
The host's standalone panic runtime subsequently aborted; capture/termios
restoration happened first, so this checks panic-hook cleanup rather than
continued execution after a caught panic. The broken-pipe case returned an
error and restored raw state. A read-only-stderr attempt was deliberately not
used as an error injection: Rust's native standard stream implementation treats
EBADF as a successful discarded write.

The independent kernel/parameter reviewer accepted the terminal ownership and
cached-event source changes.

### Independent renderer semantics and native layout audit

The renderer reuses native observations rather than reconstructing a sum or
covariance. A provisional view keeps its own scope and sampling source; an
unavailable provisional estimate does not silently fall back to an accepted
estimate from a different sampling population. Complex relative error uses the
norm of the component standard errors divided by the norm of the complex mean.
No displayed marginal errors are summed to invent a full-integral uncertainty.
QMC points and complete-shift counts come from the existing native observation.

The two progress gauges have separate denominators: accepted native work versus
current in-flight reservations. Completed but unadmitted work is never added to
accepted counts. The selected panel distinguishes invocation assessments,
accepted current-phase points, and estimator coverage. Review caught an initial
use of `SectorContribution::used_points` for the accepted label, which could
show zero accepted pilot work or incomplete-lattice packages. This was corrected
to `SectorSnapshot::completed_points`; `View pts` still reports only the points
used by the displayed estimate.

Timing rows separate wall time, process CPU, aggregate active worker elapsed and
active coordinator elapsed. Selected-sector exclusive shares use that sector's
worker elapsed denominator; global shares add the measured coordinator spans.
Inclusive integrand time contains pure evaluator time. If a cached in-flight
sample observes temporarily incomplete timing attribution, the display says
pending instead of manufacturing negative overhead or a share above 100%.
Average f64 evaluator time divides actual f64 call time by actual calls, while
mean time per assessment divides invocation worker elapsed by assessments.
Final precision-class fractions are distinct from attempted evaluator-call
counts, and maximum contribution is the recorded finite complex weighted
coefficient magnitude for the selected Laurent order.

The pinned ratatui-widgets 0.3.2 `Table::get_column_widths`, `selection_width` and `visible_rows`
were inspected against the hit-region code. The renderer uses the same native
Layout constraints, spacing, flex and two-cell selection reserve. Row regions
are built after `render_stateful_widget` updates `TableState::offset`, preserving
native scrolling and variable-height wrapped rows. Compact second-line headers
take precedence over their wider spanning header rectangles. Sorting preserves
stable sector IDs and deterministic ID tie breaking, with unavailable metrics
last in both directions. Mouse selection cannot change scientific state.

Initial native TestBackend captures at 120×30 and 80×24 exposed missing
preview/accepted column labels in the short-height header and several clipped
diagnostic labels. The revised 80×24 capture labels the two value columns in the
title and uses unambiguous diagnostic labels. The renderer owner's native
TestBackend interaction probe covers header direction toggling, compact
second-line headers, stable row selection after scrolling, wheel scope,
signed/missing-last sorting, exponent-column alignment, NO_COLOR, oversized
values and pending timing attribution. Its source and output were independently
inspected in `output/dashboard-polish/interaction_probe.rs` and
`render-probe.txt`; source-level native layout comparison provides an additional
check independent of the probe's hit-region assertions.

The four-significant-digit count formatter's native Rust probe was independently
compiled and executed. Exact counts below 1,000, K/M/B boundaries including
rounded promotion, and `u64::MAX` all passed; integer widening prevents rounding
overflow or a floating-point loss of count digits. Raw values, never formatted
strings, drive sorting. Review identified that the existing point-count sort
still converted `u64` to `f64`, potentially tying adjacent counts above 2⁵³;
the renderer owner replaced this with direct integer comparison. The revised
source and passing reversed-ID control for 2⁵³ and 2⁵³+1 were independently
inspected in both ascending and descending directions. The selected-panel,
progress and timing semantics and native mouse hit-region slice are accepted.
No numerical driver was changed for this work.
