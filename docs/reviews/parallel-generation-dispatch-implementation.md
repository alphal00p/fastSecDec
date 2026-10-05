# Caller-owned geometry workers in generation

Implemented and focused-tested on 2026-10-05. The reviewed native parallel
geometry API now has an additive, cache-aware main-library entry:
`GenerationContext::generate_with_dispatch`. Existing free generation,
serial context generation, options and event variants remain unchanged.
No CLI scheduling policy or executor is added.

The caller supplies `GeometryDispatch`, an ordinary mutable closure receiving
a borrowed lazy native job iterator and returning opaque native completions.
The caller chooses worker count, pool/scoped threads, progress channels and
joins. The native cache constructs the plan, admits chart completions, admits
and canonically merges cone completions, and stores only the accepted complete
fan. A caller cannot inject a manufactured `Decomposition` through this API.
Native job, completion, ID, progress, phase and error types are reexported from
`fastsecdec::generation`; there is no parallel wrapper schema.

Cold misses invoke both native stages, including an empty cone stage for a
native zero-dimensional chart. Hits invoke no dispatcher. A single private
lookup/store implementation is shared with the existing serial cache path,
preserving ordered support/domain/limit keys, FIFO entry capacity, disabled
retention and immutable shared results. Native worker errors remain attached
to completions for canonical resolution; caller scheduling errors can return
directly. Missing, duplicate, foreign and wrong-stage work never enters the
cache. Failed/cancelled misses do not evict a prior valid entry.

Domain assessment and support extraction precede dispatch or cache lookup.
The resulting native maps rejoin the same per-integral mapping, residual,
symmetry, subtraction, Laurent and metadata pipeline. This keeps different
weights, coefficient signs and domain certificates independent even when
geometry is reused. Empty integrands skip geometry and dispatch altogether.

The existing generation observer receives native accepted geometry completion,
reuse status and ordinary phase/timing events. Per-job native observations stay
with the caller, tagged by job ID; their local counters are not reported as
global accepted sector counts. An immutable caller token is checked at stage,
merge and ordinary generation-event boundaries, after native final completion
and before returning the final generated result. Worker callbacks should read
the same token. No callback is borrowed mutably by concurrent workers.

Cancellation at native final completion prevents insertion. Cancellation at the
subsequent reuse event or a later symbolic phase can leave valid completed
geometry cached, but cannot return a partial integral. Cancellation remains
cooperative at native arithmetic boundaries. Entry count, worker count and
retained completion count are not byte-memory bounds. The caller remains
responsible for stopping/joining its own worker tasks before returning.

## Executed gates

`cargo test -p fastsecdec-sectors --locked -- --test-threads=1` passed **41 tests**
with two existing large geometry probes ignored; evidence is
`output/geometry-dispatch-sector-tests.log`. Three new cache tests cover
caller-owned scoped workers, reversed arrivals, all domains, zero-dimensional
stages, disabled retention, cold/warm interoperability with the serial cache,
each invalid-work category, native limits and cancellation before/between/after
completion. Prior geometry, work, cache and independent scientific tests pass.

`cargo test -p fastsecdec --test generation_context --locked -- --test-threads=1`
passed **eight tests** in 0.04 s after a 20.18 s build; evidence is
`output/geometry-dispatch-context-tests.log`. The three added controls compare
all compact native coefficients, exact offsets and chart/map metadata against
serial generation, require two cold dispatches and zero warm dispatches, and
check the complete `[-1, 0, 1]` vector against independent analytic functions.
A changed weight is applied once, a same-support sign change is rejected before
dispatch, empty input dispatches no work, incomplete work returns no result,
and cancellation retains only accepted geometry. The five prior context tests
remain unchanged and pass.

Sector all-target Clippy and main-library/generation-context Clippy with
`-D warnings` pass in `output/geometry-dispatch-sector-clippy.log` and
`output/geometry-dispatch-context-clippy.log`. Owned-file formatting and diff
whitespace checks pass. The pre-existing native dependency unused-result
warning is unchanged. Independent source review and gate closure are recorded
in `parallel-generation-dispatch-independent.md`.

This is an end-to-end correctness proof for caller-scheduled geometry during
generation. It does not claim a speedup, parallel symbolic algebra, new domain
admission, CLI worker steering or a performance result on the hard examples.
