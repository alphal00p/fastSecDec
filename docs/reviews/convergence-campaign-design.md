# Convergence and first matched performance campaign

This is an execution proposal, not a completed benchmark or a change of default
lattice. It follows the [48 physical catalogue runs](qmc-catalogue-cross-case.md),
the [six independent massive references](massive-multiloop-diagnostics.md), and
the [reference evaluator audit](reference-evaluator-smoke.md). The current native
backend is Symbolica 3.0.1 / SymJIT 2.26.4; every new observation must carry its
actual source/dependency identities. Previous timings from another backend or
development profile are not relabelled as current release measurements.

## Questions and fixed decisions

The scientific campaign asks how uncertainty changes with actual point count,
whether independent seeds remain compatible with the retained external values,
and whether the historical Kuo rule's known plateau persists on other cases.
It does not assume that an individual estimated error decreases monotonically.
The performance campaign compares complete scientific work, then attributes a
measured difference to generation, loading, evaluation, precision rescue or
bookkeeping. Neither campaign changes precision safeguards to obtain a speedup.

Use only existing production capabilities: native `KernelSet`,
`WeightedEvaluationContext`, `QmcSession`, its observation/contribution/design
accessors, Numerica's `QmcEstimate`, and the native reference adapter. A small
ignored Rust campaign driver may orchestrate these APIs and serialize their
results; it must not implement another estimator or integrator. The production
CLI remains the end-to-end performance entry point. All inputs use full-integral
scope and every retained positive-dimensional sector's full Laurent vector.
Whole analytically constant sectors retain their existing exact contribution.

No adaptive pilot, support splitting, sector selection or rule search belongs to
this first experiment. These would change the estimator or workload. The current
default remains Kuo33002. HKKN alpha3 is an explicit candidate selected from
earlier evidence, not from the smallest error in a new run. Its ten-dimensional
limit is checked before starting; unsupported dimensions are errors.

## A. Holdout work scaling on the six massive cases

Freeze the six existing input cards and independently generated reference
transports. Reuse their unchanged physical artifacts only after validating the
current loader and complete kernel/source identity. Loading an old artifact with
a new compiler is a new recorded execution, not its old generation timing.

| Setting | Prespecified value |
| --- | --- |
| Cases | kite, self-energy, both two-loop three-point cases, both three-loop three-point cases |
| Rules | Kuo33002 and HKKN alpha3, both through native published catalogue APIs |
| Seeds | 20261101, 20261102, 20261103; distinct from the exploratory campaign |
| Stage A points per shift | 1024 and 8192 |
| Independent shifts per run | 16 |
| Periodization | Korobov3 |
| Work packages / workers | 1024 points / one caller-owned worker |
| Allocation | Democratic, one complete allocation, no accuracy-based early selection |
| Precision | Unchanged artifact policy and default weighted replay; all diagnostics retained |
| Numerical watchdog | 180 seconds per run, 45 minutes for the stage; partial observations retained |

This gives **72 runs = six cases × two rules × two counts × three seeds**.
At the existing 253 total kernels it is 223,838,208 accepted physical-kernel
evaluations if every allocation completes. Recompute this count from the loaded
manifest before execution; a changed kernel count is recorded and reviewed, not
silently forced to match the old partition. Count precision evaluations, point
generation, failed attempts and analytic contributions separately. The number of
Laurent outputs is not an additional number of integration samples.

Use a deterministic order fixed before launch: rotate the two rule orders by
seed and alternate small/large count order by case. Save each completed or failed
row immediately. A failure is not replaced with a successful rerun under the
same row identity; a diagnostic rerun receives a new identity and reason.
Continue other independent rows after a bounded failure unless a shared
correctness issue invalidates them. Preserve the watchdog outcome and accepted
coverage rather than inventing a final estimate.

The already chosen second stage is the same full matrix at 32768 points and 16
shifts: 36 runs and 795,869,184 evaluations at the old kernel counts. Its execution
requires a separate resource handoff after Stage A, but its rows are already
fixed and are not selected by favourable errors. If the resource budget prevents
this stage, report that the convergence-rate evidence remains limited to two
work levels. A separate six-line control at 8192 points and 64 shifts, both rules
and all three seeds, costs 18,874,368 evaluations and distinguishes increasing
independent shifts from increasing the lattice size. Do not combine that control
with the point-scaling curve as though its work allocation were identical.

For every run preserve:

- Outer artifact identity, inner kernel identity, full manifest, exact offset,
  coefficient/component layout, original reference and comparison evidence.
- Native design including catalogue provenance, actual modulus, reduced integer
  generating vector, shifts, seed, transform and actual accepted counts.
- Authoritative native total mean and full covariance, complete/incomplete
  coverage, all sector marginal estimates and statistical failures.
- Complete same-shift full-integral vectors, keyed by seed and shift, and every
  precision check/rescue/failure count. Marginal sector variances never replace
  the correlated total variance.
