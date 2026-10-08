# Independent serial sampling and statistics review

Date: 2026-10-08. Scope: native `integration/serial`, CLI `driver/serial`,
and the process fencing needed to make checkpoint replay safe. This review is
independent of those implementations. It does not certify the unfinished
artifact/generation, ordinary-refinement, resource-measurement, or full CLI
acceptance gates in `SERIAL_MODE_PLAN.md`.

Latest source-review status: all concrete findings below have been addressed in
source and independently re-reviewed. The newest numerical guards, checkpoint
integrity changes, and real-process coordinate fixture await their coordinated
test run; this document records that boundary explicitly.

## Findings and disposition

1. **P1 — temporary allocation saturation bypassed minimum residence.** In the
   original `integration/serial/scheduler.rs` resident guard, retention required
   `available(resident)`. After coverage, two workers can hold the final two
   replicas of one sector's allocation. When the first returns below the minimum
   residence, no further replica is reservable until its peer returns. The
   original scheduler could evict it for another sector despite neither global
   convergence nor explicit budget exhaustion. The implementation owner changed
   this guard to retain any nonexhausted resident before its minimum (or before
   valid production evidence), permitting `reserve(None)` to mean wait. The
   new `minimum_residence_waits_for_a_concurrent_final_replica` regression covers
   both QMC and MC. **Source fix reviewed; regression passed in the 177-test native suite.**

2. **P2 — supplied lattice refinement must respect the native point bound.**
   `integration/serial/state.rs::SerialMethod::next_points` initially uses
   `u64::MAX` for `RuleSource::Supplied`, while the native lattice implementation
   bounds points at `2^53`. A supplied rule at that bound would attempt an
   invalid doubled size instead of switching to new shifts. This can be tested
   by invoking the design transition directly, without evaluating a huge
   lattice. **Fixed to the native `2^53` bound; source and regression reviewed.**

3. **Acceptance gap — the duplicate-work matrix is not yet complete.** Current
   tests compare actual coordinate vectors for concurrent same-sector tasks,
   reversed returns, a released/reissued reservation, and checkpoint restoration
   with a smaller pool. These are valuable substantive checks. The growth test
   checks epochs/counts, however, without checking its generated coordinates
   against earlier work, and the adaptive-MC test checks phase/epoch rather than
   frozen partitions and coordinates. Actual child-process replacement and
   restored sessions with more workers need the same coordinate-level controls.
   **Do not treat the existing native tests alone as completion of the plan's
   mandatory matrix.**

No duplicate reservation admission, stream-frontier reset, coarse/fine pooling,
or pilot-to-production contamination was found in the reviewed native path.
That is a bounded source-review conclusion, not a claim that the remaining
acceptance gates have passed.

## Independent RNG verification

Public API and source were inspected in registry Numerica 3.0.1:

- `src/numerical_integration.rs:1550`: `MonteCarloRng`, xoshiro256** state,
  `new`, `export`, `import`, and `jump`.
- `src/numerical_integration.rs:1682`: documented jump spacing `2^128` and
  the four-word jump polynomial.
- `src/numerical_integration.rs:1136`: `ContinuousGrid::sample`, including
  disabled, mixed, and entirely uniform floor branches.
- `src/numerical_integration.rs:1328`: each continuous dimension samples once;
  bin selection/interpolation introduces no rejection loop.
- `src/domains/float/native.rs:102`: native `f64` samples through Rand's uniform
  float implementation, consuming one `next_u64`.

A separate Rust probe linked the already-built actual Numerica 3.0.1 library,
without running Cargo or modifying production code. For seeds `0`, `1`, `42`,
and `u64::MAX`, 32 successive jumped blocks of 64 outputs matched the upstream
Rand `Xoshiro256StarStar` implementation exactly. Export/import preserved every
output; `new(seed, 1)` equalled `jump(new(seed, 0))`.

The probe counted actual native generator calls for 4,096 samples in dimensions
1, 2, 5, and 23, for probability floors 0, 0.125, and 1. Every sample consumed
exactly `dimension` calls for floors 0 and 1, and `dimension + 1` for the mixed
floor. Thus `points * (dimension + 1)`, with checked multiplication, bounds
current native MC draws. A QMC task draws exactly one shift component per axis
and owns the whole lattice range.

