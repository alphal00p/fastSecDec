# Native HEPKit → FastSecDec notebook

For the complete gg→HH calculation in one file, open [gghh.py](gghh.py):

```sh
python -m marimo edit examples/hepkit/gghh.py
```

It starts from `Model.standard_model()`, specifies masses and zero widths inline,
generates and selects a double box with native HEPKit graph primitives, and
contracts its color/helicity numerator. It then calls
`diagram.sector_decompose(...)`, followed by explicit compilation and QMC or
Havana integration. No parameter card, saved graph or Python helper script is
required. The expensive cells start disabled and use ordinary marimo editor
execution controls. Sector inspection and streamed native results are included
in the same file. Generation and compilation use HEPKit's automatic marimo
progress presenter, also used by `generate_diagrams`. Pass `progress=None` for
silence or a callable to receive every native generation snapshot.
This new walkthrough requires the matching experimental wheel with
`Model.scalar_bindings` and automatic sector progress. The current wheels pass
all 118 native and 89 portable controls; the actual generated graph, both
integration methods and final bibliography cells have also been exercised
locally. The native editor shows live generation, sector inspection and the
bibliography; Stop/resume in this single-file notebook remains unverified.
See the [challenge inventory](../../docs/reviews/gghh-single-notebook.md).

For a browser export, add `--notebook gghh` to the existing
`examples/hepkit/export.py --wheel ... --output ...` command. This selects editor
mode and packages only the community wheel and installation manifest. The
notebook's hidden setup cell verifies and installs that wheel in Pyodide;
it downloads no model, parameter, graph or helper-script bundle.

