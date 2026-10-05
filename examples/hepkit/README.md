# Native HEPKit → FastSecDec notebook

Run `fastsecdec_showcase.py` with marimo 0.24.2 and a community wheel containing
FastSecDec's experimental bindings. Follow [BUILD.md](BUILD.md) to build the
wheel from the same pinned FastSecDec checkout as this notebook.

```sh
python -m marimo run examples/hepkit/fastsecdec_showcase.py
```

The notebook has separate **Generate** and **Integrate** actions. Editing an
input or opening the notebook does not prepare a graph, contract a numerator,
generate sectors, compile kernels, or start sampling.

1. Choose a problem and its kinematics. The four portable inputs are the massive
   triangle, massless box, rank-two box and coupled sunset. Their native fixtures,
   defaults and normalization are retained. Only relevant mass/invariant controls
   are shown.
2. Select **Generate**. HEPKit prepares the native input; FastSecDec generates
   sectors and compiles kernels. Typed native events supply phase progress,
   counts, timings and coefficient-expansion details. Generation stops with no
   integration session and zero sampled QMC points.
3. Inspect the diagram and all-sector overview. **Inspect sector** retrieves the
   selected native sector's compact coefficients, stored alias definitions,
   coordinate maps and geometry. It does not materialize expanded coefficients.
   **Inspect weighted numerator** explicitly requests native tensor contraction;
   merely opening a collapsed panel does not perform it.
4. Select **Integrate**. Only ready kernels can start a session. Each refresh
   advances at most one native package. **Cancel** stops between packages and
   saves the accepted checkpoint; **Resume checkpoint** restores that coverage
   and its numerical replay state. Changing draft physics cannot replace the
   generated input or an active result. No allocation is enlarged automatically.
5. Read the complete signed Laurent vector, real/imaginary covariance, per-sector
   coverage and highest-order history. Missing uncertainty remains missing.
   The selected-order target is 0.1% and requires complete production; completing
   the allocation does not imply convergence. Download kernels and checkpoints
   through the native codecs after stopping or completion. **Download run report**
   preserves the full native vector/covariance, events, coverage and bound settings
   as JSON; unavailable estimates remain null. This diagnostic export is not a
   restart format: use the separate native kernel and checkpoint codecs to resume.

Generation and compilation are synchronous. Native callbacks provide the live
status boundary; a long operation can delay repaint or interruption. Use marimo's
interrupt control during those phases. Integration active wall time includes
refresh waits and excludes caller-cancelled intervals. Native worker time is
separate. A direct caller harness is not a measurement of interactive end-to-end
latency.

Generated coefficients are labelled by their native signed epsilon orders and
may be complex. Compiled estimates can split these into additional real/imaginary
rows. Coordinate maps describe the density pullback before endpoint subtraction.
A chart with no numerical kernel may be exact, cancelled or truncated; this is
not a separate zero classification. Stored alias counts are not a count of
unique expression complexity.

## Native gg → HH

The native-only fifth choice regenerates one Standard Model top double box with
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

Generate prepares through the finite coefficient. Integrate uses N = 1024,
R = 8, Kuo 33002/Korobov-3, seed 20261005 and 1024-point packages. Pyodide cost
has not been validated, so gg → HH is omitted from the Pyodide selector. Use a
release wheel: native O2 describes the kernels, not the generation library's
build profile. The earlier release CLI completed generation in 61.285 seconds
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
The gg → HH finite coefficient still has **1.0273% relative standard error**:
the fixed allocation completes, but its **0.1% target is not met**.

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

## Explicit browser export

A native wheel cannot run in Pyodide. The exporter packages an existing tested
cp314 Pyodide wheel and the four portable inputs; it does not build dependencies,
run notebook cells or certify responsiveness.

```sh
python examples/hepkit/export.py \
  --wheel /absolute/path/symbolica-3.0.0-cp314-abi3-pyemscripten_2026_0_wasm32.whl \
  --output /absolute/path/new-fastsecdec-site
python -m http.server --directory /absolute/path/new-fastsecdec-site 8000
```

The notebook fetches an explicit manifest, verifies the wheel/archive hashes and
file list, then mounts the package and fixtures at `/fastsecdec-showcase` before
importing them. No repository path is presumed to exist in the browser. The
native-only ggHH helper/assets are excluded. Serve the whole output directory;
marimo runtime assets may require internet access.

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
