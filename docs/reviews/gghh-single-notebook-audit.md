# Standalone ggHH notebook: scientific input and native ownership audit

Independent source review, 2026-10-06. The current entry point is
`examples/hepkit/gghh.py`, using `diagram.sector_decompose(...)`. This review
accepts the source design and the bounded topology check below. It does not
claim that the final notebook or its new model primitive has passed installed
native or Wasm execution; those are separate delivery gates.

## Native scalar bindings

`Model::scalar_bindings` belongs to FeynKit's model crate. It extracts the
previous CLI's exact model specialization: internal parameters retain analytic
definitions unless explicitly overridden by the restriction card, couplings
remain analytic, and numerical model/card values become exact rationals for
both components of the supplied binary64 value. Cached dependent numerical
values do not replace analytic definitions. Exact caller overrides are applied
before closure and take precedence over model/card definitions.

The optional card is applied to a clone using the existing native card method.
Unknown card names propagate the native error without mutating the caller's
model. Reapplying the same card in the CLI is semantically idempotent; the CLI
still needs its separately updated model for native graph construction.

Closure preserves the old simultaneous Symbolica `Pattern::Literal`
replacements, including underscore-suffixed keys, and rejects dependencies
remaining on its own keys. Symbols/functions outside the map remain symbolic;
this is not a general UFO-function evaluator. No expansion, alternate CAS,
Python resolver or dimension/graph policy is introduced. The Python method only
transports existing Expression keys/values and native errors. The new source
tests include complete Standard Model binding equality against the archived CLI
algorithm; this independent review did not execute those tests.

The model operation does not invent a Lorentz dimension. The notebook supplies
its explicit symbolic tensor dimension through existing HEPKit tensor and
kinematics APIs, and supplies `4-2*eps` separately as the integration dimension.

## Selection and the different diagram ID

Within the generated two-loop process restricted to the native ttg and ttH
vertices, six top edges on six trivalent vertices form a hexagon when their
native subgraph is connected. The two disjoint adjacent g–H pairs enforce an
opposite internal-gluon chord, hence two boxes, each carrying one gluon and one
Higgs external leg. This relies on that constrained process; the small selector
is not a replacement for a general graph algorithm. Connectivity, endpoints,
particle identities and routing come from HEPKit/Linnet objects.

A bounded installed-native probe executed the actual notebook generation and
selection cells with three inputs. Each produced 120 diagrams and selected
8 double boxes:

| Model and generation options | Selected native ID | Comparison to accepted raw input |
| --- | --- | --- |
| Standard Model, notebook options | `63c454518496abf684563b2fab3fb259` | Only `id`, `model`, `name` differ |
| Archived model/card, notebook options | `eb51e855105cae69514fd9dcdd4bbffe` | Only display `name` differs |
| Archived model/card, original explicit options | `eb51e855105cae69514fd9dcdd4bbffe` | Only display `name` differs |

After removing only `id`, `model` and `name`, the complete native serialized
diagram objects are **exactly equal**. This includes vertices, edges,
half-edge ordering, symmetry diagnostic, overall factor, numerator, numerator
prefactor, projector, momentum routing and cuts. Native rehydration normalized
the archived expression spelling before comparison; the numerator was not
expanded or numerically compared by this probe.

The different ID is expected: `DiagramId::from_key` hashes the model fingerprint
along with the canonical topology key. The new notebook keeps the standard
model unchanged and passes its scalar point explicitly later. The old helper
applied a card first, changing model parameter/cache data and its fingerprint.
The observed difference is therefore not evidence of a different selected
physical graph or normalization. Current defaults and the original explicit
generation options select the same eight native IDs under the same model.

Local evidence is retained in
`output/diagnostics/gghh-single-notebook-audit-1/`: `result.json`, the three raw
native diagram JSON files, `watchdog.log` and `process-cleanup.json`. The probe
used one CPU, a 60-second/4-GiB guard, completed in 0.312 seconds and left no owned
processes. It performed no numerator contraction, sector generation or sampling.
The older fixture's 192-diagram generation report is historical and is not
relabelled as the current native count.

## Current diagram entry and normalization

The notebook contracts the native in-basis numerator with the color delta,
numerical plus-helicity polarization data and original projector. Existing
HEPKit tensor simplification and dot-product conversion retain the symbolic
internal dimension and the four-dimensional external Gram data. Physical
kinematics use sqrt(s)=300 GeV, mt=172.5 GeV, mH=125 GeV and cos(theta)=4/5.

The scalar projection is stored back into a copied native diagram through the
existing `with_diagram_expressions` binding. Its projector becomes one because
projection has already occurred. The authoritative evaluated overall factor
remains a separate native diagram field; the original numerator prefactor is
preserved by the copy. The notebook does not multiply either factor into the
stored numerator or divide by the diagnostic symmetry factor.

