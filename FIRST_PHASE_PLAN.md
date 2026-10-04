# FastSecDec first-phase implementation plan

## 1. Goal, governing decisions, and working method

**Goal statement:** Build a modular Rust implementation of FastSecDec for scalar, no-threshold Feynman integrals, using native HEPKit inputs and existing ecosystem functionality. Reproduce the scientific content of all FastSecDecPathFinder examples and relevant tests, independently implement sector decomposition, extend Numerica/Havana with caller-driven lattice QMC, provide a polished standalone CLI, and demonstrate correctness and performance parity under matched SymJIT O2 benchmarks.

**The governing criterion is a flexible, efficient design natural to Symbolica and HEPKit. There is no requirement to preserve pySecDec or FastSecDecPathFinder conventions, algorithms, options, sector numbering, serialization, or internal architecture.** Preserve scientific capabilities and end-to-end examples.

At implementation start:

1. Save this complete plan, including the verbatim prompts below, to `/common/dev/fastsecdec/FIRST_PHASE_PLAN.md`.
2. Set the goal statement above as the agent's active goal, with completion governed by this document's acceptance criteria.
3. Work directly on FastSecDec's local `main` branch. Commit the initial plan and repository scaffolding, then commit at each validated milestone.
4. Create `codex/havana-qmc` in the separate Numerica repository. Commit its changes there and record the corresponding revisions in FastSecDec.
5. Ignore the entire reference directory, build products, generated kernels, caches, and benchmark scratch data. Never commit reference checkouts into FastSecDec.
6. Push validated FastSecDec milestones on `main` to `https://github.com/alphal00p/fastSecDec`, as authorized on 2026-10-04. The subsequent authorization also permits publishing the finished Numerica QMC feature branch as a PR against its main branch and requesting review from `benruijl` (the user's corrected spelling); verify readiness and the target branch first. Other reference repositories remain local.

**Use extensive subagent delegation throughout implementation.** The primary agent acts mainly as coordinator: it owns interfaces, task allocation, sequencing, integration, acceptance evidence, and milestone commits. Delegate bounded slices for implementation, dependency research, mathematical audits, test migration, debugging, profiling, and performance optimization.

Use the available parallel slots continuously where useful. Give agents explicit ownership of files or modules and acceptance criteria. Assign important reviews to agents other than the implementation author. Integrate and validate each slice before dependent work proceeds; avoid concurrent edits to shared manifests and public interfaces.

Confirmed decisions:

| Topic | Decision |
|---|---|
| Inputs | Native HEPKit DOT is primary; direct parametric `U/F` input remains available |
| Compatibility | Scientific parity; no legacy interface or sector-ID compatibility requirement |
| Numerators | Scalar after native contractions/projectors; arbitrary polynomial loop-momentum rank |
| QMC | Worker-local point generation from published or caller-supplied vectors; no CBC construction in phase one |
| Scheduling | Democratic and adaptive QMC |
| Reference execution | Existing Python/pySecDec may run externally for development comparisons |
| Performance | Matched SymJIT O2; at most 5% timing regression per representative case |
| Platforms | Linux and macOS, x86-64 and ARM64 where supported by dependencies |
| Domain uncertainty | Reject inconclusive threshold checks by default; allow an explicit recorded no-threshold assertion |
| Python bridge | Deferred; its eventual implementation belongs entirely in symbolica-community |

## 2. Architecture, ecosystem reuse, and inputs

Use a Cargo workspace with three crates:

| Crate | Responsibility |
|---|---|
| `fastsecdec` | Public library; HEPKit ingestion, parametric construction, subtraction, kernels, integration sessions, persistence, status and results |
| `fastsecdec-sectors` | Exact sector geometry, coordinate maps, symmetry identification and decomposition progress |
| `fastsecdec-cli` | Configuration, commands, worker execution, terminal dashboard and reporting |

Organize each crate into small modules by responsibility. Keep graph ingestion, numerator conversion, domain analysis, subtraction, compilation, numerical rescue, scheduling, statistics, persistence, and presentation separate. Avoid large files that combine the pipeline.

The sector crate owns neutral domain/map types and does not depend on the main library. The main library reexports appropriate public types. CLI dependencies must not enter either numerical library.

### Reuse before implementation

Before implementing any computer-algebra or graph capability:

1. Search the relevant public Rust APIs and community bindings.
2. Inspect implementations, examples, and tests on the correct dependency revisions.
3. Build a focused Rust probe to determine whether existing APIs, composed together, provide the capability.

Record the evidence and the narrow missing operation. Implement new functionality only after these checks establish the gap.

Repeat a broad HEPKit integration and ecosystem-reuse audit at every major
subsystem milestone, before accepting a new dependency patch, and before final
phase-one acceptance. Assign the review to a subagent independent of the relevant
implementation. Review native graph, model, kinematics and Atom ownership;
public library interfaces; typed status, errors and persistence; caller-owned
execution; unnecessary serialization; and any duplicated graph, algebra or
numerical functionality. Include existing one-loop scalar masters and reduction
as numerical cross-check providers, without limiting the audit to those examples.
Require API, source/test and executable evidence for reuse decisions. Record
findings, fixes, justified gaps and the next audit boundary in `docs/reviews/`
and `docs/REUSE_AUDIT.md`. Resolve avoidable duplication and integration blockers
before accepting the affected milestone; do not claim future compatibility from
an unchecked interface alone.

Preserve factored Symbolica expressions throughout the pipeline. Avoid expansion
unless a specific operation requires it, and document those boundaries. Exact
support extraction may require polynomial coefficients; it does not justify
expanding the surrounding density, subtraction terms, Laurent templates, or
kernel expressions. Reuse native differentiation and series with opaque
subexpressions and late substitution where that avoids expression growth.

Use the following existing owners:

| Capability | Existing implementation to use |
|---|---|
| Physical graph and DOT interpretation | HEPKit/FeynKit `FeynmanDiagram` and `feynkit-graph` |
| DOT parsing and graph primitives | `linnet` |
| Models and parameter cards | `feynkit-model` |
| Routing, scalar products and integral families | `feynkit-kinematics` and graph APIs |
| Tensor, Lorentz, color and gamma algebra | Idenso, Spenso and applicable `feynkit-tensor` operations |
| `U/F` construction | `IntegralFamily::symanzik` |
| Numerator rewriting and denominator bases | Existing integral-family operations |
| Polynomial algebra, differentiation, series and exact linear algebra | Symbolica |
| Gamma-function Laurent expansions | Symbolica transcendental functions and series |
| Graph canonicalization | Existing Symbolica/Graphica APIs |
| JIT and batch evaluation | Symbolica's SymJIT interfaces |
| MC integration and numeric types | Numerica/Havana |
| Symbolic integration, if needed | Native Symbolica rational-polynomial integration; existing ecosystem integration APIs for other required classes |
| One-loop scalar master references and reduction | Existing native HEPKit/FeynKit integral APIs |

**Use Linnet directly.** `FeynmanDiagram` already owns a Linnet `HedgeGraph` and exposes its underlying graph. Reuse stable edge/vertex IDs, connectivity, bridges, cycles, subgraphs, spanning forests and traversal primitives. Do not create another physical graph representation, DOT parser, momentum-routing implementation, or graph canonicalizer.

Use local dependency worktrees at the revisions matching the inspected community environment:

- FeynKit: `8f834d9c62ae06fb327e4ef0b14abffda755b610`.
- Symbolica community: `98794d0d7337ba2b08e4c046dde584ad7fc1ce10`.
- Numerica QMC branch starting from `a8a8fcb8941752e265e3c4fa507c02a3f06bb70e`.

The supplied FeynKit and Symbolica directories currently have other branches checked out; prepare isolated worktrees without disturbing them. Root Cargo patches must resolve direct and transitive dependencies to one compatible Symbolica, Graphica, and Numerica instance.

The initial published Rust backend was `symjit = "=2.26.0"`; the current pin is
`=2.26.4`, verified against the live registry on 2026-10-04 before the requested
function-map experiments. Symbolica's pinned upstream revision includes the
latest published 3.0.1 release. Recheck published releases before subsequent
evaluator experiments and record the exact source revisions and local patches.
The supplied SymJIT checkout has different library packaging; that alone does
not justify patching it.

Disable Python features and unnecessary dependency defaults. Verify the production dependency graph excludes PyO3, Python bindings, pySecDec, and the full GammaLoop application. Existing numeric backends used by Symbolica remain dependencies; all new FastSecDec and QMC implementation code is Rust.

Keep dependency patches minimal and isolated: a demonstrated defect, focused regression test, small fix, and upstream-ready explanation. Numerica's requested QMC extension is the intentional larger exception.

### Native input boundary

Library callers supply native `FeynmanDiagram`, model, `Kinematics`, Symbolica atoms, and polynomials. Do not serialize these objects to strings and parse them again internally.

The CLI uses one TOML steering format referencing:

- Native HEPKit compact or stable DOT.
- Existing model JSON and native parameter-card JSON.
- Scalar-product declarations mapping directly to HEPKit `Kinematics`.
- Numerical parameter assignments.
- Expansion, generation, integration, and output settings.

Resolve paths relative to the run card. CLI overrides apply to steering settings. Preserve model-fingerprint checks. Use HEPKit's `P(i)`/`K(i)` symbols and routing conventions; do not require Pathfinder's additional propagator or loop-momentum annotations.

Reproduce examples using native HEPKit DOT, minimal model fixtures, and TOML run cards. Non-default propagator powers belong in integral settings keyed by stable internal edge IDs, without extending the DOT dialect.

The old DOT files are scientific references, not files to preserve unchanged.
Deliver a physically equivalent set using current native HEPKit conventions.
Validate every fixture by native parsing and export/reload; additionally verify
propagators, numerator routing, graph weights, and kinematics. Do not require the
old frontend's `mass`, `mom`, explicit propagator lists, or custom momentum lists.

Support:

- Standard quadratic scalar propagators with positive integer powers.
- Scalar polynomial numerators, including mixed-loop products, arbitrary rank, and epsilon dependence.
- Native tensor expressions that existing contractions and supplied projectors reduce to such scalars.
- Rational coefficients in dimension and external parameters.
- Gram-degenerate external kinematics when scalar processing does not require Gram inversion.

Reject unresolved free indices, unsupported loop-dependent denominators, custom nonquadratic propagators, finite widths, and complex masses in phase one.

Define the default measure explicitly:

\[
I=C_{\mathrm{graph}}\int\prod_{\ell=1}^{L}
\frac{d^Dk_\ell}{i\pi^{D/2}}\,
\frac{N}{\prod_j(q_j^2-m_j^2+i0)^{\nu_j}},
\qquad D=4-2\epsilon.
\]

An explicit symbolic measure multiplier defaults to one. Add no implicit scale, Euler-gamma, or \(4\pi\) factors. Preserve and apply native numerator, numerator-prefactor, projector, and overall-factor semantics exactly once. The diagnostic graph symmetry factor must not introduce another division.

Direct parametric input specifies its expressions, powers, prefactor, and domain explicitly:

- `ProjectiveSimplex`
- `UnitCube`
- `PositiveOrthant`

The hard four-loop reference is a positive-orthant integral, not an original unit-cube integral.

## 3. Symbolic and numerical generation pipeline

The pipeline is:

**Native input → normalized parametric expression → domain assessment → sector maps → endpoint subtraction and Laurent expansion → complete direct evaluators → integration.**

Preserve Symbolica expressions in factored form where possible. Expand only when required for polynomial supports, coefficient extraction, or another specific operation.

### Parametric construction

Reuse HEPKit's propagator families and Symanzik construction. Build measure factors and regulator-dependent powers explicitly around those APIs.

The principal missing physics operation identified by inspection is general polynomial-numerator conversion to Feynman parameters. After the required executable reuse probe, implement only that operation using Gaussian generating functions/moments and existing Symbolica arithmetic, differentiation, and matrices.

Do not make external Gram inversion a compulsory stage. Validate numerator conversion against propagator-cancellation identities, routing changes, odd/even moments, and independent analytic examples.

Compute intermediate expansion depth from the requested final Laurent orders and the poles of every prefactor. Convolve prefactors and coefficients exactly, including covariance propagation after numerical integration.

### Native sector decomposition

Implement one exact geometric engine based on Newton polytopes and normal fans. This gives a finite decomposition strategy without reproducing pySecDec's collection of methods. The mathematical basis is the [Kaneko–Ueda geometric construction](https://arxiv.org/abs/0908.2897).

- Map projective input through primary projective charts.
- For unit-cube inputs, restrict the normal fan to the nonnegative logarithmic domain.
- For positive-orthant inputs, retain the full normal fan, including maps resolving infinity.
- Obtain sparse supports through Symbolica; construct Minkowski support sums without expanding symbolic products.
- Implement the missing sector-specific exact cone enumeration using double description and deterministic pulling triangulation.
- Reuse Symbolica integers, rationals, determinants, ranks, and solves throughout.
- Represent each monomial map by its signed integer exponent matrix and exact determinant/Jacobian.
- Extract endpoint valuations and retain residual expressions, numerator information, and provenance separately.
- Identify equivalent complete sector integrands through existing graph canonicalization. Include numerator and exponent data; merge only verified equivalents.

Use deterministic content-based sector identities for diagnostics and caching. No existing `PSD` numbering is contractual.

Cache geometry by domain and canonical support. Parallelize independent charts and cone work. Provide progress, cancellation, and resource-limit diagnostics: formal termination does not guarantee acceptable memory or speed.

Geometry failures must never silently become zero integrals. Use existing scalelessness certificates when available; otherwise report the unresolved condition.

### Direct subtraction and evaluators

Implement endpoint Taylor subtraction, analytic endpoint terms, and the integration-by-parts reductions needed by the examples. Use Symbolica for the underlying algebra and series.

Substitute coordinate changes directly into complete Laurent-coefficient expressions. Compile a multi-output evaluator per sector or verified equivalent group. Preserve cancellations within each complete coefficient; do not integrate projector pieces independently.

Default compilation explicitly selects **SymJIT O2**. Retain an interpreted evaluator and arbitrary-precision evaluation for diagnostics and rescue. Validate JIT results against them at interior and near-boundary points before trusting the compiled path.

Also investigate representing coordinate substitutions with Symbolica's native
evaluator function maps and aliases, comparing inlined and non-inlined forms.
Measure symbolic generation, evaluator construction/compilation, expression/IR
size, and numerical runtime, including precision rescue and portable reload.
Preserve the same complete direct Laurent coefficients and Jacobians. Choose
the representation from measured correctness and performance; do not assume that
the smallest symbolic expression gives the fastest numerical evaluator. Verify
the latest published Symbolica and SymJIT releases before this work and retain
exact dependency identities in its evidence.

When generation or evaluation performance is uncertain, dissect the reference's
**direct integrand** implementation first: its subtraction and IBP formulas,
symbolic expansion order, reuse and caching, and compiled evaluator layout.
Use those findings to guide an efficient Rust design without retaining Python,
pySecDec, or unnecessary architecture conventions. The single dual evaluation of
U and F is explicitly out of scope: the user found that runtime strategy too slow.

Fuse periodization into the direct kernel when beneficial, and verify it against the unfused transformation. Whole zero-dimensional sectors are evaluated analytically.

Precision handling must detect nonfinite values, unstable endpoint evaluations, and suspicious weights. Retry at higher precision using existing numeric facilities. Never silently discard failed samples or substitute zero.

### Domain assessment and phase-two readiness

Use inexpensive sufficient sign/domain checks and runtime diagnostics. Distinguish:

- Certified no-threshold input.
- Explicit user assertion with inconclusive certification.
- Detected unsupported threshold behavior.

A user assertion is recorded in artifacts and results and does not suppress detected violations.

Keep input domains, coordinate maps, Jacobians, branch/phase metadata, and numerical evaluation separate. Retain general Symbolica map expressions at the domain boundary, even though phase-one sector maps are monomial.

This permits later domain splitting and complex contour maps without redesigning subtraction, kernels, or integration. Implement only the no-threshold strategy now. Future references are [Numerical Loop-Tree Duality: contour deformation and subtraction](https://arxiv.org/abs/1912.09291) and the [GCAD-based no-deformation approach](https://arxiv.org/abs/2603.05444).

## 4. Numerica QMC, CLI, status, and persistence

### Numerica/Havana QMC lane

Add `numerica::numerical_integration::qmc` as a separate library lane. Preserve existing MC interfaces and behavior; do not add QMC samples to accumulators that assume independent Monte Carlo points.

Expose distinct types for:

- `Rank1Rule`: modulus, vector, dimension, provenance.
- Randomization plans: seed and stable shift identities.
- Work packages: immutable rule/randomization references and point ranges.
- Worker-local point generation into reusable buffers.
- Partial results: per-shift counts and vector sums.
- Accumulators and estimates: complete replicate means, covariance, errors, and completion state.

Numerica owns neither the integration loop nor a thread pool. Callers choose workers, dispatch packages, evaluate integrands, merge results, and stop.

Generate points by index inside workers with exact modular arithmetic. Do not materialize full lattices centrally. Keep point and shift identities independent of worker count and completion order. Use canonical work packages and ordered compensated reduction for reproducibility.

Bundle the Kuo extensible generating-vector data distributed with QMCPy, with its license, attribution, source revision, construction range, and checksum. The author documents these as randomly shifted rank-one rules with power-of-two sizes. Production requires only numeric data and Rust code. [Author's catalog](https://web.maths.unsw.edu.au/~fkuo/lattice/), [QMCPy distribution](https://github.com/QMCSoftware/qmcpy).

Default to 4,096 points, 64 shifts, and Korobov exponent 3. The bundled rule supports refinement through \(2^{20}\) points; further automatic refinement adds independent shifts. Caller-supplied vectors support other moduli and larger rules. Reject unsupported explicit size requests clearly.

Estimate uncertainty from completed randomized lattice means, never from individual correlated points. Track covariance among Laurent coefficients. Reject duplicate or overlapping contributions, and retain unfinished work for resumption.

### FastSecDec integration sessions

Provide caller-driven integration sessions with operations equivalent to `next_work`, `submit`, `estimate`, and `checkpoint`. CLI code supplies execution and parallelization.

- **Democratic QMC:** evaluate all sectors with shared shifts; sum sectors within each shift before estimating total uncertainty.
- **Adaptive QMC:** use a separate pilot to estimate sector cost and variance; freeze each production allocation, use independent sector randomizations, and propagate independent sector covariance correctly. Do not treat pilot observations or unequal lattice stages as interchangeable production replicas.
- **Havana MC:** reuse current grids and adaptation. Require meaningful sector coverage before certifying errors or allowing accuracy-based stopping.

Integrate complete Laurent vectors over complete sector support. Preserve real/imaginary covariance where applicable. Stop only on completed, statistically valid estimates satisfying the requested tolerances or explicit work/time limits.

### CLI and status interfaces

Provide a compact command set:

- `generate`
- `integrate`
- `run`
- `inspect`
- `benchmark`
- `check-boundaries`

Use `clap`, `ratatui`, terminal support, and table rendering in the CLI crate. Avoid legacy backend-selection switches.

Core status objects contain structured data without ANSI escapes or terminal dependencies. Include generation progress, stage timings, sector counts, kernel counts, integration iterations, Laurent estimates, uncertainty validity, coverage, rule sizes, precision rescues, throughput, and stopping reasons.

Expose an event/observer interface and coherent snapshots. Each completed generation unit and integration iteration updates state. Render at a bounded rate so presentation does not become a bottleneck.

Interactive terminals receive a live dashboard with clear colored tables, progress, numerical results, and diagnostics. Noninteractive execution receives stable plain output or JSON events. Respect monochrome output, terminal resizing, and cancellation. Restore the terminal reliably on errors and interruption.

### Artifacts and checkpoints

Persist versioned manifests, symbolic/evaluator data, dependency identities, numerical conventions, compiler settings, and input hashes.

**O2 alone does not make machine code portable.** Portable artifacts retain expressions/evaluator IR and rebuild JIT kernels for the current architecture. Machine-code caches are optional and target-specific.

Checkpoint integration settings, rule and shift identities, completed ranges, statistical state, and precision diagnostics. Write atomically. Resumption must reject incompatible inputs or settings and must not duplicate work.

## 5. Milestones, audits, and acceptance criteria

Commit each milestone after its implementation and independent review pass. Record evidence and remaining work in the saved plan.

| Milestone | Deliverable and required gate |
|---|---|
| 0. Foundation | Saved plan and active goal; ignored references; modular workspace; reproducible Rust environment; pinned dependency identities; subagent ownership |
| 1. Reuse and baseline | Executable reuse probes; full test/example inventory; frozen external reference results and matched O2 benchmark protocol |
| 2. Native input and parametric construction | HEPKit/Linnet ingestion; model/kinematics adapters; `U/F` reuse; numerator conversion; independent normalization and graph-factor audit |
| 3. Native sectors | Exact geometry, maps, Jacobians, symmetry reduction; analytic domain checks and early hard-example generation benchmark |
| 4. Direct kernels | Subtraction, Laurent expansion, prefactor convolution, O2 kernels and precision rescue; independent mathematical and endpoint audit |
| 5. Numerica QMC | Worker-local generation, randomization, reduction, covariance, serialization; unchanged Havana MC tests |
| 6. Integrated application | MC and both QMC schedulers, checkpoint/resume, typed status, polished CLI dashboard |
| 7. Scientific parity | All example families and relevant regression behaviors reproduced in Rust |
| 8. Performance and completion | Profiling and optimization; independent final audit; documented platform evidence; final milestone commits |

Numerica work can proceed alongside symbolic and sector work once interfaces are agreed. UI work can proceed against recorded status events. Keep mathematical reviews and performance work as explicit agent assignments, not informal final checks.

### Scientific tests

Build a traceability matrix for the 182 inspected Pathfinder test functions and all examples. Port scientific requirements; document intentional retirement of tests tied solely to removed Python, pySecDec, backend, or serialization conventions.

Reproduce the scientific content of the 15 DOT examples, 11 kinematics fixtures, 17 run cards, stored targets, and hard-polynomial report under `/common/dev/fastsecdec/examples`.

Required coverage includes:

- Massive/massless triangles and boxes; double and triple boxes.
- Off-shell and rank-one/rank-two/high-rank numerators.
- Two-loop kite, three-loop self-energy, and multiloop three-point examples.
- Direct `U/F` double box, hard four-loop positive-orthant integral, and `issue_1` regressions.
- Native graph weights, numerator prefactors and projectors applied exactly once.
- Routing invariance, raised powers, propagator cancellations and Gram-degenerate scalar inputs.
- Exact map/Jacobian identities, cone coverage, non-simplicial geometry, and infinity charts.
- Analytic cube, projective, and positive-orthant integrals.
- Endpoint subtraction, prefactor poles, requested expansion depth, and zero-dimensional sectors.
- Eager/JIT/high-precision agreement, including the reference's documented double-box JIT concern.
- Worker partition invariance, covariance-sensitive cancellations, incomplete shifts, interruption, and resumption.
- Statistical coverage across independent seeds and unchanged existing Havana behavior.
- Dashboard rendering, plain/JSON output, terminal cleanup, and structured error reporting.

Normal tests and builds must run without Python, pySecDec, FORM, Normaliz, or a reference installation. External reference execution is confined to development comparisons; saved numerical fixtures carry provenance.

### Performance acceptance

Benchmark on the same host with matched O2, worker count, numerical precision, transforms, lattice rules, shift counts, coefficient orders, and statistical targets.

Measure separately:

- Cold generation and compilation.
- Warm artifact loading.
- Complete-vector kernel and integration throughput.
- Fixed-work integration.
- Time to verified accuracy.
- Peak memory, point-generation cost, reduction cost, and rescue rates.

Use at least seven paired repetitions for ordinary cases and three for expensive cases. Alternate execution order. Require timing medians within the agreed **5% band per representative case**; investigate and resolve larger regressions.

Because sector partitions may differ, scientific time to accuracy and total work take precedence over misleading sector-by-sector timing comparisons. Use identical points for evaluator comparisons wherever expressions can be matched. Use Pathfinder's corresponding full-support QMC configuration for matched estimator comparisons.

Representative gates cover small one-loop cases, double box, a numerator-heavy multiloop case, and the hard four-loop input. Fast results with inaccurate estimates or underestimated errors fail.

Provide Linux/macOS target configuration and testing where hosts are available. Clearly distinguish executed tests from unverified targets. Do not claim performance parity or platform validation without evidence.

The goal is complete only when the scientific suite, required numerical audits, dependency separation, example delivery, and performance gates are satisfied. Partial implementations and unresolved benchmark regressions remain unfinished work.

## 6. Verbatim original prompt and subsequent requirements

The following original prompt must remain verbatim in `FIRST_PHASE_PLAN.md`:

````text
So our goal now is essentially to re-implement the path-finder experimental fastsecdec pathfinder code hosted at:


```
/DO_NOT_PUSH_FOR_REFERENCE_ONLY/FastSecDecPathFinder
```



however with many important differences which I will come to list here.



a) Contrary to `FastSecDecPathFinder` the \*entire\* implementation should be in rust, nothing in python. The pyo3 python API will live entirely in the HepKIT API bridge hosted in `./DO_NOT_PUSH_FOR_REFERENCE_ONLY/symbolica-community`

At a later point you'll then be able to add a feature branch there to develop the python API connecting to fastSecDec.



b) Note that we still want a CLI for `FastSecDec` allowing for use and tests independently of HEPKit. There, the conventions can be similar to `FasSecDecPathFinder` except that many options should be removed (as we should now only aim at the "direct" implementation of sector integrand with change of variables substituted in directly). Also the input format should really be centered around the HEPKit Graph dot format, meaning in particular that any numerator conversion needed should happen without assuming any difference in the current way mult-iloop graphs are being supplied.



c) There should be *NO* dependency on pySecDec anymore, meaning that you must independently re-implement the sector identification tools used by pySecDec in fastsecdec.



