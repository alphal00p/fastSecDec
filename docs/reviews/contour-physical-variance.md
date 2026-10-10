# Matched-work physical contour variance protocol

2026-10-10. The predeclared comparison below completed all 48 runs. At matched
coordinates, the small-cap dynamic prescriptions improve finite-vector variance
by a median factor **1.0843** over fixed lambda=8e-6, but their median
time-normalized factors are **0.8667/0.8633**: this is no overall speed gain.
Both default-cap arms fail the reference diagnostic on two seeds and remain
unaccepted. No production source was changed.

## Frozen physics and existing evidence

Start with native FK05, `diagram_02_box`, at sqrt(s)=400 GeV, mH=125 GeV,
mt=172.5 GeV, cos(theta)=4/5, incoming `++`. Original input is
`target/contour-gghh400-fixed/fastsecdec/inputs/diagram_02_box/`. Its graph
BLAKE3 is `02ba1eace574ee3739d78ac4a62b615f08e3857b89ca5988688700f4f61034bc`.
Reuse graph, model, numerator, kinematics, point and measure unchanged.

The native HEPKit reduction/OneLOop reference is
`target/contour-gghh400-fixed/hepkit/result/FK05.json`. Its stored array is
finite/simple/double pole; match by Laurent order rather than array position.
The complete native reference is

| Order | Real | Imaginary |
|---:|---:|---:|
| -2 | 0 | 0 |
| -1 | -0.7278281615698874 | 0 |
| 0 | 6.550371866775194 | -1.3247910833734284 |

No additional `1/(16*pi^2)` factor is applied: these are per-diagram values in
the existing normalized measure. The reference has no declared uncertainty;
report that as unknown. Do not compare this box against the complete MadLoop
amplitude or claim diagram poles should cancel individually.

The accepted historical fixed gate used **lambda=1e-6**, eight workers and
checked pilot/unchecked production. Its refined FK05 result is
`target/contour-gghh400-fixed/fastsecdec/refined/work/diagram_02_box.result.json`.
It required a different adaptive allocation, so neither its elapsed time nor
its final covariance is a matched-work baseline.

Accepted polynomial and sign-aware FK05 programs are under
`target/contour-gghh400-dynamic-amplitude/{polynomial,sign-aware}/fastsecdec/work/diagram_02_box.fsd`.
Both directories retain four dimension-three sectors, complete layout
`[(-1,real),(-1,imag),(0,real),(0,imag)]`, and physical source identity
`4223ea8365afc9b3c2c52cda9e8eae199dbc1f37912e8b8bb1a0a383f229f9bf`.
The accepted prescription is S=.8, L=1e-5, R=1. Each saved sector has a
structural quadratic radius equation, so equality between the two constructions
is plausible here; retain separate results rather than count them as independent
evidence for a gain. This fixture does not test higher-degree root performance.

Earlier L=1 FK05 sampling failed materially despite successful pilots. Native
192-bit map/finite-difference checks subsequently resolved extremely thin
corner structure and Jacobians as large as about 3.6e15, without finding a
derivative discrepancy. This does not validate the integrated L=1 answer.
See [physical readiness](contour-dynamic-physical-readiness.md) and
[complete dynamic amplitude](contour-gghh-dynamic-amplitude.md).

## Prescriptions and fixed work

Predeclare six arms, without tuning on production seeds:

| Arm | Prescription | Reason |
|---|---|---|
| F1 | fixed lambda=1e-6 | Historical accepted physical baseline |
| F8 | fixed lambda=8e-6 | Matches the maximum S*L of the small-cap dynamic arms |
| Psmall | polynomial S=.8,L=1e-5,R=1 | Accepted explicit physical cap |
| Ssmall | sign-aware S=.8,L=1e-5,R=1 | Same cap, distinct construction |
| Pdefault | polynomial S=.8,L=1,R=1 | Preserve the unfavorable default diagnostic |
| Sdefault | sign-aware S=.8,L=1,R=1 | Same default diagnostic for the other construction |

