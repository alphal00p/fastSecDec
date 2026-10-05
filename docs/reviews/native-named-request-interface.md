# Private native request-resolution interface

Private implementation handoff, 2026-10-05. The composition owner and
request-interface owner agree the seam below. The separated request module and
its eight focused controls are implemented; the combined private controller
gate passes fourteen tests. Public integration is a subsequent slice. This
implements the local request boundary from
[the adoption slices](native-named-production-slices.md), with native ownership
and [accepted symbol hygiene](native-symbol-hygiene.md). Public option/event
names remain the later integration owner's responsibility.

## Small private seam

The common private `coefficient_first` parent owns
`type NativeSeries = Series<AtomField>`, native width retries and composition.
Its `requests` child owns one attempt's names, bodies, partial/face caches and
alias lowering; `requests/interleaved.rs` owns only the reviewed restricted
native derivative/face scheduling. No new trait hierarchy or symbolic IR is
needed. Proposed signatures are shown below with bodies omitted; items and
fields needed by the parent have `pub(super)` visibility in the implementation,
and none is a public library API:

```rust
#[derive(Clone, Copy, Default)]
struct RequestLimits {
    max_unique_requests: Option<usize>,
}

#[derive(Clone, Copy, Default)]
struct RequestCounts {
    source_bodies: usize,
    unique_requests: usize,
    cached_partials: usize,
    aliases: usize,
    interleaved_requests: usize,
    fallback_requests: usize,
}

#[derive(Clone, Copy)]
enum RequestStage { ReserveSymbols, NameCoefficients, LowerRequests }

#[derive(Clone, Copy)]
struct RequestProgress {
    stage: RequestStage,
    counts: RequestCounts,
}

struct LoweredRequests {
    coefficients: BTreeMap<i32, AliasedAtom>,
    counts: RequestCounts,
}

impl Requests {
    fn new(
        terms: &[MappedTerm], parameters: &[Symbol], regulator: Symbol,
        limits: RequestLimits,
        poll: &mut impl FnMut(RequestProgress) -> ControlFlow<()>,
    ) -> Result<Self, GenerationError>;

    fn wrap_series(
        &mut self, series: &NativeSeries,
        poll: &mut impl FnMut(RequestProgress) -> ControlFlow<()>,
    ) -> Result<NativeSeries, GenerationError>;

    fn check_scalar(&self, scalar: &Atom) -> Result<(), GenerationError>;

    fn lower(
        self, coefficients: BTreeMap<i32, Atom>,
        poll: &mut impl FnMut(RequestProgress) -> ControlFlow<()>,
    ) -> Result<LoweredRequests, GenerationError>;
}
```

`new` reserves every source symbol from admitted mapped prefactors, regular
factors and powers, plus declared coordinates and regulator. The composition
caller is downstream of existing mapped-factor/face admission; argument shape
alone is not a domain proof. The registry stores the coordinate order and
native handles locally, without retaining the borrowed callback or input slice.
Each width attempt gets a fresh registry; discard the entire previous registry
before retry. Only `lower`'s complete native alias vector escapes.

`wrap_series` deduplicates exact native coefficient bodies, records only their
actual coordinate dependencies and names coordinate-dependent bodies. Constants
remain inline. It uses native `Series::map_coeff`, preserving and checking the
native absolute bound. It does **not** restore the physical coefficient body
to validate every production mapping; small exact-restoration tests remain the
correct owner of that identity check. A regulator in a regular coefficient is
a typed invariant failure.

`check_scalar` rejects owned formal names before composition uses a scalar
coefficient in a product, inverse or other nonlinear operation. Composition
calls it for every native scalar-series coefficient and owns every actual
Series operation. Requests has no expansion-depth logic, prefactor grouping,
endpoint rule, multiplicity, order padding or numerical evaluator.

`lower` consumes the registry and preserves input signed keys and literal zeros.
It recognizes only owned native function/`DER` requests, resolves them through
native differentiation and literal simultaneous replacement, inlines
coordinate-independent values, and deduplicates equal nonconstant alias bodies.
Every returned coefficient has the same complete flat native `AliasedAtom`
definition map. Reuse existing `kernel::program::build` to register that map
once and create the exact native IR; no extra FunctionMap or persistence codec.
No physical Taylor Atom is constructed solely for diagnostics.

