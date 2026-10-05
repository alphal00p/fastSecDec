# FastSecDec marimo UI review and execution evidence

Scope: the local symbolica-community notebook worktree, specifically
`examples/fastsecdec_showcase.py`, `examples/hep/fastsecdec_views.py`,
`examples/hep/FASTSECDEC_SHOWCASE.md`, and
`examples/hep/export_fastsecdec_showcase.py`. The separate bridge owner retains
Rust/Python APIs, stubs, tests, input builders and native fixture files. This UI
work did not change FastSecDec algorithms or restart the parked IBP diagnostic.
The on-shell triple-box campaign remains closed.

## Boundaries and presentation

The four submitted choices delegate to the existing massive-triangle,
massless-box, rank-two-box and coupled-sunset native builders. A form submission
prepares native input owners; Run explicitly calls native generation and
compilation. The displayed weighted scalar numerator is diagnostic only; it is
not applied again to the input. Normalization is the native measure with no
implicit Euler-gamma, scale or 4π factors. Graph rendering calls the native
`FeynmanDiagram.render()` and displays its returned SVG without another graph
parser or layout implementation.

Generation and compilation are synchronous. Genuine immutable generation
snapshots populate a transition timeline and reported native phase timings.
During QMC, the notebook caller invokes at most one native package per refresh
on the creating Python thread. There is no background worker or integration
implementation in the helper. Cancel stops future scheduling and saves the native
accepted checkpoint, without running an extra package or fabricating a native
cancellation reason. Resume delegates to `Kernels.restore`. KeyboardInterrupt is
shown as caller interruption; other failures retain their original type/stage
and accepted prefix. The native stop reason remains a separate field.

All signed orders, real/imaginary components and joint covariance remain native.
Absent estimates stay absent. The selected-order target uses the largest signed
requested order, requires complete production, and compares each of that order's
component errors against 0.001 times its absolute mean. The native all-component
`meets` result is reported separately. A complete allocation is not automatically
a convergence certificate and does not trigger increased work. Display formatting
uses eight significant digits without changing stored values. History error bars
are the supplied native standard errors, not errors recomputed from sector sums.

The caller integration wall clock includes refresh waits during active execution
and excludes intervals between caller Cancel and Resume. Native generation phase
and worker timing remain separate. Neither callback support nor native timings
are presented as browser responsiveness evidence.

## Checks and fixes

Installed marimo **0.24.2** API signatures and source were inspected before using
forms, refresh, download and output APIs. Its strict notebook check and Python
compilation pass. A presentation-only control verifies signed-order selection,
missing-estimate handling and covariance layout.

Actual Chromium execution caught two issues not found by static checks:

- The form validator receives raw frontend dropdown selections before submission
  converts them to their Python values. The validator now handles that documented
  implementation boundary instead of attempting to hash a selection list.
- A later assignment suppressed marimo's implicit final output. The dashboard
  now explicitly calls `mo.output.replace`.

The initial failed browser attempt and its diagnostic output are retained under
`output/diagnostics/notebook-ui-1`; its tiny native calculation completed, but
that attempt was not accepted as a UI lifecycle test. It prompted the two fixes.
The coordinator independently reviewed lifecycle/assets and initial screenshots;
requested refinements include short labels, proper epsilon superscripts, compact
float formatting, caller wall timing and explicit interruption classification.

The corrected browser test is retained under
`output/diagnostics/notebook-ui-2`. It uses Playwright 1.63.0 / Chromium
153.0.8010.12 against the native marimo server and installed community wheel. It
passed Run, Cancel, checkpoint download, Resume, completion and checkpoint
download again, with no browser page errors. Native SVG rendering succeeded
without installing the separate optional Python `linnet` extension.

The actual submitted point was the equal-mass triangle **m=1, s=−1**, through
**epsilon one**, Kuo33002/Korobov3, N=1024, R=8, seed 20261005 and package size
1024. Native symmetry retained two two-dimensional sectors:

