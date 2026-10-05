# Native eight-physical-core observed finite-part accuracy

All fourteen separately labelled native rows completed: seven prescribed seeds
for triangle and seven for box. Every run met the estimated 1‰ relative-error
target for the highest requested order, **eps^0**, at the first tested complete
allocation. The excluded smoke seed is not included. This does not establish a
minimal sample count or an earlier continuous crossing time.

The frozen Pathfinder eight-worker smoke processes aborted at the installed
Symbolica instance limit. There is consequently no accepted eight-core paired
timing or speedup comparison. [The smoke record](eight-core-smoke-results.md)
retains those failures. This native-only continuation was explicitly approved
and independently source-reviewed with `paired_accepted:false`.

| Observed seven-seed median | Triangle | Box |
|---|---:|---:|
| Reported integration elapsed to first complete passing allocation | 32.969 ms | 41.599 ms |
| Complete integration-process wall time | 43.068 ms | 54.016 ms |
| Separately measured artifact load/O2 compilation | 4.714 ms | 6.712 ms |
| Finite-part relative standard error | 5.006e-9 | 8.271e-7 |
| Accepted worker cost pooled over sectors | 3.884 µs/sample | 3.841 µs/sample |
| Slowest sector's accepted mean cost | 7.600 µs/sample | 5.886 µs/sample |
| Slowest original sector ID, in every run | 1 | 0 |

The last two rows are **package-amortized means**, calculated from retained
accepted `worker_seconds / used_points`. The pooled row divides summed worker
time by summed accepted points for each run, then takes the seven-run median.
The slowest row takes the maximum sector mean in each run, then its median.
Neither is an individual-sample maximum. Worker time includes native point
generation, transformation, whole-vector evaluation, precision rescue and
accumulation within accepted work packages. It is a sum of worker intervals,
not wall time divided by worker count. No timer was inserted per sample.

## Fixed design and complete scientific outputs

The runner used frozen `fastsecdec-e8c2691`, SHA-256
`6e258a07c5064c2682f6fe67e0111eb7d6231826bfbbe5695b15f039d821fdc4`,
Symbolica 3.0.1 at the recorded upstream revision with four reviewed local
patches, and SymJIT 2.26.4. It used eight worker threads with affinity CPU0–7,
verified as eight distinct allowed physical cores. No other project build or
scientific process ran during these rows; unrelated shared-host activity is
not excluded. Native defaults, evaluator precision policy and integration
algorithms were not changed.

Seeds were exactly `20261211..20261217`. Every completed first row used
Kuo33002, Korobov3, `N=1024`, `R=16`, package size 1024, and full-integral
democratic QMC. All sectors retained all 16 common shifts and 16,384 points:
32,768 accepted evaluations per triangle row and 49,152 per box row. Every result
retains all three orders `[-2,-1,0]`, their native covariance, precise effective
design, accepted contributions, checkpoint and precision counters.

The finite coefficient's relative SE ranges were 4.4568e-9..6.0604e-9 for
triangle and 5.9543e-7..9.5038e-7 for box. Every full-vector residual check against
the frozen historical target passed. Reference uncertainty and `Unverified`
eligibility remain unchanged; these residual checks do not promote a historical
target to a new certified reference. Existing independent native master and
analytic evidence remains the separate scientific basis.

Precision handling stayed active. Triangle rows recorded 3,209–3,221 rescues,
box 4,492–4,523, with additional whole-vector weighted replays retained. Maximum
precision was 256 or 320 bits depending on the seed; all rows recorded zero
evaluation failures. Counts and full vectors are retained per seed, without
averaging different estimates or pooling their error bars.

## Timer boundaries and evidence

Reported integration elapsed is the native driver interval after eager artifact
loading and pool/problem setup. It includes scheduling, accepted worker work,
observations and checkpoint I/O; complete process wall also includes loading,
remaining setup, result serialization and teardown. Loading/O2 is separately
measured. The generation observations belong to the prior preparation stage
and are not seven repeated cold-generation measurements. All results passed at
the first allocation, so the cumulative process time to the first observed
crossing equals that row's process time. The prescribed 180-second row and
600-second cumulative process budgets were never approached.

Retained data are under `output/benchmarks/eight-core-native-only-20261005`.
`summary.json` is a descriptive jq extraction from all fourteen validated
`observation.json` files; `summary-input.sha256` binds those inputs and the
small summary script. Every timed process has exact argv, affinity, source and
build identities, stdout/stderr, native result/checkpoint and reviewed
wait4/waitid timing. Input/artifact/runner hashes verify unchanged before and
after. The separately reviewed native-only runner differs from the frozen
smoke runner only in its admitted mode/approval keys and selecting the native
program. It never launches a reference process.
