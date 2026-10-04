# Ecosystem reuse evidence

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
pool was introduced. A paired full-artifact timing remains pending.
