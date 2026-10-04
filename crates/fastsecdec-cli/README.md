# FastSecDec CLI

The CLI reads native HEPKit DOT graphs, native model JSON and parameter cards,
and TOML run cards. Paths inside a run card are relative to that card. External
scalar products use the graph's native `P(index)` basis; `inspect` reports that
basis. The shipped run cards in `examples/runs` provide complete inputs.

Inline `[parameters]` values remain exact Symbolica expressions. Derived model
parameters and couplings use their analytic definitions at the selected point;
cached values from another point are not reused. Explicit internal values in a
native restriction card remain fixed unless overridden inline.

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

Generation snapshots update after each compiled kernel and include cumulative
stage timings. Final JSON and artifacts retain input, parametrization, domain
checks, geometry, mapping, verified symmetry, subtraction, Laurent expansion,
and compilation times. Generation total ends after preparing artifact metadata,
before the final file write. Artifact commands also report cold loading and
recompilation time. These observations are excluded from scientific content
identity and checkpoint compatibility.

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
Weighted checks examine complete coefficients after the known sampling weight
is applied. A new or sufficiently larger weighted value is checked at higher
precision; the `additional_replays` counter distinguishes a new whole-vector
evaluation from a check already satisfied by ordinary precision rescue.
`[integration.replay]` accepts `growth_factor` (default 16) and `minimum_bits`
(default 128). The weight is applied once, before conversion from rescue
precision to binary64.

Replay maxima advance only for complete work packages accepted by the session.
Checkpoints preserve that accepted state and policy across worker-count changes,
pilot/production transitions, and refinement rounds. Failed package prefixes
do not influence resumed replay decisions. Checkpoint version 3 is required;
older development checkpoints require restarting integration. Different worker
schedules can change rounding within the precision policy, so bitwise equality
across schedules is not promised.

Boundary diagnostics enumerate coordinate subsets and every lower/upper side
assignment through `--max-codimension` (default 2), up to `--max-probes` total
points (default 10,000). Reports state configured/planned/completed coverage and
whether the budget truncated it. Increasing codimension includes higher face
intersections; finite samples do not prove the absence of thresholds.

Benchmark and boundary routines are public Rust APIs under
`fastsecdec::diagnostics`. Ctrl-C preserves completed diagnostic rows and labelled
partial benchmark repetitions; incomplete repetitions do not enter the timing
median. `--status-json` streams their typed progress events. Kernel benchmark
timings include coordinate generation and checked evaluation, and exclude time
spent displaying progress. The final report retains load/generation timings
separately from the measured repetitions.

Portable artifacts store exact expressions, compilation policy, source hashes,
normalization, domain assertions and exact dependency identities. Loading an
artifact recompiles portable SymJIT O2 kernels locally without repeating graph
algebra or sector generation. `integrate` needs only the artifact. `run --resume`
also verifies the original input files, including referenced model and graph
files, before using its checkpoint.

Reference comparisons are optional and never change sampling, stopping, or
checkpoint statistics. Add a root run-card section such as:

```toml
[reference]
path = "../targets/my_reference.json"
normalization_evidence = "Both results use the same stated measure and prefactor."
kinematics_evidence = "Masses and independent scalar products match this card."
independence_evidence = "The reference uses an independent analytic calculation."
```

Evidence fields may be omitted; their status then remains unknown. They record
the caller's assertions and perform no normalization or kinematic conversion.
Reference validation and uncertainty remain those declared in the target file.
Eligible comparisons require the library's complete coverage, validation,
compatibility, independence, and uncertainty checks. Other rows remain clearly
labelled diagnostics. Missing coefficients are never assumed zero, unknown
reference errors produce no pull, and absent integration estimates report
`waiting_for_coverage`.

`run --reference FILE` and `integrate --reference FILE` override the target path.
An override selecting a different resolved file clears inherited evidence;
evidence for the previous target cannot certify its replacement. Selecting the
same resolved file retains the configured evidence. Card paths are relative to the card;
override paths are relative to the current directory. The selected file is
parsed and validated before graph generation or portable-kernel compilation.
Saved artifacts retain the resolved comparison settings outside scientific
identity, so `integrate` can use them without reading the original card. Final
JSON includes the typed comparison, evidence, and the target's path and digest;
plain output uses the library's reference display.

The native JSON transport is
`{"format":"fastsecdec-reference","version":1,"reference":...}`, where
`reference` is the library's `ReferenceResult`. Native callers can pass that
type directly or use `fastsecdec::reference::encode_reference` to write a file.
Historical target JSON with `schema_version: 1` is also accepted automatically.
Historical imports remain unverified; null uncertainties remain unknown and
numeric zero uncertainties remain reported standard errors. Mixed, unversioned,
and unsupported document formats are rejected.

New artifacts fingerprint the parsed run card with only its root `[reference]`
section excluded. Changing comparison settings, comments, or TOML formatting
does not invalidate their checkpoints; numerical fields and every other parsed
root field do. Model, graph, and polynomial files retain their raw-byte checks.
Older artifacts retain their original raw-byte run-card verification.
