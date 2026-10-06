# Discrete Monte Carlo responsiveness and native reuse review

Date: 2026-10-07. Independent reviewer: parameters agent. Implementation owners:
parallel agent (caller dispatch), artifacts agent (display and terminal lifecycle).
The reviewer authored only temporary probes and this review. The deferred tests
and other examples were not changed.

## Diagnosis and precision boundary

The previous CLI entered a blocking Rayon `pool.install` over one complete wave
of native global batches. It polled cancellation and emitted accepted statistics
only after every batch returned. The default 4096-point pilot batch was therefore
a long blind interval, even with one worker. Sampling profiles obtained by the
dispatch owner show active MPFR arithmetic rather than a waiting evaluator lock.
The user's existing cancelled report records 32768 evaluated/rescued points and
no numerical failures after roughly 55 seconds; it is consistent with eight
complete pilot batches returning before the interrupt was observed.

An independent release probe cold-loaded the current D05 artifact, bound its
21 parameters, and used the public `KernelSet::evaluation_context` and
`WeightedEvaluationContext::evaluate_weighted` APIs. On the same 1920 interior
points across 30 kernels, after warming each context:

| Observation | Measured value |
| --- | ---: |
| Cold artifact loading | 755 ms |
| Binding the physical point | 1.04 ms |
| Cloning 30 individual evaluator contexts | 8–9 ms |
| Weight 1 | 32.63 ms, 16.99 microseconds per point; no rescues |
| Weight 30 | 11.49 s, 5982.58 microseconds per point; 1920 rescues |

Every sampled real component was zero. The complex range-loss safeguard treats
a tiny real component of a nonzero coefficient as potential underflow whenever
the weight exceeds one. A purely imaginary D05 coefficient therefore receives
a conservative 128/256-bit comparison at every such point. There were no
additional growth-triggered replay evaluations in this probe. This explains the
large arithmetic cost; runtime parameter binding and lazy context creation are
not the stall.

No precision policy was weakened. The existing independent single-component
underflow check deliberately recovers a tiny component even when its companion
component is enormous; replacing the component check by a complex norm would
violate that behavior. Symbolica's public `ExpressionEvaluator::is_real` tests
all stored constants, not individual output components. Its native
`set_real_params` performs phase analysis, but `export_instructions` does not
expose an output-component zero/phase proof. Native
`ErrorPropagatingFloat` stores errors as `f64`; a numerically zero error is not a
structural zero proof because it can underflow too. No custom phase walker,
private serde-field inspection, dependency fork, or inferred pure-imaginary
flag was introduced. A suitable public native output-phase operation remains a
possible future optimization, subject to scientific validation.

## Caller-owned dispatch and statistical admission

The new CLI wave retains the same native task, RNG stream, worker slot and
whole-batch callback. A scoped Rayon dispatch keeps the coordinator available
for a bounded 50 ms receive wait and dashboard polling. Worker-owned atomic
observations expose selected sector, context preparation and completed
in-flight evaluations; they do not modify the native accepted snapshot.

Cancellation is checked before context preparation and again before numerical
evaluation. It exits the native callback through its existing error boundary,
with a separate caller-aborted marker. That marker is set only at a cancellation
check, so an actual evaluation failure is not relabeled as cancellation. A
cancelled prefix has neither a native complete return nor accepted replay state.
Completed siblings are still admitted in original dispatch order. Diagnostic
counts can include performed evaluations from discarded work; numerical sample
coverage and covariance remain based only on complete accepted batches.

The stop-on-drop guard also runs if a coordinator poll fails. Worker panics are
caught and reported through the completion channel, so a missing completion does
not strand the coordinator. A single native numerical evaluation is not
preempted halfway through: ordinary cancellation latency includes its remaining
time. Terminal restoration and repeated-interrupt behavior have a separate
implementation and acceptance check owned by the display agent/root.

## Native sampler, checkpoint and covariance evidence

Numerica 3.0.1 supplies `DiscreteGrid`, `ContinuousGrid`, `Sample`,
`MonteCarloRng::{export,import,jump}`, training, merge and update operations.
FastSecDec's pre-existing worker calls the native discrete/continuous sampler,
passes the root inverse probability to the evaluator exactly once, and feeds
the unweighted envelope back to native pilot training. No sampling algorithm,
random stream, grid adaptation, work-package size or integration loop moved
into the numerical library.

The session checkpoint stores proposal partitions, all task RNG seeds and
accepted records. It intentionally omits pending reservations. Restoration
starts with empty pending state, restores the scientific design and accepted
records, and `next_work` rescans missing batch IDs rather than advancing an
irreversible cursor.

An independent public-API probe aborted batch 0 after 16 evaluations, accepted
complete siblings 6, 3 and 1, and restored a production checkpoint. Missing
batches 0, 2, 4, 5 and 7 were reissued with exactly the original task/RNG
identities. Completing them in reverse order produced exactly the same mean,
full 4-by-4 covariance matrix and per-sector sample counts as uninterrupted
execution. The vector included correlated real/imaginary Laurent components;
off-diagonal signs and the expected fourfold variance relation were checked.
Reversing pilot completion order yielded the same adapted sector probabilities
and next task identities. Pilot checkpoint creation correctly failed because
pilot work must restart rather than masquerade as production.

Full global-batch vectors remain the covariance authority. Stochastic sector
marginals retain `ReplicaRelation::SharedAcrossSectors`; their covariance
matrices are not independently summed. Replay envelopes advance only after a
complete native return passes submission, and original wave order remains
unchanged. The existing replay contract does not promise bitwise equality when
the worker count itself changes; matching a fixed worker count must retain the
same scientific sequence.

Evidence is kept under ignored `output/discrete-mc-review/` (precision timings,
checkpoint results and both probe sources), with dispatch-owner profiles under
`output/integration-hang/`. These bounded measurements are diagnostic, not a
claim of converged ggHH integration or a broad performance benchmark. Final CLI
old/new numerical comparisons are recorded in the
[dispatch review](discrete-mc-dispatch.md); actual release cancellation and
terminal acceptance are recorded in the
[acceptance record](discrete-mc-results.md).
