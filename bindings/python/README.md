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

For an existing native diagram, kinematics and regulator symbol:

```python
from symbolica.community.hepkit.sector_decomposition import QmcSettings

generated = diagram.sector_decompose(
    kinematics=kinematics, regulator=eps, max_order=0,
    observer=on_generation,
)
kernels = generated.compile(observer=on_generation)
session = kernels.session(QmcSettings(points=4096, shifts=16))
# Advance only when requested by the caller; each call is bounded.
snapshot = session.step(max_packages=1, observer=on_integration)
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

`hepkit.fastsecdec` remains a compatibility reexport with identical class and
exception objects, and `Integral(diagram, kinematics, ...).generate()` remains
available. HEPKit's object methods are optional-backend forwarding hooks; all
FastSecDec generation, bindings and numerical work remain in this repository.

Use the maintained [HEPKit example build instructions](../../examples/hepkit/BUILD.md)
to build and install the host wheel. The host's `experimental-fastsecdec` feature
selects this crate by one exact FastSecDec Git revision. The historical metadata/MC pin,
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

The default native backend selects SymJIT O2; the portable feature selects the
existing interpreted WASM backend. Native and portable features are mutually
exclusive. The host owns PyO3's ABI and extension-module settings.

On Linux x86_64, select an absolute native linker with
`CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER` before invoking Pyodide, whose
compiler wrappers redirect generic `cc` to Emscripten. The build guide shows
the command; it preserves the separate Wasm target linker.

For isolated Rust development, generate the exact owner overlay from the
FastSecDec repository root, using a new destination under excluded `output/`
or outside the repository:

```sh
bash scripts/bootstrap-dependencies.sh "$PWD" "$PWD/output/python-owners"
cargo test --manifest-path bindings/python/Cargo.toml \
  --config "$PWD/output/python-owners/overlay-python.toml" --locked --lib
```

Use the Rust/toolchain environment described in
[DEVELOPMENT.md](../../docs/DEVELOPMENT.md). This package has a separate workspace
and lockfile so root `--workspace` checks remain Python-free. Do not copy its
Python dependencies into the numerical core.

`scripts/check_dependencies.py` checks native-object ownership in Cargo
metadata. Its default mode requires the actual published Git source for this
crate and both native core crates. `--allow-local-fastsecdec` explicitly selects
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
`kernels.sector_statistics` reports each complete shared evaluator's native
program bytes and pre-SymJIT operation counts. `symjit_ir_bytes` measures the
compressed compiled application, not machine code, and is absent for the
portable interpreter. Complex evaluator outputs precede the real/imaginary
component split.

`kernels.mc_session(HavanaDiscreteSettings(...), pilot=True)` starts native
sector and coordinate importance training. The caller advances bounded
`step(max_batches=1)` calls and can pause by retaining the session object.
After a complete pilot, `adapt_pilot()` starts another pilot epoch, or
`freeze_production(points_per_batch=..., batches=...)` freezes both grids and
starts independent production. Pilot observations never enter production
estimates. `checkpoint_available` distinguishes in-memory pilot pause from
persistent production checkpoints, restored with `kernels.restore_mc(bytes)`.
Per-sector `planned_points` is `None` for this stochastic allocation;
`discrete_allocation` contains the actual native selection probability and
global batch size. The total planned budget remains an integer. The existing
QMC `session`/`restore` methods and checkpoint format are unchanged.
