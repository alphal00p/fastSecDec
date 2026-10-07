# Runtime dashboard presentation audit and evidence

Dashboard implementation and focused probe record, 2026-10-07. The dashboard
agent authored the presentation changes; the kernel/parameters agent performed
an independent source review of their statistical and timing interpretation.
Production code was frozen before the documentation review recorded below.

## Structured panel follow-up (2026-10-07)

The subsequent visual revision uses native Ratatui rounded blocks, gauges,
Tables, Lines and styled cells for the full/selected sum, memory, separate
accepted and in-flight progress, sector viewport, selected-sector details and
invocation diagnostics. Workers are captured with the same immutable observation
as accepted/live estimates; cached mouse and keyboard redraws do not sample or
refresh statistics. The two progress denominators remain separate, avoiding
admission double-counting. QMC retains explicit full-sum used-point and
complete-shift coverage.

The detail inset uses the selected stable sector ID. Assessed points describe
this invocation; accepted points are native current-phase completed work, while
view coverage is the estimator's used/preview coverage. Final outcome fractions
and attempted evaluator calls are distinct. Sector exclusive timing excludes
unassigned coordinator work. If a transient snapshot observes callback timing
before worker attribution, the display reports pending attribution rather than
fractions above 100 percent.

All integration-dashboard timings now use the shared ordinary decimal
`sample_duration` helper with dynamic µs, ms and s. Dashboard counters use the
shared four-significant-digit base-1000 K/M/B formatter. IDs and orders are never
abbreviated, and point sorting compares native u64 values directly, including
values above 2^53. Raw numerical observations remain unchanged.

Native Layout constraints, column spacing, the two-column selection marker and
post-render TableState offsets also define the cached mouse hit regions. Clicks
sort headers in both directions or select rows; wheel events only affect the
sector body. Compact two-line headers retain separate relative-error, f64-mean
and maximum-contribution click targets. Mantissas align right and exponent
cells align left, fixing the multiplication dot across exponent widths. The
maximum weighted coefficient has the same alignment. Exponents reserve seven
columns, including −324. Values that exceed their available native cell width
show an explicit ellipsis instead of a misleading truncated numeric token.

Focused evidence is retained only in ignored `output/dashboard-polish`:

- Native TestBackend captures at 160×48, 120×42, 120×30, 80×32, 80×24,
  65×26 and 40×20; text and per-cell color HTML exports. The 80×24 view retains
  two sector rows, RAM, the progress gauges, selected detail and global classes.
- Million-scale compact capture: 1.049 M assessed, 16.78 M accepted and a
  503.3 M / 1.007 B progress allocation remain readable.
- Mouse direction toggles, compact secondary headers, row selection after
  scrolling, stable IDs, wheel scope and missing-last signed sorting passed.
- Native buffers verified common dot positions for mixed real, imaginary and
  maximum-contribution exponents 0, −1, −10 and 12; real/imaginary include −324
  and differing uncertainty widths.
- NO_COLOR produced default foreground/background cells. Overflow markers,
  pending timing attribution and exact point sorting above 2^53 passed.
- A focused QMC capture retained full-sum points/complete-shift coverage.

The independent review is recorded in
[integration-dashboard-polish-review.md](integration-dashboard-polish-review.md).
The source probe and copied source harness were moved to ignored output after
execution. No permanent tests or other examples were modified. The original
controls below remain historical evidence; the dashboard duration and layout
contracts in this follow-up supersede their earlier presentation choices.

## Ownership and reuse

The CLI consumes native `IntegrationObservation`, `LiveObservation` and
`OperationalMetrics`. Accepted totals and covariance come from the native
session. Provisional estimates come from the integration owner's separate live
observation API. The dashboard neither derives an accepted total from rounded
sector means nor combines marginal errors to reconstruct total covariance.

Ratatui's `Table` and `TableState` own selection and viewport scrolling. Tabled's
`Width::wrap(...).keep_words(true)` owns Unicode-aware cell wrapping. Spenso's
public `utils::to_superscript` owns exponent glyph conversion; its existing CLI
development dependency was promoted to an ordinary dependency without adding a
package or changing the lockfile. These are CLI dependencies only.

