# Eager Marimo workflow implementation review

Author scope: `examples/hepkit` frontends, shared presentation/lifecycle helpers,
notebook tests and guides; native read-only rich protocols in
`bindings/python/src/presentation.rs` and its estimate view. The independent
binding/core and actual browser acceptance are recorded separately by the other
agents. No release/browser success is inferred from the author checks below.

## Native ownership and explicit work

The gg → HH catalogue uses the current public HEPKit particle whitelist
`particle_selection=[6,21,25]`, both loop orders, QED2, initial/final symmetry,
and no bridge/self-energy/tadpole/zero-snail filters. Native zero-flow internal
edges are explicitly allowed as part of the literal-all request. The native owner chooses
ordering and IDs; the selected default is literally its first one-loop entry,
even if the native contraction later proves that graph zero. Box choices are
identified in scientific tests from native internal-edge particles, not a custom
graph representation. The reviewed public whitelist includes antiparticles and
retains gluon/Higgs self-interaction vertices. Parameters' separate native
research and the actual installed current wheel observed 152 graphs (5 one-loop
and 147 two-loop) with zero-flow edges allowed. The first native one-loop is
D035; D150 is the first top box. Counts and IDs are evidence, not a hardcoded
catalogue or selection rule.

`Build diagrams`, `Generate sectors`, `Inspect`, `Integrate QMC` and `Integrate
Havana` are separate actions. Idle construction, selecting a diagram, rendering
metadata and opening panels perform no generation, contraction, compilation or
sampling. Generated evaluators remain in memory and are directly displayed as
`artifact`; inspection and sampling never round-trip through temporary files or
a CLI subprocess. The explicit Prepare downloads action serializes optional exports on the
notebook caller; download widgets receive immutable bytes, without native-owner
callbacks or automatic serialization during progress.
The timer component is actually mounted only while caller work is
armed. Every tick advances retained native generation units, QMC packages or
Havana global batches within a 50 ms caller budget and a 2048-unit hard cap.
Scientific observations and rich display revisions are published at 1 Hz and
explicit action/phase/completion boundaries; cached presentation cells are not
invalidated by every caller tick. Generation callbacks are similarly throttled.
The native composed generation/compilation owner, rather
than a replayed Python script, supplies pause/resume. Each algebra unit is atomic;
its duration bounds how soon the next queued button can take effect. No hidden
thread/pool is introduced. The integration pilot remains distinct from production
and uses existing native freeze/checkpoint operations.

## Runtime physical inputs

HEPKit's native tensor `dot` contracts symbolic COM vectors to establish exact
structural Gram zeros. Every nonzero momentum and polarization product becomes a native real runtime
symbols via `S(name, is_real=True)`; the native Integral constructor checks their
attributes. A real-owner regression covers this admission boundary.
The explicit integration action recomputes the native Gram values for the chosen
√s, cosθ and Higgs mass. Polarizations use HEPKit's shared dimensionless (+,+)
wavefunction convention. Its normalization product remains a runtime input as
well: retaining its numerical f64 approximation as an exact symbolic constant
was the source of the giant rational seen in the first browser run. The native
FourMomentum Python API is numerical; the fix does not invent an exact symbolic
polarization formula. The selected real planar Gram convention rejects an
unexpected complex product instead of silently discarding its imaginary part.
All contributing independent model leaves remain native runtime parameters.
Native default metadata is explicitly merged with the chosen point and passed to
`Kernels.with_parameters`; it is not supplied as scalar specialization input.

The exact zero top/Higgs widths are explicit structural overrides. The scalar
massless examples similarly specialize their sole named mass to zero; the massive
triangle retains a runtime mass. This preserves the native mass-stratum contract
and avoids binding a massless default through a nonzero runtime-mass constraint.

Projection, color/gamma/tensor contraction, rational transport, parameterization,
expression simplification and evaluation remain native. There is no independent
CAS, graph parser, covariance accumulator or numerical zero heuristic. Native
one-loop masters and reduction providers are unchanged; this work neither adds a
replacement reference algorithm nor claims an independent amplitude validation.

## Presentation boundaries

