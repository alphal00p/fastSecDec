# Geometry reuse and caller-owned scheduling proposal

Review and staged implementation plan, 2026-10-05. This addresses the explicit
geometry-cache and parallel chart/cone items in `FIRST_PHASE_PLAN.md`. The
coordinator accepted the complete-cache first slice; its implementation and
sector-only gate are recorded in `geometry-cache-core.md`. The additive generation
context is implemented and tested as recorded in
`generation-context-geometry-reuse.md`. The native two-stage parallel geometry
API and its focused tests are recorded in `parallel-geometry-implementation.md`;
main-library adoption and its separate focused controls are recorded in
`parallel-generation-dispatch-implementation.md`. The additive
per-sector identity source/gates are recorded in `sector-content-identity.md`.
Complete on-shell generation and pending reference outcomes retain priority.
The sector crate already owns the required exact mathematical operations.

## Reuse evidence and boundaries

| Existing owner | Reusable contract | Limit |
| --- | --- | --- |
| `fastsecdec-sectors::PolynomialSupport` | `from_exact` validates dimensions/nonnegative native integers, then sorts and deduplicates monomial exponent rows. | It canonicalizes each support; it does not identify arbitrary coordinate permutations or translated supports. |
| `decompose.rs`, `stages.rs`, `cone.rs`, `triangulate.rs` | Native exact Minkowski candidates, deterministic double description, candidate normal cones, pulling triangulation, determinants/Jacobians/valuations. | Serial and caller-scheduled paths share native stage helpers; ordinary generation remains serial while an additive context entry accepts caller-owned dispatch. |
| `arithmetic.rs` / Numerica | Existing exact integer/rational matrix rank, inversion and determinant. | No floating hull heuristic, custom bigint/matrix backend or additional CAS is required. |
| `generation/support.rs` | Caller-generation-local native Atom to exact support reuse. | This saves support extraction, not complete geometry. Domain certificates and mapped residuals remain outside it. |
| Numerica `numerical_integration::qmc` | Immutable plans/work descriptions, caller dispatch, typed completion and duplicate rejection illustrate the existing ownership model. | Those types contain lattice/randomization/statistical semantics; they are not generic geometry jobs or an executor to reuse directly. |
| CLI `driver/execution` | Caller-owned Rayon pool, ordered batch collection, checked acceptance, explicit cancellation. | The numerical libraries own no pool. Geometry need not acquire a Rayon dependency or copy an integration scheduler. |

The source inspected is the current native dependency in
`DO_NOT_PUSH_FOR_REFERENCE_ONLY/numerica`, not an unrelated checkout. Numerica's
QMC module explicitly creates no threads, chooses no stopping rule and invokes
no integrand. A standard caller `std::thread::scope`, existing Rayon pool or
HEPKit runtime can execute geometry jobs. This proposal adds only geometry
work/ownership boundaries; no generic task framework is justified.

## Small first slice: complete in-memory reuse

Add a caller-owned, bounded `GeometryCache` in the sector crate. Its entries
contain only successfully completed native decompositions behind immutable
shared ownership. No global cache, disk schema, Atom strings, geometry parser,
dependency patch or new mathematical normal form is needed. A hit must not
clone every map merely to inspect it; borrowing or `Arc` can reuse the result.
Entry creation stays private to the validated native completion path. A public
`SectorMap::validate` is not a proof that an arbitrary collection covers a fan,
so accepting caller-inserted arbitrary maps would be the wrong shortcut.

The initial key is an exact structured tuple:

- input domain and source dimension;
- the **ordered** vector of existing canonical polynomial supports, including
  ordered coordinate axes and every native integer exponent;
- the three existing decomposition resource limits, conservatively retained
  in the lookup policy so warm reuse cannot bypass a tighter request.

Including the limits is a policy qualification, not a claim that they alter
mathematical geometry. A later measured extension could retain exact peak
resource counters and admit compatible limits without recomputation; do not
invent or infer those counters in the first slice. Use native equality/ordering
on these values. A lookup hash, if useful, must retain exact equality checks;
it is not a new scientific artifact identity.

Preserve support-list order initially. `factor_valuations` is positional, and
intermediate Minkowski workloads depend on summand order. Sorting/deduplicating
factors or dropping monomial translations solely for cache hits would need
explicit reconstruction of occurrence valuations and a resource-policy review.
The existing `translation_does_not_change_fan_but_shifts_factor_valuations` test
demonstrates why equal cones do not imply equal cached outputs. These broader
equivalences are unnecessary for a useful same-support cache.

Coefficients, exponents, graph numerators, prefactors and parameter names are
excluded only because this cache owns **geometry alone**. Main-library domain
assessment, singular-factor selection, exact mapping and residual admission
still run for each integral. A mass/kinematic change that cancels a polynomial
monomial produces a different native support and hence a different key.
Zero-dimensional cube/orthant inputs and projective gauges remain distinct.