Numerica's public `StatisticsAccumulator::format_uncertainty` handles ordinary
uncertainty ratios. A focused probe showed that its order-one and large-error
conventions differ from the requested last-digit parentheses. A presentation
adapter therefore uses Rust's native float formatting for those ratios, decade
carries and extreme finite exponent spans. It introduces no arithmetic backend
or symbolic algebra. Parentheses always contain integer last-digit uncertainty;
normalization, an explicit outer exponent and two significant uncertainty digits
are preserved. Missing uncertainty remains explicitly unavailable.

Sysinfo 0.39.6 supplies current-process `accumulated_cpu_time` and memory readings.
CPU is measured against a fresh invocation baseline, includes all process
threads, and is converted from native milliseconds to seconds. The monitor does
not sum per-thread RSS or run shell processes. Normal sampling is bounded and
only invoked when an observation is published; cached integration redraws reuse
the saved reading. Initial/final observations can refresh immediately.

## Presentation contract

One configured cadence defaults to 1000 ms for terminal, plain and status-JSON
observations. Caller-side gates precede expensive native observations; worker
prefix publication uses the same configured interval. Cancellation retains its
independent approximately 50 ms coordinator polling. Initial, boundary and final
observations can appear immediately. JSON also records `elapsed_seconds`.

Up/Down, PageUp/PageDown and Home/End select a sector. Left/Right select the
Laurent order; the initial order is the requested accuracy target or the highest
available order. Tab or `s` cycles sort columns, Shift-Tab goes backwards, and
`r` reverses direction. Selection persists by sector ID through sorting and
refreshes. Real and imaginary columns sort signed central values; missing data
sort last in both directions, with sector IDs breaking ties. Keyboard actions
redraw cached observations without recomputing estimates.

The table shows points, real/imaginary estimates and uncertainties, relative
complex error, mean primary-f64 evaluator duration, and the maximum final
importance-weighted coefficient magnitude for the selected order. The relative
error is `hypot(error_re,error_im)/hypot(mean_re,mean_im)` and is unavailable for a
zero denominator. Observed maxima have no uncertainty annotation. A comparative
table fits 120 columns; narrower displays use multiline sector rows. Per-sector precision fractions appear in the selected detail inset. The full or explicitly
selected sum remains above the table.

QMC rows display used points and complete-shift coverage. Sector preview coverage
may differ from full-sum coverage. The accepted full sum counts points from the
native contribution report's used rows, rather than all completed points that
may lie outside common-shift coverage. One complete lattice can provide a mean
without an error. Numerical-range failure, unsampled data and missing uncertainty
are distinct from zero. Live sources are explicitly provisional; accepted
statistics remain separately identified.

Final precision fractions classify each assessed point once, with evaluations
as denominator. Unstable includes failures and configured cutoff zeros exactly
once; their counts remain separate. Historical missing classes remain unknown.
The arbitrary class is labeled `Arb<1000>` for distance routing and `Arb
(variable)` for validated routing. The abbreviated per-sector heading is `Arb`.
The maximum bit-depth counter is labeled **Maximum successful-point precision**:
failed evaluator attempts retain timing but need not produce a precision report.

Operational percentages use measured work elapsed, defined as aggregate worker
elapsed plus active coordinator integrand and integrator spans. The exclusive
parts are:

- evaluator: native evaluator-call elapsed;
- integrand: inclusive worker integrand elapsed minus evaluator elapsed, plus
  coordinator integrand preparation;
- integrator: worker elapsed minus inclusive integrand elapsed, plus active
  coordinator integrator bookkeeping.

Coordinator waiting is excluded. Wall time, summed worker elapsed and actual
process CPU are separate quantities; no CPU difference is used to infer an
unmeasured overhead. Operational data include pilots and discarded prefixes.
Primary-f64 mean time excludes conditioning evaluators and higher-precision
calls, while including f64 attempts subsequently superseded by escalation. The
slowest sector is selected by this mean, with a stable ID tie break.