A separate temporary bit-linear state-space probe independently exponentiated
the 256-bit state transition. It verified the implemented jump polynomial
against `T^(2^128)` on **all 256 basis states**. It also verified
`T^(2^256) = T` and the full nonzero orbit order using the complete prime
factorization of `2^256 - 1`:

```
3 * 5 * 17 * 257 * 641 * 65537 * 274177 * 6700417
  * 67280421310721 * 59649589127497217 * 5704689200685129054721
```

For a nonzero basis state, `T^N(v) = v` and `T^(N/p)(v) != v` for every factor
`p`, where `N = 2^256 - 1`. This verifies the period and jump spacing separately
from simply trusting distinct seed integers. The audit probes remain temporary
under `/tmp/fastsecdec-serial-independent-audit/`; no alternate RNG was added to
the implementation.

`Streams` reserves at most `u64::MAX` consecutive jumped streams, and a
reservation consumes at most `u64::MAX` raw draws. The last allocated position
is strictly below `2^193`, far short of the verified period; each reservation's
budget is also far below the `2^128` spacing. `checked_add` and checked draw
products reject exhaustion before updating the frontier. This proof assumes
the reviewed native sampling implementation; changing native RNG consumption
requires rerunning the counted-draw regression and this audit.

Independent finite-precision coordinate coincidences do not invalidate the
stream proof. A task or stream being assigned twice is a different failure and
must remain a hard rejection.

## Reservation, recovery, and process boundaries

`SerialSession::reserve` owns the single frontier. A reservation includes
integral identity, sector, epoch, pilot flag, replica, stream, recovery run, and
lease. Existing tasks reserve an entire shifted lattice or MC batch; the worker
internally traverses its points and cannot ask for another random stream.
Sector changes do not construct fresh RNGs from the master seed.

`release` requires the exact current identity and leaves the original work
available for a new lease. `validate_return` compares the issued reservation,
its complete serialized task digest, vector shape, timing, and training phase.
Submission removes that reservation; later duplicate returns are unissued and
rejected. New-epoch creation cannot happen while any old allocation replica is
pending. Checkpoint restore preserves the RNG frontier and pending identities,
increments the recovery run, and fences old returns. A reduced worker count can
reissue old pending reservations serially before admitting additional work.

The native API explicitly requires the caller to confirm the old worker's death
before `release` or restore. CLI process termination waits for the child before
releasing capacity. The new shared residency lock is inherited by native child
processes, so an old surviving child keeps a replacement coordinator from
reserving checkpoint work. Native control travels over a dedicated local
socket; stdout cannot corrupt protocol frames. An EOF watchdog exits the child
if its coordinator disappears. This is the correct ownership pattern; retain
an actual crash/restart regression proving the lock remains held until the last
old child exits.

CLI admission prepares replay/diagnostic/operational updates before numerical
submission, then commits those prepared values and the accepted session in one
checkpoint envelope. Periodic checkpoints may lose recent uncommitted work on
crash, which may be replayed after all old workers die. Such work must never be
simultaneously reissued or admitted twice.

Operational metrics intentionally describe the current invocation, while
accepted native counts and cumulative diagnostics survive resume. Presentation
should distinguish these meanings; resetting invocation metrics is not itself
loss of accepted numerical work.

## Statistical and ecosystem review

The implementation reuses native Havana grids, native QMC workers/transforms,
Numerica RNGs, and native numeric types. It does not add an algebra, graph,
physical-input, or alternate sampling implementation. A library caller controls
jobs and scheduling; process ownership remains CLI-only.

Numerica's public `StatisticsAccumulator` is scalar and cannot provide the full
cross-component covariance needed here. Existing FastSecDec QMC reduction
requires all replica vectors. The narrow online `ReplicaMoments` helper uses
Numerica `DoubleFloat` for centered means and the symmetric Welford outer
product. Covariance of the mean divides by `n * (n - 1)`. Full Laurent vectors,
including real/imaginary components, remain intact.

Serial sector randomizations are independent; summing their covariance matrices
is appropriate. Exact offsets are included once in the global mean. The global
sum retains compensated centered means until the final conversion to `f64`,
including previous accepted epochs. This is necessary for cancellation such as
`(1e16 + 1) - 1e16`; rounding each sector's mean first would erase it.

