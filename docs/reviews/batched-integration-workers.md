# Bounded evaluator batches in caller-owned integration

Date: 2026-10-07. Worker and CLI implementation slice; native evaluator reuse and
independent kernel checks are reviewed separately.

## Ownership and execution

The three FastSecDec worker adapters use bounded point-major matrices. QMC's
`evaluate_weighted_batch` and Havana's `evaluate_weighted_batch_observed` take an
explicit positive chunk size and caller callback. Existing scalar APIs adapt to
the same implementation with size one. The CLI supplies the runtime
`evaluation_batch_size`, default 256, and owns its existing worker pool. Neither
sampling sessions nor evaluator wrappers create a new pool or integration loop.

Ordinary Havana fills a chunk using native `ContinuousGrid::sample` and its
existing `MonteCarloRng`; weights and native `Sample` objects remain intact.
Discrete Havana uses native nested `DiscreteGrid::sample`, gathers each sector's
points in their original encounter order, and invokes that sector's weighted
batch evaluator. It scatters output rows back to global sampling order before
native training and full-vector/sector accumulation. Sample buffers are reused
within each package rather than cloned for each row. No custom sampler or
statistics engine is introduced.

The released Numerica API has no sampler or statistics matrix entry point.
Its sampling methods read the fixed proposal and RNG; training insertion only
updates accumulators, and probabilities/partitions change at explicit grid
update. The existing package already fixes that proposal. Sampling a bounded
chunk ahead of training therefore preserves the RNG/point sequence. QMC uses
its existing native point generator and Korobov transform before forming the
matrix; every final vector enters the existing partial in original lattice
order.

The CLI calls `WeightedEvaluationContext::evaluate_weighted_batch_controlled`
with the contiguous coordinates and weights. The kernel owner uses Symbolica's
native real/complex matrix evaluator for eligible f64 rows; higher-precision and
portable eager rows retain the native fallback. Ordered precision routing,
weighted maxima and replay remain kernel-owned. Matrix calls have explicit
invocation/point counters, so a batched API cannot conceal scalar f64 dispatch.

## Numerical and lifecycle contracts

Chunk boundaries do not change statistical replica, package or checkpoint
boundaries. MC admits only a complete native batch and QMC only a complete
native package. Existing deterministic admission, full Laurent covariance,
pilot exclusion and saved replay-state validation remain in place. The runtime
chunk size is an operational option, excluded from scientific checkpoint
compatibility; restoration reissues the original missing statistical work.

Cancellation remains polled by the coordinator at bounded waits and by the
kernel before matrix work and between ordered rows. A native evaluator call is
indivisible; a chunk is never reported as accepted work. `KernelError::Cancelled`
marks an aborted prefix and does not increment numerical failure counters.
Genuine failures retain the successful diagnostic prefix and all actual native
attempt costs, but discard the incomplete statistical return/replay candidate.
Grouping independent sectors can change which sector failure is encountered
first within a chunk; no failed prefix becomes an accepted estimate.

Diagnostics merge once per evaluator chunk. Classification counts come from
processed point reports, while actual timing/call counters come from the
context's cumulative delta, including speculative matrix rows beyond a later
failure. This prevents both timing double counting and lost work. Sector peak
contributions include only final successfully processed vectors. A discrete
matrix has one measured sector span; its shares sum to that span rather than
pretending to measure individual row latency. Integrand overhead remains
separate from actual evaluator time.

## Validation evidence

The independent owner probe under `output/batched-evaluation-review/` checks
native sample-ahead/training against interleaved sampling at 513 points. Native
points, weights, RNG state, training accumulators and explicitly updated grids
match exactly for continuous and nested discrete grids. Native f64 matrix tails
and real/complex layouts are covered by that separate review.

The ignored `output/batched-evaluation/worker_probe.rs` passed against the final
native release library. MC and discrete MC at chunk sizes 1, 7, 256, 513 and
1024 produced bit-identical point/weight streams, pilot adaptation, scientific
production-checkpoint fields, whole-vector covariance and native live snapshots
at point 257 and the end of each 513-point batch. Timing fields alone were
excluded from checkpoint equality. Sector IDs and dimensions differed, covering
nonuniform group sizes and odd tails.

QMC at the same five chunk sizes and every existing periodization (none,
Korobov 2, Korobov 3) preserved exact transformed points/weights, complete-shift
pending previews, and the complete vector/covariance after reversed package
admission. A failed callback chunk in every method left the accepted checkpoint
unchanged; restore reissued the identical task. Zero chunk size was rejected.
Evidence is in `output/batched-evaluation/worker-probe.txt`.

The existing `runtime_mc.rs`, `runtime_mc_discrete.rs`, and `runtime_qmc.rs`
integration test sources also passed unchanged (8, 5 and 16 controls respectively;
29 total), compiled with `rustc --test`
against the same final release library and its exact dependency fingerprints.
This avoided contending with the coordinating Cargo build without copying or
modifying the tests. Logs are retained as `runtime_*-tests.txt` in the same
ignored evidence directory. Permanent test files and other examples remain
unchanged.
