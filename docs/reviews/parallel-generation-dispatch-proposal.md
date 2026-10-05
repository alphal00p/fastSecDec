# Cache-aware geometry dispatch during generation

Reviewed next-slice proposal, 2026-10-05. The native two-stage work API has
passed its focused correctness and independent review gates. This proposal
connects that existing API to `GenerationContext`; it adds no mathematics,
executor, CLI scheduling or new worker framework. The implemented entry and
executed end-to-end correctness gates are now recorded separately in
`parallel-generation-dispatch-implementation.md`; the design below is retained.

## Minimal additive surface

Keep `decompose`, `GeometryCache::decompose`, free `generate`, and
`GenerationContext::generate` unchanged. Add one sector-cache method and one
main-context method. A named native callback type makes their shared contract
readable without introducing a trait hierarchy:

```rust
pub type GeometryDispatch<'a> = dyn FnMut(
    &mut dyn ExactSizeIterator<Item = GeometryJob>,
) -> Result<Vec<GeometryCompletion>, SectorError> + 'a;
```

`GeometryCache::decompose_with_dispatch` takes the same domain, ordered support
slice and decomposition options as `decompose`, plus `&mut GeometryDispatch`,
a cancellation predicate and the existing final native progress observer.
`GenerationContext::generate_with_dispatch` takes the integral/options,
`&mut GeometryDispatch`, an immutable `Fn() -> bool` cancellation predicate,
and the existing `GenerationEvent` observer. The native alias/job/completion
types should be naturally reexported alongside the new context method; HEPKit
need not invent a geometry adapter or import private modules.

The callback is invoked exactly twice on a successful cold nontrivial request:
chart jobs, then candidate-cone jobs. It receives a borrowed lazy exact-size
iterator, so the caller can bound in-flight jobs using its existing runtime.
An empty native cone stage is still passed as an empty iterator; it is distinct
from an omitted stage. A warm cache hit invokes it zero times. A chart error
can mean fewer/no cone jobs before canonical final failure, under the existing
work API's ordering contract. The callback returns only native opaque
completions, never a caller-created `Decomposition`.

The callback owns job execution, channels and worker joins. It captures any
per-job observation/cancellation handles. It may return a typed error for its
own scheduling failure, or native completions containing scientific errors;
the latter lets the native assembler retain canonical error precedence.
Returning missing/duplicate/foreign completions remains a typed native work
error. The library does not catch a caller panic, create/join external threads,
or convert an incomplete execution into success. A caller that returns early
is responsible for stopping/joining its own workers.

## Native cache owns admission and final merge

On a hit, reuse the existing exact key: input domain, ordered canonical support
vectors and all three native limits. Poll cancellation, report the retained
native `Complete`, and return `reused: true`; never dispatch fabricated jobs.
On a miss, the cache constructs `GeometryPlan` itself, passes only its jobs to
the callback, calls native `prepare`, dispatches its cone jobs, then calls
native `finish`. The caller cannot select the scientific input, substitute a
fan, alter valuations or change the complete map order at this boundary.

Use one private completed-result insertion helper shared with the old serial
cache path. Insert only after native `finish` and the final observer accept
the complete fan. No public insertion or user-supplied decomposition closure
is added. Dispatch/worker/stage/limit/cancellation errors cannot insert or
evict an entry. Zero capacity still disables retention and capacity still
counts entries, not bytes. Returning an `Arc` does not change canonical map
order or guarantee bounded total memory when the caller retains results.

The main generation entry retains domain assessment and exact support
extraction before cache lookup. It passes the validated inputs to this native
cache method, then uses the same mapping, residual admission, symmetry,
subtraction, Laurent and metadata pipeline. Changing coefficient signs,
prefactors or numerator data still rechecks that integral independently even
when its support hits the geometry cache. Empty integrands continue skipping
geometry entirely. The generation result remains all-or-error.

## Progress and cancellation ownership

Preserve existing event variants and old-method behavior. The new dispatch
entry forwards final accepted geometry completion, `GeometryReuse` and the
ordinary generation phases/timings through its existing observer. Native job
progress remains tagged by `GeometryJobId` and delivered through the caller's
dispatcher; those local counters must not masquerade as a global accepted
sector count. No new shared mutable observer is passed to multiple workers.

The immutable cancellation predicate can read a caller-owned atomic token.
The main entry checks it at its ordinary generation-event boundaries and the
cache checks it before lookup/dispatch, between stages, during final merge and
before completed admission. The caller's worker callbacks read the same token
at existing native arithmetic boundaries. A mutable observer can request
cancellation in the ordinary way; a dispatcher can also set the token while
draining its own observation channel. There is no asynchronous interruption
inside native matrix operations and no new terminal/UI policy.

As with the existing context, cancellation at native final completion prevents
cache insertion. Cancellation at the later `GeometryReuse` event or a later
generation phase may leave valid completed geometry cached; it never returns
a partial integral. This distinction must remain documented and tested.

## Acceptance before claiming parallel generation

Add a small actual main-library generation test with a caller-owned scoped
two-worker dispatcher and reversed completion arrivals. Compare every native
Laurent coefficient, exact offset and retained chart/map metadata against the
unchanged serial entry, with an independent analytic coefficient control.
Require both stages to execute on the cold call and neither on a warm call;
compare warm complete outputs after a changed nonsingular weight. Exercise
the existing domain reassessment failure on same-support sign changes before
dispatch, and verify no erroneous hit supplies a positivity certificate.

Native cache tests must cover cold/warm/zero-capacity dispatch, all supported
domains and zero-dimensional stages, plus missing/foreign/duplicate returns
and callback cancellation. Require unchanged cache entries after every failed
or cancelled miss, including final completion. Main tests additionally cover
empty input with a dispatcher that must never run, cancellation during native
work and after accepted cache insertion, and no complete output on failure.
Existing serial/cache/context tests remain unchanged and must pass.

These tests establish reusable end-to-end scheduling, not speedup. Any later
CLI adoption or geometry timing campaign should separately measure native
preparation, cone work, merge, cache overhead and retained memory. The global
symbolic/runtime ownership and unchanged default serial execution remain
explicit.
