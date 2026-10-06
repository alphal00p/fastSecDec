# Development

Implementation follows [FIRST_PHASE_PLAN.md](../FIRST_PHASE_PLAN.md). The numerical
product is Rust. The Python API and marimo notebook belong to the separate HEPKit
community bridge. Historical Python/pySecDec code stays in its external reference
environment for baseline comparisons; it is not a FastSecDec dependency.

## Toolchain and dependencies

Rust 1.96 or newer is required by the pinned Symbolica version. On Nix, use
`nix-shell` with a recent nixpkgs channel. The development shell includes the C
toolchain required by existing GMP/MPFR dependency builds; it does not add Python,
FORM, Normaliz, or a generated C++ integrand backend.

Initial local validation uses Rust/Cargo 1.98.1 on Linux x86-64. Cargo.lock records
registry resolution. Build from a fresh checkout by preparing the exact public
sources and reviewed patches:

```sh
mkdir -p output
./scripts/bootstrap-dependencies.sh "$PWD" "$PWD/output/dependencies"
nix-shell
cargo --config output/dependencies/overlay-root.toml metadata --format-version 1 --locked
```

The bootstrap needs Bash, Git, `sha256sum` (GNU coreutils on macOS), and network
access to the public repositories below. It verifies the four remaining
Symbolica patch hashes before applying them, fetches exact revisions, and refuses
any existing output
directory or symlink. A failed attempt is retained for diagnosis; use a new
directory for a retry. It neither rewrites source manifests nor modifies the
historical reference worktrees. Keep generated sources, configs and build outputs
untracked. Place the new dependency directory either outside the FastSecDec
checkout or beneath its excluded `output/` directory, as in the example. An
arbitrary directory inside the checkout can make Cargo assign dependency crates
to the wrong workspace.

The generated configs contain absolute paths for this checkout. Move neither the
checkout nor the generated source directory without preparing a new config.
Pass the appropriate config explicitly on every Cargo invocation:

| Consumer | Generated config |
|---|---|
| FastSecDec CLI and native workspace checks | `overlay-root.toml` |
| Standalone `tests/portable-kernel` consumer | `overlay-portable.toml` |
| HEPKit community bridge | `overlay.toml` |