d) In general, \*ONLY\* implement a compute-algebra-type-of-feature in fastsecdec if you have triple-checked that this capability is not already available in symbolica-community or hepkit. Only then can you implement your own version.



e) You can find in `./DO_NOT_PUSH_FOR_REFERENCE_ONLY` a lot of material which serves as useful look-up or, for the rust one, as dependencies you can link against directing in `Cargo.toml` so you can do local edits there when needed. However, in general and except for the lattice QMC implementation discussed later, you should keep all you implementation in fastsecdec and not in any dependencies. However, if there is a bug or a severe limitation, you can apply a local patch to those dependencies which I can escalate to the author (who are close collaborators of mine) and which will be implemented upstream quickly then.

But try to keep these patches short and to a minimum with just what is essential for proper working of your implementation of `FastSecDec`.



f) As said before, you have to re-implement from scratch a quasi monte-carlo integrator, and you should do this in a feature branch of `numerica` (also in the references provided to you). It should be seamlessly integrated within `Havana` without disturbing its current Monte-Carlo capabilities but extending it with a separate lane implementing the lattices needed for quasi monte-carlo. Make it possible to let the lattice generation take place *by the parallel worker* as it may be a significant workload for faster integrands.

As for normal havana, the qMC capabilities should remain a library, i.e. the user is the one steering the integration loop and the parallelization.



