# FastSecDec bindings for HEPKit

This optional Rust crate owns FastSecDec's PyO3 implementation. The
`symbolica-community` extension links it and registers the public
`symbolica.community.hepkit.sector_decomposition` module. It is not an independent wheel.
The numerical workspace and default CLI do not depend on Python or PyO3.

The native API accepts the existing HEPKit `FeynmanDiagram`, `IntegralFamily`
and `Kinematics` objects and Symbolica expressions. Generation and compilation emit native
snapshots through caller-supplied observers. Integration uses caller-requested
QMC packages or native Havana global batches, with native replay, covariance and
checkpoint validation. The
binding introduces no graph, algebra or numerical integration implementation.

For an existing native diagram, kinematics and regulator symbol (declare a
runtime Gram symbol with `S("kinematics::s", is_real=True)` before using it in
the kinematics):

```python
from symbolica.community.hepkit.sector_decomposition import (
    Integral, CompilationSettings, QmcSettings, StabilitySettings,
)

integral = Integral(diagram, kinematics, regulator=eps,
                    runtime_parameters=[s], model_parameters="runtime")
work = integral.generation_session(
    max_order=0, compilation_settings=CompilationSettings(backend="eager"))
# An explicit Generate/Resume action calls one native unit per UI tick.
work.step(max_units=1, observer=on_generation)
# After work.complete; inspection itself performs no generation or compilation.
generated, template = work.generated, work.kernels
# Build an explicit physical point; defaults are inspectable metadata only.
point = {**generated.runtime_parameter_defaults, s: -1.0}
kernels = template.with_parameters(point, stability=StabilitySettings())
session = kernels.session(QmcSettings(points=4096, shifts=16))
# Only an explicit Integrate/Resume action advances sampling.
snapshot = session.step(max_packages=1, evaluation_batch_size=256,
                        observer=on_integration)
```

The free function `sector_decompose(diagram, ...)` is the same entry point.
`IntegralFamily.sector_decompose(...)` uses the family's native kinematics unless
explicitly overridden. Overrides may add auxiliary-vector assumptions; existing
external products must agree after scalar binding because family construction
may already have substituted them into its denominators. Use `scalar_values`
to specialize stored symbolic invariants consistently, or construct a new native
family for a different fixed point. It requires `powers=[...]` and a scalar `numerator=...`:
one signed power per native denominator, including auxiliary entries. Zero powers
drop a denominator; negative powers multiply that denominator into the numerator.
The family numerator must already include the intended projector and diagram
weights. Both inputs use the normalized Minkowski loop measure
`prod_l d^D k_l / (i*pi^(D/2))`, with an optional extra `measure_multiplier`.
The default integration dimension is `4-2*eps`; the native symbolic tensor
dimension is specialized consistently during parametrization.

The diagram route retains its native numerator, projector and weights and accepts
positive power overrides by native edge ID. It does not accept a second numerator.
Use the existing diagram-expression replacement helper when preparing a different
diagram numerator. Generation returns inspectable sectors and metadata without
compiling kernels or creating an integration session.

`Integral(diagram, kinematics, ...).generate()` remains a synchronous generation
entry, followed by explicit `generated.compile(backend="eager")`. Neither is
called by passive displays. `generation_session()` instead retains native work
through both stages: stop scheduling `step()` to pause and call it again to
resume the same owner. A unit is a chart/cone, mapping/symmetry operation,
representative coefficient expansion, sector evaluator build or final assembly.
An indivisible native algebra operation can exceed a UI tick. Observer `False`
or `KeyboardInterrupt` retains the completed unit; a genuine native error marks
the owner failed. No library thread or worker pool is started.

All generation entrypoints accept the same explicit route selection:

```python
generated = integral.generate(mode="symbolic", subtraction="taylor", progress=None)
work = integral.generation_session(mode="numerical_dual", subtraction="integrate_by_parts")
```

