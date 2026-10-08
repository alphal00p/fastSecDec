# Serial integration foundations

This implementation retains caller-owned scheduling and executes one complete
randomized lattice or Havana batch per reservation. Native sessions do not own
threads, processes, evaluators, files, or the integration loop.

## Reuse evidence

Inspected public Numerica 3.0.1 APIs and source for `MonteCarloRng`,
`ContinuousGrid`, `StatisticsAccumulator`, and `DoubleFloat`; inspected the
existing FastSecDec QMC workers, replica covariance reducer, Havana workers and
pilot/freeze transitions. The focused Rust probe in ignored
`output/serial-reuse-probe` compiled and ran before adding the vector helper. It
verified jumped/exported RNG state, exact Havana draw counts for all three
uniform-floor branches, native grid serialization, and lossless serialization
of compensated low-order numeric components.

Numerica's public accumulator is scalar. FastSecDec's existing vector replica
reducer accepts a complete slice and does not provide bounded incremental
covariance. The narrow added operation, `ReplicaMoments`, uses Numerica
`DoubleFloat` arithmetic for centered online vector means and full covariance.
No alternate RNG, sampler, evaluator, transform, or scalar numeric type was
introduced. Existing QMC/Havana workers still perform actual coordinate
generation and weighted evaluation. Whole coefficient vectors remain intact.

Cross-sector aggregation retains the compensated centered means until after
the final sum. Saving an accepted allocation also saves its compact moments;
rounding a displayed sector mean before cancellation would lose contributions
such as `(1e16 + 1) - 1e16`.

The accumulator rejects a nonzero diagonal variance update or covariance
normalization that underflows to zero. The same checked normalization feeds
both sector displays and global convergence; identical tiny replicas remain
valid. Numerical range failure cannot silently become exact-zero uncertainty.

## Random-stream partitioning and bounds

Numerica's xoshiro256** implementation has period `2^256 - 1`; native `jump()`
advances by `2^128` generator draws. One coordinator-owned frontier reserves a
fresh jumped stream for each complete replica across every sector and phase.
At most `u64::MAX` stream reservations are admitted. Each consumes at most
`u64::MAX` draws, so the allocated intervals remain below `2^193` of the period.
Counters and draw products are checked before mutation.

A QMC replica draws one shift coordinate per dimension. Native
`ContinuousGrid<f64>::sample` consumes one draw per axis and at most one draw
for the uniform-floor decision; its per-axis bin lookup has no rejection loop.
The MC budget is conservatively `points * (dimension + 1)`. Tests count actual
native calls for floor zero, mixed floor and floor one. A future Numerica sampler
change affecting this bound must update the corresponding audit and regression.

Reservations bind integral, sector, pilot/production phase, epoch, replica,
stream, recovery run and lease. Only complete issued returns can be admitted.
Retry retains coordinates but receives a new lease, after the caller confirms
the previous worker is dead. Restore increments the run fence and makes old
reservations available again. No worker number, PID or clock value seeds
numerical work. Process IDs and timestamps identify IPC lifetimes only.

## Statistics and scheduling

Serial QMC uses independent sector randomizations. Serial MC reuses per-sector
native Havana grids; the discrete-sector mode remains a separate ordinary
integration lane. Adaptive pilots never enter production covariance. Fixed-size
continuation retains frozen grids and appends new replicas; point-size changes
start a distinct epoch, with adaptive variants repeating their native pilot.

The accepted estimate uses the latest complete homogeneous allocation from each
sector. An incomplete replacement cannot erase this accepted evidence or block
global convergence forever while fast sectors refine ahead of slower sectors.
Separate current/live fields expose every newly completed replica. Accepted
contribution rows retain the original replica counts and point design, so saved
result validation never confuses old evidence with current work progress.

First-coverage selection uses explicit caller-provided residency claims, not
merely in-flight tasks. This prevents a second worker taking a sector while its
first worker is between replicas. After coverage, complete-replica boundaries
and the minimum sampling residence control error-priority reassignment.

A temporary allocation drain is not an eviction boundary: a resident below its
minimum waits while another worker finishes that sector's last outstanding
replica. Restored unissued reservations remain eligible during first coverage.
After shrinking the pool, those reservations drain before fresh work can enter
the bounded ledger, even if an unreserved sector currently has larger error.
After the minimum residence, a previous complete allocation also counts as
production evidence when deciding whether to yield during a replacement pilot
or before its second new replica. A resident does not acquire a new compulsory
visit merely because refinement started.

