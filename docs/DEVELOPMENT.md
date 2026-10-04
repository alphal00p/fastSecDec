# Development

Implementation follows [FIRST_PHASE_PLAN.md](../FIRST_PHASE_PLAN.md). The numerical
product is Rust. Python and pySecDec are permitted only in the separate historical
reference environment used to establish baseline evidence.

## Toolchain and dependencies

Rust 1.96 or newer is required by the pinned Symbolica version. On Nix, use
`nix-shell` with a recent nixpkgs channel. The development shell includes the C
toolchain required by existing GMP/MPFR dependency builds; it does not add Python,
FORM, Normaliz, or a generated C++ integrand backend.

Initial local validation uses Rust/Cargo 1.98.1 on Linux x86-64. Cargo.lock records
registry resolution. Local dependency source identities are:

| Checkout | Revision / branch |
|---|---|
| `DO_NOT_PUSH_FOR_REFERENCE_ONLY/worktrees/feynkit` | `8f834d9c62ae06fb327e4ef0b14abffda755b610` |
| `DO_NOT_PUSH_FOR_REFERENCE_ONLY/worktrees/symbolica` | `98794d0d7337ba2b08e4c046dde584ad7fc1ce10` |
| `DO_NOT_PUSH_FOR_REFERENCE_ONLY/numerica` | QMC commit `617f7a56f8f168cd7498177c7db4a40b098eb135` on `codex/havana-qmc`; includes the reviewed numerical fixes and an additive completed-package accessor for checkpoint validation |
| Published SymJIT Rust crate | `2.26.0`, registry checksum in Cargo.lock |
| `DO_NOT_PUSH_FOR_REFERENCE_ONLY/worktrees/oneloopmaster` | Development-only scalar references at community lock revision `a42a60aa5fe0b3ba0a5b9bb37a17c8465c06ba5a`; default features disabled |

The first two checkouts are detached worktrees of the supplied repositories. The
FeynKit worktree has one small literal-substitution fix for kinematic symbols
whose names end in an underscore; its native input regression passes. The
Symbolica worktree has three small local fixes: evaluating fixed-argument external
constants in its error-tracking numeric domain, and preserving parentheses around
complex coefficients in canonical products, and retaining registered aliases
when parsing canonical symbol references. All focused upstream regressions pass;
a fresh-process Gamma artifact test also passes. The patches and reproductions
are recorded under `docs/dependency-patches`.
The original working trees remain unchanged. The root Cargo patches select one
Symbolica/Graphica/Numerica identity across every consumer. Do not use the local
SymJIT checkout's Python/C-ABI manifest as a Rust path dependency.

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

The supplied restricted Symbolica runtime permits one active symbolic
computational thread. Serialize symbolic generation and test processes; no license
settings have been changed. Compiled numerical worker evaluators do not construct
Atoms and have been exercised with caller-owned parallel workers.
The independent Numerica QMC tests do not require Symbolica and also exercise
caller-owned threaded workers.

See [REUSE_AUDIT.md](REUSE_AUDIT.md) for the API/source/probe record. The standalone
Numerica feature branch has its own tests and commits; its dependency revision
is recorded at each reviewed milestone.

The native one-loop reference crate is a development dependency only. It shares
the workspace's Symbolica and SymJIT versions and introduces no Python or Fortran
runtime. The separate ignored `one-loop-reduce` checkout at community lock
revision `b53a70776a43bd14c6562c52a03bc4909568e473` is available for reuse inspection;
it is not currently a FastSecDec dependency.

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
