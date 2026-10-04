# Saved-result observation and dependency review

This review is independent of the native saved-result author. The separate
HEPKit review covers the broader public interface and CLI adaptation. This
record focuses on accepted coverage, numerical failure, source identity and
presentation cost. The combined workspace gate is recorded at the milestone.

## Native coverage accessor

Numerica's new `QmcAccumulator::complete_shift_ids` is metadata-only. It shares
the existing per-shift count aggregation with `shift_estimates`, without
merging floating-point sums or evaluating covariance. This makes completed
coverage observable when accepted finite observations overflow during later
statistical reduction.

The accumulator admits only complete partial packages. `merge` validates the
package against the plan, validates count and sum buffer shapes against the
package interval, and rejects overlapping accepted intervals. Each shift count
is therefore bounded by that shift's lattice size; the checked total plan size
also bounds global coverage. Deserialization goes through the same constructor
and merge checks. The accessor introduces no alternative coverage arithmetic,
estimator, summation method, or execution loop. Its shared helper leaves the
order of numerical accumulation unchanged.

The focused native regression accepts large finite one-point packages out of
order, retains a gap in one shift, and checks coverage both before and after
filling the gap while ordinary numerical reduction fails. This is a meaningful
reason for the small dependency extension, rather than duplicating Numerica's
private coverage state in FastSecDec. The extension belongs on the existing
Numerica feature branch. It initially remained local; the user's later
publication authorization and upstream PR are recorded below.

Numerica commit `e4638da22a17cfa931fa14c6829d3350b7a8de2b` passes all 30 QMC
tests and nine existing MC tests. Evidence is retained locally in
`output/numerica-coverage-tests.log` and
`output/numerica-coverage-mc-tests.log`. The reference checkout was clean at
this review. After the user's subsequent publication authorization and full
readiness gates, the feature was published as
[Numerica PR #8](https://github.com/symbolica-dev/numerica/pull/8).

## Failure and scope boundaries

The ordinary `estimate` and `contributions` interfaces remain strict. The new
diagnostic observer attempts the existing total and marginal estimators
independently. Shared-shift cancellation can leave a representable total while
individual marginal covariance overflows. That valid authoritative total must
survive; the unavailable marginal is explicitly marked and is not assigned a
zero estimate or merely a waiting-for-work status. Conversely, a failed total
must not be reconstructed from rounded marginal means or variances.

Review identified that the first observer draft converted all integration
errors into statistical failures. The implemented correction is a narrow typed
numeric-range classification: structural errors still propagate. A focused
regression distinguishes an output-layout error from a numeric-range failure. This
keeps a malformed state distinguishable from an unavailable floating-point
statistic, without adding another statistics engine.

The parent kernel manifest binds the full coefficient layout, stochastic IDs,
dimensions and folded exact vector. Selected-sector scope separately declares
its IDs and exact-offset policy. Completing a selected allocation does not
make it a full integral or permit full-integral reference extraction. The
manifest's scientific kernel identity remains distinct from the CLI artifact
identity used to isolate checkpoints; neither identity is migrated by saving.

Stored-reference extraction is explicit and preserves the original reference.
Computed-estimate extraction preserves the saved evidence and standard-error
meaning, including zero standard error; it never upgrades an unverified result.
Cancellation, numerical failure and incomplete production prohibit extracting
the computed result. Reading and displaying a document uses numerical metadata
only and does not require the original graph, artifact or Symbolica evaluator.
Structural validation is not an independent convergence or covariance
positive-semidefiniteness certificate.

## Presentation cost

The first display draft cloned every contribution, including its covariance,
and repeatedly searched the requested ID ordering while sorting. The corrected
implementation uses a borrowed row renderer and indexed ID lookup. A hard-case report
may contain thousands of sectors; displaying already accepted data should not
copy every covariance matrix or add repeated linear searches. The view order
must remain separate from the saved payload and its statistics.

The independent HEPKit reviewer also identified coverage-state combinations
that were too permissive and a metadata-validation path that constructed a
sampling plan. Those findings are handled in that review: saved metadata must
not allocate all random shifts merely to validate a declared design.

The independent CLI review additionally found that finite saved coefficients
and a finite stored target can overflow only when their comparison is derived.
Such a comparison failure must not prevent reading the accepted numerical
record. The native saved-result comparison adapter records the existing
`ReferenceError::NumericRange` key and quantity as an unavailable-comparison
reason; it preserves both source objects and does not replace the comparison
arithmetic. Malformed reference metadata and incompatible identities still
produce their ordinary validation errors.
