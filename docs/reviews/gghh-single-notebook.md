# A self-contained gg → HH notebook

The user requested the complete example in one notebook, with no helper scripts,
`Model.standard_model()`, native diagram generation and a short visible setup.
They also clarified that numerical parameters should live in the notebook rather
than in a parameter-card file.

The new `examples/hepkit/gghh.py` contains its physics setup, native calls,
streamed presentation and optional QMC/Havana runs. It imports installed HEPKit,
Symbolica, marimo and the Python standard library; it reads no model, parameter,
graph or result files. The earlier dashboard remains a separate demonstration
of the richer workflow, not a dependency of this notebook.

## Challenges and decisions

1. **Model and point.** `Model.standard_model()` already embeds the complete
   model. A constructor-only check proves that updating MT/ymt to 172.5 and
   WT/WH to zero reproduces the previous model and card exactly. The notebook
   instead supplies these exact values directly through scalar overrides;
   no card object or file is required. The missing generic operation is closing
   analytic parameter/coupling definitions consistently after those overrides.
   The existing Rust CLI logic has moved to the owning `feynkit-model`
   crate and exposed as `Model.scalar_bindings`, with CLI reuse and thin Python
   transport. No Python dependency resolver is added.
2. **Selecting the topology.** The short native ttg/ttH generator call produces
   120 diagrams in a 0.111-second local observation. A predicate on native edges
   and native connectedness selects eight double boxes, with the same first
   physical topology as the established example. Six connected top edges form the
   fermion hexagon; two disjoint external g-H pairs identify its opposite gluon
   chord. No graph identity, saved DOT or custom graph-search implementation is
   used to choose the diagram.
3. **External states and momentum routing.** HEPKit exposes the physical leg
   identities, native loop-momentum basis and GammaLoop wavefunctions. The
   notebook still assembles the small Gram matrix explicitly using native
   `Tensor.dense`, Minkowski `dot` and `Kinematics.with_scalar_product` calls.
   A future owner-level physical-point constructor and wavefunction-to-tensor
   adapter could shorten this portion. Binary64 wavefunctions are transported
   exactly with `Fraction`, without inventing their numerical conventions.
4. **Scalar numerator and normalization.** Native `simplify_algebra` and
   `to_dots` contract the color delta and helicity projection. Internal algebra
   retains D while numerical external states have four components. The native
   numerator prefactor and overall factor enter once. The existing native
   expression-copy API places the contracted scalar numerator on a HEPKit
   diagram and resets its consumed projector to one. The notebook then calls
   `diagram.sector_decompose(...)`, as requested. The diagram supplies its seven
   physical propagators and graph weights; no family or power vector is needed
   in the notebook. Full numerator expansion is avoided.
   This route exposed an admission-order defect: default model widths were
   checked before the explicit scalar point. The native atomic constructor now
   validates effective masses and widths while continuing to reject unsupported
   unbound defaults and nonzero-width overrides. Both the Python and CLI input
   paths use it; the original strict constructor remains available.
5. **Execution controls.** Ordinary cells expose the actual calls. The costly
   decomposition and integration cells start disabled for explicit execution in
   marimo's editor; no custom Generate/Resume buttons or state machine are
   required. Sessions live in separate cells, so rerunning only the integration
   cell resumes accepted work. Browser interruption can still be delayed inside
   long native operations.
6. **Inspectability versus brevity.** The shared native HEPKit presenter displays
   generation status, and a caller-owned integration loop displays Laurent
   estimates. Sector tables and selected maps/prefactors/exponents read existing
   native owners. The short example does not recreate the full dashboard's
   lifecycle controller or checkpoint UI.
7. **Browser installation.** The hidden import cell installs the exported,
   hash-verified community wheel only under Pyodide. `export.py --notebook gghh`
   selects editor mode and emits no helper or physics input archive. All physics
   remains in the notebook; the manifest describes the library installation.
   The existing multi-example dashboard export retains its original assets.

## Validation in progress