Eight seed pairs are fixed before execution:
`34723,92711,125231,169087,203449,250007,301747,351043`.
Every arm uses native Kuo33002, 1024 points, 16 independent shifts, Korobov3,
1024-point work packages and 256-point evaluator batches. Four sectors give
65,536 accepted sector points per arm, or 3,145,728 across 48 complete runs.
No adaptive allocation, appended shifts, growing lattices or accuracy stopping
enters this first variance experiment. A work-limit label is expected.

Execute one arm at a time with eight caller-owned workers, enough for the four
sectors and multiple shifts. Cycle the six-arm order by pair index and reverse
it on odd pairs to reduce systematic run-order effects; record the exact order.
Workers process whole native tasks and return complete vectors. Failed arms
are retained with their partial/failed status; never substitute zeros or pool
reruns. Do not change a cap separately for a seed.

Use an explicit independent 27-point grid from `{0,.5,1}^3` for each chart,
with native homotopy checks and `finish_contour_pilot`. Production uses Pilot
policy and must report zero optional production certificates. A fixed arm
failing causal admission is reported as rejected rather than silently weakened.
Each arm/pair owns fresh precision/replay state. Use the same native stability
and replay settings, with no unstable cutoff, across all arms.

## Artifact and coordinate proof

The old fixed artifact predates the current owner revisions and lacks canonical
physical source identity. Generate a fresh fixed FK05 singleton using the copied
accepted driver `target/contour-gghh400-dynamic-amplitude/fastsecdec` and original
card, bounded to 60 seconds, only after protocol approval. Preserve the old
artifact as historical evidence. No dynamic regeneration is planned.

Before production, load all three recipe programs through the public native
`ProgramArchiveReader::select(...).load_all()` path. Confirm input hashes,
current owner dependencies, generation mode/subtraction, coefficient expansion,
full layout, source identity, and native source-chart association. Compare the
undeformed coordinate maps/sector maps, target coordinate ordering, chart
representatives/permutations/multiplicities and designated F/U expressions with
native Atom equality. The deformed images/Jacobian are expected to differ.
Four equal dimension numbers alone are insufficient. If exact chart grouping
differs, stop the sector comparison and design an explicit native chart-group
correspondence; do not rewrite symmetries or pretend IDs match.

An ignored Rust driver adapts the already proved native pattern in
`examples/contour_variance.rs`: separate `QmcSession::democratic` owners with
their actual distinct mathematical content IDs, native `QmcWorker` callbacks,
`WeightedEvaluationContext`, `QmcReturn` and `ContributionReport`. A direct
rustc probe can link an existing matched rlib dependency set; record its exact
fingerprints/source boundary and executable digest. No alternate sampler,
accumulator, graph loader, CAS or root solver is introduced.

`QmcSession::stream` and `QmcSettings::plan` currently derive democratic native
coordinates from seed/rule/design, independently of the mathematical content
ID. Content IDs still fence tasks and returns. This source fact is checked in
execution: hash the exact post-transform coordinate and weight bit patterns at
the real weighted evaluator callback, along with chart group, shift, index,
start and count. Sort bounded per-package hashes canonically after threaded
completion. All corresponding hashes must match across arms; full pair hashes
must differ across the eight seed pairs. Never submit a task/return to a
different session to achieve pairing. Neither a shared seed nor matching
configuration replaces this actual-coordinate test.

## Native statistics, scientific gate and timing

Retain the complete four-component mean/covariance, exact offsets, each sector's
contribution, all complete shift counts and native `ReplicaRelation`. Democratic
QMC shares shifts across sectors: **use native global covariance**, not the sum
of marginal sector variances. Pole/finite cancellations and cross-order
covariance stay intact. No pointwise variance is presented as a QMC uncertainty.

For the finite complex block, report V=Var(mean Re)+Var(mean Im), each pair's
fixed/dynamic V ratio, and `(t_fixed*V_fixed)/(t_dynamic*V_dynamic)`. Preserve
off-diagonal covariance even though the scalar summary is a trace. Ratios with
missing, nonpositive, nonfinite or numerically unresolved variance are undefined.
Report all eight raw vectors, uncertainties and residuals before medians/ranges;
do not select successful seeds or attach an unsupported small-sample confidence
interval to variance ratios.

