# Caller-owned geometry reuse during generation

Author design/implementation record, 2026-10-05. The coordinator approved this
additive adoption of the independently reviewed sector `GeometryCache`. The
generation algorithms, ordinary `generate` entry and existing
`GenerationOptions`/`GenerationProgress` types retain their contracts. CLI
rendering and cache steering are outside this slice.

`GenerationContext::new(geometry_capacity)` owns only a native geometry cache.
Capacity is an entry count, not a byte limit; zero disables retention. Borrowed
cache accessors permit inspection, explicit clearing, and direct native geometry
reuse. The context stores no physical input, Atom, domain certificate, evaluator,
subtraction or Laurent result, and owns no threads.

The context's additive `generate` method accepts `GenerationEvent` callbacks:
ordinary progress is wrapped in `Progress`, while `GeometryReuse` carries the
serializable/displayable `GeometryReuseStatus`. Its `sectors` count describes
native geometry maps **before** symmetry merging and exact coefficient
extraction. It must not be interpreted as the resulting number of numerical
kernels. The old free entry uses the same internal generation path without
consulting a cache or changing its callback type. Existing exhaustive callback
matches therefore remain source-compatible.

Every request still performs the full original domain assessment and native
support extraction before consulting the cache. Coefficient changes, exponents,
numerators, prefactors and threshold assertions are not inherited from the
previous integral. All maps then pass through the existing mapping/residual
admission, symmetry, subtraction and Laurent owners. Returned chart/domain
metadata belongs to the current integral. Shared cache maps are borrowed; only
the map copies required by the result's owned metadata are cloned. Compact
native coefficient bodies are not materialized to implement or observe reuse.

Native geometry callbacks remain cancellable on both paths. Rejection at native
`Complete` prevents insertion of a new entry. The subsequent `GeometryReuse`
event is also cancellable before mapping; that cancellation, or a later
generation failure, can leave already complete valid geometry in the cache.
No partial generated integral is returned. Empty integrals bypass geometry and
emit no reuse event.

The five integration tests cover compact exact cold/warm/default outputs,
owned chart metadata, native compiled vectors with independent analytic
coefficients, changed polynomial/prefactor/numerator weights, renewed domain
and explicit-assertion checks, domain-separated reuse, early/final/hit/post-cache
cancellation, and empty inputs. They do not call the materialized coefficient
accessor or expand a large expression for equality.

Source/test compilation passed with
`cargo test --locked -p fastsecdec --test generation_context --no-run`; a
concurrent requested CLI `cargo check --locked -p fastsecdec-cli --tests` also
passed. Logs are `output/generation-context-build.log` and
`output/generation-context-cli-check.log`.

After the frozen full-graph trial and capture relinquished the runtime, the
focused target passed **five tests, zero failures** in 0.02 seconds. Evidence:
`output/generation-context-corrected-tests.log`. The initial attempt is retained
in `output/generation-context-tests.log`: three tests passed and two new author
assertions incorrectly expected constant poles in the separate exact offset.
The existing contract extracts only wholly constant vectors; these poles stay
in their mixed coordinate-dependent sector vector. The tests were corrected
to assert the same analytic poles there, without changing any production code,
finite/higher-order target, tolerance or cold/warm equality check.

The subsequent serial workspace gate passed **305 tests, zero failures, 18
ignored diagnostics** across 57 suite summaries, including the corrected five
tests and all existing generation/numerical/CLI regressions. Evidence:
`output/geometry-context-workspace-tests.log`. Workspace formatting and
all-target Clippy with `-D warnings` passed; logs are
`output/geometry-context-fmt.log` and `output/geometry-context-clippy.log`.
The pre-existing native dependency warning at Symbolica `atom.rs:774` remains
unchanged. These gates establish small scientific equivalence and API behavior,
not a geometry-reuse speedup. The independent review is recorded separately in
`generation-context-independent.md`.
