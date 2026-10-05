# Caller-owned native parallel geometry

Implemented in `fastsecdec-sectors`, 2026-10-05, following the independently
reviewed `parallel-geometry-api-proposal.md`. The crate exposes two native work
stages and creates no executor. Existing serial `decompose`, complete-cache
behavior and main-library generation APIs remain unchanged.

`GeometryPlan::new` validates the exact input domain and native ordered
supports. The caller executes `charts()` jobs and returns opaque completions
to `prepare`. It then executes the prepared `cones()` jobs and returns them to
`finish`. Both iterators are lazy; jobs share immutable native input/facet
owners through `Arc`. A caller can use scoped standard threads, an existing
pool or its own runtime. `GeometryCompletion::id/error` supports caller-owned
reporting without exposing writable maps or a partial scientific fan.

The private `stages.rs` helpers contain the original chart preparation and
candidate-cone operations extracted from `decompose.rs`. Both execution paths
use those helpers. Minkowski supports, exact rank, double description,
pulling, determinant, Jacobian and factor-valuation mathematics are unchanged.
The serial driver still completes each chart before preparing the next and
retains its progress stream. No new CAS, geometry algorithm or dependency was
introduced; Numerica remains the exact arithmetic owner.

Completions are bound to their originating plan and prepared stage by private
shared ownership. Equal inputs do not permit cross-plan or cross-stage result
substitution. Missing, duplicate, unknown and foreign records are typed errors.
Successful merge preserves chart, candidate and native simplex order, including
signed orthant maps, positional valuations and fixed projective parameters.
Zero-dimensional charts have no cone jobs and contribute only their native
trivial map at final merge.

Native failures retain successful conversion prefixes privately. Final merge
checks cumulative sector limits on every prefix map before returning any later
conversion error; a pulling failure has no converted prefix. Later chart
preparation errors wait until earlier-chart cones are resolved. Cancellation
may terminate incomplete dispatch but never returns a partial decomposition.
Job callbacks describe local work and emit no `Complete`. The final observer
receives the accepted global completion and may still cancel it. Invalid
provenance remains an error even when cancellation is also observed.

All public owners and their actual native payloads satisfy `Send + Sync`.
The caller owns cancellation and observation channels; it must not borrow one
mutable observer concurrently. Native arithmetic remains cooperatively
interruptible at its existing callback boundaries. Prepared facets and
out-of-order map completions can consume more memory than the serial driver;
bounded in-flight work is not a byte-storage bound.

## Executed acceptance

`cargo test -p fastsecdec-sectors --locked -- --test-threads=1` passed **38 tests**,
with the two existing large geometry probes ignored. The final log is
`output/parallel-geometry-final-tests.log`; the initial 37-test pass is retained
separately. The final addition explicitly counts native facet preparation in
a private test: three projective charts prepare facets exactly once each, and
all candidate jobs reuse them without further preparation.

The ten new tests cover real caller-owned scoped threads with one/two workers
and reversed arrivals; exact serial maps/counts in all three domains; a
four-dimensional non-simplicial fan; the independent exact cube moment 1/6;
translated valuations and signed infinity charts; zero-dimensional and
one-parameter projective inputs; all three native limits and admission errors;
foreign equal-input plans and prepared stages, missing/duplicate/wrong-stage
records; worker/stage/merge/final cancellation; and private canonical
prefix/error precedence controls. Existing serial geometry, independent
scientific geometry, cache and map-validation tests pass unchanged.

`cargo clippy -p fastsecdec-sectors --locked --all-targets -- -D warnings` passed
in `output/parallel-geometry-clippy.log`; crate formatting and diff whitespace
checks pass. These sector-only runs use exact Numerica arithmetic and initialize
no Atom or Symbolica runtime. The independent implementation review is recorded
in `parallel-geometry-api-independent.md`.

This establishes the native scheduling boundary and deterministic correctness,
not a speedup. Subsequent cache-aware dispatch and the additive
`GenerationContext` entry have their own end-to-end correctness gate in
`parallel-generation-dispatch-implementation.md`.
No public arbitrary cache insertion, new sector identity, serialized job codec,
CLI scheduling change or pool was added here.
