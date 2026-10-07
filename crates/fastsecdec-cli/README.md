# FastSecDec CLI

The CLI reads native HEPKit DOT graphs, native model JSON and parameter cards,
and TOML run cards. Paths inside a run card are relative to that card. External
scalar products use the graph's native `P(index)` basis; `inspect` reports that
basis. The ggHH example includes a runtime point file. Graph cards under
`examples/runs` declare symbolic dot products and retain their reference points
in `[integration.parameters]`.

Graph inputs compile contributing independent model inputs as runtime evaluator
parameters by default. Native HEPKit resolves dependent parameters and couplings
into expressions in those inputs. The model card supplies human-readable defaults
and zero-width restrictions; its cached numerical values do not freeze dependent
expressions. Integration requires the declared values through `[integration.parameters]`,
`--parameters`, or repeated `--parameter NAME=VALUE`. Inputs use `model::NAME`, with `_re` and
`_im` components for complex leaves. Kinematic `symbol` declarations likewise
require integration-time values; `value` declarations are fixed expressions.

Inline `[parameters]` values request exact fixed specializations. Setting
`model_parameters = "fixed"` in `[input]` instead specializes the complete model
with its card and inline overrides. All internal widths must be zero. Generic
runtime propagator masses must remain finite, real and nonzero; a zero-mass
specialization requires regeneration because endpoint structure can change.
These checks do not certify thresholds.

Numeric fixed inputs use native rationals preserving their supplied binary
floating-point value. For exact decimal or rational expressions, use strings
such as `mass = "1725/10"`. Runtime point expressions are evaluated once to
finite binary64 values and passed unchanged to evaluator and precision-rescue
inputs; higher evaluator precision does not add physical input precision.

Numerator-only external vectors, such as numerical gluon helicities, can be
declared with `kinematics.auxiliary_momenta`. Supply their scalar products using
the same native atoms as the graph numerator. Integer product labels retain the
`P(index)` shorthand; strings are native Symbolica momentum expressions. When
writing cards, use each vector atom's `to_canonical_string()` and normal TOML
string escaping for both auxiliary names and product labels. Canonical spelling
preserves Spenso's rank-one tensor metadata on cold import; a previously
undeclared plain symbol does not carry that metadata.

These vectors extend the native numerator scalar-product space, preserving the
graph's propagators and routing. Their numerical pair products may be complex
or Gram-degenerate; the Gaussian numerator algorithm does not invert that Gram
matrix. Internal tensor algebra still uses the requested dimension. This input
does not replace the graph's projector or generate a polarization sum.

Graph input retains its original propagator family by default. To request the
existing native single-term family preparation before parameterization, use the
case-sensitive native policy spelling:

```toml
[generation.family_preparation.SingleUnitTerm]
max_states = 32
```

`max_states` bounds native partial-fraction states, not elapsed time. Only an
admitted single unit-coefficient projection is used; other supported outcomes
retain the original family and report the native fallback reason. Original
graph, model and parameter-card hashes remain in provenance. The optional
`family_preparation` report records original powers, active original indices
and active powers, including fallback. `inspect` on an opted-in card also shows
`active_parameters`; its existing `parameters` field still counts original
propagators. Generated coordinate maps describe the prepared parameter space,
not a reconstruction of discarded Schwinger coordinates. This option is
independent of coefficient expansion and is rejected for direct parametric
input. Omitting it, or setting `family_preparation = "Original"` in
`[generation]`, preserves the original route and omits the report.

```sh
fastsecdec inspect examples/gghh_double_box/run.toml
fastsecdec generate examples/gghh_double_box/run.toml --output output/gghh_double_box.fsd
fastsecdec integrate output/gghh_double_box.fsd --full-integral \
  --parameters examples/gghh_double_box/point.toml --workers 8
```

Integration uses evaluator batches of 256 points by default. Set
`--evaluation-batch-size N` (alias `--batch-size`), or `evaluation_batch_size = N`
in runtime integration settings, to change this operational chunk size. It is
independent of statistical batches/shifts and may change when resuming a
checkpoint. SymJIT evaluates eligible f64 rows through its native SIMD matrix
interface; eager and higher-precision evaluation retain their native owners.

