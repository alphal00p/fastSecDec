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

**Completion and stopping rule (user clarification, 2026-10-05):** Complete the
first-phase scientific capabilities and establish representative performance
parity, then mark the active goal complete and stop. Keep performance work
bounded to resolving acceptance gaps. Reuse valid measurements, and stop tuning
each case once it meets the agreed criterion. Do not add optional optimization
campaigns or pursue further speedups after acceptance. Threshold support and
its plan await the user's subsequent instruction.

**Reference-resource clarification (2026-10-05):** Do not keep pursuing
pySecDec/FORM cases that exceed this machine's reasonable resources. Preserve
their failures and use those providers only where they complete practically.
FastSecDec must still complete the required cases. FastSecDecPathFinder remains
the performance comparator, using its existing functional routes and explicit
versioned environments. Complete coverage and establish parity or better for
the required generation, highest-requested-order convergence and sector/sample
metrics, then commit, push, mark the goal complete and stop. An infeasible
pySecDec reference is not an additional completion gate.

**On-shell scope clarification (2026-10-05, supersedes earlier acceptance
requirements for this case):** The user accepted completed FastSecDec generation
of the original on-shell triple box as sufficient for phase A:

> The fact that you can now complete generation of the `on-shell triple-box` within fastSecDec is good enough for now, let's not benchmark performance of it too much and instead focus on the faster-to-generate simpler integrals.

Its full generation, portable artifact publication and cold loading are already
demonstrated. Preserve the partial numerical checkpoints and reference attempts,
but stop further integration, reference construction and performance campaigns
for this case. Full-integral numerical agreement and convergence remain explicitly
unverified, deferred rather than claimed successful. They no longer block phase-A
completion. Focus the remaining bounded comparisons on the existing faster
representatives, starting with triangle, box and double box, and retain the
existing numerator and hard-polynomial coverage without expanding the campaign.
The user also supplied an authorized Symbolica license for subsequent runs;
verify the existing eight-worker Pathfinder route and use matched eight-core
measurements when it succeeds. Keep the license outside tracked files and logs.

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

Complete the plan's capability and scientific coverage before the broad
optimization campaign. Performance work needed to make a required example
generate or evaluate successfully remains part of closing that capability gap.
Retain the current baselines and all unsuccessful experiments; do not replace
missing full-vector coverage with a favourable throughput measurement.

After coverage, assign focused generation, convergence and evaluator work only
where needed to resolve a remaining acceptance gap, with independent reviews.
Profile factored generation and compiler costs; compare published lattice rules
and periodizing transforms on frozen multi-seed workloads when convergence
requires it; and prioritize reducing both average and maximum observed sample
evaluation time where necessary for parity. Reuse accepted evidence and do not
broaden the representative set to pursue marginal improvements. Keep sector IDs, full coefficient
vectors, dimensions, sample/batch sizes, ordinary arithmetic and rescue counts,
precision, and point-generation/transform costs with those observations. Report
batch-amortized time separately from individually timed sample latency, quantify
instrumentation overhead, and retain tail observations rather than dropping
slow boundary samples. Use caller-owned structured diagnostics so the same
measurements can later be exposed through HEPKit. Distinguish a faster numerical
kernel from fewer samples needed for verified accuracy.

Benchmark on the same host with matched O2, worker count, numerical precision, transforms, lattice rules, shift counts, coefficient orders, and statistical targets.

Measure separately:

- Cold generation and compilation.
- Warm artifact loading.
- Complete-vector kernel and integration throughput.
- Fixed-work integration.
- Time to verified accuracy.
- Peak memory, point-generation cost, reduction cost, and rescue rates.

Report an explicit eight-physical-core time to one-per-mille estimated relative
uncertainty for the coefficient at the **largest signed requested epsilon
power**, together with independent reference checks and the complete vector.
The user clarified that this is what they meant by the hardest coefficient:
usually the finite part for a request through order zero, and the highest
positive order when requested. It is not the most singular pole. Record the first observed crossing,
actual work, seed/rule/transform and worker affinity; separate generation and
loading from integration. If the selected coefficient is known to vanish, use a
labelled absolute zero check rather than dividing by zero or a noisy near-zero
estimate; do not silently switch to another coefficient. Do not
infer eight-core timing by scaling a one- or two-worker measurement. Keep
per-sector average worker cost, maximum sector-average cost and instrumented
individual-sample maximum latency as distinct metrics.

Use at least seven paired repetitions for ordinary cases and three for expensive cases. Alternate execution order. Require timing medians within the agreed **5% band per representative case**; investigate and resolve larger regressions.

