# Native direct-translation external-call reuse

## Reuse evidence and reproduced defect

The public dynamic bubble pilot exposed repeated execution of an identical
radius callback. The independent probe uses the actual generated symbolic
Laurent vector, its native aliases and Symbolica's public evaluator builder and
instruction export. With direct translation and zero Horner iterations it
retains four calls with identical helper tags and argument slots. The native
tree translation produces one call. No hand-written evaluator or expression
expansion was used.

Public API/source inspection confirms that zero Horner iterations deliberately
skips later optimization in `AtomView::to_evaluator`. However, direct
linearization already maintains a scope-local native expression cache. Its
external-function branch returns immediately after emitting an instruction,
bypassing the cache insertion used by other expressions. Aliases inherit that
cache, ordinary named functions have separate binding scopes, and conditional
branches clone it without exporting branch-local entries.

The latest upstream `community` revision was fetched and checked as
`f4e787074d45f1dddd0b9646a5ebc85690deb2d1`; it has the same omission. The isolated
owner change adds the missing insertion before the external return. It does
not introduce an optimizer, change Horner settings, hoist lazy work, or add
sampling-time memoization. Native `EvaluatorComposer` can also re-optimize a
program, but rebuilding an instruction representation to repair this missed
existing cache entry is unnecessary for the owner fix.

## Validation boundary

Five focused owner controls reproduce excessive callback counts before the
patch: complete real/complex output vectors, aliases, distinct tags/arguments,
lazy branch-local reuse and separate inline-function bindings. Their numerical
results remain correct before the count assertions fail. The lazy control also
requires that an untaken callback never runs, and that a branch-local value
cannot supply a later unconditional output. A sixth control verifies parent
call reuse by a selected conditional branch. All six patched controls pass,
as do 15 existing evaluator integration tests, with one pre-existing stress
test ignored. The linked tests use the isolated owner source and the pinned
native dependency graph. Root and generation-agent source reviews pass.

The fix is published as [Symbolica PR 60](https://github.com/symbolica-dev/symbolica/pull/60),
commit `85a993fd9b8d9e25099261dff476c64afac15f7c`, authored and published by
ValentinHirschi. GitHub denied formal reviewer assignment with
`RequestReviewsByLogin`; an explicit `@benruijl` review invitation records that
limitation. The PR is attached to the task. The separate combined consumer child
`516beb37d31af8e3d6ee321a7070f407a0b1b42d` retains all prerequisites from
`1e1cb169` and adds only this accepted fix. Its six new cache controls, 15
existing evaluator controls (one existing ignored stress test) and eight
direct-vector/cache controls pass. It is published on
`ValentinHirschi/symbolica:codex/contour-external-cache-consumer`.
The three maintained FastSecDec consumer manifests now select this revision.
Locked metadata verifies one Symbolica/Numerica owner in every consumer; full
lockfile comparison confirms that only those source revisions changed. Native
library tests pass (366 passed, 18 explicitly ignored), as do the three public
dynamic scientific controls. The standalone portable host suite passes all 76
tests, and the same three public controls pass in actual Emscripten/Wasm
execution. The broad native core/QMC/sectors gate passes 692 tests with 24
explicit ignores (including the library and public dynamic tests above); the
CLI suite separately passes 168 tests with eight explicit ignores. Strict
workspace all-target Clippy also passes. The four-mode
variance baseline was measured with the preceding `1e1cb169` source and is not
relabeled as a measurement of this optimization.

The separate runtime observation correction coalesces only identical exact
candidate centers at identical precision within one admitted request bundle.
Each callback still returns its own numerical value and tracked uncertainty;
independent certification checks the shared center against every retained
source context. Conflicting candidates, unknown requests and missing work
remain failures. This restores correctness for valid repeated calls, while the
owner change addresses their unnecessary solve cost.
