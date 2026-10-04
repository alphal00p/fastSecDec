# Native saved integration results: proposal

This is a design proposal only. No result reader, writer, converter or CLI
command is implemented by this document.

## Full-integral and selected-sector scope

The saved numerical scope must be explicit. A caller can construct an
`IntegrationProblem` containing only selected sectors, so complete allocation
and valid covariance do not establish coverage of the whole kernel. Store a
typed full-integral or selected-sector scope, bound to the parent kernel
identity, its full sector manifest and the selected sector identities, with
the exact-contribution policy recorded. Validate a claimed full scope against
that manifest. A selected-sector result may be displayed and exported with
its restricted scope, but must never become full-integral convergence or a
full-integral reference merely because its own work allocation completed.
Reference extraction and comparison require compatible explicit scopes;
missing scope in a legacy document cannot silently mean full integral. This
retains the scientific requirement of regression-matrix test 6376 alongside
the saved-result cases below.

The first implementation should reject estimate-to-reference extraction for
selected-sector results while the existing reference API has no typed scope.
They remain readable and displayable. Freeform provenance is insufficient to
prevent an accidental full-integral comparison; supporting scoped references
can be a later deliberate extension without migrating all reference documents
in this slice.

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

## Concrete implementation sequence after the catalogue milestone

Keep the library slice in `fastsecdec::results` with separate `types`,
`manifest`, `validation`, `document`, `reference` and `display` modules. No
source changes are authorized by this section during the current source
freeze; it specifies the next accepted implementation slice.

The owned native types should include:

```text
KernelResultManifest {
    kernel_content_id,
    orders, components,
    sectors: Vec<SectorSpec>,
    exact_coefficients,
}
ResultScope = FullIntegral
            | SelectedSectors { sector_ids, exact_policy }
ExactContributionPolicy = IncludeAll | ExcludeAll
SavedIntegrationResult {
    manifest: KernelResultManifest,
    scope: ResultScope,
    contributions: ContributionReport,
    stopping_reason: StoppingReason,
    requested_tolerance: Option<Tolerance>,
    evaluation_diagnostics: Option<EvaluationDiagnostics>,
    qmc_design: Option<QmcDesign>,
    provenance: ReferenceProvenance,
    validation: ReferenceValidation,
    stored_reference: Option<StoredReference>,
    timings: optional descriptive elapsed/load/generation timings,
}
StoredReference { reference: ReferenceResult, context: ComparisonContext }
```

The manifest, scope and exact policy are native saved-document fields, never
CLI-only metadata. `FullIntegral` selects the complete stochastic manifest and
all exact coefficients. A selected scope names only stochastic sectors and
explicitly includes or excludes the entire folded exact vector; it does not
pretend to recover individually folded analytic sectors. Missing scope is an
error. Empty selected stochastic support is still selected scope and must not
be upgraded automatically to full scope.

Provide an optional manifest constructor from public `KernelSet` metadata
(identity, layout, dimensions and exact vector), plus a native-data constructor
for callers that own equivalent metadata. The latter is an explicit declaration
of the complete parent manifest, not proof that a caller-supplied hash is true.
Reading, validating and displaying an existing document must not initialize
Symbolica, touch an input graph or load a compiled evaluator. The numerical
kernel identity remains distinct from a CLI artifact's source/provenance hash.

Implement validation in this order:

1. Extract the existing estimate representation checks from `reference::compare`
   into the integration owner's reusable `VectorEstimate::validate` and make
   comparison use that method. Factor reference/context metadata and identity
   checks into reusable methods so stored references can be checked even when
   the computed total is absent. Preserve current comparison arithmetic and
   eligibility decisions; do not construct a dummy estimate to trigger them.
2. Validate manifest layout, finite exact values, nonempty identity, unique
   stochastic IDs and positive dimensions. Validate selected IDs as a subset.
   Require contribution rows to match exactly the selected manifest IDs and
   dimensions, and require the same full coefficient layout. Exact offsets
   must match the selected include/exclude policy.
3. Validate coverage and states from existing native fields: used counts cannot
   exceed completed/planned counts; replica and point totals must agree with
   the recorded design; pilots cannot contain production estimates; shared
   democratic sectors must use the same complete common replica count.
   `production_complete` must agree with complete support in the saved scope.
   Exact-only status is valid only with empty stochastic support. Preserve
   authoritative totals and all covariance entries without recomputing them
   from rounded marginal values or constructing a covariance model.