Pilot moments cannot provide production evidence. Fixed-size continuation
retains the frozen grid and existing moments, doubles the target, and issues
only new replicas. Size changes start new epochs and reset production moments;
previous completed evidence remains available until a replacement completes.
Status separately exposes current/live progress. At least two production
replicas are required; absent sector estimates prevent constructing a complete
global stochastic estimate.

## Remaining focused regression requirements

- Cross product of QMC/MC, both growth settings, retries, reversed returns,
  process eviction, and resume with fewer **and more** workers. Record coordinate
  sequences/ranges and verify that only an explicitly retried identity repeats.
- Adaptive MC: freeze the exact partitions, continue with fixed points, and
  prove no pilot runs and no earlier stream/coordinates are reused. For growth,
  drain old tasks before training/production of the replacement design.
- Unissued and duplicate returns, stale run and lease, invalid phase and task
  digest, stream/lease/epoch/replica counter exhaustion; rejected operations must
  not alter accepted statistics or the frontier.
- Complex Laurent controls with cross-component covariance, large cancelling
  sectors, exact offsets, zero tolerance, and an unsampled or pilot-only sector
  alongside sampled zeros. Missing evidence must remain missing.
- Real child replacement and coordinator-death recovery while another child
  still owns an active reservation; new work is forbidden until its death.
- Verify updates during a long residence at every complete replica boundary,
  retention of a still-winning sector, and no early eviction while waiting for a
  peer's final replica. Explicit exhaustion must remain an unconverged stop.

Ordinary shared-shift QMC and discrete MC append behavior require a separate
review once their implementations are ready. Their covariance/proposal models
must not be silently replaced by the independent-sector serial estimator.


## Follow-up: ordinary fixed-size continuation and expanded serial controls

The later native run in `output/serial-validation/native-lib-tests.log` completed
177 tests with no failures (16 explicitly ignored). It includes all 17 serial
controls and four ordinary-refinement controls. The initial matrix gap above is
now substantially narrowed: `serial/tests/matrix.rs` covers QMC/MC, adaptive or
ordinary production, both growth policies, reversed returns, serialized reload,
retry, smaller/larger configured capacity, pilot exclusion, frozen MC grids,
complex full covariance, missing-sector and pilot-only zeros, and transactional
identity/refinement counter exhaustion. The matrix's reload is serialization
plus a fresh native worker; it is not by itself an OS-process trace test.

Separate CLI process tests exercise real interruption, worker reclamation and
resume with changed worker counts, residence durations and evaluation batching.
Their source also checks both artifact origins against all four serial methods
with a complex analytic control. A direct test-only subprocess coordinate
fixture was recommended to tie the native sequence matrix to actual process
replacement without adding sampling logs to production.

### Ordinary append review

`QmcSession::extend_production_shifts` requires complete, drained production and
keeps the lattice and periodization. It reconstructs the deterministic enlarged
plan, and `QmcAccumulator::extend_plan` checks all old shift coordinates bit for
bit before admitting the new plan. Only accepted package identities are updated;
no old interval becomes missing. The session increments the allocation epoch to
fence old tasks. Democratic QMC keeps the same shifts across sectors, and the
existing common-shift vector reduction retains their covariance, including exact
cross-sector cancellation.

The added allocation-boundary list correctly preserves a previous short final
package when the package size does not divide the old allocation. A separate
Rust probe linked the actual compiled native API and tested package sizes
**1, 3, 7, 16, 31, 32, 33, and 97**, each through four successive allocations of
a 16-point lattice starting at two shifts. Every allocation returned packages
in reverse order and restored its checkpoint after extension. Counts matched
exactly, no actual coordinate pair repeated, and each allocation completed.
This includes single-point, nondividing, exact-boundary, and larger-than-old-
allocation packages.

Ordinary Havana and discrete Havana append reserve new streams from the shared
`integration::streams::Streams` frontier. The numerical spacing/draw-budget proof
above therefore applies to these new reservations as well. Discrete sampling
adds exactly one native uniform draw for the top-level sector selection before
the continuous child, bounded by `points * (max_dimension + 2)`. Existing grids
and discrete probabilities are retained; accepted batch means are retained and
new batch identities are appended. Rewriting their enclosing grid identity does
not recompute samples. Existing pilot/freeze transitions stay separate.

