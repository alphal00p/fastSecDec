# HEPKit integration and ecosystem reuse audit

Date: 2026-10-04. Reviewed the working tree after milestone `f559059` and the
subsequent 133-test workspace gate. This is an independent review of the native
input, generation, symmetry, kernel wiring, and CLI authored by other agents.
The reviewer authored the integration coordinators and Numerica QMC lane;
their contracts are assessed here for compatibility, while their independent
scientific reviews remain the earlier peer/coordinator reviews. The current
findings below were resolved before the 147-test milestone gate; the future
Python-wrapper accessors remain explicitly deferred to that bridge phase.

## Prioritized findings

### P1: inline model overrides can retain stale cached dependencies

`fastsecdec-cli/src/input.rs::model_values` prefers a parameter or coupling's
cached numeric `value` to its symbolic expression. `load` then extends that map
with TOML `[parameters]` overrides. Native
`feynkit-model/src/model.rs::Model::apply_parameter_card` explicitly invalidates
expression-backed internal values and all coupling caches; this path is used
for a separate parameter-card file, but is bypassed for inline overrides.

For external `a=2`, internal `b=2*a` with cached value 4, overriding `a=3` should
recompute `b=6`. The current map construction keeps the cached 4. This can change
physical masses or weights silently, so the input owner is adding an executable
regression and a fix before the next milestone. Reuse native invalidation and
literal Symbolica substitution, preserving explicit internal restriction-card
values; do not create another model evaluator or silently drop unknown values.
This finding is confirmed at the API/source level; its reproducer and resolution
must be recorded below after the owner's focused test runs.

### P2: reusable numerical diagnostics are stranded in the CLI

`fastsecdec-cli/src/diagnostics.rs` owns `KernelTiming`, `BoundaryProbe`, the
benchmark loop, and boundary-point generation/evaluation. A future HEPKit bridge
would have to duplicate that numerical work or depend on a binary crate. These
results also encode face orientation in display strings rather than typed data,
and the loops have no caller cancellation or partial-report API.

Move the typed requests/results and bounded, cancellable diagnostic routines to
the library. Retain CLI formatting and command parsing in the CLI. A report
should state its sampling policy, planned/completed probes, coverage truncation,
and cancellation explicitly. The caller must be able to receive each result
and cancel without losing completed probes. This is a current library-boundary
fix, not a request to implement the Python bridge now.

The current boundary command samples each individual lower/upper coordinate
face plus all-lower, all-upper, and two alternating corners. In dimensions above
two this omits many face intersections and is not an exhaustive check of
"every boundary" as its help suggests. A bounded policy should enumerate all
endpoint assignments for coordinate subsets up to a requested codimension,
with an explicit total budget. Finite samples are diagnostics, never a positivity
or integrability certificate. The CLI already limits decimal exponents to 1–15
so upper-boundary points remain representable; this validation belongs in the
reusable request as well.

### P2: regular mapped factors still expand unnecessarily

`generation/mapping.rs::map_terms` still constructs every factor's support and
expands its mapped residual, including fixed positive polynomial numerator
factors. This is narrower than expanding a whole sector density, but can destroy
a compact high-degree numerator and conflicts with the factored-expression
invariant. The existing `ecosystem_probes` confirms native
`to_polynomial_in_vars` accepts factored input and native signed `mul_exp` removes
monomial valuations. No new polynomial algebra is needed.

The generation owner has accepted this follow-up: preserve the mapped Atom when
its valuation is zero; use the audited native signed-polynomial operation when
division by a monomial is needed, and avoid regular-factor support work when the
factor can remain unrefined. Source changes and matched generation checks are
still pending at the time of this audit. The singular-factor predicate now
correctly excludes fixed nonnegative integer powers from fan construction.

### Future bridge blocker: graph/kinematics wrapper accessors

The pinned `feynkit-py` publicly exports `PyFeynmanDiagram` and `PyKinematics`, but
their native fields are `pub(crate)` and no public borrowing accessor is exposed.
`PyIntegralFamily::as_family()` demonstrates the needed existing pattern.
A bridge hosted in the separate `symbolica-community` crate cannot directly
borrow those two objects today. `PyFeynmanDiagram` can also represent a selected
subgraph while its `inner` still refers to the whole diagram: exposing that field
without checking selection would be scientifically incorrect.

