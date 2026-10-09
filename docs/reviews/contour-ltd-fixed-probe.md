# Bounded fixed-strength LTD ladder probe

2026-10-09. This is generation, recovery and numerical-feasibility evidence for
the archived physical `2L4P.b.K1` fixture. It does **not** complete its scientific
accuracy gate or establish fixed-versus-dynamic variance improvement.

## Source and execution

Input: `examples/contour/ltd/2l4p_k1/run.toml`, with the graph, kinematics,
normalization and rounded analytic reference documented by the
[fixture audit](contour-ltd-fixtures.md). The reference is
`-1.0840618909337886e-6 + 2.8682065140371712e-6 i`; no reference uncertainty was
supplied. The graph produces 186 six-dimensional source charts.

The private debug CLI was frozen from the registered implementation committed
as `55c3d7c0420a5e887f90e08bb0519d1109e29d39`. Its SHA256 is
`9546a2f28cca1145f10b120a9c6a5fb2556c02315f7c89cc8fced8e76e465d74`.
Evaluators use the default portable SymJIT O2 settings. These are debug-host
observations, not release performance comparisons. The user's release binary
and other host workloads were left untouched.

The raw artifact, copied executable, monitor scripts, logs and results remain
untracked under `target/contour-ltd-template-k1/`. A monitor sampled RSS of only
the owned CLI process and its children. All owned processes exited after every
probe. The generation monitor enforced a seven-GiB aggregate RSS cap; none of
these runs approached it.

## Generation and recovery

```sh
fastsecdec --plain --status-json generate \
  examples/contour/ltd/2l4p_k1/run.toml \
  --output /tmp/ltd-k1/integral.fsd --serial --workers 4
# After intentional cancellation, repeat with --resume.
```

The first run reached its 120-second test budget with all 186 charts mapped and
85 evaluator receipts persisted. Peak aggregate RSS was 264,671,232 bytes.
Changed-process resume completed the artifact in 48.621 monitored seconds,
peaking at 268,759,040 bytes. The sum of both monitored executions was 169.271
seconds, including cancellation/recovery overhead; it is **not** a cold,
uninterrupted generation timing. Publication produced approximately 233 MiB of
sector data plus a 442 KiB manifest and cleaned heavyweight staging records.

The earlier bounded probe stopped after ten minutes without completing its
first four mapped charts. Different development snapshots and interrupted runs
do not support a controlled speedup ratio. The new result establishes that the
existing Symbolica determinant-template optimization makes this fixed-mode
fixture practical to generate within the measured budget.

## Integration and causal checks

Every production probe used eight workers and an independently identified
16-point checked pilot. Production validation was disabled after the pilot.
The two completed runs used different sampling designs and work budgets, so
their variances must **not** be interpreted as a controlled comparison of
strengths.

| Fixed strength | Sampling | Result | Evidence |
| --- | --- | --- | --- |
| `0.001` | Serial adaptive QMC, 1,024 initial points, 8 shifts, 3 allocations per sector | `(-7.0013e-3 ± 5.1500e-3) + i(-4.4528e-3 ± 3.3208e-3)` | Work limit, unconverged; 10,665,984 evaluations; 295.943 s native elapsed; peak aggregate RSS 393,932,800 bytes. |
| `0.005` | Ordinary QMC, 4,096 points, 8 shifts, one allocation | `(-9.8617e-5 ± 8.3958e-5) + i(5.5435e-5 ± 7.3866e-5)` | Work limit, unconverged; 6,094,848 evaluations; 7.866 s native integration, 15.999 s artifact load, 37.635 s total monitored command; peak aggregate RSS 683,114,496 bytes. |
| `0.02` | Same ordinary design, pilot only | Rejected before production | Chart 79's residual `U[0]` acquired a certified negative real part; approximately 22.09 s command time. |

Reproduce the `0.005` probe with:

```sh
fastsecdec --plain --json integrate /tmp/ltd-k1/integral.fsd \
  --workers 8 --method qmc --points 4096 --shifts 8 --max-rounds 1 \
  --absolute-tolerance 0 --relative-tolerance 0.001 \
  --contour fixed --lambda 0.005 \
  --contour-validation pilot --contour-pilot-points 16 \
  --checkpoint /tmp/ltd-k1/qmc.checkpoint.json \
  --save-result /tmp/ltd-k1/qmc.result.json
```

Both completed estimates have uncertainty much larger than the analytic
reference. Neither establishes accurate agreement. They returned work-limit
results rather than claiming convergence. The `0.005` complex covariance is
`[[7.048901456058402e-9, 1.7212628955036065e-9],
[1.7212628955036065e-9, 5.456134149300203e-9]]`.
It had 44,717 double-float rescues and no failed evaluations. The native timing
counter reports 1.479 microseconds per attempted f64 kernel evaluation on
average, plus separate rescue cost; this is not a maximum-sector measurement.
Both completed probes report zero production causal checks, as required for
the selected pilot policy.

The rejected strength remained unchanged. Its reported coordinates were
`[0.777341267408845, 0.7410389864042828, 0.9801107914029802,
0.85457770468797, 0.740119819962665, 0.7495056068842351]`, with homotopy
fraction one. This demonstrates the positive-factor guard, not a global
certificate of the successful strengths or proof of a complex pole crossing.

An earlier launch with a mistyped license failed during worker startup and is
excluded from numerical evidence. One serial-monitor summary includes lengthy
JSON-log postprocessing; the table uses the native result's integration time
instead of that inflated monitor duration.

## Remaining gate

Finish certified dynamic production before another convergence campaign.
Compare fixed and dynamic recipes at matched actual coordinates, with repeated
independent seeds, complete complex covariance, separate validation overhead,
and variance times sampling cost. A wider safe displacement alone does not
establish improved convergence. The physical ladder's one-per-mil gate and the
remaining required two-/three-loop cases are still open.