Both notebooks share short native calls and hidden UI plumbing. Only the selected
native diagram, bounded expressions, retained charts and current numerical
observations are rendered. Both raw and simplified numerators use retained
native scoped pagers whose creation and display belong to stable prepared-input
cells. Inspection has its own action-only revision and owning cell. Progress,
integration, pause and resume do not invalidate either cell; an intentional
replacement closes the old Pager and constructs a fresh model.
Inspection ranks at most ten evaluator programs, preserves stable sector IDs,
and distinguishes pre-subtraction monomial factors from a full integrand limit.
Threshold certificate dumps are absent. Compact mathematical previews use the
native formatter with namespaces hidden, avoiding its LaTeX namespace annotation
bug (a literal tab inside the emitted text command). Qualified symbol identity
remains visible in metadata and exact source exports.

Explicit Inspect materializes only the selected native compact coefficient via
its owner's `expression()` operation and opens
`TensorExpression(expression).paged(page_size=25)`. The existing HEPKit/Spenso
Pager supplies bounded pages, focus/navigation, cache and widget behavior;
`mo.as_html` invokes its native `_display_` route. Study retains that owner and
closes it when another inspection or generation replaces it. No full `to_html`
export or alternate expression viewer is introduced. The inspector exposes a
physical epsilon order (default zero) alongside the stable sector index; the
native retained schema resolves the requested order to its coefficient position.

The Rust `_repr_html_` implementations use public existing getters and Symbolica's
bounded `formatted(...)._repr_html_` protocol. They never call generation,
compilation, alias expansion, parameter binding, session stepping or statistical
observation collection. Mathematical output is not parsed/reformatted by another
CAS. Native estimates keep all signed orders/components and full covariance;
missing QMC mean/error coverage remains unavailable. Native live observations are
explicitly provisional and never enter stopping/checkpoints. Integer counters do
not pass through f64, and small nonzero numerical values use scientific notation.
Large collections have explicit bounded previews rather than an unlabelled dump.

## Author checks and remaining acceptance

- Python syntax and `marimo check` pass for both frontends, including the pinned
  0.24.2 Marimo environment. Its actual controls, mounted refresh HTML and open
  monitor HTML also construct successfully in a non-scientific host probe.
  Open monitors use compatible styled native containers;
  the unsupported newer `accordion(expanded=...)` option is not used.
- Native Marimo invalidation exposed its micropip shim's missing
  `spec_from_loader` global. Browser-only imports now use `import_module` inside
  the Emscripten branch, so native static module discovery never probes them.
  The actual pinned Marimo compiler/module registry confirms this for both
  frontends; no upstream runtime monkeypatch is added.
- Actual browser validation also caught a timer mismatch: Python accepted a
  50 ms refresh, but Marimo's shipped frontend enforces a 100 ms minimum. The
  final timer uses 125 ms while retaining the 50 ms caller work budget and 1 Hz
  scientific observation cadence. The frontend validator was inspected directly;
  a constructor-only check is not recorded as browser acceptance.
- Shared dispatch controls cover retained-generation
  pause/KeyboardInterrupt resume, stale ticks, no sampling before Integrate,
  checkpoint error provenance, explicit runtime binding, portable asset hashes, runtime report shape and read-only inspection.
  A deterministic clock control proves budgets stop scheduling after the deadline
  (while permitting the final atomic unit to cross it); another proves native
  observation reduction happens only when due or at an explicit boundary.
  A failed prior-report handoff retains the original session, numerical point
  and error rather than allowing a new binding around the old session.
- Native Kuo-33002's minimum 1024 points is reflected in the default; a smaller
  QMC selection remains idle with an explicit message. Smaller Havana batches
  remain valid. Integrate before Generate does not create a session or throw an
  attribute error.
- Native binding check and strict Clippy with `python_stubgen` pass the new rich
  protocol implementation (executed independently by the binding author).
- Eighteen real-owner input controls pass on the installed matching native wheel,
  including the full catalogue, symbolic box preparation and Integral admission,
  alternate/symmetric-angle Gram points, massless structural overrides, native
  numerator pager lifetimes and compiled symbolic model/Gram inputs.
  The complete notebook suite passes 88/88 on the final installed wheel,
  including the additional native formatting, stable view revisions and
  caller-owned download preparation controls.
- The actual D150 box probe generates four eager sectors in 48 retained native
  units. Thirteen runtime inputs bind two physical points with distinct numerical
  identities and byte-identical template artifacts. Native QMC and pilot/frozen
  Havana both return finite full vectors and covariance. The measured generation
  path took 0.718 s and the complete bounded probe 5.432 s on this host; these are
  acceptance timings, not a performance claim. Evidence is retained under ignored
  `output/notebook-workflow-review/box-workflow-final.txt`.
