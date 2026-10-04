# FastSecDec CLI

The CLI reads native HEPKit DOT graphs, native model JSON and parameter cards,
and TOML run cards. Paths inside a run card are relative to that card. External
scalar products use the graph's native `P(index)` basis; `inspect` reports that
basis. The shipped run cards in `examples/runs` provide complete inputs.

```sh
fastsecdec inspect examples/runs/bubble.toml
fastsecdec generate examples/runs/bubble.toml --output output/bubble.fsd.json
fastsecdec integrate output/bubble.fsd.json --workers 2
fastsecdec run examples/runs/analytic_endpoint.toml --workers 2
fastsecdec benchmark output/bubble.fsd.json
fastsecdec check-boundaries output/bubble.fsd.json
```

`--plain` disables the live terminal dashboard. `--json` writes one final JSON
document to stdout; `--status-json` streams public status snapshots to stderr.
The documented Symbolica banner display setting is applied before library
initialization so machine-readable stdout remains clean. Licensing is unchanged.

Integration methods are `qmc`, `adaptive_qmc`, `mc`, and `adaptive_mc`.
Bundled QMC rules support powers of two from 1024 through 2^20 points.
With multiple refinement rounds, the driver doubles points up to that cap and
then doubles independent shifts. Adaptive QMC also increases its frozen
production budget after reaching the cap. The covariance matrix covers every
Laurent output, including real/imaginary components for complex scalar weights.

The caller owns the integration loop, worker pool, lattice point generation,
stopping decisions, and checkpoint writes. The CLI is one such caller of the
Rust library. Pilot statistics are excluded from production estimates.

`--checkpoint PATH` selects the checkpoint and `--resume` restores it. Input and
integration settings must match; the worker count may change. A completed
checkpoint returns the existing result without repeating work. Ctrl-C, `q`, or
Escape stops after the current worker batch and saves completed production
work. QMC pilots can also be resumed. Havana MC pilots must be restarted because
the upstream library does not expose its mutable pilot training state; the
report marks this as `pilot_restart_required`.

Precision diagnostics count cumulative evaluation attempts, conditioning
checks, and rescues across pilots and refinement rounds. Checkpoints preserve
these counters. They differ from the current stage's completed-point count.
Boundary diagnostics include individual faces and simultaneous lower, upper,
and alternating corner approaches; these probes diagnose numerical evaluation
and do not prove the absence of thresholds.

Portable artifacts store exact expressions, compilation policy, source hashes,
normalization, domain assertions and exact dependency identities. Loading an
artifact recompiles portable SymJIT O2 kernels locally without repeating graph
algebra or sector generation. `integrate` needs only the artifact. `run --resume`
also verifies the original input files, including referenced model and graph
files, before using its checkpoint.
