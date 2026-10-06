# HEPKit sector decomposition entry points

This milestone follows the user's request to use HEPKit's existing structures
and to expose the notebook's actual scientific calls. Its canonical namespace
is `symbolica.community.hepkit.sector_decomposition`.

## Ownership and reuse

`FeynmanDiagram` and `IntegralFamily` remain the native FeynKit-Py classes.
Their optional `sector_decompose()` methods forward to the canonical function;
they introduce no reverse Rust dependency on FastSecDec. Community owns module
registration, imports and generated stubs. FastSecDec owns the optional binding,
input preparation and existing decomposition pipeline. The native numerical
workspace remains independent of Python.

The reuse audit checked the native family constructors and accessors,
`IntegralFamily::sector`, diagram `propagator_family`, family completion and
the existing FastSecDec `ParametricIntegrand::from_family` implementation.
`IntegralFamily.from_diagram` may append auxiliary denominators to complete
the scalar-product basis; assigning all of them unit powers would change the
integral. The new family entry therefore requires a signed power vector and
an already weighted scalar numerator. Native sector selection retains positive
denominators; native Symbolica powers incorporate negative-power factors into
the numerator. Zero powers omit the corresponding factors.

The diagram entry retains native numerator/projector/prefactor/overall-factor
ownership and accepts its existing positive power overrides by graph edge ID.
It requires explicit kinematics. Family input uses its existing assumptions
unless the caller supplies a compatible override. Existing external pair
products must agree after the same scalar substitutions: a changed assumption
cannot undo substitutions already embedded in the family's denominators.
Auxiliary-vector assumptions may be added. Scalar specialization and auxiliary
momentum preparation share the graph path's native operations in
`input::prepare_family_input`; neither Python glue nor the notebook duplicates
those rules. The helper returns a native family, positive powers and a native
Symbolica numerator, without introducing another input class.

## Execution and compatibility

Decomposition returns the existing generated result with sectors, charts and
pre-subtraction metadata. Compilation remains explicit, and sampling starts
only when the caller creates and advances a session. Native observer errors and
cancellation pass through the new owner methods without replacement.

`hepkit.fastsecdec` reexports the canonical objects, preserving class and
exception identity. The existing `Integral(...).generate()` path remains
available and shares the generation implementation.

The notebook visibly defines decomposition, compilation, QMC/MC session
creation and bounded stepping. Its helper owns action/lifecycle state and
presentation. Defining or displaying the functions performs no scientific work.
Generate/Integrate actions and active integration ticks invoke those exact
functions; no illustrative second calculation is introduced.

## Validation status

Independent source review accepts native owner reuse, weights/measure, signed
powers, the kinematics consistency guard and the thin forwarding hooks. The
notebook's 27 focused controls and marimo source validation pass. The corrected
native input/parametrization suite passes 29 tests, with formatting and strict
scoped Clippy. The installed native wheel passes all 95 binding/frontend tests,
with no failures or skips.
Strict binding Clippy also passes for all targets with stub generation enabled.

Those tests exposed a scalar-binding admission issue: a declared momentum can
occur as a Symbolica function head in a tensor dot product, rather than as a
standalone variable. The shared input helper now uses Symbolica's existing
`get_all_symbols(true)` to reject momentum-dependent scalar values consistently.
The regression includes a substitution that would otherwise silently change a
valid quadratic denominator; this is not merely a check of a downstream error.
The initial 93-pass/two-failure observation is retained separately. The other
failure was a test allocation below the published lattice's minimum size.

The actual native marimo run passes: the canonical import and scientific calls
are visible, Generate produces two triangle sectors without sampling, native
diagram rendering and mapped formulas work, and QMC cancel/resume completes.
The same kernels then complete a separate Havana pilot and production run with
pause/resume and native checkpoint downloads. Input edits and selecting optional
ggHH remain inert. The first browser driver raced asynchronous code-editor
mounting; waiting for that component fixed the driver without changing notebook
code. Both runs were reaped, and the successful run's sources stayed unchanged.

Local receipts are retained under `output/diagnostics/`:

- `sector-decomposition-input-correction-2/result.json`: corrected native tests.
- `hepkit-sector-entrypoints-controls-2/controls.json`: installed 95-test suite.
- `hepkit-sector-entrypoints-ui-2/attempt-1/`: actual notebook and screenshots.

The accepted native wheel uses a local final-host `lto=off` validation override;
dependency optimization and native SymJIT O2 remain enabled. It is not a new
performance observation. Inventory-only corrections split two FeynKit class
declarations and use `typing.Union`/`typing.Optional` in the binding metadata:
this stub generator silently renders PEP 604 union annotations as `Any`.
These changes are conditional on stub generation and leave the compiled runtime
body identical; the native evidence explicitly qualifies those source-only
differences. The regenerated inventory passes syntax, native method signatures,
precise diagram/family dispatch types, compatibility identity and native render
type checks. The standalone FeynKit Rust test harness was not run: Cargo rejects
testing a dependency package with development dependencies outside its workspace.
The exact embedded Python forwarding control was instead executed against the
installed native owners, including identity, signatures, observer exceptions and
the missing-backend error. The native runtime and stub inventory both compile.
Actual portable validation remains in progress. Historical Wasm wheel/UI checks
do not establish acceptance of these new entry points.

The minimal FeynKit owner change is published as
[`c6710fe`](https://github.com/alphal00p/gammaloop/commit/c6710fe017815b7540c444741a7bb359bb1da235).
The bootstrap selects that revision. The installed final native wheel only adds
or updates four generated `.pyi` members and its wheel RECORD; independent ZIP
comparison verifies that every tested runtime member is unchanged. Its hash is
`d8efe1f959ee825264df33acfdf31d8f18a398c237e1dab75dddd2e5fd79c960`.