At bridge implementation time, request narrow upstream accessors returning the
native kinematics and a validated whole-diagram `Arc`. A selected region must be
rejected or explicitly materialized through the existing native graph API.
Do not serialize a graph to DOT and parse it again to bypass privacy. No patch
is needed in this phase, and no Python build/probe was run for this static
visibility finding.

## Native object and execution boundaries

`GraphIntegral::new(Arc<FeynmanDiagram>, &Kinematics)` already gives HEPKit a
direct Rust entry point. The graph retains topology, edge identities, routing,
model, numerator fragments, projector, and overall weight. FastSecDec adds
propagator powers, closed scalar bindings, and a measure multiplier. Its native
family is exposed by reference. DOT is only a CLI/convenience constructor.
The `native_input` tests exercise native graph/Linnet identity, DOT roundtrip,
weight application exactly once, scalar products, symbolic dimensions, and
parameter binding before native scalelessness analysis.

`ParametricIntegrand`, `PolynomialFactor`, `ParametricTerm`, generated sectors,
and exact coefficients all use the shared Symbolica `Atom`/`Symbol` directly.
Graph parameterization uses the shared `IntegralFamily` and native Symanzik API.
There is no alternate physical graph, DOT parser, tensor notation, or CAS value
type. A future family-only convenience constructor could expose the existing
internal parameterization path, but native graphs and direct U/F already cover
the agreed phase-one interface; it is not a current blocker.

The input error enum retains native model, diagram, and family causes, with typed
unsupported-domain errors. Generation, kernel, and integration errors preserve
their subsystem categories. Tensor and generic worker errors carry messages;
a future Python adapter should map categories explicitly rather than parse
display strings. Such an adapter belongs entirely to the community bridge.

`GeneratedIntegral::compile` passes native expressions directly to evaluators.
Canonical strings/JSON appear for scientific identity hashing and explicit
portable artifacts/checkpoints, not as an in-process graph or Atom conversion.
The prior complex canonical-printer and fresh-process special-function identity
defects have regression tests and narrow upstream patches; those artifact tests
must remain part of integration-readiness checks.

The library owns no worker pool, OS integration thread, terminal, or refinement
driver. QMC/MC sessions expose `next_work`, worker contexts, `submit`, estimates,
snapshots, and checkpoints. A worker builds points and evaluates a full Laurent
vector on the caller's thread. Kernels clone their existing native evaluators;
precision rescue uses prebuilt numerical IR, avoiding symbolic work on numerical
workers. Caller-owned threaded tests cover this contract.

QMC shift statistics combine all stochastic sectors before democratic covariance;
adaptive pilot observations are excluded from frozen production. Real/imaginary
components are explicit, and covariance includes inter-order and inter-component
correlations. Exact whole sectors remain exact. Existing independent runtime
reviews and tests cover these scientific contracts; this audit does not claim
a second independent authorship review of the reviewer's own implementation.

Generation callbacks, compilation callbacks, `IntegrationSnapshot`,
`GenerationSnapshot`, `GenerationTimings`, `EvaluationDiagnostics`, and typed
stopping/coverage states are terminal-independent. Timings are observations
outside scientific identity. Diagnostic API extraction is the remaining current
presentation-boundary issue identified above.

## Capability ownership and executable evidence

