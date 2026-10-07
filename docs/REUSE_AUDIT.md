# Ecosystem reuse evidence

## Batched numerical execution (2026-10-07)

The f64 runtime uses Symbolica's public real/complex
`JITCompiledEvaluator::batch_evaluate`, which delegates to SymJIT's native matrix
evaluator and SIMD tail handling. Point-major input matrices include the bound
runtime parameters. SymJIT's internal threading is explicitly disabled so the
CLI remains the worker-pool owner. Native eager and higher-precision evaluator
owners currently expose scalar evaluation only; the batch adapter retains those
owners for portable consumers and selective precision rescue.

QMC transforms points before batching. Both Havana paths retain Numerica's
sampling, training and accumulators; discrete-sector chunks group by sector for
evaluation and restore original sample order for training and covariance.
Precision admission and maximum-weight updates retain sector-local point order.
No alternative sampler, evaluator interpreter, graph representation, algebra or
one-loop reference implementation is introduced. The independent
[batch API and numerical review](reviews/batched-evaluation-reuse.md) records
native ownership evidence and validation limits. The
[runtime validation report](reviews/batched-evaluation-results.md) records actual
ggHH comparisons, checkpoint continuation, portable eager execution and terminal
cleanup, including the deferred permanent-test coverage boundary.

## Configurable native evaluator optimization (2026-10-07)

`CompilationSettings` is a serializable adapter around Symbolica's public
`OptimizationSettings` and `EvaluatorBuilder`, not an optimizer implementation.
The native defaults are retained except for the explicit CPE cap of 1000 rounds;
Horner iterations now default to 10 instead of the previous local override of
zero. Serial and caller-dispatched compilation use the same settings, including
runtime-dependent exact offsets. Native eager and SymJIT O2 consumers share the
same exact optimized program.

Native seeded single-core searches preserve deterministic sector compilation.
Generation workers remain caller-owned. Maximum common-pair distance is passed
through and recorded but is currently unused by the pinned upstream optimizer;
the cache-entry limit is active. Native expression hot starts and abort callbacks
are programmatic hooks, not scalar run-card options. Verbose generation requires
plain human output, and loading an artifact never replays optimizer log output.

The existing artifact policy string binds the full requested settings without a
new binary layout. Legacy zero-Horner policies restore their original settings,
bytes and identities. Independent API/IR evidence is recorded in the
[native optimizer review](reviews/horner-reuse-review.md),
[implementation controls](reviews/horner-defaults.md), and
[CLI/metadata review](reviews/evaluator-settings-cli-review.md). Native graph,
algebra, one-loop master and reduction owners remain unchanged; no alternative
CAS, optimizer, graph representation or integration method was introduced.

## Dashboard estimate and macOS memory semantics (2026-10-07)

Completed-batch/lattice estimates remain native `VectorEstimate` values. The
dashboard keeps a separately labeled last completed allocation for display
during later refinement, without changing admission, checkpoint or accuracy
logic. Waiting states distinguish pilot work and insufficient independent
coverage. The peak-sample display reuses the existing per-sector maximum of the
importance-weighted complex coefficient magnitude.

The macOS memory correction reuses sysinfo's native free-memory query. Its
available-memory estimate includes active pages and overlaps used memory;
it cannot be presented as free RAM. Raw counters remain available in structured
observations. Independent native-source/probe evidence is recorded in the
[memory and counter review](reviews/dashboard-counter-semantics.md) and
[accepted-estimate audit](reviews/accepted-estimate-audit.md). No algebra,
numerical reference, graph, integrator or evaluator implementation changed.

## Integration dashboard layout and mouse interaction (2026-10-07)

The integration presentation reuses Ratatui blocks, tables, gauges, Unicode
cell widths and table selection for the full-sum, sector and runtime panels.
Crossterm owns mouse capture and event decoding; the existing terminal lifetime
owner restores mouse mode together with raw mode, cursor and alternate screen.
Clicks and wheel events act on cached observations and rendered hit regions.
Sysinfo's existing whole-process/system snapshot supplies RAM and CPU counters.

Precision dispatch, sample counts and timing panels consume existing native
operational observations. No sampling, replay, covariance, algebra, graph, or
one-loop master/reduction implementation changed. The established uncertainty
formatter remains authoritative; layout aligns its multiplication separator.
Dashboard durations use a native fixed-decimal adapter with µs/ms/s units.
Counts use a presentation-only four-significant-digit K/M/B adapter, retaining
the original integer counters for statistics and sorting.
The [independent presentation/reuse review](reviews/integration-dashboard-polish-review.md)
and [acceptance record](reviews/integration-dashboard-polish-results.md) record
formatting, layout, cached interaction and terminal cleanup evidence. Other
examples and the permanent test/gate migration remain deferred.

## Runtime stability and live observations (2026-10-07)

The default runtime stack reuses native Symbolica evaluators and Numerica numeric
domains: f64/SymJIT O2, 106-bit DoubleFloat, and Float at 1000 decimal digits
(3322 bits). Higher levels map the retained exact evaluator through its existing
numeric-domain API and cache the resulting eager evaluator. GammaLoop's public
runtime settings and Pathfinder's endpoint-distance/max-weight behavior informed
the policy; their private momentum-rotation checker is not transplanted. The
explicit validated policy retains the earlier checks. Distance routing deliberately
does not interpret a zero real component as an underflow certificate or failure.

Original endpoint powers come from the existing native Rational affine-exponent
admission. They remain associated with retained subtraction pieces through IBP;
mapped conservative bounds retain source alternatives before normalizing by
per-power thresholds. Format 7 extends the existing context-aware binserde
artifact with this provenance; explicit version-5/6 readers preserve historical
bytes and identities. No new algebra, DOT parser, graph representation or
phase-analysis helper was introduced. See the independent
[endpoint provenance review](reviews/runtime-endpoint-profiles.md) and
[stability/codec review](reviews/runtime-stability-review.md), with supporting
[native stack evidence](reviews/runtime-stability-stack.md).

MC preview statistics reuse Numerica `StatisticsAccumulator<DoubleFloat>`.
The missing public operation is merging differently centered live prefixes with
implicit zero draws for discrete-sector marginals. A focused observation adapter
pools native means and standard errors without replacing accepted full-vector
covariance or training. QMC previews use complete native shifted-lattice rows and
the existing authoritative common-shift/independent-sector reduction. One
complete shift has a mean but no uncertainty. Caller-owned dispatch, complete
work admission, RNG ownership and checkpoint semantics remain unchanged.
Independent dense/native controls and old/new complete-statistics equivalence
are recorded in the [observation implementation review](reviews/live-integration-observations.md)
and [independent numerical review](reviews/runtime-live-numerics.md).

The CLI uses Ratatui's native table/selection, Tabled's report layout, Spenso's
Unicode superscripts and Numerica's uncertainty formatter where its policy
matches the requested notation. A presentation adapter supplies fixed normalized
exponents, dominant-error handling and strict last-digit parentheses. OS CPU time
uses the existing sysinfo process counter, independently of summed instrumented
worker/coordinator elapsed spans. No library owns a worker pool or refresh loop.
The [dashboard review](reviews/runtime-dashboard-display.md) records native
formatter/API probes, cadence and terminal controls, CPU-counter validation,
and an independent review of the displayed numerical quantities.

Focused portable-host execution passed fresh eager generation, cold format-7
loading, DoubleFloat/3322-bit evaluation, runtime bindings, cutoff/replay controls,
and a complete 4096-point complex QMC calculation with full covariance. This
does not establish actual Wasm/Pyodide execution. Native one-loop master and
reduction reference APIs remain the existing owners; no new reference formula
was needed for this runtime-only change. Other examples and permanent test/gate
migration remain deferred. Final executable checks are recorded in the
[runtime acceptance record](reviews/runtime-dashboard-results.md).

## Discrete MC responsiveness and terminal ownership (2026-10-07)

The CLI reuses its caller-owned Rayon pool and existing scoped QMC dispatch
pattern to poll status and input while native Havana batches execute. Numerica
still owns the discrete/continuous sampler, adaptation and RNG; FastSecDec's
existing native session owns whole-batch statistics and full-vector covariance.
Completed batches retain original admission order. Cancelled prefixes update
only work diagnostics, not the accepted estimate, replay state or checkpoint.
Native restore reissues missing reservations with identical task/RNG identity.
There is no numerical-library worker pool or replacement integration algorithm.

CLI-only atomics expose unfinished worker activity separately from accepted
statistics. Terminal ownership composes Crossterm and signal-hook: signal
callbacks use atomics, and a CLI-owned blocking signal iterator performs forced
cleanup in ordinary Rust. Scoped drop and a panic hook restore owned terminal
modes; the listener and signal registrations are released on normal return.

The independent [numerical and API review](reviews/discrete-mc-responsiveness-review.md)
and [dispatch review](reviews/discrete-mc-dispatch.md) record native checkpoint,
RNG and full covariance controls. The existing complex-underflow safeguard
conservatively rescues purely imaginary ggHH outputs when sampling weights
exceed one. Public Symbolica APIs do not expose the required individual-output
structural zero proof; neither numerical zero nor error-tracking zero is such a
proof. No precision shortcut or duplicate phase-analysis helper was added.
One-loop master/reduction owners and portable numerical backends are unchanged.
Terminal probes and final release checks are recorded in the
[terminal review](reviews/discrete-mc-terminal.md) and
[acceptance record](reviews/discrete-mc-results.md). Permanent tests/gate migration
and other examples remain deferred as requested.

## Artifact inspection and saved generation facts (2026-10-07)

Artifact inspection reads the existing native `GenerationMetadata`,
`CoordinateMap`, pre-subtraction terms and `EvaluatorStatistics`. Stable kernel
IDs come from the kernel slice; chart associations and representative
permutations come from the retained records. Native Atom multiplication assembles
the recorded endpoint factors for display without inferring a new leading term,
named U/F factor or threshold certificate. Symbolica's `Atom::printer` and
`PrintOptions` own expression formatting. Namespace suppression uses native
symbol inventories to avoid collapsing distinct symbols into the same label.
The CLI's existing `tabled` dependency supplies width-aware tables, with its ANSI
feature handling colored native expressions.

The Pathfinder reference's bounded sector table informed the presentation, not
the physical representation or algebra. No graph helper, evaluator traversal,
CAS or numerical reference implementation was added. The exact program byte
count and operation counts reuse observations captured when the shared native
Laurent-vector evaluator was built; compressed SymJIT data is labeled separately.
Existing one-loop master and reduction APIs remain unaffected.

The optional human JSON generation record adds the saved worker count and
requested expansion method. Other summary facts reuse existing kernel metadata
and timings. These observations are excluded from scientific identity and do
not alter the binary artifact. The [persistence review](reviews/generation-record.md)
and independent [native inspection review](reviews/artifact-inspect-review.md)
record source/API probes, attribution and validation boundaries. The
[release acceptance](reviews/artifact-inspect-results.md) confirms actual
terminal behavior, portable compilation and byte-identical fresh ggHH
evaluators. Other examples and the user's deferred tests/gates migration remain
unchanged.

## Generation terminal summary and count availability (2026-10-07)

The human generation summary uses typed native kernel metadata and measured
`GenerationTimings`; it pairs each epsilon order with the native real/imaginary
component tags. Static table layout reuses `tabled`, already present in the
resolved ecosystem, through a CLI-only direct dependency. The existing stdout
color policy remains authoritative. No numerical or graph owner changes.

Live coefficient-count availability follows the existing typed phase events.
Composition reports the subtraction-piece count at coverage, and coefficient
requests/shared expressions become running counts during resolution. Earlier
placeholder zeros are omitted; final measured zeros remain visible. Raw JSON
fields and numerical events remain unchanged. The independent
[presentation review](reviews/generation-summary-review.md) records public-API
reuse, terminal-width/color controls and focused native-event probes. Other
examples and the deferred tests/gates remain untouched.

## Deterministic generation and dashboard follow-up (2026-10-07)

Complete-density assembly and Graphica canonicalization now run as independent
opaque jobs in the existing caller-owned generation dispatch. Admission orders
results by original chart index and retains Symbolica's exact simultaneous
substitution check before merging densities. Native and portable probes compare
serial, one-worker, four-worker and reversed-completion execution, including
asymmetric and opposite-sign numerators. The D05 scheduling-only comparison
preserves the entire binary evaluator artifact byte for byte. No graph owner,
canonicalizer, equality engine or numerical-reference implementation is added.
See [parallel symmetry](reviews/parallel-symmetry.md) and the
[independent review](reviews/generation-followup-review.md).

CLI output-basename preflight reuses the artifact path validator before input
loading. Dashboard RAM sampling uses `sysinfo`'s current-process RSS and native
system memory counters on existing polls, with a sampled peak and explicit
unavailable values. It adds no background thread, per-worker memory estimate or
core dependency. See [preflight](reviews/generation-preflight.md) and
[memory ownership and measurements](reviews/generation-memory.md).

