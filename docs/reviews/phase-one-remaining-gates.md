# Phase-one remaining gates

Current ledger, 2026-10-06. [FIRST_PHASE_PLAN.md](../../FIRST_PHASE_PLAN.md)
remains authoritative. This reconciliation adds no requirement or execution
campaign. The previously identified scientific subsystems are implemented.
The subsequently requested native HEPKit entry points and visible notebook
workflow are under validation; the three performance cases below remain open.

## Current HEPKit API extension

The user approved the canonical `hepkit.sector_decomposition` namespace and
`sector_decompose()` methods on the existing native `FeynmanDiagram` and
`IntegralFamily` classes. Implementation includes signed family powers, shared
native input specialization, compatibility exports and visible notebook calls
behind the existing explicit actions. Independent source review, 29 native
input/parametrization tests and all 95 installed Python/frontend controls pass.
The actual native triangle UI validates visible scientific calls, generation,
sector inspection, and same-kernel QMC/Havana cancel/resume. Generated type-stub
validation also passes. Fresh portable validation is pending; the older delivery
records below do not certify this extension.
See the [entry-point review](hepkit-sector-entrypoints.md). Performance work
remains parked until this requested API/notebook milestone is validated.

## Accepted implementation and delivery

- The regression inventory retains **182 rows: 101 Covered, 81 Retired,
  zero Partial/Pending**. Retired interfaces do not retire their scientific
  replacements. Original examples, native graphs/numerators, direct densities,
  complete Laurent vectors, guarded precision, covariance and saved results
  remain covered by the [regression matrix](../REGRESSION_MATRIX.md).
- The historical broad workspace gate passed **419 tests**, with zero failures
  and 25 explicitly ignored heavy probes. Formatting and strict all-target
  Clippy passed. The later `e25ca59` literal-zero recovery fix passed **24 focused
  artifact/replay controls**, formatting and scoped strict Clippy; this is not
  a claim that the broad gate was rerun for every later documentation revision.
