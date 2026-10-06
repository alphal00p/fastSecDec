# Native Standard-Model double box

This input is one generated `g g -> H H` diagram: six massive top propagators
form a hexagon, and one gluon joins opposite vertices. HEPKit generates the
Standard-Model diagrams using only `ttg` and `ttH` interactions. Linnet selects
circuits of lengths `4, 4, 6`, with one external gluon and one Higgs on each
four-edge circuit. The first native generated-order match is `FK018`; eight
of the 192 generated diagrams match this selection.
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

The measurements below are historical results from the earlier build with fixed
kinematics. They do not measure or validate the new symbolic-parameter generation.
That build contracted 179 native input terms, generated 30 six-dimensional
kernels with orders `[-1, 0]`, and saved them in 61.285 seconds at 264 MiB peak
RSS, including 0.453 seconds of SymJIT O2 compilation. The native color option
above resolved the earlier `cas(2,coad(8))` input issue.

The historical coarse allocation completed all 245,760 points and eight shifts per sector in
8.781 seconds on eight workers, including 0.548 seconds of artifact loading.
It gives the following coarse, correlated estimates in the stated loop measure:

| Coefficient | Imaginary part | Standard error |
|---|---:|---:|
| `eps^-1` | 1.429705028 | 0.014733129 |
| `eps^0` | -33.713280106 | 0.346321087 |

Both real components are zero in the native result. The complete four-component
covariance and accepted checkpoint are saved. The finite coefficient's relative
standard error is 1.027%; this allocation stops at its work limit and does not
establish one-per-mil accuracy. There are 32,550 numerical rescues, reaching
256 bits, and no evaluation failures. These are native feasibility observations,
without an independent amplitude reference or a matched performance comparison.
The [HEPKit notebook](../hepkit/README.md) also offers this case as an optional
longer browser run; browser completion and costs remain unmeasured. See the
[native feasibility review](../../docs/reviews/gghh-native-feasibility.md).

The higher-statistics allocation can be requested with:

```sh
fastsecdec integrate output/gghh_double_box.fsd --full-integral \
  --parameters examples/gghh_double_box/point.toml \
  --method qmc --workers 8 --points 32768 --shifts 16 --seed 20261007 \
  --lattice hkkn-alpha3 --relative-tolerance 0.001 \
  --checkpoint output/gghh_double_box.permil.checkpoint.json \
  --save-result output/gghh_double_box.permil.result.json
```

This retains the card's Korobov-3 transform and ordinary precision rescue.
The measured allocation completes **15,728,640 evaluations in 377.530 seconds**
on eight workers. The full Laurent vector, including both real components and
all covariance entries, remains in the saved result:

| Coefficient | Imaginary part | Standard error |
|---|---:|---:|
| `eps^-1` | 1.439027539 | 0.000136375 |
| `eps^0` | -33.936949143 | 0.003180097 |

The finite-part relative standard error is **0.00937%**, below the requested
0.1%. Both nonzero components agree with the original coarse run within 0.65
combined standard errors and with the intermediate independent-seed Kuo run
within 0.97. This checks numerical consistency, not agreement with an
independently computed amplitude. There are 2,053,614 rescues, at most 256 bits,
and zero evaluation failures. These observations use the retained native build
and artifact; they do not predict browser runtime. The smaller allocation above
remains useful for exploring the example.

The same generated kernels also support ordinary Havana sampling with adaptive
discrete sector probabilities and continuous grids:

```sh
fastsecdec integrate output/gghh_double_box.fsd --full-integral \
  --parameters examples/gghh_double_box/point.toml \
  --method discrete_mc --workers 8 --points 32768 --shifts 32 --seed 20261008 \
  --relative-tolerance 0.001 \
  --checkpoint output/gghh_double_box.mc.checkpoint.json \
  --save-result output/gghh_double_box.mc.result.json
```

Here points and shifts mean **global points per batch** and **independent
batches**. The default three pilot epochs use 98,304 samples, excluded from the
1,048,576-sample production estimate. The completed run gives
`eps^-1 = 1.437574727 i ± 0.000846799 i` and
`eps^0 = -33.904619327 i ± 0.019561493 i`, retaining the full covariance.
Finite-part relative standard error is **0.05770%**. Both coefficients agree
with QMC within 1.70 combined standard errors. The eight-worker process took
262.477 seconds including the pilot, with no evaluation failures. These are
separate observed allocations, not a matched performance comparison. See the
[sampling review](../../docs/reviews/havana-discrete-sector-sampling.md) for
precision-rescue counts, checkpoint validation and reference qualifications.
In the notebook, **New integration** retains the generated owners and previous
report so the same kernels can be used for either method.

The independent [sector geometry check](../../docs/reviews/gghh-sector-sanity.md)
matches all 30 native maps to pySecDec's denominator decomposition. A native
numerator-inclusive check before endpoint subtraction finds maximum denominator
power `a = 0`: extracted coordinate factors are `1`, `x^eps`, `x^(1+eps)` and
`x^(-eps)`. The Laurent pole is not an endpoint `1/x` power.

The historical artifacts retained these facts directly: 30 charts, 54 mapped
terms and 324 endpoint powers, with their native prefactors and variable maps.
The notebook's selected-chart view exposes them before any integration. The
fresh generation/reload check reproduces every exact evaluator program from the
original artifact; only the optional inspection metadata changes. For this
build, each shared two-output complex program contains 2,881–2,998 native
evaluator operations and occupies 35,084–35,506 exact-program bytes. Its
compressed SymJIT application occupies 42,261–43,322 bytes. These are recorded
program statistics, not machine-code sizes or per-sample timing estimates.

The earlier 600-second input timeout, rational-only domain rejection, first-sector
support-expansion timeout (19.5 GiB peak RSS) and bounded diagnostic failures are
retained.
Native early source elimination and fixed-variable scaling admission now preserve
the factored Gaussian numerator instead of expanding its full parameter support.

`raw-diagram.json`, `raw-diagram.dot` and the `raw-*.txt` files retain the native
generated numerator and its separate factors. `graph.dot` carries the native
color-projected numerator and the remaining Lorentz projector, and evaluates
the generated bookkeeping factor through HEPKit's existing factor API.
`color-projected-numerator.txt` records that numerator. Its couplings and diagram
weight enter exactly once; the diagnostic symmetry factor is not multiplied again.
`generation.json` and `provenance.json` record selection and input construction.
The full native model and numeric parameter card are included.

To regenerate into a fresh directory, use the ordinary Cargo dependencies
described in the [development guide](../../docs/DEVELOPMENT.md) and supply the
native model JSON as input:

```sh
cargo run --locked -p fastsecdec --example gghh_double_box -- path/to/SM.json output/gghh-input
```

The Rust generator lives in `crates/fastsecdec/examples/gghh_double_box`. Native
canonical vector names in `run.toml` carry Spenso tensor metadata required by a
fresh CLI process; preserve their tagged spelling when editing the card.
