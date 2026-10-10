# D05 double box at 400 GeV

This is an input fixture for the original D05 s-channel `gg -> HH` double box
at `sqrt(s)=400 GeV`, `cos(theta)=4/5`, `mH=125 GeV`, and `mt=ymt=172.5 GeV`,
with incoming `++` gluons. It references the unchanged native
[graph](../../gghh_double_box/graph.dot), [model](../../gghh_double_box/model.json)
and [model card](../../gghh_double_box/parameters.json); it does not copy their
large assets. The original 300 GeV example remains unchanged.

The input has passed native import, topology, on-shell, momentum conservation,
helicity and complete Gram-data checks. **No double-box generation, contour
pilot, numerical integration or independent numerical reference has been run
for this fixture.** Being above the top-pair threshold does not establish
contour admissibility or numerical convergence.

`run.toml` retains the original symbolic kinematics, regulator, order and
coefficient expansion, and enables contour-capable generation. The two incoming
self-products remain exact zero before sector finding. `point.toml` contains
all 13 regenerated runtime Gram products and the six unchanged model inputs.
`point-exact.json` retains native exact momenta/products and the actual native
helicity components. `validation.json` records input hashes and the checks.
All point-card numbers are finite TOML floats; notably the native polarization
product remains `eps1eps2=-0.9999999999999998`, without idealizing it to `-1`.

The normalization is the original D05 **unnormalized `delta_ab` contraction**,
`D=4-2*eps`, and loop measure `prod_l d^D k_l/(i*pi^(D/2))`, with
`measure_multiplier="1"`. There is no colour/spin average, diagram sum or
additional physical prefactor. The complete projected tensor numerator,
Lorentz projector and evaluated overall factor remain in the original native
DOT. This individual diagram is not asserted to be gauge invariant. Its
normalization differs from the full one-loop `delta_ab/8` amplitude with its
separate gamma factor and final `1/(16*pi^2)` conversion.

To reproduce only the input checks and cards, from the repository root with the
licensed native Rust environment:

```sh
nix-shell --run 'cargo run --locked -p fastsecdec --example gghh_double_box_400 -- . target/gghh-double-box-400-input-check'
nix-shell --run 'cargo test --locked -p fastsecdec --example gghh_double_box_400'
```

The first command requires a fresh output directory and never overwrites one.
The optional `--sqrt-s GEV` argument selects another native point; the default
remains 400 GeV. The maintained [1000 GeV fixture](../gghh_double_box_1000/README.md)
uses the same exporter and shared physical input.
It calls the existing `Point::with_sqrt_s` and native topology/model APIs; it
does not contract the numerator or generate/integrate sectors. The emitted
`../../gghh_double_box/` references deliberately target **this maintained
fixture directory**. Staged cards are for comparison; do not execute their run
card at an arbitrary output depth. The test regenerates the four cards in a
temporary directory and compares them byte for byte with this fixture.

Generation and integration remain separate future actions with independently
chosen, documented resource bounds and runtime contour settings. No default
or small dynamic cap has been accepted for D05 at this point. See the
[fixture audit](../../../docs/reviews/contour-gghh-double-box-fixture.md).
