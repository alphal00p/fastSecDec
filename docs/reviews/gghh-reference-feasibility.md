# Bounded ggHH reference feasibility

Neither FastSecDecPathFinder nor direct pySecDec completed generation for the
native ggHH example within the user's ten-minute budget. Both attempts stopped
and all owned processes were reaped. No further reference attempt is planned.
The existing FastSecDec CLI result remains the only completed calculation in
this comparison; it is not independently certified by these reference attempts.

| Observation | FastSecDec CLI | FastSecDecPathFinder | pySecDec 1.6.6 |
|---|---:|---:|---:|
| Ready-to-integrate artifact | Completed in 61.285 s | Not obtained | Not obtained |
| Bounded reference process time | — | 587.533 s | 587.402 s |
| Time charged including shared preparation | — | 597.533 s | 597.402 s |
| Observed generation peak memory | 264.05 MiB | About 132.10 MiB | About 76.80 MiB |
| Eight-worker integration | 8.781 s | Not reached | Not reached |
| Finite-part relative standard error | 1.0273% | Unmeasured | Unmeasured |
| Mean worker cost per sector sample | 261.35 µs | Unmeasured | Unmeasured |
| Largest sector mean cost per sample | 306.26 µs | Unmeasured | Unmeasured |
| Individual maximum sample latency | Unmeasured | Unmeasured | Unmeasured |
| Time to one-per-mille finite-part error | Unmeasured | Unmeasured | Unmeasured |

The native result covers all 30 sectors, both complex Laurent orders `[-1, 0]`
and full covariance. Its fixed allocation is 1,024 points times eight shifts
per sector, Kuo33002/Korobov3, seed 20261005. Worker costs include lattice and
transform work, weighted evaluation, conditioning, precision rescue and vector
accumulation. See the [native calculation](gghh-native-feasibility.md).

Both references received the same complete HEPKit-contracted scalar numerator
and seven propagators: six massive top lines and the internal gluon, with
sqrt(s)=300 GeV, mH=125 GeV, mt=ymt=172.5 GeV, cos(theta)=4/5, incoming `++`
helicities and the unnormalized external color delta. This is one Feynman-gauge
diagram contribution. Internal algebra retains `D=4-2*eps` and external states
are four-dimensional. No FastSecDec Gaussian reduction, parametric numerator,
U/F polynomials, sectors or evaluators were supplied to either reference.

The indexed-momentum transport retains 247 terms, loop rank six and explicit
epsilon degree one. Native exact reconstruction, seven propagator identities
and all 180 scalar bindings pass independent review. The complete constant
`i*GC_11^4*GC_94^2`, approximately `-1.0792466937788773 i`, is factored exactly
for the reference real-polynomial interface and would be restored once on every
result coefficient. Neither attempt produced a result. Ordinary reference
numerical input rounding remains in place: maximum relative coefficient error
6.94e-16 and maximum absolute Gram-entry error 2.46e-12. Pathfinder's subsequent
Gaussian converter also retains its original binary64 and 1e-15 pruning policy;
exact input transport does not establish exact downstream term retention.

Pathfinder uses its supported pySecDec numerator reducer, iterative sectors and
direct complex SymJIT O2 configuration. Its logs establish an unfinished
generation call without identifying a narrower bottleneck. Direct pySecDec
accepts the rank-six input in about 1.65 seconds, then times out in
`loop_package` generation. No FORM subprocess or C++ compilation was observed;
the timeout is not attributed specifically to FORM.

Each reference has a separate 600-second cumulative budget. A conservative
10-second allowance covers common input preparation and failed preparation
prefixes; the provider watchdog triggers at 587 seconds. Aggregate owned-process
RSS is sampled at intervals of at least 0.5 seconds, with a 12 GB stop trigger below
the requested 15 GB ceiling and a separate 14 GB per-process address-space
backstop. The host does not permit an atomic cgroup memory cap. Reported reference
peaks are rounded sampled tree RSS; native memory is `ru_maxrss`. They describe
different measurement methods and different amounts of completed work.

These are single feasibility observations, with different CPU affinities and
unrelated host work running concurrently. Native generation uses CPUs 10–11
(about 60.66 CPU seconds); reference generation uses CPU 16. Native integration
uses eight workers on CPUs 10–17. No matched speedup, unrestricted reference
runtime or convergence estimate follows from these observations.

Independent source/input and final outcome audits pass. Both references exit
with watchdog code 124, leave no completed bundle or numerical result, and
preserve their sources and existing formula read caches. Exact commands,
transport proofs, logs, process cleanup and both reviews are retained under
`output/diagnostics/gghh-reference-feasibility-1/`, including `REPORT.md`,
`result-summary.json` and `independent-final-review.json`. Reference code and
generated diagnostic files remain untracked.
