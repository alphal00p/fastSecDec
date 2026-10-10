# Fixed versus dynamic contour variance protocol

Updated 2026-10-10. This records the variance-monitoring requirement in
`CONTOUR_DEFORMATION_PLAN.md` and the first public analytic measurements below.
Both dynamic prescriptions pass the native analytic admission gates. Physical
multiloop comparisons and eight-worker time-to-accuracy remain pending.

## Nonlinear cubic comparison and optional observations

The subsequent `--dynamic --cubic-cube --symjit` control uses
`F=(1-2*x)*(1+y)*(1+z)` with density `F^(-eps)/eps` on the unit cube.
Its pole is one and its finite coefficient is `3-4*log(2)+i*pi/2`.
The maintained nonlinear sampling test verifies the logarithmic primitives and
harmful cubic ray coefficient with Symbolica. Unlike the earlier linear square,
this exercises the higher-order causal envelope and iterative native root.

Eight predeclared seed pairs use 1,024 points and 16 independent shifts,
Korobov3 and the supplied `[1,433,1277]` rule. All four prescriptions retain
matching actual coordinate/weight/range digests, complete covariance and the
existing analytic-reference checks. A separate 27-point boundary/interior
pilot enables unchecked production. The core is a development build, with
SymJIT O2 programs, one worker, public Symbolica/Numerica `516beb37` and
SymJIT `d74993f`. The source is `184803d` plus the subsequent diagnostics and
cubic-harness increment. These are not release or physical benchmark results.

| Prescription | Median finite-coefficient variance | Median production seconds | Median inclusive evaluator µs/point |
|---|---:|---:|---:|
| Fixed `lambda=0.05` | `5.70986e-8` | 0.1374 | 0.873 |
| Fixed `lambda=0.25` | `3.87447e-10` | 0.1381 | 0.909 |
| Polynomial dynamic, `S=0.8`, `L=R=1` | `5.41690e-8` | 0.2502 | 7.480 |
| Sign-aware dynamic, same settings | `5.49004e-8` | 0.2774 | 9.106 |

The variance-gain ratio against fixed `0.25` has paired median **0.007236**
(range 0.006473–0.008128) for polynomial and **0.007139**
(0.006389–0.008013) for sign-aware. Thus the default dynamic prescriptions
have about **138–140 times higher variance** on this control. Their median
elapsed-time-normalized gains are 0.00404 and 0.00357. All eight pairs passed
the declared reference checks; no seed was discarded or cap silently changed.
This unfavorable result remains part of the comparison alongside the earlier
linear-square improvement. Exact causal safety does not ensure reduced variance.

Production timing includes coordinate auditing and caller/session work; native
evaluator timing includes precision retries. There is one stochastic sector,
so the average and maximum *sector mean* coincide; no maximum individual-point
latency was measured. Generation took 0.0506 seconds for the shared fixed
recipe, 0.1844 for polynomial and 0.2255 for sign-aware, with artifacts of
4,372, 112,897 and 139,979 bytes respectively. These small timings are affected
by host load and are not eight-core time-to-accuracy measurements.

Repeating all 32 runs with `--diagnostics` gives **bitwise-identical means,
covariances, mathematical identities and coordinate/weight digests**. The first
dynamic pair records 16,515 actual callbacks, including precision retries, for
16,384 production points. Polynomial reports 94,775 native solver iterations;
sign-aware reports 94,500. Their observed strength ranges are approximately
`[0.35785,0.8]`, exceeding both fixed controls. Observations are optional and
disabled by default, distinct from causal-validation policy. Their values are
approximate diagnostic centres, not certified enclosures or accepted-sample
counts.

Raw paired reports and their summary remain ignored under
`target/contour-variance-cubic-owner516*`. The harness records the complete
native results; the summary records the executable digest and source boundary.

## Questions and controlled comparisons

The primary question is whether dynamic strength reduces the uncertainty of the
last requested **complete complex Laurent coefficient** for the same production
work. The last coefficient means the largest signed epsilon order, often zero.
The second question is whether that reduction pays for computing the radius,
its derivatives and the modified Jacobian. A larger displacement or a smaller
pointwise peak does not answer either question.

Run two distinct experiments:

1. **Matched production design:** freeze the sector allocation, lattice and
   transform, number of complete replicas, precision policy and production
   proposal. Compare fixed and dynamic prescriptions on equal work. This
   isolates the effect of the contour on the statistical problem.
