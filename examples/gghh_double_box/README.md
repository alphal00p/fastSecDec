# Native Standard-Model double box

This input uses the supplied **D05 s-channel** `g g -> H H` diagram. Six
massive top propagators form a hexagon, and an internal gluon joins opposite
vertices. Both incoming gluons attach to one four-edge circuit; both outgoing
Higgs legs attach to the other. The complete input is retained in `graph.dot`,
including its vertex and edge labels, fermion flow, projected tensor numerator
and loop routing (loop edges `4` and `7`).

HEPKit generates the Standard-Model diagrams using only `ttg` and `ttH`
interactions. Linnet verifies circuits of lengths `4, 4, 6` and external boxes
`[g,g]` and `[H,H]`. HEPKit's native colored canonical key then verifies that
D05 belongs to the generated set. The exported input preserves D05 itself,
including its routing; it does not choose the first matching channel.
The model's internal gluon propagator is in Feynman gauge,
`-i g(mu,nu) / p^2`.

The point uses `sqrt(s) = 300`, `mt = ymt = 172.5`, `mH = 125`, zero widths and
`cos(theta) = 4/5`. The incoming gluons are massless; each outgoing Higgs has
momentum squared `125^2`. The input construction checks exact momentum
conservation and these invariants.
Incoming `(+,+)` gluon wavefunctions use HEPKit's shared GammaLoop/MadGraph
convention, with metric `+---`. The generator checks transversality,
circular-polarization null products, normalization and the complete finite Gram
data before export. `point.toml` contains ordinary decimal floating-point values,
with enough digits to preserve the original runtime f64 inputs exactly.

Color is contracted with **unnormalized `delta_ab`**. There is no color average,
spin average, helicity sum or sum over diagrams. This individual contribution
is not asserted to be gauge invariant or a cross section. Internal Lorentz and
Dirac algebra retains `D = 4 - 2 eps`; only external wavefunctions have four
components. FastSecDec uses its normalized Minkowski loop measure
`prod_l d^D k_l / (i*pi^(D/2))`, with no extra scale or Euler-gamma factor.
The exporter closes color through HEPKit/Idenso's native explicit SU(3)
invariant option, with `T_F = 1/2`, before exporting the numerical input.
This consumes the color delta once; the remaining projector contains the two
Lorentz wavefunctions. A native exact check compares this result with the
symbolic-Casimir policy followed by the existing invariant conversion.

From the repository root:

```sh
./target/release/fastsecdec generate examples/gghh_double_box/run.toml --output output/gghh_double_box.fsd --workers 8
fastsecdec inspect output/gghh_double_box.fsd
fastsecdec integrate output/gghh_double_box.fsd --full-integral \
  --parameters examples/gghh_double_box/point.toml \
  --workers 8 --points 1024 --shifts 8 --seed 20261005 \
  --checkpoint output/gghh_double_box.checkpoint.json \
  --save-result output/gghh_double_box.result.json
```

The run card fixes the incoming gluon self-products `P(0)^2 = P(1)^2 = 0`
exactly with `value = "0"`. These conditions are applied before parametrization
and sector finding, so their infrared boundaries are included in the generated
Laurent expansion. The other thirteen scalar products remain symbolic runtime
inputs; `point.toml` supplies their numerical values when integrating.

Regenerate artifacts previously built with runtime `p0p0` and `p1p1`. Binding
those generic inputs to zero only at integration time cannot recover sectors or
poles absent from the generic generation. The new point card omits these two
fixed inputs; it is intended for freshly generated on-shell artifacts.
Use repeated `--parameter NAME=VALUE` options to override values in that file.
Values can be finite real numbers or native Symbolica expressions such as
`"22500-3000*sqrt(11)"`. Every declared symbol requires a value, including zeros;
unknown, missing, and nonfinite values are rejected. Runtime values are passed
unchanged to each evaluator, its precision-rescue path, and folded exact offsets.
They are included in checkpoint and numerical-result identity.

