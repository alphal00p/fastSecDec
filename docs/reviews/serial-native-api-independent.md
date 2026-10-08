# Independent native API and HEPKit reuse review

Independent source review, 2026-10-08. The reviewer did not implement the
serial-generation, indexed-storage or serial-integration APIs. This review
accepts their ecosystem and caller-ownership boundaries. It does not replace
the separate scientific, process-recovery, residency or executable acceptance
gates.

## Scope and method

Read the approved `SERIAL_MODE_PLAN.md`, repository guidance, the relevant
first-phase requirements and the current reuse audit. Inspected the public
exports and implementations of `generation::streaming`, `kernel::indexed` and
`integration::serial`; their CLI consumers; the existing HEPKit input boundary;
and the native, portable and isolated Python consumer manifests.

The review also read the existing native-codec, numerical-composition and
Numerica accumulator evidence. It inspected registry Numerica 3.0.1's manifest
and scalar accumulator source independently. No Cargo command was run alongside
the coordinator's builds. The final workspace run and focused family cleanup
rerun validate **638 distinct native workspace tests, with 28 intentional
ignores**. The sole failure in the full run was the order of test-file cleanup;
its focused two-test rerun passes without a numerical implementation change.
The reviewer inspected the logs directly and avoids counting the repeated
already-passing test twice.

The portable host suite passes **72 tests**. The isolated Python binding check
with `python_stubgen` also passes. Its feature graph exposed a type-inference
ambiguity fixed by an explicit `BTreeSet<usize>` annotation; the numerical and
admission algorithms are unchanged. These are inspected coordinator-owned
results, not independently repeated builds. The final strict workspace Clippy
rerun, formatting checks and optimized CLI build also pass; the release log
records successful completion in 3 min 03 s.

## Input, algebra and graph ownership

Serial generation's preparation process calls the existing CLI `input::load`.
Its graph branch still uses `feynkit_graph::FeynmanDiagram::from_dot`, native
`Model`, `ParameterCard`, `Kinematics`, scalar binding resolution and
`GraphIntegral`. The library continues to reexport the actual HEPKit
`FeynmanDiagram` and `IntegralFamily` types. No second DOT parser, momentum
routing model, physical graph owner or general graph canonicalizer appears in
the serial implementation.

The new generation entry point accepts the existing `ParametricIntegrand` and
`GenerationOptions`. Runtime parameters remain Symbolica `Symbol`s and native
mass-constraint Atoms. Geometry, polynomial support, mappings, subtraction,
Laurent construction and evaluator compilation delegate to their existing
owners. The streaming options/map/recipe structures are wire descriptions of
those owners, not replacement algebra implementations. Native integer parsing
restores saved geometry integers; it does not parse symbolic expressions.

Symmetry hashes only choose candidate buckets. Admission calls the existing
native canonical preparation and exact equivalence/permutation verification.
Subtraction-formula reuse reconstructs and compares the native full key before
using the saved recipe. The source/chart/formula codec encodes actual Symbolica
Atoms with their native state context, preserving alias and symbol ownership.

Existing one-loop validation remains in `hepkit_one_loop.rs` and
`hepkit_numerator_reduction.rs`, using OneLOop and native one-loop reduction.
The serial change adds no replacement master integrals or reduction formulas.

## Indexed programs and native APIs

`IndexedWriter` and `IndexedReader` operate on caller-owned `Read`/`Write`/`Seek`
objects. An in-memory `Cursor` remains sufficient for the archive interface.
The catalogue contains layout, identity and inspection descriptors; numerical
programs are restored only by an explicit record/sector/all/exact load.

Sector records contain the existing native evaluator serialization. Restoring
a local record does not regenerate the sector or optimize its expression.
The local-to-global output projection scatters numerical values after native
evaluation, including the existing weighted and precision-rescue paths. It does
not build an evaluator composer merely to pad output vectors. The existing
composer audit explains why composition would repeat optimization on loading.

