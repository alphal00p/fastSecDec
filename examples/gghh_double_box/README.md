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

From the repository root:

```sh
fastsecdec generate examples/gghh_double_box/run.toml --output output/gghh_double_box.fsd.json
fastsecdec inspect output/gghh_double_box.fsd.json
fastsecdec integrate output/gghh_double_box.fsd.json --workers 8
```

The card requests native named coefficient expansion and keeps the ordinary
domain check enabled. The sub-top-pair energy is a choice of physical point,
not a substitute for the check on this actual graph. Native input contraction and
Gaussian parameterization now complete with all 179 terms and seven parameters.
The ordinary CLI now certifies the actual F polynomial's algebraic coefficients
through Symbolica's exact real embedding and completes native geometry with 30
sectors. Native one-coordinate collection now preserves the factored regular
numerators during mapping. The latest bounded attempt completes 17 mapping
intervals before its 180-second deadline (186 MiB peak RSS); recorded mapping
and symmetry times are 42.3 and 131.8 seconds respectively. No threshold
assertion or change of physical point was used. No
artifact or completed integral has been produced, so this example's generation
prerequisite remains incomplete. No browser performance claim is made here.

The earlier 600-second input timeout, rational-only domain rejection, first-sector
support-expansion timeout (19.5 GiB peak RSS) and bounded diagnostic failures are
retained.
Native early source elimination and fixed-variable scaling admission now preserve
the factored Gaussian numerator instead of expanding its full parameter support.

`raw-diagram.json`, `raw-diagram.dot` and the `raw-*.txt` files retain the native
generated numerator and its separate factors. `graph.dot` changes only the
external projector and evaluates the generated bookkeeping factor through
HEPKit's existing factor API. The numerator, couplings and diagram weight enter
exactly once; the diagnostic symmetry factor is not multiplied again.
`generation.json` and `provenance.json` record selection and input construction.
The full native model and numeric parameter card are included.

To regenerate into a fresh directory, use the native model JSON as input:

```sh
cargo run -p fastsecdec --example gghh_double_box -- path/to/SM.json output/gghh-input
```

The Rust generator lives in `crates/fastsecdec/examples/gghh_double_box`. Native
canonical vector names in `run.toml` carry Spenso tensor metadata required by a
fresh CLI process; preserve their tagged spelling when editing the card.
