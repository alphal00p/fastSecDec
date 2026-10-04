# Native additive sector estimates: proposal

This proposal addresses Pathfinder `test_integrals.py:6351`. The original test
checks that stored sector means add back to the reported Laurent vector and that
sector sample counts account for the total. It does not justify adding marginal
errors as though sectors sampled with common random shifts were independent.

## Capability ownership and reuse evidence

The existing Numerica QMC lane already owns the required numerical primitives:
`QmcAccumulator::shift_estimates()` returns complete replica means with identities;
`estimate()` and `QmcEstimate::from_shift_means()` compute centered vector means and
covariance. FastSecDec's `qmc/results.rs::common_shift_rows` selects the common
complete shifts across every sector. Its total estimator centers each sector
before summing and adds exact contributions only to the mean. The proposed view
must reuse this same selection and retain that total estimator unchanged.

For Havana, `mc/session/results.rs` already reduces equally sized, independent
production batch means through the same native replica statistic. Its immutable
production records distinguish completed batches from partial or pilot work.
Numerica's older `StatisticsAccumulator` owns scalar adaptive-iteration statistics;
it cannot replace this complete-vector/shared-replica covariance with equivalent
semantics. No new accumulator, covariance formula, or summation primitive is
needed: native statistics and existing `precise_sum` remain the owners.

The HEPKit community bridge exports its native graph and one-loop APIs but has no
FastSecDec shared-lattice sector reporting contract. GammaLoop's public
`build_integration_result` and `settings/runtime.rs::{IntegralEstimate,
SlotIntegrationResult,ComponentDiscreteBreakdown}` provide process/integrand and
adaptive-discrete-grid views using real/imaginary scalar accumulators. They do not
carry Laurent-vector cross-order covariance or common lattice-shift coverage.
Their presentation is useful precedent; importing those process-specific result
containers would obscure this runtime's statistical assumptions. Source/API
inspection plus the existing native QMC/MC runtime tests provide the reuse checks;
new tests will exercise only the thin attribution adapter's scientific contract.

## Proposed public API

Add `QmcSession::contributions()` and `HavanaSession::contributions()`, returning
a serializable `ContributionReport` with:

- method, stage, order/component layout, and the exact coefficient vector;
- a typed replica relation: `SharedAcrossSectors` for democratic QMC,
  `IndependentAcrossSectors` for adaptive QMC and Havana;
- rows containing the existing `SectorSnapshot` progress, used replica/point
  counts, `UncertaintyStatus`, and an optional native `VectorEstimate`;
- the existing authoritative total estimate and uncertainty status.

The report's display will label each sector error as marginal. A shared-replica
report must explicitly say its marginal covariances cannot be summed to obtain
the total covariance. No separate cross-sector covariance engine is introduced.
Exact contributions remain a separate vector, not duplicated across sector rows.
Sector IDs identify compiled representatives; generation metadata separately
retains chart multiplicities and coordinate permutations.

For democratic production, every row uses **exactly the same common complete
shift IDs** used by the total estimator. A sector with extra completed shifts
reports those in progress but excludes them from its used counts and mean. Fewer
than two common shifts means every sector estimate and the total remain absent,
with explicit waiting coverage. Per-sector `production_complete` is true only
when all its planned replicas are included in the common selection.

For independent production, each sector may expose its native estimate after two
complete local replicas; the total still waits until every sector is covered.
Pilot rows retain progress but never expose pilot samples as production estimates.
An all-exact problem has no sector rows and reports its exact vector and zero
uncertainty. Typed errors propagate rather than hiding nonfinite arithmetic.

Means are additive contributions, subject to binary64 rounding. Users should keep
the authoritative total instead of recomputing it from rounded displayed marginal
means: large sector offsets can lose small differences in those absolute values,
while the existing total's centered path preserves their covariance.

## Meaningful tests before CLI use

1. Two common-shift sectors with opposite varying contributions and an exact
   offset: each marginal variance is positive while the total variance vanishes;
   means plus the separate exact vector agree at ordinary scale.
2. Unequal completion across sectors: exclude a first sector's extra completed
   shift, verify used-versus-completed counts, and leave all estimates absent until
   at least two common shifts exist. Cover different sector dimensions.
3. Adaptive independent streams and Havana batches: marginal means plus exact
   offset reproduce the total; independent marginal covariances reproduce total
   covariance. Partial local coverage remains visible while the total waits.
4. Pilot and all-exact reports: no pilot statistical evidence or fake stochastic
   sectors, with explicit uncertainty states.

This is a proposal only. No runtime source is changed until the coordinator
approves the contract; CLI persistence/presentation is a later caller adaptation.

## Implementation evidence

The approved adapters are connected as `QmcSession::contributions()` and
`HavanaSession::contributions()`, with `ContributionReport`, `SectorContribution`
and `ReplicaRelation` exported by `integration`. The existing total-estimate
implementations are unchanged. The report takes its authoritative total and
uncertainty directly from the session snapshot and obtains marginal statistics
through native `QmcEstimate::from_shift_means`; no covariance formula was copied.

The release-profile focused gate passed **4 new contribution tests, all 15 QMC
runtime tests, and all 8 Havana runtime tests**. These targets use numerical
closures/replica data only and do not initialize Symbolica. Logs are
`output/sector-contributions-tests.log`,
`output/sector-contributions-runtime-qmc.log`, and
`output/sector-contributions-runtime-mc.log`. The tests cover the proposed common
coverage/cancellation-covariance, independent allocation/batch, pilot exclusion,
exact-offset and all-exact behavior, plus typed report serialization. Independent
source review is recorded in `sector-contributions-scientific-review.md`.

The coordinator's independent API review found a presentation-boundary panic for
malformed deserialized coefficient vectors. The display now checks the report and
all present estimate layouts before indexing and prints an explicit invalid-layout
message. A regression deserializes a shortened marginal mean vector. The final
four-test target and targeted Clippy both pass after this fix:
`output/sector-contributions-final-tests.log` and
`output/sector-contributions-final-clippy.log`.
