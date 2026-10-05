# Caller-owned parallel geometry: concrete API proposal

Reviewed source/interface proposal, 2026-10-05. The sector-only implementation
and executed correctness gates are now recorded in
`parallel-geometry-implementation.md`; no timing campaign or main-library
parallel-generation adoption has run. The original review-only skeleton is
`output/probes/parallel_geometry_api.rs`. This
follows `geometry-reuse-scheduling-proposal.md`; completed GeometryCache and
GenerationContext slices remain separate, accepted behavior.

## Public boundary

Use four opaque owners and one work identity: `GeometryPlan`,
`PreparedGeometry`, `GeometryJob`, `GeometryCompletion`, and `GeometryJobId`.
These expose exactly chart preparation and candidate-cone decomposition. They
are not a generic task graph, distributed-work codec or executor. Neither jobs
nor completion payloads are publicly constructible or deserializable.

| Owner | Operation | Contract |
| --- | --- | --- |
| `GeometryPlan` | `new(domain, owned_supports, options)` | Reuse existing whole-input dimension/homogeneity validation; own immutable native input and policy. |
| `GeometryPlan` | `charts()` | Lazy jobs in native gauge order, or one cube/orthant chart. |
| `GeometryJob` | `id()`, `run(native_progress_callback)` | Caller executes the unchanged native stage; completion always retains its origin, ordinal and success/error. |
| `GeometryPlan` | `prepare(chart_completions, cancelled)` | Validate chart provenance/coverage and own prepared native candidates/facets. |
| `PreparedGeometry` | `cones()` | Lazy jobs in chart/candidate order sharing each chart's prepared data. |
| `PreparedGeometry` | `finish(cone_completions, cancelled, final_progress)` | Validate complete work, merge ordered native maps with cumulative limits, then emit accepted `Complete`. |
| `GeometryCompletion` | `id()`, `error()` | Caller can retain all observed errors without altering private scientific payloads. |

`GeometryJobId::{Chart {chart}, Cone {chart,candidate}}` identifies work within
one plan. It is neither a sector ID nor a content digest. A single opaque job
type keeps the API small; a private stage tag prevents using chart completions
as cone results. A small typed `GeometryWorkError` payload of `SectorError`
distinguishes foreign plan, wrong stage, unknown job, duplicate job and missing
jobs. Missing coverage reports its count and first canonical missing identity.

The plan owns an `Arc` of exact native inputs. Prepared geometry owns a separate
private stage identity and immutable per-chart data. Jobs retain those owners;
completions retain the matching private identity. Native `Arc::ptr_eq` suffices
for in-process admission. Equal scientific inputs in a newly constructed plan
still cannot exchange completions. Cloning a job can repeat computation, but
duplicate acceptance is rejected. No global counter, address serialization,
unsafe reset, Atom or new identity hash is required.

## Exact operations and extraction seams

Extract the existing `decompose_chart` body into private native helpers; both
the old serial driver and the new work API call those helpers. The serial
driver continues finishing one chart before the next and retains its existing
observer/error behavior and memory lifetime. It need not force old callers
through a collect-all parallel stage.

Chart preparation retains the existing support restriction, Minkowski candidate
ordering, positive-orthant affine-rank check, lifted generators and one
`extreme_rays` call. Shared data are native support/candidate/facet vectors,
fixed-axis identity and the unchanged cube/orthant mode. A candidate-cone job
reuses the existing incident-ray sorting/deduplication, native rank, inequality
construction, `pulling`, determinant, Jacobian and positional factor valuations.
The pulling cache remains local to that candidate, as it is today. The
double-description loop and its deterministic minimum-pair tie-break stay
serial. No numerical or symbolic algorithm is copied into the scheduler.

A legitimate nonvertex returns a successful empty contribution with geometric
vertex count zero. A native failure is an error-bearing completion, never an
empty contribution. A zero-dimensional chart uses the existing trivial map;
it has no candidate-cone jobs and retains its map privately for the final merge.
Thus an empty cone stage is valid only when established by native preparation,
not when a caller omits work.

The existing Numerica QMC package/accumulator design supplies an ownership
example, not reusable geometry types: its work descriptors carry lattice,
randomization and statistics. Native integer/matrix arithmetic already provides
all geometry operations. Callers can use `std::thread::scope`, an existing
Rayon pool, or HEPKit's runtime. The crate acquires no pool/channel/affinity or
Rayon dependency. Existing CLI numerical execution demonstrates bounded ordered
batch collection; no CLI scheduling change is part of this first API slice.

## Deterministic completion, limits and errors

Completion order does not determine map order. Assemble charts by original
gauge ordinal, candidates by the existing sorted candidate vector, and maps by
the native simplex order. Preserve every exponent row, signed determinant,
Jacobian power, fixed parameter and ordered factor valuation. Candidate and
geometric-vertex totals use checked accumulation. Do not silently sort final
maps into a different index convention.

Keep all three exact native limits. `max_support_pairs` and `max_rays` apply
inside their current native operations; `max_sectors` still bounds each native
pulling result and the complete ordered output. Cone workers can conservatively
stop at the full per-call sector limit, while `finish` applies the cumulative
limit in canonical order. It cannot expose an over-limit prefix. This can
perform extra work before learning that earlier maps exhausted the global
limit; it does not turn races for a shared quota into physical sector selection.