## Request accounting and failure boundary

The unique-request key remains the prototype's exact native tuple
`(source Symbol, derivative depths, argument Atoms)`. Check and reserve one unit
for a new full key before resolution; commit its completed count and cache
entry only on success. Cache hits consume no
new unit; one request with several derivative steps still consumes one unit.
Check increment overflow and `max_unique_requests` before expensive work.
`None` supplies no additional cap; zero permits only outputs needing no owned
request. This limit does not bound source-body bytes, naming volume or memory
inside a native operation. Native width/attempt and subtraction-piece limits
remain the composition caller's independent controls.

Keep separate `(source, depths)` unsubstituted partial and full-request caches.
Interleaved results enter only the full-request cache. Permit the specialized
path only when every argument is its own coordinate or literal zero/one and
at least one face is present; use the accepted zero-depth-first scheduling.
All other tuples use the original derivative-then-simultaneous-substitution
path. The resolver choice is private, with an original-only test control; it
does not become a user numerical option.

Native depth/arity decoding must return an error rather than panic. Malformed
owned requests and unresolved owned names use `GenerationError::Invariant`;
checked count/depth/counter overflow and the caller cap use the existing
`ResourceLimit` variant. Native unrelated functions remain the evaluator's
responsibility. The new reserved-name conflict variant described in the hygiene
review distinguishes foreign process metadata from a generated-name invariant.
None of these failures starts an automatic physical fallback or supplies zero.

Before using a candidate name, native lookup skips a handle already in the
local input-occupied set. For other candidates, explicit empty-metadata native
builder admission plus `Symbol::is_exportable()` is mandatory before any Atom
construction. Keep the checked local `f`/`a` counters and existing namespace;
install no callbacks, unique-job names, global body registry or state reset.

## Borrowed cancellation and native callback adaptation

The integration owner maps `RequestProgress` into the existing
`GenerationEvent::Progress` bridge. `ControlFlow::Break(())` becomes
`GenerationError::Cancelled`. Poll during source reservation and naming, before
each distinct request and cache-hit lowering, before/after each native
derivative or face substitution, and before final alias assembly/return.
Counts are observations of completed local work, not physical contribution
counts. Requests emits no integral `Complete` event and owns no clock, worker
or presentation. The outer coefficient phase owns its exclusive duration.

The inspected native APIs are deliberately infallible callbacks:
`Series::map_coeff` takes `Fn` (`poly/series.rs:717`), while
`AtomCore::replace_map` takes `FnMut` (`atom/core.rs:2093`). There is no inspected
native fallible variant. Adapt these calls with a narrow **first-error latch**:
record the first error and do no further request work or polling after it.
Thereafter `map_coeff` returns each original coefficient unchanged and
`replace_map` leaves `Settable` unset. Immediately after the native call, inspect
the latch before bound/unresolved checks, cache updates or final assembly;
discard the entire candidate and return that original error. `map_coeff` may need a temporary
local `RefCell` for this callback adapter; it does not require globally shared
state or a custom Series constructor. A native call or traversal is still
nonpreemptible. On any `wrap_series` or `lower` error the composition caller
drops the entire attempt registry and never resumes it. A fresh registry is
created only for a separate valid native-width retry. No partly named series or
partially lowered vector is usable.

## Focused adoption checks

Move the existing small exact-restoration, composed/swapped-argument, negative
power, exact-zero, early-face and partial-cache-contamination tests beside the
new request owner. Add zero/one/duplicate request-budget boundaries, count/depth
overflow, cancellation before a request and between native steps, first-error
preservation, and no complete output after failure. Exercise the accepted
foreign metadata/hook and occupied-input cases through the actual allocator,
plus repeated jobs and two simultaneously live generated alias vectors.

The composition owner confirmed the exact signatures/progress/count types and
the canonical parent path `generation/coefficient_first.rs`; the request files
are `coefficient_first/requests.rs` and `requests/{interleaved,tests}.rs`. HEP
accepts the source seam with the explicit latch/drop semantics above. These
source signatures do not replace later public opt-in generation, complete
original-oracle/cold replay or full-graph acceptance gates.
