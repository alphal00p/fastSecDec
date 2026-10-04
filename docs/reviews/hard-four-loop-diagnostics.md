# Hard four-loop positive-orthant diagnostic

The first bounded whole-integrand trial completes generation, portable SymJIT
O2 compilation and a full numerical allocation for
`examples/runs/four_loop_hard.toml`. This closes the initial generation/vector
execution gap; it does not establish convergence, an independent reference or
matched performance.

The input remains the nine-dimensional positive orthant with density
`U * F^(eps - 3)`, unit prefactor and highest requested order zero. The integer
power of U permits its signed values. No graph measure or projective-simplex
normalization is introduced. Production domain analysis identifies only F as a
singular factor: U's fixed nonnegative integer power stays in the numerator.
Consequently this production decomposition has **2760 charts**, whereas the
earlier **3496-map** geometry stress test deliberately decomposes the combined
U/F supports. Those are different workloads; their timings must not be compared
as a geometry speedup. Both cases have explicit native geometry tests.

## Identity and bounded execution

The executable is the preserved clean `561657b` release build used for the
other initial triple-box/orthant trials, with Symbolica 3.0.1 and SymJIT 2.26.4:

```
binary SHA-256: 845ad38432c4e6303c666c581736af9ca382297c55dbe3ba4911c1d092d7dad9
artifact ID: 6861bd7ada651f6900e00b9c95380db679e38ce7026ef359c22c3ef9f469429e
```

Exact argument vectors, card/U/F hashes, JSON status, process watchdog reports,
artifact, checkpoint and native saved result remain under the ignored local
directory `output/diagnostics/four-loop-hard/`. The executable is preserved at
`output/diagnostics/triple-box-offshell/fastsecdec-561657b`. Generation had a
600-second external limit; integration had a 180-second limit. Both exited zero
without timeout. Peak resident memory is process memory sampled every 50 ms.
A separate diagnostic test link overlapped the beginning of generation; these
wall times are resource diagnostics, not isolated benchmark results.

| Stage | Seconds |
| --- | ---: |
| Input | 0.001587 |
| Parameterization | 0.000122 |
| Domain analysis | 0.000169 |
| Geometry | 0.398627 |
| Mapping | 3.666345 |
| Symmetry | 4.438160 |
| Subtraction | 0.544175 |
| Laurent extraction | 0.553774 |
| Compilation | 19.470978 |
| Native generation total, including other bookkeeping | 36.080459 |
| Whole generation process | 36.754877 |

The resulting artifact retains **699 kernels** and the complete generated
real-component order vector **`[-2,-1,0]`**. Generation peak memory was
981,200 KiB. The artifact occupies about 264 MiB as JSON; filesystem compression
or sparse allocation is not counted as its serialized size.

## Full-vector numerical result

The democratic native QMC run uses Kuo38005, Korobov3, 1024 points per shift,
eight common shifts, seed `20261004`, package size 1024 and two CLI workers.
Every kernel completes its allocation: **5,726,208 / 5,726,208 evaluations**,
699 complete sectors, no evaluation failures. The saved scope is `FullIntegral`,
uncertainty is available and the production estimate is complete. The stop is
`WorkLimit`, with `converged = false` and validation `unverified`.

| Order | Mean | Standard error of mean |
| --- | ---: | ---: |
| -2 | -4.067405136814634 | 0.28758494646617144 |
| -1 | -18.169409299577108 | 1.0692199476005346 |
| 0 | -162.1952374736587 | 8.959016118757578 |

The authoritative native covariance of the mean, in that order, is:

```
[0.0827051014339507, 0.1844876701397855, 0.6094001846855066]
[0.1844876701397855, 1.1432312963468898, 7.339009447423726]
[0.6094001846855066, 7.339009447423726, 80.26396981615811]
```

The native integration loop took 95.912382 seconds; fresh artifact loading took
28.235483 seconds and the complete integration process took 124.492898 seconds.
Peak resident memory was 1,381,004 KiB. Accepted worker time totals 148.563237
seconds across the two workers. There were 1,232,525 conditioning checks,
1,160,471 rescues, 1,366 weighted checks and 555 additional replays, reaching
320-bit precision. These diagnostics preserve whole-vector weighted replay and
do not indicate independently calibrated uncertainty. Status and periodic
checkpoint costs are included; this executable predates caller-status cadence.

The historical target's three corresponding central values are
`[-3.60617208198, -16.6719719507, -149.867746145]`. They and their reported errors
were transcribed from a rounded Pathfinder report and remain uncertified.
No target was attached to this execution, no statistical eligibility has been
asserted, and neither omitted lower orders nor imaginary components are filled
with fabricated observations. A scientifically independent full-orthant
reference and prespecified multi-seed/work-scaling campaign remain required.

The independent review is recorded in
[hard-four-loop-independent.md](hard-four-loop-independent.md).