`--plain` disables the live terminal dashboard and report/run-time error colors. Setting
`NO_COLOR` keeps the live dashboard but uses terminal-default colors throughout
the dashboard, boundary report and errors. Redirected reports omit color.
Short or narrow terminals show a compact view of the same public status.
Clap owns argument-validation and help formatting; use `NO_COLOR` to request
monochrome output before argument parsing as well.
`--json` writes one final JSON
document to stdout; `--status-json` streams public status snapshots to stderr.
Dashboard, plain and JSON observations default to a minimum 1000 ms interval. Set
`--status-interval-ms 0` for every update, or choose a longer interval to
reduce observation work. This option affects neither numerical settings nor
checkpoint identity. Cancellation polling remains independent, approximately every 50 ms. Initial,
stage/round-boundary and final states are always emitted, including cancelled or
failed integration outcomes. Generation progress counts, coefficient attempts
and representative completions are coalesced at the same configured interval.
Generation failures keep their final error report and do not emit completion.
Standalone diagnostic progress is unchanged.

Worker evaluation/submission failures and replay acceptance remain checked at
batch admission. Discrete MC also polls terminal input during active batches
and checks cancellation before each point. Numerical-statistics range failures
discovered by snapshot reduction are checked at the selected observation cadence and
unconditionally at stage/final reduction; this can delay their detection by the
chosen interval. They still retain accepted data, save the supported checkpoint
and exit unsuccessfully. No unavailable estimate is replaced by a zero.
The documented Symbolica banner display setting is applied before library
initialization so machine-readable stdout remains clean. Licensing is unchanged.

Generation snapshots update after each compiled kernel and include cumulative
stage timings. Final JSON and artifacts retain input, parametrization, domain
metadata, geometry, mapping, verified symmetry, subtraction, Laurent expansion,
and compilation times. The opt-in named route uses one separate
`coefficient_expansion_seconds` duration, including exact physical fallback;
its work is not counted again under subtraction or Laurent expansion.
Numerical-dual runs separately record `formula_preparation_seconds`: the wall
interval for building distinct subtraction formulas before sector instantiation.
Parallel worker durations are not summed into this phase. Historical artifacts
omit this observation instead of claiming that it took zero time.
Generation total ends after preparing artifact metadata,
before the final file write. Artifact commands also report cold loading and
recompilation time. These observations are excluded from scientific content
identity and checkpoint compatibility.

New artifacts also save the generation worker count and requested coefficient
expansion method in their JSON metadata. Inspection reconstructs the generation
summary from those saved observations, timings and evaluator layout; missing
observations in older artifacts are labeled as not recorded. The displayed
artifact path is resolved from the current basename, so moving the artifact
does not preserve or expose its old working-directory path.

The final human generation report presents aligned run facts, Laurent
orders/components and stage timings. Durations use readable units and bounded
precision; paths and identifiers wrap to the available terminal width. Colors
follow the shared stdout terminal policy, including `--plain` and `NO_COLOR`.
The JSON report retains its structured metadata and includes `components`
aligned with `orders`. The `orders` array follows
the scalar output layout: `[-1,-1,0,0]` for a complex result means real and
imaginary components at epsilon^-1, then real and imaginary components at
epsilon^0. The human table groups those into two explicitly labeled orders.

For native graph inputs, choose the numerator contraction policy in the run card:

```toml
[generation]
contraction_mode = "dots"
```

The choices map directly to Idenso: `minimal` (the unchanged default) preserves
independent sum alternatives; `dots` completes structural contractions and
produces canonical scalar products; `full` completes contractions, followed by
FastSecDec's scalar-product notation normalization; `none` only permits the
structural prerequisites of the enabled gamma, color and epsilon identities.
The latter identities remain enabled in every mode. `dots` and `full` may
distribute sums to finish contractions that `minimal` leaves indexed. All modes
must still produce a scalar polynomial in native loop scalar products.
Nondefault modes are rejected for direct parametric inputs, which have no graph
numerator to contract. The selected graph mode is saved in generation metadata.

Select the generation lane and endpoint subtraction independently:

```toml
[generation]
mode = "symbolic"          # default; alternative: "numerical_dual"
subtraction = "taylor"      # default; alternative: "integrate_by_parts"
```

These names select the native generation options. `symbolic` preserves the
existing symbolic generation route. `numerical_dual` selects deferred native
maps and numerical jets, retaining source charts without symbolic density
symmetry matching. Ordinary nonnegative regulated charts use the deferred route;
charts needing exact unregulated endpoint admission or signed monomial maps use
an explicit local symbolic fallback. Both subtraction strategies keep analytic
endpoint terms and the complete Laurent vector.
Generation mode is independent of the evaluator backend and of numerical
integration/periodization settings. The selected mode and subtraction strategy
are saved in human artifact metadata and shown by `generate` and `inspect`;
older artifacts report that these settings were not recorded. Actual execution
modes are also saved by stable source-chart ID; `inspect --sector` joins those
IDs to the loaded kernel, retaining the distinction after exact-sector folding.
For deferred charts, regular-body storage describes the unmapped source Atom;
explicit symbolic fallback reports mapped-body storage.

