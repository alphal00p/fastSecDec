# Phase B delivery-gap audit

2026-10-10. Independent read-only audit begun at `a0c058a`, updated with the
subsequent reviewed K1 release smoke and K1* finite admission, against
`CONTOUR_DEFORMATION_PLAN.md` and the later requirements retained in
`FIRST_PHASE_PLAN.md`. Production implementation is the validated compact
increment at `3303005`; intervening evidence and the lightweight D05 fixture
do not establish additional numerical acceptance. No new implementation,
numerical run, dependency change or upstream defect is asserted here.

**Later priority change:** the user subsequently requested focus on the physical
D05 double box at 1000 GeV, bounded tuning and four five-minute/50-core
fixed/dynamic × QMC/discrete-MC results, then a green cleanup/push and an explicit
goal pause. The open LTD and extended physical gates below remain historical
backlog; they do not authorize more variance-only campaigns or delay that
requested pause. The root-owned plan addendum preserves the exact instruction.
The [1000 GeV audit](contour-gghh-double-box-1000-audit.md) follows the new scope.

The public native, CLI and HEPKit interfaces are implemented and exercised.
The remaining completion gaps are predominantly the required physical
multiloop and performance evidence, plus reproducible delivery of those
comparisons. A successful build, generated archive or finite pilot cannot
close those gaps.

## Current implementation and evidence

| Required surface | Live implementation and accepted coverage | Boundary |
| --- | --- | --- |
| Opt-in capabilities and runtime selection | CLI `GenerationInput::recipe_family` selects all four recipes for `--contour`, an explicit `--recipe` remains a singleton, and no-contour selects the existing undeformed path. Native `RecipeFamilySession` retains separate artifact default and requested resident selection. `contour_programs` actual-process tests cover ordinary/serial generation and consumption. | The saved family default is undeformed; storing a recipe never silently chooses a physical deformation. |
| Fixed and two dynamic constructions | Native settings expose fixed positive lambda, polynomial/sign-aware dynamics, S, L and R; default runtime validation is Always. CLI/TOML/Python forward the same native settings. Public analytic and sampling tests exercise both constructions, generation modes, subtraction strategies, complete complex outputs and saved restoration. | Default L=R=1 is a valid construction setting, not a promise of efficient sampling. |
| Optional validation and precision | `kernel/contour` retains actual owner/point pilot provenance, checked row execution, native certified arithmetic, precision retries and essential failure handling under Off. Public and lifecycle tests cover policy cycles and changed bindings. | A finite pilot is not a global floating-point certificate; Off retains termination/nonfinite safeguards. |
| Native differentiation and compact generation | Native coefficient function definitions, explicit native derivative bodies, FunctionMap/Dualizer lowering and owner codecs preserve implicit derivatives and face associations. Source-witness symmetry precedes opaque compact arithmetic. Current mixed-signature, higher-jet and fresh-process controls pass. | Actual 6D chart probes do not prove higher-dimensional performance. |
| Universal artifacts and bounded ownership | Native program archives, v12 retained-definition sidecars and historical readers preserve selected record restoration. Caller-owned family dispatch and serial workers bound heavy residency. Actual saved-owner, cancellation/resume and fresh-StateMap tests exist. | Loading retains native backend preparation; it is not claimed to restore cached machine code without JIT. Optional expensive semantic revalidation remains distinct from mandatory structural checks. |
| Sampling, covariance and checkpoints | Existing native QMC and Numerica Havana/discrete owners retain complete vectors and covariance; Korobov remains outside the complex map. Mathematical settings enter compatibility, while validation/diagnostic modes do not. Tests cover S/L/R/construction refusal, permitted policy changes, actual coordinate identity and frozen adaptive continuation. | Independently timed adaptive runs can choose different allocations; identical statistics are required for the same assigned work. |
| Operational status | Native `ContourRuntimeReport` and optional boxed `EvaluationDiagnostics.contour_runtime` preserve adaptation/production and nested preparation/exact/pilot/evaluation/conditioning work. Existing batch drains cover success, failure, discarded work and serial interruption; CLI text/JSON and frozen Python getters expose the data. | Ranges are approximate centres, not certificates or accepted-sample distributions. Abrupt worker death can lose unpublished IPC observations. |
| HEPKit and portable integration | Retained native graph/IntegralFamily ownership, deferred parametrization and caller-stepped generation remain in the isolated FastSecDec binding. Inspection holds native owners and performs no eager derivative/materialization. The compact snapshot passes installed native Python, actual Pyodide, portable native and generated-stub checks. | Actual Pyodide execution is not browser-widget UI testing. Shared installations remain untouched. |

