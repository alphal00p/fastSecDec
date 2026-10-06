# Phase-one remaining gates — current bounded completion work

Source review, updated 2026-10-06. This reconciles the earlier remaining-gates audit with
validated milestones through `a3d09e1` and the retained full-graph attempts.
[FIRST_PHASE_PLAN.md](../../FIRST_PHASE_PLAN.md) remains authoritative; this
document adds no requirement. The latest workspace test gate is
**419 passed, zero failed, 25 explicitly ignored heavy probes**. The rebuilt
native HEPKit wheel passes **81 controls**, and the portable backend on a native
host passes 41 focused controls. Formatting and strict all-target Clippy pass;
the final equivalent Option-guard cleanup also passes all five discrete-MC
controls. The later conservative cold-load literal-zero fix passes 24 focused
artifact/replay controls, formatting and scoped strict Clippy. It reuses
Symbolica's native instructions and preserves artifact identity and accuracy
policy. Its same-allocation scalar-double-box repeat reaches the unchanged
0.714-per-mille target in 152.996 seconds, versus 172.715 before the fix; rescues
fall from 46.52% to 31.53%. All five reference checks and 103 covariance/vector
comparisons pass within retained roundoff bounds. This remains a single
observation above the 91.584-second Pathfinder result. One subsequent trial of
the existing IBP strategy passes the same five references and reaches 0.978 per
mille, but takes 164.945 seconds with larger evaluator programs. Taylor remains
the default; this trial does not close the performance gap. The earlier
380-test milestone's evidence
remains in the
[current status and evidence](implementation-status-2026-10-05.md) and
[independent public integration audit](native-named-request-interface-independent.md).

The later existing adaptive-QMC lane supplies one faster scalar-double-box
target-reaching observation: **82.859 seconds including a fresh pilot and
loading, at 0.989 per mille**. Complete vectors, covariance and all five references
pass. The earlier 320-worker-second allocation misses at 1.035 per mille in
73.298 seconds; together the two integration attempts cost 156.157 seconds.
The successful allocation uses a 400-worker-second production budget. It is
below the retained Pathfinder 91.584-second observation, but repeated matched
timings and per-sample/sector latency acceptance remain open. No accuracy crossing
is interpolated and no previous result is replaced.