2. **Time to accuracy:** permit each prescription to use the same documented
   adaptive/refinement policy, measure end-to-end integration on eight workers,
   and stop through native `AccuracyTarget::LaurentOrder(k)` at relative
   uncertainty `1e-3`, with a declared absolute tolerance. This measures useful
   performance, including different sector allocations and precision rescues.

Do not compare adaptive final results as if they had equal work. A run ending
at a work limit remains unconverged and is reported with its last complete
estimate, observed work and elapsed time.

## Frozen inputs and tuning

Record graph/kinematic identity, requested Laurent orders, decomposition,
subtraction strategy, generation backend, recipe version, all runtime
parameters, precision settings, SymJIT version/O2, hardware and worker count.
Match sector charts by their retained native mathematical metadata, not only
their display IDs. Deforming a density can change which exact symmetries are
available: when the two catalogues differ, compare native chart groups with
equal multiplicity, and mark unavailable one-to-one sector comparisons. Never
change a valid symmetry proof just to manufacture matching sector numbers.

Record fixed lambda and dynamic `S`, `lambda_cap`, displacement cap, envelope
variant and regularity parameter. Include the documented defaults and a
pilot-tuned comparison. Give each prescription the same tuning budget, state
the candidate settings before evaluating them, and freeze the chosen settings
before production. Pilots used for tuning and causal preflight are not
production samples. A fixed candidate failing its causal checks is recorded as
rejected; it is never silently reduced during production.

The initial suite should progress from the existing physical B0/C0/D0 and
endpoint-subtraction controls to the complete one-loop `gg -> hh` at 400 GeV,
the physical double box, then the required LTD two- and three-loop cases.
Keep None/Korobov2/Korobov3 as separate comparisons. Changing the outer
transform changes the weighted integrand and cannot be concealed in a
fixed-versus-dynamic contour ratio.

Include a stationary-point conditioning control before interpreting physical
variance results. With dimensionful F, gradients and Hessians can be large. At
a stationary nonzero F, `v=0` makes the displacement constraint vanish and the
radius can approach L, while the Jacobian still contains `-i*lambda*D v`.
For the exact quadratic toy `F=-1+K*(x-1/2)^2`, the center has
`u=1`, `lambda=S*L`, and `J=1-i*S*L*K/2`. Large K therefore permits a narrow,
large Jacobian feature despite exact causality and zero center displacement.
Once the callback is wired, exercise that native analytic control and its
complete complex integral, not just the radius/causal inequality. Report
defaults L=R=1 alongside cap choices made by equal-budget independent pilots.
Do not assume the causal proof or the displacement cap controls Jacobian
conditioning or automatically improves convergence.

Use at least eight predeclared independent seed pairs for the inexpensive
controls. Start expensive cases with four pairs and label that evidence as
limited; expand only when justified by observed variation and resource limits.
Alternate execution order to reduce temperature/load effects. Report all pairs,
including failed and slower dynamic runs, plus medians and ranges. Do not select
the best production seed or attach an unjustified normal/F-distribution
confidence interval to a small sample of variance ratios.

Within one fixed/dynamic pair, use the same production coordinates and matching
replica indices in **separate integration sessions** to reduce comparison noise.
The mathematical integral identities remain distinct. Verify actual coordinate
sequences: equal seed integers alone do not prove equality when a scheduler
includes the integral identity in its stream derivation. If necessary, a bounded
caller-owned comparison replays the same native sampling task against each
prescription without merging their reservation ledgers or statistical epochs.
Across pairs, use independent reserved streams. Pairing is intentional common
random numbers for a comparison; fixed and dynamic outputs must never be
pooled as independent replicas of one result. In each session, retain the
existing reservation/lease protections against duplicated worker sampling.
The contour calculation must consume no production RNG draws.

## What is meant by variance

Retain the native flattened vector of real and imaginary Laurent components
and its complete covariance of the mean, `C`. For target order `k`, take the
real/imaginary block `C_k`. The scalar diagnostic consistent with native
accuracy assessment is

```text
V_k = trace(C_k) = Var(mean Re I_k) + Var(mean Im I_k)
relative RMS uncertainty = sqrt(V_k) / abs(mean I_k)
```

Keep the off-diagonal real/imaginary covariance and cross-order covariance in
the result even though this scalar summary uses a trace. They are necessary
for correlated projections, coefficient combinations and amplitude-derived
observables. Pole cancellations are checked on the complete vector. Keep
exact offsets in the native total, with zero sampling covariance; sampled
zeros and missing estimates are never converted to exact contributions.

For each matched sector and the authoritative global estimate, report

```text
variance gain                  = V_fixed / V_dynamic
work-normalized gain           = (W_fixed * V_fixed) / (W_dynamic * V_dynamic)
elapsed-time-normalized gain    = (t_fixed * V_fixed) / (t_dynamic * V_dynamic)
```

