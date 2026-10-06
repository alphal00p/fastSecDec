# Independent notebook and high-level API audit

This is the historical audit of the pre-relocation interface. The replacement's
native triangle and gg→HH execution/inspection lifecycles are now accepted in the
[relocation audit](hepkit-relocation-audit.md); current portable acceptance remains
separate. The findings and source identities below are retained as the original
requirements and evidence, rather than a description of the replacement.

## Replacement status — 2026-10-06

The native findings are closed by the relocated implementation in
[`examples/hepkit`](../../examples/hepkit/README.md) and FastSecDec's isolated
binding crate. Community retains registration, dependency linkage and stubs.
The original audit below remains unchanged as historical evidence.

- **Separate execution and native inspection (P1):** actual triangle and fixed
  gg→HH notebook workflows retain generated objects, stop before session
  creation, and expose all-sector statistics and selected compact coefficients,
  aliases and source charts before Integrate. Input edits do not replace the
  generated physics. Cancel/Resume preserves the exact accepted prefix and
  complete Laurent-vector/covariance layout.
- **Live presentation (P1):** actual native browser evidence captures changing
  generation counts before ready, sector inspection and integration updates.
  The gg→HH log contains 102 rendered count observations; its whole-page label
  scan does not establish 102 distinct active phase transitions.
- **Native status detail (P2):** the current view exposes optional `stop_detail`
  and reads aggregate `snapshot.worker_seconds` directly. A separately labelled
  retained-data replay verifies informative imaginary histories first and
  expandable recorded-zero histories without implying symbolic exactness.
  All 354 saved observations remain available; seven focused presentation
  controls pass. This display follow-up performs no scientific calculation.
- **Evidence rebinding (P2):** the relocated release wheel passes 61 native
  API/input/inspection and notebook state/report controls. The new public-pin
  Wasm wheel passes generic smoke and all 50 portable controls. The actual
  explicit browser triangle lifecycle on showcase `0cf08c6` is independently
  accepted with the `539019a` core wheel: nine actual active-stat phase/count
  states, no work on optional gg→HH selection, zero-session inspection,
  cancel/pause/resume and all 16,384 points with exact accepted prefixes.
  Generate took 3.152 seconds and the whole supervised session 63.294 seconds;
  full-vector/covariance and the analytic triangle control passed, with all
  owned processes reaped. Evidence is retained in
  `output/diagnostics/hepkit-relocated-wasm-browser-2/run-1/`. The stale
  planned count of 46 is reconciled from frozen sources (18 API + 6 inspection +
  12 input + 14 wavefunction cases) and the retained 50-pass log, without rerun.
  Earlier pre-relocation Pyodide evidence is not reused as acceptance.

The actual one-caller native triangle session completed in 15.079 seconds and
agrees with its analytic finite coefficient within 3.14 × 10⁻⁹. The one-caller
native gg→HH session completed in 236.872 seconds: Generate took 122.489 seconds,
active integration 89.326 seconds, and 245,760 points completed across 30 sectors
after a retained 4,096-point cancelled prefix. Every owned process was reaped.
All four means agree exactly with the retained native CLI result, and all 16
covariance entries agree within 1.39 × 10⁻¹⁷. These are implementation-consistency
and interactive-capability checks, separate from the eight-worker CLI timings;
they do not establish performance parity or an independent amplitude reference.
The gg→HH finite-term relative standard error is 1.0273%, so its 0.1% target is
not met for that quick allocation; later native precision evidence is separate.
The optional browser gg→HH path reuses the same helper and input assets; its
actual preparation/completion and interruption remain unvalidated.

The subsequent sector-explainability extension captures the original native
mapped prefactor and epsilon powers before symmetry/subtraction. Selected
detail presents exact rational constant/slope and native Taylor requirements,
using bounded Symbolica LaTeX instead of rebuilding expressions. Actual shared
evaluator operations, exact program bytes and compressed SymJIT application
bytes replace alias counts as the overview's size measure. Older artifacts
explicitly lack the optional record. Native metadata/artifact controls pass 17
cases; portable-host metadata and MC/QMC controls pass 41 cases, and notebook
state/report/presentation controls pass 21 cases. The rebuilt native wheel passes
all 81 maintained controls. Its actual triangle UI completes metadata inspection, 16,384 QMC points, same-kernel New integration,
2,048 pilot points with in-memory pause/resume, explicit production freeze and
8,192 production points with exact checkpoint-prefix resume. Full vectors and
covariances pass, with zero numerical failures and every process reaped in
24.985 seconds. A subsequent CSS-only spacing change has separate four-control
and labelled retained-data rendering evidence; it repeats no scientific work.
The accepted older Wasm wheel does not establish portable delivery of these
new APIs.