- Loading/context preparation, numerical loop and serialization times; peak
  process memory and watchdog outcome. These diagnostic runs may share a host
  only when explicitly labelled unsuitable for timing acceptance.

Each seed's native estimate remains a primary observation. For each fixed work
level, concatenate the two rules' same-seed/same-shift complete vectors and pass
the 48 joint rows to `QmcEstimate::from_shift_means`. This uses the existing native
joint covariance and retains correlation between rule choices. A corresponding
joint table across the two work levels may use the same 48 aligned rows; those
levels are correlated and must not be compared by an independence assumption.
Do not pool rows from different lattice sizes into a single claimed production
estimate. Absolute diagnostic shift vectors can lose small differences beside
large offsets: retain the session's centered total as authoritative, and limit
this joint diagnostic to the finite massive cases unless that representational
issue has been checked explicitly.

Native `reference::compare` aligns coefficients and retains source uncertainty.
The six frozen targets have nonzero uncertainty and bounded validation evidence;
they are not exact answers. Report all seed-level and joint comparisons, observed
uncertainties and error ratios. A discrepancy above five combined standard errors
is a prespecified investigation trigger, not automatic proof that either program
is wrong. Reference uncertainty can dominate an accurate native estimate; do not
claim a measured convergence slope from that external residual floor. Three seeds
are a useful initial check, not a coverage calibration or a normality proof.
Preserve the earlier 48 exploratory observations separately from this holdout.

## B. First paired triangle/box benchmark

Start with the native `triangle.toml` and `box.toml`, and frozen Pathfinder's
`dot_triangle.yaml` and `dot_box.yaml`. The kinematics, unit measure multiplier,
complete orders through zero and final Gamma convention have existing analytic
and native-master checks. Run a fresh 1024-point/eight-shift smoke comparison
before timing. Verify every physical real coefficient, the reference's imaginary
residual and error, complete support/coverage, and zero numerical failures. A
missing imaginary row is not silently fabricated in the native comparison.

The known-correct frozen reference uses **complex SymJIT O2**, not either of its
incorrect real O2 translations. It uses Symbolica 2.1.0 / SymJIT 2.18.6, while the
native real kernels use Symbolica 3.0.1 / SymJIT 2.26.4. This is an explicitly
versioned program comparison; it is not a claim that identical machine kernels
were timed. The reference ordinary-kernel benchmark omits global-prefactor
convolution, so it cannot serve as the complete-vector baseline.

After the smoke gate, use 8192 actual points, 16 shifts, one worker, package/batch
size 1024, Korobov3, full sector support, democratic shared-shift aggregation and
one round. There are seven paired fresh-process repetitions per fixture, with
seeds 20261201 through 20261207 and alternating program order. Preserve all
repetitions. Equal integer seeds across Havana and QMCPy do **not** produce equal
shift arrays; timing pairing controls host drift and does not supply correlated
cross-program statistical samples. Use native joint estimates only where the
actual rows are shared. Identical-point kernel timing is a separate experiment
requiring an independently verified expression/map correspondence.

Commands below are source-checked templates, not evidence of execution. Before
running, create the ignored output directory, record the release executable
identity and substitute `CASE`, `SEED` and a unique `ROW`. Do not use `--resume`.

```sh
# cwd /common/dev/fastsecdec
target/release/fastsecdec --json --plain run examples/runs/CASE.toml \
  --output output/benchmarks/first-paired/ROW.native.fsd.json \
  --method qmc --lattice kuo33002 --points 8192 --shifts 16 \
  --seed SEED --workers 1 --absolute-tolerance 0 --relative-tolerance 0 \
  --checkpoint output/benchmarks/first-paired/ROW.native.checkpoint.json \
  --save-result output/benchmarks/first-paired/ROW.native.result.json \
  > output/benchmarks/first-paired/ROW.native.stdout.json \
  2> output/benchmarks/first-paired/ROW.native.stderr.log

# cwd /common/dev/fastsecdec/DO_NOT_PUSH_FOR_REFERENCE_ONLY/FastSecDecPathFinder
SYMBOLICA_HIDE_BANNER=1 .venv/bin/python FSD.py run \
  --run examples/runs/dot_CASE.yaml \
  --target examples/outputs/dot_CASE_pysecdec_target.json \
  --explicit --jit-compile --jit-optimization-level 2 --complex-evaluator \
  --sampling-mode qmc --qmc-lattice-backend qmcpy --qmc-order linear \
  --qmc-support-mode full --qmc-refine-sectors democratic --qmc-correlate-sectors \
  --qmc-korobov-alpha 3 --no-qmc-optimized-evaluators \
  --samples-per-iter 8192 --qmc-max-samples-per-iter 0 --qmc-shifts 16 \
  --seed SEED --max-iter 1 --min-iter 1 --workers 1 --batch-size 1024 \
  --quiet-summary --no-progress --json --restart \
  --result-path /common/dev/fastsecdec/output/benchmarks/first-paired/ROW.reference.json
```

