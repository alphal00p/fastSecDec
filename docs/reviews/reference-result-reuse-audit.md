# Reference-result API reuse audit

Read-only audit, 2026-10-04. This proposes a library adapter; it does not add a
new algebra, uncertainty estimator, or integration stopping rule.

## Existing capabilities checked

Three checks were made: public exports and types, implementation source, and
actual consumer paths/tests.

- Numerica `numerical_integration::StatisticsAccumulator` owns sample/iteration
  means, standard errors, chi-square, and scalar uncertainty formatting. Its
  public `get_live_estimate` and `format_uncertainty` concern one accumulator;
  they neither align Laurent/component keys nor compare external references.
  The QMC `QmcEstimate` owns vector covariance from independent shifts. Existing
  FastSecDec runtime adapters already reuse these accumulators and estimates.
- Symbolica owns exact expression evaluation and Laurent series. HEPKit's
  `src/oneloop.rs` delegates to the native one-loop master/reducer packages;
  the new native master and numerator tests demonstrate numerical reference
  composition without a second evaluator. FeynKit graph/kinematics/tensor and
  Linnet public modules do not expose a Laurent uncertainty-comparison API.
- GammaLoop does contain target comparison presentation in
  `gammalooprs/src/integrate/status_update.rs`, including
  `format_delta_fields_from_estimate`. It is private, consumes GammaLoop's
  scalar wrapper/status model, and assumes an exact target without target
  uncertainty or provenance. Its zero-error and zero-target display branches
  return zero deltas, which would be inappropriate for certification here.
  `IntegralResult` is a table of formatted strings, while public `StatusUpdate`
  is tied to GammaLoop slots and integration state. Their consumers confirm
  these are application presentation paths, not a reusable comparison primitive.

No suitable existing public primitive was found. A small FastSecDec adapter
over its existing `VectorEstimate` is justified. It should not depend on all of
GammaLoop or copy its display logic. Arithmetic is scalar difference and
independent-error combination, not a computer-algebra feature.

## Recommended library contract

Use a `reference` module with typed, serde-compatible reference coefficients,
provenance and comparison rows. Reuse `CoefficientComponent` and align by the
unique pair `(epsilon_order, component)`, never by vector position. A reference
coefficient carries a finite central value and explicit uncertainty state:
`Exact`, `StandardError(f64)`, or `Unknown`. Exactness is declared explicitly,
not inferred from a missing error, a printed zero, or an error rounded to zero.

Reference provenance retains source/revision/engine/convention and a typed
validation level. Normalization and kinematic compatibility must be explicit
caller assertions or checked identities; the adapter must not silently convert
conventions or construct new symbolic normalization formulas. Kernel content
identity can be checked when recorded, but historical references without it
remain useful as visibly unverified comparisons.

The comparison function takes an existing `VectorEstimate`, a reference, and
compatibility/independence policy. It returns one row for every requested key,
including explicit missing-estimate and missing-reference cases, plus unused
reference keys. Missing Laurent orders are not silently filled with zero.
Each matched row contains signed difference, optional relative difference
(undefined at a zero reference), and the estimate/reference uncertainty status.
For declared independent, known uncertainties use `hypot(estimate_error,
reference_error)` and a signed component pull. Unknown reference uncertainty
means no combined-error pull; a difference is still useful. Zero combined error
requires an explicit exact-agreement/exact-disagreement status instead of an
infinite or misleading zero JSON number. Validate dimensions, unique keys,
finite values and nonnegative errors before comparison.

Keep numeric comparison distinct from scientific certification. A completed
production estimate, verified compatible convention/identity, independent error
assumption, and validated reference are prerequisites for a certified result.
An unverified but reported error can yield a clearly labelled diagnostic pull;
unknown error cannot. Comparison must not change integration stopping decisions.

Per-component pulls are sufficient initially. A multivariate chi-square would
need complete reference covariance and cross-estimate correlation assumptions;
do not invent diagonal covariance or add matrix inversion/regularization now.

## Historical files and CLI boundary

The version-one files in `examples/targets` should be parsed by a library reader
into that typed model. The CLI supplies the path and renders returned rows.
Future HEPKit callers can construct the same reference directly from native
master results without any JSON serialization detour.

- `double_box.json` and `issue_1.json` have `standard_errors: null`; retain
  unknown errors. Their listed zeros and rounded decimal values do not establish
  exact coefficients or statistical certification.
- `four_loop_hard.json` records errors but explicitly says they and its numerical
  stability are not independently certified. Retain that qualification, including
  its leading zero placeholders and rounded-report provenance.
- `box.json` and `triangle.json` record external engine errors and source
  provenance; their historical status must not be upgraded just because the
  file is parseable. Independent native checks may be recorded separately.

Tests should cover shuffled/repeated-order real/imag keys, missing/extra targets,
duplicate keys, unknown versus exact errors, zero targets, zero combined errors,
large finite errors without overflow, unfinished estimates, mismatched identity
or normalization, unverified references, and the actual historical null-error
files. No terminal cosmetic tests or replacement numerical integrator is needed.

## Status

Proposal only. Implementation remains queued behind weighted-evaluation
correctness and checkpoint persistence. No runtime or benchmark was necessary
for this API/source/call-site audit.
