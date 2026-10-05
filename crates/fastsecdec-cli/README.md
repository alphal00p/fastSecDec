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

`--plain` disables the live terminal dashboard and report/run-time error colors. Setting
`NO_COLOR` keeps the live dashboard but uses terminal-default colors throughout
the dashboard, boundary report and errors. Redirected reports omit color.
Short or narrow terminals show a compact view of the same public status.
Clap owns argument-validation and help formatting; use `NO_COLOR` to request
monochrome output before argument parsing as well.
`--json` writes one final JSON
document to stdout; `--status-json` streams public status snapshots to stderr.
Integration JSON snapshots default to a minimum 100 ms interval. Set
`--status-interval-ms 0` for every worker batch, or choose a longer interval to
reduce observation work. This option affects neither numerical settings nor
checkpoint identity; terminal/plain intervals remain 40 ms/1 s. Initial,
stage/round-boundary and final states are always emitted, including cancelled or
failed outcomes. Generation and standalone diagnostic progress are unchanged.

Worker evaluation/submission failures, replay acceptance and cancellation remain
checked after every batch. Numerical-statistics range failures discovered by
snapshot reduction are checked at the selected observation cadence and
unconditionally at stage/final reduction; this can delay their detection by the
chosen interval. They still retain accepted data, save the supported checkpoint
and exit unsuccessfully. No unavailable estimate is replaced by a zero.
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
QMC uses the historical `kuo33002` catalogue by default. Select another native
published catalogue explicitly, without regenerating the symbolic artifact:

```sh
fastsecdec integrate output/bubble.fsd.json --lattice hkkn-alpha3
```

The equivalent run-card setting is `[integration] lattice = "hkkn-alpha3"`.
The available catalogues have these native capability bounds:

| CLI/TOML value | Stochastic sector dimensions | Points (powers of two) |
| --- | ---: | ---: |
| `kuo33002` (default) | 1–9125 | 1024–2^20 |
| `kuo38005` | 1–5000 | 1024–2^20 |
| `kuo39101` | 1–3600 | 1024–2^20 |
| `hkkn-alpha3` | 1–10 | 2–2^20 |

Unsupported requests fail explicitly; the library does not substitute another
catalogue. These are capability bounds, not uncertainty guarantees. With
multiple refinement rounds, the driver doubles points up to the selected cap and
then doubles independent shifts. Adaptive QMC also increases its frozen
production budget after reaching the cap. The covariance matrix covers every
Laurent output, including real/imaginary components for complex scalar weights.
Final JSON includes the native `qmc_design`, with the effective settings and
actual points/shifts allocated to each sector; plain output gives a compact
summary. Adaptive allocations can differ from the initial requested counts.

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

The selected catalogue and refinement round must agree with the native session
inside a checkpoint. Missing historical `lattice` fields retain their original
`kuo33002` meaning. A CLI lattice override leaves the symbolic kernel identity
unchanged but changes sampling/checkpoint identity. Editing integration settings
in a source run card still follows the artifact's existing source-fingerprint
checks; it is not equivalent to overriding a saved artifact at integration time.

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
points (default 10,000). `--exponents 3,6,9,12,15` is the default distance list.
Each physical Laurent component is compared at adjacent actual endpoint distances;
`--growth-tolerance` defaults to 0.5 per approached axis, plus a numerical slack
of 1e-6. A large constant component cannot conceal another component's growth.
Zeros, failed samples and incomplete coverage are represented explicitly.

`--retry-scales 0.01,0.0001` retries flagged or inconclusive sectors; there are no
retries by default. Each scale refers to the original distances, and all attempts
share the point budget. The plain report shows an attempt table; JSON retains
every raw vector and growth pair in `attempts`, including earlier flags. Growth
flags are diagnostic and do not cause a failing exit. Numerical evaluation
failures cause a nonzero exit while still producing one complete JSON report.
Ctrl-C returns retained completed rows and a `Cancelled` execution state.
Finite samples or acceptable growth do not prove integrability or the absence of
thresholds.

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

Save numerical results separately from compiled artifacts and resumable
checkpoints:

```sh
fastsecdec run examples/runs/bubble.toml --save-result output/bubble.result.json
fastsecdec show-result output/bubble.result.json
fastsecdec show-result output/bubble.result.json --sort error --order 0 --component real
fastsecdec export-reference output/bubble.result.json --source estimate --output output/bubble.reference.json
```

`integrate` also accepts `--save-result`. The native versioned result retains
the complete coefficient layout/covariance, accepted sector contributions,
exact offsets, execution status, diagnostics and effective QMC design. Its
manifest binds the inner kernel identity; the CLI artifact identity remains
separate provenance and continues to identify checkpoints. Saving changes no
sampling, stopping or reference evidence. Result output paths must differ from
input files, artifacts and checkpoints.

`show-result` reads numerical data only. It works after the original graph,
artifact, checkpoint and comparison-target files have been removed. Plain output
labels the result's scope, exact-offset policy and marginal uncertainties;
the authoritative total retains shared-sector covariance. Sort by `id` (default),
`magnitude`, or `error`; the latter two require a Laurent `--order` and accept
`--component real|imag`. Unavailable estimates sort last, with sector IDs breaking
ties. `--json` emits a view containing the original typed result plus the derived
comparison and sector ordering; this view is distinct from the persistence
envelope written by `--save-result`.

Reference export requires `--source estimate` or `--source stored`. The former
requires complete full-integral production and retains the estimate's validation
status; CLI computations remain `Unverified`. Reported standard errors,
including zero, remain standard errors. `stored` returns the original retained
reference, including unknown errors and original evidence, without falling back
to the computed estimate. Its historical comparison context is preserved in the
result; comparing an exported target to a new computation requires new evidence.

Cancelled, partial, pilot-only and failed records remain viewable. They cannot
be exported as complete estimate references. Numerical failure saves accepted
coverage, displays one final report and exits unsuccessfully; failed package
prefixes are excluded. A statistical range failure can leave some estimates
unavailable. A representable authoritative total survives unavailable correlated
marginals. Imported selected-sector records retain their declared restricted
scope and exact-offset policy and cannot be promoted to full-integral references.
Malformed or unsupported result documents are rejected by the native reader;
unversioned historical CLI output is not inferred to be a saved result.

Compiled-sector subsets use original kernel IDs and an explicit policy for the
folded exact offset:

```sh
fastsecdec integrate integral.json --sectors 2,7 --exact-contributions include --save-result subset.json
fastsecdec integrate integral.json --sectors none --exact-contributions exclude
```

`include` adds the complete exact coefficient vector; `exclude` adds none of it.
An empty list is spelled `none`. Even a selection naming every stochastic sector
remains a qualified subset, so its final `converged` field is false;
`scoped_target_reached` reports whether its own requested tolerance was reached.
Live status, final JSON and saved results retain the native scope. Native
comparison and estimate-reference export do not treat a subset as a full-integral
reference. The original stored reference can still be exported explicitly.

The same native scope can be placed in a run card:

```toml
[integration.scope.SelectedSectors]
sector_ids = [2, 7]
exact_policy = "IncludeAll"
```

IDs refer to compiled representatives, whose existing symmetry multiplicities
remain included; they are not geometric chart IDs. Unknown or duplicate IDs are
rejected. ID order is canonicalized. Resume permits worker-count changes but
rejects scope or exact-policy changes, including selected-all versus full scope.
`--full-integral` explicitly clears a scoped card/artifact default and is mutually
exclusive with the subset flags. Omitting scope continues to mean the full integral and preserves historical
full-run checkpoint settings. Only selected worker evaluators and integration
work are created; loading the artifact still compiles its complete kernel set.

`inspect integral.json` shows retained native domain certificates, chart to
representative/kernel associations, permutations, exact exponent matrices and
support valuations. `--expressions` adds canonical coordinate images and the
positive real measure in plain output. JSON includes the existing complete native
portable metadata record. Projective images are gauge-fixed, and valuation rows
use the native deduplicated support order rather than invented U/F names. A chart
with no kernel can be exact, cancelled or truncated; individual chart exact
coefficients are not retained. Legacy artifacts explicitly report unavailable
metadata.
