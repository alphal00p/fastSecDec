# Persistent generation step list

The CLI dashboard now keeps the applicable generation phases visible alongside
the active phase's aggregate progress, memory use and full-width worker table.
The active phase is highlighted; completed, pending and unneeded phases have
distinct markers as well as colors. The progress bar and ETA still describe
the active phase rather than an invented overall work estimate.

Native `GenerationOptions`, `GenerationMode`, `CoefficientExpansionMethod` and
`GenerationStage` are reused. Source inspection found no native declarative
phase-plan API. A small CLI-owned presentation plan groups native operations
that repeat inside coefficient construction; no numerical execution, native
status schema or worker scheduling changes are needed.

The symbolic plan includes symmetry reduction and coefficient expansion. The
numerical-dual plan instead includes subtraction-formula preparation and sector
assembly. Taylor/IBP and full-expression/coefficient-series work are internal
choices within coefficient construction, not additional globally sequential
phases. Nested native progress events do not restart earlier global phases.
Unobserved phases remain distinguishable from completed work when there is no
work to perform.

A narrow CLI input observer reuses the existing run-card parse and integral
preparation timing boundaries. The plan is configured before expensive native
preparation without rereading the card or adding a graph/algebra helper. Existing
input callers retain the original wrapper. Artifact delivery starts explicitly
after compilation returns; only successful artifact saving completes the plan.
Rendering consumes cached status and the caller's plan, retaining update cadence
and terminal cancellation behavior. Plain and JSON status retain their schemas.

Independent reviews traced both native pipelines and checked the presentation
boundaries, native API reuse and unchanged scientific semantics. Focused
controls pass for actual tiny native CLI generation through artifact saving in
symbolic coefficient-series, symbolic full-expression and numerical-dual modes.
Ratatui buffer checks retain every applicable step at 40×20, 64×24, 80×24,
120×32 and 160×42 terminal sizes. They also cover repeated inner phases,
unneeded work, zero required formulas, incomplete delivery, and the parsed-card
callback occurring before source-file loading. No three-loop generation or
expensive ggHH benchmark is required for this presentation change.

All four focused controls pass; the complete CLI binary suite passes 60 tests
with one existing ignored test. Strict CLI Clippy, formatting and diff checks pass. Independent
review accepted the final source, phase-state controls and color/layout checks.
