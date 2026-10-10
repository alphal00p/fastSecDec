# Above-threshold native D05 input fixture

2026-10-10. The new `examples/contour/gghh_double_box_400/` fixture prepares the
existing D05 s-channel double box at the established 400 GeV `gg -> HH`, `++`
point. This is input admission only. No parametric/sector generation, contour
pilot, sampling or independent double-box reference has run. It does not turn
the unresolved coarse default-L=1 one-loop box result into an accepted result.

## Native ownership and unchanged source

The fixture references `examples/gghh_double_box/{graph.dot,model.json,parameters.json}`.
No asset is duplicated or modified. Strict `FeynmanDiagram::from_dot` import
uses `Model::from_json` and the same native `ParameterCard`. Applying the card
leaves the stored model fingerprint unchanged:
`1408ea162eab3d6c6d4bd68bc0f01c97525464a8cf3ad93248d03ddf5b17e390`.

The maintained `select::double_box` reuses Linnet circuit enumeration and the
native diagram: two loops, six top edges `[4,5,6,7,8,9]`, gluon edge 10, circuit
lengths `[4,4,6]`, and external box contents `[g,g]` and `[H,H]`. This is D05,
not the earlier FK018 crossed placement. The original labelled routing and
complete projected tensor numerator remain inside the unchanged native DOT.
No new standalone scalar numerator, color contraction, graph parser or CAS is
introduced. The source graph's BLAKE3 is
`bbf4a489ac1bfd93b68104f47c7ffa773bdabcb37625a866f446bd0f1432b986`.
The remaining seven source/owner identities are recorded in `validation.json`.

The reproducer includes the existing native `point.rs` and `select.rs`; it does
not duplicate their kinematic or topology construction. The full one-loop
exporter already calls this same `Point::with_sqrt_s` API. `RuntimeModelBindings`
checks model defaults and the actual internal-width restrictions without
contracting a numerator or constructing a parametric integrand. There is no
new production API or numerical implementation.

## Complete physical point and model audit

Native exact incoming momenta are `(200,0,0,+/-200)`, with outgoing momenta
`(200,+/-15*sqrt(39),0,+/-20*sqrt(39))`. External indices are explicitly required
to be `[0,1,2,3]`. Native `FourMomentum::dot` and Symbolica arithmetic verify
`p0^2=p1^2=0`, `p2^2=p3^2=15625`, `p0+p1=p2+p3`, and
`2*p0.p1=160000` exactly. Every evaluated component matches the tracked
`example/gg_hh_one_loop_ME/threshold/point.json` bit for bit.

The existing HEPKit `FourMomentum::wavefunction(Epsilon, PLUS)` regenerates
both incoming wavefunctions. Native checks cover transversality, circular
polarization null products, conjugate norms, cross-polarization product,
diagram routing and finite complete Gram data. The actual binary components
are transported to exact rational native Atoms before Gram construction;
the resulting runtime polarization product is preserved as
`-0.9999999999999998`, not rounded to an ideal normalization.

All 15 native Gram entries are matched to the original run card by native Atom
equality. The two incoming self-products remain generation-time exact zeros.
The other 13 entries are all regenerated, including ones whose values remain
zero. The reproducer first verifies that the same native method reproduces
every old 300 GeV value bit for bit, fencing the runtime names and routing.
The changed nonzero runtime values are:

| Input | 300 GeV | 400 GeV |
|---|---:|---:|
| p0p1 | 45000 | 80000 |
| p0p2 | 12550.1256289338 | 15020.008006406406 |
| p1p2 | 32449.874371066202 | 64979.99199359359 |
| p2eps1, p2eps2 | 35.17811819867572 | 66.23820649745885 |

`p2p2=15625`, all six zero momentum/polarization and self-polarization
products, and `eps1eps2` are recomputed, not copied. The exact native record
lists every product and both wavefunction component vectors.

The reused `parameters.json` has 26 keys, each admitted against native model
parameter names. It contains no momentum invariant or wavefunction binding.
The six retained evaluator model inputs (`Gf`, `MT`, `MZ`, `aEWM1`, `aS`, `ymt`)
are checked against `RuntimeModelBindings::defaults`. Their values are unchanged.
The card's `MH=125`, `MT=ymt=172.5`, `WT=WH=0` and relevant couplings are also
checked against the established 400 GeV reference point. The unrelated W/Z
width defaults remain in the full model card; only actual internal top/gluon
width restrictions are claimed, as enforced by the native admission API.

## Normalization and limits

D05 retains its unnormalized `delta_ab` color contraction and
`prod_l d^Dk_l/(i*pi^(D/2))` measure with multiplier 1. No spin/color average,
helicity sum, diagram sum, identical-particle factor or extra loop prefactor is
introduced. Internal Lorentz/Dirac algebra remains D-dimensional; only the
external native states are four-dimensional.

This differs from the coherent one-loop amplitude's `delta_ab/8` projection,
explicit gamma normalization and final `1/(16*pi^2)` factor. Matching the
physical point does not identify those observables or their normalizations.
An individual D05 contribution is not asserted to obey the full amplitude's
Ward cancellation. The earlier 300 GeV or FK018 calculations provide neither
an above-threshold reference nor convergence evidence for this input.

The run card enables contour capability but does not choose or validate a
runtime strength/cap. Polynomial/sign-aware construction, checked pilot points,
full Laurent covariance, convergence and independent references are subsequent
bounded gates. In particular, the successful one-loop small-cap prescription is
not silently adopted as an accepted double-box prescription.

## Validation record

The native-feature-gated maintained Cargo example passed after the explicit
external-label guard. Its three focused tests passed in 0.51 seconds: finite
native float export, complex-input refusal, and complete native reproduction of
the four committed cards (including refusal to overwrite existing output).
Focused Clippy with `-D warnings`, workspace formatting and diff whitespace
checks passed. The only lock change is the existing TOML dependency edge added
to FastSecDec's dev dependencies; no dependency version, pin or production
setting changed.

Executing the maintained binary alone took 0.515 seconds with 12,340 KiB peak
process RSS. This is a lightweight input check, not a double-box generation or
sampling measurement. The reproducer source SHA-256 is
`d04b5b40fc52034668bf986c78637f9dedaee73fcf4bda14fb6b1f498a2b42f1`;
the debug executable SHA-256 is
`91d6cbdda3515d26983eeceae8b8a6885d6029ad15eefd876242b0c1d69b8b4d`.
Ignored build/run logs, exact commands and resource records are in
`target/contour-gghh-double-box-400-fixture/`. The initial direct-rustc check
(0.616 seconds, 12 MiB) is retained separately and is superseded by this
maintained-entry verification.

The four emitted files are deterministic. Their relative input paths target
the maintained fixture directory; a staged output is for comparison and must
not be used as an executable run card at arbitrary directory depth.
Independent foundation review passed native reuse, all runtime inputs,
normalization, relative asset paths and file ownership. Root review also passed
the maintained reproducer, complete cards, unchanged native assets and explicit
normalization/acceptance boundaries. The fixture is accepted for subsequent
bounded generation; no contour or numerical result is thereby accepted.