The public expansion names are `coefficient_series` and `full_expression`;
legacy input aliases remain for the explicitly deferred example/test migration.
The installed native toolchain supplies focused checks on this host, where
`nix-shell` is unavailable. No deferred full-suite gate is claimed.

## Runtime model inputs and readable native DOT (2026-10-07)

`RuntimeModelBindings` composes HEPKit's `Model::scalar_bindings` and its native
independent-parameter boundary. External and expressionless internal leaves
become real evaluator inputs (ordered real/imaginary components for complex
leaves). Dependent definitions remain native expressions; cached model values
and card defaults do not replace those expressions. Only inputs used by the
prepared integral or its mass constraints are retained, in stable name order.
The native particle mass API distinguishes structural `ZERO` from a named mass
whose numerical default happens to be zero. Width restrictions remain explicit.

The missing operation was admission of declared symbolic real masses through
FastSecDec's previously numerical-only boundary, with a finite real nonzero
runtime-domain check. Symbolica evaluates these constraints; FastSecDec adds no
dependency solver, evaluator, physical graph type or threshold certification.
Constraints use the same context-aware Atom binserde and exported Symbolica
state as the other artifact expressions, and contribute to semantic identity.
Binary version six has an explicit version-five reader. See the
[native model API and frontend review](reviews/runtime-model-parameters-review.md)
and [independent runtime and DOT review](reviews/generation-dot-and-runtime-review.md).

Linnet's structured DOT import, escaping and serializer produce graph attributes
on separate lines. A complete native diagram JSON round trip verifies the
presentation. HEPKit's stable DOT contract requires the global numerator to
equal its local fragments, so removing the duplicate in `v0` is invalid; the
original supplied source is preserved. Existing one-loop master and reduction
providers remain unchanged; exact dependency checks, parameter variation and
prior-template comparison are the appropriate local references for this work.
The [final D05 acceptance record](reviews/generation-followup-results.md)
includes byte-identical four/eight-worker generation, complete-vector coupling
variation, artifact relocation and the bounded full-integral comparison.

## Supplied gg→HH s-channel diagram (2026-10-07)

The example now targets the user's supplied `D05` graph: the two gluons attach
to one box and the two Higgs legs to the other. Native HEPKit DOT import, model
identity, diagram canonical keys and numerator construction remain the owners;
Linnet provides circuit enumeration and structured DOT transport. The original
model snapshot in the pinned GammaLoop source matches the supplied fingerprint
and permits strict native import. The example retains its explicit mass/coupling
card, color projection and helicities. No graph parser, canonicalizer, algebra
or numerical-reference implementation is added. The earlier mixed-leg `FK018`
measurements remain historical and do not validate this corrected input.

The [independent s-channel review](reviews/gghh-s-channel-review.md) records
the source/model compatibility checks and the final verification boundary.

## Parameterized CLI generation and binary artifacts (2026-10-06)

Runtime scalar products use the existing HEPKit `Kinematics` and Symbolica
ordered evaluator inputs. The same native evaluator owns exact offsets,
ordinary samples and multiprecision rescue. Native and portable eager backends
remain available. No new graph, algebra, sampler or estimator is introduced.

Artifacts reuse Symbolica's native Atom binserde with exported state and
context-aware decoding, following GammaLoop's `HasStateMap` pattern. Evaluator
programs use the existing owner serde/bincode adapter because a focused probe
and gg→HH cold reload exposed sign loss in Numerica 3.0.1's native GMP encoding
of large negative integers; no new numerical codec or dependency patch is added.
Scientific identity is separate from state-dependent binary integrity.
Generation dispatch composes the existing geometry, mapping and Laurent owners;
the CLI owns threads and presentation. One-loop masters and reduction remain
the existing independent numerical-reference owners, unchanged by this work.

See the [revision record](reviews/gghh-generation-revision.md) and the independent
[runtime review](reviews/generation-runtime-review.md),
[artifact review](reviews/generation-artifact-review.md), and
[parallel review](reviews/generation-parallel-review.md). Other examples and the
test/gate migration are deferred by explicit user instruction.

## Native notebook progress and citations (2026-10-06)

The public `generate_diagrams(progress="auto")` behavior, its Rust source and an
installed marimo detection probe were checked before adding sector progress.
Its private presenter is extracted into `feynkit_py::MarimoProgress`; the
diagram generator and FastSecDec share that implementation. Native generation
snapshots remain authoritative, callbacks receive every event, and only automatic
widget painting is coalesced. The standalone notebook calls this API directly.
Focused helper lifecycle and strict scoped checks pass; integrated validation
of the combined wheel is recorded separately.

Citation transport reuses Symbolica's native `Citation` type, rich display,
BibTeX export and cumulative deduplicating collector. FastSecDec owns its four
references and a process usage flag; community adds only its feature-gated
collector call. The independent audit matches native cone/triangulation and
endpoint subtraction to the primary Kaneko–Ueda and Binoth–Heinrich papers.
The user-requested pySecDec reference credits development and cross-checks;
FastSecDec itself has an explicit software repository citation. Both notebook
final cells depend on native work before refreshing the bibliography. See
[citation data and sources](../citations/README.md).

This is an implementation record, not permission to replace ecosystem features.
Source paths below are relative to the pinned local checkouts documented in
[DEVELOPMENT.md](DEVELOPMENT.md). Each new capability must pass the three checks
in the plan: public API, implementation/tests, and an executable Rust probe.

| Operation | API and implementation inspected | Executable evidence | FastSecDec responsibility |
|---|---|---|---|
| Native DOT and physical graph | FeynKit `feynkit-graph/src/lib.rs`, `FeynmanDiagram::from_dot`, `diagram.graph`; Linnet `HedgeGraph` | `native_input::native_dot_and_linnet_remain_the_graph_owner` includes DOT roundtrip checks | Validate integral settings around native objects; no parser or graph replacement |
| Momentum routing, tensor contraction | Native loop basis `route_expression`; Idenso/Spenso `SymbolicTensor`, algebra settings and scalar structure | Native tensor, free-index and graph-weight tests in `native_input.rs` | Require a scalar after existing contraction and projector application |
| Integral families and numerator basis | `feynkit-graph/src/integrals.rs`: `scalar_products`, `complete`, `rewrite_numerator` | `ecosystem_probes::family_completeness_and_isp_rewriting_are_native` | Compose these APIs; no parallel denominator-basis solver |
| Symanzik polynomials | `feynkit-graph/src/integrals/parametric.rs`: `symanzik` | Native bubble U/F identities and normalization tests | Supply powers, dimensional regulator and measure convention |
| Polynomial arithmetic, derivatives, factoring | Symbolica `src/atom/core.rs`, `src/poly/polynomial.rs`, and community wrappers | `ecosystem_probes::exact_polynomial_operations_stay_in_symbolica` | Orchestrate operations for sector subtraction; no CAS implementation |
| Gamma Laurent series | Symbolica `AtomCore::series`, transcendental functions and series implementations | Exact `gamma(eps) = 1/eps - euler_gamma + O(eps)` probe | Determine required expansion depth and convolve coefficients |
| Evaluators and portable JIT representation | Symbolica `src/evaluate/backend.rs`, `JITCompilationSettings`, `JITCompiledEvaluator` | `ecosystem_probes::symjit_o2_runs_and_roundtrips_portable_ir` includes batch evaluation and serialization | Explicit O2 settings, validation, kernel grouping and artifact compatibility |
| Numerical conditioning and multiprecision | Native `ExpressionEvaluator::map_coeff_with_prec`, `ErrorPropagatingFloat`, and Numerica `Float`/`Complex` | Real endpoint regressions and complex Gamma/logarithm rescue on an independent worker at `x=1e-80` | Prebuild numeric evaluator IR, detect conditioning loss, compare escalating precisions, and report rescues; no alternate arithmetic or special functions |
| Literal-zero facts after artifact loading | Symbolica `ExpressionEvaluator::export_instructions`, exact rational constants, native `Instruction` and `Slot` | [24 focused artifact/replay controls](reviews/weighted-replay.md) preserve real/complex underflow rescue, initial replay and unchanged bytes/IDs | Recognize only a single direct literal-zero output assignment, excluding deferred constants and uncertain instructions; no sampled proof or simplifier |
| Exact integers, rational matrices | Numerica integer/rational domains and matrix determinant/rank operations | Exact sector map, moment and cone tests | Sector-specific normal-fan and triangulation orchestration only |
| Factored polynomial admission | Symbolica `AtomCore::is_polynomial` and `to_polynomial_in_vars` | `parametric_ir` accepts a compact power of 10,000, retains external-function coefficients through the native fallback, and rejects hidden coordinate dependence | Check declared variables and input domain; no expansion for the common admission path |
| Complete-density variable symmetry | Symbolica's reexport of Graphica `Graph::canonize`, followed by native literal `replace_multiple` | Seven author tests and three independent full-integral tests, including asymmetric numerators and opposite/unequal prefactors | Encode the existing factored Atom DAG; merge only when the proposed permutation reproduces the entire density exactly |
| One-loop scalar master references | Community HEPKit's `oneloop` dependency, `evaluate_with_backend` and `ScalarIntegral` | Three `hepkit_one_loop` tests compare complete generated/JIT/QMC vectors at 11 B0/C0/D0 points, with explicit native normalization and scales | Development-only cross-check composition; no copied master formulas or reduction algorithm |
| One-loop numerator reduction references | Native `oneloopreduce::reduce_family`, public reduction terms and `OneLoopMasters::symbol_with_scale`; community bridge composition | Five `hepkit_numerator_reduction` tests compare complete generated/JIT/QMC vectors at eight rank-one/rank-two/rank-five points, including zero external Gram determinant | Test-only composition of native reduction, Symbolica coefficient series and native masters; no duplicated reduction or Gram solver |
| Rational antiderivative diagnostics | Current Symbolica `RationalPolynomial::integrate`; the separate Rubi wrapper delegates its rational fallback to this native API | `full_double_box_integration::double_box_leading_pole_is_an_exact_total_derivative` verifies the actual ten nonzero leading-coefficient terms using native integration, differentiation and cancelling boundary values | Diagnostic proof that the generated leading pole integrates to zero; no replacement antiderivative engine or production analytic lowering |
| External reference comparison | Numerica accumulators/QMC covariance, HEPKit native reference providers, and GammaLoop comparison consumers | Nine author and independently repeated `reference` tests cover sparse real/imaginary keys, explicit uncertainty, compatibility evidence and strict versioned transport | Align the existing `VectorEstimate` with typed external results and present scalar differences; no new estimator, covariance model, normalization algebra or stopping rule |

The initial probes establish the existing owners. The implementation must add
evidence to this record before introducing further algebraic functionality.

## Narrow missing operations

- General polynomial loop-numerator **integration into parameter space** is not
  provided by the inspected family numerator-basis APIs. The implemented adapter
  uses native scalar-product bases and auxiliary Symanzik sources, then Symbolica
  derivatives. Eight Gaussian-moment tests pass, including raised propagator
  cancellation, routing shifts, Gram-degenerate kinematics, rank-four tadpoles,
  and a factorized two-loop mixed moment. An independent review reran them and
  found no issue; see [the numerator review](reviews/gaussian-numerator-initial.md).
  Removable auxiliary Gamma poles at an exact integer dimension still require
  a regulated dimension and are rejected explicitly. Full numerator example
  integration remains a separate acceptance gate.
- The inspected ecosystem has general exact linear algebra and graph
  canonicalization, but no complete sector-specific Newton-fan decomposition
  pipeline. `fastsecdec-sectors` supplies that pipeline while using Numerica exact
  arithmetic. The independent review is recorded separately under `reviews/`.
- Existing Havana accumulators describe independent Monte Carlo samples. The new
  Numerica QMC lane supplies indexed randomized rank-one points and statistics
  over complete shift means. It does not replace Havana MC or its numeric types.

Complete-integrand graph canonicalization now uses Graphica; see the independent
symmetry review in `reviews/`. Symbolic integration remains an optional future
application of existing APIs. Precision rescue now uses
prebuilt native numeric evaluator IR; workers do not construct or evaluate Atoms.

## Recurring integration audits

