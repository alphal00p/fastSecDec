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

The supplied Symbolica build could not convert fixed-argument external constants
to its error-tracking domain when that function only registered a multiprecision
hook. The [minimal dependency patch](dependency-patches/symbolica-fixed-argument-constant-domain.md)
reuses that registered hook and native conversion, with a passing focused
regression. FastSecDec does not implement special-function constants itself.
