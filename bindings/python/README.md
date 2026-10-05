# FastSecDec bindings for HEPKit

This optional Rust crate owns FastSecDec's PyO3 implementation. The
`symbolica-community` extension links it and registers the public
`symbolica.community.hepkit.fastsecdec` module. It is not an independent wheel.
The numerical workspace and default CLI do not depend on Python or PyO3.

The native API accepts the existing HEPKit `FeynmanDiagram` and `Kinematics`
objects and Symbolica expressions. Generation and compilation emit native
snapshots through caller-supplied observers. Integration uses caller-requested
QMC packages with native replay, covariance and checkpoint validation. The
binding introduces no graph, algebra or numerical integration implementation.

Use the maintained [HEPKit example build instructions](../../examples/hepkit/BUILD.md)
to build and install the host wheel. The host's `experimental-fastsecdec` feature
selects this crate by one exact FastSecDec Git revision. Its default native
backend selects SymJIT O2; the portable feature selects the existing interpreted
WASM backend. Native and portable features are mutually exclusive. The host
owns PyO3's ABI and extension-module settings.

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
`PYODIDE_DIST_DIR`. The ggHH example is native-only. The corresponding community
workflow invokes these files from its exact pinned FastSecDec checkout.

The public module/class names and native serialized representations remain
stable across this ownership move. Compatibility still follows native content,
precision and checkpoint identity validation; moving the wrappers does not
override those checks.
