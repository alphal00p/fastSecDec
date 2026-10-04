# Independent weighted-sample replay review

Started 2026-10-04 after milestone `4b493bc` (147 workspace tests, formatting,
and Clippy passed). The initial `kernel/weighted.rs` was a disconnected draft;
its illustrative tests under ignored `output/probes/weighted_replay.rs` were not
yet executable. This report distinguishes source review from validation of the
subsequent implementation.

## Numerical finding requiring correction

The draft replays an unweighted coefficient vector in MPFR, converts each
coefficient to binary64, then multiplies by the sample weight. This can silently
lose a representable weighted contribution before the weight is applied:

- For `f(x)=x^2`, `x=1e-200`, and weight `1e300`, the unweighted coefficient is
  about `1e-400` and rounds to zero. Its weighted value is about `1e-100`.
- For the complete vector `[x^2,x^3]` at `x=1e-160` with weight `1e300`, the
  correct weighted vector is about `[1e-20,1e-180]`. The first unweighted value
  is subnormal and loses substantial relative accuracy; the second rounds to
  zero. Two MPFR evaluations converted before weighting agree on these wrong
  binary64 values.

Thus detecting only nonzero-MPFR-to-zero conversion is insufficient. Precision
agreement and tolerance must concern the quantity actually accumulated. Either
fail explicitly when the unweighted output contract cannot represent it at the
needed accuracy, or add a clearly named already-weighted output path that scales
native multiprecision values before conversion. Existing unweighted callbacks
must retain their exactly-once weighting semantics. The author and coordinator
have been notified; independent dynamic-range regressions are required after
the revised API is available.

The draft also rejects an overflowing weighted binary64 product before giving
forced multiprecision replay an opportunity to correct a spurious raw value.
Final nonfinite contributions must fail, but a suspect initial evaluation should
not prevent a potentially successful native replay.

## Source checks that passed in the draft

Replay state is bound to kernel scientific content, sector index, numerical
output length, policy, and a version. Invalid policies, negative/nonfinite maxima,
and nonzero maxima in an unverified state are rejected before mutation. Maxima
merge is componentwise, commutative and idempotent; it does not merge numerical
samples or increment diagnostic counters.

The first finite observation is checked; a previously zero maximum triggers a
check on a later positive magnitude. Growth in any numerical component requests
a replay of the whole Laurent vector, including paired real/imaginary components.
The default growth factor is 16. Already completed native two-precision rescue
can satisfy the requested minimum precision without a duplicate replay. Failed
evaluations do not update the state within a single context call.

The draft context owns a cloned numerical kernel and buffers. Evaluation and
state merges perform no Atom construction or symbolic algebra on workers.
Policy/state serialization uses existing serde and identity hashing; no new
numeric representation or special-function evaluator is introduced.

## Integration and checkpoint review requirements

The runtime's additive weight-observing callbacks already passed reciprocal-
weight regressions for QMC periodization and a nonuniform trained Havana grid.
They expose a weight and apply it once after receiving unweighted values.
An already-weighted callback must have an unambiguous separate contract.

For MC pilots, native `ContinuousGrid::add_training_sample` multiplies its input
value by the sample weight internally. Passing an already-weighted envelope to
that method would weight training twice; any new path must preserve the native
training statistic without duplicating grid algorithms.

CLI persistence must be reviewed after it is implemented:

1. Validate the complete state layout/identity/policy before resume and allow a
   worker-count change without sharing mutable contexts across workers.
2. Merge state with a defined relationship to accepted work. In particular,
   failed-package prefixes must not silently change replay decisions on a retried
   package while its numerical partial was discarded.
3. Save successful complete packages, diagnostics and compatible replay state on
   cancellation or another worker's failure. MC pilot restart limitations remain
   explicit; numerical samples from pilots must not enter production.
4. Preserve whole-vector replacement and append each accepted sample once.
5. State the reproducibility scope. Idempotent maxima merging alone does not
   prove scheduling-independent precision selection: first-observation/growth
   decisions depend on observation history. Test resumed/changed-worker behavior
   against a known accurate vector, not merely equality of serialized maxima.
6. Keep the guard a documented heuristic. Growing by less than 16 relative to
   successive accepted maxima need not trigger it; it is not a proof that every
   accepted binary64 evaluation has the requested accuracy.

## Connected implementation review

The revised real and complex rescue paths apply the sampled weight in native
MPFR before binary64 conversion and before the whole-vector two-precision
agreement check. This addresses both underflow and recoverable unweighted
overflow. The author reports six connected replay tests and four complex tests
passing. The state now explicitly documents accepted-package-only merges and
observation-order-dependent precision choices; those statements accurately
describe the implementation's scope.

