# Build the HEPKit notebook environment

HEPKit's community wheel registers FastSecDec through its ordinary community
feature. FastSecDec owns the Rust/PyO3 implementation in `bindings/python`; the
core library and CLI remain Python-free. Use a current community checkout that
links this FastSecDec revision. Record both revisions and the resulting wheel
hash when validating or distributing a notebook. The ggHH workflow requires
`Integral.generation_session(mode="numerical_dual", subtraction="taylor", ...)`
and the retained session, explicit eager backend and runtime parameter APIs.
An older wheel may expose the latter APIs while still lacking numerical-dual
generation; the notebook must not silently switch its generation mode.

The initial host bridge is [community PR #22](https://github.com/symbolica-dev/symbolica-community/pull/22),
commit `882ad55ef41fe9c76bcdec63efe8c831bf2fd7d3`, which pins FastSecDec
`73b0442c4ba7e1e9dd3379f985f80dd11b337291`. That historical pin predates
numerical-dual generation. Use a community host linking the current FastSecDec
revision and its matching public Symbolica prerequisite when building these
notebooks, and verify the native dependency identities before compiling.

## Native wheel

Use Rust ≥1.96 and the native build tools from `nix-shell` as described in
[DEVELOPMENT.md](../../docs/DEVELOPMENT.md). The notebook host uses Python 3.12,
`maturin==1.15.0`, `marimo==0.24.2`, `pytest`, Typst and anywidget. Build the
community extension with its existing release build and feature selection; no
separate FastSecDec wheel or vendored numerical implementation is required.

From the community checkout, activate that environment and run:

```sh
maturin develop --release --locked
```

Then, from the FastSecDec checkout, verify the registered interface and run the
maintained controls:

```sh
python -c 'from symbolica.community.hepkit.sector_decomposition import Integral, GenerationSession, CompilationSettings, StabilitySettings'
python -m pytest bindings/python/tests examples/hepkit/tests
python -m marimo check examples/hepkit/gghh.py examples/hepkit/fastsecdec_showcase.py
python -m marimo run examples/hepkit/gghh.py
```

`bindings/python/scripts/test-native.sh` supplies the full native binding gate
when called with the community checkout. The dependency identity checker must
still confirm a single native owner of HEPKit, Symbolica and tensor objects.
A release extension improves symbolic generation as well as eager execution;
these notebooks explicitly request eager evaluators even on a native host.

## Pyodide wheel

Use the community's existing Wasm build scripts, Python 3.14,
`pyodide-build==0.39.0`, Pyodide 314.0.x and its compatible Emscripten/Rust
`wasm32-unknown-emscripten` toolchain. The host's Wasm feature forwards FastSecDec's
portable implementation. On Linux, capture the absolute native linker before
Pyodide modifies `PATH`; its `cc` wrapper targets Emscripten, while Rust build
scripts must execute on the host:

```sh
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER="$(command -v cc)"
bash scripts/build_wasm_performance.sh output/wasm-wheel
```

The linker environment is local build configuration, never saved in evaluator
artifacts. Run the maintained `bindings/python/scripts/test-pyodide.mjs` gate
against the resulting wheel and matching community source. Then use
`examples/hepkit/export.py` and `serve.py` from the [notebook guide](README.md).
Every packaged helper and wheel is hash-bound; the notebook verifies them before
installation. Exporting does not execute generation or sampling and does not
establish browser feasibility.

Keep the Symbolica license in the private environment, never in source, exported
assets, logs or reports. Preserve both host and FastSecDec revision identities in
validation records. Native test success does not replace a real Pyodide test or
an actual browser interaction check, particularly for native algebra units that
may take longer on one Wasm core.
