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
fastsecdec generate examples/gghh_double_box/run.toml --output output/gghh_double_box.fsd.json
fastsecdec inspect output/gghh_double_box.fsd.json
fastsecdec integrate output/gghh_double_box.fsd.json --full-integral \
  --workers 8 --points 1024 --shifts 8 --seed 20261005 \
  --checkpoint output/gghh_double_box.checkpoint.json \
  --save-result output/gghh_double_box.result.json
```

The card requests native named coefficient expansion and keeps the ordinary
domain check enabled. The sub-top-pair energy is a choice of physical point,
not a substitute for the check on this actual graph. Native input contraction and
Gaussian parameterization now complete with all 179 terms and seven parameters.
The ordinary CLI now certifies the actual F polynomial's algebraic coefficients
through Symbolica's exact real embedding and completes native geometry with 30
sectors. Native one-coordinate collection now preserves the factored regular
numerators during mapping. The optimized ordinary CLI now generates and saves
all 30 six-dimensional kernels, with orders `[-1, 0]`, in 61.285 seconds and
264 MiB peak RSS. Native SymJIT O2 compilation accounts for 0.453 seconds of
that total. The earlier unresolved `cas(2,coad(8))` is closed by the native
color option above. No threshold assertion or change of physical point was used.

The command above completes all 245,760 points and eight shifts per sector in
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
Keep this as a native example for now; browser generation and integration costs
have not been measured. See the
[native feasibility review](../../docs/reviews/gghh-native-feasibility.md).

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

To regenerate into a fresh directory, use the native model JSON as input:

```sh
cargo run -p fastsecdec --example gghh_double_box -- path/to/SM.json output/gghh-input
```

The Rust generator lives in `crates/fastsecdec/examples/gghh_double_box`. Native
canonical vector names in `run.toml` carry Spenso tensor metadata required by a
fresh CLI process; preserve their tagged spelling when editing the card.