Because sector partitions may differ, scientific time to accuracy and total work take precedence over misleading sector-by-sector timing comparisons. Use identical points for evaluator comparisons wherever expressions can be matched. Use Pathfinder's normal boundary-support, optimized QMC configuration while retaining its complete physical coefficient vectors and correlated all-sector sum. Record actual evaluator dispatch; requesting optimized evaluators does not prove that a fused path ran. Earlier full-support, optimization-disabled runs remain explicitly labelled historical observations.

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

```text
Continue as planned, but once you have full capability coverage within goal plan, look into optimizing performance of both generation, lattice and transform choices for convergence, and evaluation time per sample and for each sectors (mostly focusing on getting max and average sample evaluation time per sector down).
```

```text
Continue as planned, but summarize where you're at regarding what's planned to being implemented for the plan, and what are your current performance measurement (generation time, convergence time to 1 per mil convergence for deepest pole on 8 cores, and max and avg run time per sample and sector)
```

```text
Continue as planned, but summarize where you're at regarding what's planned to being implemented for the plan, and what are your current performance measurement (generation time, convergence time to 1 per mil convergence for deepest pole on 8 cores, and max and avg run time per sample and sector). Organise the table showing your result putting them side-by-side with the same results from the FastSecDecPathFinder ref. code. Then continue with the implementation as planned.
```

```text
Can you build an MRE for this symbolica bug that I could escalate to the author, i.e. Ben Ruijl? Put it in a standalone folder in the workspace (and make it in the form of a standalone rustscript reproduction, with a README describing it and the patch for the fix you'd want). Delegate an agent to setting up this MRE and continue on your side as planned.
```

```text
Sorry when I meant the "deepest" I always mean the hardest to compute which is the one with the largest signed epsilon power (so often, the finite part), not the deepest pole of course, sorry.
```

```text
Continue as planned, but don't overdo it on the performance part, and once you're on par and feature-full regarding the objectives of the first phase of the active goal, declare it completed and stop yourself.
We'll then draft a plan for approaching the physical cases with threshold.
```

```text
Continue as planned, but don't push the pySecDec/FORM references where it can't complete on this machine. Only make sure the fastSecDec version completes there and only benchmark against cases that complete within reasonable resources in the original pySecDec. You should however always be able to benchmark against fastSecDecPathFinder.

And as I said, once you're feature complete within what's stated in the goal, and reached parity or better everywhere according to the metric I mentioned earlier vs fastSecDecPathFinder, then commit+push, set the goal as completed and stop yourself so that we can plan together for the next step.
```

## Implementation record

- 2026-10-05: the saved on-shell artifact resumes successfully with eight bounded contexts, reaches 397,312 evaluations with zero failures and peaks at 9.03 GiB. It is intentionally stopped after 1,305.505 seconds for a scheduling handoff, not by its deadline; its checkpoint is retained. Observed heterogeneous batches permit at most 32.09% worker-time utilization in the audited prefix. The CLI now refills free workers through the existing native task/submission/checkpoint APIs, retaining the context bound and coordinator-owned accepted replay. **20 focused driver tests**, formatting and scoped all-target Clippy pass; cancellation, in-flight checkpoint recovery, full covariance and panic draining are covered. An initial overlapping test abort is traced to Symbolica's occupied permit; a separate test-only failed-attempt count is corrected, with both records retained. The next same-allocation continuation has one 7,200-second bound. Pathfinder's supported guarded projector route meanwhile generates a complete 968-sector bundle through order zero in 39.637 seconds. A harness Gamma-metadata assumption is corrected against its native external-prefactor convention, enabling numerical reuse without regeneration. Neither bundle generation nor partial native work closes full-vector agreement or representative parity.

- 2026-10-05: the CLI now bounds QMC evaluator ownership to one active sector per worker and restores each sector's accepted native replay state on revisit. All **16 focused driver tests pass**, including full-vector/covariance equivalence with actual precision rescue, failed-prefix exclusion and eight-worker checkpoint continuation; formatting and scoped all-target Clippy pass. Native libraries and artifact/checkpoint formats are unchanged. The existing full on-shell artifact can therefore be resumed without repeating generation; its eight-worker, 1,800-second numerical continuation remains a separate measured capability gate. Pathfinder's cached attempt reaches its 1,800-second generation bound without producing a bundle or changing the 137 retained formula entries. Its processes are reaped and inputs verified; inspect the stall before another attempt. No infeasible pySecDec/FORM route is reopened.