Source boundaries checked include native `contour/{settings,definitions,
determinant,diagnostics}.rs`, `kernel/contour`, `generation/family`, artifact
readers, `status/{diagnostics,contour_runtime}.rs`; CLI `config.rs`,
`contour_cli`, `generate/family.rs`, execution/serial adapters and diagnostic
presentation; Python `family`, retained input, contour/runtime and lazy
inspection modules. Relevant maintained controls include native
`contour_dynamic`/`contour_dynamic_sampling`, CLI `contour_programs`,
`contour_dynamic`/`contour_runtime`, Python `test_contour_family` and
`test_contour_family_input`, and portable contour-family/diagnostics tests.

The latest compact regression evidence is 904 distinct workspace tests
passed, with 33 explicit ignores; its 401-test native library subset must
not be added again. The same accepted compact snapshot has 79 portable-host,
249 installed-native Python and 138 actual-Pyodide passes, plus current stub
and strict lint checks. This audit reads those existing gate records and
source; it does not claim a new test execution.

## Open scientific and performance gates

1. **Required massless K1 numerical result.** All 186 polynomial maps are
   independently compared with the retained fixed source geometry/F/U/faces,
   and four fixed/polynomial prescriptions now pass complete finite pilots.
   The subsequent release smoke completes all 186 sectors for four
   fixed/polynomial prescriptions and two frozen seeds. Actual coordinates
   and weights match, and the retained full common-shift covariance and
   complete complex vectors pass independent result review. No arm reaches
   the requested accuracy: polynomial L=0.1 has 14.32% and 13.89% relative
   joint SE, while the smaller caps and fixed choices are badly underresolved.
   These noisy variance comparisons do not establish convergence,
   published-reference accuracy or time to target. The single sign-aware
   chart's saved-owner parity is still one-chart evidence. See
   [the release smoke report](contour-ltd-k1-variance.md). A fresh standard
   CLI adaptive accuracy run was subsequently interrupted for the user's new
   priority, with its partial checkpoint/result retained; no 0.1% acceptance
   is claimed.

2. **Required massive K1* result.** All four recipes publish successfully
   with 30 stochastic sectors each. The completed saved-owner gate verifies
   all 240 record digests, 60 native source-map comparisons and six complete
   finite pilots, with 2,880 actual source points and separate aggregate exact
   readiness. The conditional source-zero phase completes 98,304 assigned
   points with matching native coordinates/weights and no production causal
   checks under Pilot. This establishes finite admission and source-zero
   feasibility; complete 30-source integration, independent reference
   agreement, full covariance, repeated variance and practical time to target
   remain open. Its stored undeformed recipe is a capability, not physical
   acceptance on the real contour. See the
   [massive runtime report](contour-ltd-massive-runtime.md).

3. **Other required multiloop cases.** The tracked `2L6P.a.I` fixture has no
   accepted complete contour integral in this evidence set; native
   preparation is not numerical acceptance. The required physical
   `3L4P.K1` ladder likewise needs generation, validated execution and its
   analytic reference gate. Run analytic/two-loop prerequisites first as
   the plan requires. The four-loop `4L4P.a.I` fishnet is explicitly an
   extended bounded stress test, not an initial delivery blocker.

