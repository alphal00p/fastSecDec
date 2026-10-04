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
| Exact integers, rational matrices | Numerica integer/rational domains and matrix determinant/rank operations | Exact sector map, moment and cone tests | Sector-specific normal-fan and triangulation orchestration only |

The initial probes establish the existing owners. The implementation must add
evidence to this record before introducing further algebraic functionality.

## Narrow missing operations

- General polynomial loop-numerator **integration into parameter space** is not
  provided by the inspected family numerator-basis APIs. The implementation slice
  under development uses native scalar-product bases and auxiliary Symanzik
  sources, then Symbolica derivatives. It must pass cancellation and independent
  Gaussian-moment tests before this gap is considered implemented.
- The inspected ecosystem has general exact linear algebra and graph
  canonicalization, but no complete sector-specific Newton-fan decomposition
  pipeline. `fastsecdec-sectors` supplies that pipeline while using Numerica exact
  arithmetic. The independent review is recorded separately under `reviews/`.
- Existing Havana accumulators describe independent Monte Carlo samples. The new
  Numerica QMC lane supplies indexed randomized rank-one points and statistics
  over complete shift means. It does not replace Havana MC or its numeric types.

Graph canonicalization, high-precision rescue and symbolic integration are
**not yet marked implemented** by the initial probes. Their existing APIs must
be used and tested when the corresponding slices land.