The [binding ownership review](reviews/hepkit-binding-ownership.md) follows the
user's correction: substantive PyO3 bindings belong in FastSecDec's isolated
`bindings/python` crate; community only links/registers them and supplies its
public reexports/stubs. Dedicated notebook helpers, fixtures, tests and build
support move with the implementation. Core and default-CLI metadata remain
Python-free. The new read-only inspection views retain `Arc<GeneratedIntegral>`
and native indices, forwarding existing chart/domain certificates, exact integer
geometry and Symbolica expressions. They add no graph, algebra or numerical
implementation and never restore expanded coefficients for an overview.
Independent review accepts that design and the relocated native release wheel's
61 installed controls, including six native inspection tests. Actual native
triangle and gg→HH notebooks preserve zero sampling through generation/inspection,
then complete explicit integration with native checkpoint-prefix preservation.
Generated stubs preserve all previous members and expose the eight new inspection
classes, with installed-export and Python 3.9 grammar checks passing. The public
dependency setup and hosted native development-wheel CI now pass; the optimized
public-source Wasm wheel passes its generic smoke test and all 50 portable
controls. The optional browser gg→HH entry reuses the same native HEPKit helper
and physical assets. Those observations belong to the earlier `539019a` wheel.
The [current public metadata/MC delivery](reviews/hepkit-metadata-mc-portable.md)
passes native 81 / portable 58 controls and actual triangle native-math inspection,
QMC resume and same-kernel Havana pilot/production semantics. Its final
supplemental screenshot failure and bounded gg→HH interaction limits remain
explicit; no repeated science or new mathematical owner is introduced. See the
[notebook controls audit](reviews/hepkit-notebook-controls-audit.md) for the
explicit Generate/Integrate and lazy-inspection requirements.

The [gg→HH sector sanity check](reviews/gghh-sector-sanity.md) independently
decomposes pySecDec's denominator polynomials and matches all 30 native chart
maps up to target-coordinate permutations, including Jacobians and factor
valuations. It does not repeat the infeasible full numerator reference or
certify Laurent coefficients. The implemented [discrete-sector Havana lane](reviews/havana-discrete-sector-sampling.md)
uses Numerica's existing `DiscreteGrid` and continuous children, with independent
native physics and current portable lifecycle acceptance. It preserves pilot/
production separation, caller-owned scheduling and native complete-vector
covariance; no separate sampler or estimator was introduced.

The [Korobov-2 review](reviews/korobov2-integration.md) accepts the additive
Numerica transform and its thin FastSecDec dispatch. Public API, source/tests
and an external Rust probe established the missing numerical operation;
Numerica owns all map/Jacobian arithmetic. Independent source and final-evidence
reviews accept the weighting, checkpoint identity, caller-owned execution and
unique dependency owners. The existing Korobov-3 default remains unchanged.
The HEPKit notebook bindings have separate accepted native and current portable
integration/reuse reviews. The K2 numerical milestone itself does not certify
the demo or physical performance.

An independent subagent reviews HEPKit integration and ecosystem reuse at each
major subsystem milestone, before new dependency patches are accepted, and at
final acceptance. The review covers native ownership and public interfaces,
status/error types, caller-owned execution, persistence, unnecessary conversions,
and reusable graph, algebra and numerical operations. Each review records fixes,
remaining gaps and the next audit boundary under `reviews/`.

The [2026-10-04 broad audit](reviews/hepkit-integration-audit-2026-10-04.md)
records current findings and their resolution checklist. Its numerical-reference pass identified
the pure-Rust `oneloop` dependency already used by symbolica-community as the
scalar-master provider; FeynKit's tensor routines are a separate reduction API.
The [native master review](reviews/hepkit-one-loop-native.md) records the passed
normalization probe, eleven graph-to-master comparisons and the CLI model-cache
reproducer/fix. The [native numerator reference review](reviews/hepkit-numerator-reduction.md)
records the reduction API/source/probe evidence and complete comparisons.
All-example certification remains separate work.

The [reference API reuse audit](reviews/reference-result-reuse-audit.md) and
[independent review](reviews/reference-result-independent.md) justify the small
typed comparison adapter. Native HEPKit callers can construct references directly;
JSON is only a persistence boundary. The CLI records comparison evidence without
changing sampling, numerical checkpoints or scientific artifact identity. The
[dependency provenance review](reviews/dependency-provenance.md) additionally
closes an untracked-source hashing gap and a Cargo watch issue in local builds.

The [boundary-growth review](reviews/boundary-growth-independent.md) verifies a
thin componentwise diagnostic over the existing sampler. Native Numerica
logarithms handle finite extreme magnitudes without forming overflowing ratios;
typed reports retain actual endpoint distances, missing observations, every
retry and a shared evaluation budget. GammaLoop's existing constant-dropped
multi-point fitter was inspected and has a different contract; no replacement
fitting or sampling engine was added. Eleven new tests and five existing
diagnostic tests pass, with six pure numerical cases independently rerun.

The [sector-contribution review](reviews/sector-contributions-scientific-review.md)
checks new QMC/Havana reporting adapters over native complete-replica statistics.
Democratic sector rows use precisely the common complete shifts used by the
total; independent rows use complete local replicas. Exact offsets remain
separate, pilot observations remain progress only, and marginal covariances are
explicitly distinguished from the authoritative total covariance. Four new
scientific tests and the existing 15 QMC/eight Havana runtime tests pass. Public
formatting rejects malformed imported layouts without indexing absent values.

The [boundary CLI/HEPKit contract audit](reviews/boundary-cli-hepkit-contract-audit.md)
reviews native ownership, streamed typed events, caller cancellation, portable
metadata and presentation. It found a malformed imported attempt-index overflow
in Display; checked presentation and a pure-data regression resolve that issue.
Sampled growth flags remain diagnostic and cannot be mistaken for a proof of
integrability or an evaluation failure.

## Factored direct generation

The [reference implementation review](reviews/direct-generation-performance.md)
records why the first full-expression Laurent expansion was slow. Native
`replace_map` supplies opaque coordinate subexpressions, `series` expands the
small regulator template, and native substitution restores the sector expressions
afterward. This changes operation ordering without implementing a second series
engine. Whole-expression expansion and unbounded rational combination are avoided;
exact support and endpoint-power extraction remain justified coefficient queries.

An additional executable native probe confirms that `to_polynomial_in_vars`
collects factored sums/products/powers directly: its implementation in
`symbolica/src/poly.rs:1547` uses existing sparse polynomial arithmetic without
requiring an expanded Atom first. With signed exponents, native `mul_exp`
(`src/poly/polynomial.rs:2136`) removes a monomial valuation before `flatten`.
The probe passed for two factored polynomials and a Laurent-polynomial shift;
the same cases are retained in `ecosystem_probes.rs`. General rational functions
still require appropriate native simplification or explicit rejection.

The [factored monomial review](reviews/native-monomial-stripping.md) and its
[independent audit](reviews/native-monomial-stripping-independent.md) extend that
reuse: after exact support determines a valuation, native `collect_factors`
can remove its monomial without flattening the residual. Native polynomial
recognition plus a check of coordinate-dependent indeterminates supplies a
sufficient acceptance condition; hidden cancellation retains the previous exact
sparse fallback. No new factoring or valuation engine was introduced.

The [native evaluator function-map audit](reviews/native-function-map-audit.md)
distinguishes aliases, which always inline, from registered functions with native
`Always`/`Never` policies. This is evidence for a controlled representation
experiment, not a measured production improvement. The experiment must retain
complete Laurent vectors, weighted precision replay, worker cloning and native
persistence, and control translation mode when comparing inlining policies.
The [release verification](reviews/evaluator-release-verification.md) records
Symbolica 3.0.1 and SymJIT 2.26.4 and the small development-only OneLOop cache
compatibility patch.

The [compatibility probe](reviews/function-map-compatibility.md) passes 32 native
FunctionMap combinations, including complex coefficients, weighted MPFR replay,
worker clones and fresh-process reconstruction. The [rank-five experiment](reviews/function-map-rank-five.md)
adds an explicit controlled numerical pullback to actual generated coefficients;
119 longer measured cases preserve the complete vectors. Aliases save preparation
and builder work in that experiment, while retaining tiny function calls is
slower. Existing phase-one production representation and artifacts remain
unchanged because earlier symbolic work and general runtime gains are separate
questions.

The supplied Symbolica build could not convert fixed-argument external constants
to its error-tracking domain when that function only registered a multiprecision
hook. The [minimal dependency patch](dependency-patches/symbolica-fixed-argument-constant-domain.md)
reuses that registered hook and native conversion, with a passing focused
regression. FastSecDec does not implement special-function constants itself.

## Multiloop reference transport and lattice quality

The [independent multiloop review](reviews/massive-multiloop-reference-independent.md)
checks six externally generated C++ reference results against the native graph
attachments, kinematics and normalization. The frozen files in
`examples/references` use the existing `ReferenceResult`, `read_reference`,
`encode_reference` and `compare` APIs. Their positive reported errors and
explicit real-only projection are preserved; unknown actual work counts remain
unknown. No external engine enters the normal build or test dependency graph.
These initial comparisons do not certify convergence.

The [six-line investigation](reviews/six-line-qmc-convergence.md) uses native
Numerica plans, worker-local point generation, periodization and complete-shift
statistics throughout. A constant-integrand control reproduces a plateau of the
current Kuo33002 rule with Korobov3. Published alternative vectors improve the
actual five-dimensional integral substantially, but broader dimension controls
show that the best vector for that fixture is not universally best. Explicit
catalogue choices and cross-case validation are under review; no new estimator,
adaptive vector search or CBC generator is justified by this evidence.

The [mapping attribution](reviews/rank-five-mapping-attribution.md) measures
repeated native original-support extraction as 99.63% of rank-five mapping time.
The implemented generation-owned lazy cache reuses existing `PolynomialSupport`
results with native Atom keys and fixed parameter ordering. It preserves the
coordinate-face fast path before support extraction and all exact validation;
mapped densities and occurrence exponents remain outside the cache. The
[independent review](reviews/native-cache-and-catalogue-independent.md) found no
new algebra or native ownership issue. Sixty-one focused scientific tests pass;
three alternating release pairs measure generation at 11.371 seconds uncached
versus 2.522 seconds cached. Parameterization and separate diagnostic validation
are outside those timers, and matched Pathfinder acceptance remains separate.

The [catalogue proposal](reviews/qmc-catalogue-proposal.md) keeps published data,
bounds and provenance validation in Numerica. The
[cross-case campaign](reviews/qmc-catalogue-cross-case.md) reuses native covariance
for 48 runs over four physical cases, four rules and three seeds. HKKN improves
the uncertainty over the current rule in each tested dimension, while individual
cases favor other rules. Explicit choices preserve old Kuo semantics; any default
change requires a separate decision. The recurring review also checks binary
data provenance, library-owned status and caller-owned execution. Remaining
example acceptance is tracked in the
[scientific campaign schedule](reviews/remaining-scientific-campaigns.md).

The [CLI catalogue audit](reviews/cli-catalogue-independent.md) verifies thin
native selection and refinement, legacy Kuo settings identity and reusable
`QmcDesign` reporting. Review found and closed an outer-checkpoint versus native
session consistency gap: resume now binds settings, method and refinement round
before requesting work. The final design retains actual allocations without
dumping generators into every streamed update. Twenty-three focused CLI tests
pass; the subsequent combined workspace gate passes 231 tests with 12 explicit
probes ignored. The exact shipped massive-box point also passes against the
existing native D0 provider, bringing the scalar reference campaign to twelve
physical points without another master implementation.

## Saved numerical results and coupled numerator

The [saved-result HEPKit review](reviews/saved-result-hepkit-independent.md)
checks the numerical-only `results` module and its public typed scope, manifest,
validation, reference selection and display contracts. It reuses native
contribution reports, complete covariance, reference objects, comparison logic
and status types. No Atom, graph, kernel reconstruction or new estimator is
required to read a result. A selected allocation remains explicitly distinct
from a full integral, and exporting an estimate does not promote its evidence.
The CLI remains a caller-side adapter for persistence, viewing and explicit
reference extraction.

