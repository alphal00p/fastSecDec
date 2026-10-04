# Independent saved-result CLI review

The native result author reviewed the CLI adapter independently of its author.
This review covers `results/{mod,storage,display}.rs`, the QMC/MC execution and
final report paths, command dispatch, retained references and process tests.
Native mathematical validation is covered by the separate HEPKit and
coordinator reviews; it is not independently certified by this CLI review.

## Resolved finding

The first view implementation called the native comparison before rendering
either JSON or plain output. Finite accepted coefficients and a finite reference
can overflow only during derived subtraction or pull calculation. Propagating
that error prevented viewing an otherwise valid saved observation. The native
adapter now converts only `ReferenceError::NumericRange` into a typed unavailable
comparison, retaining the coefficient key and quantity. The original numerical
record and reference remain unchanged and selectable. A focused pure-data test
uses opposite finite `1e308` source values and verifies read, view and original
reference export. Structural reference/context errors still propagate.

No other actionable CLI finding remains in the reviewed source snapshot.

## Boundaries checked

- Result-only commands dispatch directly to native numerical readers, ordering,
  display and explicit reference extraction. They do not load graphs, artifacts,
  evaluators or integration dashboards. The only process bootstrap sets the
  documented Symbolica banner display variable, without initializing Symbolica.
- The complete native manifest uses the inner kernel content identity. The outer
  artifact identity remains checkpoint isolation and descriptive provenance.
  Saving does not migrate or relabel checkpoint identities.
- The final report retains native authoritative total covariance and accepted
  sector contributions. CLI code constructs no estimate, error or covariance.
  Native diagnostic observations retain valid totals when only a correlated
  marginal exceeds numerical range; CLI failure classification examines the
  authoritative total, not the marginal display status.
- Worker return processing still submits every successful package before
  finalizing a failure. Rejected prefixes do not enter accepted coverage or
  replay maxima. Numerical failure saves/displays the typed observation and
  returns a nonzero exit, with the existing reported-failure sentinel preventing
  a second JSON object. Setup and I/O errors remain ordinary errors.
- MC pilot failures retain pilot-only observations and the existing caller-side
  restart-required status; they do not manufacture a production checkpoint.
  Cancellation retains accepted observations and blocks estimate-reference
  extraction through native status/scope validation.
- Stored references and historical contexts are copied from the original typed
  inputs. Neither is reconstructed from rendered comparison rows. Extraction
  requires an explicit source; computed results stay unverified, including
  zero standard errors. Selected-sector records remain selected on viewing and
  cannot be exported as full-integral estimates.
- Save destinations are checked against inputs, artifacts, checkpoints and the
  retained reference path. Atomic output writes use the existing helper. Native
  transport is distinguished from a JSON view containing derived ordering and
  comparison information.

## Evidence and limits

The CLI author's focused gate passed 26 tests: 15 unit, two catalogue process,
four existing CLI process, two reference process and three saved-result process
tests (`output/cli-saved-result-tests.log`). It covers an actual complex QMC run,
viewing after deleting graph/artifact/checkpoint/reference files, explicit target
versus estimate extraction, selected-scope rejection, zero-error meaning,
worker failure, accepted-statistics overflow and real SIGINT result retention.
The numerical-failure process parses stdout as one JSON value and checks a
nonzero exit. The native saved-result follow-up passed ten tests before the last
derived-range regression was added; the final workspace gate records that
additional test and the complete frozen snapshot.

This review does not establish performance acceptance for difficult scientific
integrals, prove covariance positive semidefiniteness, or independently validate
caller-provided provenance. It establishes the adapter's preservation and
process boundaries; the numerical estimators remain owned by Numerica/Havana.