The community config also patches FastSecDec to this checkout for development.
Published community integration must pin the published FastSecDec revision and
remove that local FastSecDec patch group to verify delivery from Git. Its build
helper installs the remaining config in a generated Cargo home, scoped to the
build command, so both metadata discovery and compilation inherit it. Follow the
community `examples/hep/FASTSECDEC_BUILD.md` recipe for maturin and Pyodide:
[maturin 1.15's metadata argument builder](https://github.com/PyO3/maturin/blob/v1.15.0/src/cargo_options.rs#L155-L182)
does not forward its `--config` option during metadata discovery.

These consumer-specific overlays omit unused reference/Python patch groups and
packages, keeping lockfile resolution stable. The checked-in manifests name
public dependency sources; the generated config selects the exact patched owners
below. It also sets `FASTSECDEC_{FEYNKIT,SYMBOLICA}_SOURCE_ROOT` for the remaining
local path overrides. The CLI records public Git and registry identities from
its resolved workspace lockfile; only path dependencies need a configured source
root and dirty-source fingerprint. Missing path roots or unreadable identities
fail the build; artifacts never receive an `unavailable` dependency identity.
Do not point these variables at a different checkout from the config's path
patches. The workspace excludes `output` so
generated owners retain their own workspace inheritance.

The root and portable lockfiles have been deliberately resolved with their
respective overlays. Normal builds use `--locked`; dependency updates require a
separate intentional resolution and review of the lockfile and owner identities.
The pinned source identities are:

| Generated owner | Revision / branch |
|---|---|
| `feynkit` from `alphal00p/gammaloop` | Published `feynkit` commit `259df8790f27b8d3ef32778cd7195942691b4ef0`, including native scalar bindings, shared marimo progress and the literal-symbol substitution fix |
| `symbolica` from `symbolica-dev/symbolica` | Public `community` revision `473b4b8dbc2f9bff8658a047196ba0877238bf9e`, plus the four local changes below |
| Published Numerica crate | Ordinary numeric backends, RNG and Havana MC; QMC now belongs to `crates/fastsecdec-qmc`, without a Numerica fork |
| Published SymJIT Rust crate | `2.26.4`, registry checksum in Cargo.lock; latest non-yanked release verified against the registry index on 2026-10-04 |
| `oneloop` from `alphal00p/oneloopmaster` | Development-only scalar references at merged `main` revision `27c3723434b7d99cf70ce612b0b8041d3f5c0e78`, with a compatible SymJIT minimum and source-IR cache handling; default features disabled |
| `one-loop-reduce` from `lcnbr/one-loop-reduce` | Development-only numerator references at `b53a70776a43bd14c6562c52a03bc4909568e473`; default features disabled |

The Numerica QMC branch is published as
[upstream PR #8](https://github.com/symbolica-dev/numerica/pull/8), targeting
`symbolica-dev/numerica:main` from `ValentinHirschi:codex/havana-qmc`.
The user subsequently requested moving QMC into FastSecDec. That branch and PR
remain available as history; production QMC now lives in `fastsecdec-qmc`.
See the [upstream-readiness evidence](reviews/numerica-qmc-upstream-readiness.md).

The updated public Symbolica source already includes the canonical complex
product, attributed alias and literal-series-variable corrections. Their local
patch files and applications have been removed. FeynKit's literal kinematic
substitution and OneLOop's compatible SymJIT minimum are also upstream and no
longer need patches.

Four Symbolica changes remain local: fixed-argument external constant conversion,
structural validation while decoding evaluator IR, fallible coefficient mapping,
and configurable coefficient fields in fixed-variable polynomial conversion.
The [delivery audit](reviews/regular-hepkit-build.md) lists their precise scope.
The fixed-argument fallback assigns nominal output uncertainty; it does **not**
track arithmetic errors inside an arbitrary callback. Its use for fixed native
polygamma values and the outstanding upstream design issue are documented in the
[constant-error audit](reviews/fixed-constant-error-tracking.md).

The local IR validation was prompted by deliberately corrupted serialized
fixtures, not bad generated programs. It checks indices and instruction shapes
before use without introducing a FastSecDec interpreter. The standalone
[literal-series reproducer](../mre/symbolica-literal-series-variable/README.md)
retains its original patch solely as historical evidence. Earlier validated
source identities remain recorded in the individual reviews; they are not
relabeled as tests of the updated dependency revision.

The original user working trees remain unchanged. Development overlays select
one Symbolica/Graphica owner across each consumer. Numerica is the ordinary
registry crate. Do not use the local SymJIT checkout's Python/C-ABI manifest as a
Rust path dependency.

FastSecDec and the developing community bridge now select the same published
FeynKit lineage above, including its shared external-state API. The earlier
`worktrees/feynkit` checkout and its unrelated renderer edits remain untouched.
The current revision also supplies native `DiagramRender` objects and thin
`FeynmanDiagram.sector_decompose()` / `IntegralFamily.sector_decompose()`
forwarders to the optional FastSecDec-owned backend. The integrated native API
passes 95 installed controls and 29 core regressions; generated declarations
preserve the native argument types. See the
[entry-point review](reviews/hepkit-sector-entrypoints.md) for scope and the
separately qualified local wheel build.
Cargo metadata confirms one graph, model, kinematics, tensor, Linnet, Idenso,
Spenso, Symbolica and Numerica owner in each consuming dependency graph; the
identity reports are in `output/diagnostics/bridge-*-identities.json`. The
isolated shared-wavefunction publication and HEPKit PR are recorded in the
[shared-wavefunction review](reviews/shared-external-wavefunctions.md).

The CLI's embedded FeynKit provenance follows the configured owner. An optimized
build review caught and corrected its remaining reference to the older
`worktrees/feynkit` path before running the ggHH example. The three existing
provenance controls pass; `output/diagnostics/gghh-native-release-build-2` also
compares the emitted dependency revisions with Cargo's actual compiled source
owners. The first build is retained as build evidence and was not used for
scientific execution.

Browser evaluator construction also needs the additive native
[`try_map_coeff_with_prec` patch](dependency-patches/symbolica-fallible-coefficient-map.md).
Its ten focused controls pass. It preserves existing mapping behavior and adds
typed error propagation for unsupported constants/callbacks, without a second
evaluator or arithmetic implementation. The
[portable FastSecDec feature](reviews/portable-kernel-feature-split.md) now passes
the native/portable host controls and actual-library Emscripten/Node smoke.
Select exactly one of `native` (default) or `portable`; the latter uses the same
library APIs with Symbolica's interpreter and Malachite/Astro. Its standalone
validation consumer excludes native-only one-loop reference dev-dependencies.

Projective admission of large factored regular numerators also uses the additive
[`to_polynomial_in_vars_with_field` patch](dependency-patches/symbolica-fixed-variable-coefficient-field.md).
It exposes the existing native conversion's coefficient-field policy, preserving
the old default while allowing conservative deterministic zero tests. Its three
native controls and the affected FastSecDec parametrization controls pass.

The earlier latest-release check and source ancestry are recorded in the
[evaluator version review](reviews/evaluator-release-verification.md).
OneLOop's merged source stores Symbolica evaluator IR and recompiles it with
the selected backend; it no longer needs the old exact-pin/cache patch. See
[the historical patch record](dependency-patches/oneloop-symjit-2.26.4.md).
Its optional prebuilt caches are disabled; no master formulas were changed.

Before the existing-IBP evaluator probe, a 2026-10-05 03:32 UTC recheck of the
published crate documentation still reports [Symbolica 3.0.1](https://docs.rs/crate/symbolica/latest)
and [SymJIT 2.26.4](https://docs.rs/crate/symjit/latest). The registry API was
unavailable (HTTP 403); this supplementary check uses the latest documentation
pages, rather than claiming a successful new registry-index query. The pins
remained unchanged at that check; the two additive Symbolica APIs above were
subsequently added for portable evaluation and factored polynomial reuse.

A 2026-10-06 recheck of the same official crate documentation still reports
[Symbolica 3.0.1](https://docs.rs/crate/symbolica/latest), published 2026-09-29,
and [SymJIT 2.26.4](https://docs.rs/crate/symjit/latest). This is a documentation
check, not a registry API query; no dependency pin changes follow.

Standard checks as implementation lands:

```sh
cargo --config output/dependencies/overlay-root.toml fmt -p fastsecdec -p fastsecdec-sectors -p fastsecdec-qmc -p fastsecdec-cli --check
cargo --config output/dependencies/overlay-root.toml test --workspace --locked -- --test-threads=1
cargo clippy --config output/dependencies/overlay-root.toml --workspace --all-targets --locked -- -D warnings
cargo --config output/dependencies/overlay-root.toml tree --locked --duplicates
cargo --config output/dependencies/overlay-portable.toml check --manifest-path tests/portable-kernel/Cargo.toml --locked --all-targets
```

The explicit formatting package list avoids walking the local path dependencies
and reporting their unrelated formatting differences. Tests and Clippy use the
four-member FastSecDec workspace. For the external `clippy` subcommand, put
`--config` after `clippy` so its nested Cargo invocation receives the overlay.

Earlier restricted-runtime attempts required one active symbolic thread and
remain recorded. The latest user-supplied key is accepted by the native
`set_license_key` API, with `is_licensed()` returning true before any community
module import; its outdated-format warning does not mean rejection. The private
environment value is excluded from source and evidence. The status check is in
`output/diagnostics/native-license-status-1/result.json`. Keep performance runs
isolated from competing scientific work. Compiled numerical worker evaluators
do not construct Atoms and have been exercised with caller-owned parallel workers.
The independent `fastsecdec-qmc` tests do not require Symbolica and also exercise
caller-owned threaded workers.

See [REUSE_AUDIT.md](REUSE_AUDIT.md) for the API/source/probe record. The standalone
Numerica feature branch retains the original tests and commits; their QMC
counterparts now live alongside the moved FastSecDec implementation.

The native one-loop master and numerator-reduction crates are development
dependencies only. They share the workspace's Symbolica, SymJIT, FeynKit graph
and kinematics identities and introduce no Python or Fortran runtime. Cargo
source patches also redirect the reducer's Git dependencies to these existing
owners. The [numerator reference review](reviews/hepkit-numerator-reduction.md)
records the native API probe and distinguishes it from complete integration
validation. Neither reference provider restricts the production Gaussian
parameterization to its own reduction or degeneracy limits.

Keep source and tests within their owning subsystem. Current parallel ownership:
native graph ingestion and parametric construction; exact sector geometry;
FastSecDec QMC. The coordinator owns shared contracts, milestone commits, evidence,
and regression mapping. New assignments replace these owners explicitly.

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

The dependency bootstrap was checked against the existing accepted owners,
including all 3,776 tracked source files and symlink identities. Live metadata and
build checks use these freshly fetched owners. Earlier native/Pyodide wheels,
notebook lifecycle runs and native ggHH results keep their original source and
artifact identities; the path migration does not relabel them as freshly rebuilt
runtime tests. See the [dependency-delivery review](reviews/dependency-delivery.md)
for exact gates and launcher issues. The community Git publication gate now
passes in [draft HEPKit PR #18](https://github.com/symbolica-dev/symbolica-community/pull/18):
39 fresh installed-native tests pass with zero skips. Its
[bridge review](reviews/hepkit-fastsecdec-bridge.md) distinguishes the new public-Git
native build from the earlier actual Wasm/browser checks and documents the
remaining upstream dependency prerequisite.
