# Build the HEPKit bindings

Symbolica-community builds the wheel and registers FastSecDec through its
ordinary `community` feature for native and Wasm builds. FastSecDec owns the
Rust/PyO3 implementation under `bindings/python`; there is no separate FastSecDec
wheel. The numerical library and CLI remain Python-free.

Use the community feature branch linked from the
[dependency review](../../docs/reviews/regular-hepkit-build.md). Cargo fetches
upstream dependencies directly; no dependency-preparation script or patched
checkout is required. Keep the showcase checkout at the host's exact FastSecDec
Git revision unless a separate receipt verifies a demo-only difference. Record
that revision with the wheel hash. Historical wheels do not validate the latest
source changes; the review records the current test scope.

## Native wheel and notebook

Use Rust 1.98.1, Python 3.12, Git, a C/C++ toolchain, `m4`, `pkg-config` and
`sha256sum`. In the community checkout, activate a Python environment with
`maturin==1.15.0`, `pytest` and `marimo==0.24.2`. Set `SHOWCASE_CHECKOUT` to the
FastSecDec checkout containing this guide, then run:

```sh
export SHOWCASE_CHECKOUT=/absolute/path/fastsecdec
git -C "$SHOWCASE_CHECKOUT" rev-parse HEAD
maturin develop --release --locked
bash "$SHOWCASE_CHECKOUT/bindings/python/scripts/test-native.sh" "$PWD"
python -c 'from symbolica.community.hepkit.sector_decomposition import sector_decompose, HavanaDiscreteSettings, PreSubtractionMetadata'
python -m marimo run "$SHOWCASE_CHECKOUT/examples/hepkit/fastsecdec_showcase.py"
```

The checkout above supplies examples and maintained tests; it does not configure
Rust dependencies. The host manifest and lockfile select the public owners.
Native and portable backends are exclusive, and all shared HEPKit objects retain
one Rust owner. `bindings/python/scripts/check_dependencies.py` checks this
metadata boundary. Use a release wheel for costly generation: native O2 kernels
alone do not optimize the generation library.

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
bash scripts/build_wasm_performance.sh /absolute/path/new-wasm-wheel
export PYODIDE_DIST_DIR="$(pyodide config get dist_dir)"
node "$SHOWCASE_CHECKOUT/bindings/python/scripts/test-pyodide.mjs" \
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

Historical metadata/MC validation: the optimized native wheel passed 81 controls and an actual triangle
notebook lifecycle covering endpoint/evaluator inspection, QMC checkpoint
resume, same-kernel method changes, and Havana pilot/production pause and resume.
Its generated stubs are packaged without changing the tested extension bytes.
That validation used FastSecDec `a3d09e177196013326fd1532eb938f559be87401`;
community `c9bacce` builds the accepted optimized Wasm wheel. Generic smoke and
all 58 actually collected portable controls pass. The workflow-only `b827`
follow-up also passes [hosted native CI](https://github.com/symbolica-dev/symbolica-community/actions/runs/37406327401)
with 81 controls. The actual triangle browser lifecycle passes its scientific
checks; a final supplemental screenshot failure remains explicitly qualified.
See the [current portable review](../../docs/reviews/hepkit-metadata-mc-portable.md)
for exact wheel/runtime identities and the separate bounded gg→HH outcome.
These historical wheels do not provide the newer canonical namespace or native
`sector_decompose()` methods. See the
[entry-point review](../../docs/reviews/hepkit-sector-entrypoints.md) for their
separate validation and source identities.

For the earlier `539019a` binding revision,
[Dedicated hosted CI](https://github.com/symbolica-dev/symbolica-community/actions/runs/37391742450)
builds a development-profile wheel and passes 61 controls plus its import smoke.
That public-source portable wheel passes the host smoke and all 50 maintained
API/input/inspection/wavefunction controls, and the `0cf08c6` showcase passes the
actual browser triangle lifecycle. Those historical checks do not certify the
new metadata/MC APIs, every browser, or the optional gg → HH calculation.