Values greater than one favor dynamic strength. Equal-work comparisons have
`W_fixed = W_dynamic`; still retain actual work in the data. `W` counts accepted
production sector-point evaluations, with the complete vector evaluated once
per point. Record discarded work, pilots, internal face evaluations, radius
calls and precision replays separately; none is free in the timing comparison.
Use native replica/point counts and do not multiply work by the number of
Laurent outputs. For serial/adaptive runs preserve the per-sector work vector.

If a variance is missing, zero, underflowed or not statistically valid, the
ratio is undefined; report its status rather than infinity or a fabricated
gain. Near-zero target means use the declared absolute criterion instead of a
relative-error division. `t*V` is a local observed efficiency diagnostic, not
a promise of inverse-time variance scaling at different lattice sizes.

### Shifted QMC

For a fixed lattice of `N` points and `M` independent shifts, each complete
shift produces a vector estimate `Y_r`. The native covariance of the mean is
the sample covariance of the `Y_r` divided by `M`. Thus `M*C` is a
replica-covariance diagnostic **only when those replicas have the same design**.
At equal `N` and `M`, compare `C` directly. Record the effective lattice rule,
actual point count, shifts, transform and sector allocation; requested point
counts alone do not establish equality.

Ordinary democratic QMC deliberately shares shifts across sectors. Its global
covariance includes inter-sector covariance and must come from
`ContributionReport.total` / `QmcSession::estimate()`. Summing marginal sector
variances would erase cancellations and can reverse an apparent gain.
Independent-sector serial/adaptive QMC uses the native sum of complete sector
covariance matrices. Preserve `ReplicaRelation` in every comparison.

Do not combine nested coarse/fine lattices as independent evidence. Compare
one complete statistical epoch at a time. With fixed-size refinement, appended
shifts must be new identities; with growing lattices, retain the previous
complete estimate only as a separate epoch. Live partial-lattice estimates
are progress information, not inputs to variance ratios.

Pointwise variance of the transformed integrand is a separate diagnostic.
Variance among the correlated points of one lattice, divided by `N`, is not
the QMC uncertainty. If pointwise variance is useful, use a separate native
uniform-random diagnostic sample and report the variance of the weighted
integrand, including the outer transform Jacobian. Do not add unconditional
per-point diagnostic accumulation to the production hot path. A divergent or
poorly stabilized second moment cannot be described as a measured finite
variance merely because a finite diagnostic sample exists.

### Havana MC

Use the existing Havana production batches with distinct jumped streams and a
frozen proposal. The relevant integrand is the fully weighted `f(x)/p(x)`,
including all contour and outer-transform Jacobians. Compare the authoritative
covariance of complete batch means, not adaptive pilot observations.

For the matched-design experiment, freeze one common valid proposal before
production so that common uniform draws generate the same points for both
prescriptions. Separately report workflows in which each mode trains its own
proposal with equal pilot budgets. Those measure the combined effect of
contour and adaptation. Per-sector Havana and ordinary discrete MC remain
distinct statistical modes; discrete-sector covariance comes from its native
global batches and cannot be reconstructed by summing marginal sectors.

## Timing and validation policy

Compare identical validation policies first. Use checked-pilot / unchecked
production as the main sampling-cost comparison, with separate `always`
measurements to expose numerical-certification overhead. Record actual pilot
coverage and outcome. `off` benchmarks do not acquire a certificate by being
close to a checked run. Changing the prescription or kinematics invalidates
old pilot evidence.

Separate generation, artifact writing, loading/JIT, independent preflight,
adaptation, production wall time and total elapsed time. Retain process CPU
time and summed worker elapsed time as different quantities. Include aggregate
RSS and sector residency for serial runs. A warm-kernel microbenchmark can
isolate root/Jacobian cost but cannot replace complete-run measurements.

Report each sector's evaluator time divided by evaluator calls, its inclusive
worker time per production point, the maximum **sector mean** cost, and the
point-weighted mean over sectors. Existing timing counters do not retain the
maximum latency of an individual sample. Label these statistics accurately;
an actual worst-sample latency needs a separately instrumented bounded run.
Similarly, maximum weighted contribution is an amplitude diagnostic, not
maximum evaluation time. Record dynamic root iteration/radius/displacement
diagnostics when the native dynamic implementation exposes them; these are
currently missing and must not be inferred from generic evaluator time.

## Existing native owners and implementation boundary