`symbolic` and `taylor` preserve the existing defaults. The alternatives select
native numerical-dual generation and integration-by-parts subtraction; they are
independent of eager/SymJIT compilation and QMC/Havana sampling. The free
`sector_decompose(...)` function accepts these keywords as well. Unknown values
raise `ValueError` before generation observers or parameterization run. Retained
sessions and generated objects expose `.mode` and `.subtraction` without doing
symbolic work, and their rich displays show these settings. In `numerical_dual`
mode, the native coefficient-series composer handles formal endpoint recipes;
`coefficient_expansion` retains the requested choice without changing that
internal recipe engine. This lane retains source charts without symbolic density
symmetry matching. Exact unregulated endpoint admission and signed maps can
require a local symbolic fallback; each sector's `.generation_mode` identifies
the actual route. Compact roots remain cheap recipe views. Only the explicit
`coefficient.expression()` request materializes the actual native expression,
which can be expensive; passive displays do not call it.

`Integral` defaults to independent runtime model inputs. HEPKit resolves analytic
dependencies; named masses remain symbolic before sector support analysis.
Declare real Symbolica kinematic symbols with `runtime_parameters`; their
attributes must be specified when the symbols are first created. The generated object's
`runtime_parameters`, `model_parameter_defaults` (string keys) and
`runtime_parameter_defaults` (native symbol keys) describe the required point.
Explicit `scalar_values` specialize selected inputs; `model_parameters="fixed"`
selects a deliberately fixed model for reference calculations. Widths must be
explicitly zero and runtime masses must remain finite, real and nonzero.
`with_parameters()` returns independent native evaluator owners and preserves
immutable artifact bytes. It never recompiles or mutates the template.

The API remains available in the canonical
`hepkit.sector_decomposition` namespace. HEPKit's object methods are
optional-backend forwarding hooks; all
FastSecDec generation, bindings and numerical work remain in this repository.

Use the maintained [HEPKit example build instructions](../../examples/hepkit/BUILD.md)
to build and install the host wheel. The host's ordinary `community` feature
includes this crate for native and Wasm builds; core-only Symbolica builds remain
separate. Cargo selects the public dependencies without local source patches.
See the [delivery review](../../docs/reviews/regular-hepkit-build.md) for current
validation and publication status. The historical metadata/MC pin,
`a3d09e177196013326fd1532eb938f559be87401`, passes dependency bootstrap and locked
native/portable ownership checks. Its actual portable wheel, built with the
community host at `c9bacce12dd4fffd171aecfbc4f76a2a273d39e7`, passes the generic
Pyodide smoke test and all 58 collected controls from the five maintained
binding, inspection, MC, input and shared-wavefunction modules. No tests failed
or were skipped. The actual browser triangle completed metadata inspection,
QMC checkpoint resume, and same-kernel MC pilot/production resume; its full means,
errors and covariance match the native run. A final supplemental screenshot
scroll failed after the complete reports/checkpoints were saved; that failure
remains recorded. The sole optional browser ggHH attempt reached 30 compiled
kernels in 546.67 seconds, with zero samples. Its report download timed out near
the bounded deadline; no generated artifact, selected-sector inspection or
numerical prefix was accepted. Repeated RPC timeout messages also exposed a
responsiveness limitation during long synchronous generation. The complete
attempt exited cleanly after 596.00 seconds; no retry or browser convergence
claim follows.

That optimized native wheel passed 81 controls and an actual triangle notebook
lifecycle covering metadata, QMC, and Havana pilot/production pause and resume.
Its later source qualifications (one test fixture, an equivalent Option guard,
and presentation CSS) are recorded in the delivery reviews; the portable wheel
includes the published final sources. Hosted native CI for community `b8279dd`
builds the public-pin development-profile wheel and passes all 81 controls plus
the host import check. That workflow installs the documented
`marimo==0.24.2` presentation-test dependency.

Earlier portable evidence at `539019a` covered 50 controls and a triangle
browser lifecycle using showcase `0cf08c6`. It does not validate the newer
endpoint/statistics or discrete-MC APIs. The build guide records compiled and
showcase revisions separately when only notebook assets change.

Python compilation defaults to Symbolica's existing eager evaluator on both
native and portable builds. `CompilationSettings` exposes the native Horner/CPE
controls, with deterministic `cores=1`; an explicit `backend="symjit"` is
available only on native hosts. The CLI/core's default `auto` policy remains
SymJIT O2 natively and eager on portable builds. Native and portable features
are mutually exclusive. The host owns PyO3's ABI and extension-module settings.

On Linux x86_64, select an absolute native linker with
`CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER` before invoking Pyodide, whose
compiler wrappers redirect generic `cc` to Emscripten. The build guide shows
the command; it preserves the separate Wasm target linker.