4. **Physical double box.** The maintained 400 GeV D05 fixture passes native
   kinematics, actual incoming helicity, inherited normalization and input
   reproduction checks. No double-box generation/integration or independent
   numerical comparison is accepted yet. A single D05 diagram must not be
   described as a complete gauge-invariant two-loop amplitude. The full
   one-loop 400 GeV amplitude and both numerical Ward contractions have
   separate accepted fixed/dynamic evidence and do not close this gap.

5. **Required performance matrix.** The private verified opt3/thin-LTO
   consumer is now available, but compilation itself measures no sampling
   performance. Required multiloop measurements still include evaluator
   size, bounded process RSS, per-sector cost, root/checking overhead,
   repeated matched-design variance and time to 10^-3 relative uncertainty
   in the last complex coefficient on eight cores. Existing toy/physical-box
   results already include neutral and worse dynamic outcomes. Default-cap
   physical failures must remain failures, and algebraically identical
   low-degree polynomial/sign-aware constructions are not independent
   corroborating numerical methods or pooled estimates.

6. **Timing scope.** Existing phase and batch counters provide sector mean
   costs and the maximum of those means. They do not retain the worst latency
   of an individual sample. The latter needs a separately bounded measurement
   if reported as the plan's maximum sample time. Do not rename a maximum
   weighted amplitude or a slowest-sector mean as maximum sample latency.

7. **Maintained reproduction.** Required input cards and native importers
   are tracked and standard CLI generation/integration is available. The
   current massless K1 map/pilot/paired drivers are ignored exploratory
   harnesses; the maintained `contour_variance` example covers toy controls.
   Final accepted multiloop comparison publication needs either a concise
   maintained native driver reusing those owners or precise standard CLI
   commands, seeds, frozen allocations, selected recipes, settings,
   normalization, references and report interpretation sufficient to
   reproduce it. Private authorization markers, local fingerprint discovery
   and process monitors need not become numerical library code. This is not
   a blocker to the exploratory cost or matched-work measurements.

## Limits to retain honestly

The current residual-U admission uses exact rational nonnegative native
polynomial coefficients with a positive constant lower bound. This covers the
admitted graph-U cases; arbitrary parameter-dependent positive residuals
without that proof are rejected. Expanding that class is not an established
requirement blocker for the prescribed graph fixtures, and sampled positivity
would not justify relaxing it.

The native determinant template cache is bounded to dimensions 4–6; other
dimensions use the existing native determinant path. Compact 6D success does
not establish practical 3-loop generation cost. A separate native factored
bordered-matrix probe and all-face proof now give a plausible eight-/nine-
dimensional route, but physical mapping and higher-jet acceptance remain
pending. That research also reproduced an unrelated `det_in_place` row-swap
sign defect and published its narrow owner fix; the current contour path uses
the unaffected `det()`. See the
[higher-dimensional audit](contour-higher-dimension-determinant-audit.md).

Large retained map metadata still has a measurable restoration cost. The
approved default loader avoids optional full semantic/canonical revalidation;
the trusted K1 comparison instead fences complete file identities and performs
its stated native comparisons. Neither policy is a license to omit mandatory
structure/backend/schema checks. Further optimization must preserve those
owner boundaries rather than introduce another serializer or evaluator.

## Historical wording versus current omissions

Several early review paragraphs are dated design states: fixed-only public
generation, closed dynamic admission, missing operational diagnostics,
pending installed HEPKit/Pyodide execution and incomplete K1 mapping/pilots.
Those statements are superseded by later executable milestones and the live
interfaces above. For example, the early diagnostics section in the variance
protocol predates `ContourRuntimeReport`, while the later native/status
reviews record its implementation and tests. The progress ledger's later
saved-K1 row supersedes the earlier memory row's pending-pilot wording.

These historical notes should not trigger duplicate implementation. Conversely,
the earlier multiloop numerical/accuracy/performance gates and reproduction
deliverables remain unfulfilled evidence boundaries. The later user priority
controls current work and its explicit pause point; retaining this backlog
does not authorize continuing after that pause.
