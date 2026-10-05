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
complex evaluator, its normal boundary-support correlated democratic QMC and
enabled optimized evaluators. The old full-support opt-out had no documented
correctness obstacle justifying its use as the only final comparator. Keep all native weighted checks/replay and all
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
sector IDs. Their full-support/optimization-disabled settings remain usable
for historical attribution after ordinary strict loading, not the final default
Pathfinder acceptance route or new generation observations. No `manifest.json` for a complete double-box,
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
where it differs from a jointly estimated physical standard error. Stop a row once the reported highest-order target is observed and available
reference comparisons pass. Missing union rows do not justify more sampling;
they remain separate scientific acceptance gaps, not timing failures. The
ladder never exceeds Kuo33002’s supported 2^20 points. Do not add a tuning campaign.
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
Pathfinder maximum therefore needs a narrow ignored diagnostic using captured
actual transformed points and its existing `evaluate_batch(sector, row[None,:])`
API plus the same coefficient masking and QMC weights. Compare complete results
and precision counts with the ordinary batch path. Label this observed single-row
API latency, including Python/NumPy dispatch and native precision/rescue work;
vectorized batch time cannot yield individual maxima. Exclude lattice/transform,
shift reduction and recording from that bracket and retain clock overhead.
Reference global-prefactor convolution remains separately attributed.
The existing `runtime_benchmark.py` forced-f64 interior-point route is unsuitable.
Preserve the actual QMC policy: its weighted-rescue behavior is not the separate
Havana records path. No new evaluator, precision policy or generic instrumentation
framework is proposed. This is a concrete remaining metric, not a permanently
unavailable row.

This source-only proposal awaits coordinator scheduling and makes no performance
claim. After coverage, run only missing rows under agreed finite bounds, retain failures,
report medians and paired ratios, and investigate only regressions beyond the
plan's 5% allowance. Once representative parity is demonstrated, stop.


## Concrete ignored adaptation

Source is in `output/probes/minimal_paired/`: `cases.py` fixes the five existing
cards and supported choices; `run.py` reuses the process timer and existing CLI
commands; `native_observe.rs` reuses `read_result`, `read_reference` and native
`reference::compare`; `observe_reference.py` retains the complete Pathfinder
report and reuses its scalar formatting/comparison functions without inventing
covariance. `summarize.py` computes descriptive medians only when the complete
prescribed block is present. The sources have not executed scientific workloads. Their separate data-only
reader build and frozen plan will bind the current release libraries.

The rank-two native row selects the already validated `SingleUnitTerm` policy
with bound 32 and retains physical coefficient generation. The other rows retain
the existing physical/original route. Copied card file paths become absolute;
physical expressions and original cards stay unchanged. Current native outer-v2
artifacts are loaded by the production CLI; the reader consumes its validated
saved-result API instead of reconstructing an outer artifact or expanding its
program-byte arrays in JSON.

Prospective default Pathfinder admission checks its own scheduled-sector IDs,
group counts, raw sample target, aggregate/evaluated counts and each active
order's complete sixteen shift estimates. Work is the sum over actual groups;
zero-dimensional groups use one point per shift. It is not sectors times the
nominal point count. The existing owner sums completed group/sector vectors at
matching shift indices before estimating aggregate errors. Optimized artifacts
being prepared does not prove fused evaluation ran: correlated precomputed
coordinates can use the ordinary component path. Both the requested default and
actual diagnostic route remain visible; no reference executor is changed.

The source proposes 180-second ordinary / 1,800-second expensive generation
limits, 180-second ordinary / 1,200-second expensive numerical-row limits, and
600/3,600-second cumulative numerical ladders. These finite scheduling bounds
must be agreed before execution, not extended after seeing a row. Only ladder mode launches prepared numerical work; its first row is also the
fixed-work observation. There is no separate duplicate integration launch.
Observers run outside the scientific process interval. Existing results are
not overwritten.
Unknown or missing reference components, notably the hard-case earlier leading
zero row, remain explicit and cannot be silently padded to make a block pass.

The summary separates complete process blocks and descriptive timing medians
from numerical acceptance. Readers report matching-row residuals, complete-real
projection coverage and full complex-union coverage separately. Missing imaginary
references do not become zero references. The hard-case exact zero-through-−3
[certificate](hard-four-loop-lower-order-scope.md) is bound as separate evidence;
the native comparison still returns `MissingEstimate`, and neither its mean nor
covariance is padded. Unexpected missing orders remain unresolved. Generation
medians require completed program admission, and formula read-cache inventories
are checked for additions as well as changed files.

The corrected source review accepted the build/preparation boundary. The
standalone native data reader compiled successfully in 118.887 seconds against
the exact fullgraph-4 libraries; source and library hashes passed afterward.
`output/diagnostics/minimal-paired-reader-build-1/build-evidence.json` binds the
reader, and `output/benchmarks/minimal-paired-current-1/plan.json` binds all five
cases and 616 inputs. Preparation performed no scientific generation or
integration. The source-only Pathfinder individual-row counterpart is in
`output/probes/pathfinder_sample_latency_current/`; its invocation remains tied
to a later ordinary paired result and actual default-dispatch observation.

After the bounded-context CLI fix passed its focused gate, a fresh
`output/benchmarks/minimal-paired-current-2/plan.json` was prepared with the same
five cases and 616 bindings. Its SHA256 is
`36cebea94d5159f8775f838a15bd012098e9ec2c043a459d65001235759fc1f9`.
The CLI is `43606467dc2146a3b7b4d2a04703182eac8c6419c8a56107f996aa0b5c36c9fa`;
the build evidence records its accepted precommit overlay, byte correspondence
with commit `a1f103d`, and unchanged native mathematical libraries. The original
plan remains immutable. Neither plan has produced a paired scientific row.