This is a caller-side measurement protocol, not a new integrator, statistics
engine, random generator or CAS. The source/API review found these reusable
owners:

| Need | Existing owner and source |
| --- | --- |
| Complete vector/covariance and target | `VectorEstimate`, `AccuracyTarget::LaurentOrder`, `Tolerance`; `integration/estimate.rs`, `integration/accuracy.rs` |
| Sector estimates, exact offsets, covariance relationship | `ContributionReport`, `SectorContribution`, `ReplicaRelation`; `integration/contributions.rs` |
| Stable democratic total and complete shifts | `QmcSession::estimate()` and `complete_shift_estimates()`; `integration/qmc/results.rs` |
| QMC replica covariance | `fastsecdec_qmc::QmcEstimate`, retained through native sessions |
| Frozen MC proposals and independent batches | Numerica `ContinuousGrid`, FastSecDec `HavanaSession`; `integration/mc/` |
| Serial centered vector sufficient statistics | Existing `integration/serial/moments.rs`, using Numerica `DoubleFloat`; no parallel accumulator |
| Scientific results and provenance | `results::read_result`, CLI `IntegrationReport` / saved-result format, native `QmcDesign` |
| Work and sector cost | `OperationalMetrics`, `SectorOperationalMetrics`; `integration/live.rs` |
| Evaluator/precision counters | `EvaluationDiagnostics`, `EvaluationTimings`, `EvaluatorTiming`; `status/diagnostics.rs`, `kernel/timing.rs` |
| Validation settings, pilots and observed checks | `ContourRunReport`, `ContourPilotProvenance`; `status/contour.rs` |

`complete_shift_estimates()` supplies diagnostic absolute binary64 vectors.
Do not recompute the authoritative covariance from them: native estimation
centers sectors before summation to preserve small errors beside large exact
offsets and cancellations. Numerica's scalar statistics accumulator does not
replace the existing full-vector covariance path. No missing algebra or
statistics operation is asserted here, so no new helper or probe is needed
for this document. Any later new accumulator or confidence-interval feature
still requires the normal API/source/executable-probe audit.

Store one compact row per case, prescription, seed pair, target and complete
epoch, alongside the retained native result. Include means, full covariance,
sector work/covariance, all settings, validation evidence and timing boundaries.
Publish a small summary table of matched-work variance gain, time-normalized
gain, time to `1e-3`, sample cost and failures; keep raw timing logs/build
artifacts ignored. Audit the consumer with an existing large-cancellation
control and a complex correlated-vector control before trusting its ratios.

## Read-only readiness review: full one-loop ggHH at 400 GeV

This subsection records the original preparation review. Its fixed-mode work
has since been completed in
[`example/gg_hh_one_loop_ME/threshold`](../../example/gg_hh_one_loop_ME/threshold/),
as recorded in the [acceptance ledger](contour-phase-b-progress.md). The dynamic
comparison at that point remains pending.

The existing `example/gg_hh_one_loop_ME` is a completed **300 GeV** reproduction,
including two Higgs-exchange triangles, six top boxes, the coherent finite
amplitude, pole checks, HEPKit Ward checks and an independent MadLoop reference.
It is a useful reusable workflow, but it is not yet a tested 400 GeV fixture.
Changing its top-level `point.json` alone would change MadLoop while leaving
native inputs at 300 GeV.

Required narrow preparation changes before running the new point:

- Parameterize the existing native point builder in
  `crates/fastsecdec/examples/gghh_double_box/point.rs`. It currently hardcodes
  energy 150, the outgoing `sqrt(11)` momenta and incoming helicity momenta.
  Preserve its 300 GeV default for the double-box example. At 400 GeV use
  incoming `(200,0,0,+/-200)` and outgoing
  `(200,+/-15*sqrt(39),0,+/-20*sqrt(39))`, with opposite correlated signs.
  These retain `mH=125` and `cos(theta)=4/5` exactly. Continue using native
  `FourMomentum`, the native wavefunction primitive and Symbolica expressions.
- Let the one-loop exporter select that point and record it instead of its
  hardcoded `sqrt_s:300.0` manifest entry. Export a fresh separate directory;
  do not overwrite the accepted 300 GeV cards/results. The exporter currently
  accepts only one fresh output-directory argument and does not read
  `point.json`. Regenerate `point-exact.json` and `point.toml` consistently.
- Parameterize the reproduction shell paths, or invoke native executables
  directly on the new input directory. `fastsecdec/run.sh` currently reuses
  an existing manifest in its fixed 300 GeV location and supplies no contour
  flags. Generate contour-capable artifacts once and bind fixed strengths at
  integration time. Strength tuning is required; the dimensionful raw F
  convention gives no reason to assume that lambda 0.1 is suitable here.
