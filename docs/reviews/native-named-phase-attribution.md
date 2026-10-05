# Attribute the bounded named-coefficient timeout

The first actual named generation timed out at 180 seconds with only a generic
composition marker. It supplies no complete-vector acceptance and does not
distinguish expensive native series from formal assembly or physical derivative
body resolution. This next proposal changes diagnostic visibility only.

The disconnected draft is in `output/probes/named_telemetry/`: `prepare.py`
produces instrumented copies of the existing `series_first.rs` and `named.rs`
after exact source-anchor checks. `instrumentation.diff` and `baseline.json`
retain the changes and source hashes. `trace.rs` is a scoped test-only JSON-lines
recorder, and `adapter.rs` is a temporary cfg(test) entry point. No tracked
production source has been changed by this preparation.

## Boundaries to measure

Keep every native mathematical call, iteration order, cache and truncation guard.
Record timestamped begin/end events with parent span IDs and small metadata:

* Each composition attempt, requested native width and returned absolute bound.
* Regular-factor native series separately from coefficient naming and its exact
  restoration control. Native series cache misses record input bytes and the
  resulting native bound/term count; cache hits do not generate per-call logs.
* Each endpoint axis, with incoming/outgoing piece count. This includes native
  formal derivatives, face substitutions and endpoint scalar operations.
* The **per-piece coordinate-weight series products**, separately from each
  **incremental prefactor-group series addition**. The latter records the prior
  group's byte size without printing its expression. These distinguish repeated
  scalar convolutions from additions to an increasingly large native Atom.
* Grouped prefactor native series, its series multiplication, and final group
  sums as separate operations; final coefficient extraction retains its bound.
* All-order lowering, each coefficient's lowering, and cache-miss request
  resolution. Native partial construction and native literal face substitution
  have separate spans, source-name/multiindex metadata and input/output sizes.
  The existing native derivative and face caches are unchanged.

Intervals are nested and therefore inclusive; a report must not add a parent
and its children as independent CPU time. An unmatched begin event at timeout
identifies the active call; a long phase is evidence of where time was spent,
not proof of a particular native algorithm or allocator defect. Trace writes
are flushed so a timeout preserves the last boundary. No full expressions,
coefficient expansion, native graph rebuilding or sampling are added.
Span durations include metadata construction and trace-write overhead. A
`session_complete` record means the trace closed normally, including when the
wrapped native operation returned `Err`; it is not a scientific-success flag.
Only the test/process result and required complete-vector evidence decide that.

The recorder is installed explicitly for one thread and reset by a guard even
on unwinding. It carries no Atom, callback or evaluator state. Existing normal
tests have tracing disabled. Two focused controls check reset after a deliberate
panic and exact equality of all three Gamma-regulated coefficients, aliases,
native widths, bounds and piece counts with tracing on versus off. The existing
eight small composition controls also rerun in the instrumented build. The
native program/weighted/cold-IR small controls remain previously accepted; no
program or precision implementation changes are proposed.

## Source comparison motivating the boundaries

The active reference `_g_coefficients_by_symbolic_diff` constructs only requested
regular epsilon coefficients before `_local_taylor_coefficient_expr` computes
their coordinate derivatives. `_two_stage_derivative_fused_components` groups
requests by boundary/zero tuples, forms residual/local polynomial inputs for
each group and caches the resulting expressions. Its surrounding implementation
also contains explicit custom series/log/factorial/chain-rule helpers; none is
proposed for transfer. The old dual-evaluator wording in that function's
docstring is not evidence of the selected final evaluator route.

The current native experiment instead keeps native `Series<AtomField>` through
each endpoint operation, then performs up to nine successive coordinate-factor
series multiplications per surviving piece and incrementally adds each result
into a common-prefactor group. It resolves unique whole native derivatives
before applying their literal faces. The inspected native `AtomField` ring
multiplication preserves the ordinary Atom product rather than calling a blanket
`.expand()`. None of these source observations identifies the dominant measured
phase yet. The trace deliberately separates them before changing this ordering.

If scalar coordinate products dominate, a later native-API investigation can
examine compact scalar weight composition with certified positive coordinates,
while keeping native Series responsible for all coefficients and bounds. If
lowering dominates, examine actual native partial/face requests and cache reuse.
These are conditional questions, not approved optimizations or custom algebra.

## Proposed bounded execution

After independent source review, temporarily install the diagnostic copies,
compile/freeze the exact test binary and source/native archives, then restore
all tracked files byte-for-byte. First run the ten small controls under the
existing 180-second/five-second-grace, 30-GiB address-space, CPU8 bound. Only after
they pass, freeze a fresh generation-only attempt with the original capture80
inputs and the same separate 180-second bound. The sole changed question is
which phase exhausts it; there is no automatic extension or following program,
oracle or whole-graph run. No Symbolica overlap is allowed.

The recorder adds diagnostic I/O and timing overhead, so results cannot be used
as a speedup benchmark. Preserve both the prior uninstrumented timeout and every
partial trace. Even if generation finishes, mathematical acceptance still needs
the independently frozen original-expression oracle records and all-order
checks from the actual-target protocol.

## Instrumented small gate