`diagram.sector_decompose` uses the existing diagram input path, whose native
`contract_numerator` multiplies numerator, projector, numerator prefactor and
overall factor once. Its original seven propagators retain unit powers.
Scalar values and auxiliary polarization vectors enter through the shared
native input boundary. The domain check remains active; no assertion of absent
thresholds bypasses it. Decomposition, compilation and integration use the
existing caller-driven APIs.

Earlier construction produced weighted numerators agreeing with the established
helper at three generic Gram holdouts to about 7e-16 relative difference. That
is useful numerical evidence for the contraction, but not exact equality and
not a final runtime test of the subsequently requested diagram entry. The
final installed notebook must still validate the new scalar-binding primitive,
owned diagram route and displayed native event fields. Final Wasm execution is
also pending. No new owner change or build was initiated by this audit.

## Effective scalar-point admission follow-up

Independent review of the frozen correction found **no blocker**. The new
`GraphIntegral::new_with_scalar_values` checks explicit width and mass bindings
before consulting cached model defaults or constructing the native quadratic
family. An explicit zero width can therefore specialize the unchanged Standard
Model. The original `new` constructor supplies an empty map and retains its
previous strict default-point behavior.

Width bindings must be exactly zero; nonzero and unresolved widths remain
errors. Explicit masses pass the existing realness and finite numerical-value
checks, so complex, nonfinite and unevaluable values are rejected. Missing
bindings retain the previous model checks. The custom propagator-template
comparison remains unconditional and cannot be bypassed with scalar overrides.

A nonempty map still passes through the shared native family specialization,
including literal substitutions, closure against all bound keys, formal
momentum/function-head checks, tensor-dimension protection and existing
external-product consistency. The Python and CLI routes now supply the map to
this constructor and remove their former second binding call. Numerator,
projector, prefactor, overall weight and measure ownership are unchanged.
Repeated family rebuilding for auxiliary vectors is idempotent for this closed
map. Native `Kinematics::with_momenta` inserts identities into its existing set
without clearing scalar-product assumptions; the earlier scalar specialization
therefore preserves auxiliary Gram data for the subsequent rebuild.

The author gate at
`output/diagnostics/effective-point-admission-3/result.json` reports 35 passing
native/CLI controls, strict scoped Clippy and formatting, with unchanged frozen
inputs and no surviving owned processes. It includes actual Standard Model
zero-width specialization, exact mass-dependent F-polynomial/dimension checks,
rejected width/mass/dimension/momentum overrides and the custom-template guard.
The added installed-Python regression compares the weighted diagram with a
native family control and checks model immutability; execution of that final
wheel regression and notebook remains a separate runtime gate. This follow-up
performed source review only and did not change compiled files or restart a
build.

## Shared progress and bibliography follow-up

Independent source and primary-reference review, 2026-10-06. HEPKit already
provided automatic Marimo progress inside its native diagram generator. Its
public Python API, Rust implementation and lifecycle test were inspected, then
a bounded installed-native probe confirmed its behavior. No generic presenter
was exported: `GenerationProgress` and `GenerationReport` contain diagram-specific
data and cannot represent sector generation faithfully.

The existing widget lifecycle is now extracted into the Rust-only
`feynkit_py::MarimoProgress`. Both generators reuse its detection of an already
imported, running Marimo, spinner/bar selection, absolute count updates, explicit
flushes and context cleanup. Diagram-generation stage labels, its 150-ms event
coalescing and cancellation remain in their original owner. FastSecDec retains
its own typed snapshots and caller-owned observer; presentation does not create
workers or drive scientific work. The extraction introduces no Python rendering
framework or community-owned implementation.

A focused Rust unit compiled against the existing PyO3 library passes count
resets, zero/unknown totals, flushes, close-once behavior, original-error
preservation and failure while replacing a widget. Strict scoped Clippy passes.
These checks validate the shared helper, not the newly linked full extension or
browser rendering. Evidence is in
`output/diagnostics/hepkit-shared-progress-1/`. The original comprehensive
`automatic_progress_tracks_marimo_lifecycle` test remains unchanged; its Python
payload is prepared for replay through the final installed extension after the
maintained native controls pass.

The four bibliography entries have been checked against their actual roles:

| Reference | Implementation or acknowledgement |
| --- | --- |
| FastSecDec repository | Software attribution without an invented publication DOI. |
| [Kaneko–Ueda, 0908.2897](https://arxiv.org/abs/0908.2897) | Sections 3–4 describe dominant-monomial cones, simplicial monomial maps and recursive triangulation. These correspond to the native sector geometry in `stages.rs` and `triangulate.rs`; FastSecDec owns its exact cone construction. |
| [Binoth–Heinrich, hep-ph/0004013](https://arxiv.org/abs/hep-ph/0004013) | Section II.C, equations 15–16, gives endpoint Taylor subtraction and analytically integrated pole terms, matching `generation/subtraction.rs`. |
| [Borowka et al., 1703.09692](https://arxiv.org/abs/1703.09692) | Explicit acknowledgement of pySecDec's influence on development and scientific cross-checks, without representing it as a runtime dependency. |

Authors, titles, journal metadata and DOIs match the primary records. Method
citations identify mathematical foundations; they do not claim identical
implementations or imported software. The records and BibTeX stay in FastSecDec.
Community adds only the feature-gated call into the existing global collector,
which already merges references and reasons by stable identifier.

Usage is cumulative and querying does not reset it. Importing the module or
cancelling at the initial event does not mark use; starting native generation
or successfully loading kernels does. The compact notebook explicitly depends
on `kernels` before querying `symbolica.get_citations()`, so its bibliography
appears after generation and compilation. Rendering uses the existing native
Citation HTML and BibTeX methods. The advanced dashboard follows its existing
run revision dependency.

The updated native editor protocol in
`output/diagnostics/gghh-single-notebook-editor-2/` checks visible automatic
generation and compilation updates, closed progress contexts, all four rendered
references and downloaded BibTeX after generation, plus interruption and
resumption of the same integration session. It is prepared and syntax-checked;
final extension and actual editor/Wasm acceptance remain separate gates.

The subsequent installed lifecycle replay found an inherited test-fixture issue:
the published `c6710fe` test expected numerator-grouping phases while leaving
grouping disabled by default. A native default/explicit comparison isolated
exactly those three missing phases. The test now explicitly requests
`NumeratorGrouping("up_to_scalar")`; every lifecycle assertion is unchanged.
The complete replay passes in 0.554 seconds against the fresh native extension
that passed all 118 maintained controls. Byte comparison establishes that this
one test-only line is the entire difference from its compiled generation source;
no runtime change or rebuild is implied. The initial failure, comparison and
passing receipt remain in `output/diagnostics/hepkit-shared-progress-replay-1/`.

The final installed extension subsequently passed all 118 maintained native
controls. The source-cell scientific run generated 30 sectors in 59.42 seconds;
its first diagnostic writer incorrectly assumed nested covariance and stopped
after QMC. The preserved generation was then cold-loaded, with no regeneration,
for the corrected recorder. This continuation completed the actual notebook's
245,760-point QMC and Havana pilot/8,192-point production cells, retaining four
signed components, all 16 covariance entries and checkpoints. The recorder was
first checked against an actual tiny triangle, including exact QMC and MC
checkpoint restoration. These are bounded execution checks, not new convergence
claims. Evidence is in `gghh-single-notebook-native-2/` and `-native-3/` under
`output/diagnostics/`.

The actual native editor check has partial acceptance only. Initial attempts
identified two driver assumptions: marimo defaults to `auto_instantiate=False`,
and its cell action menu uses ARIA `option`. Neither attempt ran expensive
generation. After those UI corrections, the ordinary Enable execution action
completed generation and compilation in 63.28 seconds, with 110 observed native
progress states; a final bounded attempt took 62.95 seconds with 107 states.
Both closed the automatic progress context, exposed all 30 sector choices,
rendered the four FastSecDec references after compilation, and downloaded their
BibTeX through the visible control. The selected-sector screenshot shows the
native map, prefactor and power table.

The browser driver stopped on an accordion selector ambiguity, then an overly
strict accessible-name selector, before enabling integration. No notebook or
extension error occurred, but Stop/resume in this particular notebook remains
unverified by the editor test. No further generation retry was launched. All
owned processes were reaped and maintained input hashes remained unchanged.
Marimo autosave pruned unused terminal cell-wrapper exports; a strict comparison
confirmed all scientific statements unchanged, and negative controls reject a
changed generation order, nested return or removed downstream dependency.
Original failures, screenshots, DOM, downloaded bibliography and the explicit
partial-acceptance receipt are preserved in
`output/diagnostics/gghh-single-notebook-editor-2/`. This evidence does not claim
Wasm execution or a successful complete editor lifecycle.

The subsequent fresh Wasm delivery passes its generic Pyodide smoke and all
89 maintained portable tests across nine modules, with zero failures or skips.
The build took 27 minutes 30 seconds; generic smoke took 22.50 seconds and the
focused pytest run took 5.60 seconds. Fresh import leaves all four FastSecDec
citation identifiers absent; the suite checks actual use and valid cold loading,
as well as Model bindings and progress callbacks. No full gg→HH browser run was
repeated. All stage processes were reaped and frozen compiled sources stayed
unchanged.

The first generic smoke consumed the current native `DiagramRender` as a string
and failed. A test-only correction uses its public `to_svg()` method and native
typed render settings. Independent review verifies all 29 assertions and import
restrictions are preserved; the correction changes no production implementation
and needs no wheel rebuild. The failed attempt remains in
`output/diagnostics/sector-decomposition-portable-runtime-1/`; the passing receipt
and exact source/artifact identities are in
`output/diagnostics/sector-decomposition-portable-runtime-2/result.json`.
