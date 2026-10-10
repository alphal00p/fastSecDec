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

## Implemented native ownership and measured cost (2026-10-10)

The next implementation slice supplies the public `ContourDiagnosticsMode`
(`Disabled` by default, or `Aggregate`) and `ContourRuntimeReport`, independently
of validation policy. `KernelSet::set_contour_diagnostics` prepares replacement
numeric factories from the saved native IR before changing the live owner.
`KernelSet`, detached `SectorKernel`, and `WeightedEvaluationContext` expose
snapshot and atomic drain methods. No mathematical ID, saved artifact or replay
state contains the optional mode. Python/status presentation is a separate
consumer of these native DTOs.

A separate optional attempt scope carries bounded callback events. Disabled
factories construct no event and perform no diagnostic TLS access, centre
conversion, range arithmetic or certificate work. Aggregate factories reuse the
native successful solver result's iteration/evaluation/termination fields;
structural quadratic roots count their closed-form operation separately. The
existing native complex correction is counted once. Failed solver partial work
remains unknown, while the failed callback and solver invocation are counted.
The numerical failure fence and candidate certificate scope are unchanged.

The report separates evaluation, conditioning, preparation, exact binding and
pilot work. A provisional matrix that fails contributes its actual work before
whole-vector rescue; retries are additional work, not additional accepted
samples. Native direct-vector exact evaluation remains lazy and shares executed
function values across coefficients. Clones start fresh observation histories.
Changing factory modes preserves prior histories, including work before a
failed atomic numerical rebind. Pilot-only owners transfer observations before
being released. Sticky checked-counter overflow prevents a report drain, without
changing the numerical value or failure slot.

Ranges describe successful callback returns before later evaluator/certificate
acceptance; they are not statistics of accepted integration points. Physical
displacement is scaled by the bound R when each event is accumulated, so a later
rebind cannot reinterpret earlier history. A rounded `a2=1`, failed display
conversion, or nonpositive/overflowed display result contributes `unavailable`,
never a fabricated zero. Callback failures do not contribute a successful-return
range observation.

A bounded ignored probe uses the public generation, binding, mode-remapping and
sector-evaluation APIs on `F=1-2x` and the genuine cubic
`F=(1-2x)(1+y)(1+z)`, each with the complete `(1/eps) F^(-eps)` vector. It uses
polynomial construction, S=0.8, L=R=1, validation Off, and exactly the same 32,768
deterministic interior coordinates for five alternating Disabled/Aggregate
pairs per backend. Every pair retains identical output bits, content ID and
saved bytes. Disabled reports no callbacks; Aggregate records exactly 32,768.
Each cubic run reuses 182,259 native iterations and 280,563 native solver
evaluations, with no second solve.

These are **development-core measurements**, with dependency optimization at
level 2 and SymJIT's native generated code; the Rust probe itself uses `-O`.
They measure optional observation cost in this executable, not release overhead
or a comparison with a pre-feature binary. Other build work was present.

| Control | Backend | Disabled median seconds | Aggregate median seconds | Median paired ratio | Paired ratio range |
|---|---|---:|---:|---:|---:|
| Linear, structural quadratic root | Eager | 0.115701 | 0.159815 | 1.381 | 1.380–1.392 |
| Linear, structural quadratic root | SymJIT | 0.069888 | 0.113074 | 1.624 | 1.612–1.631 |
| Cubic, iterative root | Eager | 0.667475 | 0.714472 | 1.070 | 1.068–1.077 |
| Cubic, iterative root | SymJIT | 0.233846 | 0.276000 | 1.179 | 1.178–1.182 |

The 40-run probe completed in 12.25 seconds with peak RSS 18,432 KiB, under a
180-second/3-GiB cap. Ignored evidence is
`target/contour-diagnostics-overhead{.rs,.log,-summary.json,-result.json}`.
Source SHA-256 is
`46c9ffe280be64c4dbb745d5b8e9362a28414302a1f29fd116fba3dabb9db071`;
run-log SHA-256 is
`ede98bd32ad8cb4d80ce5b79c4eec21e2e2cd9dba220fac9cce5ffed86a1e99e`.
Native owner revision is `516beb37`, SymJIT `d74993ff`, FastSecDec source is
`184803d` plus this diagnostics slice. Focused native, portable-host and actual
WASM regression results follow.

The native focused `diagnostics` gate passed **11/11** in 0.13 seconds (nine new
controls plus two existing status controls), in
`target/contour-full-family-diagnostics-tests.log`. It covers real native solver
termination/work, tracked complex uncertainty, rounded displacement,
nested/unwound scopes, independently injected counter overflow, Eager/SymJIT
mode transitions and failed atomic rebinds, clone/drain ownership, actual pilot
and policy remapping, exact shared-cache/lazy-IF execution and multiprecision
rescue, and discarded mixed matrix work followed by a healthy batch. The
portable-specific public cubic control remains a separate acceptance gate.

The standalone portable-host public diagnostics control also passed **1/1**
(0.44 seconds), with both polynomial and sign-aware cubic recipes. It performs
actual pilot validation, changes to unchecked production, then compares
Disabled/Aggregate complete vectors and identities, iterative native work,
R=2 physical versus normalized displacement, and clone/drain ownership. This
is recorded in `target/contour-portable-family-diagnostics-tests.log` and is
separate from actual WASM execution.

The **same unchanged public diagnostics test passed on actual WASM: 1/1 in
0.38 seconds**, using the existing isolated Emscripten build and runtime. The
execution log is `target/contour-diagnostics-wasm-tests.log`. This verifies the
portable callback/report mode on that runtime; it does not establish HEPKit
browser binding or family storage acceptance.
