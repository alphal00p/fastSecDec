# Aggregate dynamic runtime diagnostics proposal

2026-10-10. Design only; registered source remains frozen for the current
milestone. Numerical acceptance, sampling state and mathematical artifact IDs
must remain independent of these optional observations.

## Existing native data and proof

Both native envelope constructions have

\[
a_2=1+L^2\|v\|^2/R^2,\qquad
D/R=\lambda\|v\|/R=(\lambda/L)\sqrt{a_2-1},\qquad D=R(D/R).
\]

`DynamicEnvelope::base_level_from_norm` supplies the sole quadratic term.
Polynomial causal terms start at power four, as do the squared sign-aware
envelopes and the positive-factor contributions. Face restriction preserves
this identity in the original full source dimension. The callback already
receives `a2` and `L` and computes `lambda`; physical displacement additionally
requires the bound `R`, available to the owner when draining its report.
No gradient, saved raw program or certified primitive needs reevaluation.

The ignored Rust probe `target/contour-diagnostics-a2-probe.rs` uses the public
native envelope/Atom APIs and proves the coefficient and displacement-square
identities in both constructions for zero, one and two dimensions, through
degree five with nonconstant positive factors. Its 24 full/face controls pass
on owner `516beb37`; the adjacent log records them. It also demonstrates that
`1 + 1e-18` rounds to one in f64 despite nonzero displacement. Therefore `a2=1`
must report **zero or below diagnostic resolution**, never a certified zero.

Symbolica's public `BracketedRoot` already returns iterations, evaluations and
termination. The current callback discards these after taking `root`. The
existing native budget probe records, for example, 3300 iterations and 3303
evaluations at 3322 bits; no new solver instrumentation is needed for successful
refinement. Errors expose no partial iteration count. Preserve that absence;
do not infer iterations from evaluations or report the work limit as work done.
The structural quadratic solution has zero refinement iterations and one native
H/H' evaluation for its existing complex correction. Generic refinement also
performs that additional correction evaluation after the solver's own count.

## Proposed bounded report

Use a separate operational `ContourDiagnosticsMode::{Disabled, Aggregate}`,
independent of Always/Pilot/Off validation. Proposed low-level default is
Disabled; a caller requesting runtime statistics explicitly selects Aggregate.
The CLI default is a presentation/performance decision to settle at handoff.
Neither setting changes saved expressions, tags, content ID or RNG state.

| Quantity | Meaning and source |
| --- | --- |
| Callback calls / failed calls | Actual executed root callbacks, including repeated native calls, faces, discarded batches and precision retries. No observation deduplication. |
| Solver calls / closed-form calls | Iterative refinement versus structural quadratic evaluation. |
| Successful solver iterations / evaluations / maxima | Native result fields; failed solver calls are separately counted and their unavailable partial work remains unknown. |
| Correction evaluations | Existing post-root H/H' evaluation, counted separately from solver work. |
| Strength and normalized displacement ranges | Finite approximate f64 centre extrema with contributing count and unavailable/rounded counts. Physical displacement scales the normalized range by bound R at reporting. |
| Accepted points / rescues / precision class | Reuse `PrecisionReport`, `ReplayReport` and caller-owned `EvaluationDiagnostics`; never derive these from callback counts. |

Keep preparation, exact binding, independent contour pilot and conditioning
separate from production evaluation. Caller `IntegrationStage` already separates
adaptation from production. Attempt ranges describe executed work, not an
unbiased distribution over accepted sampling points. In particular, a failed
native matrix remains in work counts while all its provisional rows are
discarded and scalar rescue proceeds. Do not switch unchecked batches to scalar
execution merely to obtain diagnostics.

Initially retain counts and extrema, not a new moment/covariance accumulator.
Numerica's `StatisticsAccumulator` was inspected: it manages integration sums,
squared sums and iteration weights, with unchecked internal sample counters.
That operation is unnecessary for bounded display ranges and would add overflow
and statistical interpretation concerns. Existing full Laurent covariance is
untouched.

## Ownership and API boundary

Select a plain or aggregate callback factory during existing native mapping;
retain that choice in `MappingRequirements` for precision/conditioning remaps.
Extend the existing dynamic failure-attempt RAII scope with an optional bounded
aggregate, rather than adding a candidate observer or a second global registry.
The callback contributes one compact event, then the numerical owner merges the
attempt delta. Nested scopes restore their parent; cloned numerical owners start
fresh aggregates. Exact direct-vector evaluation uses the same attempt boundary
and native lazy cache, so untaken/canceled roots contribute no calls.

Proposed public additions are `KernelSet::set_contour_diagnostics(mode)` and a
fallible `SectorKernel::take_contour_runtime_report()` alongside the existing
validation-report drain. Mode changes use the existing atomic saved-IR remapping
path, preserving prior counters and avoiding symbolic rebuilds. Integrate the
new optional snapshot with caller report/checkpoint aggregation at existing
work/update boundaries. Historical absence means unknown, not zero. Reporting
DTOs containing approximate f64 ranges should use PartialEq, not claim Eq.

Use checked integer addition and the existing `status::DiagnosticsOverflow`
boundary. A sticky diagnostic-overflow state is returned when draining; it must
not set the callback's numerical-failure slot, return NaN, alter accepted values
or masquerade as a scientific uncertainty. Merge/drain admission is atomic.
Native solver failures retain their existing scientific failure handling.

## Cost and implementation handoff

Disabled factories must compile out the new event path: no extra TLS access,
centre conversion, range arithmetic, root solve, observer or ball workspace.
Aggregate factories add a bounded counter update through the existing attempt
state and, for ranges, three centre conversions plus f64 subtraction, division,
square root and multiplication per successful callback. These are an operation
budget, **not a measured overhead claim**. No per-root clock, per-point map,
retained point vector or diagnostic multiprecision calculation is proposed.

If a centre conversion overflows/underflows, `a2<=1`, or the display calculation
loses a positive quantity, record an unavailable/rounded range contribution.
Do not clamp, insert a floor, escalate precision for display, or reevaluate a
checker. The ranges are explicitly approximate; certificate bounds remain in
the separate validation report. Physical scaling by R receives the same finite
display admission at the reporting boundary.

After milestone handoff, runtime ownership covers callback events, existing
attempt scopes, evaluator/precision aggregation and atomic mode remapping;
status/CLI ownership covers serialization, phase aggregation and presentation;
generation ownership covers exact-vector attempt propagation. No generation
mathematics or artifact wire change is required.

Before acceptance: test counts across eager/SymJIT, native/portable, scalar and
matrix attempts, retries, lazy exact branches, cloning and nesting; inject
counter overflow independently of numeric failure. Verify disabled mode performs
zero diagnostic events and Aggregate leaves all values/identities unchanged.
Use the existing paired variance harness to measure Disabled versus Aggregate
separately under Off/Pilot/Always, including quadratic and iterative roots.
Implementation and those timing gates remain pending.