For each stored component, record the native-reference residual against
`max(8*reported_standard_error,1e-8)`, with the floor a declared numerical
comparison tolerance, not invented reference uncertainty. Also report relative
finite uncertainty. This is a coarse reference diagnostic, not a certified
Gaussian coverage statement or a per-mil target. Any reference failure remains
visible and prevents an accepted aggregate improvement claim for that arm.
The default-cap arms remain explicitly unaccepted diagnostics unless separately
resolved; a small reported variance with a wrong mean earns no improvement.

Separate artifact decoding/JIT, parameter binding, pilot, worker-context
preparation and production clocks. Generation is a one-off reported cost,
excluded from the production ratio. Production wall time includes native
sampling/submission and the coordinate audit; also record hash time and native
evaluator/precision timing separately. Use the same diagnostics mode for every
timed arm (Disabled initially), retaining ordinary rescue/failure evidence.
Any Aggregate repeat is an untimed or separately timed observation campaign,
not mixed into the first timing sample. No microbenchmark or eight-core speedup
claim follows from this small debug-driver run.

The first campaign has a shared **300-second elapsed / 3-GiB process-group RSS**
limit including fresh fixed generation and executable runs, a 60-second
generation cap, and a 30-second per-arm cap. Stop cleanly and retain all work if
any bound fires; unfinished pairs remain unfinished. Additional work, tuning,
None/Korobov2 transforms, or time-to-accuracy comparisons are distinct declared
follow-ups. This protocol authorizes no hidden refinement until the answer looks
favorable.

## Executed comparison and limits

The approved protocol completed without changing seeds, work, tolerances or
caps. Native chart-map/F/U/face equality passed for all four sectors in both
comparisons. Every arm completed 65,536 sector points; all 48 runs completed
3,145,728 points. The exact weighted-coordinate package hashes matched across
all six arms within each pair, and all eight pair hashes were distinct. Native
global covariance, all complete shift vectors, per-sector contributions and
each component's oracle residual are retained before descriptive summarization
in `target/generation-agent-physical-variance/report.json`. No uncertainty was
re-estimated outside `QmcSession`.

| Arm | Reference passes | Median finite variance V | Median production seconds |
|---|---:|---:|---:|
| F1 | 8/8 | 0.1470348291 | 0.0520927 |
| F8 | 8/8 | 1.694950962e-5 | 0.0530279 |
| Psmall | 8/8 | 1.566851187e-5 | 0.0661915 |
| Ssmall | 8/8 | 1.566851187e-5 | 0.0668385 |
| Pdefault | 6/8 | 204.3218774 | 0.0628423 |
| Sdefault | 6/8 | 204.3218774 | 0.0630512 |

The paired F8/small-cap variance ratio has median **1.0842564**, range
**1.0707758–1.0909066**. The paired time-normalized ratio is **0.8666813**
(0.8318634–1.2526676) for polynomial and **0.8632651**
(0.8358896–1.2512149) for sign-aware. The slight variance benefit therefore
does not provide an overall production-time benefit in this measured driver.
Compared with F1 the median variance ratio is 10031.9, but that much weaker
fixed baseline must not hide the close comparison with F8. These are
descriptive paired ratios, without an inferred confidence interval or claim of
universal dynamic superiority.

Polynomial/sign-aware complete means and covariance matrices are bit-identical
at each cap on every pair. They remain separate executions and timings, not
independent evidence: this physical fixture has the same quadratic radius
equation for both constructions.

Both default-cap arms fail the component-wise oracle on seeds **92711** and
**169087** (zero-based pairs 1 and 3). Their variance spans
0.2464612–4720.5885; broad error bars on the other six seeds do not establish a
useful default-cap result. All default-arm comparison gains remain **null**,
including seeds whose loose reference diagnostic passed. The earlier default
integration failure remains unresolved as an accuracy gate.

All pilots completed; optional production certificate counts were zero.
Across eight runs per arm, native double-float rescues were respectively
2306, 2320, 2286, 2286, 754 and 754 in the table's order. No arbitrary-precision
production point, numerical failure, unstable point or cutoff-zero fallback was
reported. Native evaluator timings include the original calls and rescue calls.

### Native evaluator cost per assigned sample

The driver already retained each worker's `context.evaluation_metrics()` and
assigned sample count. Sum f64, double-float, arbitrary and conditioning
nanoseconds over the two workers assigned to each sector, then divide by that
sector's 16,384 assigned samples. The following summaries use those native
counters, not pointwise stopwatch estimates. They include all timed precision
attempts, but exclude allocation, precision mapping, coordinate hashing,
sampling and caller work by the native timing API's definition.

