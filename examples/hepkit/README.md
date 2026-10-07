# HEPKit → FastSecDec notebooks

Open the gg → HH study with a current HEPKit wheel containing this checkout's
FastSecDec bindings:

```sh
python -m marimo run examples/hepkit/gghh.py
```

For the same workflow in one transferable file, use
[gghh_complete.py](gghh_complete.py):

```sh
python -m marimo run examples/hepkit/gghh_complete.py
```

This version includes the input, presentation and lifecycle helpers in folded
notebook cells. It imports only the standard library and installed packages;
no neighboring Python scripts, graph fixtures or parameter cards are needed.
Generation and integration still run only after explicit button presses.
The browser exporter accepts `--notebook gghh_complete` and packages only the
community wheel and its manifest, without a helper or input-data archive.

The smaller scalar examples use the same controls:

```sh
python -m marimo run examples/hepkit/fastsecdec_showcase.py
```

Both notebooks use **eager evaluators and one caller on every host**, including
native Python. They do not create a worker pool or compile SymJIT code. Native
HEPKit owns diagrams, routing, projectors and tensor algebra; Symbolica owns
expressions and evaluators; native FastSecDec sessions own generation, sampling,
complete Laurent vectors and covariance. Python helpers provide UI state and
call the public owners. [BUILD.md](BUILD.md) describes matching wheels and browser
exports. Historical notebook measurements do not validate this revised workflow.

## The gg → HH calculation

**Build diagrams** explicitly runs the native Standard Model process
`model.process([21, 21], [25, 25], particle_selection=[6, 21, 25])` at one and two
loops, QED order 2, with initial and final symmetrization. The native whitelist
includes antiparticles. No vertex whitelist removes gluon self-interactions or
the Higgs trilinear coupling. The bridge, self-energy, tadpole and zero-snail
filters are disabled and native zero-flow internal edges are allowed, retaining
the requested complete catalogue including tadpole/zero candidates. Graph names and
ordering belong to HEPKit; the default is the first **one-loop** graph in that
ordering, which need not be a nonzero box. Use the diagram selector to choose a
box or another graph and repeat the workflow.

The initial point is √s = 300 GeV, mH = 125 GeV, mt = 172.5 GeV and cosθ = 4/5,
with incoming (+,+) helicities, color projection δ_ab and zero top/Higgs widths.
The selection is a single diagram contribution, with its native overall weight
and numerator prefactor applied once. It is not the summed gauge-invariant
amplitude. No threshold deformation or absence-of-threshold certificate is
implemented; choosing an admissible physical point remains the caller's task.

