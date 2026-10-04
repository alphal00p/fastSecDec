# Independent review: qualified sector selection and retained metadata

This review is independent of the implementation author. It covers the native
projection API, CLI selection/checkpoint wiring and inspection of retained
metadata. The final source review, independent native projection tests and
author's focused native/CLI gates are complete. No blocking finding remains.
The coordinator's combined workspace gate is separate from this scoped review.

## Native scope boundary

`KernelResultManifest::canonical_scope` reuses the complete manifest validator,
rejects duplicate and unknown IDs, and canonicalizes selected IDs without
promoting an explicit empty or all-sector selection to `FullIntegral`.
`integration_problem` preserves the parent's stable sector IDs and dimensions,
full coefficient/component layout, and declared `IncludeAll`/`ExcludeAll` exact
offset. It delegates numerical problem validation to `IntegrationProblem`.
Saved-result validation now uses that same projection instead of maintaining a
second subset/exact-policy implementation. No new statistics or algebra is added.

The inner kernel identity remains the scientific parent identity; the outer
artifact identity remains the accumulation/checkpoint identity. An explicit
all-sector selection can produce the same numerical problem as full scope, so
the separate scope declaration must remain bound by CLI settings and preserved
in every report. The review requested an executable selected-all versus full
checkpoint mismatch check for this otherwise easy-to-miss case.

## CLI source boundary

The current source requires an explicit exact-contribution policy with a CLI
selection, canonicalizes through the native manifest, and serializes selected
scope in integration settings. Absent historical scope still defaults to full
scope, which is omitted on serialization; this preserves historical full-scope
checkpoint settings identity. Worker count remains the only intentionally
mutable resume setting.

Worker contexts are maps keyed by original kernel ID and are constructed only
for projected sectors. Accepted replay metadata retains the full parent-sized
version-three layout, preserving existing checkpoint semantics and original IDs.
Both QMC and Havana numerical callbacks continue to use their existing workers,
weighted replay and accepted-package transaction. They do not sum estimates or
create a second error calculation in the CLI.

Artifact loading still validates and compiles the complete kernel set before
selection. This slice narrows worker-context cloning and actual integration
work; it does not introduce lazy artifact compilation. Invalid resume settings
are rejected before any new numerical work package is evaluated, rather than
before the existing artifact-loading step.

Live status is qualified by scope; the final report distinguishes scoped target
completion from unqualified full-integral convergence. Saved-result scope is
retained, and comparison/reference extraction still delegates to the native
selected-scope rejection. Completing all work in a selected allocation is not
evidence of a full-integral estimate. Empty selections retain their explicit
exact-offset policy and are not silently reinterpreted as all sectors.

## Retained metadata boundary

The exported `PortableMetadata` is the existing artifact transport, constructed
from validated native `GenerationMetadata`. The borrowed `MetadataView` reads
retained domain certificates, chart/representative associations, coordinate
maps, determinant and measure powers; it performs no new geometry or polynomial
support calculation. Legacy absence is reported explicitly rather than replaced
by invented metadata. The display also explains that a chart without a kernel
can be exact, cancelled or truncated, and that support valuation indices are not
unique named U/F-factor IDs.

The human scope ID preview is now bounded to six IDs plus a total count, following
the review suggestion. The complete typed ID list remains available in JSON.

## Executed independent evidence

The already-built `selected_scope` target was independently rerun without Cargo,
Symbolica initialization or symbolic-runtime contention: **two tests passed** in
0.00 seconds. The log is `output/selected-scope-independent-tests.log`. The
scientific test uses original IDs 7 and 99 from an unsorted complete manifest,
oppositely varying shared-shift contributions, and both exact-offset policies.
It verifies a nonzero marginal uncertainty with an exactly cancelling total,
as well as retained IDs and selected-only coverage. The other test checks
duplicate/unknown IDs and explicit empty/all selections. No alternate numerical
estimator was used in the review.

## Author gate and review closure

The authored CLI tests now cover original-ID selection, include/exclude exact
offsets, explicit empty/all scopes, selected QMC/Havana execution, partial
completion and resume with a changed worker count, scope/exact-policy mismatch
rejection, old absent-field identity, and rejection of selected estimates as
full references. Nonempty MC/adaptive-MC selection was added after the review
identified that empty-selection tests do not execute the changed callback maps.
The selected-all versus full checkpoint distinction is explicitly tested even
though their projected numerical payloads can coincide. Metadata tests compare
the existing native portable record. A valid version-one artifact exercises
explicit legacy unavailability after native loading. The final `--full-integral`
override clears a card/stored subset without editing input bytes, preserves
full-scope checkpoint semantics, and rejects conflicting selection flags.
The native estimator/geometry tests need not be duplicated by implementation
mirror tests in this adapter review.

The author gates passed **31 CLI tests** (18 unit plus 13 process tests) and
**17 native tests** (11 saved-result, two selected-scope, four metadata tests).
Evidence is in `output/selected-scope-cli-tests.log`,
`output/selected-scope-process-tests.log` and
`output/selected-scope-native-regression.log`. The first log also retains the
initial process-fixture failure: the author had assumed a constant term inside
a stochastic chart would become a separately folded exact sector. The corrected
fixture uses a genuine zero-dimensional artifact for the exact-only CLI check;
no production semantics were changed to satisfy that assumption. The separate
native covariance/projection test covers selected stochastic rows together with
a nonzero exact offset. Final source and corrected test evidence were reviewed.