| Arm | Median all-sector average (microseconds/sample) | Median maximum sector average | Largest sector average among eight pairs |
|---|---:|---:|---:|
| F1 | 1.5467 | 1.5898 | 1.6435 |
| F8 | 1.5449 | 1.6591 | 2.1607 |
| Psmall | 3.1425 | 3.2581 | 3.2902 |
| Ssmall | 3.1717 | 3.3042 | 3.3806 |
| Pdefault | 2.8286 | 2.8918 | 3.7574 |
| Sdefault | 2.8514 | 2.8951 | 3.5631 |

Every entry is an average over assigned samples, **not a maximum single-point
latency**. Complete per-sector/per-pair counters remain in the raw report and
`evaluation-timing-summary.json`. Aggregate worker hash time is about 0.0416 s
per arm; it overlaps across workers and must not be subtracted directly from
wall time. Median load/JIT time was about 0.103 s fixed and 0.176–0.179 s dynamic;
median pilot time was 0.1335 s fixed and 0.1059–0.1074 s dynamic. Binding and
context preparation are separately retained. This is a small debug-driver
measurement with native SymJIT O2, not an optimized end-to-end benchmark.

### Resource and reproducibility record

Fresh fixed generation took 7.130 s. The successful 48-arm driver took
17.417 s; a retained pre-production setup failure took 0.218 s. Total monitored
execution was **24.765 s**, maximum aggregate RSS **97,292,288 bytes**
(92.8 MiB), below the declared 300 s/3 GiB bounds. No limit fired. The initial
setup attempt parsed physical symbols before restoring native symbol
attributes, causing `model::Gf` realness redefinition. Restoring saved native
state before parsing the binding names fixed the scratch-driver ordering;
that attempt sampled no point. Its log and resource use remain recorded.

The ignored driver, build command, fixed generation, protocol card, exact map
proof and raw reports are under `target/generation-agent-physical-variance/`.
The production library was the existing coherent debug rlib
`libfastsecdec-88a3f23976bd40a9.rlib`, before the current operational-status
transport edit. It links Symbolica/Numerica
`516beb37d31af8e3d6ee321a7070f407a0b1b42d` and SymJIT
`d74993ffd76a6fc322a7bcf3963fa786783a38a8`. It is not relabelled as the newer
Python candidate source. The exact dependency fingerprints and rustc command
are retained; no Cargo graph was rebuilt for this experiment.

| Evidence | SHA256 |
|---|---|
| Copied accepted CLI | `84c4afceaaabccdc2231d7411dcbda5f1159f8577e0c7ffaf97c878a1b394a70` |
| Scratch Rust source | `43343c293195cb7323a0447311122907e661029b375a9263244fd7fb9bdf8bdb` |
| Scratch executable | `c00845334194f9fec15466af82325c4e3869944ef6922a87451b672c0650362d` |
| Exact build command/fingerprint | `8c68d973048bb6b22d99fa140d409e4e658984928e133deb2a725deac274ca2f` |
| Complete raw report | `09d94f0a68efa1069ca035305b91c3ed505977d449cc112982b33bbb521cf169` |
| Native map proof | `6dcb8880f003fba40e8b489dbc057fc27c4704cfb779f548be0334b2583d9005` |
| FastSecDec rlib | `134d2994dee06cce03af1d06d2f741815deade4c5f9cfae7a23940d18c017d3c` |
| Symbolica rlib | `d8d7eb3ba5170a9cfc5d4c6015eafd78f58e240998c5af8733568c74d7106dfc` |
| Numerica rlib | `de69167d69f775bf1dc4a7e80b0e44aad12f3fb6125db1ed7ac0b1ee76103b29` |

The runtime owner independently reviewed native source/map/factor matching,
actual coordinate hashes, full native covariance, task ownership and timing
separation. No duplicate numerical accumulator, sampler, CAS or reference
correction was found. This closes the first bounded physical variance comparison;
it does not close default-cap accuracy, higher-degree root performance,
multiloop generation memory or broad performance acceptance.
