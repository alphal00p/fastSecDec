# Runtime dashboard presentation audit and evidence

Dashboard implementation and focused probe record, 2026-10-07. The dashboard
agent authored the presentation changes; the kernel/parameters agent performed
an independent source review of their statistical and timing interpretation.
Production code was frozen before the documentation review recorded below.

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
table fits 120 columns; narrower displays use multiline sector rows. Extra
per-sector precision fractions appear on wide screens. The full or explicitly
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