- 2026-10-05: the committed persistence fix (`c3ec42a`) saves the entire original on-shell prepared graph successfully: 1,026 sectors and all orders −6 through 0, whole generation/compilation/publication **1,695.486 seconds**, peak **16,063,700 KiB**, artifact **957,122,802 bytes**. Separate-process cold inspection succeeds in **136.719 seconds**, peak **8,458,348 KiB**, with the same content identity and coverage. The subsequent original 180-second integration stage times out and is reaped after **186.784 seconds**, with 38,912/8,404,992 points in its last status, zero completed sectors and zero evaluation failures. It retains an accepted checkpoint but no result. All 71 frozen checks pass; artifact publication and reload are now established, while full-integral agreement and parity remain open. Continue numerically from the saved artifact using measured costs; do not repeat accepted generation. A fresh Pathfinder direct attempt resumes after the user pause with the same retained cache provenance. Final representative comparisons permit Pathfinder's actual default boundary/optimized QMC mode; earlier opt-out/full-support observations remain separately labelled evidence, not the final optimized-reference baseline.

- 2026-10-05: the persistence capacity fix passes **380 workspace tests**, zero failures and 23 explicit ignored probes across 63 summaries, plus formatting and all-target Clippy. Independent focused gates pass 16 native and 42 CLI tests. Borrowed native payloads and streaming hashing retain exact v3 bytes/IDs; cold loading avoids re-encoding an unused envelope. The CLI keeps embedded native JSON opaque, streams atomic output, retains legacy v1 identity/loading and explicitly writes a v2 outer identity. No algebra or dependency patch is added. These controls cover cold complete vectors, tamper rejection and interrupted writes; the bounded original full-graph rerun still determines capacity. The prepared Pathfinder direct continuation reuses 137 successfully built formula-cache entries with explicit provenance and unchanged physics; infeasible pySecDec/FORM routes remain closed. Representative performance work is limited to the existing five-case set and missing required metrics.

- 2026-10-05: the prepared native whole graph completes all 1,026 coefficient expansions (1,531.885 seconds) and all 1,026 sector JITs (127.529 seconds). An allocation then fails under the 30-GiB address-space cap before artifact publication. The watchdog subsequently reaches its deadline during core dumping and reaps the process after 1,807.811 seconds; all 60 frozen checks pass and no artifact or integral result exists. The exact failing allocation site is not established. Source review identifies avoidable full-buffer and per-byte JSON-value copies in native/CLI persistence; a focused Serde-based correction is being implemented with explicit legacy identity handling. This is a persistence capacity gap, not unfinished coefficient mathematics or successful full-integral acceptance.

- 2026-10-05: the latest user clarification ends further infeasible pySecDec/FORM attempts for the on-shell case. The existing Pathfinder direct route is the next comparator; its first attempt stops at a missing formula-cache entry after 19.186 seconds, before any complete bundle/result. The documented native fallback builder is the next configuration change, with the same bounded full-integral scope. A separately built matching Python binding imports correctly but its small smoke reveals an old Pathfinder return-type assumption; the existing working versioned environment remains available without an API-porting project.

- 2026-10-05: a fresh independent projected reference using the provider's documented pure-Taylor option avoids the earlier recursive IBP memory failure but reaches its 600-second generation deadline while writing FORM sector 41 of 968. Shutdown completes in 605.098 seconds; the outer process finishes in 608.398 seconds. All 62 frozen checks pass, all owned groups are absent and partial files remain; no complete reference vector exists. The prepared native fullgraph release passes its build and independent review and starts under the existing bounded protocol. No successful whole-graph or parity result is inferred from progress.

- 2026-10-05: the final delivery audit finds no additional missing required subsystem or HEPKit integration blocker beyond the original on-shell example and bounded numerical/parity ledger. Independent review reconciles the last regression-matrix row against its actual historical endpoint-equivalence requirement and existing native identities/full double-box vector: **182 rows, 101 Covered and 81 Retired, with none Partial or Pending**. No new numerical run or universal uncertainty claim is inferred. Full on-shell agreement and representative parity still govern goal completion.

- 2026-10-05: the CLI exposes the existing native family-preparation policy as an explicit opt-in, retaining `Original` by default. It preserves original source identities and records the native preparation/fallback report and actual active parameter count. The focused gate passes **38 tests**, formatting and CLI all-target Clippy, including the complete independently known `210 Gamma(eps)` vector through order one, cold artifacts and integration. No new algebra, graph or numerical implementation is introduced. The next full on-shell trial combines this exact preparation with NativeNamed coefficients.

- 2026-10-05: the original ten-parameter NativeNamed fullgraph trial was intentionally cancelled after 1,006.900 seconds at 372 of 1,026 representatives to use the existing exact prepared-family route. It was not a timeout and produced no artifact. Both independent external reference generation attempts reached their 30-GiB memory bounds: the original representation after 530.288 seconds and the equivalent eight-parameter representation after 431.801 seconds. Their partial output is retained, all children are reaped and frozen inputs pass their checks; neither supplies a numerical reference. Full-integral agreement remains open.