4. If a `QmcDesign` is present, validate its settings and selected allocation
   identities/counts against the numerical payload. It is observational design
   metadata, not an accumulation checkpoint. Keep cumulative precision counters
   distinct from accepted production coverage: failed discarded attempts do not
   automatically invalidate a later completed estimate.
5. Validate typed stopping claims. `TargetReached` requires recorded tolerance
   and the existing native `VectorEstimate::meets` check. `PlannedWorkComplete`
   requires complete production support within the explicit scope. Work/time
   limits may accompany incomplete or complete data, but never fabricate
   convergence. Cancellation and numerical failure remain visible and prohibit
   estimate-to-reference extraction even if some valid rows are retained.

`read_result` and `encode_result` use the strict version-one envelope and call
the same validation. Do not add legacy CLI-output guessing. New document-owned
types reject unknown fields; existing nested native types retain their defined
contracts. JSON float round trips use the existing workspace `float_roundtrip`
feature. Imported malformed layouts must produce typed errors, not formatting
panics or silently truncated output.

Expose explicit estimate/stored-reference extraction and selected-sector
rejection as typed errors suitable for a future Python bridge. Stored-reference
extraction preserves the original object even when the computed run failed.
Reusing that extracted object in a new comparison requires a fresh caller
`ComparisonContext`; its historical independence assertion is not transferred.
Full-scope complete estimates preserve their recorded `Unverified` or `Checked`
evidence, and every exported error remains `StandardError`, including zero.

Use native sorting of references to validated contribution rows for result-only
views: ID order, absolute value of an explicit coefficient key, or that key's
standard error. Sort missing estimates last and use sector ID for ties; never
fill an absent coefficient. Scope and exact-offset policy must be visible in
every rendered result, and marginal errors remain labelled as correlated when
the replica relation requires it.

The initial pure-data tests should include a partial one-sector allocation
that is internally complete but selected from a two-sector manifest, proving
that it stays viewable yet cannot become a full-integral reference. Add a
scope/manifest mismatch, a full versus selected exact-offset-policy case,
real/imaginary covariance round trips, all explicit reference-source choices,
and malformed count/layout/stop claims. These test scientific boundaries rather
than JSON field spelling. A thin subsequent CLI adapter can save this native
document and add `show-result`/explicit `export-reference`; its fresh-process
test must still work after removing graph and kernel files.

## Failure-safe observation and CLI handoff

The existing strict `snapshot()` and `contributions()` methods can themselves
fail when finite accepted replicas produce a mean or covariance outside binary64
range. For example, replica means `+1e200` and `-1e200` have finite observations
but overflow the native second-moment calculation. A CLI failure finalizer that
simply calls `contributions()?` would lose precisely the state it needs to save.

Add a narrow diagnostic contribution accessor for QMC and Havana. It retains
accepted coverage directly from the existing accumulator/record metadata,
attempts the existing native total and marginal estimators independently, and
keeps every estimate that those estimators can represent. Unavailable numerical
statistics receive a typed statistical-failure status and no estimate; they are
never assigned synthetic zero, `Available`, `Exact`, or merely waiting-for-work
status. Ordinary `estimate()` and `contributions()` retain their strict behavior.
The QMC metadata accessor should live in Numerica and reuse stored per-shift
counts, rather than make FastSecDec duplicate coverage reconstruction or perform
an alternative summation. No arithmetic fallback or new statistical estimator
belongs in this reporting path.

The saved validator must allow structurally consistent completed-but-unestimable
records. A valid total may coexist with an unavailable marginal (or conversely),
so do not erase independent valid fields solely because another estimator
failed. The authoritative total continues to govern numerical comparisons;
`StoppingReason::NumericalFailure` blocks estimate-reference extraction even
when some estimates survived. A retained original stored reference remains
selectable. This behavior needs a native numeric regression with accepted
large finite replicas, as well as the ordinary worker-evaluation failure test.

The CLI should finalize actual accepted state, save/display the typed failed
result, and return a nonzero exit status. Setup, validation and filesystem errors
remain ordinary errors without a fabricated numerical record. MC pilot failure
retains pilot-only coverage and caller-side restart-required information. Result
scope binds to the inner `KernelSet::content_id()`; the existing outer artifact
identity continues to isolate checkpoints and appears only as result provenance.
No checkpoint identity migration is needed.

For view coordination, the native result API will provide `Display` and
`sector_order(ResultSectorSort::{Id, Magnitude(CoefficientKey),
StandardError(CoefficientKey)})`. Numeric ordering is descending, absent estimates
are last, and sector IDs break ties. Unknown coefficient keys are typed errors.
The returned ordering does not mutate the saved payload; CLI JSON views may
retain the original result and separately label derived comparison and order.
