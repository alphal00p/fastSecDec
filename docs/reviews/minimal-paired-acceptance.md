# Minimal remaining Pathfinder comparison

This is a source-only execution proposal after required capability and scientific
coverage. It reuses `output/probes/first_paired.sh` for alternating repetitions,
`eight_core.sh` for separate generation/prepared integration and an accuracy
ladder, and their existing process timer and pure-data result readers. No new
benchmark framework, evaluator, reference port or parallel executor is needed.
The current prepared on-shell correctness attempt has priority and is separate
from this representative performance block.

## Fixed representative scope

| Case | Native card under `examples/runs/` | Pathfinder card under its `examples/runs/` | Paired repetitions |
| --- | --- | --- | ---: |
| Triangle | `triangle.toml` | `dot_triangle.yaml` | 7 |
| Box | `box.toml` | `dot_box.yaml` | 7 |
| Double box | `double_box.toml` | `dot_double_box.yaml` | 3 |
| Numerator-heavy multiloop | `triple_box_offshell_rank2_numerator.toml` | `dot_triple_box_offshell_rank2_numerator.yaml` | 3 |
| Hard four-loop full orthant | `four_loop_hard.toml` | `four_loop_hard_from_U_and_F.yaml` | 3 |

Do not add the duplicate double-box U/F route: its exact density identity is
already proved. Do not replace the numerator-heavy multiloop row with an easier
one-loop numerator, or the hard full orthant with selected sectors. Keep each
card's original numerator, normalization, domain and highest requested order 0.
Use the fastest already validated supported native choice for each card and
record it explicitly. In the absence of comparative evidence, retain the shipped
`Physical` / `Original` defaults; do not mandate `NativeNamed` for all five.
The difficult on-shell capability uses that opt-in separately. Distinct valid
sector partitions remain recorded rather than forced to agree.

Use one worker on the same allowed physical CPU for both programs, alternating
native/reference order by repetition. Native and Pathfinder both use their
ordinary SymJIT O2 routes; the reference explicitly selects the already validated
complex evaluator, full-support correlated democratic QMC and disables its
optional optimized evaluators. Keep all native weighted checks/replay and all
reference precision tiers active. Compare against the same complete physical
vectors and accuracy targets, retaining actual rescue/precision/failure counts;
identical internal rescue decisions are not required or fabricated.

The working Pathfinder environment is Symbolica 2.1.0 / embedded SymJIT 2.18.6.
The native release uses Symbolica 3.0.1 with its reviewed fixes / SymJIT 2.26.4.
The isolated same-source 3.0.1 binding built, but its smoke failed at an existing
Python return-type API assumption. Preserve these explicit versions and the
[failure](reference-python-binding-alignment.md); do not turn alignment into a
porting project. Results compare the actual available programs, not identical
backend binaries.

## Reuse before running missing rows

The [first seven-pair block](first-paired-performance-independent.md) already
retains all 28 successful triangle/box processes, complete vectors and actual
work. Its median process times are 1.012243/2.884075 seconds for triangle and
1.231616/4.404233 seconds for box (native/reference). Reuse these as historical
observations and reuse their attribution; they are not current streaming-v2
persistence or separate generation/loading medians.

The fourteen [native eight-core target rows](eight-core-native-results.md) and
accepted [individual-sample diagnostic](native-sample-latency-results.md) remain
valid for their frozen versions. Do not repeat them merely to fill a table.
Pathfinder's eight-worker processes hit the retained concurrent-instance license
limit. Its eight-core comparison stays **unavailable**; neither scaling the
one-worker result nor changing its executor/license is proposed. Missing larger
native eight-core accuracy rows remain required measurements, distinct from the
viable one-worker paired block. Whole-process one-worker ratios cannot close them
or the individual-maximum comparison.

A read-only inventory found complete Pathfinder strict bundles only at
`output/benchmarks/eight-core-preparation-20261005/{triangle,box}.reference.bundle/`.
Their manifests retain explicit complex O2, orders through zero and prepared
sector IDs. They can supply existing-environment prepared integration after the
ordinary strict loader accepts their frozen sources/options. They cannot count
as new generation observations. No `manifest.json` for a complete double-box,
rank-two triple-box or hard-four-loop Pathfinder bundle was found under the
reference cache/examples outputs or current `output/benchmarks` and
`output/diagnostics` roots. The pySecDec libraries/fixtures are not substitutes
for Pathfinder bundles. The three larger bundles must therefore be generated
through its existing CLI before a prepared numerical comparison exists.

