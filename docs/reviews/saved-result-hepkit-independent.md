# Saved numerical result: HEPKit and public-API review

The native result and statistical-observation implementation was reviewed
independently of its author. This reviewer owns the thin CLI adapter, so its
process tests are implementation evidence rather than independent review of the
CLI itself. The coordinator's separate [review](saved-result-coordinator-review.md)
covers numeric-range classification and rendering costs.

## Reuse and native boundaries

The existing HEPKit/GammaLoop result containers do not own FastSecDec's Laurent
layout, shared-shift covariance, exact offset and explicit parent-sector scope.
The recurring capability audit recorded in the
[contract](saved-result-contract-proposal.md) therefore supports this small
native data adapter. It does not justify a second estimator, model converter,
graph representation or symbolic evaluator.

The implementation uses the existing native `ContributionReport`,
`VectorEstimate`, `QmcDesign`, reference types and typed stop/uncertainty states.
Reference comparison's prior estimate representation checks were moved to the
integration owner and reused, preserving the original comparison arithmetic.
The result validator does not sum marginal variances, reconstruct totals from
rounded means, calculate new covariance, or certify positive semidefiniteness
or convergence. Original covariance entries and reference evidence survive.

Native results pass directly between library and CLI as Rust objects. JSON is
used at file/checkpoint boundaries, not to translate internal graphs, expressions
or numerical results between owners. Result-only reading, validation, sorting,
display and export use no Atom, graph parse or compiled evaluator. The optional
kernel-manifest constructor reads public metadata only. These types and typed
errors are suitable for later HEPKit bindings without a Python implementation
inside FastSecDec.

## Identity, scope and reference evidence

The parent manifest owns the inner scientific kernel identity, complete ordered
coefficient layout, all stochastic sector IDs/dimensions and folded exact vector.
Selected scope separately declares its sector subset and whether the complete
folded exact offset is included. Full scope requires every parent sector.
Completing a selected allocation cannot establish a full-integral result, and
selected estimates cannot be exported as unscoped full-integral references.
The current CLI constructs full scope only; imported selected results remain
qualified. A caller-supplied manifest is a declaration, not verification of the
hash's authenticity.

Existing CLI artifact/session/checkpoint identity remains unchanged. Saved
reference binding uses the inner kernel identity, while the outer artifact hash
and exact dependency/source provenance are descriptive saved metadata. Effective
native allocations are retained separately from the original CLI settings.

The original `ReferenceResult` and historical `ComparisonContext` are preserved,
including when no computed estimate is available. Export selection is explicit
and has no fallback. An extracted stored target does not inherit compatibility
or independence with a new computation. Computed exports retain validation
status and `StandardError`, including zero; successful saving and a tolerance
stop do not grant independent evidence. The CLI assigns `Unverified` to its own
new computed results.

## Failure and coverage review findings

The native diagnostic observer retains accepted coverage from existing metadata
owners and attempts strict native total and marginal estimation independently.
Range failures yield an explicit unavailable statistic, never zero or merely
waiting for work. A finite shared-shift total can survive unrepresentable
individual marginal covariance; that total remains authoritative. Conversely,
failed total statistics are not reconstructed from marginal values. The ordinary
estimate and contribution APIs keep their strict behavior.

This review identified three draft issues, all corrected by the native owner:

1. A missing total could be labelled waiting even with sufficient selected
   replicas everywhere. The validator now requires genuinely missing coverage
   for that status; unavailable completed statistics need an explicit failure.
2. Point/replica counts could describe impossible coverage. Incomplete replicas
   each require at least one missing point; shared selected-replica counts must
   satisfy the feasible intersection bounds across sectors. These checks do not
   infer missing sample values or add a statistical estimator.
3. Validating design metadata constructed a complete random-shift plan. The
   native `validate_allocation` path now shares the existing rule/capability and
   checked-count validation without allocating or generating all random shifts.

The coordinator additionally required numeric-range-only failure capture and
borrowed/indexed display. Structural state errors still propagate. These changes
were inspected in the final source. The native metadata-only Numerica coverage
accessor is the appropriate owner; FastSecDec does not duplicate its private
per-shift accumulation state.

The CLI finalizes returned accepted state, persists native observations and
prints a nonzero-exit numerical-failure report. Worker failures discard local
prefixes, while a statistical failure can retain complete accepted replicas.
Cancellation/pilot state remains explicit. Setup, structural validation and
filesystem errors do not create fictitious numerical results. A valid total
with unavailable marginals is not misclassified as whole-run failure.

## Validation evidence

The native author initially ran 24 tests covering saved documents, statistical
observation, existing reference behavior and sector contributions. Later focused
coverage includes the review findings above. CLI execution evidence is appended
after its separate process gate; a source review alone is not an end-to-end
claim.

The focused CLI gate subsequently passed 26 tests: 15 unit tests, two catalogue
process tests, four existing CLI processes, two reference processes and three
new saved-result processes (`output/cli-saved-result-tests.log`). The actual
complex run preserves all four real/imaginary covariance entries and native
sampling design, then remains readable in a fresh process after its input,
artifact, checkpoint and target files are deleted. Explicit exports preserve the
distinct estimate and original target. Selected scope, zero reported error,
unavailable reference selection and malformed version handling are exercised.

The failure process test covers both a nonrepresentable point evaluation and
finite accepted values whose covariance overflows. Both produce one final JSON
report, a native readable saved record and a nonzero exit; the former excludes
its failed package prefix, while the latter retains accepted complete replicas.
The existing real Ctrl-C process now also validates the saved cancelled result
and its rejection for estimate-reference export. The native saved-result target
then passed all ten tests, including actual pilot/partial observations and
selected exact-offset exclusion (`output/saved-result-native-followup-tests.log`).
All 24 native run cards continue to load in the CLI gate. No new scientific
reference-validation status is inferred from any of these transport checks.