After discovering endpoint requirements, numerical-dual generation precomputes
each distinct subtraction formula on the caller's worker pool. Its own dashboard
phase reports completed unique formulas, eligible sectors, and shared uses
(eligible sectors minus distinct formulas). Sector instantiation then applies
the prepared formulas to each sector's retained source expressions. Native
source and dual evaluators are reused only when all their exact inputs and
settings match; derivative shapes remain specific to each request. This is an
in-memory generation cache, with no additional run-card option or cache file.
Final `generate` output, saved JSON and `inspect` show the same formula counts
and separate preparation time; these observations do not affect kernel identity.

The coefficient-expansion method is selected in the run card:

```toml
[generation.coefficient_expansion]
method = "coefficient_series"
max_series_attempts = 12
max_relative_width = 128
max_unique_requests = 10000
```

In `symbolic` mode, `coefficient_series` expands and shares the regular coefficient functions,
then composes the complete Laurent vector with the endpoint terms.
`full_expression`, the default, subtracts endpoints in the complete expression
before expanding it in epsilon. Both use native Symbolica operations. The older
input names `native_named` and `physical` remain accepted aliases respectively;
new configuration examples and status output use the descriptive names.
The Python `coefficient_expansion` argument accepts these same names.
In `numerical_dual` mode, both requested methods are accepted, but the small
formal endpoint recipes use the native `coefficient_series` composer. Its
series/request limits apply to those recipes; the saved requested method does
not claim that the symbolic full-expression route executed.

Native evaluator optimization is configured separately:

```toml
[generation.evaluator]
horner_iterations = 10
cpe_rounds = 1000
cores = 1
max_horner_scheme_variables = 500
max_common_pair_cache_entries = 1000000
max_common_pair_distance = 1000
verbose = false
direct_translation = true
```

These are the defaults. `horner_iterations` controls native Horner-scheme
optimization. `cpe_rounds` caps native common-pair elimination rounds; `0`
disables those rounds and `"unlimited"` uses the native unlimited setting.
`max_cpe_rounds` is accepted as an alias. With the direct translator,
`horner_iterations = 0` selects the native immediate linearization path, which
also bypasses its subsequent common-subexpression/common-pair optimization.
`direct_translation = false` requests the native expression-tree route; native
non-inlined function boundaries can still require direct translation.

The variable and cache limits are passed directly to Symbolica. The pinned
native optimizer stores `max_common_pair_distance` but does not yet consult it;
changing it currently has no optimization effect. Only `cores = 1` is supported
because parallel native Horner candidates can resolve equal-cost choices in a
scheduling-dependent order. Use `generate --workers` for deterministic parallel
sector compilation. Native expression hot starts and cancellation callbacks
are not run-card tuning settings; the callback-only abort level is not exposed.

`verbose = true` enables native optimizer logging and requires `--plain` without
`--json` or `--status-json`. Scalar settings are saved in generation metadata and
the evaluator artifact policy. Existing artifacts retain their original zero
Horner/unlimited-CPE settings, bytes and identities when loaded. These settings
apply to sector evaluators and runtime-dependent exact offsets, including native
eager/portable artifacts; local SymJIT compilation remains at O2. Auxiliary
literal-zero checks and mass-constraint predicates keep their separate fixed
settings.

The coefficient-expansion limits `max_series_attempts`, `max_relative_width`,
and `max_unique_requests` are optional caller caps, shown above as an example.
They apply only to `coefficient_series`. Existing subtraction limits govern the exact
`full_expression` fallback when an unregulated endpoint needs its pruning
decision. Failed series, resource limits and cancellation return errors; they do
not trigger a different algorithm. Relative depth is measured from each native series' leading epsilon power,
not an absolute Laurent cutoff for the final integral or a count of nonzero
terms. The unique-request cap counts distinct
derivative/face tuples within one pass, not intermediate memory or body size.

Coefficient-series snapshots include `coefficient_expansion`, with canonical
requested/effective methods, the expansion pass, formal pieces and request/alias
counts. Those counts reset on a new pass and do not count physical
contributions. The dashboard shows `epsilon expansion pass N (relative depth W)`.
Human counters are stage-aware. Subtraction-piece counts appear after the
composition finishes; resolved coefficient requests and shared expressions
appear during resolution, labeled as running counts, and become final counts
on completion. Earlier placeholder zeros are hidden. A genuine zero remains
visible once that count is available. These counters belong to the current
sector and pass, rather than the aggregate worker workload.