| Capability | Existing owner and checked entry points | Executable evidence / scope |
|---|---|---|
| Graphs, topology, routing | FeynKit `FeynmanDiagram`, Linnet `HedgeGraph`, native loop basis | `native_input`; graph identity and routing tests passed |
| Tensor contraction | Idenso `SymbolicTensor::infer/simplify_algebra/to_dots`, Spenso scalar structure | Native numerator and projector tests; no custom contraction engine |
| Covariant tensor reduction | `feynkit-tensor::TensorReducer`, `FeynmanDiagramTensorExt`, native Weingarten engine | Public API, implementation, and native tests inspected; FastSecDec does not reimplement this operation |
| Denominator basis/partial fractions | Native `IntegralFamily::complete/rewrite_numerator/partial_fraction` | Existing family completeness/ISP Rust probe passed; any future reduction must reuse these |
| Parameter-space polynomial numerator moments | Native Symanzik sources plus Symbolica derivatives | Eight Gaussian moment tests and separate peer audit; this adapter is not another covariant tensor reducer |
| Polynomial collection/valuation, differentiation, series | Symbolica native sparse polynomials, `mul_exp`, `derivative`, `series` | Four `ecosystem_probes` including signed valuations passed; mapper follow-up above |
| Complete-density symmetry | Graphica canonization and exact native literal permutation verification | Seven author tests plus three independent integral identities passed |
| Exact geometry arithmetic | Numerica integers/rationals/matrices | 11 author and eight independent geometry tests; sector-specific fan orchestration remains FastSecDec |
| Point generation and QMC statistics | Numerica's new separate Havana QMC lane | Standalone QMC/MC regression suites and peer review; no CBC search or owned pool |
| Adaptive ordinary MC | Existing Numerica Havana grid/RNG/accumulators | Seven runtime MC tests, frozen-production/pilot separation |
| Scalar/complex evaluators and precision | Symbolica SymJIT O2, native error floats, Numerica MPFR | Native evaluator/complex worker/artifact regressions; no custom special functions |
| Symbolic antiderivatives if later needed | Existing Rust `symbolica-integrate` | Owner recorded in plan; no general antiderivative engine introduced in this phase |
| One-loop scalar masters | Separate Rust `oneloop` crate used by community HEPKit | API/source/normalization checked below; direct cross-check suite being added |
| One-loop integral reduction | `one-loop-reduce::reduce_family(&IntegralFamily, powers, numerator)` | Native family API and recurrence source inspected; no FastSecDec reduction engine needed |

The broad workspace gate passed 133 tests. That does not substitute for a new
probe when adding a capability absent from the current tests. In particular,
new master evaluations and the stale-cache regression have their own pending
execution evidence.

## One-loop reference path and normalization

At the pinned revision, `feynkit-tensor` provides covariant tensor reduction,
not A0/B0/C0/D0 scalar masters. Community HEPKit's `src/oneloop.rs` composes two
existing Rust packages: `oneloop` and `one-loop-reduce`. Their exact community
lockfile revisions have been checked out under ignored worktrees:

- `oneloopmaster`: `a42a60aa5fe0b3ba0a5b9bb37a17c8465c06ba5a`.
- `one-loop-reduce`: `b53a70776a43bd14c6562c52a03bc4909568e473`.

The master API takes squared masses and squared renormalization scale, with
coefficients ordered `[finite, simple pole, double pole]`. The reduction API
accepts the native family directly, validates one loop and a symbolic dimension,
and returns exact coefficients with native master arguments. The community's
coefficient-combination adapter rejects coefficient poles requiring unavailable
positive-order master expansions. Tests must respect that limit.

Use the pure Rust master library as a development-only reference dependency;
do not copy formulas or invoke Python/Fortran in product tests. At this pin,
`default-features=false` selects the existing Symbolica expression backend.
The separately named `Native` backend requires `generated-evaluators`; disabling
that feature is not a request to implement a new evaluator.

FastSecDec's normalized Minkowski measure needs the explicit one-loop multiplier

```
(mu_squared)^eps * Gamma(1-2*eps) /
    (Gamma(1-eps)^2 * Gamma(1+eps))
```

before comparing with these masters. The source of the convention is the
original OneLOop normalization comment and the Rust triangle/box normalization
functions, which add `pi^2/12` times the double-pole coefficient to the finite
term. Expanding the multiplier gives
`1+EulerGamma*eps+(EulerGamma^2/2+pi^2/12)*eps^2` at unit scale, consistent with
that shift. A pole-only comparison cannot validate this normalization; include
finite coefficients of a divergent triangle or box, and a massive tadpole or
bubble with a nontrivial scale. The native input owner is implementing these
cross-checks, while preserving the independent analytic regression tests.

## Next milestone checklist

1. Reproduce and fix inline override cache invalidation with a derived parameter
   and cached coupling; compare separate card and inline override behavior.
2. Extract reusable diagnostics with bounded typed coverage, incremental results,
   partial cancellation, and a thin CLI adapter. Test missing-face intersections
   independently of the enumeration implementation.