**Latest user scope decision:** Successful generation of the original on-shell
triple box is sufficient for phase A. Its full artifact and cold-load checks
pass; further full-integral numerical agreement, convergence and performance
work on that case are deferred. Preserve partial attempts without treating them
as complete numerical results. The previous on-shell gates below remain a
historical record, superseded by this decision and the corresponding amendment
in `FIRST_PHASE_PLAN.md`. Remaining work concentrates on the faster existing
representatives. The user supplied an authorized license, and the subsequent
eight-worker Pathfinder capability check passes. Current paired triangle
generation/integration and individual-latency comparisons are complete. The box
also completes seven pairs and its initial latency diagnostics, with faster
generation, integration and mean calls but a higher observed native maximum. A subsequent
three-point diagnostic confirms genuine native first-use precision preparation,
and finds that the old Pathfinder recorder timed rows only after warming them
with an ordinary batch. The original observations remain unchanged. The corrected
full-stream comparison times first use in both providers and preserves every
scientific equivalence check: native mean/max are 9.878479/5,863.705 microseconds,
versus Pathfinder 70.390258/43,051.430 microseconds. This closes the earlier
asymmetric observed-latency comparison; the large Pathfinder maximum remains
unattributed, and neither finite observed maximum is a worst-case guarantee.
The [double-box lattice follow-up](double-box-lattice-followup.md)
now observes a native finite-part relative standard error of 0.393 per mille:
437.039 seconds at its final eight-worker level, or 664.554 seconds across both
tested HKKN levels. Subsequent current-main validation reaches the same error in
479.451 seconds using larger packages, with exactly equal results and covariance.
The existing Pathfinder prime catalogue reaches 0.577 per mille in 275.506 seconds.
The pre-literal-zero-fix native Korobov2 allocation reaches **0.714 per mille in
172.715 seconds** on eight workers, with all five reference checks passing and
zero evaluation failures. It reuses HKKN N8192/R16 and retains the full vector
and covariance. The same-Korobov2 Pathfinder check reaches only 5.062 per mille
at prime 8,311 in 42.756 seconds, but its single larger prime-17,807 allocation
reaches **0.362 per mille in 91.584 seconds** with all five reference checks
passing. The same pre-fix revision's reduced all-sector sample timings retain a native mean gap:
106.314 versus 74.846 microseconds (observed maxima 44.481 versus 52.645 ms).
Those API boundaries and different lattices remain explicit. The later adaptive
observation above improves target time; scalar-double-box mean-latency and
repeated-timing acceptance remain open. No further
reference ladder is planned. Other representative comparisons remain open. The retained earlier
license failures are unchanged. The HEPKit marimo extension's generated gg→HH
helicity double box passes its [native CLI prerequisite](gghh-native-feasibility.md).
The subsequent complete 15,728,640-point HKKN-alpha3 QMC allocation reaches
**0.00937%** finite-part relative standard error in 377.530 seconds on eight
workers, closing the newly requested **0.1%** target. Its full vector agrees
with both earlier independent-seed allocations within one combined standard
error. No further QMC tuning is required for this example. The optional browser example is
implemented and published in FastSecDec `0cf08c6`, with the scalar triangle
selected first. The new browser gg→HH trial completes generation in 546.667
seconds, then times out downloading its first report; browser integration and
detailed inspection remain unverified. Native triangle
and gg→HH Generate/inspect/Integrate/Cancel/Resume workflows pass. The earlier
compiled Wasm wheel passes generic smoke and **50 portable controls**; its
actual browser triangle lifecycle also passes, including generation
streaming, inspection before sampling, cancellation and exact checkpoint-prefix
resumption. Selecting the optional gg→HH entry remains idle. Community delivery is
published in [draft PR #18](https://github.com/symbolica-dev/symbolica-community/pull/18).
Its heavy binding, tests and examples are owned by FastSecDec; community only
links/registers/reexports them and supplies stubs. The documented dependency
patches remain necessary. See the [relocation audit](hepkit-relocation-audit.md),
[bridge review](hepkit-fastsecdec-bridge.md),
[UI review](fastsecdec-showcase-ui.md) and
[dependency-delivery review](dependency-delivery.md).

The new ordinary-Havana lane now implements discrete importance sampling over
sectors with Numerica's existing `DiscreteGrid` and continuous child grids.
Its 29 native MC/QMC controls and 74 CLI controls pass; the thin HEPKit binding
and actual notebook lifecycle pass too. The complete gg→HH MC allocation reaches
0.577 per mil finite-part relative standard error and agrees with QMC within
1.70 combined component standard errors, with full covariance retained. The
completed checkpoint restores with one worker without adding samples. See the
[native sampling review](havana-discrete-sector-sampling.md).

The latest sector-exploration extension retains native prefactors and affine
epsilon powers before endpoint subtraction, together with the variable maps,
Jacobian and actual shared evaluator statistics. Seventeen native metadata and
artifact controls pass. The selected-chart notebook panels pass actual native
UI validation, including rendered math. Fresh gg→HH generation/reload retains
all 30 charts and 54 mapped terms with unchanged exact evaluator programs.
The new public Wasm wheel passes generic smoke and all 58 portable controls.
The actual browser triangle completes metadata inspection, QMC and ordinary
Havana with exact accepted-prefix checkpoint restoration; its full means and
covariance equal the native notebook results. A final supplemental screenshot
fails after those scientific/lifecycle assertions, so the raw harness status
remains failed and an independent additive review qualifies the retained data.
The corrected hosted native CI passes all 81 controls. The single bounded
browser gg→HH trial reaches all 30 sectors and generation-ready in 546.667
seconds. Its first report download then times out; no integration starts.
Long native calls produce refresh-RPC timeouts despite visible streamed progress.
The complete process tree is reaped after 596.001 seconds, with peak sampled
RSS 3.86 GB and unchanged inputs/export. This establishes browser generation,
not smooth long-call responsiveness or the gg→HH inspection/integration lifecycle.
See the
[metadata review](retained-generation-explainability.md).
The requested basic denominator geometry check
is closed: native pySecDec decomposition agrees with all 30 sector maps,
Jacobians and factor valuations. See the [sector check](gghh-sector-sanity.md).
No infeasible full reference generation is being repeated.

The [regression matrix](../REGRESSION_MATRIX.md) retains **182 rows: 101 Covered,
81 Retired, zero Partial and zero Pending**. The double-box row
`test_integrals.py:6113` is closed by its actual endpoint-formula identities,
complete-vector allocation and checked five-order reference. That historical
test does not require universal uncertainty calibration. Retired backend,
Python, sector-numbering and serialization interfaces do
not retire their scientific replacements. The plan's original 15 DOT examples,
11 kinematics fixtures, 17 run cards, stored targets and hard-polynomial report
remain the scientific inventory; the current 24-card/17-DOT loading gate is
additional input coverage, not proof of numerical completion.

## Closed implementation gates

| Item | Accepted evidence and remaining boundary |
| --- | --- |
| Caller-owned geometry cache and parallel chart/cone work | [Complete decomposition cache](geometry-cache-independent.md), [context/dispatch adoption](parallel-generation-dispatch-independent.md) and [CLI adapter](cli-geometry-dispatch-independent.md) pass their scientific/interface gates. Native plans, deterministic admission and cancellation remain library-owned; executors remain caller-owned. Symbolic representative work is not made parallel by this adapter. Cache speed and memory remain performance questions. |
| Public native named coefficients | [Public opt-in integration](native-named-public-integration.md) passes 41 distinct focused tests and the subsequent 370-test combined gate. Native Series, request/face resolution, native aliases, typed limits/progress, conservative conditioning, context dispatch, hidden complex bodies, weighted replay, clones and separate-process artifacts are exercised. `Physical` remains the default. Actual public representative and full original graph gates remain below. |
| Captured representative coefficient agreement | The [three original-expression oracles and cold reader gate](native-dual-reader-admission.md) pass 21 order/point comparisons and 24 weighted vectors, covering 168 real components. All 5,417 frozen hashes pass; rescue uses 256/384 bits. This closes the earlier disconnected candidate's oracle gap, not public input reconstruction or integral-level agreement. Earlier 180/600-second exact-Series oracle failures remain retained. |
| Content identity and CLI presentation | [Sector content identities](sector-content-identity.md), structured errors/status, and [actual Linux PTY controls](terminal-policy-results.md) cover cancellation, resize, cleanup and color policy. Identities bind native IR/layout/policy, not general CAS equivalence. Other platform/terminal evidence remains separate. |
| Numerica QMC Rust 1.89 compatibility | [Exact MSRV checks](numerica-qmc-upstream-readiness.md) pass 237 default, 240 serde and 217 alternative-backend tests on Linux with unchanged sources. The caller-owned QMC branch is published as [upstream PR 8](https://github.com/symbolica-dev/numerica/pull/8). Recorded review/CLA disposition is an upstream follow-up, not an unimplemented QMC or MSRV capability. Other platforms remain unqualified. |

## Previous on-shell gates and retained evidence

1. **Public representative prerequisite is now accepted.** The real `UnitCube`
   input passes exact native regular/density identity, ordinary `NativeNamed`
   generation, compilation and artifact persistence. The actual reversed
   coordinate map, multiplicity one and literal-zero exact offsets are checked.
   All 21 signed coefficient/point comparisons and 24 cold weighted vectors
   pass, including twelve forced 1,024-bit replays. See the
   [independent public reconstruction audit](native-named-public-actual-independent.md).
   This closes the prerequisite; it does not establish full-integral agreement.

2. **Complete the original on-shell triple-box graph.** Use the reviewed
   [native named full-graph protocol](native-named-fullgraph-protocol.md), with
   explicit native named coefficients and exact `SingleUnitTerm` family
   preparation. The ten-parameter `native-named-fullgraph-2` was intentionally
   cancelled at 372/1,026 representatives after 1,006.900 seconds; it did not
   time out. The subsequent equivalent eight-parameter prepared trial completed
   all 1,026 coefficient expansions and sector compilations by 1,668.040 seconds,
   then failed allocation before artifact publication under the 30-GiB
   address-space bound. The narrow persistence fix removes large serialization
   copies and passes the 380-test workspace gate, formatting and all-target
   Clippy. Its committed rerun publishes the full artifact in 1,695.486 seconds
   with peak 15.32 GiB, then cold inspection succeeds in 136.719 seconds.
   The subsequent 180-second integration stage times out with partial accepted
   coverage and zero evaluation failures. The following CLI lifetime fix bounds
   live sector contexts per worker and passes all 16 focused driver tests. Its
   eight-worker continuation restores the original checkpoint successfully and
   retains 397,312 accepted evaluations without failures. It is intentionally
   stopped after 1,305.505 seconds for a scheduling handoff, not by its deadline.
   The measured batch barrier motivates the private CLI refill queue, which
   passes all 20 focused driver tests. Reuse the newer checkpoint and unchanged
   artifact for one eight-worker, 7,200-second continuation; do not repeat generation.
   Preserve the original graph, kinematics, order zero,
   O2, full coefficient coverage, exact offsets and covariance. Original
   generation had a 1,800+5-second bound; cold inspection and the first fixed
   allocation each had 180+5 seconds. The next continuation's 7,200+5-second bound
   includes loading and retains the original complete allocation. Prior attempts and their distinct causes remain in
   the [results](native-named-fullgraph-results.md).

3. **Establish original-integral numerical agreement.** Successful generation,
   artifact inspection and the fixed allocation establish capability and
   transport only. The original on-shell integral still needs complete-vector
   independent numerical agreement and subsequent convergence/calibration.
   The full-integral saved scope/design, rather than command flags or a
   successful representative, determines coverage. Reuse any existing
   independently audited reference with matching input and normalization.
   The [current reference inventory](reference-onshell-full-vector-proposal.md)
   finds no Checked original on-shell full vector; the two off-shell fixtures
   do not substitute for it. Subsequent infeasible pySecDec/FORM attempts are
   closed under the user's resource instruction. The existing
   [Pathfinder direct route](reference-onshell-pathfinder-direct.md) is the
   remaining comparator, with completed native formula-cache entries retained
   for its bounded continuation.

## Remaining scientific and uncertainty gates

The table retains the earlier family-level limitations. The bounded completion
addendum below governs further work: reuse the 72 holdout rows and accepted
reference comparisons, and do not interpret these entries as a new universal
calibration campaign.

| Family | Accepted evidence | Still required |
| --- | --- | --- |
| Off-shell scalar and rank-two triple boxes | Original and exact native projected families have complete vectors and independent Checked references. [Scalar](projected-triple-reference-independent.md) and [rank-two](projected-rank-two-reference-independent.md) audits preserve normalization, numerator/Gamma factors and separate original/projected comparisons. | Prespecified repeated-seed/work/shift convergence and error calibration. Reference finite-part relative standard errors are about 2.5% and 1.67%, above one per mille. |
| Hard four-loop full orthant | All 699 native kernels complete with `[-2,-1,0]` covariance. The [reference audit](hard-four-loop-reference-phase-independent.md) accepts all four provider orders `[-3,-2,-1,0]`; a separate native generation certificate proves zero through −3 with full 2,760-chart coverage. | Calibrated complete-vector comparison and finite-part accuracy; reference relative SE is about 0.569%. The old native numerical result remains unchanged: its full-union report has `MissingEstimate` at −3 and is globally ineligible. The separate exact-zero certificate does not authorize padding its covariance. |
| Double box and Issue 1 | Complete independent references are present. Issue 1's ordinary `together=True` fixture corrects the earlier disteval cross-sector variance omission; the old fixture remains Unverified. | Difficult-case repeated-seed/error calibration and highest-requested-order accuracy. A Checked provider route does not establish universal calibration of either estimator. |
| Six massive families and numerator controls | Six independent references, 72 holdout rows, native one-loop masters/reduction and coupled-sunset analytic controls pass. | Prespecified higher-work/shift controls and final convergence/performance acceptance. Already accepted small physics proofs need not be repeated merely to fill a matrix cell. |

The [earlier campaign schedule](remaining-scientific-campaigns.md) preserves
chronology; its missing-reference entries are superseded by the newer audits
and fixtures. The hard reference uses the full orthant and U factor. Exact
graph/direct-UF double-box identity already closes input equivalence, so that
second input route does not require a duplicate physical campaign. Retain all
plan requirements for full-vector cancellations/covariance, boundary precision,
worker partitioning, interruption/resume and independent-seed coverage while
closing these remaining numerical gates.

## Performance, platforms and final acceptance

Complete required capability/scientific coverage, then run the focused
comparisons needed for acceptance. Reuse valid evidence and stop tuning when
representative parity is established, as the user clarified on 2026-10-05.
Do not add optional performance campaigns after the goal's requirements pass;
mark the goal complete and stop, leaving threshold planning to the user.
Run the plan's matched same-host O2 comparisons with equal worker counts,
precision, transforms, rules, shift counts, orders and statistical targets.
The completed native-only eight-physical-core triangle/box seven-seed campaign
and [individual-sample latency diagnostic](native-sample-latency-results.md)
remain evidence. The earlier paired eight-core Pathfinder route hit an explicit
concurrent-instance license limit. The supplied license now permits its existing
eight-worker route, and the current matched triangle measurements close that
case's target/process comparison. Continue the remaining representatives with
the same actual eight-core design; do not extrapolate earlier one-worker rows.

Required performance evidence remains:

- Separate cold generation/compilation, warm loading, complete-vector kernel
  and integration throughput, fixed work, time to verified accuracy, peak
  memory, point-generation/reduction cost and precision-rescue rates.
- Eight-physical-core time to one-per-mille estimated relative uncertainty for
  the **largest signed requested epsilon power**, usually the finite part.
  Record the first crossing, actual work, rule/transform/seed/affinity and
  independent full-vector checks. Use a labelled absolute check for a known
  zero; do not replace the target by a lower pole or extrapolate fewer workers.
- At least seven paired repetitions for ordinary cases and three for expensive
  cases, alternating order, with median timings within the agreed **5% band
  per representative case**. Cover small one-loop, double-box, numerator-heavy
  multiloop and hard four-loop cases using the reference's normal boundary-support,
  optimized QMC settings and complete correlated physical vectors, as specified
  in the [current protocol](minimal-paired-acceptance.md). Incorrect values or
  underestimated uncertainty fail acceptance.
- Separate per-sector average worker cost, maximum sector-average cost and
  individual-sample maximum latency. Retain slow boundary observations,
  complete vectors, dimensions, sample/batch sizes, precision/rescues and
  instrumentation overhead; report batch-amortized and individual costs
  separately. Published lattice/periodization comparisons need frozen
  multi-seed workloads, with speed distinguished from samples to accuracy.

Linux evidence is executed; macOS and other target configurations/tests must
be provided where hosts are available and otherwise labelled unverified.
Numerica's successful Linux MSRV gate does not qualify those platforms or
change FastSecDec's compiler baseline. Finish the independent native-reuse,
HEPKit public-API, dependency-separation, example-delivery and CLI/platform
review after the remaining scientific/performance work. Normal builds/tests
must continue to need no Python, pySecDec, FORM, Normaliz or reference-generator
runtime. The documented native Rust ecosystem checkouts remain the authorized
dependency setup; this is not a new packaging or dependency-publishing gate.

## Bounded completion addendum, after `a56107f`

The user's stopping rule narrows further work to unresolved acceptance claims.
The [delivery audit through `e42a017`](phase-one-delivery-audit.md) finds no
additional missing required subsystem or HEPKit integration blocker beyond
the bounded work below.
The family-level uncertainty entries above are limitations to assess against
the retained evidence, not instructions to launch a new calibration campaign
for every card. No finite test block can certify uncertainty for every possible
integrand. Preserve that qualification after completion rather than turning it
into an unlimited gate.

The user's subsequent reference-resource instruction ends further on-shell
pySecDec/FORM retries on this machine. Retain practical completed references
where available, but use FastSecDecPathFinder for the required performance
comparisons. FastSecDec still has to complete the difficult case. Missing
infeasible pySecDec results are not an additional completion requirement.

| Minimum remaining decision | Evidence to reuse and bounded next action |
| --- | --- |
| Original on-shell integral | Generation, publication of all 1,026 sectors/orders and separate-process inspection pass. The user accepts this capability for phase A and defers further numerical/reference/performance work. Retain actual checkpoints and interrupted observations as partial evidence; do not claim full-integral agreement or convergence. Deliver the validated generation steering without repeating the expensive run. |
| Scientific uncertainty checks for the representative set | Reuse the [72 holdout rows](convergence-stage-a-independent.md): six families, three independent seeds, two work levels and two rules, all complete and no recorded comparison beyond the frozen investigation threshold. Do not repeat that matrix or automatically launch its earlier proposed larger matrix. Reuse the double-box [64-shift observation](direct-generation-performance.md) and [all-five-order checked comparison](remaining-reference-attempts.md), whose largest absolute pull is 1.26516. Off-shell scalar/rank-two, Issue 1 and hard-reference transports already close their documented normalization and value-comparison claims. Add bounded independent-seed/work checks only where these records leave a concrete representative uncertainty or accuracy question unresolved; retain every result and the original investigation criterion. |
| Highest-order accuracy and representative performance parity | Keep the original small one-loop, double-box, numerator-heavy and hard-four-loop representative set. Use the required matched seven ordinary/three expensive paired observations to close missing parity claims, with accuracy and full-vector checks in the same records wherever possible. Existing [seven-pair one-loop measurements](first-paired-performance-independent.md) remain valid for their recorded boundaries; different precision/persistence policies prevent relabelling them as final matched acceptance. Reuse their attribution rather than repeat exploratory profiling. Stop each case once the agreed criterion passes. |
| Eight-core target and latency reporting | The authorized license resolves the reference worker obstacle. Current seven-pair triangle generation/integration and independently reviewed [single-row latency](current-sample-latency-results.md) comparisons pass their measured criteria. Continue the remaining prescribed cases with matched eight-worker accuracy rows and the existing separate latency diagnostic; do not add a duplicate one-worker integration campaign or repeat closed triangle tuning. Preserve historical native observations and failed license attempts with their original boundaries. |
| Final delivery review | Reconcile the completed scientific and performance records with the regression matrix, examples, public APIs, dependency separation and current tests. Reuse the accepted Linux/MSRV/PTY evidence. Label unavailable-platform execution as unverified, as the plan allows; it is not a reason to add speculative platform work without a host. |

The existing low-work difficult-case vectors and reference errors do not yet
prove the one-per-mille highest-order target, and successful generation or a
single accurate representative does not establish performance parity. These
remain real acceptance gaps. Conversely, unknown external cross-order
covariance, qualified finite-sample calibration, and the hard result's separate
lower-order zero certificate must remain honest labels rather than triggers
for fabricated covariance, padded estimates or a universal-certification
project. Where the remaining matched representative block can supply both
accuracy and uncertainty evidence, use one block rather than separate studies.
After the required claims pass, mark the goal complete and stop; threshold
support and further optimization await the user's next instruction.

## Retained scope limitation

The latest bounded off-shell rank-two Pathfinder checks preserve the equivalent
eight-propagator family, including its two squared lines and exact numerator.
The ordinary route times out after 602.843 seconds; a separate 62.764-second
stack observation locates work in its optional sector-symmetry hashing. One
further 600-second attempt disables only that optional squashing. It reaches
all 1,182 unsquashed sectors and builds 134 endpoint-projector signatures in
10.305 seconds, then times out during explicit formula/Taylor construction
after 602.801 seconds (2.17 GB sampled peak owned RSS). No complete evaluator
bundle, strict load or numerical result exists. Inputs remain unchanged and
all owned processes are reaped. These are capability diagnostics with concurrent
build activity, not final timing comparisons or a proof of infeasibility.
The partial projector cache is retained; further rank-two attempts are stopped.
Evidence: `output/diagnostics/remaining-pathfinder-rank2-active8-1` and
`remaining-pathfinder-rank2-active8-nosymmetry-1`. The separate original hard
four-loop Pathfinder route now completes within one 600-second/15-GB attempt:
generation takes 524.117 seconds and strict bundle loading 2.747 seconds, with
3,728 nine-dimensional sectors and all orders `[-8,...,0]`. All inputs remain
unchanged and the owned process tree is reaped. This closes reference generation
capability, while integration, highest-order accuracy and sample timing remain
unmeasured. See the [bounded capability record](hard-four-loop-pathfinder-capability.md).

General affine endpoint charts remain unsupported: the public admission
correctly rejects `(1-x)^(-1+eps)` on the unit interval, including with
`assume_no_threshold`. That assertion cannot resolve endpoint geometry. No
shipped example has been identified as requiring this additional chart type;
retain the limitation explicitly rather than silently broadening first-phase
scope. General splitting/contour/GCAD, arbitrary complex masses and CBC
construction remain deferred. The subsequently authorized Python bindings are
delivered in the HEPKit PR above. Automatic family projection is
an optional optimization; native prepared-family APIs already preserve the
original route and fallback. None of these is a newly imposed completion gate.
