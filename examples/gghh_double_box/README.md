# Native Standard-Model double box

This input uses the supplied **D05 s-channel** `g g -> H H` diagram. Six
massive top propagators form a hexagon, and an internal gluon joins opposite
vertices. Both incoming gluons attach to one four-edge circuit; both outgoing
Higgs legs attach to the other. The exact supplied source is retained in
`source-diagram.dot`, including its vertex and edge labels, fermion flow,
tensor numerator, and loop routing (loop edges `4` and `7`).

HEPKit generates the Standard-Model diagrams using only `ttg` and `ttH`
interactions. Linnet verifies circuits of lengths `4, 4, 6` and external boxes
`[g,g]` and `[H,H]`. HEPKit's native colored canonical key then verifies that
D05 belongs to the generated set. The exported input preserves D05 itself,
including its routing; it does not choose the first matching channel.
The model's internal gluon propagator is in Feynman gauge,
`-i g(mu,nu) / p^2`.

The point uses `sqrt(s) = 300`, `mt = ymt = 172.5`, `mH = 125`, zero widths and
`cos(theta) = 4/5`. The incoming gluons are massless; each outgoing Higgs has
momentum squared `125^2`. Exact external momentum components preserve momentum
conservation and these invariants. Incoming `(+,+)` gluon wavefunctions use
HEPKit's shared GammaLoop/MadGraph convention, with metric `+---`. Their supplied
binary floating-point components are transported losslessly as native rational
coefficients. The generator checks transversality, circular-polarization null
products, normalization and the complete finite Gram data before export.

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

The run card declares symbolic scalar products. Generation compiles these as
ordered runtime inputs; `point.toml` supplies the numerical point when integrating.
Use repeated `--parameter NAME=VALUE` options to override values in that file.
Values can be finite real numbers or native Symbolica expressions such as
`"22500-3000*sqrt(11)"`. Every declared symbol requires a value, including zeros;
unknown, missing, and nonfinite values are rejected. Runtime values are passed
unchanged to each evaluator, its precision-rescue path, and folded exact offsets.
They are included in checkpoint and numerical-result identity. Model masses and
couplings in `parameters.json` remain fixed generation inputs.

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

The earlier example selected **FK018**, with one gluon and one Higgs on
each box. Its numerical values, sector counts, timings and reference checks
apply to that different diagram. They are retained only in historical review
documents and establish no result for D05. The small integration command above
is a smoke allocation, not a convergence or independent-reference claim.
The [s-channel review](../../docs/reviews/gghh-s-channel-review.md) records the
exact source comparison, fresh generation and bounded runtime verification.

The same generated kernels also support ordinary Havana sampling:

```sh
fastsecdec integrate output/gghh_double_box.fsd --full-integral \
  --parameters examples/gghh_double_box/point.toml \
  --method discrete_mc --workers 8 --points 32768 --shifts 32 --seed 20261008 \
  --relative-tolerance 0.001 \
  --checkpoint output/gghh_double_box.mc.checkpoint.json \
  --save-result output/gghh_double_box.mc.result.json
```

For this method, points and shifts mean global points per batch and independent
batches. Pilot samples are excluded from the production estimate. Neither this
command nor its work limit guarantees the requested statistical accuracy.

`raw-diagram.json`, `raw-diagram.dot` and the `raw-*.txt` files retain D05's native
supplied numerator and its separate factors. `graph.dot` carries the native
color-projected numerator and the remaining Lorentz projector, and evaluates
the generated bookkeeping factor through HEPKit's existing factor API.
`color-projected-numerator.txt` records that numerator. Its couplings and diagram
weight enter exactly once; the diagnostic symmetry factor is not multiplied again.
`generation.json` and `provenance.json` record selection and input construction.
The full native model and numeric parameter card are included.

D05's original model fingerprint is verified by strictly importing its source
against HEPKit's embedded `Model::standard_model()`. The exporter then applies
the native numeric parameter card. Since HEPKit has no public model-rebinding
method, Linnet's native DOT object transports the updated model fingerprint.
A complete native serialized-payload comparison checks that only the model
fingerprint and derived diagram identity change: labels, tensor fragments,
factors, half-edge order and momentum signatures remain identical. The source
and physical model fingerprints and source hash are recorded in provenance.

To regenerate into a fresh directory, use the ordinary Cargo dependencies
described in the [development guide](../../docs/DEVELOPMENT.md). Supply the
bundled physical model, or the unmodified native Standard-Model JSON:

```sh
cargo run --release --locked -p fastsecdec --example gghh_double_box -- examples/gghh_double_box/model.json output/gghh-input
```

The Rust generator lives in `crates/fastsecdec/examples/gghh_double_box`. Native
canonical vector names in `run.toml` carry Spenso tensor metadata required by a
fresh CLI process; preserve their tagged spelling when editing the card.