3. Preserve compact regular mapped factors using existing native primitives;
   rerun numerator, subtraction, independent generation, and hard-fixture checks.
4. Run native one-loop master cross-checks with documented normalization, order
   reversal, squared masses/scales, and Euclidean kinematics. Review their source
   and results independently; keep reduction reuse recorded for numerator checks.
5. Keep one resolved identity each for Symbolica, Numerica, Graphica and FeynKit
   across direct/transitive dependencies. Recheck Cargo's duplicate tree after
   new development dependencies; test artifacts in a fresh process.
6. At the later bridge milestone, add selected-region-safe native wrapper
   accessors upstream, compile a direct Rust-to-Py-wrapper probe, and mirror
   existing status/error types. Do not move algorithms or pyo3 into FastSecDec.
7. Repeat this API/source/probe audit at each major subsystem milestone, including
   any new polynomial, tensor, reduction, graph, integrator, or precision helper.

## Resolution evidence

- The input owner fixed cached dependencies by retaining the native analytic
  expressions for internal parameters and couplings. Explicit internal
  restriction-card values remain authoritative, followed by exact inline Atom
  overrides. This composes native model/card semantics and native substitution;
  the existing native `ModelEvaluator` returns numeric values and cannot replace
  this exact symbolic binding path. Four focused load cases passed: cached
  parameter/coupling updates, exact pi input, preserved internal restriction,
  and an explicit inline internal override. The same test preserves an
  expressionless internal constant. The reviewer checked this final source and
  test independently; P1 is resolved and passed the subsequent focused CLI gate.
- Three `hepkit_one_loop` tests passed across eleven native B0/C0/D0 points,
  including massless/massive cases, a scaleless bubble, unequal spacelike channels,
  nontrivial scale, and finite coefficients of double-pole cases. The reviewer
  checked the native graph kinematics, normalization multiplier, coefficient
  ordering, and explicit Rust expression backend independently. An additional
  all-real output-layout assertion also passed the final workspace gate. No
  formulas or reduction implementation were copied into FastSecDec.
- The diagnostic API extraction is implemented in `fastsecdec::diagnostics`,
  with typed endpoint orientation, subset/side coverage counts, a hard point
  budget, streamed progress, and partial cancellation. Benchmark repetitions
  are individually retained and incomplete measurements are excluded from the
  median. The CLI now delegates to this API. This also fixes the previous
  benchmark JSON array being indexed with timing-object fields. Five focused
  API regressions and the fresh-process benchmark test passed. The coordinator
  independently reviewed subset/side traversal, bounded count arithmetic,
  strict interior validation, cancellation retention, and benchmark timing
  boundaries and found no issue. Even repetition counts use the documented
  upper median. Scan completion and numerical failures remain separate: the
  boundary CLI still exits unsuccessfully when any probe fails.
- The combined focused gate passed **46 tests**: diagnostics 5, QMC runtime 13,
  MC runtime 8, scientific examples 4, subtraction strategies 5, CLI unit 7,
  and CLI process 4. This includes new reciprocal-weight tests proving the
  exposed worker weight is applied exactly once, the three-axis complete-vector
  identity, and compact degree-10,000 regular factors. The numerical diagnostic
  fixture is symmetric in its coordinates: sector coordinate ordering is not
  an input-order contract. Its first asymmetric assertion was corrected only
  after inspecting the exact permutation matrix and generated coefficient.
- The mapped-factor owner replaced whole residual expansion by native signed
  polynomial valuation/`mul_exp` when needed, while zero-valuation factors retain
  their original factored Atom. The reviewed compact-factor and subtraction
  tests passed in the same gate. General forced expansion was not introduced.

The coordinator's final workspace gate passed **147 tests**, with four explicit
performance probes ignored. Formatting and all-target Clippy passed; only an
existing warning in the Symbolica dependency remains. The dependency inspection
confirms one identity each for Symbolica, Numerica, Graphica and the native
FeynKit crates, and no Python/pySecDec or terminal dependency in the production
library. The log is `output/hepkit-audit-workspace-tests.log`. Numerical parity
for all multiloop examples and matched performance remain separate open gates.
