# FastSecDec bindings for HEPKit

This optional Rust crate owns FastSecDec's PyO3 implementation. The
`symbolica-community` extension links it and registers the public
`symbolica.community.hepkit.fastsecdec` module. It is not an independent wheel.
The numerical workspace and default CLI do not depend on Python or PyO3.

The native API accepts the existing HEPKit `FeynmanDiagram` and `Kinematics`
objects and Symbolica expressions. Generation and compilation emit native
snapshots through caller-supplied observers. Integration uses caller-requested
QMC packages or native Havana global batches, with native replay, covariance and
checkpoint validation. The
binding introduces no graph, algebra or numerical integration implementation.

Use the maintained [HEPKit example build instructions](../../examples/hepkit/BUILD.md)
to build and install the host wheel. The host's `experimental-fastsecdec` feature
selects this crate by one exact FastSecDec Git revision. The current public pin,
`539019a72622d0997e7ee2da8c21101234df228a`, passes the actual dependency setup,
locked owner checks and native source-equivalence checks. Its dedicated hosted
CI also builds a native development-profile wheel and passes all 61 controls
plus the host import check. The public-source portable wheel passes the host
smoke and all 50 maintained API/input/inspection/wavefunction controls. The
showcase at `0cf08c6c5d77081364195d587bffdd881fbd384c` also passes the actual
browser triangle Generate, Inspect, Integrate, Cancel and Resume lifecycle.
Actual portable ggHH preparation remains pending. These accepted historical
gates do not cover the newer endpoint/statistics and discrete-MC APIs below.
Those APIs pass 81 focused controls against the current optimized native wheel
and an actual triangle notebook lifecycle covering metadata, QMC and Havana
pilot/production pause and resume. Their current portable-target validation is
still pending.
Notebook/assets use a separately recorded showcase revision without
changing the compiled binding revision; the build guide distinguishes those paths.
Its default native backend selects SymJIT O2; the portable feature selects the existing interpreted
WASM backend. Native and portable features are mutually exclusive. The host
owns PyO3's ABI and extension-module settings.

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
stable across this ownership move. Compatibility still follows native content,
precision and checkpoint identity validation; moving the wrappers does not
override those checks.

The current development API retains source-chart endpoint information at
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