## Focused controls

Temporary sources and raw results are retained only under ignored
`output/runtime-dashboard`; no permanent test or other-example migration was
performed. Temporary CLI example sources were removed after execution.

| Control | Observed result |
| --- | --- |
| Strict CLI Clippy | `cargo clippy -p fastsecdec-cli --bin fastsecdec -- -D warnings` passed |
| Native TestBackend rendering | Passed 40, 64, 80, 120, 125, 140 and 180 columns at 30 rows; separate QMC coverage views |
| Table state | Signed sorting, missing-last in both directions, persistent selected ID, order changes and empty-sector views passed |
| Uncertainty availability | Mean-only and numerical-range controls retained explicit unavailable states |
| Requested notation | `0.1234 ± 0.0056` → `+1.234(56)·10⁻¹`; `1.234 ± 5.6` → `+1.2(56)·10⁰`; dominating error → `+1(120000)·10⁻¹` |
| Extreme finite formatting | Subnormal and maximum-f64 combinations retained integer parentheses and one explicit outer exponent without scaling overflow |
| Duration formatting | Seconds, minutes, hours and days passed; sub-centisecond values retain scientific seconds rather than rounding to zero |
| Process CPU | Two busy threads over approximately 0.4 wall seconds produced 0.637–0.800 CPU seconds across runs; a subsequent 0.4-second sleep added 0–0.001 CPU seconds |
| Default cadence | Plain and JSON each emitted three snapshots over 2.3 seconds while the probe continued 162–165 input polls |
| Cached PTY interaction | Tab/reverse redraws did not request another snapshot before cancellation |
| PTY cleanup | At 40/80/140 columns, cancellation completed in approximately 20/21/13 ms and restored termios, cursor and normal screen |

The isolated PTY probe polls input every 10 ms and contains no expensive native
point evaluation. Its latency is not a bound on the full integration driver,
which must wait for the current native evaluation or setup to return. The
existing terminal lifecycle review covers interruption escalation and panic-hook
boundaries; SIGKILL cannot execute cleanup. Extremely long finite mantissas obey
the formatter contract but require additional display rows; the layout probes
used ordinary numerical vectors and do not establish visibility of every such
mantissa in a short terminal. These probes are not convergence or performance
benchmarks.

## Independent review and documentation checks

The kernel/parameters agent independently accepted final-class fractions and
unknown-history handling, primary-f64 means, the exclusive worker/coordinator
partition, current-PID CPU baseline, and signed stable sorting with missing-last.
Its finding about the successful-point precision counter was corrected before
freeze.

The dashboard agent separately reviewed the numerical owners in
[runtime stability review](runtime-stability-review.md) and
[endpoint provenance review](runtime-endpoint-profiles.md). The weighted-state
merge finding from that review was fixed and covered by the owner's
release-linked native probe: merging maximum 50 into a context with maximum 100
keeps 100, and a subsequent value 60 remains f64 under the 0.9 rule. The existing
[terminal lifecycle review](discrete-mc-terminal.md) records the earlier
restore/draw synchronization and scoped signal-listener controls.

Checked the root README, ggHH README, runtime integration guide and the related
CLI guide against final argument routing and presentation. The ggHH eight-worker
1% Havana and 0.1% QMC commands retain explicit order-zero targets, zero absolute
tolerance, bounded rounds and work-limit caveats. Runtime-overlay precedence,
preview versus accepted estimates, cadence, keyboard controls, covariance and
cutoff-bias language agree with the implementation. Two runtime-guide details
were corrected: only the first two numerical levels accept power-specific
thresholds, and maximum precision refers to successful points. The root README's
representation-ID paragraph was narrowed to its actual sector-mapping/statistics
contract. `inspect RUN.toml` remains supported input inspection and was retained;
rich `--sector` inspection requires an artifact basename.

## Final report label-width follow-up

