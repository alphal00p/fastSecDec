# D05 double box at 1000 GeV

This is the same native D05 s-channel `gg -> HH` double box as the
[400 GeV input](../gghh_double_box_400/README.md), now at `sqrt(s)=1000 GeV`,
`cos(theta)=4/5`, `mH=125 GeV`, and `mt=ymt=172.5 GeV`, with incoming `++`
gluons. It references the unchanged [graph](../../gghh_double_box/graph.dot),
[model](../../gghh_double_box/model.json) and
[model card](../../gghh_double_box/parameters.json).

The existing native `Point::with_sqrt_s` creates the exact external momenta
and HEPKit helicity wavefunctions. Native dot products verify both incoming
mass shells, outgoing Higgs mass shells, momentum conservation and
`s=1000000 GeV^2`. All 15 Gram entries are checked: the two incoming self-products
remain exact zeros before sector discovery; the other 13 are regenerated
runtime inputs. The six model inputs are unchanged. The actual native
`eps1eps2=-0.9999999999999998` normalization is retained.

The parsed generation card is identical to the 400 GeV card. Energy enters
only through `point.toml`; it does not change the graph, projected numerator,
model, regulator or parametric generation input. `point-exact.json` retains
the native exact data and `validation.json` records source identities and
input checks. The 400 GeV momentum-reference check is explicitly inapplicable
at this energy; it is not claimed as a 1000 GeV numerical reference.

The observable is the original individual D05 contribution with unnormalized
`delta_ab` contraction, `D=4-2*eps`, measure
`prod_l d^D k_l/(i*pi^(D/2))`, and multiplier 1. There is no spin/color average,
diagram sum, additional loop prefactor, or individual-diagram Ward claim.
The complete projected tensor numerator and overall factor stay in the
original native DOT. They must not be applied again.

Reproduce the input checks and cards from the repository root:

```sh
nix-shell --run 'cargo run --locked -p fastsecdec --example gghh_double_box_400 -- . target/gghh-double-box-1000-input-check --sqrt-s 1000'
nix-shell --run 'cargo test --locked -p fastsecdec --example gghh_double_box_400 -- --test-threads=1'
```

The exporter requires a fresh output directory. Its default remains 400 GeV;
the tests reproduce both maintained fixtures. Emitted relative asset paths
target this maintained directory, so temporary output cards are for comparison
and must not be executed at arbitrary directory depth.

Input admission has passed. Generation, checked contour admission and numerical
integration are separate actions. This fixture alone establishes no accepted
strength/cap, convergence, variance gain or independent double-box reference.
Saved families may contain undeformed capability, but physical integration at
this point must explicitly select an admitted fixed or dynamic prescription.
