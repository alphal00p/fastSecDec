# Eight-physical-core observed accuracy baseline

Status: root-approved and independently source-reviewed. Fresh preparation and
all four labelled smoke processes completed. Both native rows passed; both
reference eight-worker processes aborted under the installed Symbolica instance
limit. The paired stage is not accepted and no paired timing campaign has run.
See [retained smoke outcomes](eight-core-smoke-results.md). The separately
approved native-only seven-seed continuation excludes the smoke seed and must
not be presented as a paired comparison. This is a baseline measurement, not
a lattice/default optimization.
The user's latest correction is authoritative: the target is the **largest
signed requested epsilon order**, usually the finite coefficient, rather than
the most negative pole. Triangle and box request `[-2,-1,0]`, so this campaign
targets `eps^0`. A later run requesting `eps^1` or `eps^2` targets that order.

## Observable and common stopping boundary

For these real Euclidean fixtures, record the native reported standard error
`sigma` and mean `mu` at the target order. The observed target is
`sigma <= 0.001 * abs(mu)`, with finite, nonnegative uncertainty and nonzero
mean. This is an estimated sampling error, not a certified true-error bound.
Independent reference comparisons remain a separate check, with their original
nonzero uncertainties retained. Lower coefficients and the full native
covariance remain in every result; a target-order success never hides a lower
coefficient mismatch or a numerical failure.

If the selected coefficient is analytically zero, choose and document an
absolute tolerance before running it. Do not switch coefficients or divide by
a noisy near-zero estimate. This matters for future cases, although triangle
and box finite coefficients are nonzero. The double-box `eps^-4` exact-zero
check remains a separate absolute correctness result; any future double-box
run through order zero also targets **order zero**.

Existing stopping options are not interchangeable:

- FastSecDec `VectorEstimate::meets` requires the tolerance for every component
  and `production_complete`; its CLI checks that predicate after a production
  allocation.
- Pathfinder `target_accuracy_reached` uses the sum of coefficient errors
  divided by the sum of coefficient magnitudes. Its `--progress-value-order`
  changes a display value, not this stopping rule.
- Native JSON status contains the full estimate, coverage and scope but not an
  elapsed-time field. Pathfinder's existing `iteration_callback` receives a
  complete `IntegrationResult` after the correlated iteration has been fully
  absorbed; its intermediate JSON includes native elapsed time. Mid-iteration
  display values are not an equivalent stopping boundary.

Therefore the first baseline should use a **prescribed completed-allocation
ladder**, driving the existing CLIs and inspecting their unmodified final
results. No new estimator or production stop option is needed. Each row is one
full allocation with `max_rounds=1` / `max_iter=min_iter=1`; automatic tolerance
stops are disabled for that row. The external runner records the first completed
row satisfying the target. It must not call an earlier partial snapshot a
completed allocation, interpolate a crossing, or pool estimates from different
lattice sizes.

Approved ladder: `N = 1024, 2048, ..., 1048576`, `R = 16` independent shifts per
sector, stopping that program's ladder at its first successful observation or
its predeclared time/work cap. Sixteen is an explicit campaign setting, not a
claim about the default of either program. Native Kuo33002's supported minimum
is 1024. If the first row already satisfies the target, report **“reached by the
first complete 1024×16 allocation”**, including its actual time; the earlier
crossing is unmeasured. Do not reduce the starting allocation after seeing the
answer. This coarse observation boundary is a deliberate limitation.

Use seven paired seed campaigns (`20261211` through `20261217`), alternating
which program executes first. Within one ladder, give each allocation its
prescribed seed `base_seed + 100000 * ladder_index`; preserve all previous rows
and their independent estimates. Seeds identify reproducibility within each
implementation, not equal random shifts across implementations. A single
separately labelled smoke pair at seed `20261210` precedes the campaign and is
excluded from timing summaries. The bound is 180 seconds per numerical
row, 600 seconds cumulative process wall per program/case/seed; clip each next
row deadline to the remaining budget. The existing five-second watchdog grace
is retained separately. Retain any timeout and the
last completed allocation, without replacing failures by an easier run.

The reported integration elapsed is the **sum of each program's native
integration elapsed fields through the first successful row**. Report cumulative
process wall time and cumulative load/setup time: repeated artifact loading is
real orchestration cost and must not disappear from an end-to-end number. This
is a grid-observed CLI campaign time, not the optimal continuous stopping time.
An eventual continuous driver could use the same native callbacks, but is not
needed to answer this first measured baseline and is not part of this proposal.

## Frozen implementation and hardware