Independently reviewed the final-report-only helper change after the initial
freeze. `facts_table` retains the generation report's original
`min(14, width/3)` label budget. The new helper takes a caller label budget,
clamps it to at most half the available two-column width, and enters the
existing stacked layout before subtracting border/padding space at widths below
32. The first integration-report revision requested 34 label cells. JSON routing
and numerical content are unchanged.

An ignored native probe compared the exact previous helper against the wrapper
for every width from 1 through 140, both plain and colored: generation output
was byte-identical. The probe explicitly enabled colors independently of the
parent shell's NO_COLOR setting. The integration helper also rendered all these
widths without arithmetic underflow. Three representative long diagnostic labels
at 80 columns used six output lines instead of eleven, preserving the full text.
The probe and output are retained under `output/runtime-dashboard/final-labels*`.

A final source review accepted two presentation corrections from the no-work
resume check. A zero successful-point bit-depth counter now displays
`unavailable`, which correctly covers invocations with no successful numerical
evaluation, including no-work resumes; it does not invent a zero-bit backend.
Nonzero precision values, classifications and all accepted results are unchanged.
The integration label request is now 36 cells. At width 80 the helper allocates
36 label cells and 37 value cells plus seven border/padding cells. The review
identified that `Final DoubleFloat (106 bits) fraction` is 37 characters and
therefore wraps onto two ordinary word-boundary lines at 80 columns. This is an
accepted, nonblocking presentation choice, substantially less fragmented than
the former 14-cell label budget; a single-line fit is not claimed. The half-width
clamp and below-32 stacked guard remain safe, and generation retains its original
wrapper budget. No production edits were made by the reviewer for these
corrections.

## Accepted reference and memory clarification (2026-10-07)

The compact maximum label is now `Max |wgt|`, following the user's wording.
Its existing native meaning is unchanged: per selected sector and Laurent
order, the largest magnitude of the final importance-weighted coefficient seen
in this invocation, including pilot/discarded work and excluding exact offsets.
The underlying meter receives the successful weighted output and uses `hypot`
over the real/imaginary components for that order; it is not the maximum bare
sampling weight or a statistical error.

Missing current accepted totals now distinguish pilot work, incomplete batches,
incomplete shifts and statistical failure. The CLI retains a display-only clone
of the last native completed production allocation. A current accepted total,
including a partial allocation of complete statistical units, always takes
precedence. Historical values are explicitly headed **Previous allocation** and
identified as completed in the status caption; they never become current totals,
JSON statistics, checkpoint content or inputs to stopping decisions. Cache
compatibility requires matching method, scope, sector IDs, Laurent orders and
components. Starting another integration clears the cache. Statistical/range
failure prevents historical fallback, and exact-only inputs cannot inherit a
stochastic reference. Existing forced completion observations capture completed
rounds before the next pilot begins without changing the one-second cadence.

The integration memory cards now share the monitor's `free_or_available()`
policy: macOS labels the native free-pages reading **Free**; other platforms
retain **Available**. This avoids implying that macOS's broader VM-reclaimable
counter is unused RAM. Native JSON retains the original available counter and
adds a separate free counter; used memory is not inferred by subtraction.

Focused native probes in ignored `output/dashboard-accepted` passed complete-only
retention, current partial precedence, compatibility invalidation, statistical
and numerical-range failure handling, unchanged numerical observations and
80/120/160-column rendering of each waiting/reference state. They also repeat
the native mouse, exact-count sorting, alignment and no-color controls. The
memory fixture distinguishes 488.3 MiB free from approximately 14 GiB broadly
available. Probe sources were removed from the CLI examples directory after
execution; no permanent tests or numerical code were changed.

The final-release two-round ggHH control also passed in plain output and a
120-by-32 native PTY. The previous completed allocation remained visible during
the next pilot with its historical label, current waiting reasons and macOS
Free counter. Terminal attributes, alternate screen and cursor were restored.
Plain and PTY results had identical final means, errors and full covariance,
with 1024 accepted production points and zero failures. See
[the final-release CLI review](evaluator-settings-cli-review.md) and ignored
`output/dashboard-accepted/multiround-*` for the bounded evidence.