A second range defect was found independently in the connected complex fast
path: it detects both components zero or a nonzero subnormal component, but can
miss one underflowed component beside a normal component. For
`(1+i*10^-200)*(x^2+eps*x^3)` at `x=1e-100` with weight `1e300`, the final vector
is about `[1e100,1e-100,1,1e-200]`. Establishing larger maxima first prevents the
growth heuristic from masking this defect. The ordinary complex JIT may return
zero for the small components despite their final weighted values being
representable. `tests/independent_weighted.rs` covers this case and its swapped
real/imaginary version. The coordinator and author agreed on conservative
single-component zero/subnormal rescue under amplification, without new symbolic
real/imaginary decomposition. The fix and independent regression passed the
41-test focused gate (`output/native-numerator-weighted-tests.log`), alongside
seven author replay tests and the native master/reduction comparisons.

The additive `evaluate_weighted` QMC/Havana worker paths accept final weighted
vectors, while existing callbacks retain their original unweighted contract.
The coordinator independently reviewed these adapters. Production MC does not
inverse-scale. Pilot training preserves the native sampled point and weight,
reconstructing its unweighted scalar envelope only when binary64 round-trip
scaling is accurate; otherwise a dedicated `UnrepresentablePilotEnvelope` error
carries the weighted envelope and sampled weight. Per-task native grid clones
make this failure transactional. Native positive-weight validation occurs before
the callback. Public QMC14/MC8 tests passed before the final typed-error change;
three additional private MC tests subsequently passed in the 25-test library
gate (`output/native-numerator-weighted-tests.log`). They cover preserved
subnormal production values versus intentional pilot rejection, equivalent
native training with original sample metadata, and rejecting a degenerate grid
weight before invoking even a zero-valued callback.

The coordinator also identified the need to reject a Korobov product Jacobian
that underflows to zero at an entirely interior point. Numerica's previous
transform returned zero for 100 coordinates at `0.001` even though every scalar
Jacobian was positive. Native feature-branch commits `dc82bd6` and `5507289`
introduce `NumericUnderflow` and distinguish true endpoints. The follow-up
conservatively rejects subnormal attenuation even when later factors could
recover a normal final product: 46 coordinates at `0.001` followed by 54 at
`0.5`, and the reversed order, both fail explicitly. The amplification-first
ordinary product is normal; this is a documented range limitation rather than
a claim that the final mathematical weight cannot be represented. Accepted
point/Jacobian arithmetic is unchanged. All 26 native QMC tests passed with
serde and bincode enabled. The coordinator independently reviewed the guard.

## CLI persistence source review

The reviewer inspected `driver/replay.rs`, `execution/{mod,qmc,mc}.rs`,
`checkpoint.rs`, replay counters, and the focused persistence tests. Version-three
checkpoints bind policy, scientific kernel identity, sector order and output
layout. Candidate replay state is validated before numerical submission; it is
merged only after successful submission. Thus failed prefixes and rejected
duplicates do not advance accepted maxima. Every successful package from the
same dispatched batch is submitted even if another package fails, and attempted
evaluation counters remain separate from accepted statistical evidence.

Workers own their contexts and merge accepted maxima before dispatch. Cancellation
and production failures save accepted state; the existing MC pilot restart
limitation remains explicit. No new source correctness finding was identified.
A meaningful additional regression was requested: resume an incomplete weighted
production checkpoint with a changed worker count and compare new accepted work
against an analytic integral. The current completed-checkpoint worker-change
test performs no new work and cannot alone establish that behavior. The author
added the requested regression: two of four replicas are accepted with their
actual replay maxima and diagnostics, then three workers resume the remaining
work. It reaches 4096 completed points, the analytic integral `1/2`, full
production coverage, and monotonic retained counters/maxima. The reviewer read
the test and verified its successful log. All 14 focused CLI tests passed,
including failed-prefix/rejected-submission transactions, invalid replay-state
and old-version rejection, real SIGINT, fresh-process artifacts, and native run
cards (`output/weighted-cli-tests.log`).

The coordinator's zero-weight finding is also resolved: a zero integration
weight no longer bypasses evaluation of a singular coefficient at an exact
endpoint. The connected regression rejects `0 * log(0)` without advancing replay
state and still accepts a finite interior coefficient times zero. This avoids
turning an undefined endpoint evaluation into a fabricated zero sample.

## Validation status

All source findings in this review are resolved, and the targeted core range,
native QMC, worker, CLI persistence and partial-resume gates passed as recorded
above. No independent performance or full multiloop parity endorsement is implied.
The coordinator's final workspace gate remains separate from these focused
results; the previous milestone's 147-test count did not cover these changes.