- 2026-10-05: the difficult captured representative now passes the ordinary public input/generation/compile/v3-artifact path and all three independent cold readers. Generation takes 66.752 seconds, compile/save 37.370 seconds, and the complete process 108.356 seconds. All seven orders through zero, the actual coordinate permutation, multiplicity one and literal-zero exact offsets are retained. Independent review accepts 21 signed coefficient/point comparisons and 24 weighted vectors (168 real components), including twelve forced 1,024-bit replays. The first ignored helper's structural expected-expression assertion failed before generation and was corrected without modifying the imported density or production source; that failure remains recorded. This closes public representative acceptance, not the original full graph or performance parity. The next work is the bounded full-graph trial and its exact-input independent reference.

- 2026-10-05: the user clarified the stopping rule: finish first-phase capability and scientific acceptance, establish representative performance parity with bounded effort, then mark the active goal complete and stop. No optional optimization campaign or threshold-phase planning follows automatically. Existing valid measurements should be reused; remaining work is directed at specific acceptance gaps.

- 2026-10-05: the validated native named-coefficient algorithm now has a public opt-in, with the physical route retained as default. Shared native Series/request modules, exact unregulated fallback, checked caller limits, cancellation, conservative conditioning profiles and exclusive typed progress/timing rejoin the existing multiplicity, kernel and artifact owners. The CLI uses the same native option types and preserves prior geometry/final output while coalescing only new coefficient-request JSON updates. **370 workspace tests pass**, with 23 explicit probes ignored; workspace formatting and all-target Clippy pass. Focused public controls include complete Taylor/IBP vectors, Gamma/complex coefficients, context/dispatch, weighted precision replay, worker clones and separate-process cold artifacts. Two test-only compile calls, two test-only lint findings, a wrong-stream CLI assertion and an actual presentation-cadence regression were corrected with their failed evidence retained. The captured representative through the public path, complete original on-shell graph, convergence/calibration and matched performance/platform gates remain open. No performance row is updated by this acceptance.

- 2026-10-05: the unchanged Numerica QMC branch passes its declared minimum Rust/Cargo 1.89.0 on Linux x86_64: **237 default, 240 serde and 217 alternative-backend tests**, with zero failures or ignored tests. Independent review verifies exact commands, versions, all 73 source hashes and both manifest/lockfile hashes. The published PR description now records this gate and retains the requested `@benruijl` mention. No Numerica source changes were needed. macOS/other-architecture qualification, FastSecDec's own platform gates and matched performance remain open.

- 2026-10-05: the first production adoption slice extracts shared native endpoint admission and checks cancellation-degree overflow. The independently accepted focused gate passes **50 tests**, with two explicit probes ignored; formatting and package all-target Clippy pass. Existing physical Taylor/IBP schedules, exact pruning, error precedence and defaults are preserved. A separate native symbol-hygiene probe rejects all 17 foreign metadata/hook conflicts without executing callbacks, retains occupied-input names, and validates stable repeated names and independent simultaneous alias maps. This closes the native API/probe prerequisite; the actual production allocator and public named-route integration remain required. The latest full workspace gate is still the earlier 336-test run. Complete on-shell generation, convergence/calibration and matched performance/platform gates remain open.

- 2026-10-05: all three independent original-expression native oracles are accepted, completing in 28.316, 32.790 and 31.645 seconds after the preserved export-only failure was corrected using native dynamic precision and round-trip formatting. The cold candidate reader independently passes all 21 signed-order/point comparisons and 24 complete weighted evaluations (168 real components), including twelve forced replay vectors and rescue at 256/384 bits; all 5,417 frozen checks pass. This closes the captured representative's candidate agreement, not public-path or full-graph acceptance. The reviewed production module plan is ready; implementation starts with shared endpoint admission while preserving the current default. Native symbol-hook hygiene remains a focused promotion gate.

- 2026-10-05: shared native oracle helpers reproduce every semantic field of the accepted six-case/forty-row controls. The additive diagnostic reader admits four complete layouts and rejects 31 malformed cases. The first actual native-dual oracle completes exact factorization, analytic admission, monomial reversal and all seven two-precision/realness comparisons, but exits after 28.705 seconds at an incorrect export assertion that native Float results retain the requested precision. Numerica intentionally tracks result precision dynamically. The failure and unchanged inputs are retained; no complete oracle is accepted. A transport-only correction records requested and actual precision separately and uses native round-trip decimal text, with new cancellation/growth/zero controls required before a fresh actual attempt. Production generation remains unchanged.