Source-bound numerical/UI and independent reviews are retained under ignored
`output/diagnostics/hepkit-notebook-relocation-1/{triangle-ui-3,gghh-ui-1}/`.
The later presentation-only source/control/render review is retained separately
under `output/diagnostics/hepkit-history-presentation-1/`. See the
[relocation audit](hepkit-relocation-audit.md) for ownership and delivery evidence.

## Original audit

The audited implementation preserved native scientific ownership, but did not meet the newly requested separate execution stages or post-generation sector exploration. Two priority-one changes were required. This audit read source, existing reports and three retained screenshots, using standard-library file/hash checks only. It ran no notebook imports, builds or scientific code and made no implementation edits. The Korobov2 experiment remains parked.

The audited source is the community notebook worktree at `DO_NOT_PUSH_FOR_REFERENCE_ONLY/worktrees/symbolica-community-notebook`.

## Audited identity and evidence boundary

Reviewed on 2026-10-05. Community HEAD is `a698db6089f811cf48e56aad3ea00b0f01b8a916` on
`codex/fastsecdec-notebook`, with the local modified/untracked ggHH changes listed
below. This is an audit of these exact working-tree bytes, not an assertion that
HEAD alone contains the reviewed notebook. No audited source changed while it was
read. The root owns subsequent plan and bridge-review updates.

```text
 M examples/fastsecdec_showcase.py
 M examples/hep/FASTSECDEC_BUILD.md
 M examples/hep/FASTSECDEC_SHOWCASE.md
 M examples/hep/fastsecdec_views.py
 M python/symbolica/community/hepkit/fastsecdec/__init__.pyi
 M src/fastsecdec/generation.rs
 M src/fastsecdec/input.rs
 M src/fastsecdec/mod.rs
 M tests/test_hep_fastsecdec.py
?? examples/hep/fastsecdec_gghh.py
?? examples/hep/fixtures/gghh/
?? tests/test_hep_fastsecdec_gghh.py
```

Working-tree source bindings (SHA-256; paths relative to that community worktree):

| Source | SHA-256 |
|---|---|
| `examples/fastsecdec_showcase.py` | `5c40379234c090c5ec669fea8da5e1fcfdcdc9be0741d0b14d56dd15cf99ed97` |
| `examples/hep/fastsecdec_views.py` | `283f3ae11b4fee61259d194046c624bedc3381b131a4e3b60be732b44f992a06` |
| `examples/hep/fastsecdec_inputs.py` | `3c30c34f1224909b882097cfe0520ae8ae85d0aa785e215e46431dc7c11c9162` |
| `examples/hep/fastsecdec_gghh.py` | `f904b095dc3bb7118f13540f2f11280c93c015260746eb1c3d3f525a9f6af221` |
| `examples/hep/export_fastsecdec_showcase.py` | `5ecec4bab0f732725ee5042f4e63eee3ed63b3b78f80d7d690d1ddf2eb8cab05` |
| `examples/hep/FASTSECDEC_SHOWCASE.md` | `79c6996d5e340a64bcb97989aa2fe002121f096937999b0b34a1244998cab481` |
| `src/fastsecdec/generation.rs` | `c00ccf694f1e37b3750c91af4ba13c708405b0c44cafbe01f4aa114645e6a978` |
| `src/fastsecdec/kernels.rs` | `ac4f6bc683ec2297c794c8fd4e42dcd25d348c93a005aa9a05bd57eb7b9b7278` |
| `src/fastsecdec/session.rs` | `18e516763ca7df39d52e954600fef3bddd33cd86cd15e52651131344a414f17c` |
| `src/fastsecdec/status.rs` | `33caa36eb6cf63f4a26d8a961ce17a7966c7d8e2405134d6caa62cae26b71acd` |
| `python/symbolica/community/hepkit/fastsecdec/__init__.pyi` | `376229d8b98452c2c25b47da2e7fc2bd881e8b3a5975653d211d498c034e6065` |

The retained live-browser evidence and the current sources have separate
identities. `output/diagnostics/notebook-wasm-browser/lifecycle-2/validation.json`
and `coordinator-review.json` bind the previously executed portable triangle
lifecycle. `output/diagnostics/hepkit-gghh-notebook-1/portable-boot/browser-result.json`
records the newer boot/render check with `lifecycle_requested=false`.
`advanced-initial.png` predates the latest compact fixed-point cards.
The inspected screenshots and all source files above are additionally bound in
`output/diagnostics/notebook-ui-independent-audit-1/audit.json`.

The newly completed optimized wheel is a separate pre-migration build, SHA-256
`8f291c65b45b326e815d3309b72c81e5c8c114151beb334aeeff6b1a4c5b758c`, recorded in
`hepkit-gghh-notebook-1/release-build-2-result.json`. Compilation succeeded;
installation and focused migration-baseline controls were separately authorized
and are outside this audit. Neither that build nor this source review proves a
new ggHH generation/integration lifecycle. Existing CLI physics evidence is
retained with its original identities.



