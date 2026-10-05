# Portable kernel validation

This standalone consumer builds the real `fastsecdec` library with its portable
feature. It avoids the main crate's native one-loop reference dev-dependencies,
which intentionally enable GMP/MPFR and SymJIT. It contains no alternate
integration or evaluator implementation.

The seven integration-test targets point directly to the existing public test
files. Their fixture paths remain relative to those files, and the fresh-process
artifact test retains its exact child-test name. The small executable generates
a complete complex Gamma vector through `NativeNamed`, compiles and reloads the
ordinary artifact, checks weighted multiprecision replay, and runs 4,096 points
through the caller-driven QMC API. It checks full covariance and an analytic
leading integral. This is a correctness smoke, not a performance benchmark.

With the development Rust/native build environment active:

```sh
cargo test --manifest-path tests/portable-kernel/Cargo.toml --locked -- --test-threads=1
cargo run --manifest-path tests/portable-kernel/Cargo.toml --locked
```

`portable` is the default here; production FastSecDec still defaults to `native`.
The features are mutually exclusive. The normal workspace validates native O2
and its unchanged legacy/version-three identities. Portable artifacts use their
own codec/policy/hash identity and reject native legacy artifacts before loading
programs.

A library consumer selects it with `default-features = false` and
`features = ["portable"]` on its FastSecDec dependency. Do not combine the native
and portable numeric backends with `--all-features`.

For the actual-library Wasm smoke, activate a Rust Emscripten target and the
matching Emscripten SDK, then build this same consumer for
`wasm32-unknown-emscripten`. The existing isolated feasibility probe supplies the
pinned local toolchain and exception/memory flags used in the retained validation
record. Only the executable is cross-compiled; the host subprocess/tempfile test
harness is not a browser API.

```sh
RUSTFLAGS='-C link-arg=-fwasm-exceptions -C link-arg=-sALLOW_MEMORY_GROWTH=1 -C link-arg=-sENVIRONMENT=node -C link-arg=-sSTACK_SIZE=8388608' \
  cargo build --manifest-path tests/portable-kernel/Cargo.toml --locked \
  --target wasm32-unknown-emscripten --bin fastsecdec-portable-kernel-validation
node tests/portable-kernel/target/wasm32-unknown-emscripten/debug/fastsecdec-portable-kernel-validation.js
```

The last path assumes the default Cargo target directory; use the configured
target directory when `CARGO_TARGET_DIR` is set.

The portable host gate passed all 39 reused public tests, and this executable
passed its complete-vector, 512-bit weighted replay and QMC controls. The native
kernel gate separately passed 52 tests with eight existing diagnostics ignored.
Evidence and source identities are recorded in
[`portable-kernel-feature-split.md`](../../docs/reviews/portable-kernel-feature-split.md).
The actual-library Emscripten build and Node smoke also passed, including the
same 512-bit replays and all 4,096 QMC points. Its complete mean/covariance record
matched the portable host result exactly. A Node smoke does not establish
browser responsiveness or Python-wheel integration.