- 2026-10-05: the disconnected native pole-extraction/automatic-differentiation controls pass six cases and forty complete signed coefficient rows at independently repeated 512/1024-bit precision, compared with native Series of the original expressions. Known Taylor normalization, relative-only tiny nonzero coefficients, complex output, negative requested maxima, nine typed rejections and a separate literal-zero path pass independent review. This route reuses native factor collection, Dualizer and MPFR, with no custom Laurent arithmetic. Its analytic Taylor coverage is explicitly distinct from a native Series remainder. The captured-expression adapter and additive diagnostic reader remain subject to source/build review and actual-input gates; production generation is unchanged.

- 2026-10-05: the independently bounded unchanged-math original-expression oracle also times out at 600.315 seconds, after its earlier 180.167-second failure. Both stop during native width-seven series with no complete coefficient vector; all immutable checks pass and both processes are reaped. Later points, cold comparisons and production adoption remain held. Source audits identify existing native factor extraction and high-precision automatic differentiation as a possible independent oracle route, subject to exact six-pole regularization, intermediate-domain admission and small native controls. No custom Laurent algebra or further unchanged-oracle deadline extension is introduced.

- 2026-10-05: the hard four-loop full-positive-orthant reference is Checked through order zero, retaining the external order-minus-three statistical observation and separate native exact-zero proof. The original three-order native estimate and all nine covariance cells remain unchanged; the comparison keeps an explicit missing-order row and separate common-order pulls below 1.629. Seven focused transport tests, formatting and focused Clippy pass. The finite reference uncertainty is 0.57%, above 1‰, with calibration still open. The on-shell representative's native program builds successfully in 43.314 seconds with all seven orders, but the first independent original-expression oracle times out at 180.167 seconds. A fresh identical-math oracle has a separate 600-second bound; production adoption and full-graph acceptance remain gated on independent complete-order comparisons.

- 2026-10-05: the combined CLI and test-only interleaved-resolver workspace passes **336 tests**, twenty-three explicit probes ignored, with formatting and all-target Clippy passing. An initial unused test-only forwarding-call lint is corrected without changing resolver mathematics; its failed log is retained. The hard full-orthant reference returns its complete four-order physical tuple within the fresh numerical bound, with final independent transport pending. A separate native generation proves exact zero through order minus three across all 2,760 charts and 699 representatives; the old numerical estimate and covariance remain unchanged. Actual on-shell evaluator/oracles, full-graph generation, convergence/calibration and performance/platform acceptance remain open.

- 2026-10-05: the restricted test-only constant-face schedule passes ten native exact controls and 96 complete weighted-vector evaluator calls (768 component checks), including precision rescue and cold native IR. It then generates the previously stalled on-shell representative in **54.875504 seconds**, peak RSS 980,816 KiB, with all 101 immutable checks passing and formal orders `[-6,-5,-4,-3,-2,-1,0]` covered by native remainder one. Its actual evaluator and independent original-expression oracles remain pending. Original resolution stays the default; this is representative-level capability evidence, not full-graph, convergence or performance acceptance.

- 2026-10-05: a passive native trace attributes the actual named-coefficient timeout to resolving large mixed derivatives before constant-face substitution. Composition reaches the required native remainder by 19.567 seconds, but the separate 180.112-second bounded process produces no complete vector; all 103 frozen hashes pass. A restricted test-only resolver using earlier native constant substitutions is undergoing exact, evaluator and actual-input checks, with the original strategy unchanged. The hard four-loop external reference completes generation and compilation, then times out numerically without a full tuple at 750.953 seconds. A separately bounded same-science numerical retry is in progress. These diagnostics leave the on-shell capability and hard-reference gates open and do not change the frozen performance tables.

- 2026-10-05: the CLI adopts caller-owned geometry jobs through explicit `--geometry-workers` on `generate`/`run`, preserving serial defaults and independent integration settings. Its bounded Rayon adapter polls typed status on the coordinator, joins cancelled work and handles worker/coordinator unwinding. All **49 CLI tests** pass, three probes ignored; formatting and CLI all-target Clippy pass. Actual PTYs verify resize/cancellation during native geometry, no cancelled artifact, normal colored/monochrome completion and terminal restoration; one failed background-timeout driver is retained. The latest full workspace gate remains 330 tests. Separately, the first compact named-coefficient actual-target generation times out after 180.116 seconds with no vector, program or oracle comparison; all 89 hashes pass. This closes CLI adoption, not the on-shell capability, hard-reference, convergence/calibration, matched-performance or platform gates.

- 2026-10-05: the main generation context now exposes caller-owned chart/cone dispatch through the exact geometry cache, while retaining native per-integral domain and symbolic work. Independent review, 41 sector and eight context tests pass. Small named-coefficient program controls also pass 96 complete weighted-vector calls and 768 component checks, including native precision rescue and cold exact-IR reload; this remains test-only pending the actual on-shell input. The off-shell rank-two independent reference and native transport are checked through order zero, with both original/projected covariance matrices retained and maximum comparison pulls below 1.727; its 1.67% finite-part uncertainty does not meet 1‰. The combined workspace passes **330 tests**, twenty-two explicit probes ignored, with formatting and all-target Clippy passing. CLI dispatch adoption, actual on-shell generation, the hard-orthant reference, difficult convergence/calibration, and matched performance/platform acceptance remain open.

