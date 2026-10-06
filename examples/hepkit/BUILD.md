# Build the experimental HEPKit bindings

Symbolica-community builds the wheel and registers FastSecDec's optional Python
module. FastSecDec owns the Rust/PyO3 implementation under `bindings/python`;
there is no separate FastSecDec wheel. The default numerical library and CLI
remain Python-free.

The host currently pins compiled FastSecDec source at
`539019a72622d0997e7ee2da8c21101234df228a` (community commit
`d82eff433f187f818e22d811ef7e1654744447a3`). The showcase in **this checkout** is
maintained separately: its Python helpers, notebook and assets can change
without rebuilding unchanged Rust. Record this checkout's `git rev-parse HEAD` as the showcase revision alongside
the wheel hash and compiled revisions in the export/release receipt. The dependency helper's older notebook is not the updated
showcase with optional browser gg → HH.

## Native wheel and notebook

Use Rust 1.98.1, Python 3.12, Git, a C/C++ toolchain, `m4`, `pkg-config` and
`sha256sum`. In the community checkout, activate a Python environment with
`maturin==1.15.0`, `pytest` and `marimo==0.24.2`. Set `SHOWCASE_CHECKOUT` to the
FastSecDec checkout containing this guide, then run:

```sh
export SHOWCASE_CHECKOUT=/absolute/path/fastsecdec
export FASTSECDEC_DEPENDENCIES=/absolute/path/new-dependencies
git -C "$SHOWCASE_CHECKOUT" rev-parse HEAD
bash scripts/prepare_fastsecdec_dependencies.sh "$FASTSECDEC_DEPENDENCIES"
CARGO_HOME="$FASTSECDEC_DEPENDENCIES/cargo-home" \
  maturin develop --release --locked --features experimental-fastsecdec
bash "$FASTSECDEC_DEPENDENCIES/fastsecdec/bindings/python/scripts/test-native.sh" "$PWD"
python -m marimo run "$SHOWCASE_CHECKOUT/examples/hepkit/fastsecdec_showcase.py"
```

The new dependency destination must be outside the community checkout and must
not exist. The helper fetches the exact binding pin and delegates owner setup
to its `bindings/python/scripts/prepare-community.sh`. Its command-scoped Cargo
home supplies the same configuration to metadata and compilation, with isolated
caches and no global changes. Keep it available for subsequent builds. Maturin
1.15 does not forward `--config` to its metadata command.

The reviewed owner setup aligns the different SymJIT pins required by published
OneLOop and FastSecDec. Disabling an optional feature does not remove Cargo's
lockfile-resolution prerequisite. Native and portable backends are exclusive;
all shared HEPKit objects must retain one Rust owner. The pinned checkout's
`bindings/python/scripts/check_dependencies.py` checks this metadata boundary.
Use a release wheel for costly generation: native O2 kernels alone do not
optimize the generation library.

## Pyodide wheel and static showcase

Use the community Pyodide toolchain: Python 3.14, pyodide-build 0.39.0,
Pyodide 314.0.7, maturin 1.15.0 and Rust 1.98.0 with
`wasm32-unknown-emscripten`. The host's Wasm feature forwards FastSecDec's portable
backend.

On Linux x86_64, capture an absolute native linker **before** Pyodide rewrites
`PATH`. Its generic `cc` wrapper targets Emscripten, while Rust build scripts
must remain native. The host-target setting below leaves the Wasm linker alone.

```sh
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER="$(command -v cc)"
export WASM_FASTSECDEC=1
CARGO_HOME="$FASTSECDEC_DEPENDENCIES/cargo-home" \
  bash scripts/build_wasm_performance.sh /absolute/path/new-wasm-wheel
export PYODIDE_DIST_DIR="$(pyodide config get dist_dir)"
node "$FASTSECDEC_DEPENDENCIES/fastsecdec/bindings/python/scripts/test-pyodide.mjs" \
  /absolute/path/new-wasm-wheel "$PWD"
python "$SHOWCASE_CHECKOUT/examples/hepkit/export.py" \
  --wheel /absolute/path/new-wasm-wheel/symbolica-3.0.0-cp314-abi3-pyemscripten_2026_0_wasm32.whl \
  --output /absolute/path/new-showcase-site
python "$SHOWCASE_CHECKOUT/examples/hepkit/serve.py" \
  --directory /absolute/path/new-showcase-site --port 8000
```

Open `http://127.0.0.1:8000`. Export packages the chosen wheel and local showcase
assets without running cells. The scalar triangle is the default; gg → HH is an
optional larger example. Its complete browser runtime has not been measured.

The local server supplies cross-origin isolation headers. The polished run
export remains the default. An optional export with `--mode edit` exposes
marimo's existing Stop action when the browser supports shared interrupts.
Long algebra operations can delay interruption; the notebook's Cancel acts
between integration packages. See the [showcase guide](README.md) for details.

The community thin test wrapper uses `FASTSECDEC_CHECKOUT` for the pinned binding
checkout; this is separate from `SHOWCASE_CHECKOUT`. Keep the host wavefunction
tests alongside the binding tests and preserve both source identities in reports.
Provide the Symbolica license through the private environment, never source or
browser assets.

The optimized native wheel passes 61 controls and both native notebook examples.
[Dedicated hosted CI](https://github.com/symbolica-dev/symbolica-community/actions/runs/37391742450)
also builds the public-pin development-profile wheel and passes 61 controls plus
its import smoke. The public-source portable wheel now passes the host smoke and all 50 maintained
API/input/inspection/wavefunction controls. Its host test explicitly allows the
relocated crate's registration globals; the function export list is unchanged.
The updated showcase browser and actual gg → HH portable preparation remain
pending. These checks do not certify every browser or input.