- HEPKit's executable already accepts `INPUT OUTPUT [--ward1|--ward2]` and
  reads exact products and models from the selected native catalogue. It uses
  the existing `oneloopreduce` / OneLOop complex reference path. Its shell
  wrapper hardcodes the original input directory, so use the executable on
  the new directory. No new reduction or scalar-master implementation is
  required. Above-threshold references and Ward checks still need execution.
- MadLoop reads physical momenta from its parent `point.json`, writes
  `PS.input`, and verifies model parameters against the existing card. Keep
  a separate fixture tree with the 400 GeV point; the same masses/couplings,
  process and helicity driver apply. Its Fortran driver currently sets
  `MU_R=300D0`; make that scale explicit and consistent in new provenance.
  The finite complete loop-induced amplitude should be scale independent
  after pole cancellation, but this does not justify concealing mismatched
  scale conventions. Preserve its ten-minute / 15-GiB process-group limits.
- The result summarizer already accepts `INPUTS WORK OUTPUT.json`, and the
  three-method comparator accepts an optional reproduction root. Retain
  complete complex covariance, eight independently seeded diagrams and the
  existing `1/(16*pi^2)` normalization. Do not impose the below-threshold
  result's vanishing imaginary part on the physical point.

After the above input adaptation, the existing executable interfaces allow
the following command shape (paths are illustrative; `INPUT`, `WORK` and
`ROOT400` must denote the new fixture, and `LAMBDA` a frozen checked value):

```bash
./target/release/fastsecdec generate INPUT/diagram_00_triangle/run.toml \
  --contour --workers 1 --output WORK/diagram_00_triangle.fsd
./target/release/fastsecdec integrate WORK/diagram_00_triangle.fsd \
  --parameters INPUT/diagram_00_triangle/point.toml \
  --contour fixed --lambda LAMBDA --contour-validation pilot \
  --workers 8 --points 4096 --shifts 32 --target-order 0 \
  --relative-tolerance 0.001 --seed 78139 \
  --save-result WORK/diagram_00_triangle.result.json
./target/debug/examples/gghh_one_loop_reference INPUT ROOT400/hepkit/result
./target/debug/examples/gghh_one_loop_reference INPUT ROOT400/hepkit/ward1 --ward1
./target/debug/examples/gghh_one_loop_reference INPUT ROOT400/hepkit/ward2 --ward2
./target/release/examples/gghh_one_loop_me summarize \
  INPUT WORK ROOT400/fastsecdec/result.json
./target/debug/examples/gghh_one_loop_compare ROOT400
```

The integration line is repeated for all eight manifest entries with their
recorded distinct seeds. A `1e-3` target on each diagram does not guarantee a
`1e-3` target on their cancellation-sensitive coherent sum; the existing
summarizer/comparator must check the latter and trigger tighter per-diagram
work when necessary. Build binaries through the existing licensed `nix-shell`
commands. Run the copied MadLoop harness from its new fixture's `madloop`
directory using its documented matching compiler and private installation.
None of these 400 GeV production commands has been executed in this readiness
review.

## Caller-owned comparison harness API

The executable `cargo run -p fastsecdec --example contour_variance --locked`
retains a small fixed/fixed smoke control. `--dynamic` selects both fixed baselines
and both native dynamic constructions with eight predefined independent seed
pairs, 1,024 points and 16 shifts. It uses the public generation, restoration and
binding path, so it cannot bypass an unfinished dynamic admission gate. Optional
`--symjit` and `--validation=always|pilot|off` choose the native backend and
checking policy. These are single-worker analytic controls, not eight-worker
physical acceptance measurements.

The reporter compares complete native designs, full Laurent layouts and hashes
of every actual coordinate, weight and sampling range. It alternates execution
order between seed pairs, rejects returns crossing mathematical identities,
retains complete covariance and collects validation statistics from the resident
evaluation contexts. The target is the largest signed epsilon order. Undefined
variance ratios are null. Production wall time includes coordinate auditing,
whose elapsed time is reported separately; the reported ratio must not be
mistaken for uninstrumented sampling performance.

Generation is keyed by native `ProgramRecipe`: changing only the bound fixed
strength reuses the same saved optimized bytes. The generation report contains
one entry per recipe, so the fixed/fixed smoke does not charge redundant symbolic
generation to a second strength. Fixed, polynomial and sign-aware comparisons
still require their respective mathematical programs.