## Prioritized findings

**P1 — Separate Generate and Integrate.** `fastsecdec_views.py:152–180` performs native generation, compilation, session creation and sets `active=True` in one `start()` call. The next refresh samples automatically. Both the basic Run button and advanced “Generate gg → HH and run” call this path (`fastsecdec_showcase.py:203–209,337–370`). There is no resting state in which the user can inspect generated sectors before deciding to integrate.

The coordinator's clarified contract is: Apply validates/stores inputs; Generate explicitly prepares the native input, generates **and compiles**, retains both `GeneratedIntegral` and `Kernels`, and stops with no session and zero samples. Integrate creates the session and starts caller-owned package scheduling. It must never regenerate or compile silently. Cancel/Resume preserve accepted native checkpoints. Input changes need an explicit stale-input indication or deliberate invalidation, never an automatic calculation. Preserve the configuration associated with the generated result separately from draft controls.

**P1 — Native sector inspection is missing at the bridge and UI boundaries.** `src/fastsecdec/generation.rs:129–155` currently exposes only orders, sector count, exact coefficients, a snapshot and compile. `Kernels` similarly exposes aggregate identities/counts. The notebook discards the generated owner. Its only sector table (`fastsecdec_views.py:85–92,303`) is integration coverage, available after a session exists; it contains no generation semantics or individual detail.

The required metadata already exists in the Rust core: `GeneratedIntegral::{metadata,sectors}`, `GeneratedSector::{dimension,parameters,map,aliased_coefficients,conditioning_basis,cancellation_terms}`, and `GenerationMetadata::{domain_assessment,charts}`. Add thin immutable native views and keep the generated owner alive. An overview should list every numerical sector, dimensions/orders, chart associations and compact complexity/conditioning metadata actually available from the owner. Selecting one sector should expose its parameter map, measure, representative relations and compact coefficient roots/aliases. Do not parse kernel artifact JSON into a parallel Python model. Do not call `GeneratedSector.coefficients()` for the overview: that explicitly materializes shared expressions. A collapsed accordion is not lazy Python execution; populate expensive selected detail only upon an explicit user action.

Metadata semantics must remain precise. `ChartRecord.kernel_sector=None` means no numerical kernel at the requested orders; the core does not separately identify exact, cancelled and truncated contributions there. Do not label these charts “zero.” Coordinate maps describe the density pullback **before** endpoint subtraction, rather than the entire compiled coefficient vector. Folded exact coefficients belong to the complete integral and must not be invented separately for each chart.

**P2 — Apply inputs has a hidden algebra cost.** The numerator accordion at `fastsecdec_showcase.py:159–162` eagerly calls `prepared.scalar_numerator()`. That invokes native weighted tensor contraction and conversion to dot products (`fastsecdec_inputs.py:46–57`) even while the accordion is closed. The wording “prepares the graph only” is inaccurate. The four small inputs are bounded today, but this pattern defeats the requested explicit expensive-action boundary. Defer the operation to an explicit inspect action or reuse already retained native output; opening the notebook, editing controls, Apply and ordinary refresh must not trigger contractions or generation.

**P2 — The generation display omits useful native progress and lacks current long-operation repaint evidence.** `generation_view` shows stage, elapsed time, sector/kernel counts and a compact timeline, but ignores `GenerationSnapshot.detail` and the already exposed coefficient-expansion substage/representative progress. The main progress area can use native completed/total counts when available and preserve “unknown total” otherwise; raw implementation counters can remain in expandable details. Do not invent percentages from unrelated stages.

The Rust bridge generation and compilation calls are synchronous and check signals at observer boundaries (`src/fastsecdec/generation.rs:25–44,101–115`). Calling `mo.output.replace` from an observer establishes event forwarding, not browser repaint or interrupt responsiveness during a long operation. Existing short triangle lifecycle evidence does not certify that stronger behavior. Keep this limitation visible until a bounded actual UI control proves intermediate frames. Avoid moving `unsendable` native owners across threads to obtain animation. Replay of retained events is useful for layout validation, but must be labelled as replay and cannot certify live responsiveness.

**P2 — Surface the available native integration detail.** The existing integration dashboard correctly shows accepted/planned points, complete shifts, the full Laurent vector, covariance, backend, selected-order precision, rescue diagnostics and sample history. Its details omit `IntegrationSnapshot.stop_detail` and aggregate `worker_seconds`, although both already exist in the typed API. Include a native failure explanation when present, and distinguish worker execution time from caller wall time, which includes refresh waits. Do not recompute uncertainty, covariance or a convergence decision from sector sums in Python. Selected-order relative-error formatting is presentation, not a replacement estimator.