New functions are additive. Existing `KernelSet::from_bytes*`, generation and
compilation entry points remain present. The byte reader dispatches the indexed
envelope or the existing supported native codec. The isolated Python owner
continues to call those methods and does not acquire a dependency on CLI
configuration, processes or presentation. This source review does not claim
that a new Python serial scheduler or notebook mode has been implemented.

## Execution, errors and presentation

`generation::streaming` exposes synchronous preparation, discovery, exact
comparison, formula construction and representative-sector jobs. The caller
owns the staging directory, job ordering, process lifetime and durability
policy. A `GeneratedUnit` owns only the active representative's native result;
its API documentation requires persisting it before releasing the worker.
Descriptor assembly itself does not restore native expressions.

`integration::serial` owns compact scientific state and bounded reservations.
It owns no evaluator, process, thread pool, filesystem checkpoint or integration
loop. Its jobs reuse the existing QMC/Havana workers through the weighted batch
callback. The caller supplies residence time and resident claims, reserves jobs,
evaluates and submits them, and explicitly chooses when to stop or checkpoint.
The public retry/restore documentation requires the old worker to have stopped.
The CLI, separately, enforces that condition through its process and lock owner.

Generation retains the existing progress/cancellation callbacks and typed
`GenerationError` through `StreamingError`; record I/O and record-admission
errors remain distinct. Indexed operations return `KernelError`. Serial
configuration, return admission, callback evaluation, unavailable uncertainty
and numerical-range errors retain the existing `IntegrationError` boundary.
Missing estimates and numerical failures are not converted into exact zeros.

`SerialSnapshot` and `SerialSectorSnapshot` are owned serializable records.
They distinguish current and previously accepted estimates, allocation/coverage
state, epochs, counts and priorities. The existing `IntegrationObservation`
and `ContributionReport` still carry complete Laurent vectors, covariance and
the explicit independent-across-sectors relation. Process IDs, residence clocks,
load timing and checkpoint presentation belong to the CLI snapshot. Display
code consumes these snapshots without owning the scientific session.

## Numerica and dependency reuse

The only narrow new statistics owner is `ReplicaMoments`. Registry Numerica's
public `StatisticsAccumulator` is scalar; the existing vector reducer requires
the complete replica slice. The missing operation is bounded incremental
cross-component covariance, not an alternative scalar numeric type or sampler.
The helper uses Numerica `DoubleFloat`; its API/source/probe evidence and
independent numerical findings are recorded in
[integration foundations](serial-integration-foundations.md) and the
[independent statistics review](serial-seed-statistics-independent.md).

Native random streams remain Numerica `MonteCarloRng`; Havana grids remain
Numerica `ContinuousGrid`. The added direct `rand = "0.9"` edge supplies the
trait used with that RNG. Numerica 3.0.1 already enables the same dependency's
default features, so this does not introduce a new RNG, package version or
portable entropy requirement. Both leaf lockfile diffs add only the existing
`rand 0.9.5` edge to FastSecDec.

The default numerical core and CLI remain free of PyO3/Python dependencies in
their source manifests. The Python bridge remains isolated in `bindings/python`.
Native and portable backend selection is unchanged; no unconditional native
JIT dependency was added to the core's portable feature. Filesystem-backed
generation requires a usable filesystem. Its existence is not a claim that
browser consumers can spawn processes. Portable host compilation and actual
Wasm execution remain distinct validation claims.

## Recorded limits and disposition

No blocking duplication or native API ownership defect was found in this
review. The independently demonstrated Symbolica global polynomial-resource
retention caveat still applies when native state is imported; it must not be
hidden by claiming that dropping Rust objects releases all native memory. See
the [artifact/memory review](serial-artifact-memory-independent.md) for its
probe, process isolation, measured scope and remaining upstream limitation.

The native, portable-host and binding consumer checks support this review's
acceptance of the public interfaces. The [acceptance matrix](serial-acceptance.md)
records the scientific controls, bounded ggHH demonstration and remaining
delivery checks. This review does not establish browser execution, a rebuilt
Python wheel, every host-global resource history or one-per-mil physical ggHH
convergence. Final executable delivery and publication belong to the coordinator.
