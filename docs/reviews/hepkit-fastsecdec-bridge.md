# HEPKit FastSecDec bridge boundary

The Python API belongs in `symbolica-community`, alongside its existing HEPKit
bridge. FastSecDec remains a Rust library. This records the reviewed implementation
boundary; it does not claim that the bridge or notebook is already delivered.

## Shared inputs and crate identities

Accept the existing HEPKit diagram and kinematics objects. Their guarded
`as_diagram()` and `as_kinematics()` accessors provide the native objects directly.
`GraphIntegral::new` currently takes an owned `Arc<FeynmanDiagram>`, so the bridge
can clone the native diagram into that owner. Preserve the accessor's rejection
of a selected subgraph. Do not round-trip through DOT or introduce a second
model, momentum, tensor or graph representation.

Expose scalar bindings, auxiliary external vectors, edge powers and the explicit
measure multiplier through the existing graph adapter. HEPKit still owns model
cards, diagram generation, `simplify_algebra`, numerical wavefunctions and
kinematics. Symbolica expressions must stay native objects across the boundary.

Keep the kinematics tensor dimension (an integer or symbolic `D`) separate from
the parametric dimension expression, normally `4 - 2*eps`. The input wrapper
accepts that dimension expression and regulator explicitly and delegates their
relationship to native graph parametrization. Do not insert `D` into scalar
bindings; the graph adapter correctly rejects replacing the tensor dimension
there.

FastSecDec and the developing community bridge now select the published
`6c707c6b7` FeynKit lineage pinned by HEPKit PR #17. The full
graph/model/kinematics/tensor/Linnet/Idenso/Spenso group shares that owner through
the isolated `feynkit-fastsecdec-notebook` worktree. It retains only the existing
five-line literal-symbol substitution fix. Community additionally redirects its
PyO3 owner crates to that tree and resolves the same patched Symbolica/Graphica
and Numerica QMC sources. Root, community and portable-consumer metadata each
confirm unique selected owners; the native compile and installed runtime gates
are separate checks. A compatible API in two different Cargo package identities
is not enough to pass native objects between them.

## Generation and events

Wrap native input, generated-integral and kernel owners. Delegate generation and
compilation to their existing public entrypoints. Mirror typed status snapshots,
coefficient expansion, errors and complete Laurent/component layouts; never
derive numerical state from formatted terminal text.

The shared `GenerationSnapshot` observer updates belong in the Rust library.
The CLI and bridge use the same transitions, while the caller supplies elapsed
wall time, presentation frequency and cancellation. Feed every native event to
the observer exactly once before coalescing display updates, because phase
timings are increments. Generation completion means compilation can begin; it
does not certify completed compilation or artifact publication.

Generation remains synchronous with native `ControlFlow` callbacks. A Python
callback can transport real events, but that alone does not prove browser
repainting or interactive cancellation. Test those properties in Pyodide before
claiming a responsive streamed generation display. No extra evaluator or symbolic
algorithm belongs in the bridge.

## Caller-stepped integration

The existing QMC API already supports bounded stepping:

1. Request `next_work` and obtain the native worker context.
2. Run its weighted evaluation through the existing kernel evaluation context.
3. Submit the return, then accept its diagnostic/replay observation.
4. Return the native snapshot and yield to the notebook event loop.

Use a bounded package count per step. Reissue abandoned work through the native
retry operation; preserve checkpoint identity and the complete production
allocation required for convergence. Python owns the integration loop and its
parallelization policy. A browser notebook can yield between steps without
introducing a library thread pool.

Preserve real/imaginary components, all Laurent orders, full covariance,
complete-shift coverage and `WaitingForCoverage`. Missing uncertainty is not a
zero uncertainty. Report caller wall time separately from summed worker time.
Native O2 and portable interpreted kernels must report their actual backend.

## Notebook presentation

Keep the walkthrough centered on the calculation: a native diagram, its scalar
numerator, explicit masses and scalar products, followed by generation and the
Laurent result. Use submitted input forms so editing a number does not start an
expensive run. Reuse HEPKit's diagram renderer and input owners. Present the
backend and normalization beside the result, with code and detailed diagnostics
available below the main calculation.

During generation, update a compact phase timeline and counts from actual
`GenerationSnapshot` events. During integration, show the full signed Laurent
table, complete-replica coverage, elapsed wall time and the selected highest-order
estimate history. Show unavailable uncertainty as unavailable until native
coverage admits it. A completed allocation and a met accuracy target are separate
states. Sector cost and full covariance belong in expandable diagnostics, keeping
their native definitions and component ordering.

Provide deliberate run, cancel and resume actions. Integration yields to the
notebook event loop between bounded packages. Preserve accepted work and the
native checkpoint when cancelling; input changes invalidate that result instead
of silently relabelling it. Synchronous generation callbacks must be tested for
visible browser updates and interrupt handling in the actual marimo/Pyodide
runtime before the notebook claims either capability.

## Delivery checks

The first bridge check should construct an integral from existing HEPKit objects,
generate and compile it, receive real typed events, complete a small QMC design,
and restore its checkpoint. Exercise a numerical error and cancellation without
publishing partial work as a completed result. Then run the same public workflow
in the actual Pyodide wheel and measure event delivery and yielding.

The default notebook cases remain the massive triangle, massless box, rank-two
box numerator and coupled two-loop sunset numerator. The generated gg→HH diagram
enters the notebook only after the ordinary native CLI produces a valid result
and its measured cost establishes a suitable place in the walkthrough.
