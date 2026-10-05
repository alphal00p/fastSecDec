# Shared endpoint admission — independent review

Accepted source and focused scientific gate, 2026-10-05. This slice extracts
the existing exact endpoint rule without selecting the named-coefficient route
or changing the physical Taylor/IBP schedule.

`subtraction/endpoints.rs` retains the previous native rational extraction,
literal regulator substitution, native derivative and exact affine residual
check. Its checked count is zero above the integrability threshold −1 and
otherwise the same native `floor(-constant)`, i64 conversion and caller limit.
Affine validation still precedes the degree limit. Zero regulator slope remains
admissible at this stage: the physical subtraction owner alone decides whether
an actual nonzero boundary coefficient requires `UnregulatedEndpoint`.
Native exact pruning is unchanged.

The physical loops replace their local count calculation with this admission
record and otherwise retain their operation sequence. The only other production
change replaces the unchecked sum of cancellation degrees in generation with a
checked fold. Valid rows produce the same maximum; overflow now returns the
existing typed `GenerationError::ResourceLimit`. No CAS, evaluator, geometry,
status, threading or artifact owner changes.

Independent inspection of the executed logs confirms **50 passed, zero failed,
two explicit probes ignored**: 15 subtraction controls, five physical strategy
tests, 16 generation tests, five independent analytic generation tests, eight
generation-context tests and one native scalelessness/input-order test. The five
new controls cover rational/factored thresholds, invalid-affine/limit precedence,
native conversion and degree overflow, an analytic fractional boundary plus
remainder density, and exact unregulated pruning for both strategies. Existing
Gamma/full-vector, domain/face, precision rescue and caller-context controls
remain in the gate.

Commands and exits are retained in
`output/endpoint-admission-focused-results.json`, with six corresponding target
logs. Package formatting and all-target Clippy pass; the latter completes in
6.73 seconds. `output/endpoint-admission-source.sha256` matches all four final
source files. The initial build rejected unary negation of a borrowed native
Rational; the retained second build uses an owned clone with identical exact
arithmetic and succeeds. This is a focused gate, not a newly executed full
workspace or whole-graph trial.

No independent finding remains for this extraction. Later named-route modules
must reuse this owner, preserve physical unregulated fallback, and separately
validate public conditioning, limits, status, native symbol admission and the
complete scientific path before adoption.
