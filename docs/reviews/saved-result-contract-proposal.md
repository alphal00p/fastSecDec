# Native saved integration results: proposal

This is a design proposal only. No result reader, writer, converter or CLI
command is implemented by this document.

## Required scientific behavior

Pathfinder tests `test_integrals.py:6401,6481,6524,6601,6673` require numerical
results to round-trip, permit sector sorting and inspection without regeneration,
retain a stored comparison target distinctly from the computed estimate, and
prevent sparse/unreliable estimates becoming benchmark targets. The old implicit
preference for a nested pySecDec/numeric target should become an explicit source
selection, rather than tie a Rust result format to those Python nesting rules.
A stored result may be useful to inspect even when incomplete, cancelled or
failed; inspectability must not imply eligibility as reference evidence.

## Existing owners and boundaries

`integration::{VectorEstimate,ContributionReport,SectorContribution}` already
carry the full real/imaginary Laurent layout, covariance, exact offsets, coverage,
replica dependence and authoritative total. `status::StoppingReason` and
`EvaluationDiagnostics` carry typed execution outcomes and precision diagnostics.
No new accumulator or result-estimation algorithm is necessary.

The reference module owns sparse key alignment, uncertainty meanings, provenance,
validation evidence, normalization/kinematic compatibility, independence, identity
checks and comparison eligibility. Its versioned document reader/writer provide
the envelope precedent. Its estimate checks should be factored into a reusable
native validation method when implementing this slice, rather than copied or
replaced by a second statistical validator.

The current CLI `driver/report.rs::IntegrationReport` is Serialize-only and
mixes numerical fields with checkpoint `ResumeStatus`, elapsed/load timings,
string stopping prose and a duplicate estimate. `reference.rs::ReferenceReport`
contains the original `ReferenceResult` only while waiting for coverage; the
compared form contains derived rows instead. A native saved result should retain
the original reference object/context explicitly in either case, then recompute
comparison rows using `reference::compare` when requested. This avoids importing
rounded rendered data or reconstructing a target from a comparison table.

GammaLoop's native `settings/runtime.rs::IntegrationResult` is a serde process/
integrand result containing scalar real/imaginary accumulators and adaptive-grid
breakdowns. It supplies useful presentation precedent but neither this Laurent
covariance nor common-shift coverage. HEPKit's current bridge exposes no matching
saved FastSecDec result primitive. Native typed containers and existing
serialization/comparison APIs are therefore the appropriate capability owners;
there is no algebra, graph or JIT operation in the proposed result interface.

## Proposed native document and API

Use one strict version-one envelope:

```text
{ "format": "fastsecdec-integration-result", "version": 1,
  "result": <SavedIntegrationResult> }
```

Reject unsupported/missing versions, mixed format discriminators and unknown
owned envelope/document fields. Existing nested native containers are validated
according to their structural contracts; do not call mere deserialization
scientific validation. Legacy unversioned CLI output is not silently interpreted
as version one or assigned missing covariance/coverage.

`SavedIntegrationResult` contains:

- nonempty kernel content identity and native `ReferenceProvenance` describing
  the computation and normalization convention, with optional source/revision;
- one `ContributionReport` as the numerical payload, including its authoritative
  total and separately retained exact vector;
- typed stopping reason and optional cumulative evaluation diagnostics;
- `ReferenceValidation` for independently recorded evidence about the estimate,
  defaulting to `Unverified` and never inferred from a convergence flag;
- an optional `StoredReference { reference: ReferenceResult,
  context: ComparisonContext }`, retained without changing its evidence;
- optional descriptive timings, using existing timing types where applicable.

Checkpoint path, file-save success, pilot-restart instructions and worker-pool
configuration remain caller/CLI envelope metadata. They are not numerical
semantics and must not decide whether a coefficient vector is trustworthy.
No Atom, native graph object or compiled kernel is required to construct, read,
validate, compare or display a saved numerical result.