Expose cache capacity/clear controls and cache-hit observations, not an
unbounded process lifetime. Use a simple deterministic eviction policy or
caller-chosen admission; define whether capacity counts entries or retained
map storage without claiming an unmeasured byte bound. Failed or cancelled
decompositions are not successful entries. A cancellation callback must run
before returning a hit and before accepting a newly completed result. Report
reuse honestly rather than replaying fabricated computation progress.

The existing serial `decompose` remains source-compatible and cache-free by
default. An additive cache-aware entry can wrap the same algorithm. The main
library may accept this caller-owned context at an additive generation entry;
ordinary `generate` keeps its existing behavior. This allows reuse across
integrals without storing borrowed physical inputs in a global object.

## Parallel slice: expose the existing dependency stages

The smallest useful graph of work follows existing `decompose_chart` boundaries:

1. **Chart preparation.** For each original projective gauge (or the sole
   cube/orthant chart), reuse native support restriction, Minkowski candidates,
   rank checks and the one exact `extreme_rays` call. Its sorted candidates and
   facets form immutable prepared chart data.
2. **Independent candidate cones.** Each candidate vertex uses those same
   immutable data to obtain incident rays, native rank checks, inequalities,
   pulling simplices and existing map/Jacobian/valuation construction. A
   nonvertex can return a checked empty contribution. Do not run the expensive
   shared facet computation again for every candidate.
3. **Completion.** Assemble checked results in original chart order, then
   original sorted candidate order, then the existing native simplex order.
   Require each required job exactly once before exposing a completed
   `Decomposition` or inserting it into the cache.

Use private-constructed plan/job/result records bound to immutable native input
data. A result must not be accepted merely because its integer chart index
matches: duplicate, missing, out-of-range and foreign-plan returns are errors.
These are in-process Rust values, not a new serialized work protocol. Preserve
the old serial convenience function by driving the same stages sequentially.
Avoid speculative checkpointing of unfinished symbolic work in this slice.

Caller dispatch can parallelize chart preparation and candidate-cone work.
Use a bounded number of jobs in flight; do not nest automatically created pools
or materialize every job's copies of the shared facets. The library neither
starts threads nor installs a global Rayon pool. A thin CLI coordinator can
reuse its pool dependency; HEPKit can instead use its own executor. A small
caller-supplied executor closure or explicit plan/worker/finish methods should
be chosen at the focused API prototype, keeping this fixed two-stage geometry
contract rather than adding an extensible scheduler abstraction.

Keep the native double-description loop itself serial initially. Its next
constraint depends on current rays and the deterministic minimum-pair/index
tie-break, so arbitrary parallel updates would change the algorithm. Parallel
candidate normal cones already satisfy the plan's independent-cone requirement.
The pulling cache remains local to its native cone calculation.

## Determinism, limits and cancellation

Geometry arithmetic is exact, so ordered merging can preserve **every ordered
map**, not just an integral within tolerance. Candidate/geometric vertex counts,
fixed axes, signed exponent matrices, absolute determinants, Jacobian powers
and valuation rows must match the original serial result. Keep zero-dimensional
admission and the final no-full-dimensional-sector error at their correct scope;
an empty individual candidate is not an empty whole integral.

The existing limits have different meanings: support pairs and rays constrain
native intermediate work, while sector count also caps the complete output.
Workers retain the existing local checks; the coordinator checks the cumulative
sector limit in canonical merge order. It must never expose an over-limit or
partially covered result. Parallel work may complete extra unaccepted jobs
before that decision. Bound in-flight work explicitly and disclose temporary
worker storage; an atomic race for a shared quota should not decide which
physical sectors survive. No partial output is a scientific zero or usable
integral.

Preserve the existing `ControlFlow` callback model. Each worker's ordinary
native callback can inspect a caller-owned cancellation token; no concurrent
access to the caller's existing `FnMut` observer is allowed. The caller collects
tagged progress and updates its single observer. Events can interleave, but
accepted counts and final merge order cannot. Check cancellation before dispatch,
inside the current native monitor points, after completed jobs and before cache
insertion/final return. The current exact matrix operation granularity remains
cooperative; do not promise immediate interruption inside native arithmetic.

Error aggregation needs a documented deterministic policy: retain all observed
job errors for diagnostics and choose the earliest canonical completed failure
for the returned error, while cancellation remains explicit. This need not
promise identical last progress event or identical error precedence when the
user cancels before another job finishes. Completion and failure are distinct
from a caller intentionally stopping dispatch.

## Acceptance before connecting the slice

The implemented sector-only API establishes native integer/support/map
`Send + Sync` usability with caller-owned scoped workers and exact canonical
merge through the gates in `parallel-geometry-implementation.md`. The later
main-library control is recorded in
`parallel-generation-dispatch-implementation.md`. No parallel timing campaign
has run. Complete cache and generation-context tests remain recorded in their
separate implementation reviews.

Meaningful tests then cover:

- exact cold/warm equality in all three domains, zero dimensions and projective
  gauges; reordered support rows hit, translated supports/factor-order/axis/
  domain changes cannot reuse incorrect valuations;
- fresh domain/residual admission with same-support polynomials of different
  sign, so a geometry hit never supplies a positivity certificate;
- serial, two-worker and reversed completion-order equality for all maps and
  counts, including the existing non-simplicial and signed-infinity fixtures;
- duplicate/foreign/missing job rejection, immediate and late cancellation,
  limits on both hits and misses, and no successful cache entry on failure;
- unchanged complete generated Laurent vectors and kernel metadata using the
  existing generation/scientific controls, without a second integrator.

Use the existing ignored hard-geometry probes for bounded warm/cold and worker
attribution after correctness passes. They already distinguish the physical
F-only support from the combined-U/F stress workload. Record preparation, cone
work, merge/cache overhead, peak storage and complete ordered map fingerprints;
an integral's Laurent or compilation time cannot be counted as geometry speedup.

## Additive per-sector content identity

Implement this as a separate small native kernel API after coverage, rather
than treating the geometry lookup key as a physical sector identity. A proposed
`KernelSet::sector_content_id(index)` returns a versioned derived digest; the
owner needs the integral's output layout and retained semantics, so a bare
`SectorKernel` accessor cannot supply the complete contract on its own. Reject
an unknown index. Cache the derived digest lazily if measurement justifies it.

Hash a domain-separated, unambiguous typed record built from existing owners:

- exact immutable native program bytes, with the v3 native-codec and compiler
  policy identifiers;
- ordered input parameter names/dimension and the exact output-to-Laurent-order
  and Real/Imag layout, including zero padding;
- precision policy and cancellation degree/tuples used by the numerical kernel;
- native domain/branch/assertion semantics and the retained chart/measure
  records associated with that kernel, including merged-chart associations.

Reuse the artifact/metadata owners' canonical transport at this hash boundary;
do not disassemble instructions, restore aliased coefficient Atoms or create a
second map/schema implementation. Factor multiplicity already affects the
native program; the retained associated maps additionally preserve its origin.
The metadata owner can expose a borrowed per-kernel selection for hashing.
Initially keep those selected records' existing order and original chart IDs.
This deliberately permits safe cache misses if parent chart numbering changes;
inventing a chart-equivalence canonicalizer is unnecessary for the intended
content-reuse contract. Include a distinct missing-metadata marker for legacy
artifacts, never fabricate a phase-one certificate.

Exclude unrelated sector programs, global exact offsets, timings, worker count
and sampling settings. Exact-only contributions have no numerical sector ID;
their existing whole-result manifest still owns them. The same sector retained
in two otherwise different integral artifacts can consequently have the same
derived ID when its actual bytes, layout, numerical policy and selected
semantics agree. Conversely, changing any bound field must change the ID.

This is **representation content identity**, not a CAS proof of mathematical
equivalence. Factoring, parameter renaming, native instruction ordering or a
codec/backend version change may alter it despite equal mathematical values.
Use a new explicit digest version when its contract changes. A machine-code
cache additionally needs architecture/CPU and actual backend build identity;
the portable sector digest alone is not such a cache key. Dynamic replay state
still requires its existing caller policy/parent/sector binding, and numerical
results still require sampling identity. Neither can be transplanted merely
because sector content hashes match.

Keep current index selection, parent artifact/content IDs, checkpoint versions,
replay identity and saved-result scope unchanged. Do not insert the derived
field into existing serialized payloads or recompute their old IDs. Legacy
access may derive a current native view after its normal validated load, but
must preserve the original legacy bytes and label missing semantic metadata.
Tests should establish stable generated/loaded v3 identities, sensitivity to
each policy/layout/semantic field, isolation from unrelated exact offsets and
timing changes, same content in two parent sets, and explicit legacy handling.
No zero/analyser or symbolic-equivalence feature is needed.

## Unchanged scope decisions

This work resolves neither affine upper-boundary charts nor general mixed-sign
boundary admission. The currently tested `(1-x)^(-1+eps)` rejection remains
explicit until a separate mathematical scope decision. It implements no
contour/GCAD strategy.

Deterministic merge preserves existing original sector indices qualified by
their parent kernel content identity. The additive accessor above meets the
separate content-reuse requirement without changing those indices: the same
geometric map can act on different numerators, prefactors or branches. Parallel
scheduling is implemented at the sector-only work boundary with the focused
gates in `parallel-geometry-implementation.md`; the additive main-generation
entry is tested separately in `parallel-generation-dispatch-implementation.md`.
Commit `33a01ec` implements the additive content accessor with
four tests and the independent audit in `sector-content-identity-independent.md`.
Its selected parent kernel ordinal is localized to zero while original chart
IDs/order remain bound. These additions and the completed cache/context slices
do not change the existing identity or mathematical scopes above.