- 2026-10-05: caller-owned native chart/cone dispatch is implemented without a pool or new geometry algorithm. Its 38-test sector gate, independent ten-test rerun and source audit pass. The combined workspace passes **323 tests**, with twenty explicit probes ignored; formatting and all-target Clippy pass, with all eight affected controls rerun after a type-alias-only lint correction. Native map order, exact moments, domain/resource limits, error precedence and cancellation are retained. The named regular-coefficient test-only prototype passes eight small complete-vector controls, avoiding physical-body duplication during native endpoint assembly; its numerical-program/cold-reload and actual-target gates remain open. Existing IBP preparation separately timed out after its exact Taylor identity control, with no IBP evaluator. Main-context dispatch adoption, difficult references/calibration and matched performance remain in progress.

- 2026-10-05: the additive `KernelSet::sector_content_id` API binds immutable native IR, layout, numerical policies and selected retained semantics while preserving parent artifacts, checkpoints and replay meanings. It streams existing canonical transport into BLAKE3 without restoring coefficient expressions. Four focused tests, independent ecosystem review, affected formatting and library Clippy pass. This closes the standalone sector-identity API gate, without claiming algebraic equivalence or a machine-code cache. The last full workspace gate remains 305 tests; parallel geometry implementation and the large-example scientific/performance gates remain in progress.

- 2026-10-05: the off-shell scalar triple box now has a checked independent complete `[-3,-2,-1,0]` reference. The ordinary provider sums all 1182 sectors before estimating each coefficient and applies the native Gamma prefactor once. Exact transport and separate native original/projected full-vector comparisons pass, with maximum pulls 2.816 and 2.023 respectively; their full covariance is retained and the observations are not pooled. Five focused reference tests pass, two explicit recorders ignored. The two earlier bounded timeout attempts remain preserved. The external finite-part uncertainty is about 2.5%, so calibration, one-per-mille convergence and performance are still open; rank-two and hard-orthant independent references remain pending.

- 2026-10-05: caller-owned `GenerationContext` now adopts the exact geometry cache through an additive event API, rechecking every input's domain and rebuilding its expressions/metadata. Existing cache-free generation remains available. Independent source and analytic full-vector controls pass; two initial test-only exact-offset placement mistakes were corrected with the failed log retained. The combined workspace passes **305 tests**, eighteen explicit probes ignored, with formatting and all-target Clippy clean. The CLI now shares its color policy, honors `NO_COLOR`/plain runtime errors and shows compact public status in small windows; actual PTY resize, cancellation and terminal restoration checks pass. The original on-shell Taylor generation still times out at the original 1800-second limit with no artifact, after 80 completed representatives; a source-bound capture of the next input completes. Existing native IBP is the next bounded capability comparison, not an accepted optimization. Geometry parallel dispatch, additive sector IDs, difficult references/calibration and matched performance/platform gates remain open.

- 2026-10-05: caller-owned exact geometry reuse is implemented in the sector crate and independently reviewed. `GeometryCache` retains bounded immutable completed decompositions, keys native ordered supports/domain/resource limits, reports reuse explicitly, and preserves cancellation and failed-request semantics. All 28 sector tests pass (two explicit probes ignored), including six new cache controls independently rerun; sector all-target Clippy and formatting pass. This slice introduces no new algebra, dependency or executor. Main-library generation adoption, parallel chart/cone dispatch and additive per-sector content IDs remain separate deliverables; no cached-generation speedup is claimed.

- 2026-10-05: compact native Laurent aliases and exact-evaluator artifacts are implemented and independently audited. The complete workspace passes **294 tests**, with eighteen explicit diagnostics ignored; final affected two- and six-test targets, formatting, all-target Clippy and the production dependency audit pass. `AliasedAtom` remains the public compact symbolic view; compilation, conditioning and MPFR share one native exact program, while restored expressions are produced only on explicit request. Version-three artifacts retain native IR and semantic metadata; legacy bytes/identities remain supported. A fifth minimal Symbolica patch validates native decoded instructions before use, with native and preserved-byte regressions. The actual difficult representative passes exact pre-series template/image identity, all eighteen independent high-precision oracle comparisons and all seventy-two fresh/decoded weighted component checks. Two earlier structural-comparison failures are retained; five factored outputs differ structurally, so no all-six canonical-identity claim is made. Full on-shell generation remains the next bounded gate. The independent remaining-gates audit also keeps geometry caching, caller-owned parallel generation, sector identity wording and final interactive color/terminal evidence open; general affine upper-boundary inputs remain explicitly unsupported. None of these results changes the frozen performance baselines or completes phase one.

