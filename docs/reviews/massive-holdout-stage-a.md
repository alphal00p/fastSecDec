# Massive holdout Stage A: fixed two-rule convergence block

All **72 prespecified rows completed their complete allocations**, retaining
**223,838,208 accepted kernel evaluations**, full covariance, per-sector coverage,
complete shift vectors, accepted replay state and checkpoints. There were no
numerical failures, statistical failures, unavailable comparisons, timeouts or
not-started rows. No individual native comparison crossed the prespecified
absolute pull threshold of five; the largest was 3.867. This is finite-sample
agreement with the six numerical references, not a proof of their exact values.

The 8192-point HKKN rule has lower estimated uncertainty than Kuo33002 in every
fixture in this block. At 1024 points it has higher uncertainty in four of six
fixtures. These results support explicit rule selection and further scrutiny of
count-dependent rule quality; they do not justify silently changing the existing
default or checkpoint meaning.

## Frozen design and identity

The approved [campaign design](convergence-campaign-design.md) fixes six cases,
two published rules, two lattice counts (1024 and 8192), and three independent
seeds (20261101, 20261102, 20261103). Every row uses sixteen complete random shifts,
Korobov3, packages of 1024 points, one caller-owned worker and democratic
full-sector integration. Rule order rotates between seeds and count order
alternates between cases. No optional stopping, row replacement, selected-sector
support, rejected-outlier deletion or favourable rerun entered the block.

The original reference values and their nonzero standard errors were frozen
unchanged. Current production CLI generation and inspection regenerated all six
artifacts before execution. All six inner identities remain equal to the older
scientific artifacts, while their outer envelopes bind the current source and
dependency state. The independent preparation audit checked twelve successful
processes, exact outer-byte identity, reference and source hashes, and the
complete unique 72-row allocation before the first numerical child ran.

The frozen CLI SHA-256 is
`b9bcec8797df82f1a124dc607c2f6a33eb064205257bd80b36ebb07321860851`.
The separately frozen runner SHA-256 is
`e304fb622eff36d1d4c78ba503765fb1693e7ca89721493d662289984840248e`,
linked against the copied core rlib SHA-256
`c29ca3bfa77102d858fa63ef5f5ef2ae2a9bd4ccbb72d7601ca07a150ca66901`.
Actual linked dependency/build evidence, direct rlibs, link command and runner
source digests are retained under `output/diagnostics/massive-holdout-build/`.
The backend is the audited current Symbolica 3.0.1/SymJIT 2.26.4 O2 configuration;
the saved artifact and evaluator APIs retain the existing precision policy.

Each row has a 180-second numerical bound and a 240-second outer watchdog, with
a 2700-second whole-stage bound. None was reached. Every row starts with an
exclusive attempt marker, so the runner refuses to replace earlier evidence.

## Native uncertainty results

For each fixture and count, the runner aligns all 48 `(seed, shift)` vectors and
concatenates the two rule outputs before calling native
`QmcEstimate::from_shift_means`. This preserves the cross-rule covariance arising
from their matched random shifts. Counts are kept separate; vectors from 1024
and 8192 are never pooled. All twelve summaries have 48 aligned rows and no
excluded run. No new estimator or independent-error assumption between rules is
introduced by the runner.

The following are the native joint summaries' standard errors for the real
finite coefficient. These absolute-vector summaries are diagnostics; the
centered, complete-support per-session total remains each production row's
authoritative estimate. The aggregate helper's point-count fields are zero
because its input is shift means; actual work comes from the retained native
session coverage, never from those helper fields.