- Current hosted native HEPKit CI passes **84 controls**, consuming community
  `6836ca3` and FastSecDec `ef2861b`. The earlier 81-test native-wheel record
  remains historical. [Actual CI](https://github.com/symbolica-dev/symbolica-community/actions/runs/37415210274)
  and [delivery review](dependency-delivery.md) retain the precise build scope.
- The actual public Wasm core built from FastSecDec `a3d09e1`/community `c9bacce`
  passes generic smoke and **58 portable controls**. Later notebook-only timer
  ownership changes were exercised against that accepted wheel; this does not
  claim that the newer native core was rebuilt for Wasm. Actual triangle
  metadata, QMC, Havana MC, cancellation and exact-prefix restoration pass.
  The earlier supplemental screenshot failure remains explicitly qualified.
  See [portable evidence](hepkit-metadata-mc-portable.md).
- Native generation metadata, exact endpoint powers, maps/Jacobians and actual
  evaluator statistics are exposed lazily through native objects. The MC lane
  reuses Numerica `DiscreteGrid`; callers own pilot, adaptation, production,
  workers and checkpoints. [Metadata](retained-generation-explainability.md)
  and [MC](havana-discrete-sector-sampling.md) implementation gates are closed.
- FastSecDec owns the optional binding crate, tests and notebook. HEPKit only
  links/registers/reexports it and supplies stubs; the numerical core and
  default CLI remain Python-free. [Community PR18](https://github.com/symbolica-dev/symbolica-community/pull/18)
  and [Numerica PR8](https://github.com/symbolica-dev/numerica/pull/8) are published.
  Upstream merge/review disposition is not an unimplemented library capability.

## Closed scientific and representative tracks

**Triangle and box:** the accepted seven-pair generation/integration records
stand. The corrected box first-use full-stream latency comparison observes
native mean/max **9.878479/5,863.705 µs**, versus Pathfinder
**70.390258/43,051.430 µs**, with scientific equivalence preserved. The Pathfinder
outlier remains unattributed; observed maxima are not worst-case guarantees.
Do not reopen these cases. See [current sample latency](current-sample-latency-results.md).

**Original on-shell triple box:** the user explicitly accepts successful
complete generation for phase A. All 1,026 sectors/orders and cold loading are
validated. Numerical accuracy, references and performance are deferred for this
case; interrupted attempts remain partial evidence. Earlier long-run proposals
are historical, not instructions to resume them.

**Native gg→HH:** the complete QMC allocation reaches **0.0937‰** finite-part
relative uncertainty; ordinary Havana MC reaches **0.577‰** and agrees within
1.70 combined component standard errors, with complete covariance. Native UI
and checkpoint controls pass. Independent pySecDec denominator checks agree on
all 30 maps, Jacobians and valuations. These requested gates are closed; see
[native feasibility](gghh-native-feasibility.md), [MC evidence](havana-discrete-sector-sampling.md)
and [sector checks](gghh-sector-sanity.md).

**Optional browser gg→HH:** the published runnable option generates all 30
sectors in **546.667 s**. Its first report download timed out; detailed browser
inspection/integration remain unverified. The option remains runnable as the
user requested despite slow execution and interruption limitations. The later public-state timer fix
passes an actual triangle lifecycle and suppresses inactive automatic refresh;
no gg→HH rerun or new long-call responsiveness claim follows.

**Scientific controls:** reuse the accepted six-family references, native
one-loop masters/reduction, coupled-sunset identity and [72 holdout rows](convergence-stage-a-independent.md).
Off-shell scalar/rank-two, Issue1 and hard-reference transports retain their
specific normalization/value claims. There is no new universal calibration
matrix or requirement to rerun every seed/work combination. Add uncertainty
checks only where an open representative claim needs them.

## Remaining acceptance work

### Scalar double box

The existing adaptive-QMC allocation reaches **0.989‰ in 82.859 s**, including
fresh pilot and loading, versus the retained Pathfinder **0.362‰ in 91.584 s**.
All five available references and the full vector/covariance pass. The preceding
320-worker-second allocation missed at 1.035‰ in 73.298 s; both attempts remain
recorded, costing 156.157 integration-process seconds in total.

This is one successful native observation, not repeated timing acceptance.
**Open:** matched repetitions and current sample/sector latency acceptance.
The current fixed-work Taylor result is **152.996 s at 0.714‰**; 172.715 s and
the 106.314-versus-74.846 µs latency row predate the literal-zero fix. Preserve
them without relabelling them current. Existing IBP was slower and less precise;
no adoption followed. See [double-box follow-up](double-box-lattice-followup.md).

### Numerator-heavy multiloop representative

Complete native vectors and independent checked rank-two references exist.
The bounded Pathfinder rank-two attempts did **not** produce a complete bundle;
ordinary and no-symmetry attempts are retained and further retries are stopped.
**Open:** the planned numerator-heavy multiloop Pathfinder performance comparison
is unavailable, not passed or proven infeasible. The specific rank-two protocol
is historical; any replacement must explicitly preserve the scientific scope.

The existing rank-five fixture is a **one-loop box**. Its native reduction/master
comparison closes high-rank correctness, while its timing compares native
implementations or a selected sector. It is neither a matched Pathfinder
benchmark nor evidence for multiloop numerator performance. Do not silently
substitute it. See [rank-two reference](projected-rank-two-reference-independent.md),
[rank-five validation](hepkit-numerator-reduction.md) and
[historical pairing protocol](minimal-paired-acceptance.md).

### Hard four-loop full orthant

Original-density generation and the first complete fixed-work pair pass:
native **68.735 s / 65.225‰**, Pathfinder **279.689 s / 53.177‰**. Both retain
complete vectors and all available reference checks. Native retains its full
three-order covariance; Pathfinder's six additional lower rows agree with the
separate exact whole-input zero certificate. No numerical rows/covariance are
padded. The different lattices, partitions and failed pre-import prefix remain
explicit in the [four-loop record](hard-four-loop-pathfinder-capability.md).

**Open:** highest-order 1‰ accuracy, repeated timing and current latency
acceptance. The sole subsequent adaptive native allocation is independently
accepted at **316.246 s / 1.12447‰**, including loading and a fresh pilot. All
700 covariances and three reference checks pass; pilot samples are excluded from
the production estimate. It still misses the target. No further adaptive run is
queued, and no convergence extrapolation is implied by this ledger.

## Acceptance rules and stopping boundary

Keep the plan's **seven ordinary / three expensive paired repetitions**,
alternating order and **5% median timing band**. Compare actual eight-core time
to 1‰ for the largest signed requested epsilon power, with independent available
reference checks and complete vectors. Preserve generation/loading/integration
boundaries, actual rules/transforms/work, precision/rescues and memory. Separate
average worker cost, maximum sector-average cost and individual-sample maxima;
unlike evaluator buckets do not establish isolated kernel parity.

Reuse accepted evidence rather than starting another general campaign. The
arithmetic-IR candidate failed a physical check; the storage-only candidate
preserved results but gained only **0.921%**, below its predefined interest
threshold. Both [evaluator probe tracks](native-evaluator-storage-probes.md) stop.
Do not restart closed triangle/box, gg→HH, on-shell or infeasible pySecDec/FORM
work. The current open rows do not authorize another reference ladder.

Linux/MSRV/PTY evidence stands; unavailable platforms remain labelled
unverified. General affine endpoint charts, contour/GCAD, arbitrary complex
masses and new CBC vector construction are documented deferred limits, not new gates
for shipped first-phase examples. Final delivery reconciliation should reuse
existing audits and tests. Once the stated requirements pass, commit/push,
mark the active goal complete and stop for threshold planning with the user.