Both notebooks end with `symbolica.get_citations()` and a BibTeX download. The
bibliography updates after the native computation and uses Symbolica's existing
rich citation objects. FastSecDec records use when generation starts or saved
kernels load; importing it alone does not add references. Its
[software citation](../../citations/fastsecdec.bib),
[pySecDec reference](https://arxiv.org/abs/1703.09692),
[geometric sector method](https://arxiv.org/abs/0908.2897) and
[endpoint subtraction method](https://arxiv.org/abs/hep-ph/0004013) join the
references recorded by the other HEPKit libraries. Citations are cumulative
within the Python process, including previous notebook runs.

## Dashboard with several examples

Run `fastsecdec_showcase.py` with marimo 0.24.2 and a community wheel containing
FastSecDec's experimental bindings. Follow [BUILD.md](BUILD.md) to build the
wheel from the same pinned FastSecDec checkout as this notebook.

```sh
python -m marimo run examples/hepkit/fastsecdec_showcase.py
```

The notebook has separate **Generate** and **Integrate** actions. Editing an
input or opening the notebook does not prepare a graph, contract a numerator,
generate sectors, compile kernels, or start sampling.

The scientific API is `symbolica.community.hepkit.sector_decomposition`.
The notebook displays the actual callable cells with marimo's `show_code`, so
they remain visible in run view as well as the editor. `decompose_input` calls
the native `diagram.sector_decompose(...)` method with its HEPKit `Kinematics`,
Symbolica regulator and dimension, explicit parameter bindings and graph powers.
`compile_sectors` compiles that returned owner. `create_session` shows the native
QMC or Havana settings, and `advance_session` accepts one native work unit.
The lifecycle helper stores UI state, typed events, owners and checkpoints; it
does not construct an alternative integrator or hide these scientific calls.
These functions execute only through the controls below, not as a second demo
calculation alongside them.

The same backend accepts `IntegralFamily.sector_decompose(...)` with an explicit
signed power vector and an already weighted scalar numerator. Powers follow
the native denominator order: zero omits a slot, and a negative power moves its
denominator into the numerator. Auxiliary completion slots never implicitly
acquire power one. A family does not infer a graph weight, projector or measure
convention. The diagram route retains those native graph expressions once.

The canonical namespace, object methods and visible-call refactor require their
matching wheel. That preceding API milestone passed all 95 native controls;
an actual triangle UI run validated the visible calls, inspection, and same-kernel
QMC/Havana pause/resume. See the [entry-point review](../../docs/reviews/hepkit-sector-entrypoints.md)
for its scope. The versioned browser results below describe earlier sources and
do not by themselves validate these API changes.

1. Choose a problem and its kinematics. The four portable inputs are the massive
   triangle, massless box, rank-two box and coupled sunset. Their native fixtures,
   defaults and normalization are retained. Only relevant mass/invariant controls
   are shown.
2. Select **Generate**. HEPKit prepares the native input; FastSecDec generates
   sectors and compiles kernels. Typed native events supply phase progress,
   counts, timings and coefficient-expansion details. Generation stops with no
   integration session and zero sampled points.
3. Inspect the diagram and all-sector overview. **Inspect sector** retrieves the
   selected native sector's compact coefficients, stored alias definitions,
   coordinate maps and geometry. New generated records also retain the selected
   chart/term's native prefactor, exact epsilon-dependent coordinate powers and
   required Taylor subtraction counts. Maps and prefactors use bounded native
   LaTeX previews; exact names remain available in the source disclosure.
   It does not materialize expanded coefficients.
   **Inspect weighted numerator** explicitly requests native tensor contraction;
   merely opening a collapsed panel does not perform it.
4. Select **Integrate**. Only ready kernels can start a session. This action
   enables the refresh clock; each tick advances at most one native QMC package
   or global Havana batch. The clock is absent during generation and while idle,
   paused or complete. **Cancel** stops between steps and
   saves the accepted production checkpoint; **Resume** restores that coverage
   and its numerical replay state. Changing draft physics cannot replace the
   generated input or an active result. No allocation is enlarged automatically.
   After stopping, **New integration** retains the generated input and compiled
   kernels, saves the previous allocation's report, and clears only sampling
   state. Choose another method/settings and select Integrate explicitly; there
   is no need to regenerate gg→HH to compare QMC and Monte Carlo.
5. Read the complete signed Laurent vector, real/imaginary covariance, per-sector
   coverage and highest-order history. Missing uncertainty remains missing.
   The selected-order target is 0.1% and requires complete production; completing
   the allocation does not imply convergence. Download kernels and checkpoints
   through the native codecs after stopping or completion. **Download run report**
   preserves the full native vector/covariance, events, coverage and bound settings
   as JSON; unavailable estimates remain null. This diagnostic export is not a
   restart format: use the separate native kernel and checkpoint codecs to resume.

Generation and compilation are synchronous. Native callbacks provide the live
status boundary; a long operation can delay repaint or interruption. Marimo's
editor offers Stop (interrupt), also Ctrl-I or Cmd-I on macOS; run view has no
interrupt control. Browser interruption additionally needs isolation headers
(see below). Integration active wall time includes
refresh waits and excludes caller-cancelled intervals. Native worker time is
separate. A direct caller harness is not a measurement of interactive end-to-end
latency.

Generated coefficients are labelled by their native signed epsilon orders and
may be complex. Compiled estimates can split these into additional real/imaginary
rows. Coordinate maps describe the density pullback before endpoint subtraction.
A chart with no numerical kernel may be exact, cancelled or truncated; this is
not a separate zero classification. The overview reports operation counts and
exact program bytes from the actual shared complete-vector evaluator. Counts
precede SymJIT's real/complex lowering and optimization; compressed SymJIT
application bytes are not machine-code size. The portable interpreter has no
SymJIT application. Stored alias counts remain available in coefficient detail.
Pre-subtraction metadata records the original mapped prefactor and powers,
before symmetry multiplicity. Its regular-body byte count does not retain the
complete body or certify conditioning. Older artifacts explicitly lack the
new optional record, while retaining their original bytes and identities.

## Optional gg → HH

The optional fifth choice regenerates one Standard Model top double box with
an internal gluon. It fixes √s = 300 GeV, mH = 125 GeV, mt = 172.5 GeV,
cos θ = 4/5 and incoming (+,+) helicities. It uses Feynman gauge, an unnormalized
color projection δ_ab, generated weights/couplings and no spin/color average.
This single contribution is not the full gauge-invariant amplitude.

HEPKit owns diagram generation, routing, external slots, tensor/color algebra
and shared GammaLoop wavefunctions. Exact binary64 component transport and native
Minkowski contractions build the Gram matrix. Internal tensor algebra retains
D = 4 − 2ε. The ordinary domain guard is retained. Native expression copying
preserves the numerator prefactor and applies the evaluated overall factor once.

[The origin manifest](fixtures/gghh/origin.json) binds the model, parameter card
and audited raw diagram to the Rust example. Live generation must reproduce its
stable ID and every physical serialized field. Only the cosmetic generated-order
name differs: live FK015 versus audited FK018. Both are recorded. The assets
contain no kernels, checkpoints or numerical integration results.

Generate prepares through the finite coefficient. **Quick exploration** defaults
to N = 1024, R = 8, Kuo 33002/Korobov-3, seed 20261005 and 1024-point packages.
The optional **Native gg→HH accuracy observation** preset selects N = 32768,
R = 16, HKKN α=3/Korobov-3 and seed 20261007: 15,728,640 points across 30 sectors.
That native eight-worker CLI allocation reached 0.00937% finite-term relative
standard error in 377.530 seconds. This is measured precision at the fixed point,
not a guarantee for edited settings or a browser timing. Both presets expose
editable points, shifts, rule, periodization, seed and caller package size.
Selecting a preset starts no calculation; only Integrate binds the allocation.
The massive
triangle remains the default; gg → HH is an optional extended run in both native
and browser notebooks. The browser uses one CPU and portable interpreted kernels;
it may take substantially longer, and browser completion/cost remain unvalidated.
For native execution, use a release wheel: native O2 describes the kernels, not
the generation library's build profile. The earlier release CLI completed generation in 61.285 seconds
and an eight-worker allocation in 8.781 seconds, with about 1.03% finite-term
relative standard error. Those are separate CLI feasibility measurements, not
notebook timings or an independent amplitude reference.

## Validated native notebook workflow

The relocated optimized release wheel passed all 61 native binding/input and
notebook state/report controls. Actual Chromium interaction with marimo 0.24.2
then completed the triangle and the fixed gg → HH example. Each native server
used one caller worker on one CPU; the browser ran separately. These are single
interactive observations, including presentation and refresh costs, rather than
matched performance benchmarks or eight-worker CLI timings.

| Actual native UI observation | Triangle | gg → HH |
|---|---:|---:|
| Generate click to ready | 0.441 s | 122.489 s |
| Integration active wall time, excluding cancelled pause | 3.932 s | 89.326 s |
| Whole supervised session, including startup and cleanup | 15.079 s | 236.872 s |
| Numerical sectors | 2 | 30 |
| Cancelled accepted prefix | 1,024 points | 4,096 points |
| Completed allocation | 16,384 points | 245,760 points |

Both workflows displayed live native generation counts, the all-sector overview,
compact coefficient/alias and source-chart inspection, and integration coverage,
signed Laurent estimates and uncertainty history. Generate and subsequent
inspection left the session absent with zero sampled points. Cancel preserved
an incomplete accepted prefix; its report stayed identical during the pause,
and Resume preserved every accepted partial through complete production. Input
edits did not change the retained generated physics. All numerical failures were
zero, and every owned server/browser process was reaped.

The triangle finite coefficient agrees with its analytic control to an absolute
difference of 3.14 × 10⁻⁹. All four gg → HH means equal the retained native CLI
means; all 16 covariance entries agree within 1.39 × 10⁻¹⁷ absolute difference.
This checks implementation consistency, not an independent amplitude reference.
The gg → HH finite coefficient in this quick allocation has **1.0273% relative
standard error**: the fixed allocation completes, but its **0.1% target is not
met**. A subsequent higher-statistics native CLI result is recorded separately
in the [gg→HH example](../gghh_double_box/README.md); it does not change this
historical notebook measurement or imply a browser runtime.

Two earlier triangle driver failures are retained: an initial numeric-field
locator failed before Generate, and a hidden duplicate alias label stopped the
next attempt after generation and inspection, before Integrate. Corrected browser
locators and a source-backed replay-state assertion completed acceptance without
changing scientific code. Raw evidence is retained in the ignored
`output/diagnostics/hepkit-notebook-relocation-1/` directory; native runtime
acceptance does not establish current Pyodide or published-Git installation.

A subsequent presentation-only replay of the saved gg → HH report verifies that
the informative imaginary-component history appears first. Recorded zero-valued
histories remain expandable, without claiming symbolic exactness. The view also
uses the native aggregate worker time and exposes native stop details when
available. This replay performs no generation or integration.

The rebuilt native metadata/Monte Carlo wheel subsequently passed all **81**
maintained controls. One actual triangle UI session then generated and inspected
maps, prefactors, endpoint powers and shared evaluator sizes, completed 16,384
QMC points, and used **New integration** to keep the same kernels and prior
report. Havana completed a 2,048-point pilot with in-memory Cancel/Resume, then
an explicit freeze and 8,192 production points with checkpoint Cancel/Resume.
Full vectors/covariances and exact accepted prefixes passed; all processes were
reaped in 24.985 seconds. This is a small lifecycle check, not an accuracy or
performance benchmark. The evidence is under
`output/diagnostics/hepkit-rich-metadata-ui-1/run-1/`. A later CSS-only table
spacing correction is verified by four presentation controls and clearly
labelled saved-data screenshots, without repeating scientific work.

## Explicit browser export

The previously published `a3d09e` wheel passes generic smoke and all **58** collected
portable controls. Its actual triangle browser run validates native metadata
and rendered math, QMC checkpoint resume, and same-kernel Havana pilot/production
pause and resume. All scientific actions and final downloads complete; a final
supplemental screenshot failure is retained and independently qualified without
rerunning science. The [portable review](../../docs/reviews/hepkit-metadata-mc-portable.md)
records the exact gg→HH bounded outcome and interaction limits.

A native wheel cannot run in Pyodide. The exporter packages an existing tested
cp314 Pyodide wheel, the four small inputs and the optional gg → HH helper/input
assets. It does not build dependencies, run notebook cells or certify responsiveness.
The earlier `539019a` Wasm wheel passed the generic smoke test and all 50
portable API/input/inspection/wavefunction controls. Its actual scalar-triangle
browser lifecycle on showcase revision `0cf08c6` is accepted: bootstrap took
21.084 seconds and Generate reached ready in 3.152 seconds, with nine visible
native phase/count states. Inspection, Cancel, an unchanged paused checkpoint
and Resume completed all 16,384 points; the full result/covariance and analytic
control passed. The whole supervised export/browser session took 63.294 seconds,
peaked at 2.289 GB sampled process-tree RSS, and reaped all owned processes.
Selecting optional gg→HH produced no scientific work. These are single observed
UI timings, not performance benchmarks. They validate the published `539019a`
core wheel and `0cf08c6` showcase; newer metadata/API additions need their own
rebuilt wheel and do not inherit this binary's acceptance.

```sh
python examples/hepkit/export.py \
  --wheel /absolute/path/symbolica-3.0.0-cp314-abi3-pyemscripten_2026_0_wasm32.whl \
  --output /absolute/path/new-fastsecdec-site
python examples/hepkit/serve.py --directory /absolute/path/new-fastsecdec-site --port 8000
```

The notebook fetches an explicit manifest, verifies the wheel/archive hashes and
file list, then mounts the package and fixtures at `/fastsecdec-showcase` before
importing them. No repository path is presumed to exist in the browser. The
gg → HH files contain model/card and diagram identity inputs, never generated
kernels or numerical results. Serve the whole output directory; marimo runtime
assets may require internet access.

For the optional extended run, add `--mode edit` to the export command to expose
marimo's Stop (interrupt) action and Ctrl-I / Cmd-I shortcut; code cells remain
hidden initially. The local server sets the isolation headers needed by Pyodide.
On another host, interruption requires HTTPS (or localhost),
`Cross-Origin-Opener-Policy: same-origin` and
`Cross-Origin-Embedder-Policy: credentialless`. The notebook reports when its
view or hosting does not support interruption. No custom signal handler is used.

KeyboardInterrupt is cooperative: generation checks at native event boundaries
and integration checks every 256 points. Long color/tensor algebra calls may
delay it. Cancel instead pauses between accepted packages; interrupted packages
are excluded from accepted coverage, and Resume uses the saved checkpoint.
Reloading the page discards unsaved in-memory work. Long synchronous gg → HH
work can display marimo RPC timeout warnings even while native progress arrives.
The notebook removes its automatic refresh widget while generation runs and
whenever sampling is inactive. **Integrate**, **Resume**, and explicit pilot
actions enable it; Cancel, completion and errors remove it again. This suppresses
automatic refresh traffic, not every table or download request during a long
synchronous call. The prior bounded gg→HH outcome remains recorded in the
portable review; full browser convergence and prompt interruption are not claimed.

## Havana Monte Carlo

Select **Havana Monte Carlo · sector importance** to use the native nested
discrete/continuous importance-sampling grid. Its point count is global:
`points_per_batch × batches`, not a separate allocation for every sector.
The controls expose pilot and production sizes, seed, continuous bins and
probability safeguards. Lattice rules and Korobov periodization are QMC-only.

**Integrate** starts the explicit pilot allocation and stops when it completes.
Choose **Adapt another pilot** to train another epoch, or **Freeze production**
to adapt/freeze both grids and start the configured production allocation.
Pilot values are discarded from production statistics and history. The view
reports actual selected points, native sector probabilities and global batch
coverage without treating each sector's copy of a batch as independent data.

**Cancel** during a pilot retains the same native session; **Resume** continues
it in memory. There is deliberately no persistent pilot checkpoint. Downloads
become available for frozen production, whose accepted prefix and replay state
can be restored through the native codec. Reloading during a pilot loses its
unsaved training state. These new controls require the updated binding wheel;
the recorded metadata/Monte Carlo wheel and triangle lifecycle validate that
version of the interface, before the canonical-namespace refactor above.
Older wheels lack these APIs and cannot substitute for it.

Earlier portable evidence used a Pyodide 314.0.7 wheel in marimo 0.24.2's actual
314.0.0 runtime: 39 bridge/input/wavefunction controls passed, along with the
triangle Run/Cancel/Resume browser lifecycle and one 32-point package for each
other builder. That evidence belongs to the archived pre-relocation sources.
The relocated explicit-control and sector-inspection interface has separate
acceptance gates; an export or earlier wheel result does not validate it.

`showcase/inputs.py` and `gghh.py` compose native input APIs. `state.py` owns only
explicit caller actions. Generation, sector and integration modules present
native objects; they contain no estimator, graph parser or algebra implementation.
Keep existing Symbolica license configuration in the environment, never in this
notebook, source fixtures or exported assets.
