# Build the experimental HEPKit bindings

Symbolica-community remains the wheel host. FastSecDec owns the optional
`fastsecdec-python` Rust library under `bindings/python`; community only registers
and reexports it when `experimental-fastsecdec` is enabled. There is no separate
FastSecDec wheel. The host's exact Git dependency pin identifies both the binding
source and the notebook/test checkout.

The relocated native binding is validated on Linux: the optimized wheel passes
61 controls, the actual triangle and native gg → HH notebook lifecycles pass,
and generated stubs preserve all 24 public names, including eight inspection
view classes. Community still needs its new exact FastSecDec dependency pin and
the actual Git delivery gate. Its older `be9c3d29` declaration predates this
binding crate. The current portable wheel and browser gates remain pending.

Use Rust 1.98.1, Python 3.12, Git, a C/C++ toolchain, `m4`, `pkg-config` and
`sha256sum`. In the **community checkout**, create/activate a Python environment
and install `maturin==1.15.0`, `pytest` and `marimo==0.24.2`, then run:

```sh
bash scripts/prepare_fastsecdec_dependencies.sh /absolute/path/new-dependencies
CARGO_HOME=/absolute/path/new-dependencies/cargo-home \
  maturin develop --release --locked --features experimental-fastsecdec
bash /absolute/path/new-dependencies/fastsecdec/bindings/python/scripts/test-native.sh "$PWD"
python -m marimo run \
  /absolute/path/new-dependencies/fastsecdec/examples/hepkit/fastsecdec_showcase.py
```

The destination must be outside the community checkout and must not already
exist. The thin helper reads the exact `fastsecdec-python` pin, fetches that
checkout, and delegates owner setup to its `bindings/python/scripts/prepare-community.sh`.
Its command-scoped Cargo home supplies the same owner config to metadata and
compilation, with isolated caches, no copied credentials and no global config
edits. Maturin 1.15 does not forward its `--config` flag to metadata, so the
command-scoped home is required. Keep this setup available for subsequent builds.

Published OneLOop and FastSecDec currently require different SymJIT versions;
the reviewed owner setup aligns them. Optional feature disabling does not by
itself remove Cargo's lockfile-resolution prerequisite. Native and portable
backends are mutually exclusive, and all shared HEPKit objects must retain one
Rust owner. The pinned checkout's `bindings/python/scripts/check_dependencies.py`
checks the resolved metadata.

Use a normal release wheel for gg → HH. An earlier development wheel passed the
input tests but reached only 23 of 30 mapped representatives within a 300-second
capability bound. This was a build-profile interruption, not a domain rejection.
Native O2 kernels alone do not optimize the generation library.

## Pyodide

Use the existing community Pyodide toolchain (Python 3.14, pyodide-build 0.39.0,
Pyodide 314.0.7, maturin 1.15.0, Rust 1.98.0 with wasm32-unknown-emscripten).
The host's Wasm feature forwards FastSecDec's portable backend.

```sh
export WASM_FASTSECDEC=1
CARGO_HOME=/absolute/path/new-dependencies/cargo-home \
  bash scripts/build_wasm_performance.sh /absolute/path/new-wasm-wheel
export PYODIDE_DIST_DIR="$(pyodide config get dist_dir)"
node /absolute/path/new-dependencies/fastsecdec/bindings/python/scripts/test-pyodide.mjs \
  /absolute/path/new-wasm-wheel "$PWD"
```

The community thin test wrapper requires `FASTSECDEC_CHECKOUT` to identify that
same pinned checkout. Keep the generic community wavefunction tests alongside
the binding and demo tests; do not copy scientific implementations back into
the host. The [showcase guide](README.md) explains browser mounting and the
separate runtime acceptance boundaries.

Provide the normal Symbolica license through the private environment. No key
belongs in source, the dependency setup or browser assets. Builds and scientific
checks do not establish performance or convergence on every input/browser.