Suggested methods/functions are `encode_result`, `read_result`,
`SavedIntegrationResult::validate`, `comparison` and
`reference(selection: ResultReferenceSelection)`. The explicit selection enum is
`Estimate` or `StoredReference`; absence of the selected object is a typed error,
never an automatic fallback. Native callers may construct/pass the typed value
without a JSON round trip.

Structural validation reuses estimate/reference checks, then checks layout and
identity agreement, unique sector IDs, finite nonnegative timing/count data,
used-versus-completed/planned coverage, method/replica relation, pilot exclusion,
and exact-only status consistency. It must not recompute authoritative totals
from rounded marginal means or sum shared-sector marginal variances. Covariance
shape/finiteness validation is not a new proof of positive semidefiniteness or
statistical convergence; no matrix/statistics engine is introduced.

## Reference extraction and evidence

`StoredReference` extraction returns the retained native reference unchanged,
including unknown errors, validation status, kernel identity and provenance. A
failed computed estimate does not invalidate an independently stored reference.
It also does not upgrade an unverified stored reference. The associated prior
comparison context is historical context only: independence and compatibility
with a new computation must be supplied anew to the existing comparison API.

`Estimate` extraction requires production data, a present native estimate,
complete production allocation and full replica coverage. Cancellation,
numerical failure, pilot-only or missing coverage yields a typed rejection.
A work-limit stop after an entirely completed allocation can retain a statistically
well-defined estimate even if a requested tolerance was not reached; the result
must state that outcome rather than equate convergence with correctness.

Conversion must preserve evidence, not manufacture it. An accepted complete
estimate becomes a `ReferenceResult` whose validation is the saved validation;
`Unverified` remains ineligible under the existing comparison contract. No
command-line trust switch or automatic `Checked` assignment is proposed. If the
coordinator prefers reference extraction itself to reject `Unverified`, that is
a stricter policy boundary; either way it cannot become eligible evidence merely
by saving and reloading. Independently recorded `Checked { evidence }` remains a
caller assertion, as in the existing reference API.

Each converted error is `StandardError(sigma)`, including zero. Neither a zero
sample error nor an analytically evaluated floating-point coefficient implies
`ReferenceUncertainty::Exact`. Kernel identity remains attached and cannot be
silently overridden by compatibility prose. The full covariance stays in the
saved result; scalar reference comparisons retain their existing documented
scope. No missing Laurent/imaginary coefficient is filled with zero.

## Result-only viewing and tests

A thin CLI `show-result FILE` should dispatch directly to the numerical reader,
without validating an input graph, loading an artifact or compiling a kernel.
Sector sorting derives keys from validated native rows, with deterministic ID
ties and absent estimates placed last. Support ID order and magnitude/error of a
selected coefficient key (or clearly labeled maximum component magnitude).
Display exact offsets separately and label correlated marginal errors. Derived
comparison tables use the library comparison function; old stored table rows
are never treated as primary scientific data.

An explicit `export-reference FILE --source estimate|stored` is preferable to
accepting a saved result ambiguously as an ordinary `--reference` file. It can
emit the already supported native reference envelope. The existing reference
reader need not guess between computed and nested target values.

Meaningful validation for the eventual slice:

1. Native result round-trip preserves vector covariance, coverage, replica
   dependence, exact offsets, diagnostics, identity and reference evidence.
2. A record with visibly different computed/stored-reference coefficients proves
   both explicit selections, absent-selection errors and no fallback.
3. Pilot, partial, cancelled and failed records remain viewable but cannot be
   converted into estimate references; a complete unverified result never gains
   `Checked` status or comparison eligibility after round-trip.
4. Zero estimated error remains `StandardError(0)`, unknown stored reference
   errors remain unknown, and mismatched identities remain hard errors.
5. Result-only CLI viewing works after removing input/artifact files, and rejects
   unsupported versions or malformed numeric layouts without initializing a
   symbolic evaluator. Sorting handles absent estimates without fabricated zeros.

The first implementation should remain a thin native data/validation adapter;
CLI convenience and historical format migration can be reviewed separately.