| Observation | Cancelled by caller | Resumed to completion |
| --- | ---: | ---: |
| Accepted points | 4,096 / 16,384 | 16,384 / 16,384 |
| Complete sector shifts | 4 / 16 | 16 / 16 |
| Native stop reason | None | PlannedWorkComplete |
| Native uncertainty | Available | Available |
| Highest requested order | +1 | +1 |
| Selected-order 0.1% target | Not assessed: incomplete production | Met |
| Native all-vector target | Not met: incomplete production | Met |
| Evaluations | 4,096 | 16,384 |
| Rescues / maximum bits | 2 / 256 | 2 / 256 |
| Numerical failures | 0 | 0 |

The displayed native backend was `native_o2`. Its completed displayed means were
−0.46312964 at epsilon zero and 0.30202189 at epsilon one; the highest-order mean
retained in history was 0.3020218928800966 with native standard error
1.4880074505680763e−9. The generation display rounded total time to 0.173 seconds;
active caller integration wall time was 3.49 seconds, including refresh waits.
These are capability observations, not matched performance measurements.

Offline validation of both downloaded native checkpoint envelopes proves equal
configuration, problem and settings, and byte-for-byte preservation of the
accepted partials: each sector's first two 1024-point partials at cancellation
remain the first two of its eight partials at completion. `validation.json` binds
these checkpoints, tested source files, installed wheel and screenshots by
SHA-256. The coordinator independently verified all eleven binding hashes,
configuration/problem/settings/allocations equality, and both sectors' exact
accepted partial prefixes, and accepted this **native lifecycle evidence only**.
It is not WebAssembly or performance acceptance. The browser process was reaped
before releasing the scientific slot to the independent native controls.

Before subsequent presentation-only polish, all three executed Python sources
were copied to `notebook-ui-2/executed-source` and verified against the accepted
bindings. The later display diff and source hashes are retained under
`notebook-ui-2/display-polish`: completed generation's raw timeline is collapsed,
main status/backend labels are friendly while raw values remain in details,
relative errors display as percentages, and the history axis uses an explicit
native-value offset. An isolated chart replay uses thirteen actual recorded
native observations, without invoking science again. Strict/static checks still
pass; the accepted numerical lifecycle evidence remains bound to its original
executed source. The coordinator also visually reviewed the offset-history
replay and accepted its presentation; final whole-notebook display acceptance
will use the portable browser run.

## Browser export and portable execution

The export helper accepts an explicit existing cp314/pyemscripten_2026_0 wheel,
packages the four native DOT fixtures/model and Python helpers, and records an
exact file list and SHA-256 bindings. The notebook fetches and verifies the wheel
and archive, installs the wheel with the existing Pyodide `emfs:` convention,
and mounts the files before importing builders. It does not presume repository
paths exist in a browser. No license key, raw numerical result or native build
artifact is bundled by the notebook source.

The actual portable wheel is SHA-256
`7dc6f4a6df3b3c8c667e5c978a159f41707ac53aabc66344a8715189fb295d34`
(39,475,650 bytes), built with Pyodide **314.0.7**. The installed marimo 0.24.2
frontend uses **314.0.0**; these versions were not silently equated or changed.
The actual browser's recorded successful runtime requests identify 314.0.0.
Exported wheel, archive and eight explicit asset files were verified, installed
and mounted successfully. Native graph SVG rendering and submitted controls
worked with no browser, console or network errors. No private license was
injected into the notebook, wheel bundle or browser harness.

Evidence is under `output/diagnostics/notebook-wasm-browser`. `bootstrap-1`
records the import/form/render gate. `lifecycle-1` stopped before Run because a
harness CSS selector did not match the rendered wrapper. The preserved
attempt performed no integration; selecting the native SVG's own attribute
fixed the harness. No notebook source or wheel changed. `lifecycle-2` then
passed actual portable Run, Cancel, checkpoint download, Resume, completion and
checkpoint download again, with one native SVG and zero browser/console/network
errors. Only this second lifecycle attempt invoked scientific generation/QMC.