The built-in model comparison and small native graph-generation/selection probe
pass. Independent comparison finds every physical graph field identical; IDs
differ because they include the model fingerprint, which differs between the
unmodified built-in model and the earlier card-bound model. Three generic Gram
holdouts agree for the weighted scalar numerator within 7e-16 relative error;
this is numerical evidence, not an exact symbolic identity proof.
The new notebook parses and passes `marimo check`; it has no custom workflow
button calls. Five shared native scalar-binding tests and four CLI input tests
pass, including exact equality with the previous ggHH binding map. Scoped strict
Clippy passes. The corrected effective-point route additionally passes 30 native
input/family/example tests and five CLI input tests, with strict Clippy. Its new
Standard Model bubble regression checks the exact massive F polynomial and
dimensional exponents, not only successful construction. The updated installed
native wheel passes all 99 maintained Python/frontend tests with zero failures
or skips, including Model transport and direct-diagram point admission.
An installed native source-cell run completed the actual diagram route and all
30 generated charts in 61.30 seconds on one core, then completed the small QMC
allocation in 65.76 seconds. The optional large Havana default exceeded the
600-second guard; this run did not complete its final report and is not a full
notebook acceptance. The exploratory Havana default is now 1024 points across
eight production batches, following the existing four-batch pilot. Precision
claims continue to use the separately recorded native CLI runs.

Generation and compilation now use the shared HEPKit marimo presenter through
`progress="auto"`; the notebook's custom generation callback is removed.
The final `get_citations()` cell depends on completed kernels so it reflects
actual use. It renders existing Symbolica citation objects and downloads BibTeX
for the software, pySecDec and the geometric/subtraction method papers, alongside
the ecosystem's other tracked references. The dashboard notebook has the same
collector, refreshed by its existing run revision.

The combined progress/citation wheel passes all **118 maintained controls**,
including fresh-process import/cancellation/cold-load citation tracking and
native progress callbacks/cleanup. Its generated type declarations preserve the
same runtime bytes. The unchanged HEPKit diagram-generation lifecycle assertions
also pass, after explicitly enabling the grouping mode their inherited fixture
expected; no production default changes.

The actual notebook generation/inspection completed on the new wheel in 59.42
seconds (30 charts, 54 mapped terms, 324 endpoint-power records). A separate
verification writer then assumed nested covariance instead of the native flat
matrix. This diagnostic failure is retained. A reviewed continuation loaded
the saved kernels in 0.53 seconds and executed the unchanged notebook QMC,
Havana and bibliography cells. It completed QMC's 245,760 points in 65.90 seconds
and the Havana 4,096-point pilot plus 8,192-point production in 26.07 seconds,
with zero evaluation failures. Both full four-component vectors, sixteen
covariance entries and native checkpoints are saved. These single-core local
observations are not an eight-worker performance or precision certificate.

The finite imaginary coefficient is -33.7133 ± 0.3463 for the small QMC run
and -34.2042 ± 0.9612 for Havana, agreeing within 0.49 combined standard errors.
The bibliography cell returns eight native, deduplicated records, including
FastSecDec, pySecDec, Kaneko–Ueda and Binoth–Heinrich, with HTML and BibTeX output.
Evidence lives in `output/diagnostics/progress-citations-controls-2`,
`gghh-single-notebook-native-2` and `gghh-single-notebook-native-3`; generation and
continuation are explicitly separate runs. The native editor additionally
demonstrates live generation/compilation, all 30 sector choices, a selected
chart's map/prefactor, reactive citations and the BibTeX download. Its Stop/resume
workflow remains unverified: diagnostic selectors stopped before sampling,
with no notebook exception. All owned processes were reaped and scientific
statements remained unchanged. The fresh Wasm wheel passes the generic Pyodide
smoke and all 89 maintained portable controls across nine modules, with zero
failures or skips. These include native Model transport, progress callbacks and
fresh-import/use/cold-load citation tracking. The generic rendering fixture was
updated to consume HEPKit's current `DiagramRender.to_svg()` and typed
`RenderSettings`; its initial failure is retained, and no wheel rebuild was
needed. All guarded processes were reaped and compiled sources stayed unchanged.
This validates the portable API, not a new full gg→HH browser execution.
Its source and artifact identities are recorded in
`output/diagnostics/sector-decomposition-portable-runtime-2/result.json`.
The earlier API milestone's 95 tests and UI acceptance do not validate the new
Model method or this self-contained notebook. No new ggHH performance or browser
convergence claim is made.