The current native cards have no integration overrides: defaults give one round
and 1024-point packages. Assert these effective settings in the report rather
than relying on that fact forever. The reference cap is explicitly disabled;
otherwise its default cap would turn 8192 into 4096 QMCPy points. Both programs
must report their actual counts, retained dimensions, output counts and analytic
offset treatment. Different valid sector partitions are recorded; neither sector
IDs nor a nominal point request establishes equal total computational work.

Wrap each command in one existing process timer/watchdog to capture total wall
time, CPU time, peak RSS and exit status. Initial per-process watchdog is 180
seconds. Run builds and all Symbolica programs serially, with no competing
benchmark CPU load. Record host/OS/CPU/toolchain and affinity; reuse the same
allowed CPU instead of hardcoding a possibly unavailable CPU zero. Archive the
expanded command, environment settings that affect numerics, and all raw output.

This first pair measures fresh-process end-to-end execution with the reference's
distributed formula caches available. It is **not cold generation**: setting
`FSD_SUBTRACTION_FORMULA_CACHE_DIR` to an empty directory still leaves fallback
reads from the checkout's default and legacy cache roots. `--restart` resets
integration state, not those caches. Do not delete shared caches. A true cold
generation follow-up must use an isolated frozen checkout with the read roots
and initial cache inventory controlled, and explicitly allow missing-formula
generation. Label supplied universal-formula assets separately from generated
artifact reuse.

The next paired mode uses each program's existing generate/integrate transport:
prepare and numerically verify each artifact once, then measure fresh-process
loading and full integration separately. First verify that the reference bundle
retains the same global prefactor, full-support settings and complex O2 backend;
do not assume a `run` command's artifact is interchangeable. Native timing starts
after loading, includes worker context preparation and checkpoint/report work at
its documented boundaries, and differs from the reference's internal timers.
External end-to-end wall time is the common measurement; internal stages are
attribution evidence until their boundaries are aligned. The reference report
exposes per-coefficient correlated errors but not necessarily a full cross-order
covariance; preserve that limitation rather than manufacturing off-diagonal zeros.

Persistence also differs in the initial commands: native `run` publishes a
portable artifact, checkpoints accepted work and saves a typed result, while the
reference invocation writes its result and uses formula caches. Report those
costs explicitly; do not call the pair persistence-aligned. Before acceptance,
add a mode using each program's existing prepared-artifact transport with a
documented equivalent durability policy, or compare existing caller-driven
numerical loops while timing serialization/checkpoint work separately. Keep
full accepted state and evidence outside that loop timer. This matters for large
sector sets even when it is small for triangle/box. No checkpoint/precision
safeguard may be silently removed from a claimed production end-to-end result.

## Interpretation and further acceptance

Report all seven times, both medians and paired ratios per fixture and mode.
The existing acceptance target is no more than a 5% native timing regression per
representative fixture. A large repetition spread makes a near-threshold result
inconclusive; retain the first block and schedule another complete block, rather
than dropping slow runs. Keep the measured fixed-work comparison distinct from
time to verified accuracy. For the latter, predeclare a count ladder and a target
for every coefficient, include the costs of all attempted levels, and require
independent reference/seed evidence before calling a level accurate.

Precision policies remain a material comparability condition. Native checks use
conditioning plus whole-vector two-precision weighted replay; the frozen program
uses its endpoint-dependent decimal-precision tiers. Record their actual checks,
rescues, precision and failure counts. The first timing is a scientifically
validated baseline candidate. Do not claim strict matched-precision acceptance
until the differing policies have been audited against the same accuracy target
or an existing compatible explicit precision configuration is established.
Disabling rescue or using the broken reference real backend is not a remedy.

This proposal does not close the difficult numerator, double-box, triple-box,
orthant or hard four-loop performance gates. Their bounded scientific sequence is
in [remaining scientific campaigns](remaining-scientific-campaigns.md). Only
complete, current-backend results with retained uncertainty and coverage enter
later scaling campaigns. Higher double-box coefficients and historical hard-case
manual targets remain unverified where their evidence is incomplete. Linux
execution does not establish macOS performance or testing.

Execution handoff: the coordinator approves each bounded stage and allocates the
shared runtime window; a numerical owner records native statistics, a reference
owner verifies the frozen oracle command/input, and an independent reviewer
checks row completeness, identities, covariance use and timing boundaries before
any parity or default-rule decision. No campaign was executed by this document.

The independent reference owner reviewed the command and matching conditions
against the executed complex-O2 smoke. No scientific-design blocker was found;
the persistence-cost qualification above was added from that review. Runtime
preflight still has to verify the effective prepared-artifact settings.