**P2 — Rebind acceptance to the new UI and ownership layout.** The retained actual Pyodide triangle lifecycle proves old-source Run/Cancel/Resume, full coverage and exact checkpoint-prefix preservation. The current portable boot demonstrates imports, controls and native rendering only; its record explicitly says `lifecycle_requested=false`. The advanced native screenshot predates the current compact fixed-point cards. Neither proves the forthcoming split controls, sector inspection, current ggHH numerical lifecycle or a ggHH browser execution. Preserve those distinctions in documentation and relocate/rebind notebook helpers, fixtures, export manifest and tests along with any owner migration; do not silently reuse old screenshots as current-source acceptance.

## Scientific and ecosystem boundaries that pass this review

The four builders hand actual native HEPKit model, diagram and kinematics objects to `Integral`; they use native DOT loading and rendering. Tensor simplification, dot conversion, scalar products, Symbolica substitution, generation, evaluators, lattice work, vector estimates, covariance and checkpoints remain native. The helper code contains presentation and caller scheduling, not a second CAS, graph parser, QMC implementation or estimator.

The ggHH builder retains the previously audited exact physical point: √s300, mH125, mt=ymt172.5, cosθ4/5, incoming ++, unnormalized colour delta, Feynman gauge, internal D=4−2ε and four-dimensional external states. HEPKit generates the graph; the saved raw object is an identity guard, with only the cosmetic name excluded. Amplitude legs supply ports; shared GammaLoop wavefunctions and native tensor contractions supply the polarization products. The full model/numerator/projector/prefactor/weight identities and 180 bindings have existing accepted evidence. Nothing in this audit re-ran that algebra or independently recertified a new wheel.

The fixed-SM `_scalar_values` helper composes public native Model and Symbolica operations and matches the accepted CLI boundary. It is not an alternate CAS. It does duplicate parameter-binding policy and should not silently become a general model resolver. Any consolidation must first audit the ecosystem's actual APIs and preserve internal analytic expressions, explicit card overrides and exact binary numerical transport. The proposed ownership migration moves FastSecDec-specific bindings, demo/helpers/assets and detailed tests into FastSecDec, with thin dependency/registration/stubs and shared HEPKit primitives remaining in community. The coordinator owns the final layout; see the companion binding-ownership review. The notebook work must follow that ownership split rather than adding further FastSecDec-specific implementation to community.

The current native/browser boundary is honest: ggHH imports and assets are excluded from the browser export; the advanced entry is visibly native-only; the domain check remains active. Native CLI timings are labelled historical release-CLI measurements, separate from notebook timings. Native `native_o2` and portable `portable_interpreted` labels derive from the actual compiled backend. The reported finite coefficient precision remains a result of an actual allocation, with missing estimates left absent and no fabricated zero.

## Acceptance tests for the replacement UI

1. Open the native notebook and exported browser page; edit every control, Apply, wait through refreshes, select a sector and open ordinary details. Instrument the orchestration boundary to prove zero generation/compile/session/step calls until the corresponding explicit action. Native runtime/bootstrap installation is a separate startup cost.
2. Generate a small native fixture once. Assert one generation and one compile, retained native owners, every sector in the overview, no session and zero accepted samples. Wait several refreshes: nothing scientific advances. Repeat the applicable control in the actual portable browser with its bound wheel.
3. Inspect one sector. Assert stable source/chart/kernel IDs, native coordinate-map semantics and compact alias data, unchanged exact offsets and no materialized coefficient traversal for other sectors. Include symmetry representatives and charts without numeric kernels; distinguish unknown per-chart exactness rather than displaying zero. Verify details collapse/expand without recomputation or sampling.
4. Click Integrate. Assert no generation or compilation, the chosen fixed settings, one accepted native package per refresh, complete signed-order/component layout and native covariance. Cancel must stop future packages; Resume must retain exact accepted prefixes and replay state. An incomplete estimate stays absent. Input edits must not relabel an existing result.
5. Exercise native generation interruption, Python observer failure and a native integration failure. Preserve the original stage/type/detail and any accepted prefix. Inspect/download actions must remain non-sampling. Repeated Generate/Integrate clicks must not create overlapping hidden runs.
6. In the actual UI, capture at least one intermediate native generation state before completion and an integration update, then the generated-only overview, expanded sector detail and final result. Record source/wheel/runtime hashes and zero browser errors. If live repaint cannot be demonstrated, state that limitation rather than presenting replay as live.
7. Check the fixed ggHH entry's actions/labels without triggering the expensive calculation. Before claiming a new numerical ggHH lifecycle, require the coordinator's bounded native gate against the frozen physical input, complete complex vector and covariance. Do not launch it merely to obtain screenshots; retained-data rendering is sufficient for layout review. Browser ggHH remains disabled/unvalidated.

No performance campaign or larger allocation is required by this audit. The next work is the explicit execution state, lazy native inspection surface, thin owner boundaries and their small scientific/UI controls.