- 2026-10-05: the separately retained Issue 1 sector-sum reference passes native transport and independent audit. The new `issue_1_together.json` preserves all three provider values/errors and the original complete native covariance; its Checked label admits comparison under the audited ordinary-constituent uncertainty path, without claiming calibration or exactness. All three comparison pulls are below 0.857; the epsilon-two relative error remains above one per mille. The original disteval fixture remains byte-identical and Unverified. Four focused transport tests pass, with two explicit probes ignored. The ongoing native alias implementation passes 103 focused scientific tests plus artifact and legacy-CLI checks; the full workspace and difficult-representative production gates are still pending, so this reference milestone does not accept that production change.

- 2026-10-05: a separately instrumented native eight-core diagnostic now measures individual full-vector sample latency without changing native QMC/statistics. Triangle's pooled mean/observed maximum are 4.261 microseconds/3.065 milliseconds; box's are 4.358 microseconds/6.168 milliseconds. Every sector's count/mean/maximum, timer overhead, point/precision provenance and rejected-prefix control are retained. Instrumentation on/off gives exactly equal complete vectors, covariance and accepted replay states; these finite observed maxima include possible scheduling interruptions and do not replace the seven-seed package-amortized means. The independent Issue 1 ordinary-constituent sector-sum attempt also completes orders `[0,1,2]`, addressing disteval's identified omitted sector covariance using the provider's existing API. Its epsilon-two value is 760.9168 ± 0.9781, above the requested 1-per-mille relative-error target; a versioned native fixture is pending. Native alias production integration is in progress, including a small essential upstream-owned decoder-validation fix prompted by a non-evaluating malformed-input reproduction. No complete on-shell or final performance acceptance is claimed.

- 2026-10-05: the original-template/native-alias experiment passes nine small controls, all eighteen representative order/point checks with native high precision, cold exact-IR reload, and a fresh Laurent substage. That fresh substage takes 42.539 process seconds with 547,900 KiB peak memory; its six coefficients equal the cached alias inputs exactly. The separate cached evaluator builder takes 0.123 seconds, producing 1.52 MB of native exact IR without restoring giant coefficient Atoms. Ordinary precision remains unstable at every oracle point, so production precision rescue and full on-shell generation remain mandatory next gates. Issue 1's independent full positive-orthant execution and transport also pass; the fixture preserves orders `[0,1,2]` and reported errors but remains typed Unverified and comparison-ineligible because the external provider omits covariance between kernels sharing shifts. Three focused reference transport tests pass, with two explicit probes ignored. Review of other references using that provider and the existing native sector-sum integration alternative is in progress. No production strategy or sampling default is changed by this milestone.

- 2026-10-05: the separately reviewed native-only seven-seed continuation completes all fourteen eight-core rows, preserving full vectors/covariance, unchanged precision safeguards and zero evaluation failures. Every finite-part target is reached by the first complete 1024×16 allocation. Median reported integration/full process times are 0.03297/0.04307 seconds for triangle and 0.04160/0.05402 seconds for box. Pooled worker mean costs are 3.884/3.841 microseconds per sample; median slowest-sector means are 7.600/5.886 microseconds. These are not individual-sample maxima, and missing Pathfinder eight-worker execution prevents a paired speedup claim. The actual callback experiment completes preparation in 6.617 seconds but times out in native series at the unchanged 180-second bound without a complete vector. Its missing late-template-opacity step is an explicit comparison limitation; native template/alias reuse is the next reviewed proposal. The bounded independent full-orthant Issue 1 reference attempt has started separately. No production strategy or sampling default changes follow from these diagnostics.

- 2026-10-05: the eight-core smoke uses the corrected highest signed order, epsilon zero for triangle/box. Both native complete 1024×16 allocations pass the full-vector residual check and have zero evaluation failures; estimated finite-part relative errors are 4.40e-9/6.50e-7, with observed driver intervals 0.0419/0.0495 seconds and full processes 0.0532/0.0617 seconds. These are single smoke observations, excluded from seven-seed medians; an earlier crossing is unmeasured. Both frozen Pathfinder eight-worker processes abort at Symbolica's concurrent-instance license check, so the paired campaign is not accepted. Fresh generation, complete results, dependency/source hashes, precision policy, physical-core affinity and failed attempts are independently audited. No license workaround or scientific-certification promotion is used. The native polynomial-callback proof also passes four nonempty full vectors, an explicitly exercised exact-zero derivative, native numerical checks and callback-free cold IR reload. Actual-sector generation and bounded callback cache lifetime in a long-lived HEPKit process remain open. All three interrupted agents resumed their existing assignments.

