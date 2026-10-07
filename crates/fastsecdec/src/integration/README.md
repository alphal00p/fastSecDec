# Caller-driven integration contracts

The public `IntegrationProblem` identifies all generated coefficient expressions,
their evaluation settings, ordered `(Laurent order, real/imaginary component)`
pairs, whole-sector dimensions, and exact whole zero-dimensional contributions.
The caller supplies the evaluator and owns all worker threads and stopping loops.

Workers also accept bounded evaluator matrices through
`evaluate_weighted_batch` (QMC) and `evaluate_weighted_batch_observed` (Havana).
The caller supplies a positive chunk size and a callback over point-major
coordinates, one weight per row, and point-major complete Laurent outputs.
These chunks are execution units, not statistical replicas or checkpoint units.
Existing scalar callbacks use the same worker implementation with chunk size one.

Havana fills each chunk with the native fixed-proposal sampler. Discrete Havana
groups its rows by sector, preserving each sector's encounter order, and restores
the original global sample order for training and all vector reductions. QMC
applies native periodization before forming the matrix and inserts results into
its native partial in lattice order. A failed callback never produces an
admissible partial return. Cancellation is caller controlled; incomplete work
and its replay state are discarded. Discrete sector grouping can change which
independent sector failure is encountered first, but never admits a failed prefix.

`QmcSession` issues canonical point packages. A reusable `QmcWorker` generates
coordinates, applies the selected periodization, evaluates the complete vector,
and returns Numerica partials. Democratic uncertainty intersects complete shift
identities across every stochastic sector and sums sectors within a shift before
computing covariance. Adaptive pilots only estimate cost and variance; an
explicit frozen production allocation uses independent sector randomizations and
sums sector covariances. Pilot and production observations never mix.

`HavanaSession` reuses `ContinuousGrid`, `MonteCarloRng`, and
`StatisticsAccumulator<DoubleFloat>`. Every production batch has the same number
of importance samples and a distinct jumped RNG stream. Grids adapt only between
complete pilots and remain fixed in production. The covariance of independent
batch means is reduced through Numerica's existing vector replica-statistics
routine; individual points inside a lattice are never treated as MC samples.

The runtime reuses `DoubleFloat` for cross-sector compensated sums and avoids an
independent arithmetic implementation. This reuse was checked against the public
`numerica::domains::float` exports, `float/double.rs` and Havana accumulator source,
and the focused large-cancellation and analytic integral regressions in
`tests/runtime_qmc.rs` and `tests/runtime_mc.rs`.

Only a complete production allocation may satisfy `VectorEstimate::meets`.
Intermediate estimates can appear in snapshots once every required sector has
enough complete replicas, but they do not certify an accuracy stop. Absolute and
relative tolerances must be finite and nonnegative. A problem containing only
exact contributions has valid zero uncertainty immediately.

QMC checkpoints include validated Numerica plans and partials, the immutable
configuration digest, canonical package layout, and matching timing records.
Unfinished in-flight packages become available again on restoration. Havana
checkpoints currently support frozen production: they reconstruct validated
public grid partitions and batch summaries, avoiding serialization of private
adaptation internals. A Havana pilot can be completed or restarted before saving
a resumable production checkpoint. Checkpoints are consistency-checked data,
not authenticated worker messages.

No lattice-count extrapolation, automatic CBC search, user-owned evaluator
serialization, or library-owned parallel pool is introduced here. Higher-level
iteration policies can create another fixed production design; independent
stages must not be blended as interchangeable replicas.