| Fixture | Count | Kuo33002 standard error | HKKN alpha3 standard error | HKKN / Kuo |
|---|---:|---:|---:|---:|
| kite_2loop | 1024 | 4.90441e-6 | 1.77899e-5 | 3.627 |
| kite_2loop | 8192 | 8.60104e-7 | 7.02494e-8 | 0.08168 |
| self_energy_3loop | 1024 | 3.50466e-3 | 4.81230e-3 | 1.373 |
| self_energy_3loop | 8192 | 3.48200e-3 | 5.03870e-5 | 0.01447 |
| three_point_2loop | 1024 | 4.20393e-6 | 4.50783e-5 | 10.723 |
| three_point_2loop | 8192 | 6.94064e-7 | 7.14268e-8 | 0.10291 |
| three_point_2loop_6line | 1024 | 4.40161e-4 | 2.55893e-4 | 0.58136 |
| three_point_2loop_6line | 8192 | 4.44464e-4 | 1.49335e-6 | 0.003360 |
| three_point_3loop | 1024 | 3.51388e-3 | 4.81395e-3 | 1.370 |
| three_point_3loop | 8192 | 3.49110e-3 | 5.04747e-5 | 0.01446 |
| three_point_3loop_8line | 1024 | 1.25149e-3 | 1.20723e-3 | 0.96463 |
| three_point_3loop_8line | 8192 | 5.07653e-4 | 3.33788e-4 | 0.65751 |

The six-line observation that prompted the investigation is reproduced on new
seeds: its Kuo uncertainty barely changes across the eightfold count increase,
while HKKN improves substantially. The two related three-loop fixtures show a
similar Kuo plateau. The earlier exact dual-lattice mode/control evidence is in
[the six-line investigation](six-line-qmc-convergence.md). These observations
concern valid randomized-rule variance, not a demonstrated integrand or error
estimator defect. The seven-dimensional eight-line fixture remains comparatively
noisy under both rules and is a useful unresolved convergence case.

The native HKKN 8192-point diagnostic means, with their 48-shift standard errors,
are:

| Fixture | Native mean | Native standard error | Frozen reference | Reference standard error |
|---|---:|---:|---:|---:|
| kite_2loop | -0.680876155341400 | 7.02494e-8 | -0.680876131003874 | 5.22190e-8 |
| self_energy_3loop | -1.54174988808208 | 5.03870e-5 | -1.54176264215194 | 1.05374e-5 |
| three_point_2loop | -0.704846802942738 | 7.14268e-8 | -0.704846814559949 | 5.02734e-8 |
| three_point_2loop_6line | 0.143239433576719 | 1.49335e-6 | 0.143237469818583 | 2.72545e-6 |
| three_point_3loop | -1.53871999231592 | 5.04747e-5 | -1.53872465320882 | 1.06500e-5 |
| three_point_3loop_8line | 0.233987726287382 | 3.33788e-4 | 0.234110140388027 | 1.05203e-4 |

The per-row comparison objects, including their original provenance,
normalization/kinematics evidence and independence context, are retained. The
five-standard-error investigation rule is applied to those native objects, not
to a newly invented aggregate significance calculation. Unknown off-diagonal
reference covariance is not replaced with zero.

## Precision, work and timing limits

Across the block, 7,056 full-vector weighted checks required 7,056 native rescues
and additional replays. Maximum requested precision was 256 bits, with zero
evaluation failures. Every accepted package passed the existing exactly-once
weighting and replay-state submission path. All sectors and all sixteen shifts
completed in every row.

The sum of 72 child wall times is 58.709 seconds; the largest child took 3.467
seconds. Native numerical-loop timers sum to 38.045 seconds, artifact loading to
14.731 seconds, worker preparation to 0.057 seconds, final native observation to
0.067 seconds, and checkpoint serialization to 0.083 seconds. The process
sampler's largest observed resident set is 51,076 KiB. These are diagnostic
attribution measurements with different timer boundaries and 50 ms process
polling, not a persistence-aligned end-to-end comparison with Pathfinder or
pySecDec. No 5% performance-acceptance claim follows from this block.

All evidence is under `output/diagnostics/massive-holdout-stage-a/`: immutable
`plan.json`, six preparation/inspect records and artifacts, frozen references,
72 attempt markers, stdout/stderr/process outcomes, row reports and checkpoints,
and `native-joint-summaries.json`. `output/holdout-execution.log` retains the
complete execution stream. Raw generated evidence remains ignored by Git;
this report and the independent review are the durable repository record.

The existing default remains Kuo33002. Higher-dimensional coverage, small-count
tradeoffs, difficult Laurent/numerator fixtures and the separate matched
performance campaign remain relevant before any independently reviewed default
policy change. This block does not certify higher double-box coefficients or
the projected on-shell triple-box, whose generation still exceeded its bound.