The unchanged submitted triangle used m=1, s=−1, highest order +1,
Kuo33002/Korobov3, N=1024, R=8, seed 20261005 and 1024-point packages. Cancel
stopped at **3,072 / 16,384** points: sector zero had two accepted shifts and
sector one had one. The native session correctly reported **no full-vector
estimate**, waiting for coverage; no provisional zero or precision result was
shown. Resume reached **16,384 points and 16/16 shifts**, with native stop
`PlannedWorkComplete`, available uncertainty and actual backend
`portable_interpreted`. Both the highest-order 0.1% target and native full-vector
target were met. The complete displayed Laurent rows were −0.46312964 at order
zero and 0.30202189 at order one, with standard errors 2.4721835e−9 and
1.4880075e−9 respectively. The recorded highest-order history retains
0.3020218928800966 ± 1.4880074505680763e−9. Diagnostics retain two rescues at
256 bits, two conditioning/weighted checks and zero failures.

Offline checkpoint validation proves equal configuration, problem, settings,
allocations and lattice plans, and exact preservation of all accepted partials
and their recorded costs: the two/one partials at Cancel are the prefixes of the
eight/eight partials at completion. Replay identities, policies and verified
flags are unchanged; their observed maxima can grow monotonically as resumed
samples arrive. An initial validator incorrectly demanded unchanged maxima;
that validator is preserved and the corrected check records the actual replay
semantics. No scientific run was repeated for this offline correction.
`lifecycle-2/validation.json` binds eighteen source/artifact/screenshot/checkpoint
files. The five notebook/export sources are archived under `executed-source`.

The browser gate took 17.9493 seconds overall, including 11.9601 seconds for
runtime/package startup. Displayed generation time was 0.339 seconds; active
caller integration wall time was 3.55 seconds, including refresh waits and
excluding the Cancel-to-Resume pause. These are single capability observations,
not matched performance measurements or a general responsiveness guarantee.
Prepared, Cancelled and completed screenshots retain the compact generation
summary, folded events, friendly labels and explicit portable backend. The full
rendered text also retains the Laurent table and history below the completed
screenshot's viewport. The browser was closed and reaped before releasing the
scientific slot to the ggHH owner.

The separate actual Pyodide 314.0.7 bridge run passed **39 tests, zero skips**
(1.66 seconds reported by pytest). Its unchanged test sources establish this
bounded coverage without another numerical campaign (test sources and log are
bound in `notebook-wasm-browser/coverage-source/coverage.json`):

| Example | Actual portable scientific coverage | Limit |
| --- | --- | --- |
| Massive triangle | Generation/compilation, full allocation, analytic finite coefficient, joint covariance, artifact/checkpoint round-trip, cancellation and errors; actual 314.0.0 browser lifecycle above | One default browser point |
| Massless box | Generation/compilation through order +1 and one accepted 32-point package | No complete estimate or convergence claim |
| Rank-two box numerator | Generation/compilation through order +1 and one accepted 32-point package | No complete estimate or convergence claim |
| Coupled sunset | Generation/compilation through order +1 and one accepted 32-point package | No complete estimate or convergence claim |

The bridge suite also verifies all four native input owners, numerator and
normalization conventions, varied kinematics and invalid-control rejection.
The broader baseline Pyodide smoke subsequently passed after its owner updated
the test caller to the published evaluator signature and installed its explicit
NumPy dependency; the wheel and notebook were unchanged. Bridge evidence remains
under `output/diagnostics/bridge-wasm-1`.

The coordinator independently verified all eighteen bindings, the exact two/one
to eight/eight accepted checkpoint prefixes, equal configurations and complete
coverage, and visually reviewed the completed compact dashboard. Portable
lifecycle and this presentation are accepted within the stated scope; the
independent record is `lifecycle-2/coordinator-review.json`.
The gg → HH example has now passed its separate native generation/integration
prerequisite, but remains outside the notebook pending actual Wasm feasibility. No notebook
claim of four-example one-per-mille convergence or matched backend performance
is made.