For isolated Rust development, use ordinary Cargo from the FastSecDec repository:

```sh
cargo test --manifest-path bindings/python/Cargo.toml --locked --lib
```

Use the Rust/toolchain environment described in
[DEVELOPMENT.md](../../docs/DEVELOPMENT.md). This package has a separate workspace
and lockfile so root `--workspace` checks remain Python-free. Do not copy its
Python dependencies into the numerical core.

`scripts/check_dependencies.py` checks native-object ownership in Cargo
metadata. Its default mode requires the actual published Git source for this
crate and all three native library crates. `--allow-local-fastsecdec` explicitly selects
development with local overrides. Metadata is not evidence of a successful
target compilation or runtime. In particular, Cargo can report a union of
native and portable numeric features from target-specific community integrations;
the actual WASM build/runtime is a separate gate.

`scripts/test-native.sh COMMUNITY_CHECKOUT` runs the maintained binding/example
tests and the host's shared wavefunction controls against the installed wheel.
`scripts/test-pyodide.mjs WHEEL_DIRECTORY COMMUNITY_CHECKOUT` runs the small
binding/input/inspection controls in an actual Pyodide runtime selected by
`PYODIDE_DIST_DIR`. The maintained showcase also offers optional browser ggHH;
its cost is separate from these small controls. The corresponding community
workflow invokes its tests from the exact pinned binding checkout.

The public module/class names and native serialized representations remain
available through the legacy import path. The canonical namespace change has
its own [entry-point review](../../docs/reviews/hepkit-sector-entrypoints.md);
the historical wheel results above do not validate the new owner methods.
Compatibility still follows native content,
precision and checkpoint identity validation; moving the wrappers does not
override those checks.

The API retains source-chart endpoint information at
`generated.metadata.charts[i].pre_subtraction`: exact regulator, prefactors and
affine endpoint powers, with the native number of required Taylor subtractions.
These are facts before symmetry multiplicity and subtraction, not a prediction
of surviving poles. An older artifact can lack the record. Expressions remain
Symbolica objects; inspection does not expand the regular density.
`generated.sectors[i].aliased_coefficients[j]` is a cheap retained view of one
compact root and its native alias definitions. Its explicit `expression()`
method restores only that selected coefficient through Symbolica's alias owner;
call it from an Inspect action when a full expression is wanted. Passive rich
views do not perform this restoration.
`kernels.sector_statistics` reports each complete shared evaluator's native
program bytes and pre-SymJIT operation counts. `symjit_ir_bytes` measures the
compressed compiled application, not machine code, and is absent for eager
evaluators. Complex evaluator outputs precede the real/imaginary
component split.

`kernels.mc_session(HavanaDiscreteSettings(...), pilot=True)` starts native
sector and coordinate importance training. The caller advances bounded
`step(max_batches=1, evaluation_batch_size=256)` calls and can pause by retaining
the session object. QMC uses the same native batched weighted-evaluation path.
Native SymJIT handles matrices; eager and high-precision tiers use the native
owner's row evaluation because those owners have no matrix API. Numerical
batch size is operational and can change across checkpoint continuation.
After a complete pilot, `adapt_pilot()` starts another pilot epoch, or
`freeze_production(points_per_batch=..., batches=...)` freezes both grids and
starts independent production. Pilot observations never enter production
estimates. `checkpoint_available` distinguishes in-memory pilot pause from
persistent production checkpoints, restored with `kernels.restore_mc(bytes)`.
Per-sector `planned_points` is `None` for this stochastic allocation;
`discrete_allocation` contains the actual native selection probability and
global batch size. The total planned budget remains an integer. The existing
QMC `session`/`restore` methods and checkpoint format are unchanged.

Both sessions expose `observation()` for accepted full-vector estimates,
complete covariance, per-sector contributions and exact offsets. Their
`live_observation()` is explicitly provisional: QMC uses complete shifted
lattices; Havana uses native point statistics for the current phase, or only
work since a restored checkpoint. Previews never enter convergence or
checkpoints. `StabilitySettings` transports native distance/validated policy,
including optional cutoffs and per-power thresholds. Diagnostic final-class
counts remain separate from attempted evaluator points and matrix invocations.