The visible native diagram rendering follows the HEPKit
[four-loop numerator notebook](https://hepkit.org/gallery/notebooks/four_loop_numerator.html).
Generate explicitly constructs the projector and performs native tensor/color
contraction with HEPKit's `contract="dots"` mode followed by native `to_dots()`.
The weighted scalar-numerator display uses the same contraction policy. A tensor
expression can have no free indices while still containing unevaluated
contractions; those must be
resolved to scalar products before FastSecDec parametrization.
The raw native tensor numerator and simplified scalar numerator
then appear in separate native lazy-paging panels. Both viewers are retained and
displayed by stable cells until another Generate replaces the input. Sampling,
pause and resume do not recreate their widget models or reset navigation.
Collapsible numerator panels hide their content while keeping both native widgets
mounted; reopening them preserves the current page. Merely opening a panel never
contracts again.

The **Kinematic symbols** panel identifies every `dot_i_j` with its two physical
momentum or polarization vectors and the diagram's external-leg routing.
These are runtime Minkowski products. Integrate binds them at the selected
physical point. A failed generation retains its error and labels any sector
counters as incomplete; it supplies no numerical zero result.

## Explicit actions and retained work

The generated artifact lives **in place in the notebook** as a native evaluator
owner, displayed by the short `artifact = study.run.kernels` cell. Generate,
Inspect and either integration method use these retained owners directly. No
CLI subprocess, intermediate graph file, evaluator save/reload or cache directory
is part of this path. Selecting another numerical point binds an independent
native evaluator state while preserving the generated template. Downloads are
optional explicit exports; closing/resetting the Python or Pyodide kernel releases
its in-memory owners.

1. **Generate sectors** prepares the selected input and creates an inert native
   generation session. gg → HH defaults to **`symbolic` with Taylor subtraction**:
   the native owner substitutes sector maps, performs coefficient-series expansion,
   then builds eager evaluators. Scalar showcase examples also retain their native
   symbolic default. Each UI tick spends a 50 ms caller work budget on retained
   native units, including eager compilation. Progress opens automatically and shows typed
   phases, counts and timings. Completion creates no integration session and
   samples no points.
2. **Inspect** opens a compact overview of the ten largest serialized native
   sector evaluators, followed by the selected sector. Native coordinate maps,
   positive measure, retained pre-subtraction monomial factors, coefficient
   aliases and endpoint powers remain available. The retained factors are not a
   claim about the full integrand's leading term. Chart and kernel indices are
   distinct; charts without a numerical kernel may be exact, cancelled or
   truncated. The selected **Sector ID** and **ε order** (default zero) open
   the actual post-subtraction Symbolica integrand in HEPKit's native lazy
   expression viewer. Its scoped pages let you navigate large expressions
   without exporting the entire expression to HTML; a new Inspect or Generate
   releases the previous viewer. Inspection never regenerates or compiles.
3. **Integrate QMC** explicitly binds a physical point and starts shifted-lattice
   integration. Each tick advances native packages within the same short budget. A QMC mean requires
   complete lattice coverage; uncertainty requires sufficient independent shifts.
4. **Integrate Havana** reuses the generated evaluators, trains two 64-point pilot
   batches and then freezes the requested production allocation. Pilot samples
   never enter the production estimator. Complete global native batches advance
   within the caller budget. The previous allocation remains clearly separated and downloadable.

Native work yields after each 50 ms budget, with a 2048-unit cap; an individual
atomic native unit can overrun that budget. The default 125 ms caller timer (also configurable to 250 ms or 1 s) stays separate
from scientific observations and rich rendering, which refresh once per second
and at explicit actions, phase transitions and completion. Cheap units are grouped
so timer delays do not dominate generation or integration.

**Pause** stops between caller budgets. **Resume** continues the retained generation
or pilot owner, or restores the accepted production checkpoint. Individual native
algebra units and evaluator batches are atomic and may delay the next UI event;
there is no background calculation thread. A numerical error stays an error,
with its native stage, rather than becoming a zero result. Changing controls or
selecting a graph starts no work. Generate explicitly replaces the selected
input; either Integrate action explicitly starts a new allocation.

Model inputs remain runtime parameters. The gg → HH Gram matrix is constructed
from native symbolic COM vectors: only native exact structural zeros are fixed.
In particular, both incoming gluon virtualities are exact zero in native
Kinematics **before parametrization and sector discovery**; they are absent from
the runtime input schema. Setting those virtualities to zero only at integration
would describe a different generated endpoint structure.
Every nonzero momentum or polarization Gram product is an evaluator parameter,
so numerical wavefunction normalizations never become frozen large binary
rationals in symbolic numerators. At Integrate, native contractions compute their values from the
selected √s, mH and cosθ, while mt also sets the chosen Yukawa mass. Model defaults
supply other contributing independent leaves explicitly. This permits different
physical points without regeneration, subject to the native structural mass
constraints. No model point is silently inferred by the numerical evaluator.

The allocation panel controls points, shifts/global batches and seed. The default
is deliberately small for exploration: 1024 points, four shifts/batches,
Kuo-33002/Korobov-3 QMC and a 4096-point QMC package. Packages may cross
shift boundaries; the native owner handles the final tail. Kuo-33002 requires at
least 1024 points; smaller allocations are available only for Havana. Native
evaluation chunks contain at most 256 points. Larger statistical packages avoid
rebuilding the complete native snapshot for each tiny evaluator chunk. Completing that allocation is **not** an accuracy guarantee.
Results retain every signed Laurent coefficient, real/imaginary components and
full covariance. Missing uncertainty remains missing.

After pausing or completion, select **Prepare downloads** under Optional exports
to serialize the native evaluator artifact and diagnostic report on the notebook
caller. Their download links then hold immutable bytes; browser callbacks never
touch native evaluator owners. The accepted checkpoint is also downloadable. The report records the bound runtime
point, native vector/covariance, coverage, settings and events; it is not a
checkpoint format. All mathematical expressions use native Symbolica formatting.
The bibliography and BibTeX download use `symbolica.get_citations()` and reflect
actual native library use in the current Python process.

## Browser export

Use the same source with a matching eager-capable Pyodide wheel:

```sh
python examples/hepkit/export.py --notebook gghh \
  --wheel output/wheels/symbolica-3.0.0-cp314-abi3-pyemscripten_2026_0_wasm32.whl \
  --output output/gghh-notebook-site
python examples/hepkit/serve.py --directory output/gghh-notebook-site --port 8000
```

Both notebooks default to run mode. `--mode edit` optionally exposes ordinary
Marimo source editing. Export packages the chosen wheel and the shared helper
modules with a per-file SHA256 manifest, and executes no cells. Models and the
gg → HH catalogue are generated natively after the explicit action; archived
historical gg → HH fixtures are not used or bundled. The HTTP server supplies
cross-origin isolation headers. Packaging, native execution and actual browser
scientific validation are separate gates; see the current review records for
measured coverage and remaining limitations.