### Additional finding: protect ordinary checkpoint frontiers

**P1 — ordinary MC/discrete checkpoints initially did not authenticate their
saved stream frontier against accidental corruption.** Their new version-2
readers accepted an arbitrary nonzero frontier state while validating the stored
batch states only against each other. A frontier pointing back into an earlier
stream could consequently make append consume earlier random work again.
Comparing only task IDs or fresh batch numbers would miss this.

The implementation owner added reserved-state/counter checks and a complete
checkpoint integrity digest covering configuration, frontier, proposal and
accepted records. The reviewer checked the source of both digest producers and
readers. A separate probe against the first built guard version confirms that
setting the frontier to a stored reserved seed is rejected. The digest also
protects against a state moved inside an earlier stream, which a simple equality
check against stream starts would not catch. **The latest digest additions and
corruption regressions still require the coordinated test run.**

No new QMC prefix/tail, frozen-proposal, pilot-exclusion, or covariance defect was
found in this follow-up. Ordinary CLI retention of earlier complete epochs was
being wired during review; its final presentation/checkpoint regression remains
part of the implementation owner's validation, not certified by this snapshot.

### Further policy and near-zero findings

- **P2 — completed evidence must count during replacement scheduling.** The
  corrected minimum-residence guard still treats a replacement pilot or fewer
  than two current production replicas as no valid evidence, even when a prior
  completed allocation exists. It therefore retains a resident beyond its
  minimum rather than comparing error priorities after each complete batch.
  The initial two-replica requirement must not prevent rescheduling a replacement
  epoch that already has saved valid production evidence. Reported to the native
  owner with a regression requiring the higher-error peer to win after minimum
  residence, while the current sector has a saved estimate and new pilot.

- **P1 — underflow in the new online covariance can fabricate convergence.**
  An independent standalone Rust probe through the public `ReplicaMoments` API
  adds the two scalar replica means `0` and `1e-200`. The compiled implementation
  returns mean `5e-201`, standard error `0`, covariance `0`, and says a complete
  estimate meets **zero absolute and relative tolerance**. `DoubleFloat` adds
  precision but retains the binary64 exponent range; squaring a small nonzero
  deviation loses the variance. The narrow safe remedy is to reject a lost
  nonzero variance update or covariance division as `NumericRange`, while
  preserving identical tiny replicas whose true sample variance is zero. No new
  general numeric engine is required. The probe is
  `/tmp/fastsecdec-serial-independent-audit/moments_probe.rs`. Reported to the
  implementation owner; the source fix now rejects underflow both during
  the transactional update and through a shared checked covariance-normalization
  path used by sector estimates and the global aggregate. The new constant and
  normalization controls were reviewed; **their coordinated execution remains
  pending at this snapshot**. The ordinary historical slice reducer has a related limitation;
  this finding specifically blocks accepting the new serial helper as safe for
  the plan's near-zero gate.


The replacement-scheduling guard now recognizes previous completed evidence; its
new regression covers both a replacement pilot and fewer than two replacement
production replicas. Source re-review accepts this fix.

`process/sampling_tests.rs` now supplies the recommended actual-process fixture.
For QMC and MC under both growth policies it starts two distinct OS workers on
the same sector, accepts returns in reverse reservation order, retains one
unadmitted receipt across confirmed worker death, releases/reissues its original
work, and rejects the stale receipt after restoring with one worker. It then
restores with three workers and reserves three concurrent new replicas, checking
their actual coordinate prefixes and full sequences against earlier work. The
retry alone must exactly reproduce its old coordinates. This fixture follows the
real framed worker transport and synchronous native task evaluator; production
code gains no coordinate logging. **Source reviewed; execution pending.**

The strong global jump-partition proof in this review applies to serial tasks
and the shared ordinary MC reservation frontier. Ordinary adaptive QMC retains
its existing sector-namespace mixing convention; within each plan the new shifts
are jump-separated and prefix-preserving append cannot repeat old assigned work.
This review does not mischaracterize that legacy namespace mixing as a global
jump-partition proof across sectors. Democratic QMC intentionally shares shifts
between sectors so its existing cross-sector covariance is preserved.