The 2026-10-10 native eager rerun passes both example regressions and the complete
fixed/fixed smoke. Its report contains exactly one generation record, two
independent seed pairs and two complete prescriptions per pair; all actual
coordinate/weight ranges match. These checks include the current v11 admission
guards. Raw reports remain in ignored `target/contour-variance-single-artifact-*`
files. This is still a fixed-mode API control, not measured dynamic improvement.

### First dynamic measurements, 2026-10-10

The public three-test analytic gate now passes both constructions, all eight
endpoint cases and the threshold bubble. The comparison harness subsequently
passes three example tests and executes all eight predeclared seed pairs with
four prescriptions: fixed strengths 0.05 and 0.25, polynomial and sign-aware
dynamic strength with `S=0.8`, `L=R=1`. Artifacts are generated once per recipe,
restored and rebound through public APIs. This run uses consumer Symbolica
`1e1cb169` and SymJIT `d74993f`, before the PR #60 native cache improvement.

The two-dimensional control is
`(1/eps) integral_[0,1]^2 (1 - 2*x - 3*y - i0)^(-eps) dx dy`.
Its finite coefficient is `3/2 - (7/3)*log(2) + (11/12)*i*pi`; a native
Symbolica mixed-derivative check verifies the reference primitive. Production
uses one worker, 1,024 points, 16 shifts and the fixed supplied rule `[1,433]`.
All actual coordinate/weight/range hashes match within a seed pair and differ
between pairs. The complete complex Laurent vectors agree with the reference.

| Prescription | Median finite-coefficient variance of mean | Median production seconds, eager | Median production seconds, SymJIT O2 |
| --- | ---: | ---: | ---: |
| Fixed 0.05 | `8.01596e-9` | 0.1483 | 0.1380 |
| Fixed 0.25 | `1.46173e-16` | 0.1492 | 0.1403 |
| Dynamic polynomial | `1.04364e-16` | 0.2174 | 0.1991 |
| Dynamic sign-aware | `1.04364e-16` | 0.2167 | 0.2003 |

The paired variance-gain median against fixed 0.25 is **1.375**, with range
**1.137–1.656**. The SymJIT elapsed-time-normalized medians are **0.969** for
polynomial and **0.957** for sign-aware: the modest variance reduction does not
pay for the additional cost in this control. These are debug Rust driver
timings with coordinate auditing included, not release or eight-core speed
claims. The two constructions coincide because this polynomial has no harmful
higher-order terms; the result does not establish a sign-aware advantage.
The much larger gain against fixed 0.05 mainly demonstrates sensitivity to the
fixed strength, which is why both fixed baselines are retained.

Native evaluation counters, including precision rescues and excluding caller
bookkeeping, give median costs of 1.01 and 1.12 microseconds per accepted point
for fixed 0.05/0.25, and 4.39/4.43 microseconds for polynomial/sign-aware.
There is one stochastic sector, so its mean is also the maximum sector mean;
these are not maximum individual-sample latencies. Median double-float replay
fractions are 3.3%, 4.9% and 14.4% respectively. The retained per-domain counters
make this extra cost visible without weakening the shared precision policy.

The same full-budget one-dimensional bubble reaches variances near `1e-33` for
all prescriptions. Its complete coefficient agrees with the analytic reference,
but those variance ratios reflect floating-point noise and are not interpreted
as a performance advantage. The reporter retains the raw native covariance;
it does not replace small uncertainties with invented values.

A separate two-seed, 64-point/eight-shift SymJIT control measures optional
validation. Fixed production takes about 0.029 seconds under `always` and
0.003 seconds under `pilot`/`off`; dynamic production takes about 0.106 seconds
under `always` and 0.005 seconds under `pilot`/`off`. These short durations
establish removal of the expensive checker path, not stable throughput ratios.
Production counters are zero for `pilot`/`off`. Actual coordinates, weights,
ranges, mathematical identities and all reported means match across policies.
`pilot` and `off` estimates match bitwise. Dynamic `always` covariance differs
by at most `2.13e-20` after its additional numerical checks/rescues; it is not
claimed bitwise identical. Accepted replicas and full covariance remain native.

Raw reports remain ignored under `target/contour-variance-{square,bubble}-four*`
and `target/contour-variance-square-policy-*`. The next useful measurements are
physical contours with higher-degree envelopes, with equal pilot-tuning budgets
and explicit runtime-conditioning diagnostics.

The first implementation should be a Rust example, with a small typed input and
output layer around existing native sessions. It does not need another
integrator, estimator, RNG, worker pool or public benchmark framework. The
following are proposed example-local data structures, not existing library APIs:

```rust,ignore
struct ComparisonCase {
    source_identity: String,
    fixed: SelectedRecipeInput,
    dynamic: SelectedRecipeInput,
    chart_groups: Vec<ProvenChartGroup>,
    target: AccuracyTarget,
}

struct FrozenComparisonDesign {
    qmc: QmcSettings,
    pair_seeds: Vec<u64>,
    evaluation_batch_rows: usize,
    workers: usize,
}

struct PrescriptionRun {
    bound_math_identity: String,
    design: QmcDesign,
    estimate: VectorEstimate,
    contributions: ContributionReport,
    operational: OperationalMetrics,
    wall_seconds: f64,
    preparation_seconds: f64,
    coordinate_audit: Vec<CompletedCoordinateRange>,
}

struct ComparisonPair {
    pair_seed: u64,
    fixed: PrescriptionRun,
    dynamic: PrescriptionRun,
}
```

`SelectedRecipeInput` describes the native archive selector, physical bindings,
mathematical contour settings, validation policy and precision settings. Keep
the two bound mathematical identities distinct, including when both estimate
the same physical integral. `ProvenChartGroup` relates retained native source
charts and multiplicities across selected recipes. It must not identify sectors
by the accidental position of their entries in a catalogue. A recipe can prove
a different symmetry grouping: compare a proven group where possible and mark
the one-to-one marginal comparison unavailable otherwise. Do not force equal
IDs or claim point-matched global work when the chart relationship is unknown.

Start with `QmcSession::democratic(problem, settings)` for the controlled
equal-work comparison. Construct each problem through the existing
`KernelResultManifest::integration_problem` and explicit result scope, preserving
its complete coefficient layout and exact offset. Current `QmcDesign` reports
allocations; it is not a constructor for replaying an arbitrary adaptive
production design. Supporting such a design later needs a separate native API
review. A restored session for one mathematical identity must never be reused
for the other prescription.

The execution adapter only needs the existing native operations:

1. Bind and preflight one selected recipe. Record preflight and load/JIT time
   separately from production. Do not retain both heavy recipe programs merely
   to compare them: run the pair consecutively, and keep only compact reports
   between runs. A caller may use the native selective loader for sector jobs.
2. Call `session.next_work()` and `session.worker_context(task.sector_id())`.
   The caller dispatches jobs with its existing bounded process/thread policy.
   A worker calls `QmcWorker::evaluate_weighted_batch`, passing its actual
   transformed coordinates and weights to the selected sector's
   `WeightedEvaluationContext::evaluate_weighted_batch`.
3. Submit only the resulting `QmcReturn` to the session that issued its task.
   The task retains native content identity, epoch and interval checks. A fixed
   task is never submitted to the dynamic session, even when its point sequence
   is intentionally identical. A failed or incomplete batch is not a complete
   replica and cannot enter the final comparison.
4. Finish the frozen allocation, then obtain `session.estimate()`,
   `session.contributions()`, `session.design()` and the existing operational
   counters. Keep complete covariance; do not reconstruct it from absolute
   `complete_shift_estimates()` diagnostics, which can lose small fluctuations
   beside large exact offsets.

Actual coordinate equality is checked at the public worker callback boundary,
after Korobov transformation and before contour evaluation. Each record contains
the paired source-chart/group identity, native sector identity, dimension,
lattice/rule settings, shift index, lattice-index interval and a digest of every
coordinate's `f64::to_bits()` followed by the weight's `f64::to_bits()`. Native
`QmcTask::work().start()` and the lattice size identify the shift-major indices.
Packages crossing a shift boundary split their audit records at that boundary.
Hash row order, shape and interval identity as well as the numeric bits. This
requires a bounded hash state per outstanding interval, not stored lattices.
Compare completed records by canonical interval identity after reordered worker
returns. Native plan equality and settings are useful extra diagnostics but do
not replace this actual-coordinate check.

Hashing every coordinate is an audit cost. Report it separately and keep the
same instrumentation in both paired runs; evaluator timings exclude hashing.
For the final integration timing claim, repeat the frozen design with auditing
disabled only after its audited run passed, retaining the identical design,
binding identities and deterministic native point generation. Report this as a
timing repeat, not an additional independent statistical replicate. Neither
audit nor contour evaluation consumes the production RNG. Distinct predeclared
pair seeds still give independent randomized experiments, while the two
prescriptions within one pair intentionally share their production coordinates.

