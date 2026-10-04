# Independent weighted-worker review

The coordinator reviewed the additive QMC and Havana worker adapters separately
from their author. Both expose `evaluate_weighted`: the callback receives the
sampling weight and returns the complete vector with that weight already
applied. Existing `evaluate` and `evaluate_with_weight` retain their unweighted
output contract. The worker takes exactly one of these paths; it does not infer
the convention from the numerical values.

QMC still generates each point inside the caller-owned worker and submits the
complete vector to Numerica's existing partial accumulator. The new path omits
only the second multiplication by the periodization Jacobian. Native package
identity, interval validation, complete-shift statistics and covariance remain
the responsibility of the existing Numerica APIs.

Havana production accumulates the supplied vector directly. Pilot training
requires an additional adapter because `ContinuousGrid::add_training_sample`
expects an unweighted scalar and applies the actual `Sample` weight internally.
The adapter divides the weighted absolute envelope by that weight and verifies
a finite, nonzero, accurate round trip. It preserves the original native Sample
and its diagnostic coordinates/weight. It does not change the sampling grid or
implement another training algorithm.

The coordinator requested two corrections before accepting this adapter:

- Report an unrepresentable pilot envelope through the dedicated
  `IntegrationError::UnrepresentablePilotEnvelope`, carrying both the weighted
  envelope and sample weight. Library and future bridge callers need not parse
  a human-readable error message to identify the limitation. Production can
  still accept the same representable weighted vector.
- Validate that the native Havana sample weight is finite and strictly positive
  before invoking the callback. This also rejects an invalid zero-weight grid
  when the integrand envelope happens to be zero.

Both corrections are present in the reviewed source. A worker evaluates each
task on a local grid clone with local accumulators. A failed pilot returns no
training update or numerical contribution, so the calling session cannot
accidentally receive a partially trained grid. Replay-context state has a
separate caller-owned transaction boundary and must be discarded for failed
packages, as described in the kernel replay review.

The initial focused gate passed 14 QMC and eight MC integration tests, including
the new QMC exactly-once callback regression. Three private MC regressions cover
production success versus pilot representability failure, equal native training
and diagnostic metadata for weighted/unweighted callbacks, and invalid grid
weight rejection. All three passed after the final typed-error change within the
25-test library gate recorded in `output/native-numerator-weighted-tests.log`.
The subsequent independent CLI checkpoint audit passed: replay maxima merge
only after successful numerical submission, failed prefixes and rejected
duplicates cannot advance accepted state, and version-three checkpoints validate
policy/identity/layout before resume. All 14 CLI tests passed in
`output/weighted-cli-tests.log`. The new partial-resume regression accepts two
replicas, restores with three workers, and completes new work to 4096 points and
the analytic result `1/2`, retaining diagnostics and maxima.

The broader numerical review also identified two zero-weight risks outside
these adapters: a kernel shortcut could hide a singular endpoint by returning
zero without evaluation, and multiplication of positive periodization Jacobians
could underflow to zero. Both are resolved. A zero weight no longer bypasses
kernel evaluation: the endpoint regression rejects `0 * log(0)` and preserves
accepted replay state. Numerica commits `dc82bd6` and `5507289` add an explicit
range error for interior zero/subnormal attenuation, while retaining genuine
endpoint Jacobian zeros. The conservative attenuation guard can reject a final
product that is representable after amplification; its regression verifies
rejection for either coordinate order and documents this limitation. All 26
native QMC tests passed with serde/bincode enabled. These changes do not add a
second transform arithmetic implementation or hide undefined evaluations.