The [coordinator review](reviews/saved-result-coordinator-review.md) verifies the
minimal Numerica accepted-shift coverage accessor and requires numeric-range
failures to remain distinct from structural errors. Failure-safe observation
retains representable totals and marginals independently, including shared-shift
cancellation with overflowing marginal covariance. Presentation uses borrowed
rows and indexed ordering rather than copying covariance or repeatedly scanning
sector IDs. Metadata validation reuses native rule checks without generating
all random shifts. Numerica's local `e4638da22a17cfa931fa14c6829d3350b7a8de2b`
passes 30 QMC and nine existing MC tests. After the user's publication
authorization, its full default, serde and alternative-backend suites also
passed; it is published as
[Numerica PR #8](https://github.com/symbolica-dev/numerica/pull/8).
The [minimum-version follow-up](reviews/numerica-qmc-upstream-readiness.md)
also accepts exact Rust/Cargo 1.89.0 on Linux at that unchanged head: 237
default, 240 serde and 217 alternative-backend tests pass, with no failures or
ignored tests. All 73 source and two manifest/lockfile hashes remain intact.
This closes the Linux minimum-version test gap without changing native owners;
macOS, other architectures and performance remain separate checks.

The [coupled-sunset campaign](reviews/coupled-sunset-numerator.md) and its
[independent review](reviews/coupled-sunset-independent.md) close the mixed-loop
and external-momentum numerator case using a native HEPKit graph. The scalar
control uses `FeynmanDiagram::with_numerator`; density extraction, Gamma
functions, substitution and Laurent series use existing owners. Nine frozen
external parameterization points and integrated analytic identities at two
spacelike scales agree. The separate convergent scalar control checks the
measure sign before continuation. This adds no multiloop reducer or master
implementation; it provides a control for the remaining triple-box numerator.

The next audit boundary covers qualified CLI sector selection and inspection
of retained native generation metadata, followed by difficult-case numerical
calibration and matched performance. Those pending gates are not established
by result transport or the sunset control.

The combined gate passes 253 workspace tests with 12 explicit probes ignored.
A final enum-layout adjustment is covered by eleven native result and five CLI
result/reference tests; formatting and all-target Clippy pass. The production
tree retains a single Symbolica 3.0.1/SymJIT 2.26.4/Numerica/Linnet ownership
chain and excludes Python, pySecDec, CLI presentation and development-only
reference providers. Evidence is in `output/saved-sunset-workspace-tests.log`,
`output/saved-result-boxed-{native,cli}-tests.log`,
`output/saved-sunset-clippy.log` and
`output/saved-sunset-production-dependencies.log`.

## Qualified selection and retained metadata inspection

The [independent selection review](reviews/selected-sector-independent.md)
finds no blocking issue after 31 CLI and 17 native tests pass, including an
independent repeat of the two native projection tests. Scope projection belongs
to the native `KernelResultManifest`; saved-result validation and CLI execution
reuse that one implementation. Original sector IDs, complete coefficient layouts,
shared-shift statistics and explicit exact-offset policies are preserved.
Completing an explicitly selected allocation does not become a full-integral
claim, even when all IDs were selected. Checkpoints bind that scope declaration.
The CLI constructs selected worker contexts, while artifact loading still
validates and compiles the complete kernel set.

Inspection exports the existing owner's `PortableMetadata` and borrows its
native domain/chart/map records for display. It adds no graph reconstruction,
support extraction, map serializer or symbolic conversion. Legacy absence is
explicit. The [author's evidence](reviews/selected-sector-implementation.md)
records nonempty QMC/MC/adaptive-MC execution, partial resume, empty/exact-only
selection, full-scope overrides, metadata equivalence and missing-DOT handling.
The next interface audit covers the native `IntegralFamily` parameterization
entry point; scientific calibration and matched performance remain open.

## Native family entry and repeated-propagator reuse

The [family-entry evidence](reviews/native-family-entry.md) and
[independent review](reviews/native-family-entry-independent.md) expose the
existing Gaussian implementation to borrowed HEPKit `IntegralFamily` objects.
Graph entry contracts its native numerator and applies its measure weight once,
then delegates. Native `partial_fraction` owns affine/repeated-denominator
reduction; native `sector` owns positive-power projection, with the native
one-loop reducer providing an existing composition example. Public API, source
and tests, and executable exact reconstructions all support this reuse. No
automatic graph reduction or alternate family representation is introduced.

Four focused family tests verify an independently integrated raised-power
Gaussian moment, literal parameter symbols, exactly-once graph weights, strict
positive powers and collision admission. An independent review identified
dimension-dependent physical coefficients that could retain a stale tensor
dimension in U/F; a typed rejection now requires explicit caller specialization
when the requested dimension changes. Native label and common parameter
validators are reused. The focused gate passed 49 tests, including existing
native input and Gaussian controls.

The [off-shell triple-box audit](reviews/triple-box-offshell-diagnostics.md)
establishes two repeated propagator pairs in the physically equivalent native
fixture. Existing HEPKit APIs reduce ten original edges to eight active
propagators with two squared powers, with exact canonical rational-product
equality. Both unreduced scalar and numerator baselines complete their entire
four-coefficient allocations without evaluation failures. Those estimates are
preliminary; ordinary undotted-ladder formulas are not applicable references.
The next performance audit measures projected-family generation and CLI
snapshot/checkpoint costs, preserving the original baselines and full coverage.

The combined acceptance gate passed **264 workspace tests**, with twelve
explicit probes ignored. A final test-only legacy-inspection byte-slice cleanup
passed its focused regression; formatting and all-target Clippy then passed.
The production dependency tree still has one native backend ownership chain
and excludes Python, pySecDec, CLI rendering and development reference providers.
Evidence is in `output/selection-family-workspace-tests.log`,
`output/selection-family-legacy-inspect-tests.log`,
`output/selection-family-{fmt,clippy}.log` and
`output/selection-family-production-dependencies.log`.

## Caller-owned observation cadence

The [status attribution](reviews/cli-status-performance-attribution.md) restores
actual native QMC checkpoints without loading a graph or evaluator. It measures
native estimate/snapshot, serialization and checkpoint costs separately, using
the existing owners. Complete native snapshots cost about 2.2 ms; cached JSON
serialization costs about 0.2 ms. These endpoint measurements do not establish
a full-run speedup.

The resulting CLI-only policy requests snapshots when presentation is due,
keeping native estimators and checkpoint formats unchanged. Its interval is
outside sampling settings and mathematical identities. Stage and final events,
cancellation and failures bypass the display deadline; every batch still accepts
packages, handles worker/submission errors, merges replay and checks cancellation.
Statistics-only range failures can be detected at the next observation or
unconditional stage/final reduction. The
[independent review](reviews/cli-status-cadence-independent.md) documents this
explicit latency tradeoff and finds no blocker.

All 33 focused CLI tests pass, including exact estimate/covariance/design/replay
equality across different cadences, a real accepted-partial Ctrl-C/resume, scope
qualification and both numerical failure paths with final saved evidence and
nonzero exits. Formatting and all-target Clippy pass. Future HEPKit callers keep
control over when they request the same native status; no library timer or worker
pool was introduced. The subsequent three-pair full-artifact campaign preserves
identical complete numerical state across all six runs, with median integration
time reduced by 20.0%; the attribution report retains the timer and I/O limits.

## Bounded orthant and projected-family execution

The [projected-family campaign](reviews/native-family-projection-diagnostics.md)
uses native partial fractions, sector projection and the reviewed public
`from_family` entry. Both original ten-entry and projected eight-propagator
families preserve the native weighted numerator and exact rational density.
Both projected scalar/rank-two allocations complete with full coefficient
vectors and covariance. An [independent source review](reviews/native-family-projection-independent.md)
finds no duplicated parser, graph reduction, estimator or worker ownership.
These experiments do not automatically alter graph entry; multi-term and signed
power policies would require separate design and review.

The [issue-1 campaign](reviews/onshell-triple-box-and-issue-one.md) and
[hard four-loop campaign](reviews/hard-four-loop-diagnostics.md) complete their
full positive-orthant densities using the existing production pipeline. The
coordinator read issue-1 through the numerical-only saved-result API and checked
its complete vector, native covariance, coverage and diagnostics. The
[independent hard-case audit](reviews/hard-four-loop-independent.md) checks
input/build identities, exact accepted package coverage, common shift plans,
native saved-result validation and consistent estimates across reports. No
independent estimator or algebra implementation was needed. Both results remain
unverified and unconverged; historical target uncertainty is not promoted to a
scientific certificate.

The on-shell triple-box generation timeout is retained as an open scientific
gap. A test-only capture/replay diagnostic now measures the existing native
series API and expression representation. Its pending acceptance must not be
confused with a production Laurent change. The next broader audits cover that
evidence, actual current-artifact convergence preparation and paired CLI cadence
timings before any numerical-policy or performance claim.

## Native Laurent capture and depth attribution

The [native-depth experiment](reviews/native-laurent-depth-probe.md) uses
Symbolica's own Atom export/import, series, trailing exponent, absolute bound,
coefficient iteration and replacement APIs. The three API/source/executable
checks found no public reusable cross-call series cache or standalone general
transcendental valuation API. Relative-depth requests therefore query the native
series result and validate its actual absolute bound; no Laurent engine, pole
inference or coefficient convolution is reimplemented.

The [independent review](reviews/laurent-capture-independent.md) drove explicit
fractional-power rejection and a missing-target capture barrier. Test-only
capture always returns cancellation, including out-of-range targets, and its
thread-local guard restores ordinary generation. Three ordinary controls pass.
The bounded actual representative and both replays complete, with exact native
equality of all six coefficients both before and after coordinate restoration.
Formatting and all-target Clippy pass against the final diagnostic sources.

Cold import emitted a Gamma callback warning. Native source inspection and
fresh-process normalization/derivative/series controls prove it is a lazy native
initialization false positive for this capture: cold and explicitly initialized
canonical expressions are identical. Future diagnostic imports initialize native
functions explicitly; the production parser already initializes them, and its
existing fresh-process shifted-Gamma test covers subsequent evaluation. No
dependency patch or alternate Gamma implementation is justified.

The measured series reduction from 148.974 to 37.332 seconds preserves the same
large output. A single-representative result does not establish whole-generation
performance. The reference's active direct builder computes regular epsilon
coefficients before coordinate differentiation and substitutes them into a small
endpoint expression. A future production composition must use native Symbolica
series/derivative operations, preserve complete vectors and pass independent
scientific controls; the current diagnostic changes no production strategy.

## Conservative native-family preparation and completed holdout

The [prepared-family library seam](reviews/native-family-preparation.md) borrows
the original native family or owns FeynKit's exact projected sector. Native
`is_independent`, bounded `partial_fraction`, `sector`, Symanzik construction
and Atom canonical equality own the algebra. Admission accepts only one
unit-coefficient term with fewer positive denominators, unchanged momentum bases
and an exact denominator-product identity. Other valid decompositions retain
the original family; invalid native inputs still fail. Original labels are
validated before projection, and graph/numerator/measure weights are applied
once. No graph contraction, parser, routing engine or alternative CAS is added.

The [independent review](reviews/native-family-preparation-independent.md)
finds no ownership or reuse blocker. Six new scientific/admission tests and
eighteen existing family/input regressions pass. The CLI and portable metadata
are a separately reviewed follow-up; default adoption still requires no-op cost
and provenance checks. Existing explicit original-family entry points remain.

The [fixed massive holdout](reviews/massive-holdout-stage-a.md) completed all
72 allocations and 223,838,208 evaluations without failure. The
[independent audit](reviews/convergence-stage-a-independent.md) verifies
218,592 canonical packages, complete sectors and sixteen shifts per row, and
the exact 48 aligned vectors in each of twelve native joint summaries. Native
`QmcEstimate` owns every estimate/covariance; no local estimator was introduced.
The observed lattice tradeoff depends on point count and does not authorize a
default change or establish calibrated statistical coverage.

The [native Series-first prototype](reviews/native-series-subtraction-proposal.md)
uses Symbolica's coefficient mapping, differentiation, substitution, series
arithmetic and actual remainder bounds. Five exact control groups and three
capture/depth controls pass. The actual on-shell representative completes but
is slower and uses more memory than the earlier ordering; its full-vector
cross-comparison is still pending. The independent review retains this negative
result and the explicit unregulated-endpoint fallback. No production strategy
changes, custom coefficient convolution or derivative engine follow from it.

The [first paired command campaign](reviews/first-paired-performance.md) completes
all 28 triangle/box rows at matching physical sample counts and below the fixed
reference-investigation threshold. Each program owns its numerical work and
statistics. Different backend/precision/persistence boundaries remain explicit;
these observations do not close strict matched-performance acceptance. Following
the user's latest sequencing, broad generation/convergence/per-sector latency
optimization follows complete capability coverage. Work on the on-shell
generation bottleneck continues because it currently prevents that coverage.

The combined milestone gate passes **280 workspace tests**, with seventeen
explicit probes ignored, followed by formatting and all-target Clippy. Two
needless-reference comparison warnings introduced by the validation refactor
were corrected before the final gate. The production tree still has one
Symbolica 3.0.1, SymJIT 2.26.4, Numerica and Linnet owner and excludes Python,
pySecDec, CLI rendering and reference providers. Logs are
`output/family-series-{workspace-tests,fmt,clippy,production-dependencies}.log`.
The subsequent [formal-function audit](reviews/native-formal-functions-audit.md)
rechecks current releases and confirms native exact evaluator persistence and
retained function-body ownership; its new disconnected executable proof remains
separate from this accepted production API.

The first two formal-function proof attempts exposed a separate native defect:
the generic function-series fallback interpreted an underscore-suffixed
expansion symbol as a wildcard. A minimal native Gamma call returned no
coefficients for `eps_`, while the same call with `eps` returned its pole and
finite coefficient. Empty-vector comparisons from those attempts are rejected;
they establish no FunctionMap or complex-constant compatibility. This is
distinct from the earlier cold-import Gamma initialization diagnostic.

The [minimal fourth Symbolica patch](dependency-patches/symbolica-literal-series-variable.md)
uses `Pattern::Literal` for the two existing native substitutions. API/source
inspection, the failing native executable, an independent source review, and
passing native and FastSecDec regressions support the correction. Native Gamma,
Series arithmetic, derivatives, truncation and evaluator ownership remain
unchanged. The FastSecDec regression checks the complete nonempty vector through
order one for `Gamma(eps_)/eps_`, including both poles. The corrected
formal-function experiment must still pass its own nonempty controls before
any production representation change. Frozen prior benchmark binaries keep
their original dependency identities.

The subsequent combined gate passes **281 workspace tests**, with seventeen
explicit probes ignored; formatting and all-target Clippy pass. The production
dependency tree retains the same single native Symbolica 3.0.1/SymJIT 2.26.4,
Numerica and Linnet ownership and excludes Python, pySecDec, CLI rendering and
development-only reference providers. Evidence is retained in
`output/literal-series-{workspace-tests,fmt,clippy,production-dependencies}.log`.

The corrected [small formal-function proof](reviews/native-formal-function-proof.md)
now passes three complete-vector cases under both Always and Never inlining,
including exact restored identities, mixed native partials, admitted face
substitution, hidden complex polynomial constants, O2/error-tracking/weighted
MPFR evaluation and fresh-process native-IR reload. Native Symbolica owns the
derivatives, series, evaluator and serialization throughout. This supports the
bounded actual-representative experiment, not a production strategy switch or
an end-to-end performance claim.

The [standalone author-facing MRE](../mre/symbolica-literal-series-variable/README.md)
reproduces the defect against the actual unpatched published 3.0.1 crate in an
isolated Rust-script package. Its unchanged Rust source passes when explicitly
linked to the corrected local library. The folder includes the minimal patch,
observed before/after output, source/version hashes and reproducible instructions;
the patch also applies cleanly to the published source. No FastSecDec dependency
or maintainer contact is needed to run the reproduction.

The independently generated double-box reference now retains all five orders
and their reported uncertainties in `examples/references/double_box.json`.
The native recorder uses the existing `ReferenceResult` transport and `compare`
API against the original complete 64-shift estimate and its unchanged covariance.
Independent review verifies physical tuple selection, exactly-once Gamma
normalization, source/package identities and explicit imaginary projection.
Every coefficient agrees within 1.27 combined standard errors. Unknown external
work/covariance remain unknown, and the checked label does not certify error
calibration. Both fixture transport tests pass through Cargo; formatting and
all-target Clippy pass. Production code remains at the 281-test combined gate.

The [independent point-first oracle](reviews/point-first-laurent-oracle.md)
binds exact coordinates into the original captured expression, then uses native
Symbolica Series and multiprecision evaluation. All six coefficients now pass at
three prescribed points. Native relative-depth requests use actual remainder
bounds, with exact agreement to the absolute-series control at the interior
point; both earlier absolute-depth timeouts are preserved. No new series
arithmetic, interpolation or uncertainty estimator was introduced. This supplies
an independent check for the compact-generation experiment, not a full-domain
identity or an integral convergence result.

The first [actual polynomial-function prototype](reviews/native-formal-actual-prototype.md)
is a retained negative result: preparation preserves exact original subtraction
and conservative cancellation metadata, but delayed native zero pruning produces
larger intermediates and the subsequent series hits its 180-second bound.
No coefficient/JIT comparison or production switch follows. Native public
symbol derivative callbacks can potentially preserve that pruning while
Symbolica still owns the chain rule. Their subsequent
[independently audited small proof](reviews/native-formal-callback-independent.md)
passes four nonempty complete vectors, exact restored and analytic controls,
native O2/MPFR/worker checks, and a cold evaluator-IR reader without callbacks.
An explicit above-degree derivative exercises the exact-zero callback, rather
than allowing a constant simplification to make that control vacuous. Native
polynomial differentiation and simultaneous substitution still own every
zero/constant decision; no custom chain rule or series arithmetic was added.

This proof does not yet establish a viable production representation. Symbolica
retains registered callbacks globally, so strong polynomial/cache captures can
outlive a generation call in a HEPKit process. Before production adoption,
caller-owned cache lifetime and resolution of all escaping callback expressions
must be demonstrated without resetting global Symbolica state. The actual-sector
trial remains a disconnected bounded experiment: preparation falls to 6.617
seconds, but the combined 180-second process times out in native series without
a complete vector. That trial omitted the existing late epsilon-template
substitution; retaining the native template and evaluator aliases is the next
candidate for source review, not a measured improvement. A separate
[coefficient API audit](reviews/native-series-coefficient-api.md) identifies
existing custom `AtomField` normalization and native alias facilities, with
explicit cancellation and peak-memory limitations. No replacement CAS or
dependency feature patch was added.

The subsequent [native template/alias audit](reviews/native-template-alias-independent.md)
passes nine small controls, all six representative coefficients at three
independent exact points, cold native-IR reload, and fresh native Laurent
generation. `EvaluatorBuilder::add_aliases` registers the existing 201 images
once and lowers them into native common-subexpression instructions; there is no
replacement symbolic registry, derivative callback, series arithmetic or
process-global cache. Native relative-series remainder bounds govern extraction.
The 42.539-second fresh Laurent substage and separate 0.123-second cached builder
do not establish full-graph performance. Native high precision supplies the
accepted numerical agreement; ordinary precision is unstable at all three
points. Production metadata, persistence, lifetime and adaptive precision rescue
are the next independent integration gates.

The [Issue 1 reference audit](reviews/remaining-reference-independent.md)
uses the existing versioned reference encoder and comparison API, preserving
the complete physical vector, provider errors and original native covariance.
Its external disteval implementation shares shifts between kernels but sums
marginal variances without their covariance. The fixture therefore remains
`ReferenceValidation::Unverified`, and a regression verifies comparison
ineligibility despite complete source/normalization/transport checks. No new
uncertainty schema or estimator is needed. The existing provider's ordinary
constituent `IntegralLibrary` with `together=True` sums sector integrands before
native integration and is being prepared as the supported alternative. The
subsequent [randomization-route audit](reviews/reference-randomization-route-audit.md)
confirms that the six massive DOT fixtures and double box already use ordinary
IntegralLibrary and a shared advancing native RNG. Issue 1's per-kernel reset
finding therefore does not justify downgrading those seven fixtures; their
existing calibration limitations remain separate.

The [individual-sample latency diagnostic](reviews/native-sample-latency-results.md)
uses native `QmcSession`, caller-owned workers and `WeightedEvaluationContext`.
The ignored Rust caller only records clock brackets and provenance; native
point generation, transforms, accumulation, covariance, replay and checkpoint
restoration are unchanged. Exact full-vector on/off equality and a rejected
seven-sample prefix/retry control pass. Its frozen release dependency identities
remain distinct from the ongoing alias implementation.

The [ordinary-constituent Issue 1 attempt](reviews/issue-one-together-outcome.md)
now completes all three coefficients via the existing sector-sum API, with
provider uncertainty computed after summing the integrands. Independent source,
metadata, normalization, original tuple and hash checks pass. This addresses the
identified omitted sector covariance without a new estimator; cross-coefficient
covariance and general error calibration remain unverified. The original
disteval fixture is preserved. The separate `issue_1_together.json` fixture
passes independent transport review using the existing native reference encoder
and comparison API, retaining the complete native vector/covariance and all
three provider errors. Its Checked label admits comparison under the audited
sector-sum path without asserting calibration; every pull is below 0.857.

The [production alias integration](reviews/native-template-alias-production-independent.md)
now passes the complete 294-test workspace gate, final affected tests, formatting,
all-target Clippy and the normal-dependency audit. Native `AliasedAtom` owns
symbolic roots/images, and its explicit lazy restoration preserves the existing
public Atom accessor. A shared native exact evaluator owns O2, conditioning and
MPFR; worker clones share immutable bytes. Version-three persistence uses native
serialization with an exact revision/codec contract, preserving legacy IDs and
bytes. The essential fifth native Symbolica fix validates decoded IR in its
owner; FastSecDec adds no instruction interpreter or validator. All six original
malformed layouts are rejected by all three native decoders, and public artifact
tests exercise malformed IR and complex fixed constants under real-layout tampering.

The source-bound difficult representative passes exact native pre-series
template/image identity, all eighteen independent original-expression oracle
comparisons and seventy-two fresh/decoded weighted component checks. Native
renaming alone does not establish a shared factored coefficient structure; both
failed structural comparisons and every final structural flag remain recorded.
No large expansion or custom equivalence algorithm was introduced to force that
representation. Full-graph coverage and cold-load performance remain separate
gates. The [remaining-gates audit](reviews/phase-one-remaining-gates.md) identifies
the planned geometry cache/parallel execution, current parent-qualified sector
IDs, affine endpoint admission limitation and final CLI color/terminal evidence.

The [geometry-cache audit](reviews/geometry-cache-independent.md) accepts a
bounded caller-owned wrapper around the existing sector decomposition. Exact
native support/domain equality and resource limits govern reuse; existing
Numerica integers, map generation and decomposition remain the sole owners of
geometry. There is no alternative canonicalizer, CAS, scheduler or persistence
format. Completed `Arc` results preserve caller lifetime and explicit reuse;
cancellation and failed misses cannot corrupt prior entries. All 28 sector
tests and sector Clippy pass, with six new tests independently rerun. This
establishes the cache core only; generation-level domain reassessment, typed
progress integration and parallel work interfaces retain their own next audit.

The [generation-context audit](reviews/generation-context-independent.md)
accepts the additive caller-owned entry with fresh domain admission on every
request. Only exact integer geometry is shared; source coefficients, weights,
regular factors, coordinate symbols, assertions and mapped residual checks
remain current. Compact native roots, aliases, chart metadata, kernel identity
and independent complete-vector controls agree on cold/warm/free paths. The
combined 305-test workspace gate, formatting and Clippy pass. The
[terminal review](reviews/terminal-policy-independent.md) also accepts shared
CLI color decisions and compact rendering through the existing public status
Displays, with real PTY cleanup/resize/cancellation evidence. No presentation
dependency or executor enters the library. Parallel chart/cone interfaces and
additive sector content IDs remain separate work.

The [off-shell scalar reference audit](reviews/projected-triple-reference-independent.md)
now accepts all four physical coefficients through the existing native
reference encoder/reader and comparison APIs. Its source is an ordinary
external sector-sum integration with the native Gamma factor applied once;
all eight provider value/error bit patterns survive transport. Existing
`SavedResult`/QMC reduction preserves each original/projected native covariance
and supplies separate comparisons. No new uncertainty estimator, polynomial
projection, prefactor convolution or result schema was introduced. The five
focused reference tests pass; calibration and the highest-order accuracy gate
remain explicitly unverified.

The [sector-identity audit](reviews/sector-content-identity-independent.md)
accepts an additive digest over native immutable evaluator bytes and the
existing artifact/metadata transport. BLAKE3 owns hashing; Symbolica retains
instruction serialization and canonical expression transport. The accessor
introduces no new algebra, decoder, graph identity or persistence schema.
Four focused tests, formatting and library Clippy pass. Parent artifact,
checkpoint/result and replay identity contracts remain unchanged; no
machine-code cache or mathematical-equivalence claim follows from the digest.

The [parallel geometry audit](reviews/parallel-geometry-api-independent.md)
accepts a two-stage native work boundary with caller-owned threads. Both serial
and scheduled paths use the same extracted native support, cone and map helpers;
Numerica owns exact arithmetic. Private provenance and canonical complete merge
reject missing/foreign work and preserve native error/limit precedence.
The 38-test sector gate and independent ten-test rerun pass. Cache/context
adoption is a separate next slice; the core creates no library executor.

The [named-coefficient audit](reviews/native-regular-coefficient-independent.md)
accepts the bounded small test-only composition gate: eight scientific controls
pass while Symbolica retains Series/remainder arithmetic, differentiation,
literal face substitution and aliases. Local names/cache records introduce no
alternative CAS or global callback registry. Numerical evaluator, weighted
precision, cold-reload and actual-large-input gates are still pending. The
combined workspace includes these controls and passes 323 tests. All eight
affected controls pass again after a type-alias-only lint correction; formatting
and all-target Clippy pass, with the earlier lint failure retained.

The [cache/context dispatch review](reviews/parallel-generation-dispatch-independent.md)
accepts an additive orchestration seam over the same native two-stage work and
exact cache. The caller supplies workers; opaque completions retain native
provenance, admission and canonical merge. Native domain checks precede lookup
and all per-integral symbolic operations remain fresh. The focused 41 sector
and eight context tests pass; no library executor, new geometry, symbolic cache
or progress-counter interpretation is introduced. CLI adoption has a separate
review boundary.

The named-coefficient program controls now also pass native evaluator creation,
weighted precision rescue, worker cloning, exact-IR encoding and cold decoding.
Across both small Taylor/IBP densities, 96 complete-vector calls and 768 component
checks agree with independent native high-precision unchanged-subtraction
references. Native Series, differentiation, aliases and evaluator IR retain
ownership; no registry is needed by the cold reader. This closes only the small
program gate, leaving the actual large input and production adoption open.

The [rank-two reference audit](reviews/projected-rank-two-reference-independent.md)
accepts all four physical coefficients through the existing native reference
and comparison APIs. The original tensor numerator, raised powers and native
Gamma factor are retained. Exact provider bits survive encoding, both original
and projected native covariance matrices remain unchanged, and separate maximum
comparison pulls are below 1.727. Six focused reference tests pass. No new
projection, prefactor algebra, estimator or transport schema was added; the
finite reference uncertainty remains 1.67%, with calibration and 1‰ convergence
unverified.

The combined cache/context dispatch, named-program controls and rank-two
transport milestone passes **330 workspace tests**, with 22 explicit probes
ignored and zero failures. Both newly ignored program probes were separately
executed and independently reviewed. Formatting and all-target Clippy pass.
Evidence is retained in `output/dispatch-named-rank2-workspace-{tests,fmt,clippy}.log`.

The [CLI dispatch review](reviews/cli-geometry-dispatch-independent.md) accepts
the application-owned Rayon adapter over native opaque geometry jobs. Existing
standard channels carry tagged observations and complete native results;
`in_place_scope` keeps terminal ownership on the caller and joins workers.
No pool enters the library or Numerica. The coordinator unwind guard drops the
receiver before joining blocked senders, and terminal keys latch the existing
shared cancellation flag. All 49 CLI tests pass, including exact serial/parallel
artifact identity and an independent complete-vector Mellin control. Formatting,
CLI all-target Clippy and [real PTY checks](reviews/cli-geometry-dispatch-terminal.md)
pass. Worker settings remain outside mathematical and checkpoint identity.

The first actual named-coefficient generation remains unaccepted after its
180.116-second timeout. It returns no coefficients or final native bound;
independent oracle and program stages do not run. The successful small controls
do not establish the larger input. The next diagnostic instruments existing
native operation boundaries only, preserving the unchanged Series, derivative,
face and alias owners; no replacement algebra or production strategy follows
from the timeout.

The [passive attribution audit](reviews/native-named-phase-attribution-independent.md)
now closes that diagnostic: unchanged native composition reaches remainder one
at 19.57 seconds, while later derivative-request lowering creates a 367 MB
partial whose face substitution takes 62.34 seconds. The traced run still
times out at 180.11 seconds without complete roots or an evaluator. All 103
frozen hashes pass. These instrumented operation durations identify a target;
they do not establish an optimized runtime or justify changing Series algebra.

The [interleaved-face review](reviews/native-interleaved-face-independent.md)
accepts the small test-only resolver and program controls. Symbolica still owns
derivatives, literal substitutions, Laurent bounds, aliases, exact IR, O2 and
MPFR. The candidate changes only their schedule for own-coordinate/exact-zero/
exact-one requests on regular coefficient bodies admitting those faces. It
does not define singular limits; composed and other arguments retain the
original route. Face-dependent bodies enter only the full-request cache, never
the unsubstituted derivative cache. Ten exact controls pass, followed by 96
complete-vector weighted calls and 768 component checks across native fresh,
cloned, decoded and cold evaluators. All source and writer-input hashes pass.
Original resolution remains the default and no production route changes. The
single captured representative now completes bounded generation in 54.88
seconds with all seven orders from −6 through zero and 2,523 native alias bodies.
All 101 input hashes pass independent postflight. This closes only that
generation diagnostic. Its separate existing native evaluator/IR/backend path
also completes with all seven orders and 14,579,188 encoded bytes; 5,202 frozen
checks pass independently. Original-expression full-order oracle agreement and
cold numerical comparisons remain open, as does whole-graph completion.
The first original-expression point-zero oracle reached native leading order
−6 but timed out during its width-seven call at 180.17 seconds, without a
complete vector. Its inputs and failure are retained. A reviewed fresh retry
changes only the process deadline to 600 seconds; the scientific source,
binary, inputs, native Series operations and precision checks are unchanged.
That retry also times out, after 600.32 seconds in the second native width-seven
call, with all 33 input hashes intact and no full vector. Later points and cold
readers remain held. A native dual/factor alternative is source research and
disconnected small controls only, not an accepted replacement oracle.
The [independent dual review](reviews/native-dual-original-oracle-independent.md)
subsequently accepts six such controls with 40 full signed coefficients,
independent native Series/dual agreement at 512/1024 bits, known polynomial
normalization, tiny nonzero and complex guards, nine typed rejections and
literal zero. All 19 frozen hashes pass. Symbolica owns factor collection,
dual Taylor components, Gamma derivatives and MPFR evaluation; the caller owns
the original-source pole bound and strict analytic input admission. An actual
adapter must expose that coverage basis distinctly, never manufacture a native
Series remainder field to satisfy an older reader. Actual-input certification
and production adoption remain open.
The shared helper extraction also passes the same six-case/40-row control:
all semantic result fields equal the original after excluding timings, with
all 21 frozen hashes intact. The actual adapter remains disconnected and uses
this same helper. Native numeric exports distinguish stored MPFR precision
from canonical numeric zero, which supplies no symbolic-zero certificate.
The first actual point-zero adapter reaches seven precision-stable rows but
fails before complete export because its metadata asserted requested precision
equalled native result precision. Numerica deliberately tracks precision loss
through arithmetic; the observed first pair is 505/511 bits after a 512-bit
evaluation request. The failure is retained, no oracle is accepted, and the
narrow transport correction must preserve actual component precision rather
than reset it or change native arithmetic/comparison tolerances.
The corrected writer and reader now pass separate native arithmetic controls
for precision loss, precision growth, numeric zero, exact native decimal/Atom
round trips and rejected metadata substitutions. Existing native APIs supply
round-trip text; no formatter or precision-restoration arithmetic is added.
The corrected original point-zero adapter subsequently completes all seven
orders and fourteen native exports in 28.32 seconds, with all source/proof and
32+78 frozen checks intact. Actual precision is retained without padding.
The other two prescribed exact points subsequently pass with the same frozen
build in 32.79 and 31.65 seconds. Each preserves all seven signed orders,
fourteen native exports, actual component precision and the same source/proof
and 32+78 immutable checks. All three original-input numerical oracles are now
independently accepted. The native analytic Taylor coverage is not relabeled
as a Series remainder or symbolic-zero certificate.
The separately frozen three-process cold reader then passes all 21 signed-order
comparisons and 24 weighted vectors (168 real components), including worker
clones and forced replay. All 5,417 pre/postflight hashes pass. Every weighted
vector is checked and rescued, at 256 bits for points zero/one and 384 for point
two. This closes numerical agreement for the captured representative only,
without multiplicity; public generation adoption and whole-graph completion
remain open. Native evaluator/MPFR/alias/codec ownership is unchanged.
The [native symbol-hygiene audit](reviews/native-symbol-hygiene-independent.md)
also closes a caller-state admission gap before production promotion. Explicit
empty native metadata plus `Symbol::is_exportable()` rejects all 17 tested
foreign attribute/metadata/hook registrations without invoking their callbacks;
empty attributes alone are insufficient for hooks. Two input-owned names are
skipped, and 64 repeated local jobs plus two simultaneously live alias vectors
retain stable interned handles and independent local bodies. All 27 frozen
postchecks pass. This uses existing native registry and alias owners without a
reset, extra global body cache or dependency patch. Production allocator and
full scientific regression controls remain required when the adapter is wired.
The [shared endpoint admission extraction](reviews/native-endpoint-admission-independent.md)
then passes 50 focused scientific tests, zero failures and two explicit probes
ignored, with package formatting/all-target Clippy passing. Both physical
strategies reuse the unchanged native rational/affine/floor operations and
preserve zero-slope pruning/error precedence. Generation now checks cancellation
degree overflow through a typed error. This creates one reusable admission
owner without selecting a new coefficient method, changing an artifact or
adding symbolic mathematics.
The [private request-seam review](reviews/native-named-request-interface-independent.md)
accepts a source-only four-method boundary using native Series mapping,
derivatives, literal replacement and shared aliases. Existing infallible native
callbacks require a local first-error latch and complete failed-attempt discard;
there is no new replacement engine or Series constructor. The caller retains
cancellation/status ownership, and a unique-request cap explicitly does not
claim to bound native intermediate memory. Concrete implementation and its
error/cancellation/scientific controls subsequently pass 14 focused tests plus
15 existing subtraction/Series tests, with no failures and two explicit replay
probes ignored. The independent audit verifies the eight-file source snapshot,
native complete-vector/coverage/error controls, actual allocator hygiene and
local cache lifetimes. Formatting and scoped library/test Clippy pass. Public
options/status adoption, cold replay and whole-graph acceptance remain separate;
the physical default is unchanged by this private gate.
The subsequent public coordinator's independent focused audit verifies 16
archived source files and 41 distinct passing tests. Explicit `NativeNamed`
selection reuses the admitted mapped density and rejoins the existing complete
vector, multiplicity, metadata and evaluator owners. The physical default and
exact unregulated fallback retain their semantics. Fresh named conditioning is
explicitly a componentwise mapped-endpoint bound for the existing precision
heuristic; unchanged artifact rows do not fabricate a cold-loaded basis.
Public complex/Gamma vectors, independent native Series/MPFR comparisons,
weighted replay, worker clones, separate cold processes, geometry reuse,
limits/cancellation and admitted zero-chart preservation all pass. The CLI
source directly reuses native options and typed snapshots, keeps timing
exclusive and polls cancellation during coalesced presentation. Formatting
and scoped library/test Clippy pass. The combined closure then passes 370
workspace tests, zero failures and 23 ignored across 62 summaries, with
formatting and all-target Clippy passing. The CLI correction preserves existing
generation JSON and terminal/plain behavior and coalesces only new coefficient
JSON; the original chart/cone regression remains exercised. Its first cadence
failure and the separate test-only wrong-error-stream assertion are retained.
This accepts public opt-in adoption and the bridge. The actual reconstructed
public representative, whole-graph completion and performance remain separate
gates; no new graph, Series, evaluator, serialization or statistics owner was
introduced.
The [ordinary public representative gate](reviews/native-named-public-actual-independent.md)
subsequently closes the reconstructed-input boundary at the same committed
production source. Existing public parametric constructors admit the captured
two-factor density with exact native identity; public generation, compilation
and v3 save complete in 108.36 seconds. The actual unit-measure chart reverses
the nine coordinates, and all three fresh public cold readers apply that map
with multiplicity one and literal-zero exact offsets. All seven signed orders
pass at three independent original-expression points (21 comparisons), with
24 weighted vectors/168 real components and twelve forced 1,024-bit replays.
Normal rescues use 256/256/384 bits. All 39 build, 170 generation and 173
per-reader frozen checks pass. The first helper-only structural exponent
expectation failure remains retained; its one-line correction changed no
production algebra or input. Diagnostic native IR decoding follows ordinary
public artifact validation and uses the existing native codec. This closes
one representative's public capability/equivalence gate without adding an
algebra, graph, evaluator or statistics owner. Original full-graph/integral
parity remains separate; these timings add no tuning or performance claim.
The [prospective production boundary review](reviews/native-named-production-boundaries.md)
records the original interface decisions for admission, conditioning,
resource/cancellation and public progress; the accepted opt-in and public
representative evidence above qualify which parts have now been exercised.

The earlier combined test-only interleaved milestone passes 336 workspace tests with
zero failures and 23 explicit probes ignored across 60 summaries. Formatting
and all-target Clippy pass. The initial dead test-wrapper lint is retained;
its fix routes the original writer through the existing original wrapper and
leaves candidate semantics unchanged. The separately executed small writer,
cold reader and captured generation evidence above remain necessary beyond
this ordinary workspace gate.

The [CLI prepared-family adapter](reviews/cli-family-preparation-independent.md)
reuses `FamilyPreparationPolicy`, `FamilyPreparationReport` and
`ParametricIntegrand::from_graph_prepared` directly. The original CLI path still
calls `from_graph`; opt-in preparation supplies all original parameter labels
and retains the returned active labels and report. Original sources remain in
provenance, with the native report covered by artifact identity. No separate
graph, reduction, algebra or coordinate-pullback implementation is added.
The focused gate passes 38 tests, formatting and CLI all-target Clippy, including
full weighted Laurent vectors, separate-process cold artifacts, fallback and
tamper rejection. Full original on-shell integral acceptance remains separate.

The [persistence memory review](reviews/native-persistence-independent.md)
confirms reuse of Serde's borrowed serialization, streaming writers and
`RawValue`, plus the existing native evaluator codec and BLAKE3 identity owner.
Borrowed payload views preserve native v3 bytes and IDs. Cold loaders retain
their validated input without rebuilding an unused envelope; callers can borrow
artifact bytes while the existing owned accessor remains available. The CLI
uses an explicit v2 envelope identity and keeps its v1 identity/load branch.
Native payload verification remains mandatory, including when the claimed
embedded ID is unchanged. Atomic streaming output retains failure cleanup and
the CLI releases the generated symbolic object after compilation. No second
algebra, graph, evaluator or parallel executor is introduced. The focused gates
pass 16 native and 42 CLI tests, including legacy identities, complete complex
vectors, cold processes, tampering and interrupted writes. Full-graph capacity
is determined by the subsequent original-input run, not inferred from these
small controls.
The combined milestone passes 380 workspace tests, zero failures and 23 explicit
ignored probes across 63 summaries, plus formatting and all-target Clippy.

The subsequent [CLI QMC context-lifetime change](reviews/qmc-worker-context-lifetime.md)
reuses `QmcSession::worker_context`, `KernelSet::restore_evaluation_context`,
`WeightedEvaluationContext::merge_state` and the existing accepted-replay store.
The caller retains one active sector context per worker and restores accepted
history when revisiting a sector. No library executor, evaluator, arithmetic,
lattice, checkpoint format or public API is added. Ordered successful package
submission still owns accepted replay advancement. All 16 focused driver tests
pass, including precision rescue, vector/covariance equivalence and checkpoint
recovery with eight workers; formatting and scoped all-target Clippy pass.
Large-run memory and speed remain measured questions for the saved-artifact
continuation rather than inferred benefits of the smaller ownership bound.

The [CLI refill scheduler](reviews/qmc-refill-scheduling.md) then reuses native
`next_work`, `submit` and in-flight checkpoint restoration, with the caller's
existing Rayon pool and standard completion channel. Numerica keeps canonical
full-vector reduction and covariance ownership; only the coordinator advances
accepted replay. No public executor, estimator, lattice or checkpoint format is
added. Twenty focused driver controls pass, including deterministic refill
before a straggler, full covariance, cancelled/failed-prefix exclusion,
pending-work restoration and worker panic draining. Formatting and scoped
all-target Clippy pass. The change responds to a measured batch barrier;
whole-integral completion and speed remain separate runtime observations.

## HEPKit helicity input extension

The generated gg→HH example needs loop products with external polarization
vectors outside the graph's momentum-routing basis. Public API and source review
confirm that `Kinematics::with_scalar_product`, `IntegralFamily::new` and the
existing native Symanzik/Gaussian source path already support the required
formal vector labels and numerical, possibly degenerate, Gram data. No external
Gram inverse or new tensor reduction is introduced. `GraphIntegral` now appends
those vectors through native family construction; CLI cards transport canonical
tagged Atom names and preserve the existing integer P(i) shorthand.

The independent [input review](reviews/hepkit-auxiliary-vectors-independent.md)
finds no actionable source issue. Two native tests compare real and genuinely
complex rank-two Gaussian coefficients against HEPKit's `TensorReducer` through
epsilon one, including degenerate Gram data, and check binding order and native
label rejection. Four CLI input tests pass, including exact model overrides and
native imaginary-number transport. The latter fixes an observed use of a plain
namespace symbol `i` by constructing `Atom::i()` instead. Numerical coefficients
now use native `Rational::try_from(f64)` to preserve supplied binary values in
the native affine family's exact field. The broader affected input gate passes
19 tests (two auxiliary, three example and fourteen existing input controls),
with workspace all-target Clippy and formatting passing. A first test invocation
omitted the documented serial-test flag and hit a native UFO symbol-registration
race; its log remains alongside the passing standard serial run.

The generated fixture selects eight eligible top/gluon double boxes from 192
native diagrams, retaining the first, its factors and the exact requested
physical point. The corrected export takes 1.098 seconds in this diagnostic.
The following ordinary release CLI generation reaches its 600-second deadline
without a sector artifact, with peak RSS 50,123,276 KiB. A subsequent stepwise
native probe measures `simplify_algebra` at 0.184 seconds and `to_dots` at
0.050 seconds; the whole tensor/input path finishes within 0.37 seconds. The
remaining Gaussian `from_family` call reaches its separate 120-second bound,
locating the bottleneck in parametrization rather than HEPKit contraction.
Neither full numerical feasibility nor browser suitability is accepted. All
failed export/import attempts remain
under `output/diagnostics/gghh-native-*`; the checked-in fixture is the corrected
third export, not either earlier failed numeric transport.

GammaLoop's existing numerical external states have been extracted into shared
FeynKit kinematics, covering scalars, massive/massless vectors and spinors with
their existing adjoints and phases. Six focused native controls pass against the
current FastSecDec dependency identity. The latest upstream branch additionally
passes six native controls, five existing GammaLoop regressions, the installed
binding check and generated stub export. The tested extraction is published as
GammaLoop `6c707c6b7`; [HEPKit PR #17](https://github.com/symbolica-dev/symbolica-community/pull/17)
is ready for review after the complete installed community wheel passes
fourteen wavefunction cases, the namespace/stub check and eight literal
documentation examples. Baseline CI/documentation limitations are recorded in the
[shared-wavefunction review](reviews/shared-external-wavefunctions.md) records
source ownership and validation. The Python API stays in the HEPKit ecosystem,
with no Python dependency in FastSecDec's production crates.

The [portable evaluator proposal](reviews/portable-kernel-feature-split.md)
identified one narrow native admission gap: Symbolica's interpreted coefficient
mapping could panic for unsupported fixed constants or callbacks, while its JIT
constructor already handled those cases fallibly. The additive
[`try_map_coeff_with_prec` patch](dependency-patches/symbolica-fallible-coefficient-map.md)
reuses the same constant and callback owners and preserves existing mapping
behavior. Ten focused native controls and independent source review pass.
The subsequent actual FastSecDec feature passes 52 native and 39 portable host
tests and an Emscripten/Node complete-vector smoke. The latter includes artifact
reload, 512-bit weighted replay and 4,096 QMC points with full covariance, using
the existing library APIs throughout. The selected portable dependency tree
contains Malachite/Astro and excludes GMP/MPFR/SymJIT. Browser event delivery and
the Python wheel remain separate checks.

The parametrization bottleneck above is now isolated and corrected without
expanding regular numerator support. Gaussian sources are eliminated only when
their remaining derivative order is zero. Native fixed-variable polynomial
conversion then checks the common scaling degree of each regular factor;
unresolved cases retain the original sparse-support fallback and singular U/F
admission remains unchanged. The additive
[coefficient-field configuration seam](dependency-patches/symbolica-fixed-variable-coefficient-field.md)
allows deterministic coefficient zero tests inside that existing Symbolica
conversion. The configurable full-polynomial conversion, collection APIs and
post-conversion ring mapping were checked and cannot supply that missing seam.
No degree walker or polynomial arithmetic is implemented in FastSecDec.

Thirty focused controls pass, including native numerator-reduction comparisons,
large factored powers, cancellation, nonpolynomial rejection and the shared
status transitions. The unchanged public gg→HH `from_family` path admits all
179 terms in 1.881 seconds; its complete process takes 3.719 seconds and peaks
at 70,816 KiB. This closes parametrization only: ordinary sector generation,
integration and notebook suitability remain separate gates. Evidence is under
`output/diagnostics/gghh-homogeneity-1` and `gghh-input-admitted-1`.

### Shared HEPKit owners and exact algebraic domain signs

The native library and developing community bridge now share the published
FeynKit `6c707c6b7` lineage, with the existing literal-symbol substitution fix
in an isolated worktree. Cargo metadata verifies unique native graph, model,
kinematics, tensor, Linnet, Idenso, Spenso, Symbolica and Numerica owners. Two
explicit `BTreeSet` collection annotations accommodate the community feature
set without changing validation behavior. The
[independent bridge source audit](reviews/hepkit-bridge-source-audit.md) checks
direct native inputs, caller-stepped weighted QMC, typed snapshots, full
covariance and native persistence. Its interruption finding is being validated
through the installed Python bridge; successful native type checking alone
does not close that runtime or browser gate.

The next ordinary gg→HH CLI attempt exposed an admission limitation: exact
algebraic F coefficients were unsupported by the rational-only sign adapter.
Symbolica's existing `AlgebraicContext` and `RealEmbedding::try_sign` certify
all 57 gathered coefficients positive. The small adapter caches those native
results and leaves unsupported or non-real values inconclusive; it introduces
no radical arithmetic or approximate positivity rule. Native coefficient
gathering uses deterministic zero tests. The
[exact-sign review](reviews/exact-algebraic-domain-signs.md) records API,
source/test and executable reuse evidence.

Forty-two controls pass on the aligned owners: four exact-domain tests, sixteen
generation tests, eight generation-context tests, four regression-gap controls,
five native one-loop scalar-master comparisons and five numerator-reduction
comparisons. All six test processes exited successfully and were reaped. The
coordinator independently verified the eight bound source files and six report
hashes in `output/diagnostics/algebraic-domain-1/result.json`. Workspace formatting
and all-target Clippy also pass. Full gg→HH
generation/integration and actual notebook event delivery remain separate gates.

The follow-up mapper probe attributes the gg→HH stall to full support extraction
of regular numerator factors. The
[regular-monomial review](reviews/native-regular-monomial-mapping.md) records the
replacement: collect one coordinate with Symbolica while keeping the other
coordinates in Atom coefficients, then accept a common monomial only through
the existing native factor collection and polynomial-recognition guard. This
does not claim a maximal valuation or nonzero certificate. Singular factors
and signed orthant maps keep their previous path. Independent source review,
six mapper controls, fifteen subtraction controls and sixteen complete-vector
generation controls pass. The existing native signed exponent limit remains
documented, including its behavior before conversion returns.

The installed community bridge now passes all 39 focused tests with no skips:
native input selection, all four showcase inputs, typed events, cancellation,
checkpoint identity and accepted work, complex covariance, native kernel bytes,
numerical failure and retry, and shared external states. Native status and stub
checks pass separately. Evidence is `output/diagnostics/bridge-native-2`, with
the precise compiled sources preserved before subsequent lint-only changes.
The [native notebook UI review](reviews/fastsecdec-showcase-ui.md) additionally
records an actual Chromium Run/Cancel/Resume/Complete workflow. The coordinator
verified its eleven bound inputs/artifacts and the unchanged accepted checkpoint
prefix. Those checks cover a native server; actual Pyodide execution and browser
responsiveness remain pending.

The ggHH input's native color closure now uses Idenso's existing explicit SU(N)
invariant option, matching GammaLoop's evaluator preprocessing. API/source/test
inspection and an actual generated-fixture Rust probe verify the exact
symbolic-to-explicit conversion and `T_F = 1/2` normalization. The external
color delta is consumed once; raw diagram data, couplings, graph topology,
physical point and D-dimensional Dirac algebra are preserved. An independent
read-only ecosystem review accepted the boundary. The
[color-closure review](reviews/gghh-native-color-closure.md) records this example
preparation fix. Its subsequent ordinary CLI generation and full-vector
integration now complete; the [native feasibility review](reviews/gghh-native-feasibility.md)
records the 61.285-second generation, 8.781-second eight-worker allocation,
complete covariance and 1.0273% finite-part relative error. This closes the
native example prerequisite without claiming browser feasibility, one-per-mil
accuracy or an independent amplitude reference.

The actual Pyodide community bridge now passes the same 39 focused controls as
the native wheel. The [notebook UI review](reviews/fastsecdec-showcase-ui.md)
records the real marimo browser's native graph rendering and caller-stepped
Run/Cancel/Resume/Complete workflow, including exact accepted checkpoint prefixes
and full covariance. The triangle reaches its requested target; the other three
inputs have bounded portable probes. No Python evaluator, graph representation
or library-owned integration loop was added.

The [dependency-delivery review](reviews/dependency-delivery.md) records the
replacement of machine-local manifests with public sources and an explicit
pinned bootstrap. Its fresh owners reproduce the existing reviewed contents and
patches, retain one native owner per ecosystem crate, and select exactly one
FastSecDec backend. Native all-target Clippy, three CLI provenance controls,
portable-host checks and locked metadata pass; both lockfiles remain unchanged.
The coordinator verified all ten source/lock bindings in the delivery report.
An independent read-only audit also verified the patch hashes, emitted configs,
locked owner graphs, backend separation and bootstrap/provenance boundaries;
it found no delivery blocker within the tested Linux scope.
CLI builds reject missing provenance instead of publishing an unknown identity.
Community validation against the published FastSecDec Git revision subsequently
passes in [merged HEPKit PR #18](https://github.com/symbolica-dev/symbolica-community/pull/18).
The fresh installed native wheel passes 39 tests with zero skips; independent
review and postflight checks bind all 169 compiled source files and 38 delivered
files. The preparation helper uses an isolated Cargo home for Maturin/Pyodide,
and the actual portable PEP 517 metadata flow passes. This does not relabel the
earlier actual Wasm/browser execution as a new build. Upstream adoption of the
documented dependency patches remains open; unpatched optional-feature-off
resolution still conflicts on the OneLOop SymJIT pin, as that delivery review explains.

The [bounded double-box follow-up](reviews/double-box-lattice-followup.md) uses
Numerica's existing published HKKN catalogue, ordinary native QMC stepping,
saved-result validation and reference comparison. No lattice construction,
variance formula or mathematical kernel was reimplemented. Independent review
accepted the effective rule, complete common-shift coverage, full covariance and
reference-only result transport. The prescribed 8,192- and 16,384-point
observations reach 0.393 per mille finite-part relative standard error at the
second level; both complete processes total 664.554 seconds. These observations
do not establish Pathfinder parity or uncertainty calibration. This bounded
lattice follow-up stops at that target; the production default remains unchanged.

Current public-owner generation subsequently passes normal strict loading with
the exact retained kernel envelope. The sole larger-package experiment uses
ordinary run-card settings, native generation, Numerica's existing identical
point/shift plans, per-point weighted replay, native saved-result validation and
reference comparison. It preserves all sector estimates and covariance exactly,
but provides no timing improvement; no default or mathematical source changes.
The bounded Pathfinder counterpart uses its own existing prime-table backend and
retained bundle, without external package generation. Both reach the reported
finite-part target; different rules, timing intervals and error semantics remain
explicit, and performance parity is still open.

- 2026-10-06: independent source/contract review accepts the retained endpoint/evaluator metadata and native discrete-sector Havana boundary. Source-chart powers reuse native endpoint admission; the CLI and immutable Python owner/index views expose those records without rebuilding maps or expanding the regular density. Actual program/operation/SymJIT application sizes retain their distinct meanings and legacy missing metadata remains explicit. The MC lane reuses Numerica DiscreteGrid/ContinuousGrid, jumped RNG streams and shared complete-batch covariance; weighted precision replay applies the full proposal weight once. Pilot training is excluded, production checkpoints bind native identities, and stochastic sector allocation exposes actual probabilities/counts rather than invented fixed quotas. Outer execution and parallelism remain caller-owned. The only dependency change is Numerica’s published recursive sample-free nested-grid clone fix (`f6ecdac`), with 249 native and 226 portable controls including documentation. Native metadata17, existing/new MC29, CLI41+33 (three explicit old heavy ignores), and binding all-target Clippy with stub generation pass their separate gates. The new installed native wheel/UI and actual portable target remain pending; no physical ggHH accuracy or performance claim follows. CLI MC caches can retain every visited sector per worker; Python pilot pause is in-memory, while persistent checkpoints require frozen production.

The subsequent native milestone passes 419 workspace tests (25 explicit heavy
ignores), 81 installed binding/frontend controls, 41 portable-backend host
controls, formatting and strict all-target Clippy. The real native notebook
exercises retained metadata and rendered formulas, QMC, same-kernel allocation
replacement, and Havana pilot/production pause/resume. Fresh ggHH generation
retains all 30 charts and 54 mapped terms with every exact evaluator program
unchanged. Independent root review accepts the complete ordinary-Havana vector,
covariance, production coverage and completed-checkpoint restoration; component
agreement with QMC is within 1.70 combined standard errors. Both finite-part
estimates meet one per mil. This uses existing native reference adapters and
preserves their UnverifiedReference qualification, rather than claiming an
independent analytic amplitude. Actual Wasm validation of the new interfaces
and the bounded remaining representative parity claims are still open.

The HEPKit entry-point extension adds `sector_decompose()` to the existing
native `FeynmanDiagram` and `IntegralFamily` owners and the canonical
`hepkit.sector_decomposition` namespace. Community contains registration,
reexports and generated stubs; the optional FeynKit methods forward without a
reverse Rust dependency. Family support reuses native denominator ordering,
`IntegralFamily::sector`, constructors and kinematics; Symbolica supplies
negative-power numerator factors and scalar substitution. Explicit signed powers
avoid assigning unit powers to family-completion denominators. The graph and
family paths share native preparation, and neither duplicates graph or algebra
primitives in Python. Symbolica's existing symbol inventory also guards against
momentum-dependent scalar bindings, including tensor function heads.

The [entry-point review](reviews/hepkit-sector-entrypoints.md) records independent
ownership and scientific-contract review, 29 native input/parametrization tests,
95 installed Python/frontend tests, and the actual native notebook lifecycle.
Visible notebook functions are the callbacks its buttons execute. Native
`DiagramRender` reaches marimo through its existing HTML protocol. Generated
type-stub and actual Wasm validation remain separate delivery gates; no new
performance or ggHH accuracy claim is inferred from the triangle UI control.

The self-contained ggHH extension reuses `Model.standard_model()`, native
process generation, Linnet-backed connectivity, existing tensor contractions,
GammaLoop external states and `diagram.sector_decompose`. Inline parameter data
uses the existing Rust scalar-binding algorithm moved from the CLI to its native
FeynKit Model owner. Both the CLI and previous notebook helper now call that
owner; the Python dependency-resolution loop is removed. Five Model and four
CLI controls pass, including exact equality of the complete earlier ggHH binding
map. Independent [input audit](reviews/gghh-single-notebook-audit.md) also verifies
exact physical graph equality after accounting for the model fingerprint and
display name. The direct diagram call exposed a pre-existing admission-order
defect for explicit zero-width overrides of a nonzero-width default model. The
atomic constructor reuses the existing admission guards and native family
specialization; 30 input/family/example and five CLI tests plus strict Clippy
pass. Final installed-runtime checks remain in progress.

## FastSecDec QMC ownership and ordinary dependency delivery

The user's subsequent instruction relocates the tested QMC implementation from
Numerica's feature branch to `crates/fastsecdec-qmc`. The new owner retains its
catalogue data and licenses, caller-owned work scheduling, lattice/transform
algorithms, full-vector covariance and checkpoint formats. It reuses public
Numerica RNG state export to obtain the same seed words rather than introducing
another generator. Frozen old-owner fixtures reproduce plan bytes, point bits,
partial accumulators and resumed covariance. The original branch and PR remain
historical evidence; production uses registry Numerica 3.0.1.

Ordinary Havana remains in Numerica. Its released discrete-grid sample-free
clone omits the returned child clone; a private FastSecDec adapter composes the
existing grid operations for the one-level discrete/continuous shape used here.
Unsupported shapes produce an error. There is no replacement integration grid,
sampler, RNG, algebra or library-owned worker pool.

Independent reviews and 37 native plus 37 portable-feature QMC tests pass, as do
43 focused core QMC/MC controls and strict QMC Clippy. Portable-feature tests in
this milestone ran on the host, not in a new Wasm wheel. Actual linked SymJIT
version reporting, source provenance and artifact compatibility pass another
46 focused tests and scoped strict core/CLI Clippy. SymJIT requirements are
compatible minimums; lockfiles record resolution. CLI provenance uses the
consumer workspace's resolved Git/registry identities, with source-root
fingerprinting only for explicit local path overrides.

The updated HEPKit literal-substitution owner passes three native regressions.
OneLOop's public cache change stores evaluator IR and recompiles it; two owner
controls and the updated FastSecDec one-loop cache regression pass. These
dependency and compatibility checks introduce no new physics, convergence or
performance claim. Upstream cleanup and ordinary-build readiness are tracked in
the [dependency delivery review](reviews/regular-hepkit-build.md).

The subsequent public-source cleanup removes five superseded dependency patches.
Independent production-hunk/semantic review confirms the three Symbolica fixes
in `473b4b8`; merged OneLOop `27c37234` has the tested PR's exact source tree.
All 23 affected artifact, complex-kernel and Gamma regressions pass against the
updated Symbolica base plus four remaining patches. There was no observed bad
IR from generation: decoder regressions deliberately corrupt saved programs.
The [constant-domain audit](reviews/fixed-constant-error-tracking.md) records
that generic fallback conversion assigns nominal uncertainty without tracking
callback-internal errors. This remains an upstream design issue rather than a
new downstream numerical implementation.


## Unpatched upstream Symbolica integration

The user supplied the required coefficient-field API upstream in Symbolica
`community` revision `58652fabc2f736302a570deaaf8d517679f7fe6e`. The remaining
local patches and dependency-preparation scripts are removed. Ordinary Cargo
resolves one public owner per shared ecosystem crate, with registry Numerica
and Graphica; no package version change or alternative algebra implementation
is needed.

The mapping adapter reuses public `ExpressionEvaluator::export_instructions`,
`EvaluationDomain::resolve_function`, `EvaluationDomain::try_from_complex_float`,
`EvaluationInfo::evaluate_constant` and upstream `map_coeff_with_prec`. The
existing fallible JIT constructor remains unchanged. Callback requirements and
native parsed tags are collected once at construction; workers share immutable
requirements and keep bounded numerical evaluator caches. It does not evaluate
Atoms per sample, copy a mapping implementation, or add special functions.

Native Gamma/polygamma already support ordinary and multiprecision arithmetic.
A missing error-tracking callback now disables only the conditioning shortcut.
The same boundary, nonfinite and range-loss predicates lead to the existing
precision rescue, with unchanged tolerances and two-precision agreement.
Unsupported required callbacks or constant conversions return typed errors;
there is no catch-unwind dependency or generic conversion assigning an assumed
error to a callback result.

Independent source review accepts the separation of these responsibilities.
Five focused controls cover nested native bodies, missing callbacks, failed
constant conversion, nonreal fixed arguments, and real/complex special-function
rescue with worker cloning. Existing public artifact and portable controls remain
part of the runtime gate; actual outcomes are recorded at milestone completion.

The optional native-IR structural validation proposal is retired. Kernel caches
retain their producer bytes, identity and outer format checks, while their
public loader documents trusted native-program provenance. The intentionally
corrupted raw-index test is removed before using the unpatched decoder. This
changes the cache trust contract, not generated integral expressions or numerical
methods. It is not replaced by a FastSecDec instruction validator.

The ordinary public-source runtime gate passes 475 native workspace tests with
25 existing ignored diagnostics, plus all 58 maintained portable-backend host
controls. Strict workspace all-target Clippy, root/leaf formatting and the Python
binding's all-target check with `python_stubgen` pass. Root and standalone
consumer graphs retain one ecosystem owner per shared type without path overlays;
the portable active tree excludes GMP. The corresponding community promotion
keeps numerical implementation in FastSecDec and removes the preparation script
and experimental feature gates. These checks establish upstream compatibility;
they do not relabel historical native/Wasm wheels or add physics/performance
acceptance. See the [delivery review](reviews/regular-hepkit-build.md).


## Cooperative eager notebook owners (2026-10-07)

The new caller-stepped generation owner reuses native geometry job/completion
admission, existing mapping/canonization and coefficient expansion, and shared
Laurent assembly. It retains completed work rather than replaying generation on
resume. Four scientific owner tests pass in both native and portable host builds.
The independent coordinating review is recorded in
[cooperative generation](reviews/cooperative-generation-review.md).

A separate independent audit accepts runtime eager selection through Symbolica's
existing ExpressionEvaluator and coefficient/callback mapper, plus resumable
CompilationSession ownership. Native real/complex analytic controls pass direct
versus stepped compilation, two runtime points, cold loading, exact-offset-only
kernels, batch tails and f64/DoubleFloat/Arb routing. No new algebra, graph type,
interpreter, numerical dependency solver or numerical worker pool is introduced.
See [notebook workflow reuse](reviews/notebook-workflow-reuse.md) for the precise
owner APIs, evidence, fixed pause/error-boundary findings and limitations.
The native unit bound cannot interrupt a single expensive upstream call. Host
portable tests are not a claim of actual Wasm execution, and upstream FeynKit's
static method keyword documentation remains a recorded limitation.

The final [native browser review](reviews/notebook-browser-acceptance.md) records
actual ggHH/scalar Generate, Inspect, QMC, Havana, runtime point changes,
pause/resume, lazy numerator/integrand navigation and cold artifact import.
Both raw and simplified views reuse HEPKit's native scoped Pager. Dedicated
owning cells and persistent HTML disclosures respect Marimo's comm-disposal
contract; exported bytes are prepared explicitly on the native caller.

Native QMC work packages now use the existing 4096-point setting while evaluator
batches remain 256 points. Independent two-million-point comparisons reproduced
the entire mean and covariance exactly and exposed mandatory snapshot reduction
as the old notebook overhead. No alternate reduction or integrator was added.
Final acceptance includes 486 owned-source workspace tests (25 intentionally
ignored), 59 portable host tests, 73 installed Python binding tests, 88 notebook
tests, strict Clippy and formatting. Concurrent numerator-contraction work is
excluded from this milestone. The native massless-box QMC cold 1000-digit
polygamma preparation cost remains explicitly documented; actual execution of
this new notebook in Pyodide remains a separate future validation.