The frozen development/test build in
`output/diagnostics/native-named-telemetry-build-1` compiled successfully in
12.08 seconds. Its exact compiled source and native dependency archives are
retained; all four temporarily changed tracked files were restored byte for
byte. The copied probe SHA-256 is
`b47042b46f3dc34d231c0e82bc4de7ac6352ae2631b170c7204b29771529601c`.

`output/diagnostics/native-named-telemetry-small-1` passed all ten normal
controls, with two existing replay tests ignored, exit zero and no timeout.
All 21 frozen hashes passed before and after the process. Wall time was
0.171154873 seconds and peak native child RSS was 15,360 KiB. The intentional
caught panic in stderr belongs to the successful unwind/reset control.
These results validate the declared small telemetry layer; they do not certify
actual representative coefficients or establish a generation speedup.

## Actual traced representative: bounded negative result

The independently reviewed freeze is
`output/diagnostics/native-named-actual-generate-trace-1`. All 103 hashes passed
before and after execution. It binds the same captured representative 80,
source chart 119, original mapped terms and completed Taylor-872 identity as
the preceding uninstrumented attempt, together with the exact instrumented
build and successful small-control records. Multiplicity four was not applied.
The wrapper and process were reaped with status 124 after the unchanged
180-second limit: SIGINT, wall 180.111590142 seconds, user CPU 168.541128 seconds,
system CPU 10.476930 seconds, peak child RSS 1,736,912 KiB. The 30-GiB limit was
an address-space limit. Guarded external compilation on other CPUs could
overlap this diagnostic; there was no other Symbolica process or timing claim.

Only `progress.json` and `phases.jsonl` were produced. There is no completed
coefficient export, program, numerical oracle agreement or integral. The trace
has 13,175 complete JSON events, no partial final line, and no session-complete
marker. Its parsed terminal summary is retained as `trace-summary.json`; the
pure-data summarizer is `output/probes/summarize_named_phase_trace.py`.

The native composition completed its two attempts before request lowering.
Width one produced absolute bound minus five in 2.402 seconds. Width seven
produced absolute bound one in 17.330 seconds, with 1,536 conservative pieces.
At 19.735 seconds, lowering began on seven formal orders minus six through
zero, totaling 40,723,715 Atom bytes. Those are intermediate formal orders,
not validated physical coefficients: native resolution can still expose exact
zeros, and the complete original-expression oracle remains mandatory.

| Completed phase | Calls | Inclusive seconds |
| --- | ---: | ---: |
| Native face substitutions | 117 | 142.895 |
| Native partial cache misses | 60 | 15.535 |
| Incremental group additions | 3,072 | 8.221 |
| Coordinate-weight products | 3,072 | 5.195 |
| Formal endpoint axes | 18 | 4.311 |
| All native scalar/regular series misses | 30 | 0.072 |
| Initial regular native series spans | 2 | 0.004 |

These are completed inclusive spans, not an exhaustive disjoint wall-time
budget. Parent request-resolution spans include the partial and face spans;
regular/weight/prefactor phases can contain native-series misses. Do not add
parents to children. Trace metadata and writes are included in timings.

The largest observed partial expanded a 6,601-byte regular coefficient to
367,174,024 bytes under native coordinate derivatives with depths
`[0,2,2,2,0,1,0,2,0]`. Its native literal face substitution took 62.343 seconds
and returned only 33,303 bytes. Another partial grew from 3,958 to 125,764,394
bytes. All completed face results were at most 63,918 bytes. The process
stopped during another native face substitution while resolving formal order
minus four; lowering of minus six and minus five had completed, but no
complete-vector result was returned.

This trace attributes the dominant measured cost to constructing full native
partials and only then applying their faces. It does not support prioritizing
scalar-weight Series assembly, native epsilon-series depth, JIT compilation or
the allocator as the immediate fix for this particular timeout. The separate
conditional native weight-composition note remains source research.

## Smallest next source proposal

Investigate a native-only request resolver specialization for arguments that
are exactly their own formal coordinate or an exact numeric face constant.
For independent coordinates, finish the native derivatives in one axis, then
apply that axis's literal face before differentiating other axes. Substitution
in a different independent axis commutes with differentiation; all requested
derivatives in the replaced axis must happen first. This can avoid carrying a
large off-face expression through the remaining derivatives. It needs no
custom derivative, chain rule, Series arithmetic, limit or zero inference.

Keep the existing whole-partial/simultaneous-substitution path for arbitrary
composed arguments. A coordinate substituted by an expression depending on
another differentiation coordinate does not satisfy the simple commutation
argument. Exact face-domain admission remains required, and only native zero
or independence decisions can prune anything. The cache must include the full
request or equivalent derivative-and-face prefix, not only derivative depths.

Before another actual attempt, compare the proposed resolver against the
unchanged resolver on small mixed derivatives and faces, including zero
partials, face constants, poles/logarithms with admitted nonzero denominators,
and composed arguments that deliberately use the fallback. Then rerun all
existing native-series and complete-vector program/weighted/cold-reader
controls. The captured actual target, all inferred orders and three original
point-first oracles remain separate gates. No implementation or further
scientific process is authorized by this attribution record itself.