Model inputs are runtime parameters too. The native model resolves dependent
masses and couplings analytically into their contributing independent inputs;
only those used by this integral are retained, under names such as `model::MT`
and `model::ymt`. `point.toml` includes their default values. Changing a mass,
Yukawa input or coupling input uses the existing evaluator without regeneration.
The card in `parameters.json` supplies metadata defaults, the strict source
model identity and zero-width restrictions; its numerical defaults are not
substituted into the generated evaluator. Complex independent inputs use
`model::NAME_re` and `model::NAME_im`; dependent complex coupling phases remain
native analytic expressions of real inputs.

All internal widths must be explicitly zero. A generic named propagator mass
must evaluate to a finite, real, nonzero value at integration time. Setting it
to zero requires a separately generated massless specialization because the
endpoint and Laurent structure can change. The model's literal `ZERO` remains
exactly massless. For an intentional fixed specialization, set
`model_parameters = "fixed"` in `[input]`, or fix selected native `UFO::NAME`
values in the run card's `[parameters]`. This restriction does not certify
thresholds or impose their signs.

The output base `output/gghh_double_box.fsd` names two adjacent files:
`.fsd.json` contains human-readable metadata and `.fsd.dat` contains native
binary programs and expressions. Move both files together. The base itself is
not a file. Generation does not certify thresholds; the physical domain and
absence of unsupported thresholds are the caller's responsibility.

The local CLI uses SymJIT O2. The library's existing `portable` feature retains
its eager Symbolica interpreter with the same runtime-parameter and caller-owned
integration APIs; it needs no SymJIT. This is the route for subsequent Pyodide/
marimo work. See the [runtime API review](../../docs/reviews/runtime-kinematic-parameters.md)
for the concrete feature and API sequence; no new notebook run is claimed here.

Generation keeps the existing `symbolic` mode by default. To construct endpoint
coefficients with native Taylor jets instead of substituting entire sector maps
into the numerator, set these entries in the run card:

```toml
[generation]
mode = "numerical_dual"
subtraction = "taylor" # or "integrate_by_parts"
order = 0
```

Both subtraction choices retain exact analytic endpoint denominators and the
complete interior function. The new mode composes the original polynomial
evaluators, monomial maps and normalized derivative coefficients into native
evaluator programs. Runtime kinematics, model inputs, SymJIT O2, eager evaluation
and point batches use the same interfaces. Korobov smoothing remains a subsequent
integration-coordinate transformation, after endpoint subtraction or IBP.

The numerical-dual mode keeps source charts separately rather than comparing
large symbolic densities for symmetry. Unsupported ordinary-jet charts, including
signed infinity maps and unregulated endpoint admission, use the recorded exact
symbolic route. Native dense jets have a guarded component limit. Full expressions
are restored only for explicit inspection of the retained generated object;
generation and saving do not perform that restoration. See the
[implementation review](../../docs/reviews/numerical-dual-evaluator.md) for the
precise native ownership and limitations.
The [double-box comparison](../../docs/reviews/numerical-dual-benchmark.md) records
historical generic-kinematics generation and evaluator measurements. Those
artifacts kept the gluon self-products as runtime inputs, so their sector counts,
pole structure and timings do not validate this on-shell specialization.

The earlier example selected **FK018**, with one gluon and one Higgs on
each box. Its numerical values, sector counts, timings and reference checks
apply to that different diagram. They are retained only in historical review
documents and establish no result for D05. The small integration command above
is a smoke allocation, not a convergence or independent-reference claim.
The [s-channel review](../../docs/reviews/gghh-s-channel-review.md) records the
exact source comparison, fresh generation and bounded runtime verification.

For Havana importance sampling over sectors, target **1% relative precision in
the complex ε⁰ coefficient** with eight workers:

```sh
./target/release/fastsecdec integrate output/gghh_double_box.fsd --full-integral \
  --parameters examples/gghh_double_box/point.toml \
  --method discrete_mc --workers 8 --points 32768 --shifts 32 --seed 20261008 \
  --target-order 0 --relative-tolerance 0.01 --absolute-tolerance 0 --max-rounds 8 \
  --checkpoint output/gghh_double_box.mc.checkpoint.json \
  --save-result output/gghh_double_box.mc.result.json
```