Subtraction pieces are intermediate endpoint terms before final grouping.
Coefficient requests identify distinct coefficient functions, derivatives and
boundary substitutions. Shared expressions are coordinate-dependent results
given reusable names. Raw JSON retains `formal_pieces`, `unique_requests` and
`aliases`: use the accompanying stage to distinguish unavailable placeholders
from reported counts.

JSON keeps the `attempt` and `relative_width` field names; zero denotes admission
before a series pass or the exact full-expression fallback. The outer completed
count advances only when a representative finishes. Runs directly selecting
`full_expression` omit this optional snapshot. Cancellation is checked on every
callback even when status output is coalesced; an individual native symbolic
call remains nonpreemptible. The CLI supplies options and presentation only;
the library owns coefficient generation, aliases and numerical conditioning.

This explicit opt-in is under validation. Accepted representative controls do
not establish completion of every full graph or matched performance. Saved
conditioning rows retain their existing numeric meaning; the fresh result's
descriptive conditioning basis is not inferred from a loaded artifact.

The on-shell triple-box card selects `coefficient_series` together with
`SingleUnitTerm { max_states: 32 }`. This preserves its original graph,
kinematics and normalization while using the validated eight-parameter family:

```sh
fastsecdec generate examples/runs/triple_box.toml --output output/triple_box.fsd
```

This case has completed generation of all 1,026 sector kernels through epsilon
order zero and separate-process artifact loading. Keep the artifact for later
integration. Full numerical agreement, convergence and benchmarking of this
case are deferred; successful generation does not establish those results.
The [retained generation evidence](../../docs/reviews/native-named-fullgraph-results.md)
records the completed stages and partial numerical attempts.

`generate --workers N` and `run --generation-workers N` select the caller-owned
generation pool (default: available logical cores). The earlier
`--geometry-workers` spelling remains an alias. Independent geometry, mapped
sectors, density/graph preparation, coefficient expansion and kernel compilation
use this same bounded pool. The coordinator registers prepared sector densities
in source order, confirming any candidate permutation with exact Symbolica
substitution before merging. It retains terminal input and cancellation, joins
launched jobs, and admits only complete native results.

“Finding equivalent sectors” distinguishes parallel comparison preparation from
the ordered exact-comparison step. Worker activity and the aggregate bar report
actual current-stage jobs. Percentage and ETA apply to that stage; downstream
work is discovered during generation. Elapsed time covers the whole run. Worker
rows describe task activity rather than operating-system CPU utilization.
The dashboard also samples whole-process RSS, its observed peak, and system
used/total/available RAM. Samples refresh at most every 500 ms on existing
dashboard polls; worker threads are included in process RSS. Unavailable
counters are reported explicitly, and the sampled peak is not an OS lifetime
high-water mark. JSON status includes the sample age and interval.
This option is separate from integration `--workers` and does not change artifact
or checkpoint identity. A resumed run loads its artifact without dispatching
generation. The CLI retains no geometry cache between commands; library callers
can supply their own dispatcher and retain a `GenerationContext`.

Integration methods are `qmc`, `adaptive_qmc`, `mc`, and `adaptive_mc`.
QMC uses the historical `kuo33002` catalogue by default. Select another native
published catalogue explicitly, without regenerating the symbolic artifact:

```sh
fastsecdec integrate output/bubble.fsd --lattice hkkn-alpha3
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
Escape requests cooperative cancellation and saves completed production work.
Discrete MC stops unfinished batches at a point boundary; their partial values
and replay state are excluded from the estimate and checkpoint. Resuming
production reissues missing complete batches with their original RNG streams.
QMC pilots can also be resumed. Havana MC pilots must be restarted because
the upstream library does not expose its mutable pilot training state; the
report marks this as `pilot_restart_required`.

During a discrete MC batch, the dashboard shows per-worker in-flight point
counts separately from accepted points. These observations keep changing even
when no complete batch is available for a new estimate. `--status-json` exposes
them as `in_flight_unaccepted`; plain progress also reports the active work.
The first interrupt waits for the current native point or evaluator setup to
return. A second Ctrl-C forces exit after restoring terminal settings, the
normal screen and cursor; it cannot save additional work. Repeated SIGINT or
SIGTERM also restores the terminal before forced exit on Unix. Normal returns,
errors and the panic hook restore terminal state as well. An uncatchable
SIGKILL cannot run cleanup; `reset` repairs an already affected shell.

The selected catalogue and refinement round must agree with the native session
inside a checkpoint. Missing historical `lattice` fields retain their original
`kuo33002` meaning. A CLI lattice override leaves the symbolic kernel identity
unchanged but changes sampling/checkpoint identity. Editing integration settings
in a source run card still follows the artifact's existing source-fingerprint
checks; it is not equivalent to overriding a saved artifact at integration time.

Runtime precision defaults to cancellation-distance routing through f64, native
106-bit DoubleFloat, and 1000-decimal-digit arbitrary precision. The dashboard
shows the final fraction in each class, plus Unstable, and distinguishes explicit
cutoff zeros from failures. The cutoff is disabled by default. Attempted native
evaluator calls and their timing are separate from final point classifications.
Weighted checks compare each complete complex coefficient with its previous
sector maximum and can advance the entire Laurent vector. The default f64
large-weight fraction is 0.9; higher levels disable this additional test.

Use `--integration-settings PATH` for a runtime TOML overlay. Select
`[stability] mode = "validated"` to retain the previous validation policy;
only that policy uses `[integration.replay]` with `growth_factor` (default 16)
and `minimum_bits` (default 128). Higher precision applies the sampling weight
before converting to binary64. See the
[runtime guide](../../docs/RUNTIME_INTEGRATION.md) for complete level settings,
exact endpoint-power overrides and the limitations of distance routing.

`--target-order 0` selects the full ε⁰ complex coefficient's RMS error as the
accuracy target; all coefficients and covariance remain stored. `--max-rounds N`
bounds refinement, independently of reaching the target. The
[ggHH guide](../../examples/gghh_double_box/README.md) includes eight-worker
Havana and QMC commands targeting 1% and 0.1% respectively.

The sector table supports arrows/PageUp/PageDown/Home/End for selection,
Left/Right for epsilon order, Tab or `s` to choose a sort column, and `r` to
reverse sorting. Click column headers to sort or reverse them, click a sector row
to select it, and use the mouse wheel to browse. These actions redraw cached
observations. Bordered panels show the full sum, iteration/lattice progress,
process/system RAM, selected-sector details and global diagnostics. Result
columns align their scientific multiplication dots; dashboard times use plain
decimal µs, ms and s. Sample and operational counts use four significant digits
with base-1000 K, M and B suffixes. The selected inset reports attributed sector
work, while
global totals include unassigned coordinator overhead.
Havana previews update during batches; QMC means
require a complete lattice and errors require two complete independent shifts.
Operational timing includes pilots and discarded prefixes and separates
integrator overhead, integrand overhead and evaluator calls. Wall time, summed
worker time and actual operating-system process CPU time are separate quantities.

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

New CLI artifacts use outer format version 2. Its scientific identity binds the
source provenance and the native kernel content ID; loading also verifies the
complete native payload and exact evaluator IR before returning kernels.
Historical outer version 1 files retain their canonical identities and loading
support. The embedded native kernel format remains version 3, with its bytes
preserved unchanged. Current writes stream that native JSON payload directly,
without expanding program byte arrays into a second JSON value tree.

Portable artifacts store exact native evaluator IR, compilation policy, source hashes,
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

`inspect output/gghh_double_box.fsd` shows an artifact overview, saved generation
facts and the ten largest sector evaluators, sorted by serialized evaluator size
with stable kernel IDs. Add `--sector 5` for the selected kernel's endpoint
monomials, remapping equations and detailed evaluator statistics. Human reports
use colored tables and Symbolica's native expression printer; `--plain`,
`NO_COLOR` and redirected output retain readable uncolored tables. `--json`
keeps the structured metadata route.

Evaluator sizes describe the serialized shared program for the complete Laurent
vector. Compressed SymJIT representations are identified separately; neither is
executable machine-code size. The `.dat` file additionally stores expressions,
Symbolica state and retained metadata, so its size need not equal the evaluator
sum. Source charts, symmetry representatives and numerical kernel IDs remain
distinct. Remapping equations and endpoint powers describe the density before
endpoint subtraction, while the evaluator can include boundary contributions.
Projective images are gauge-fixed, with their positive real measure factor.
Legacy artifacts explicitly report unavailable metadata rather than inventing
missing generation facts. A chart with no kernel can be exact, cancelled or
truncated; individual chart exact coefficients are not retained.