Use the current frozen native release executable
`output/diagnostics/eight-core-baseline-build/fastsecdec-e8c2691`, SHA-256
`6e258a07c5064c2682f6fe67e0111eb7d6231826bfbbe5695b15f039d821fdc4`.
It uses Symbolica 3.0.1 at
`98794d0d7337ba2b08e4c046dde584ad7fc1ce10` with four documented local patches
(source state `dirty:673824754bfa02274b53b2c8c3d12b18c5289976090632ab60d09c0696a753fa`)
and SymJIT 2.26.4. Prepare fresh validated artifacts with this executable; do
not bypass the provenance checks on earlier artifacts.

Use frozen Pathfinder revision
`582d8c7f6dde9bf750750d4c2a2d85a94ce940cd` and its existing environment:
Symbolica 2.1.0 / embedded SymJIT 2.18.6, QMCPy 2.3.0, pySecDec 1.6.6. Record
actual package metadata, Python executable hash, source diff, and all commands.
The comparison necessarily includes these backend differences. Pathfinder's
real O2 box path previously failed the analytic check; retain its validated
**complex O2** route, not the incorrect faster route.

Current allowed logical CPUs are 0–383. Sysfs/lscpu identifies CPU0–7 as eight
distinct physical cores on socket0, NUMA node0; their SMT siblings are
CPU256–263. Proposed affinity is `taskset -c 0-7`, inherited by all workers,
with `--workers 8` on both programs. Revalidate and save topology/allowed cpuset
at launch; refuse fewer than eight unique `(physical_package_id,core_id)` pairs
or a disallowed CPU. Save `thread_siblings_list` and node membership. The parent
coordinator shares these eight cores; this is eight physical cores allocated,
not a promise that eight workers remain busy throughout small problems. No
other project build or timing process may overlap a measured row. Shared-host
load and lack of exclusive host reservation remain explicit limitations.

Record the existing timer's whitelist of performance environment variables;
never enumerate credentials/licensing values. Set OMP/OpenBLAS/MKL/NumExpr
thread counts to one to avoid accidental nested oversubscription. Do not force
`RAYON_NUM_THREADS=1` while claiming eight native workers; the CLI's explicit
eight-thread pool is authoritative. Record host/cgroup/CPU frequency policy
without changing the host configuration.

## Scientific and evaluator settings

Reuse the existing triangle/box native cards and physically matched reference
DOT cards, all through order zero and normalized per-loop measure already
cross-checked against native one-loop masters. Hash graph, model, parameter
card, kinematics and target sources. Do not compare sector IDs between tools.

Use native `qmc`, `kuo33002`, `korobov3`, packages of 1024 and full scope;
reference `qmc`, `qmcpy`, linear ordering, full support, democratic refinement,
correlated sectors, Korobov alpha3, batch size1024, and no optimized-QMC
component evaluators. Disable the reference point cap (`--qmc-max-samples-per-iter
0`), use exactly one iteration, `--min-error 0`, and no aggregate tolerance or
time target. Verify actual N/R and complete raw/aggregate coverage in the result;
requested counts alone are insufficient. Both routes evaluate all requested
coefficients and use native shared-shift covariance/aggregation. Do not invent
cross-order covariance where the reference does not export it.

Preserve production precision behavior. Native policy is boundary threshold
1e-3, initial128/max4096 bits, relative evaluator tolerance1e-12, absolute1e-300,
plus its existing weighted replay policy. Reference defaults use endpoint
thresholds1e-3/1e-6/1e-8 and decimal precisions32/100/1000, with its existing
maximum-weight guard. Capture the serialized policies and precision counts;
these policies differ and must not be described as identical. Do not disable
rescues to obtain a performance number.

## Runner, timing and evidence

Prepare each native artifact with the frozen CLI `generate`; prepare each
reference bundle with its native `generate --output` command. Time these
processes separately using the reviewed Rust `output/probes/process_timer`.
Reference nested generation timers overlap, so their summed `total` is not an
exclusive generation duration. Keep individual stage records as diagnostics
and compare process generation boundaries separately.

Both generation commands construct their requested evaluator route, but their
artifacts differ. Native `generate` compiles O2 and then persists canonical
expressions; `integrate` recompiles O2 during artifact loading before its driver
timer starts. Reference generation persists the hot evaluator and precision
fallback; `PreparedEvaluatorStore` reloads these lazily, potentially inside the
reported QMC integration interval. Therefore each tool's native elapsed field
is labelled with its actual boundary, not presented as identical pure-kernel
timing. End-to-end process time is the common complete boundary.