- 2026-10-05: independent double-box reference milestone: the copied native pySecDec package completed after two retained bounded build failures. Its full physical vector through order zero, uncertainties and exactly-once Gamma normalization were independently audited. The existing native comparison API finds every coefficient of the retained 64-shift FastSecDec vector within 1.27 combined standard errors. The new versioned reference preserves all source identities, measured uncertainties and unknown work/covariance; the historical target and run card remain unchanged. Both reference transport tests pass through Cargo, with formatting and all-target Clippy passing; production code remains at the preceding 281-test combined gate. The independent original-expression oracle also now supplies all six Laurent coefficients at three exact points, with 512-/1024-bit agreement and an exact absolute-/relative-series control. The user clarified that the requested 1‰ target is the largest signed requested epsilon power, usually the finite coefficient. An eight-physical-core baseline protocol for the validated small examples is approved under that criterion; no measured crossing is claimed yet. Full on-shell generation, difficult-case calibration, remaining scientific references and final performance/platform gates remain open.

- 2026-10-04: literal-series dependency milestone: **281 workspace tests passed**, seventeen explicit probes ignored; formatting and all-target Clippy pass. A native reproduction proves that generic function-series substitution treated a valid underscore-suffixed regulator as a wildcard and could erase the full Gamma series. The minimal fourth Symbolica patch makes the two substitutions literal; native Gamma/composed-function regressions and a FastSecDec full-vector Gamma/endpoint regression pass. The first two formal-function probe attempts are rejected as empty-vector evidence. The corrected six-case proof passes full-vector native identities, O2, weighted MPFR and fresh-process native-IR checks, with independent review; actual difficult-representative validation remains open. Existing performance binaries retain their original dependency identities. At the user's request an agent delivered the standalone Rust-script MRE, README, patch and observed outcomes in `mre/symbolica-literal-series-variable/`: published 3.0.1 fails and the unchanged script linked to the corrected checkout passes. The patch applies cleanly to the published source. This milestone does not complete the phase-one goal.

- 2026-10-04: native-family preparation and convergence milestone: **280 workspace tests passed**, seventeen explicit probes ignored; formatting and all-target Clippy pass. The production dependency audit retains one Symbolica 3.0.1/SymJIT 2.26.4/Numerica/Linnet ownership chain and excludes Python, pySecDec, CLI rendering and reference providers. The new conservative family-preparation API reuses native partial fractions/sector projection, preserves original labels and weights, and has independent review; automatic CLI adoption remains deferred. All 72 fixed massive holdout rows complete 223,838,208 evaluations without failures, with independently audited full coverage and native joint covariance. The three-pair CLI cadence study preserves identical numerical state and reduces median integration time by 20.0%. All 28 paired triangle/box command rows complete at matching sample counts, with median native/reference times 1.012/2.884 s and 1.232/4.404 s; differing backend/precision/persistence boundaries keep strict parity open. The test-only Series-first experiment passes its controls but is slower and more memory-intensive on the difficult on-shell representative; no production switch follows. Shared native FunctionMap bodies are the next bounded proof. The first independent double-box reference attempt times out during FORM/C++ package preparation at 600 seconds with no numerical result, retaining its complete failure evidence. Eight-core one-per-mille timing and individual-sample maxima are explicitly unmeasured in the new status report. Full capability, difficult scientific references and final performance/platform acceptance remain open.

- 2026-10-04: the user specified the post-coverage optimization priorities: generation, lattice/transform convergence, and lower average and maximum per-sample evaluation cost for each sector. The performance section now sequences that campaign after full capability/scientific coverage, retaining work on generation bottlenecks that currently prevent required examples from completing. Native structured profiling, full-vector accuracy, rescue-tail costs and independent ecosystem audits remain mandatory.

- 2026-10-04: native Laurent attribution milestone: three focused controls, fail-closed native capture, bounded absolute/relative replays and exact comparison of every template/restored coefficient pass; formatting and all-target Clippy pass. The captured on-shell representative has a 5.7 MB template and a 256 MB restored finite coefficient. Native relative depth one followed by the measured required width reduces its series work from 148.974 to 37.332 seconds with identical outputs; this is one-representative attribution, not full-generation acceptance. Independent review resolved a cold-import Gamma warning as a lazy-initialization false positive through native normalization, derivative, series and canonical-identity controls. Production parsing is separately covered, and no dependency patch is needed. The diagnostic is entirely `cfg(test)` and cannot return a partial generated integral. Production series behavior remains unchanged pending whole-input evidence and a compact regular-coefficient design informed by Pathfinder's direct path. Projected on-shell generation, current-artifact holdout preparation and matched benchmark execution remain queued or in progress.

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
