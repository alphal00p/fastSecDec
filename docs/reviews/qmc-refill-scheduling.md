# Independent review of CLI QMC refill scheduling

The eight-worker on-shell continuation exposed a substantial batch barrier.
At its retained 200,704-point status, 160 newly completed 1,024-point packages
had consumed 753.659922077 worker-seconds. The sum of the longest worker interval
in each of their twenty eight-task batches was 293.606118797 seconds. Even with
no context or coordination overhead, that schedule's worker-time utilization
for this observed prefix could not exceed 32.09%. Sector 80 alone consumed
90.770544847 worker-seconds; sector 83 consumed 82.378846680 seconds.

The source data and calculation are retained in
`output/diagnostics/native-named-fullgraph-resume-1/independent-package-attribution-1.json`.
This is a heterogeneous completed prefix, not a whole-graph runtime prediction
or isolated performance benchmark. The continuation ran its frozen implementation
and was later intentionally stopped for handoff to the validated caller change.

## Timing and reuse boundaries

`QmcWorker::evaluate_weighted` measures point generation, periodization,
weighted numerical evaluation, precision rescue and accumulation. Its interval
excludes the CLI's preceding `QmcSlot::prepare`. Native precision-cache misses
inside the weighted callback clone/map evaluator constants and are included.
There are no per-sample or cache-miss counters in this observation, so it cannot
separate first-use conversion from repeated multiprecision arithmetic.

All 160 new packages in that observation touch different sectors during this
process. None yet revisits an evicted sector. Cache reconstruction on later
round-robin revisits remains a possible cost, but cannot explain this prefix as
repeated same-run revisits. The measured barrier is sufficient to motivate a
bounded caller scheduling change without altering precision policy.

The existing public owners already support refill on completion:

- `QmcSession::next_work` reserves canonical packages. `None` can mean that
  remaining packages are in flight, rather than complete or erroneous.
- `QmcSession::submit` validates identity, canonical interval and duplication
  before merging a complete return through Numerica's native accumulator.
- `QmcAccumulator` retains packages by canonical index and reduces full vectors
  in that order. Existing reversed-arrival and covariance tests cover this
  behavior in `crates/fastsecdec/tests/runtime_qmc.rs`.
- Native checkpoint restoration reissues unfinished packages and preserves
  accepted complete packages. Pending ownership is not serialized as completed
  work.
- The CLI's accepted replay store remains separate from worker contexts and
  advances only after successful numerical submission.

No new lattice, numerical estimator, algebra, public library pool, checkpoint
format or library API is needed. The existing caller-owned Rayon pool supports
`in_place_scope`, keeping the coordinator outside its worker threads. The
proposed private queue transfers one bounded context with each task and returns
it on completion for immediate refill.

## Independent review status

The implementation and focused test design pass independent source review.
`qmc/queue.rs` owns the private caller loop; `QmcSession` and the native
accumulator retain their existing responsibilities. There are exactly
`workers` slots across the idle list and running jobs. Only the coordinator
reserves tasks, submits complete returns and advances accepted replay state.
An exhausted reservation scan with work in flight waits for a return rather
than treating it as completion or an error.

The coordinator polls cancellation and existing status/checkpoint handlers
while jobs run. Each numerical callback checks a cooperative stop flag before
evaluation. Such a stopped prefix is explicitly distinguished from a real
numerical error and cannot update accepted values or replay state. Complete
successful returns already in flight are drained and submitted normally.
Diagnostics retain attempted numerical callbacks, including genuine failed
prefixes, rather than pretending all attempts were accepted points. A worker
panic is caught and delivered through the completion channel. A scope guard
sets the stop flag on coordinator errors before Rayon joins borrowed jobs.

Four new tests cover deterministic refill before a blocked first job, exact
two-component means and the full covariance against serial native transport,
cancellation with pending checkpoint reissue, exclusion of failed/cancelled
prefixes, and panic notification/draining. The existing sixteen driver
controls remain in the focused filter, including actual MPFR rescues and
eight-worker checkpoint restoration. The pre-cancelled resume control now
expects its original 2,048 accepted points: cancellation is checked before
dispatch instead of issuing one extra package.

Worker scheduling can change weighted replay decisions within the existing
precision policy. The change must not claim bitwise equivalence across arbitrary
schedules; the deterministic native callback control verifies arrival-order
independence of accumulation separately. Source and focused execution are
accepted below; performance of the larger case remains unmeasured. No full-workspace or
new algebra-oracle gate is added for this CLI scheduling change.

The first focused invocation is preserved under
`output/diagnostics/qmc-refill-scheduling-1`. Compilation and scoped Clippy
passed. The test binary is SHA-256
`6cc780a122d9de3dd393d0c2dcfedc7e58da67afb3c6306188cbf9790cbbb9a7`.
The process aborted with signal 6 after 0.190639056 seconds at the first test,
before any completed test result. Its 6,144 KiB resident peak and 0.000942 user
CPU seconds do not establish that the cancellation loop was reached. The
implementation author found the still-running native continuation holding
Symbolica's restricted-thread permit; Symbolica's existing permit-failure path
explicitly aborts. Libtest captures its printed warning, so the empty stderr
does not disprove this explanation. This invocation is not accepted as a test
pass. No license policy or implementation was changed.

After the existing native process was fully reaped, the serialized run reached
all twenty controls. Nineteen passed; the new failed-prefix control incorrectly
expected two diagnostic evaluations after two successful callbacks and one
failed callback. Existing `EvaluationDiagnostics::record_failure` intentionally
counts that third attempted evaluation. Only this test expectation was corrected
to three; its zero accepted points and unchanged replay assertions remain.
The original abort and the 19-pass/one-failure logs are retained unchanged.

The final focused run, `output/diagnostics/qmc-refill-scheduling-2`, passed all
20 tests with no failures or ignored tests in 3.214242293 process seconds,
exit 0 and reaped. Scoped formatting and Clippy also passed. Independent checks
verified all 172 current and archived source bindings, the two explicit
documentation/test overrides, the ten frozen bindings and the reported process
and stdout hashes. The final test binary is SHA-256
`3e28536717d9674b37a81b0fb388bb86ac0b93d23196cfd41b7628bf5621b646`.
The independent acceptance record is
`output/diagnostics/qmc-refill-scheduling-2/independent-validation.json`,
SHA-256 `d733ac4203e809c57b811f669b984c8af4578bf0295b78a139492e35958d7a09`.

This accepts the bounded CLI scheduling milestone. It does not establish
completion, convergence or performance parity for the unfinished full graph.
