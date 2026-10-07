# Development

Implementation follows [FIRST_PHASE_PLAN.md](../FIRST_PHASE_PLAN.md). The
numerical product is Rust; the isolated FastSecDec Python binding crate is
linked by HEPKit's community extension. Historical Python/pySecDec references
remain outside the production dependency graph.

## Toolchain and ordinary Cargo builds

Use Rust 1.96 or newer. `nix-shell` provides a compatible Rust compiler and the
native C/GMP/MPFR build tools. The numerical workspace does not need Python,
FORM or a generated C++ integrand backend.

```sh
nix-shell
cargo metadata --format-version 1 --locked
cargo test --workspace --locked -- --test-threads=1
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all --check
cargo tree --locked --duplicates
```

On macOS, if `cc` resolves to MacPorts GCC, use Apple's Clang linker for Rust:
`CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER=/usr/bin/clang cargo test --workspace --locked -- --test-threads=1`.
The GCC-linked test executable on our macOS validation host aborted even for a
minimal `catch_unwind`; relinking with Clang restored panic recovery. This is an
environment setting, not a reason to disable cancellation or worker-panic tests.

Cargo obtains dependencies from their public Git repositories and the registry.
There is no dependency-preparation script, generated path overlay or local
Symbolica source patch. The standard `[patch.crates-io]` entry selects the public
Symbolica Git revision for every consumer of the registry package; it does not
modify that source. Library dependency declarations remain registry-based so
the final consuming workspace owns this single shared Symbolica identity.

The numerical workspace, standalone portable validation package, standalone
Python binding package and community host each own their root manifest and
lockfile. Dependency-local Cargo patch tables are not inherited by consumers.
Use `--locked` for normal builds; deliberately update and review lockfiles when
changing dependency revisions. Old bootstrap-generated Cargo configurations
must not be carried into ordinary-source checks.

| Owner | Selected source |
|---|---|
| Symbolica | Public fork revision `1deccb8538ccb91dc2c1e58fc0a2e900d2276bf4`, including configurable coefficient fields and native evaluator composition; [upstream PR #54](https://github.com/symbolica-dev/symbolica/pull/54) targets `community` |
| FeynKit / Linnet / Spenso ecosystem | Public `feynkit` branch, locked at `259df8790f27b8d3ef32778cd7195942691b4ef0` |
| Numerica and Graphica | Registry 3.0.1 |
| SymJIT | Compatible minimum `2.26.4`; exact resolved release in Cargo.lock |
| OneLOop | Public `main`, locked at `27c3723434b7d99cf70ce612b0b8041d3f5c0e78`; development-only reference provider |
| one-loop-reduce | Public source, locked at `b53a70776a43bd14c6562c52a03bc4909568e473`; development-only reference provider |

The CLI reads public Git revisions and registry checksums from its consuming
workspace's resolved lockfile. No `FASTSECDEC_*_SOURCE_ROOT` variables are needed
for normal builds. Developers deliberately selecting local path dependencies
must supply matching source roots for their source-state fingerprints.

## Portable and Python consumers

Select exactly one arithmetic backend: `native` provides GMP/MPFR and supports
either SymJIT O2 or Symbolica's eager evaluator; `portable` uses the eager
evaluator with Malachite/Astro. The CLI defaults to SymJIT O2 locally. Python
notebooks explicitly choose eager compilation and one caller-owned worker, also
when running locally. `CompilationSettings.backend` selects evaluator generation
without changing expressions or runtime parameter ownership. Portable host tests
exercise that arithmetic backend, but do not establish actual Wasm execution.
The standalone validation consumer excludes native-only reference providers:

```sh
cargo test --manifest-path tests/portable-kernel/Cargo.toml --locked
cargo check --manifest-path bindings/python/Cargo.toml --locked
```

The Python binding crate has its own workspace so root checks remain Python-free.
HEPKit owns the extension-module ABI and wheel. Follow the
[HEPKit build guide](../examples/hepkit/BUILD.md) for native/Pyodide tooling,
installed-wheel tests and notebook export. A matching FastSecDec checkout supplies
examples/tests; it does not configure the host's dependencies.

Provide a Symbolica license through the environment when required by the chosen
workload. Never put it in source or browser assets. Numerical workers evaluate
native programs without constructing Atoms; execution and parallelism remain
caller-owned. QMC tests do not require Symbolica.

## Ownership and compatibility

QMC lives in `crates/fastsecdec-qmc`; Numerica supplies the existing RNG,
numerical types and ordinary Havana Monte Carlo. The earlier Numerica
[PR #8](https://github.com/symbolica-dev/numerica/pull/8) remains historical
implementation evidence, not a required fork.

Native and portable evaluators use upstream Symbolica mapping APIs. Unsupported
numerical domains are checked through native metadata before mapping. The
error-tracking conditioning shortcut is optional; a flagged point without that
shortcut uses the existing increasing-precision rescue. FastSecDec does not
supply a generic special-function-to-error-tracking conversion.

Kernel artifacts retain native evaluator IR and are application caches from a
trusted producer. Envelope, metadata, compatibility and byte-consumption checks
remain; the loader does not certify arbitrary or deliberately rewritten native
instruction streams. The experimental structural-decoder patch is no longer a
build requirement. No faulty IR from the native generator was observed.

The [reuse audit](REUSE_AUDIT.md) and
[ordinary dependency review](reviews/regular-hepkit-build.md) distinguish current
checks from historical patched builds. Original dependency-patch narratives and
the standalone series reproducer remain historical records; their fixes are not
applied by the build. Keep reference checkouts and generated artifacts untracked.

Native one-loop master and numerator-reduction references share the same public
Symbolica and graph owners. They introduce no Python or Fortran runtime and do
not restrict production Gaussian parameterization to their reduction limits.
See the [numerator reference review](reviews/hepkit-numerator-reduction.md).

## Reference comparison

The frozen Pathfinder source revision is
`582d8c7f6dde9bf750750d4c2a2d85a94ce940cd`.
Its external environment can be created in its own directory using
`uv sync --frozen --python 3.12`; building QMCPy needs a C compiler on PATH.
Never add this environment or its packages to FastSecDec dependencies.

Historical timing reports are not performance acceptance evidence for this host.
The [benchmark protocol](BENCHMARK_PROTOCOL.md) defines paired measurements, and
the [regression matrix](REGRESSION_MATRIX.md) records coverage. Pending rows and
unmeasured cases must remain visibly pending.

## Delivery validation boundary

Native tests, portable-feature host tests, actual Wasm execution and installed
Python wheels are separate evidence. Earlier notebook and ggHH results keep
their original source identities; an ordinary dependency migration does not
relabel them as fresh runtime validation. Current delivery is tracked in
[merged HEPKit PR #18](https://github.com/symbolica-dev/symbolica-community/pull/18)
and the [dependency review](reviews/regular-hepkit-build.md).
