# FastSecDec implementation guidance

Read `FIRST_PHASE_PLAN.md` before changing scope or architecture. Its original
prompt, subsequent user requirements, and acceptance gates remain authoritative.

- Implement the first phase in Rust. Use HEPKit's native graphs and Linnet
  primitives; reuse Symbolica/Numerica algebra, numeric types, and evaluators.
- Before new algebra or graph helpers, check public APIs, source/tests, and a
  focused Rust probe. Record the reuse evidence and actual missing operation.
- The root agent primarily coordinates. Delegate implementation, research,
  independent review, debugging, and performance tasks to subagents with explicit
  file ownership. Do not edit another active agent's files without coordination.
- Keep FastSecDec on local `main` and commit validated milestones. Numerica QMC
  belongs to its separate `codex/havana-qmc` branch. Do not push.
- Never track `DO_NOT_PUSH_FOR_REFERENCE_ONLY`, build artifacts, caches, or raw
  benchmark output. Keep dependency fixes small and evidence-based; preserve
  existing worktrees and unrelated host workloads.
- Keep libraries caller-driven, and CLI/status presentation independent of
  numerical code. No library-owned QMC worker pool or integration loop.
- Scientific equivalence is required; legacy options, sector numbering and cache
  formats are not. Preserve complete Laurent-vector cancellations and covariance.
- Prefer small modules with clear responsibilities. No alternate CAS, DOT parser,
  physical graph type, or general-purpose graph canonicalizer.
- Run meaningful scientific tests and independent reviews at milestone boundaries.
  Record unimplemented features and unverified performance honestly. Never turn
  a numerical failure or inconclusive scalelessness check into a zero result.

Use `nix-shell` for a Rust >=1.96 environment with native dependency build tools.
See `docs/DEVELOPMENT.md` for local dependency identities and test commands.
