# Threshold-decomposition repository and dependency foundation

Date: 2026-10-10. This records preparation for
[the approved implementation plan](../../NO_DEFORMATION_PLAN.md), not a completed
threshold integrator or general algebraic resolver.

## Published history and handoff

The contour branch was 49 commits ahead of main with no divergent commits.
Regression repair `cf10c6942b92496d29aba3a0f47fde7ae5959b10` was added, then main
was fast-forwarded and published at that commit. The repair changes only a
test: native primary JIT caches serialize a function-name set in a potentially
different order, so cache byte equality is not mathematical equivalence.
The two originally failing artifacts passed full native integrity validation,
restored their saved primary evaluators, and agreed exactly in all 24 tested
scalar outputs. The corrected test checks each identity at its own layer,
record identities, full restored Laurent vectors and native cache restoration;
it retains byte equality for exact/eager artifacts.

The preexisting exporter commit
`99f96a5623eb5059f83c340833fafc2858498966` was rebased onto that main as
`5897feee01962852edaebdc68c6f36d469c55761`. Conflict resolution retained both
documentation additions and both CLI dependencies (`libc` and `numerica`).
Native graph admission still resolves contour Jacobian overrides and retains
explicit factor semantics. No exporter, graph fixture or interoperability
change was dropped. Publication used an explicit lease against `99f96a5`.
The other agent's checkout was not modified.

`NO_DEFORMATION_PLAN.md` was copied verbatim from the approved text and linked
from `FIRST_PHASE_PLAN.md` in `f79b81c`. The implementation goal was activated
before threshold implementation changes. Later milestones stay on
`no_deformation`; mathematical acceptance remains that of the full plan.
Commits and publication use `ValentinHirschi <valentin.hirschi@gmail.com>`.

## Regression evidence

The consolidation gate covered all 89 workspace test executable targets:
946 passed and 34 existing ignores, plus passing doctests. The initial run
stopped at the test assertion above after 848 passing tests; the corrected
target and remaining targets supplied the other 98 passes. Three extra focused
worker-count runs passed. This is composite coverage, not a claim that the
initial command exited successfully. Strict workspace/all-target Clippy and
format checks passed after the repair.

The standalone portable consumer passed all 88 tests. Python binding
`python_stubgen` checked successfully with an explicit Nix Python interpreter;
the initial binding attempt failed because Python was absent from PATH.
These are host portable and binding-build checks, not fresh browser execution.

The rebase gate passed 134 tests: native graph/Symanzik controls, A446 polynomial
identities, contour admission and source scope, serial generation, restored
worker equivalence, process supervision and the complete CLI unit target.
Eight ignores comprise two external symGCAD process tests and six subprocess
fixtures. It also passed strict workspace/all-target Clippy and formatting.
The two external process comparisons were not rerun in this rebase gate;
their earlier evidence remains in the
[exporter review](no-deformation-interface.md). Direct library solve/verify
probes belong to the next dependency/adapter milestone.

Raw logs and command/source hashes are retained locally under
`target/no-deformation-gates/{contour-consolidation,exporter-rebase}` and are not
tracked. No production Rust implementation or dependency changed in the
consolidation repair. The existing release executable was not rebuilt or
overwritten during these checks.

## New algorithm boundaries

The [resolver specification](../NO_DEFORMATION_RESOLVER.md) and
[proof addendum](../NO_DEFORMATION_RESOLVER_PROOFS.md) record the marked-ideal,
ramification, atlas, parameter and analytic-family requirements. They are
research and implementation contracts, not evidence of a working general
resolver. In particular, finite ownership of a supplied blow-up atlas does not
establish a closed-face cube normal form, and generic CAD coverage does not
establish auxiliary-regulator cancellation.

Keep original signed physical factors and complete regulator-dependent powers
separate from canonical projection/root factors. A verified open-cell solve
cannot yet be published as a complete integral. General resolution, all-face
certificates, auxiliary continuation, execution/interfaces and physical
benchmark acceptance remain pending.