## Caller-owned resident execution

The CLI retains one selective evaluator context per process and routes its
local output layout into the complete common Laurent vector. Its loading policy
honors the caller's optional artifact validation setting. Frozen parameter,
precision and replay settings are checked for every request. Eviction reaps the
old process before creating a replacement; stopping reports no live residents.

Heavy requests and final receipts use bounded-lifetime staging files; progress
uses the shared dedicated control socket. Native stdout is a diagnostic log.
The final receipt prepares replay, diagnostic and operational changes before
native numerical admission, then checkpoints the combined transaction. A
checksum covers that entire envelope in addition to the native session digest.
Progress carries cumulative timing and diagnostic summaries for its current
task: visible and cancelled-work operational totals include these observations,
while scientific statistics and accepted diagnostics require a complete replica.

A stable checkpoint lock is inherited by every resident child. The coordinator
never unlinks or explicitly unlocks it: after a coordinator crash, restoration
cannot reserve the abandoned stream until all surviving old children release
their inherited lock by exiting.

Exact-offset binding runs in a disposable setup process before the resident
sampling pool starts. That process reads only exact records, applies native
parameter and mass constraints, and returns a compact result manifest. Its
stochastic sector IDs and dimensions come from the full catalogue, not the
exact-only kernel owner's empty sector list. The manifest retains the common
Laurent layout and bound point identity. Reaping setup prevents Symbolica's
append-only polynomial variable tables from surviving in the coordinator.
Setup still has its own symbolic memory cost; it is separate from the bounded
resident sampling envelope.

A fresh-process CLI probe of the prepared massive vacuum graph verified the
exact-only route with no stochastic sectors and zero sampled points. At masses
one and two, its full complex Laurent layout matches
`210 * Gamma(eps) * (mass^2)^(-eps)` through order one within `1e-10`.
The mass-zero request is rejected by the retained native mass constraint rather
than changing endpoint structure silently. The unprepared graph agrees with
the same result after serial sampling. Probe inputs and output remain ignored
under `output/serial-validation/cold-family-probe`.

## Validation boundary

All twenty serial tests passed in the complete native library run
(`181 passed; 0 failed; 16 ignored`), including actual coordinate-sequence
comparisons for concurrent same-sector workers, retries and restored sessions;
changed worker capacity; both refinement policies; centered covariance;
residence and first coverage; pilot exclusion; counter exhaustion; transactional
rejection; and asynchronous convergence without a global barrier. Additional
compensated cross-sector and accepted-observation tests accompany the latest
implementation. The expanded coordinate matrix covers plain/adaptive QMC and
MC, both refinement policies, reversed returns, serialization/reload, fenced
retries and worker capacity changes. Additional controls cover complex full
Laurent covariance, missing/pilot sampled zeros, identity/refinement overflow,
the supplied lattice's native index limit, and both restart scheduling edges.
The real-process CLI suite passed all eight tests: three inspection controls,
two ordinary/streamed generation and crash-recovery controls, and three serial
integration controls. The integration controls cover all four sampling methods
with either artifact generation mode, frozen-size refinement against ordinary
integration and a complex analytic integral, completed checkpoint restoration,
and SIGINT during sampling followed by restoration with fewer workers and a
different residence/batch size. Resident PIDs are confirmed dead after stopping;
the recovered result contains the required complete replicas. Resource-envelope
and independent scientific review gates remain separate from these checks.

The direct OS-process coordinate regression also passed in the CLI unit suite.
For plain QMC and MC, with both point growth and fixed-size continuation, two
different child processes sample the same sector and return in reversed order.
A complete unadmitted receipt is retained across reaping and replacement: the
retry reproduces its coordinates exactly, while its old receipt is fenced out.
A durable checkpoint is restored with one worker, then again with three, and
every fresh replica has a distinct first point and complete coordinate sequence.
Adaptive pilots and frozen grids are covered by the separate eight-way native
coordinate matrix and the real CLI method matrix. SIGINT while sampling covers
partial-work cancellation and confirms resident processes are dead before
recovery. The OS fixture itself checks lost complete receipts rather than
recording partial coordinates during a kill.

The latest complete workspace attempt passed the native library and CLI unit
suites but exposed unrelated CLI routing/expectation failures under repair.
New cross-mode ordinary integration, adaptive-QMC refinement, relative
checkpoint and pre-residence live-progress assertions await the coordinator's
focused rerun; this document does not claim those unexecuted additions pass.