g) The CLI should deliver exceptionally clean visuals during integration and generation, using beautiful coloured tabled views, with similar data as already shown in `fastSecDecPathFinder`. Also keep in mind that for future implementation of the pyo3 python API in HEPKit, it's nice if all the information you show through the CLI is abstracted behind clear status structs with display functions, so that the port to a python API can happen later seamlessly by mirroring these structures in python.

In particular during generation, I would be nice to have a `ratatui`-like dashboard streamed environment which updates after each iteration with live results and crystal clean display. Once again, each stream update should be properly abstracted so that later it's easy to integrate in HEPkit.



h) There are a bunch of tests in `fastSecDecPathFinder` and examples (which you should reproduce in an  `./examples` folder) which you should all have available in the end in your implementation of `fastsecdec`. In particular performances *must* be on par or equal to what you get with `fastSecDecPathFinder`. Compiled kernels by default should use `symJIT` O2 (so that the output remains portable).



i) You are starting the implementation of this project from scratch (although you have many references to draw directions from), so you must spend considerable amount of time studying the structure you need with a clean design, many submodules (or even subcrates) and not a large flat collection of large files. This will be a large project and I need you to spend considerable time and effort on it.



j) You should for now focus on this first phase-only where you will cover "no-threshold"/"euclidean-like" integrals (same as what fastSecDecPathFinder) currently supports. However, once you'll have completed this first phase, we'll proceed to plan for the second one which will support an advanced contour-deformation idea (similar to `./DO_NOT_PUSH_FOR_REFERENCE_ONLY/literature/1771932.pdf`, but applied to feynman integrals) and also a no-contour deformation which will use a Generalized Cylidrincal Algebraic Decomposition (GCAD) to avoid contour deformation, how it is described in \`./DO_NOT_PUSH_FOR_REFERENCE_ONLY/literature/2603_05444.pdf\`. Although you should not implement any of this yet, the structure of your implementation must keep in mind that this will need to be implemented next, so it should be already ready to welcome it.



Write a plan for the above, which *must include verbatim this prompt in it*. Also add a goal statement in it and the instruction to write it in `FIRST_PHASE_PLAN.md` and set it to yourself as a goal upon starting its implementation. You should also work within the main branch of the local repository and periodically commit to it at various key implementation steps/milestone. I'll give you a remote to push into at a later point.



Ask any question you may have to make sure the path is fully clear and devoid of any ambiguity.
````

Subsequent requirements, also preserved verbatim:

```text
Yes, it's important that you specify in your plan that you DO NOT NEED TO stick to any existing convention of pySecDec or FastSecDecPathFinder, you only want to be able to achieve the same end-to-end examples, but you should chose the design that you find most flexible, appropriate/natural for Symbolic and our design and also most efficient. That is the most important criterion.
```

```text
And with inputs most in-line with the library environment and existing example in HepKit, although for now you focus only on a CLI for steering examples and tests.
```

```text
Also make sure that you link against existing crates of the HEPKit ecosystem to borrow functionality that are needed, In particular linnet will be needed to implement a lot of graph functionalities and primitives that you'll need. Same for the Graph dot objects that you parse where HEPKit toolset will provide many primitives you need already.
```

```text
And include in the plan that you should use many subagents to organise the various implementation slices, their audit, the research, the debug, the performance hunt, etc... you will be acting mostly as a coordinator only!
```

```text
Continue as planned, but note that the `All 15 historical DOT` must not remain identical, you should instead start from a phyically equivalent set of dot file fully compliant with the new HEPkit standards.
```

```text
(And when in doubt of how to achieve good performance, dissect how `FastSecDecPathFinder` achieved its work; we don't want the same structure necessarily, and we want it fully in rust and with no pySecDec dependencies, but its innerworking should give a good close-to-optimal implementation plan, although you can ignore building the sector integrands from a single dual evaluation of the U and F polynomials, this idea proved to be too slow at runtime).
```

```text
(Continue as planned, but indeed avoiding expansions unless absolutely necessary is key for a powerful use of Symbolica (which unlike FORM does not need to ever expand))
```

```text
Continue as planned, but periodically run an audit verifying that your implementation is perfectly setup for a future integration in HEPKit and also uses HEPKit primitives wherever it can, without re-implementing its own versions (e.g. for numerical test/cross-checks the one-loop master scalar integrals are already available there, and even on-loop reduction if need be; that's just an example, your auditing subagents should review that aspect broadly).
```

```text
Ok, continue as planned, but probably a good point at which you can now commit+push fastSecDec to:
[https://github.com/alphal00p/fastSecDec](https://github.com/alphal00p/fastSecDec)
and periodically push there when you reach milestones.
```

```text
(continue as planned, but don't forget that one potential useful way to get a small expression for each evaluator is to encode the change of variables as a function map int he evaluator builder arguments, and play with the possibility of inlining aliases or not (though this need generation and runtime generation check)). Make sure you are on the latest symbolica and SymJIT realeases when doing all that.
```

```text
(continue as planned, but if you already have a final version of the lattice QMC version for numerica, you can open a PR for merging into its main and tag BenRuijl as a reviewer)
```

```text
no the reviewer you must ask for is `benruijl`
```

## Implementation record

- 2026-10-04: bounded orthant and projected-family evidence milestone: issue-1 completes all 2,686,976 evaluations and orders `[0,1,2]`; the hard four-loop density completes generation/O2 compilation in 36.755 process seconds and all 5,726,208 evaluations of 699 kernels in 124.493 process seconds, preserving orders `[-2,-1,0]`. Both retain complete covariance, zero evaluation failures, work-limit stopping and unverified results. The hard case's 2760 physical F-support charts differ from the earlier combined-U/F geometry stress workload; no speedup is inferred from their counts or timings. Independent saved-result/coverage audit passes. Native partial fractions and sector projection also complete both off-shell triple-box vectors with eight active propagators and seven integration dimensions; exact native denominator equality supports equivalence, while first numerical errors remain preliminary and potentially correlated with original trials. No new production algebra or automatic graph rewrite is introduced. The bounded on-shell failure is retained, with native Laurent capture/replay investigating it. Independent difficult references, prespecified convergence, matched performance and platform evidence remain open. See the hard-four-loop, on-shell/issue-1 and projected-family reviews in `docs/reviews/`.

- 2026-10-04: caller-status cadence milestone: **33 focused CLI tests passed** against the 264-test workspace baseline, followed by formatting and all-target Clippy; independent review passes. A numeric-only profile of the complete triple-box checkpoints measured native snapshots at about 2.2 ms, versus about 0.2 ms for JSON serialization; each original diagnostic had emitted roughly 806 MB of status. The CLI now checks its presentation deadline before requesting a snapshot, with JSON default 100 ms, existing terminal/plain cadences, and `--status-interval-ms 0` for every batch. Initial/stage/round/final/cancel/failure events are forced; package errors, accepted replay, cancellation and checkpoint checks remain per batch. Statistical-range errors discovered only by reduction can surface at the next observation or forced boundary, and failure-save tests cover that explicit latency change. Native estimators and checkpoint formats are unchanged. The first on-shell triple-box generation timed out after 300 seconds plus cancellation grace during Laurent extraction; it produced no artifact or usable partial integral. Native template replay and reduced-family diagnostics now target that bottleneck. Orthant completion, convergence and matched performance remain open; no end-to-end cadence speedup is claimed before the paired run.
- 2026-10-04: qualified-selection and native-family milestone: **264 workspace tests passed**, twelve explicit probes ignored; a final legacy-inspection test-only byte-slice adjustment passed its focused regression, followed by formatting and all-target Clippy. Native manifest projection preserves original kernel IDs, exact-offset policy, covariance and checkpoint scope, with selected-only worker contexts and explicit scoped completion. Artifact inspection exposes the retained owner's domain/chart/map/measure records and legacy absence. Independent HEPKit reviews pass. The new borrowed `IntegralFamily` entry delegates to the existing Gaussian implementation, validates positive powers and native label collisions, and rejects stale dimension-dependent physical coefficients; exact raised-power integration and weight-once proofs pass. Both original off-shell triple-box scalar and rank-two baselines complete all 9,682,944 evaluations and four Laurent orders without failures, but remain unconverged and independently uncertified. Native partial fractions and sector projection exactly identify eight active propagators, preserving two squared lines; the default graph path is unchanged pending a measured probe. On-shell/orthant examples, difficult-case calibration, projected-family/reporting performance and matched acceptance remain open. Production dependencies still retain one Symbolica 3.0.1/SymJIT 2.26.4 ownership chain without Python, pySecDec, CLI presentation or reference providers.
- 2026-10-04: the user corrected the requested reviewer to `benruijl`. Retrying that exact username still received GitHub's permissions denial. PR #8's description now contains the verified exact `@benruijl` mention, with the earlier casing removed; its formal review-request list remains empty.
- 2026-10-04: the user authorized publishing the completed Numerica lattice-QMC feature as a PR targeting its main branch, with `benruijl` requested as reviewer (corrected handle). Upstream readiness, branch ancestry and the complete native test suite are checked before publication; this does not authorize publication of other reference repositories.
- 2026-10-04: opened [Numerica PR #8](https://github.com/symbolica-dev/numerica/pull/8) against upstream `main` from the authenticated user's fork, at `e4638da22a17cfa931fa14c6829d3350b7a8de2b`. Full default, serde and alternative-backend suites passed 237, 240 and 217 tests respectively, plus the serial/threaded example, formatting and package-content checks. No existing MC algorithm changes or new runtime dependencies are included. GitHub denied formal reviewer assignment for insufficient permissions; the verified PR description now explicitly tags the corrected `@benruijl` handle. The CLA check was pending at publication; the feature has not been merged. See `docs/reviews/numerica-qmc-upstream-readiness.md`.
- 2026-10-04: the user supplied `https://github.com/alphal00p/fastSecDec` and authorized the initial push and periodic pushes at validated milestones. This supersedes the initial no-push restriction for FastSecDec only; excluded reference repositories and raw outputs remain local.
- 2026-10-04: implementation authorized; goal activated; plan saved. FastSecDec remains on `main`. Isolated pinned dependency worktrees prepared. Three agents assigned native input/dependency integration, exact sector geometry, and Numerica QMC. Acceptance gates remain pending.
- 2026-10-04: foundation milestone prepared: three-crate workspace and local Nix shell; native HEPKit/Linnet input and scalar U/F normalization; executable CAS/JIT reuse probes; exact normal-fan sector geometry. Native library tests passed (14); geometry author and independent tests passed (11 + 8). The hard nine-dimensional support probe produced 3,496 sectors in approximately 125 seconds in its initial mixed-optimization build. This is feasibility evidence, not performance acceptance. Full numerator conversion, symmetry reduction, subtraction, runtime and CLI remain in progress. See `docs/reviews/sector-geometry-initial.md` and `docs/REUSE_AUDIT.md`.
- 2026-10-04: Numerica QMC implementation committed independently as `e26d3dd3ee0683c5acd9706eb95fb6b66f24147b`, followed by peer-review fixes in `e9b7481d66b8f9d0c5e58ccabc4d6644e7fd479a` on `codex/havana-qmc`. The author reports 174 library, 14 API regression, 23 QMC and 22 documentation tests passing with serde. Independent review identified and drove corrections to large-offset covariance and maximum-modulus periodic shifts; the reviewer executed both reproductions against the fix and verified their expected results. No pushes or changes to existing MC interfaces.
- 2026-10-04: native fixtures now cover all 15 historical DOT examples, plus a bubble, with 19 graph TOML cards and three direct-polynomial cards. The two example-input tests pass, including exact numerator and propagator-multiset checks for all five numerator graphs. This validates native topology and routing, not numerical integration of every card. Historical targets retain provenance; the double-box file's manually supplied target has no usable reference uncertainty. The triangle-numerator card resolves the historical unused-leg inconsistency while preserving its actual explicit-propagator integral. See `examples/README.md`.
- 2026-10-04: direct-generation and standalone-runtime milestone: native Gaussian polynomial numerators, factored endpoint subtraction and full Laurent vectors, portable SymJIT O2 kernels with native MPFR rescue, caller-driven QMC/MC sessions, checkpoint validation, and CLI generation/integration/inspection/boundary diagnostics with typed status and a live terminal dashboard. The complete workspace gate passed 109 tests, with two expensive probes explicitly ignored. Native two-worker bubble, triangle and analytic endpoint runs, fresh-process complex/Gamma artifact reloads, and terminal cancellation/checkpoint restoration were exercised. Three small Symbolica fixes are recorded as upstream-ready patches; Numerica is pinned at `617f7a56f8f168cd7498177c7db4a40b098eb135`. There are now 23 run cards. Boundary assertions cannot override unresolved singular faces. Factored Laurent templates reduced the diagnostic double-box generation time from 100.309 to 26.211 seconds; these development-build observations do not establish performance parity. Complete-density symmetry, further cancellation/IBP and geometry optimization, all-example numerical certification, and matched performance acceptance remain pending.
- 2026-10-04: subsequent workspace gate passed 133 tests with four explicit performance probes ignored. It includes verified full-density Graphica symmetry, independent integral multiplicity checks, all 23 native run-card loads, native massless-box numerical validation, literal caller-symbol regressions, Taylor/IBP identities, per-axis cancellation metadata and typed stage timings. The release geometry stress case improved from 99.351 to 11.276 seconds with matching ordered-map fingerprints. Fresh factored double-box generation produced 102 representatives from 152 charts; Taylor was faster to generate and compile than IBP and remains the default. These are local diagnostic measurements, not matched performance acceptance. The recurring HEPKit audit identified a reproducible stale model-cache bug, CLI-only reusable diagnostics and remaining mapped-factor expansion; their fixes and native one-loop master cross-checks are in progress before accepting the next milestone. See `docs/reviews/hepkit-integration-audit-2026-10-04.md`.
- 2026-10-04: HEPKit reuse/diagnostics milestone validated: **147 workspace tests passed**, four performance probes intentionally ignored; formatting and all-target Clippy pass. The audit's model-cache, reusable-diagnostic and mapped-expansion findings are fixed. Three native master tests cover eleven one-loop points with exact normalization and scales. Boundary diagnostics now expose bounded typed face coverage, streamed results and cancellation; benchmarks preserve partial repetitions. Regular degree-10,000 factors stay compact, and a three-axis Taylor/IBP integral identity passes without forced expansion. Production dependencies still exclude Python, pySecDec and CLI presentation crates; the native master provider is development-only. Future wrapper accessors are documented for the bridge phase. Weighted-sample replay, complete multiloop/numerator/orthant numerical certification, and matched performance gates remain open.
- 2026-10-04: weighted-evaluation and retained-metadata milestone: **176 workspace tests passed**, six explicit scientific/performance probes ignored by the ordinary gate; formatting and all-target Clippy pass. Whole-vector MPFR replay now applies the sampling weight before binary64 conversion; caller-owned QMC/MC workers accept already-weighted vectors. Checkpoint version three preserves accepted replay state across worker-count changes. Portable kernel version two retains native coordinate maps, exact geometry, factor-level domain certificates, and the phase-one branch policy, with semantic load validation and explicit legacy handling. Independent HEPKit reviews drove zero-dimensional and symbol-collision fixes and removed quadratic chart association. Five native reducer tests cover eight numerator points, including rank five and zero Gram determinant, alongside the eleven scalar-master points. Numerica is pinned at `55072895f98be8830bcf6400e32546bc9470de7a`, with independently reviewed periodization range checks. A separately executed native rational-integration proof verifies the generated double-box leading pole integrates to zero; the sampled value is still reported unchanged. The 64-shift double-box convergence study, all-example numerical gates, downstream generation optimization, and matched performance acceptance remain open.
- 2026-10-04: reference-reporting and source-provenance milestone validated after the 176-test workspace baseline: nine library reference tests and 22 CLI tests passed, with final focused repeats after review fixes; formatting and all-target Clippy pass. Native callers can compare typed sparse Laurent/component results directly, with explicit exact/reported/unknown uncertainty and recorded normalization, kinematic and independence evidence. CLI targets remain outside numerical settings and artifact identity; changing a target clears inherited evidence, and native TOML fingerprints preserve non-reference value types. Independent reviews found and closed a missing untracked-dependency-source hash and absent-file Cargo watches. The double-box diagnostic completed 6,684,672 samples and 64 shifts without failures: its leading pole is −0.000181020268 ± 0.000588690140, consistent with the separate exact-zero proof. Higher coefficients and matched performance remain uncertified. The validated history has been published to the user-supplied FastSecDec remote; the next slices address factored monomial extraction, all-example multiloop checks and remaining regression gaps.
- 2026-10-04: latest-backend and factored-mapping milestone: **202 workspace tests passed**, eight explicit scientific/performance probes ignored; formatting and all-target Clippy pass. The live registry identifies Symbolica 3.0.1 and SymJIT 2.26.4; the existing Symbolica revision contains that release, and SymJIT is upgraded to 2.26.4. A two-line development-only OneLOop patch updates its exact backend pin and cache identity, with native cache rebuild/reload and old-header rejection passing. Native monomial collection preserves compact residuals, retaining exact support minima and the signed sparse fallback; independent review found no correctness or reuse defect. Four new tests close graph-entry timelike rejection, signed Gamma depth, negative highest Laurent order and complete native rank-two MPFR replay gaps. The rank-five master comparison passes with much smaller development-build symmetry/Laurent costs; controlled release and reference timings remain pending. The native FunctionMap audit and disconnected probe prepare the requested alias/inlining comparison; no production representation is selected yet. All-example multiloop, boundary-growth, saved-result reporting and matched performance gates remain open.
- 2026-10-04: boundary/contribution and evaluator-measurement milestone: **220 workspace tests passed**, ten explicit probes ignored; a subsequent malformed imported attempt-index display regression also passed. Formatting and all-target Clippy pass. Componentwise boundary growth, bounded retries, typed status and clean CLI tables preserve all attempts, selected-sector coverage and cancellation; independent HEPKit reviews found and resolved two public-formatting robustness issues. Native QMC/Havana contribution reports reuse existing complete-replica statistics, keep exact offsets separate and label shared-sector marginal errors without changing the authoritative total. All six smaller massive multiloop cards generated and integrated successfully; initial independent external references are recorded for all six. A six-line higher-work repeat agrees within one reported combined error but shows slow uncertainty reduction, now under investigation. The controlled same-release rank-five generation comparison improves from 95.328 to 12.066 seconds through factored mapping. Native FunctionMap compatibility passes 32 cases; 119 longer actual-coefficient cases confirm compact alias preparation, essentially unchanged batch throughput and slower retained small functions. Production evaluator representation remains unchanged. The next work includes QMC rule/convergence analysis, remaining scientific examples, native saved-result reporting, lazy support profiling and matched performance acceptance.
- 2026-10-04: six independent multiloop references are frozen in the native versioned format with source identities, explicit real projection and reported uncertainties. Three focused data/comparison checks passed, including the two explicitly invoked ignored record/replay checks; the six-case CLI harness compiles and targeted Clippy passes. A constant-integrand control reproduces the five-dimensional Kuo33002/Korobov3 error plateau. At equal work, alternative published vectors substantially improve the actual six-line result, while broader dimensional controls rule out selecting a universal default from that case alone. Production rule defaults remain unchanged pending further physical comparisons. Separate rank-five attribution finds repeated native support extraction accounts for 99.63% of mapping time; a lazy reuse implementation and its independent acceptance checks are in progress. This milestone adds scientific evidence and reference transport, not a convergence or matched-performance certificate.
- 2026-10-04: native support reuse and published-catalogue milestone: **231 workspace tests passed**, twelve explicit probes ignored; formatting and all-target Clippy pass. The production dependency audit retains one Symbolica 3.0.1/SymJIT 2.26.4 owner and excludes Python, pySecDec, CLI presentation and development-only reference providers. The generation-owned lazy support cache preserves compact factors and complete vectors: three alternating release pairs improve rank-five generation from 11.371 to 2.522 seconds, and an untimed independent pair has identical complete-vector fingerprints. Numerica is committed locally at `5d768eea73c525bb28affea31c298db9358b2f5f` with attributed HKKN/Kuo catalogue choices, native bounds and provenance validation; its 29 QMC and nine MC tests pass. Forty-eight physical comparisons retain native joint covariance and show case-dependent rule quality. The default remains unchanged. The CLI exposes explicit choices, reports the effective native design, and rejects mismatched native/outer checkpoint settings before work. Independent HEPKit audits pass, and the exact massive-box run-card point now agrees with native D0. Saved-result APIs, coupled numerator and remaining triple-box/orthant campaigns, difficult-case convergence and matched performance remain open.
- 2026-10-04: saved-result and coupled-numerator milestone: **253 workspace tests passed**, twelve explicit probes ignored; the final enum-layout adjustment passed eleven native result and five CLI result/reference tests, followed by formatting and all-target Clippy. Saved results preserve full covariance, scope, exact offsets, original references and effective QMC design without symbolic loading. Numerical failures retain accepted coverage and produce nonzero CLI exits; independent reviews closed coverage, metadata-allocation, numeric-range and rendering-cost issues. The minimal Numerica coverage accessor is committed at `e4638da22a17cfa931fa14c6829d3350b7a8de2b`, with 30 QMC and nine existing MC tests passing. The native coupled-sunset fixture passes exact routing/export-reload, nine independent density points, a convergent scalar sign control and complete Laurent vectors through epsilon one at two spacelike scales. All 24 run cards and the 17-DOT inventory pass CLI checks. Production dependencies retain Symbolica 3.0.1/SymJIT 2.26.4 and exclude Python, pySecDec and CLI/reference-provider dependencies. Qualified CLI sector selection, remaining triple-box/orthant examples, difficult-case convergence and matched performance remain open; Numerica upstream PR readiness is being checked under the new publication authorization.
