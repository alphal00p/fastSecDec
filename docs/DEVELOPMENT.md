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
registry resolution. Local dependency source identities are:

| Checkout | Revision / branch |
|---|---|
| `DO_NOT_PUSH_FOR_REFERENCE_ONLY/worktrees/feynkit-fastsecdec-notebook` | `6c707c6b77a437256eb1180da13d4d327b371d13` on local `codex/fastsecdec-hepkit-deps`, plus the existing literal-symbol substitution fix |
| `DO_NOT_PUSH_FOR_REFERENCE_ONLY/worktrees/symbolica` | `98794d0d7337ba2b08e4c046dde584ad7fc1ce10` |
| `DO_NOT_PUSH_FOR_REFERENCE_ONLY/numerica` | QMC commit `e4638da22a17cfa931fa14c6829d3350b7a8de2b` on `codex/havana-qmc`; includes the reviewed numerical fixes, completed-package and shift-coverage access, periodization range checks, and explicit attributed published catalogues |
| Published SymJIT Rust crate | `2.26.4`, registry checksum in Cargo.lock; latest non-yanked release verified against the registry index on 2026-10-04 |
| `DO_NOT_PUSH_FOR_REFERENCE_ONLY/worktrees/oneloopmaster` | Development-only scalar references at community lock revision `a42a60aa5fe0b3ba0a5b9bb37a17c8465c06ba5a`; default features disabled |
| `DO_NOT_PUSH_FOR_REFERENCE_ONLY/worktrees/one-loop-reduce` | Development-only numerator references at community lock revision `b53a70776a43bd14c6562c52a03bc4909568e473`; default features disabled |

The Numerica QMC branch is published as
[upstream PR #8](https://github.com/symbolica-dev/numerica/pull/8), targeting
`symbolica-dev/numerica:main` from `ValentinHirschi:codex/havana-qmc`.
FastSecDec still uses the exact local revision above while that PR is reviewed.
See the [upstream-readiness evidence](reviews/numerica-qmc-upstream-readiness.md).

The first two checkouts are isolated worktrees of the supplied repositories. The
FeynKit worktree has one small literal-substitution fix for kinematic symbols
whose names end in an underscore; its native input regression passes. The
Symbolica worktree has five local fixes: evaluating fixed-argument external
constants in its error-tracking numeric domain, preserving parentheses around
complex coefficients in canonical products, retaining registered aliases
when parsing canonical symbol references, and treating expansion variables and
points literally in the generic function-series fallback, and validating decoded
evaluator IR before native evaluation or export. The series fix addresses a
silently empty Gamma Laurent series for a valid underscore-suffixed regulator;
see the [isolated patch and regression](dependency-patches/symbolica-literal-series-variable.md).
The [native IR validation patch](dependency-patches/symbolica-evaluator-ir-validation.md)
checks native structural invariants without adding a FastSecDec instruction
format or interpreter; its focused native and preserved-byte codec tests pass.
An author-ready [standalone Rust-script reproducer](../mre/symbolica-literal-series-variable/README.md)
pins the unpatched published crate and includes the patch and verified outcomes.
All focused upstream regressions pass;
a fresh-process Gamma artifact test also passes. The patches and reproductions
are recorded under `docs/dependency-patches`.
The original working trees remain unchanged. The root Cargo patches select one
Symbolica/Graphica/Numerica identity across every consumer. Do not use the local
SymJIT checkout's Python/C-ABI manifest as a Rust path dependency.

FastSecDec and the developing community bridge now select the same published
FeynKit lineage above, including its shared external-state API. The earlier
`worktrees/feynkit` checkout and its unrelated renderer edits remain untouched.
Cargo metadata confirms one graph, model, kinematics, tensor, Linnet, Idenso,
Spenso, Symbolica and Numerica owner in each consuming dependency graph; the
identity reports are in `output/diagnostics/bridge-*-identities.json`. The
isolated shared-wavefunction publication and HEPKit PR are recorded in the
[shared-wavefunction review](reviews/shared-external-wavefunctions.md).

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

The latest-release check and exact source ancestry are recorded in the
[evaluator version review](reviews/evaluator-release-verification.md). The
development-only OneLOop worktree has an additional two-line compatibility patch
updating its exact SymJIT pin and compiled-cache identity together; see
[the patch record](dependency-patches/oneloop-symjit-2.26.4.md). Its optional
prebuilt caches are disabled; no master formulas were changed.

Before the existing-IBP evaluator probe, a 2026-10-05 03:32 UTC recheck of the
published crate documentation still reports [Symbolica 3.0.1](https://docs.rs/crate/symbolica/latest)
and [SymJIT 2.26.4](https://docs.rs/crate/symjit/latest). The registry API was
unavailable (HTTP 403); this supplementary check uses the latest documentation
pages, rather than claiming a successful new registry-index query. The existing
pins and five local Symbolica fixes remain unchanged.

Standard checks as implementation lands:

```sh
cargo fmt -p fastsecdec -p fastsecdec-sectors -p fastsecdec-cli --check
cargo test --workspace --locked -- --test-threads=1
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo tree --locked --duplicates
```

The explicit formatting package list avoids walking the local path dependencies
and reporting their unrelated formatting differences. Tests and Clippy use the
three-member FastSecDec workspace.

Earlier restricted-runtime attempts required one active symbolic thread and
remain recorded. The latest user-supplied key is accepted by the native
`set_license_key` API, with `is_licensed()` returning true before any community
module import; its outdated-format warning does not mean rejection. The private
environment value is excluded from source and evidence. The status check is in
`output/diagnostics/native-license-status-1/result.json`. Keep performance runs
isolated from competing scientific work. Compiled numerical worker evaluators
do not construct Atoms and have been exercised with caller-owned parallel workers.
The independent Numerica QMC tests do not require Symbolica and also exercise
caller-owned threaded workers.

See [REUSE_AUDIT.md](REUSE_AUDIT.md) for the API/source/probe record. The standalone
Numerica feature branch has its own tests and commits; its dependency revision
is recorded at each reviewed milestone.

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
Numerica QMC. The coordinator owns shared contracts, milestone commits, evidence,
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