For randomized lattice QMC, target **0.1% (one per mil)** in the same coefficient:

```sh
./target/release/fastsecdec integrate output/gghh_double_box.fsd --full-integral \
  --parameters examples/gghh_double_box/point.toml \
  --method qmc --workers 8 --points 4096 --shifts 32 --seed 20261008 \
  --target-order 0 --relative-tolerance 0.001 --absolute-tolerance 0 --max-rounds 8 \
  --checkpoint output/gghh_double_box.qmc.checkpoint.json \
  --save-result output/gghh_double_box.qmc.result.json
```

For Havana, points and shifts mean global points per batch and independent
batches. Pilot samples train importance sampling and are excluded from the
production estimate. For QMC, they mean points per shifted lattice and the
number of random shifts. The initial allocation is refined up to eight rounds:
MC doubles points per batch, while QMC grows the lattice before adding shifts
when the selected catalogue reaches its size limit. Reaching the work limit
does not establish the requested accuracy; the final stopping reason distinguishes
it from reaching the target. Only complete accepted production allocations can
satisfy the target. All Laurent orders and the full covariance remain stored.
The complex target compares `sqrt(C_RR + C_II)` to the relative tolerance times
`hypot(mean_R, mean_I)`; zero absolute tolerance prevents an absolute-error
fallback from satisfying these relative targets.

The dashboard refreshes once per second by default. Change this with
`--status-interval-ms`; cancellation remains responsive independently. Havana
previews show current-iteration point statistics during batches. QMC updates a
central value after a complete lattice and an uncertainty after at least two
independent complete shifts. Preview coverage can differ between sector rows
and the full sum, and previews are never checkpointed as accepted results.

Runtime precision routing defaults to f64, native 106-bit DoubleFloat and
1000-decimal-digit arbitrary precision, selected by effective cancellation
distance. It also escalates large f64 weighted contributions against previous
sector maxima. This distance policy is a numerical heuristic; persistent
nonfinite results are errors. An optional distance cutoff that returns zero is
disabled by default; its bias is not included in sampling errors. The explicit
validated policy retains the earlier additional precision checks. Runtime
settings can be supplied with `--integration-settings PATH`, overriding artifact
defaults; explicit CLI options take precedence. These settings do not recompile
the generated expressions. See the [runtime settings guide](../../docs/RUNTIME_INTEGRATION.md)
for precision settings and live diagnostics.

The directory contains the six files needed to document, generate and integrate
this example: `README.md`, `run.toml`, `graph.dot`, `model.json`,
`parameters.json` and `point.toml`. The complete graph, color-projected tensor
numerator, Lorentz projector and native evaluated overall factor are in
`graph.dot`. These factors enter exactly once; the diagnostic symmetry factor
is not multiplied again. Separate numerator snapshots are not required.

D05's original model fingerprint is verified by strictly importing its developer
fixture against HEPKit's embedded `Model::standard_model()`. The exporter then
applies the native numeric parameter card. Since HEPKit has no public
model-rebinding method, Linnet's native DOT object transports the updated model
fingerprint. A complete native serialized-payload comparison checks that only
the model fingerprint and derived diagram identity change: labels, tensor
fragments, factors, half-edge order and momentum signatures remain identical.

To regenerate into a fresh directory, use the ordinary Cargo dependencies
described in the [development guide](../../docs/DEVELOPMENT.md). Supply the
bundled physical model, or the unmodified native Standard-Model JSON:

```sh
cargo run --release --locked -p fastsecdec --example gghh_double_box -- examples/gghh_double_box/model.json output/gghh-input
```

The Rust generator lives in `crates/fastsecdec/examples/gghh_double_box`. Native
canonical vector names in `run.toml` carry Spenso tensor metadata required by a
fresh CLI process; preserve their tagged spelling when editing the card.