Generation repetitions start from the same frozen per-case formula-cache
inventory, copied into each fresh attempt-owned write directory; cache growth
from one repetition must not warm the next silently. Both existing reference
read roots remain immutable and bound. Such rows are labelled **generation with
available formula inputs**, not cold formula construction. Native fresh-process
local caches and reference precomputed formula assets differ and remain explicit;
no historical warm row is promoted to a cold-generation measurement. Prepared
integration uses the same frozen program artifact/bundle for a case and does not
rebuild it silently through fallback during strict loading.

## Smallest new execution block

For current-release medians, use the existing `generate` then `integrate`
commands. Seven ordinary and three expensive pairs give 23 pairs per phase:
46 timed generation processes and 46 initial prepared-integration processes
across the five rows, if none of those current medians already exists at launch.
Every repetition has a fresh output location. Generate persists a complete native
artifact or strict Pathfinder bundle; integrate loads that representation and
retains its normal result/resume state. Time both complete processes using the
same timer, including serialization and teardown. Preserve native checkpoint
and reference resumable-result costs instead of disabling safety or pretending
the file formats perform identical I/O. Internal timings are attribution only:
Pathfinder's generation record sum contains nested intervals and is not the
common elapsed-generation stopwatch.

Use Kuo33002 / QMCPy linear, Korobov3, packages of 1,024 points and sixteen shifts.
The existing reader already accepts explicit point/shift/worker arguments; its
triangle/box-only admission needs a narrow pure-data extension for the three
larger cards' actual order layouts and existing references. Reuse native result
validation/comparison and preserve the complete signed-order union rather than
synthesizing missing rows. Use 8,192 points for the ordinary fixed-work rows, retaining their established
allocation, and start expensive rows at 1,024 points. Seeds are fixed before
launch, paired by index and never selected by observed errors. Read the actual
native design/reference metadata and total physical-kernel work; equal integer
seeds do not imply identical cross-program random shifts.

The initial integration rows also provide the first accuracy observation. If
highest-order estimated relative error is above one per mille, reuse the existing
prespecified doubling ladder and include every attempted level's process cost
as **cumulative tested-process wall time to the first observed SE ≤ 1‰**.
This is neither continuously resumed runtime nor certified true-error time.
For Pathfinder, preserve the actual reported L1 prefactor-error propagation label
where it differs from a jointly estimated physical standard error. Stop a row once
the criterion and complete-vector independent checks pass; do not add a broad tuning campaign.
If a component is independently known to vanish, preserve the labelled absolute
zero check rather than dividing by a noisy mean. Fixed-work medians and accuracy
medians stay separate. Unequal sector counts make actual total work and scientific
accuracy more informative than nominal point counts alone.

Every result retains all signed orders, physical prefactor handling, native
covariance, reference reported errors with their existing interpretation,
complete/incomplete status and diagnostics. Existing independent reference
uncertainties are preserved. In particular the rank-two and hard finite targets
are not one-per-mille reference certificates; their uncertainty and the remaining
calibration limits cannot be hidden by a small estimated production error.
No new expensive external reference generation is introduced by this proposal.

## Cost summaries and honest limits

Reuse saved native contribution reports for pooled accepted worker seconds per
sample and the largest sector-average cost. Reuse Pathfinder's native evaluator
and Python timing buckets, but keep their names: they exclude globally charged
lattice/transform work that the native worker interval includes. Do not present
these unlike buckets as a matched scalar-JIT ratio. The complete process boundary
is the primary cross-program comparison; loading, point generation, reduction,
precision and persistence remain visible attribution.

An individual-sample maximum is a different metric. The accepted native weighted
callback diagnostic already records mean, observed maximum, rescue state and
clock overhead without changing the result. Its small-case evidence can be
reused; if the required larger native maximum is missing, apply that same bounded
diagnostic to an already prepared artifact. The reference's existing batch
reports do not contain individual maxima, and the ordinary native `benchmark`
command times batches at interior points rather than actual weighted QMC replay.
Neither supplies the missing statistic by division or relabelling. The required
Pathfinder maximum therefore needs a narrow development-only adaptation of its
existing timing hook around the ordinary weighted complete-vector evaluation,
retaining precision dispatch and the actual slow sample. Reuse the native
instrumented/uninstrumented result and clock-overhead control pattern; do not
replace batches with a new evaluator or build a generic instrumentation framework.
This is a concrete remaining metric, not a permanently unavailable row.

This source-only proposal awaits coordinator scheduling and makes no performance
claim. After coverage, run only missing rows under agreed finite bounds, retain failures,
report medians and paired ratios, and investigate only regressions beyond the
plan's 5% allowance. Once representative parity is demonstrated, stop.