An ignored Rust coordinator or a shell coordinator plus a small Rust JSON
reader can drive the ladder. It should invoke only the existing CLIs, use the
reviewed timer for process-group watchdog/wait4 accounting, and keep every
stdout/stderr/result/checkpoint. Native `integrate` must save its typed result;
reference `integrate --output BUNDLE` must preserve its full physical final
JSON and original error convention. The reader selects `max(orders)` by explicit
layout, validates finite full vectors and completed coverage, and evaluates only
the stated scalar inequality. It performs no error recombination, covariance
reconstruction, convergence fit or hidden zero fill.

At each row save target key/value/error/ratio, actual design, all lower rows,
reported integration elapsed, external process wall/CPU/RSS, exit/deadline outcome,
artifact and build hashes, command/affinity, and first-crossing state. An optional
monotonic status receipt log can retain live evidence, but final native elapsed
and the complete allocation are the comparison boundary. Source timestamps or
file polling intervals must not be presented as exact completion timestamps.

Report warm-up, generation, measured artifact load/JIT, reported integration
elapsed and complete process as distinct measurements. Complete process time
is the common boundary for the side-by-side crossing comparison. A process
remainder after subtracting reported integration elapsed is
**load plus setup/serialization/teardown**, not pure load. Native load time is
already explicit; if reference pure-load timing is needed, use a separately
reviewed external-only wrapper around its existing loader rather than infer it
from unrelated timing buckets.

## Individual sample latency: separate diagnostic

The current per-sector `worker_seconds / used_points` is a batch-amortized mean.
Its maximum over sectors is a maximum **sector mean**, not the latency of the
slowest sample. Existing Pathfinder runtime benchmarking also reports sector
means and deliberately disables rescues, so it cannot answer the requested
production-tail question unchanged.

A bounded ignored native Rust caller can wrap each existing
`QmcWorker::evaluate_weighted` callback with `Instant` immediately around
`WeightedEvaluationContext::evaluate_weighted`. Record per-sector count, sum,
maximum, precision report and point/shift/index for the slowest observation;
retain the complete vector and normal replay acceptance semantics. This true
callback-sample measurement includes kernel evaluation, conditioning, weighted
replay and MPFR rescue, but excludes lattice generation, Korobov transform and
accumulator insertion (those occur outside the callback). Keep the ordinary
worker average beside it to disclose those excluded costs.

For Pathfinder, its native `SectorProcessor.evaluate_batch` / QMC batch route
can be exercised with one row at a time, retaining its existing precision and
weight guard. A separately reviewed external-only adapter is needed to define
the exact matching boundary: a singleton batch includes Python/array overhead
and is not equivalent to its production vectorized batch throughput. Do not
label `batch_time / batch_size` as a sample maximum. If an exactly matching
inclusive point-generation/transform boundary cannot be exposed without
duplicating algorithms, leave that maximum unavailable rather than invent it.

Run sample-latency diagnostics separately from time-to-accuracy measurements,
on a fixed disclosed set of actual QMC points with boundary/MPFR cases retained.
Use the same unchanged artifact and full vector. Compare instrumented and
uninstrumented paired full-vector/checkpoint outcomes; measure an empty clock
bracket distribution and report timer overhead without subtracting it from
individual maxima. An observed maximum depends on sample count, scheduling and
interruptions and is not a worst-case upper bound. Production profiling changes
are outside this baseline slice; individual-sample diagnostics follow it as a
separate reviewed task within the authorized implementation.

## Source anchors checked

- Native `status/mod.rs`, `integration/estimate.rs`, `integration/worker.rs`,
  `fastsecdec-cli/src/display.rs`, `driver/execution/qmc.rs`, `driver/report.rs`.
- Native Numerica `qmc/catalogue.rs`: Kuo33002 count/dimension support.
- Reference `src/integrator.py`: `target_accuracy_reached`, `integrate_qmc`,
  `_qmc_live_stats_from_aggregate`, `_stats_from_reduced_batches`, existing
  `iteration_callback`; `FSD.py::_make_integration_output`.
- Reference `src/runtime_benchmark.py`: the existing ordinary-only benchmark
  disables rescue thresholds, hence is unsuitable for a production-tail claim.

Root has approved the fixed ladder, precision/coverage criteria and watchdog
budget. The runner and actual smoke evidence have independent review; the
reference startup failure prevents paired acceptance. Native-only continuation
has separate source review and explicit qualification. The result note retains
the observed smoke values and boundaries; this protocol supplies no missing
reference measurement or unexecuted seven-seed result.
