# Phase-one remaining gates — reconciled through `22dc1d9`

Source review, 2026-10-05. This reconciles the earlier remaining-gates audit with
validated milestones through `22dc1d95f3cf5a96f412f60f1d151bafe334b0d8`.
[FIRST_PHASE_PLAN.md](../../FIRST_PHASE_PLAN.md) remains authoritative; this
document adds no requirement and claims no new test execution. The latest
combined gate is **370 passed, zero failed, 23 ignored**, with workspace
formatting and all-target Clippy passing. See the
[current status and evidence](implementation-status-2026-10-05.md) and
[independent public integration audit](native-named-request-interface-independent.md).

The [regression matrix](../REGRESSION_MATRIX.md) retains **182 rows: 100 Covered,
81 Retired and one Partial**. The Partial row, `test_integrals.py:6113`, now has
an independent complete double-box reference; uncertainty calibration remains
open. Retired backend, Python, sector-numbering and serialization interfaces do
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

## Next end-to-end capability gates

1. **Public representative prerequisite is now accepted.** The real `UnitCube`
   input passes exact native regular/density identity, ordinary `NativeNamed`
   generation, compilation and artifact persistence. The actual reversed
   coordinate map, multiplicity one and literal-zero exact offsets are checked.
   All 21 signed coefficient/point comparisons and 24 cold weighted vectors
   pass, including twelve forced 1,024-bit replays. See the
   [independent public reconstruction audit](native-named-public-actual-independent.md).
   This closes the prerequisite; it does not establish full-integral agreement.

2. **Complete the original on-shell triple-box graph.** After that public gate,
   freeze the new release/source/dependency identities and run the reviewed
   [native named full-graph protocol](native-named-fullgraph-protocol.md).
   The sole scientific steering change is the explicit named coefficient
   option. It preserves the full original graph, kinematics, order zero and O2:
   generation/compilation/save has a 1,800-second limit plus five seconds of
   grace, followed only on success by cold inspection and a complete fixed
   full-integral allocation, each with its own 180+5-second limit and 30-GiB
   address-space cap. Require complete native coverage, all coefficients,
   exact offsets, covariance and portable reload. The previous physical trial
   [timed out](native-series-fullgraph-attribution.md) after 1,805.689 seconds
   with 80 of 1,026 representatives complete and no artifact; it remains
   unchanged evidence. The new release campaign is independently preflighted
   under `output/diagnostics/native-named-fullgraph-2`; its scientific outcome
   remains pending. The first release build succeeded, but its freezer rejected
   a logging-only feature difference. That failed freeze is retained; the
   second bundle records the exact ordinary-release feature set and the
   byte-identical executable.

3. **Establish original-integral numerical agreement.** Successful generation,
   artifact inspection and the fixed allocation establish capability and
   transport only. The original on-shell integral still needs complete-vector
   independent numerical agreement and subsequent convergence/calibration.
   The full-integral saved scope/design, rather than command flags or a
   successful representative, determines coverage. Reuse any existing
   independently audited reference with matching input and normalization.
   The [current reference inventory](reference-onshell-full-vector-proposal.md)
   finds no Checked original on-shell full vector; the two off-shell fixtures
   do not substitute for it. Its minimal ordinary pySecDec reuse proposal is
   separate from the source-only Pathfinder generation-baseline protocol.

## Remaining scientific and uncertainty gates

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
remain evidence. The paired eight-core Pathfinder route hit an explicit
concurrent-instance license limit; one-worker measurements and the new
capability diagnostics do not close that comparison.

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
  multiloop and hard four-loop cases using corresponding full-support reference
  estimators. Incorrect values or underestimated uncertainty fail acceptance.
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
must continue to need no Python, pySecDec, FORM, Normaliz or reference checkout.

## Bounded completion addendum, after `a56107f`

The user's stopping rule narrows further work to unresolved acceptance claims.
The family-level uncertainty entries above are limitations to assess against
the retained evidence, not instructions to launch a new calibration campaign
for every card. No finite test block can certify uncertainty for every possible
integrand. Preserve that qualification after completion rather than turning it
into an unlimited gate.

| Minimum remaining decision | Evidence to reuse and bounded next action |
| --- | --- |
| Original on-shell integral capability and agreement | The ten-parameter NativeNamed trial was intentionally cancelled at 372/1,026 representatives after 1,006.900 seconds, with no artifact. Use the existing exact native prepared-family route for the next bounded trial of the same integral. Both original and projected external reference generation attempts reached their 30-GiB memory limits without a result. The projected representation preserves the completed native repeated-propagator identity, on-shell point, raised-power measure and Gamma factor. Review the provider's existing Taylor-subtraction option before another bounded attempt. A projection is an equivalent representation, not reduced integral coverage. Retained failed attempts supply no final integral vector. |
| Scientific uncertainty checks for the representative set | Reuse the [72 holdout rows](convergence-stage-a-independent.md): six families, three independent seeds, two work levels and two rules, all complete and no recorded comparison beyond the frozen investigation threshold. Do not repeat that matrix or automatically launch its earlier proposed larger matrix. Reuse the double-box [64-shift observation](direct-generation-performance.md) and [all-five-order checked comparison](remaining-reference-attempts.md), whose largest absolute pull is 1.26516. Off-shell scalar/rank-two, Issue 1 and hard-reference transports already close their documented normalization and value-comparison claims. Add bounded independent-seed/work checks only where these records leave a concrete representative uncertainty or accuracy question unresolved; retain every result and the original investigation criterion. |
| Highest-order accuracy and representative performance parity | Keep the original small one-loop, double-box, numerator-heavy and hard-four-loop representative set. Use the required matched seven ordinary/three expensive paired observations to close missing parity claims, with accuracy and full-vector checks in the same records wherever possible. Existing [seven-pair one-loop measurements](first-paired-performance-independent.md) remain valid for their recorded boundaries; different precision/persistence policies prevent relabelling them as final matched acceptance. Reuse their attribution rather than repeat exploratory profiling. Stop each case once the agreed criterion passes. |
| Eight-core target and latency reporting | Reuse the [fourteen native eight-core rows](eight-core-native-results.md) for their actual finite-part target crossings, and the accepted [individual-sample diagnostic](native-sample-latency-results.md) for its distinct timer boundary. Neither needs a new diagnostic merely for documentation. The reference eight-worker instance-limit failure still prevents a paired eight-core claim; repeating the same rejected configuration or extrapolating fewer workers would not close it. Record the remaining measured target/worker gaps explicitly and resolve only those needed by the plan. |
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

General affine endpoint charts remain unsupported: the public admission
correctly rejects `(1-x)^(-1+eps)` on the unit interval, including with
`assume_no_threshold`. That assertion cannot resolve endpoint geometry. No
shipped example has been identified as requiring this additional chart type;
retain the limitation explicitly rather than silently broadening first-phase
scope. General splitting/contour/GCAD, arbitrary complex masses, Python
bindings and CBC construction remain deferred. Automatic family projection is
an optional optimization; native prepared-family APIs already preserve the
original route and fallback. None of these is a newly imposed completion gate.