A chart preparation error stays attached to its canonical chart in
`PreparedGeometry`. Cone jobs need only be exposed for successful charts before
the earliest failed chart. `finish` merges earlier charts first and resolves any
earlier native cone/sector-limit error before that later preparation error.
This avoids a stage-one error in a later chart masking an earlier canonical
failure merely because all chart preparations ran first. Within a successful
chart, process candidates canonically, including the native successful map
prefix before any terminal conversion error within that candidate. This detail
is necessary: the serial algorithm computes pulling first, then converts and
appends each simplex map while checking the global sector limit. A later
conversion failure must not mask a cumulative limit already exceeded by an
earlier map in the same cone. Consequently the private cone outcome retains
`successful_maps` plus an optional terminal error, rather than a plain
`Result<Vec<SectorMap>, _>` that discards its prefix. An initial native pulling
failure has no converted-map prefix. No public partial fan is exposed, and no
new mathematics or map conversion is performed to establish precedence.
Callers may log all observed errors through `GeometryCompletion::error()`
before consuming results.

Each stage first rejects foreign/wrong-stage/duplicate/out-of-range received
records. With no observed cancellation it also requires complete prescribed
coverage. No missing work means zero. Native output-empty validation occurs at
the serial algorithm's chart-prefix boundary rather than at each individual
candidate. This preserves its scope, including projective gauge handling.

Cancellation has an explicit exception to complete coverage: a caller token or
accepted cancelled job returns `SectorError::Cancelled`, never a partial
`Decomposition`. Its timing can change which extra jobs finished or which
noncancellation errors were observed; no identical error precedence under
asynchronous external cancellation is promised. Without cancellation, canonical
completion/error resolution is deterministic. If a caller stops dispatch early
for its own error policy, it returns that error itself; `finish` does not accept
an incomplete success.

## Progress and storage ownership

`GeometryJob::run` retains the existing `FnMut(&DecompositionProgress)` callback
and monitor points. The caller captures `job.id()` to tag each stream. Progress
fields describe that job's local native work; in particular `sectors` is not a
global accepted count. Jobs emit no artificial complete-integral event. A
single existing user/UI observer must not be borrowed concurrently. Each
worker may read a caller-owned atomic cancellation flag and send its tagged
observations to a caller-owned channel; the caller decides observation cadence.

Check cancellation before a job starts, at all existing monitor points, at
stage transitions, during final merge and at the final `Complete` callback.
The completed fan is returned only after that final callback accepts it.
Native matrix operations remain cooperatively interruptible at existing
boundaries, not asynchronously preemptible.

The lazy iterators permit a caller-selected bound on in-flight work. Prepared
geometry retains chart supports/candidates/facets once; jobs share them and own
only cone-local work. Collecting out-of-order completions can retain substantial
maps, and preparing several charts together retains more facets than the old
serial driver. Neither a worker-count bound nor the existing entry-count cache
capacity is a byte bound. These costs must be measured, not hidden as a claimed
parallel speedup.

No public arbitrary cache insertion is added. The first work API is cache-free;
a later native cache-aware coordinator may admit only the private successfully
finished path under the same exact key and limits. Existing cache hits and
GenerationContext domain reassessment stay unchanged. This proposal adds no
per-sector content identity, affine endpoint charts, domain admission or CAS
equivalence machinery.

## Meaningful acceptance gates

Before implementation acceptance, compile actual public plans/jobs/completions
as `Send + Sync` with their real native payloads. The placeholder sketch is not
that proof. Use caller-owned scoped threads in tests, not a library-created
executor.

The core tests should require exact serial equality for one worker, two workers,
and deliberately reversed completion order in all three input domains. Include
the existing non-simplicial fixture, signed infinity maps, homogeneous multiple
gauges, zero-dimensional cube/orthant and one-parameter projective charts.
Reuse independent exact cube-moment/valuation controls so equality alone is not
the scientific oracle. Check that each prepared chart's facet computation is
invoked once, with a private test counter rather than public metrics.

Exercise duplicate jobs, equal-input foreign plans, chart/cone stage confusion,
missing completions and native errors; preserve private constructors. Test the
three resource limits at their existing boundaries and cumulative sector
overflow spread over several successful cone jobs. Include an earlier cone
failure versus a later chart-preparation failure, and cumulative sector-limit
exhaustion in a cone's successful prefix before its later conversion error.
Use a private native-stage fault-injection seam for the latter precedence
control if no natural small failing geometry is available; do not add a public
partial-result constructor or manufacture a scientific coverage certificate.
Cancellation controls cover
before dispatch, native worker callbacks, between stages, merge and the final
observer; none yields a completed cacheable fan. Existing serial and cache
tests must pass unchanged.

An initial bounded prototype can refactor the private stage seams and exercise
the public ownership contract without enabling parallelism in production
generation. Main-library or CLI adoption and performance measurements are
subsequent reviewed slices. No speedup, memory reduction or completed numerical
integral follows from these proposed tests.
