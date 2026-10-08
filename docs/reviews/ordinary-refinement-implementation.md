# Ordinary refinement implementation notes

This slice applies `integration.double_points` to ordinary democratic/adaptive
QMC, per-sector Havana MC, and global discrete Havana MC. Serial execution
retains its separate native scheduler and checkpoint mode.

## Reuse audit and scientific behavior

The native FastSecDec QMC plan already guarantees identical earlier shift
coordinates when its shift target increases. Its accumulator had no extension
operation: its package identity includes the entire plan. The new narrow
`QmcAccumulator::extend_plan` first proves identical native lattice rules and
bitwise-identical earlier shift vectors, then preserves accepted native partials
while changing only their enclosing plan identity. It accepts complete plans
only. Native tests audit generated coordinates, rather than assuming distinct
task identifiers establish distinct work.

`QmcSession::extend_production_shifts` retains shared shift identities and full
democratic cross-sector covariance. Adaptive allocations keep their established
sector ratios and independent streams without another pilot. Stored allocation
boundaries preserve canonical partial tail packages even when `package_points`
does not divide the lattice length. Checkpoint restoration validates those
boundaries and all completed packages. Published catalogue exhaustion uses this
same append path; larger lattice sizes are distinct statistical designs.

Ordinary adaptive QMC retains its existing sector namespace mixing. Appended
shifts within each sector are rigorously jump-separated, and only new shift
indices are assigned. That legacy mixing supplies different sector start states;
it is not claimed to prove a single global jump partition across all ordinary
adaptive sectors. Serial integration instead uses the stronger central frontier
required by its simultaneous same-sector worker contract. Democratic QMC's
cross-sector sharing remains intentional.

Numerica 3.0.1's public MonteCarloRng, ContinuousGrid and DiscreteGrid APIs own
sampling, grid adaptation, jump partitioning and state export/import. Source
and existing focused probes confirm that one native jump spans 2^128 draws and
the xoshiro period is 2^256−1; continuous sampling uses a bounded number of draws
per coordinate and discrete sampling adds its sector selection. No Numerica
grid operation implements FastSecDec's multi-output production-session
extension. The narrow session adapters therefore reuse the existing grids,
accepted vector batch records and the same audited `integration::streams`
allocator as serial integration. The helper moved out of `serial` so its draw
bounds and checked stream frontier have one owner.

MC and discrete MC fixed-size refinements keep their frozen continuous and
discrete proposals, append only new jumped streams, and retain accepted vector
statistics. Pilots remain excluded. Native version-2 MC checkpoints persist
the checked stream frontier and a digest of the complete typed checkpoint;
restore rejects corrupt state, repeated reserved streams and frontier overlap.
An exhausted frontier rejects an append before changing the current session.

## Previous completed production

Each ordinary refinement durably checkpoints its completed allocation before
starting a replacement. The outer checkpoint can also carry a separately
identified `PreviousProduction` with its round, full observation and its own
QMC design. Current session statistics and convergence always remain separate.
This preserves an earlier complete result through cancellation or recovery
without counting nested coarse/fine designs as independent replicas.

The CLI reuses its existing previous-allocation display cache and current live
panel. A resumed previous result seeds that cache; it remains explicitly labeled
as the previous allocation until the new allocation completes. A statistical
failure is never replaced with an older estimate. JSON final reports carry an
optional `previous_complete` object when current production is unfinished.

## Validation inventory

Native refinement tests cover actual appended coordinate sequences, shared-shift
cancellation/covariance, adaptive pilot exclusion, frozen MC proposals,
checkpoint restoration, corrupt/repeated random frontiers and counter exhaustion.
CLI integration tests cover both refinement policies for QMC, MC, adaptive MC
and discrete MC, including changed-worker-count resume.

All five native refinement tests and the ordinary CLI integration test passed
in the combined workspace run on 2026-10-08. That rerun includes the complete
checkpoint digest, previous-production persistence and frontier-exhaustion
changes. The CLI test exercises all four ordinary methods under both refinement
policies and restores their checkpoints with a changed worker count. This is
the validation result for this slice; complete-workspace acceptance is tracked
separately by the coordinator.