The report computes only presentation diagnostics from the native estimates.
For target complex coefficient `k`, let `V` be the sum of its real and imaginary
diagonal entries in the complete covariance of the mean. Preserve the full
matrix in each report, and show `V_fixed / V_dynamic`, actual evaluations times
`V`, summed worker seconds times `V`, native evaluator seconds times `V`, and
production wall seconds times `V`. Label each timing basis explicitly. A zero
or unavailable variance makes the ratio unavailable; it does not imply an
infinite speedup. Apply the same calculation to native sector/group marginals,
and show their actual IDs and multiplicities. For democratic sampling, use the
native complete global estimate for the total: summing sector marginal
variances would omit shared-shift covariance.

Per-sector evaluator seconds divided by actual evaluations gives the mean
evaluation cost. The maximum of those means is the slowest sector mean, not the
maximum latency of an individual point. Keep that distinction in the report.
The initial harness deliberately avoids computing a paired standard error for
the difference of two highly cancelling integrals: native per-prescription
covariance and repeated independent pairs suffice for this variance study.
If paired-difference uncertainty is later required, first audit a native
centered joint-vector accumulator, rather than subtracting large diagnostic
shift totals or writing another covariance implementation.

### Implemented API smoke control

`crates/fastsecdec/examples/contour_variance.rs` now exercises this boundary
using two fixed strengths on the analytic above-threshold logarithmic bubble.
It is intentionally an API control, not a dynamic-mode comparison or a physical
performance benchmark. Run it with `cargo run -p fastsecdec --example
contour_variance` using the currently validated owner dependency configuration.
The output is a self-contained JSON report; raw runs remain under ignored
`target/` paths.

The native eager evaluator runs one worker, two independent seed pairs, and
64 points in each of eight complete shifts. The existing supplied-vector API
uses the one-dimensional generating vector `[1]`; published multidimensional
catalogues are not bypassed below their supported minimum. Package size 19
deliberately crosses shift boundaries. Both strengths reload the same saved
optimized program and bind independent mathematical identities, without
regeneration. Deterministic causal preflight stays separate from production.

The executed control passed: all 34 coordinate/weight interval digests matched
within each pair; independent pairs differed; a return from the other strength
was rejected; reversing pairs of completed work returns preserved native
admission; both complete Laurent vectors matched their analytic reference;
and the finite complex coefficient retained nonzero covariance for both
strengths. A focused digest regression changes coordinates, weights and range
identity separately. The report retains each native full covariance and sector
contribution report, rather than reconstructing statistics in the example.
At a larger lattice the control reaches floating precision, so its zero
sample covariance must not be used as evidence for an infinite improvement.

### Public dynamic comparison controls

The same example accepts `--dynamic`, `--symjit` and
`--validation=always|pilot|off`. The full dynamic comparison freezes eight
listed seed pairs, 1,024 points and sixteen shifts. `--smoke` deliberately
reduces this to the two-pair 64-point/eight-shift API check, and labels its
output as insufficient for a variance improvement claim. A recipe is generated
once per invocation; strength changes reuse its saved optimized program.

The default bubble can approach arithmetic resolution very quickly. Before
observing any dynamic comparison, a second control is specified with
`--linear-square`:

```text
(1/eps) integral_[0,1]^2 (1 - 2*x - 3*y - i0)^(-eps) dx dy
pole:   1
finite: 3/2 - (7/3)*log(2) + (11/12)*i*pi
```

Symbolica independently verifies the mixed primitive
`h(F)/6`, where `h(t)=t^2*log(t)/2-3*t^2/4`. Its four causal endpoint values
give the stated reference; the imaginary coefficient also agrees with the
negative-F area, `11/12`. The comparison retains both original fixed controls
(`lambda=0.05` and `lambda=0.25`), dynamic `S=0.8`, `L=R=1`, and supplies the
fixed native rank-one
generating vector `[1,433]`. Its components are coprime to both declared lattice
sizes. This is a reproducible untuned rule, not a claimed optimized lattice.
The nine preflight points form the Cartesian three-point boundary/interior
grid; they remain separate from production. Both controls use the complete
native Laurent-vector covariance and matching actual-coordinate digests.

These are analytic runtime controls, not physical multiloop benchmarks. Very
small nonzero empirical variances near arithmetic resolution must not be
interpreted as meaningful large convergence gains. Dynamic measurements remain
pending the full public scientific gate. An initial two-pair bubble smoke run
has passed the native means, matched-coordinate and zero unchecked-production
validation-counter checks for both dynamic constructions. Its large raw
variance ratios relative to `lambda=0.05` are not a general gain claim: the
earlier `lambda=0.25` control already achieved still smaller variance. The
four-prescription comparison therefore reports both fixed baselines explicitly,
rather than choosing a weak static reference.
