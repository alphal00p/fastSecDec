# Observed individual-sample latency, 2026-10-05

Both bounded diagnostics pass on eight distinct physical cores, using the
unchanged triangle/box artifacts and exact previous release FastSecDec library.
This measures a clock bracket around the native **weighted complete-vector
kernel call**, including conditioning, MPFR rescue and extra weighted replay.
It excludes lattice generation, periodization, partial accumulation, scheduling
and record keeping. The retained seven-seed package-amortized worker means have
a broader boundary and are a separate measurement.

| Case | Original sector ID | Accepted full-vector samples | Mean µs/sample | Observed individual maximum µs |
|---|---:|---:|---:|---:|
| Triangle | 0 | 16,384 | 0.141429 | 182.151 |
| Triangle | 1 | 16,384 | 8.380121 | 3,065.290 |
| Box | 0 | 16,384 | 7.093086 | 6,167.589 |
| Box | 1 | 16,384 | 0.138077 | 198.040 |
| Box | 2 | 16,384 | 5.843036 | 5,286.366 |

Pooled sample-weighted means are **4.260775 µs** for triangle and **4.358066 µs**
for box. All five maxima have a native precision-rescue report: 256 bits except
box sector 0 at 320 bits. Triangle sector 0 and box sectors 1/2 also replayed
after weighted checking. This identifies the observed sample's native path;
it does not attribute its entire latency to arithmetic. Scheduler interruptions
are inside the bracket, and a finite observed maximum is not a worst-case bound.
No corresponding Pathfinder individual-sample measurements exist.

## Scientific and instrumentation controls

The caller uses native QMC with Kuo33002, Korobov3, N1024, R16, package1024 and
seed20261211. It owns eight worker contexts, issues deterministic batches and
submits results in issue order. This is a diagnostic caller, not the production
Rayon scheduling policy. Point generation, transforms and statistical reduction
remain entirely in the existing native libraries.

Instrumentation-off and instrumentation-on runs have exactly equal complete
vectors, covariance, complete-shift estimates, design, accepted coverage,
precision diagnostics, accepted replay state and per-package point/output bit
digests. Only observational timing fields are excluded from that comparison.
All 32 triangle and 48 box packages are accepted. Triangle retains 32,768
evaluations, 3,221 rescues and nine additional replays; box retains 49,152,
4,492 and eight. Both have zero failures. No estimates or errors are pooled
between the two runs.

Each separate rejection control injects a caller error after seven evaluated
samples, verifies unchanged accepted observations and replay state, restores
through the native checkpoint API, and successfully retries all 1024 points.
Rejected-prefix measurements are retained separately and excluded from the
table. Maximum provenance includes task, worker slot, attempt, shift/lattice
index, transformed coordinates, weight, output vector and precision report.

Each run also retains 4096 empty clock brackets. Both upper-middle durations
are 30 ns; means are 28.976 ns for triangle and 27.413 ns for box, minima20 ns,
and maxima6431 ns/31 ns respectively. No overhead is subtracted from sample
latencies. The caller-loop off/on observations are 44.678/45.282 ms for triangle
and 77.254/62.368 ms for box. A single pair with shared-host variation cannot
estimate instrumentation overhead or establish a throughput improvement.

## Build, boundaries and retained evidence

The standalone caller was built with Rust1.98.1 `-O`, linking the exact previous
release FastSecDec rlib (SHA-256
`e1d10a56d588252b302c89f070b195721267a5cbcb035f62c4fa08d3c094a384`).
Its binary SHA-256 is
`90a3cd4dee5c8550024e1bdc63a405acbf2174556f945fc8095c39b5514c7456`.
It therefore measures the four-patch Symbolica3.0.1/SymJIT2.26.4 baseline,
not the subsequently edited native-alias or decoder-validation implementation.
Source, copied direct rlibs, release dependency inventory and original build
evidence are frozen in `output/diagnostics/native-sample-latency-build`.

The coordinator independently reviewed all three diagnostic source modules,
then verified the frozen manifest before and after execution. All other team
builds/scientific processes were stopped during the two measured calls;
unrelated shared-host work is not excluded. Each call used affinityCPU0–7,
verified as eight distinct physical cores, with OMP/BLAS threads1 and the
existing 180-second process timer. Both exited zero without timeout; process
walls were 0.349/0.291 seconds including CLI inspect, loading, both calls,
controls, provenance and output. These are not time-to-accuracy measurements.

Evidence is retained under
`output/benchmarks/native-sample-latency-20261005/{triangle,box}` and sibling
`.process` directories. Root's separate descriptive extraction agrees with
all accepted counts, weighted means, maxima, exact-equivalence flags and
rejected-prefix controls. No new sampler, uncertainty estimator or production
profiler was introduced.
