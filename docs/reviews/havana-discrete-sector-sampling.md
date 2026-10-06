# Native Havana sampling over sectors

The new explicit `discrete_mc` lane uses Numerica's existing `DiscreteGrid`
with a `ContinuousGrid` for each native sector. Each worker generates its own
sector selections and coordinates. A global batch includes randomly allocated
sector counts; it is not a round-robin list of fixed-sector tasks. The original
`mc`, `adaptive_mc`, QMC and their checkpoints remain separate.

Reuse was checked against the native public sample/weight/training/adaptation/
merge API, its `multichannel` and probe tests, and a focused Rust probe. The
probe exposed one essential upstream bug: `DiscreteGrid::clone_without_samples`
discarded the returned nested-grid clone, leaving its pending training samples.
The one-line assignment fix and recursive proposal/merge regression are
published in Numerica `f6ecdac8237a30adfcd1be5944a95c5160e474ce` on its existing
QMC branch/PR. Its complete native suite passed 249 tests and the portable host
suite passed 226, including documentation. Formatting and diff checks passed.
Evidence is retained in `output/diagnostics/numerica-discrete-clone-1/`.

Pilot batches train both native grids. Adaptation and the transition to production
advance independent jumped RNG streams; pilot observations never contribute to
the production estimate. Production uses a frozen proposal. The complete
Laurent-vector mean and covariance come from independent global batch means using
Numerica's existing estimator. Sector marginal means include the observed zero
contribution whenever another sector was selected. Their shared covariance
cannot be reconstructed by summing marginal covariance matrices. A batch with
no visit to a particular sector supplies a stochastic zero, not a symbolic-zero
claim or a reason to discard that batch.

`SectorSnapshot.planned_points` is now optional: historical integer quotas still
decode and serialize as integers, while randomly allocated sectors report no
fixed quota. Typed allocation metadata records the actual frozen probability
and global batch size. The global planned budget remains exact. Saved-result
validation checks the complete shared-batch coverage and total selected counts.
Old methods require their original fixed quotas. Existing checkpoints keep
unchanged settings serialization when the new optional CLI steering is absent.

Workers receive the full native inverse probability, including the selected
sector and its continuous coordinates. The weighted kernel callback applies it
exactly once. CLI replay states advance only after a complete batch and every
candidate state have been accepted. Kernel contexts are created lazily for
selected sectors and retained across pilot phases; worst-case memory is all
visited sector contexts per caller worker. No library owns a worker pool.
Production checkpoints retain frozen proposals and accepted batches; pilot
checkpointing remains explicitly unavailable, as in the existing Havana lane.

The five native discrete-MC controls passed, together with all eight existing
MC and sixteen QMC controls. They exercise nonuniform sector frequencies,
analytic full-vector means and shared covariance, pilot exclusion, independent
streams, zero-hit batches, weighted callbacks, failed-batch replay, checkpoint
restoration with reordered workers, and malformed or stale state rejection.
The first compiler-only attempt reached its 300-second bound; the cached
continuation compiled in 3 minutes 8 seconds and all 29 controls passed.
Evidence is retained in `output/diagnostics/havana-discrete-native-1/`.

The CLI passed 41 unit controls and 33 process/integration controls, including
the discrete allocation, saved-result validation and worker-count-independent
checkpoint test. Three existing heavy tests remained ignored. Two initial
boundary-process controls encountered Symbolica's legacy-license warning before
their JSON status stream; using the supplied signed license removed the warning
and the process suite passed without a code change. Failed-prefix evidence is
retained alongside the passing logs. The portable-host gate also passed all
five discrete-MC, eight existing MC and sixteen QMC controls, together with
twelve kernel/artifact metadata controls; no portable test was skipped. Its
build took 5 minutes 41 seconds, with the fixed published Numerica source selected.
The physical ggHH comparison also completed with full native covariance and no
evaluation failures, as recorded below.

CLI callers select `method = "discrete_mc"`. `points` is the number of global
samples per independent batch and `shifts` is its batch count. Optional
`[integration.discrete_mc]` steering controls native pilot points/batches/epochs,
continuous bins and density floor, maximum sector probability ratio, and the two
learning rates. The ordinary fixed-sector `mc` lane remains available.

The strongest retained QMC reference is the completed HKKN run in
`output/diagnostics/gghh-native-qmc-permil-2/`: finite coefficient
`-33.93694914257183 i ± 0.0031800967245191247 i`, reported relative standard error
`9.370602852835371e-5`, with the complete four-component vector/covariance and no
evaluation failures. This is an integration comparison at the same physical
input, not an independent analytic amplitude reference.

The ggHH run used three native pilot epochs of 4,096 global points × eight
batches, then a frozen proposal with 32,768 global points × 32 independent
batches, seed 20261008 and eight caller workers. Production accepted 1,048,576
samples; all 98,304 pilot samples were excluded from the estimate. Native sector
probabilities ranged from 0.0144928 to 0.0776358, with actual random sector counts
retained. The full four-component vector and 4×4 covariance passed the native
saved-result reader. Restoring a copy of the completed checkpoint with one worker
preserved every contribution and diagnostic without evaluating another sample.

| Complete ggHH estimate | HKKN QMC | Native discrete Havana MC |
|---|---:|---:|
| Imaginary ε⁻¹ coefficient | 1.43902754 ± 0.000136375 | 1.43757473 ± 0.000846799 |
| Imaginary finite coefficient | −33.93694914 ± 0.003180097 | −33.90461933 ± 0.019561493 |
| Finite relative standard error | 0.093706‰ | 0.576957‰ |
| Production samples over all sectors | 15,728,640 | 1,048,576 |
| CLI integration wall time, eight workers | 377.530 s | 262.477 s |

MC timing includes the pilot. These are single observed runs, not matched timing
medians or a general convergence ranking. MC's ordinary weighted precision policy
performed 1,144,364 rescues across 1,146,880 evaluations, reaching at most 256 bits,
with zero failures. The real component rows recorded zero means and uncertainties;
this observation is not a new symbolic-zero proof.

The existing native reference adapter calculated imaginary-component pulls of
−1.69383 and +1.63131 against the stronger QMC run, and +0.53327 and −0.55161
against the original coarse QMC result. These are componentwise comparisons of
independent standard errors. Both full covariance matrices remain available;
no diagonal covariance model or joint chi-square was substituted. The adapter
correctly retains `UnverifiedReference`: this is numerical consistency between
integration methods, not independent analytic validation.

The original artifact was rejected before sampling because it predates the
required Numerica fix. A fresh generation under the updated dependency identity
took 58.461 seconds. All exact evaluator program bytes, layouts, precision,
orders, offsets and cancellation data match the original; removing only the new
pre-subtraction records also makes the chart metadata identical. Its new content
ID was therefore used for MC and for an explicitly derived reference retaining
the original source, ID, coefficients and covariance. Neither original QMC
result was modified. The failed zero-sample prefix and the successful run use
less than the original 600-second integration allowance together.

Evidence is retained in `output/diagnostics/gghh-native-discrete-mc-2/`, especially
`final-assessment.json`, both native comparison documents and the checkpoint
resume result. `qualified-postflight.json` preserves the raw check and records
only two coordinator-approved non-runtime changes during integration: a test
fixture's optional quota assignment and the example README's metadata paragraph.
The archived executable, kernel artifact and scientific inputs remained unchanged;
all owned processes were reaped. The data-only Rust comparison helper reached
its 60-second compile bound, so comparison used the existing compiled CLI's
native export/read/compare path; no integration was repeated for that fallback.
