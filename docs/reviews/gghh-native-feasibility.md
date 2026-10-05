# Native ggHH single-diagram feasibility

The ordinary CLI completes the genuine Standard-Model top/gluon double box at
the documented sub-top-pair physical point, using the native color-closed input
in `examples/gghh_double_box`. It saves and reloads a portable artifact, then
integrates every sector and both Laurent orders. The input's provenance and
scope are unchanged: one `(+,+)` diagram with unnormalized external color delta,
not a complete gauge-invariant amplitude. Native domain admission remains
enabled; no threshold assertion is supplied.

| Observation | Result |
|---|---:|
| Ordinary generation process wall time | 61.2853 s |
| Generation peak RSS | 270,384 KiB (264.05 MiB) |
| Sectors / dimensions / Laurent orders | 30 / 6 / `[-1, 0]` |
| Mapping / symmetry / coefficient expansion | 39.7147 / 10.0325 / 9.3049 s |
| Native SymJIT O2 compilation | 0.4529 s |
| Saved artifact size | 3,747,640 bytes |
| Separate cold inspection wall time | 0.5273 s |
| Integration process wall / native active time | 8.7809 / 8.2155 s |
| Integration artifact loading | 0.5481 s |
| Integration peak RSS | 49,252 KiB |
| Accepted sector samples | 245,760 |

The allocation uses Kuo33002, Korobov3, 1,024 points per shift, eight shifts per
sector, seed 20261005 and eight caller-owned workers. Every sector accepts all
8,192 samples; all 240 sector/shift replicas complete. The originally proposed
64-point allocation is invalid for this published catalogue and was rejected
before any QMC work. That 0.477-second preflight failure is retained separately;
the corrected allocation uses the catalogue's existing minimum without changing
the rule or periodization.

The native four-component estimate, ordered by `(-1 real, -1 imaginary, 0 real,
0 imaginary)`, is
`(0, 1.4297050282815504, 0, -33.71328010619019)`, with standard errors
`(0, 0.014733128718976767, 0, 0.34632108662346217)`. All 16 entries of the joint
covariance are saved. The pole/finite imaginary cross-covariance is
`-0.005100389448479103`; it is retained rather than treating the coefficients as
independent. Offline checks verify complete coverage, covariance symmetry,
finite values and equality of the displayed and saved native estimates.

The finite coefficient's relative standard error is 1.0273%; the pole's is
1.0305%. Native stopping is `WorkLimit`, with convergence false. Time to one
per mil is unmeasured. No independent amplitude reference is supplied. This is
a successful native capability and coarse numerical estimate, not a precision
or comparative-performance acceptance claim.

Native worker-loop costs sum to 64.2290 seconds. Dividing by accepted samples
gives 261.35 microseconds per sector sample; the largest sector mean is
306.26 microseconds (sector 7). Mean and maximum accumulated worker time per
sector are 2.1410 and 2.5089 seconds. These costs include lattice point
generation, Korobov transformation, weighted kernel evaluation, conditioning and
precision rescue, and vector accumulation. They exclude worker construction and
submission and are not isolated JIT timings. Individual maximum sample latency
was not measured. The native diagnostic reports 32,550 rescues, maximum
precision 256 bits, 90 additional weighted replays and zero failures.

Generation and integration were bounded to 180 seconds with five seconds of
grace and a 30 GiB address-space limit. Both completed below these limits; all
owned processes were reaped, with source/input/executable postchecks passing.
Release builds 2 and 3 have identical CLI and core-library bytes; build 3 binds
the corrected example source and fixture and verifies the actual dependency
owners. Its CLI uses ordinary release optimization, and numerical kernels use
the default SymJIT O2. Evidence is under
`output/diagnostics/gghh-native-cli-generation-9` and
`output/diagnostics/gghh-native-cli-integration-{1,2}`. The latter's
`assessment.json` binds the saved result, checkpoint and native process reports.
The coordinator independently verified all six artifact bindings, the saved
total estimate, complete 30-sector coverage, all covariance entries and their
standard errors, and the worker-cost sum. This acceptance is recorded in
`gghh-native-cli-integration-2/coordinator-review.json`.

The native prerequisite is met. Keep this advanced example outside the default
four-example browser walkthrough: its actual Wasm generation cost, interpreted
evaluation cost and browser responsiveness remain unmeasured. No additional
accuracy or optimization campaign is needed to establish this feasibility.
