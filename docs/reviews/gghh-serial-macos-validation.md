# Fresh ggHH double-box verification on macOS

Date: 2026-10-08. Requested scope: pull latest `main`, build the release CLI,
regenerate the shipped ggHH double box with and without serial generation, and
integrate both artifacts with ordinary execution and `--serial 10`.

The pulled source is `91d2ac9f3a129d1b2999c57c8df622a06e535ee8`. The initial
release build succeeded in 3 min 55 s. Serial startup exposed a macOS transport
defect, corrected by explicitly making an accepted TCP socket blocking before
cloning it for framed worker communication. The listener remains nonblocking;
coordinator polling still uses a nonblocking channel. A dedicated reader thread
owns blocking frame reads. Socket shutdown and child reaping remain part of
cancellation and destruction.

The independent reviewer inspected the fix and its subprocess regression, which
delays and fragments two frames across their magic, length and payload
boundaries, then checks both messages and the released process slot. The fix
changes process communication, not algebra, integration rules or evaluators.
The [socket investigation](macOS-serial-worker-control.md) records the native API
probe and passing focused process tests.

## Inputs and artifacts

The shipped `examples/gghh_double_box/run.toml` and `point.toml` were unchanged.
Generation uses **symbolic**, **Taylor**, **minimal** numerator contraction,
`coefficient_series`, runtime model/kinematic parameters, exact incoming-gluon
on-shell constraints, and native **SymJIT O2**. Both runs use eight workers and
produce 30 six-dimensional sectors. The complete output layout is
`[-1, -1, 0, 0]` with components `[Real, Imag, Real, Imag]`.

Both generation modes have scientific content ID:

```text
c0df7cb1f4c47a8a0a885bc237da43c77e02aa3d02e3bf54b64ca5110dac0198
```

Their 19 runtime parameter names, dimensions and global Laurent layouts match.
Both manifests refer to an existing generation-specific binary sibling by a
relative basename. Native binary byte identity is not required: sector-local
layouts and native symbol-state encodings can differ. The ordinary archive is
19.42 MiB and the serial archive 16.93 MiB; their manifests are 104.57 KiB and
119.81 KiB, respectively.

| Generation | Wall time | Sampled parent peak RSS | Sampled parent + children peak RSS | Maximum children |
| --- | ---: | ---: | ---: | ---: |
| Ordinary, eight workers | 78.00 s | 1,980.16 MiB | 1,980.16 MiB | 0 |
| Serial, eight workers | 62.38 s | 8.17 MiB | 5,392.94 MiB | 8 |

These are single sequential observations on the user's desktop, sampled every
200 ms through `psutil`. Aggregate RSS sums owned processes, including shared
pages in each process, and may miss brief peaks. The ordinary generation used
the first release build; serial generation and all four integration rows below
used the rebuilt executable containing only the communication fix. This is not
a controlled general speed or memory benchmark. In this eight-worker workload,
serial generation substantially reduces coordinator memory but has **higher
aggregate peak RSS** than ordinary generation. It must not be presented as an
overall memory reduction. The earlier one-worker Linux result remains a
different measurement.

## Four-way integration matrix

Every row uses QMC with 4,096 points per lattice, 32 shifts, eight workers,
seed `20261008`, epsilon order zero as the accuracy target, and one explicitly
capped allocation. Relative and absolute tolerances are zero so a work-limit
stop is expected. Both serial rows use `--serial 10`.

All four commands exit successfully and report **work limit**, not convergence.
Each contains exactly **3,932,160 accepted samples**, with all 30 sectors having
32 complete shifts and 131,072 points. Complete real/imaginary Laurent means and
all 16 covariance entries are retained; covariance is finite, symmetric and
positive semidefinite within floating-point tolerance. Both real components,
their errors and covariance rows/columns are exactly zero. There are no
numerical failures, unstable samples or cutoff zeros.

The table gives the imaginary finite coefficient; the real coefficient is zero.

| Generation → integration | Whole-command wall time | Integration time | Aggregate peak RSS | Im epsilon-zero coefficient | Relative error |
| --- | ---: | ---: | ---: | ---: | ---: |
| Ordinary → ordinary | 4.12 s | 3.30 s | 110.50 MiB | 355.86109 ± 1.27175 | 0.3574% |
| Ordinary → serial | 12.51 s | 12.38 s | 206.64 MiB | 354.79884 ± 0.42622 | 0.1201% |
| Serial → ordinary | 4.51 s | 3.83 s | 93.52 MiB | 355.86109 ± 1.27175 | 0.3574% |
| Serial → serial | 13.79 s | 13.67 s | 202.92 MiB | 355.39946 ± 0.40657 | 0.1144% |

The ordinary integrations have **identical full-total means**. The largest
total covariance difference is `8.88e-16`, and the largest sector-mean difference
is `3.55e-15`; full serialized estimate equality is therefore false, with only
floating-point rounding differences observed. This is stronger evidence than
merely comparing their finite coefficients, without claiming bytewise equality.

Ordinary democratic QMC uses shared shifts and their cross-sector covariance.
Serial QMC uses independent sector randomizations and sums their full covariance
matrices. Serial task admission can assign different streams to sectors as
workers finish, so cross-mode and cross-run bitwise equality is not expected.
Across all pairs, neither imaginary coefficient differs by more than **1.020
quadrature-combined reported standard errors**. This is a useful consistency
diagnostic, not a formal independent-run significance test: some streams may
overlap between runs, and no independent analytic reference is claimed here.

Final point classification fractions, including every class:

| Generation → integration | f64 | DoubleFloat | Arb<1000> | Unstable |
| --- | ---: | ---: | ---: | ---: |
| Ordinary → ordinary | 99.047674% | 0.952326% | 0% | 0% |
| Ordinary → serial | 99.980596% | 0.019404% | 0% | 0% |
| Serial → ordinary | 99.047674% | 0.952326% | 0% | 0% |
| Serial → serial | 99.982020% | 0.017980% | 0% | 0% |

These are observed fractions, not acceptance targets. The modes have different
sampling and evaluator-state histories. Saved result contributions match the
final CLI reports. Each completed serial checkpoint has 960 reserved streams,
no pending work and 32 durably accepted replicas per sector.

The residence setting is a **minimum sampling time per loaded sector**, excluding
loading/JIT; it does not limit total runtime or reduce execution to one worker.
Explicit work limits may end a sector before ten seconds. These capped rows
therefore verify the requested flag and bounded workflow, without claiming that
every resident actually sampled for ten seconds.

## Commands and evidence

Generation commands:

```sh
./target/release/fastsecdec --plain --json --status-json \
  generate examples/gghh_double_box/run.toml --workers 8 \
  --output output/latest-main-double-box-review/normal.fsd

./target/release/fastsecdec --plain --json --status-json \
  generate examples/gghh_double_box/run.toml --workers 8 --serial \
  --output output/latest-main-double-box-review/serial.fsd
```

For each artifact origin, the integration command uses this common allocation,
with `--serial 10` added only for the serial integration row and separate paths
for each checkpoint and result:

```sh
./target/release/fastsecdec --plain --json --status-json \
  integrate output/latest-main-double-box-review/normal.fsd --full-integral \
  --parameters examples/gghh_double_box/point.toml \
  --method qmc --workers 8 --points 4096 --shifts 32 --seed 20261008 \
  --target-order 0 --relative-tolerance 0 --absolute-tolerance 0 --max-rounds 1 \
  --checkpoint output/latest-main-double-box-review/integration-normal-normal.checkpoint.json \
  --save-result output/latest-main-double-box-review/integration-normal-normal.result.json
```

Commands, input/output artifacts, sampled process metrics, status observations,
the initial failed serial startup, final result/checkpoint files and the
independent `analyze_matrix.py` review remain ignored under
`output/latest-main-double-box-review/`. The reviewer reran the analyzer and
independently checked the covariance differences, actual manifests and metrics.
No three-loop generation or integration was performed.

## Additional accuracy and continuation checks

An additional serial run uses the shipped `qmc.toml` settings, `--serial 10`,
and the serial-generated artifact, retaining the initial 4,096 points, 32 shifts,
eight workers, seed, epsilon-zero target, relative tolerance `0.001`, absolute
tolerance zero and up to eight allocations. It **reaches the requested accuracy**
and exits successfully after 62.92 s whole-command wall time (62.56 s reported
integration time). Its finite coefficient is
`i * (355.6288030296537 +/- 0.2897319889403825)`, a **0.08147%** relative error.
The complete four-component estimate and covariance remain finite and complete;
all 30 sectors are represented.

Actual status observations show more than ten seconds of sampling residence for
sectors 0–22, with their observed maxima ranging from 10.028 to 10.920 s. Later
residents may stop earlier when global accuracy is reached, as designed. Thus
this additional run exercises the actual residence behavior beyond the explicit
one-allocation caps in the matrix.

The final estimate uses **26,345,472 points** from the latest completed
per-sector allocations. Durably admitted diagnostics record **70,582,272
evaluations**, including earlier/refined allocations; current-invocation
operational metrics record **70,615,552**, additionally including unfinished
prefixes discarded when accuracy was reached. These counts are not pooled into
a single estimate. There are 1,178 final DoubleFloat classifications
(0.001669%), with the remainder f64, and no Arb, Unstable or numerical failures.
Observed aggregate peak RSS is 198.41 MiB. This remains a single execution, not
a comparative time-to-accuracy benchmark.

The exact additional command is:

```sh
./target/release/fastsecdec --plain --json --status-json \
  integrate output/latest-main-double-box-review/serial.fsd --full-integral \
  --parameters examples/gghh_double_box/point.toml \
  --integration-settings examples/gghh_double_box/qmc.toml --serial 10 \
  --checkpoint output/latest-main-double-box-review/integration-serial-accuracy.checkpoint.json \
  --save-result output/latest-main-double-box-review/integration-serial-accuracy.result.json
```

The corresponding ordinary accuracy run uses the ordinary-generated artifact
and the same shipped overlay without `--serial`. It also exits successfully with
**accuracy reached**: `i * (355.50826903643156 +/- 0.15446092610106874)`, a
**0.04345%** relative error. Whole-command time is 76.13 s (75.21 s reported
integration), with sampled aggregate peak RSS 520.83 MiB. The final estimate uses
62,914,560 points; cumulative diagnostics/operational metrics include 121,896,960
points across refinement rounds. Classification is 99.968281% f64, 0.031719%
DoubleFloat, 0% Arb and 0% Unstable, with zero failures. Its finite coefficient
differs from the serial accuracy run by **0.367 quadrature-combined standard
errors**. Both accuracy results preserve the complete four-component vector,
full covariance and all 30 sectors. Their different allocations and covariance
models preclude a matched-work performance conclusion.

Finally, each completed capped ordinary→ordinary and serial→serial checkpoint
was resumed with worker count changed from eight to one, the same statistical
settings, and a separate saved-result path. Both commands exit successfully at
the existing work limit. Full estimates, every sector contribution and retained
evaluation diagnostics are **exactly unchanged**, with **zero new operational
evaluations**. Whole-command times are 0.84 s ordinary and 0.23 s serial.
This checks completed checkpoint restoration; it does not substitute for the
separate crash/restart regressions in the serial acceptance suite.