- An earlier bounded-caller native probe, using 128-point packages, finishes the default D150 QMC allocation
  (16,384 sector points) in two work budgets of 52.8 and 27.2 ms, with 43.6 ms
  native worker time. The prior one-package-per-timer approach required 128
  250 ms ticks. Generation takes five budgets rather than one tick per each of
  48 native units. These host observations establish removal of per-unit timer
  pacing, not an evaluator speedup. The final default timer is 125 ms (above Marimo’s frontend minimum), with
  250 ms and 1 s available as slower options. Cooperative event-loop scheduling,
  message transport and rich rendering still add overhead; no full-throughput
  claim is made. See
  `output/notebook-workflow-review/caller-budget.txt`.
- Independent timer-free D150 controls identified repeated mandatory native
  snapshots as the large-allocation bottleneck: 2,097,152 points with 128-point
  packages and one package per call took 57.986 s; 1024-point packages took
  4.382 s, and the final 4096-point packages took 4.104 s (3.976 s native worker
  time, 512 calls). Means and the entire covariance were bit-identical in all
  controls. The notebook therefore uses the existing public 4096-point package
  setting while preserving 256-point evaluator chunks and the 50 ms caller
  budget. This reduces redundant snapshot reductions without adding a new
  estimator or bypassing native admission. A package may overrun the caller
  deadline; Pause is serviced at its next boundary. Evidence is in
  `output/notebook-workflow-review/qmc-step-4096.txt` and the independent reuse
  review. These are timer-free host measurements, not browser throughput claims.
- The actual Marimo comm lifecycle was independently reproduced: widgets belong
  to their creating cell and are closed when that cell or a descendant reruns,
  even when a Python Pager and HTML remain retained. Reusing old HTML cannot
  reopen its disposed model. The final prepared/inspection cells create and
  display their own native widgets and depend only on stable scientific view
  revisions; monitor updates have no dependency edge to them. The focused native
  regression proves dispatch/monitor/integration reruns leave the viewer alive.
  Source and evidence: `output/notebook-workflow-review/marimo-widget-lifecycle.txt`.
  A second browser regression isolated a separate container issue: Marimo's
  accordion unmounts its closed content, and the native Pager disposes its last
  detached widget view. Even a stable cell cannot reuse that stale model ID.
  Numerator panels now use ordinary HTML details elements with independently
  toggled visibility, preserving both mounted widget DOM trees and their native
  Pager/UI wrapper owners. No private Marimo API or upstream widget patch is
  added. The independent native detach probe reproduces the failure without any
  cell invalidation (`marimo-accordion-lifecycle.txt`); the focused author control
  verifies two mounted native widget containers with no accordion wrapper.
  Actual repeated collapse/reopen validation remains part of root's final
  browser acceptance.
  Artifact/report downloads likewise contain precomputed immutable bytes. No
  claim is made that Marimo's RPC route itself uses a background executor.
- The literal default D035 contracts to exact zero natively, completes with zero
  numerical sectors and returns the native exact total `[0.0]`; this is not a
  numerical-failure fallback or a topology-based zero guess.
- The final binding suite passes 73/73, including all three native rich-owner
  and selected-integrand pager controls covering small Laurent
  values, covariance, scoped viewer release and byte/checkpoint immutability.
  The first wheel predated those methods; the final Clang/ThinLTO wheel executes
  them successfully. Native ABI3 uses PyO3's owned String extraction for exact
  Unicode text. The test fixture uses valid positive stability thresholds.
- Root records actual Marimo computer-use acceptance and final gate results in
  `notebook-browser-acceptance.md`. Its final live checks confirm stable native
  viewers across QMC/Havana, explicit artifact download and cold eager reload,
  and responsive pause/resume on a larger D150 allocation. Native and Pyodide
  qualification remain distinct; export packaging is not browser execution.

The untracked user `examples/hepkit/__marimo__/` directory is preserved. Both
exports package only current shared Python helpers and the small scalar fixtures,
with exact SHA256 manifests; the historical stored gg → HH graph is neither used
nor bundled by the new catalogue workflow.
