# Independent auxiliary-vector input review

The native graph adapter and CLI transport have no unresolved source-review
finding. The retained focused gates pass two library tests and four CLI input
tests. This review does **not** certify a fresh-process import or the generated
gg→HH example: cold CLI validation remains pending.

## Native ownership and identity

[`GraphIntegral::with_auxiliary_external_momenta`](../../crates/fastsecdec/src/input/graph.rs)
extends the external list through the existing HEPKit `IntegralFamily::new`.
Native `Kinematics` and family construction continue to own momentum admission,
duplicate-label and loop-assumption rejection, scalar-product bases and rank.
There is no new tensor reduction, Gram solve, DOT parser or momentum-routing
implementation. The diagram `Arc`, ordered propagators, powers and graph routing
are preserved. Existing numerator contraction and Gaussian parameterization
still apply the graph numerator, projector, scalar prefactors and measure once.

`rebuild_family` preserves scoped kinematics and binds scalar-product values
through existing literal Symbolica substitution. It supports binding scalar
Gram values before or after auxiliary declaration. Empty auxiliary input leaves
the historical route unchanged. The Gram-degenerate admission is specific to
this Gaussian route; it does not extend the contract of Gram-inverting APIs.

Native Spenso vector atoms become representation-bearing function heads in
scalar products. A literal scalar-variable replacement does not replace such
a function head; the initially investigated bare-name collision was therefore
not established as a defect, and this review adds no speculative guard.

## CLI schema and cold-import boundary

[`MomentumName`](../../crates/fastsecdec-cli/src/config.rs) retains integer
`P(index)` shorthand and adds strings parsed by the existing Symbolica parser.
The optional `auxiliary_momenta` list defaults to empty. Existing cards remain
valid, and the existing `RunCardWithoutReference` fingerprint covers the new
kinematics fields while preserving historical card hashing.

The CLI and native DOT parser use the same default `feynkit_graph` namespace.
Generated cards use `Atom::to_canonical_string()` for vector labels, retaining
Spenso tensor/rank-one tags before DOT parsing in a new process. Plain
previously unregistered names are not an equivalent metadata transport. Native
DOT model-fingerprint checks remain in place. Existing numerator contraction
uses native Lorentz-dimension conversion and contraction; the adapter does not
reinterpret numerical four-dimensional external vectors as four-dimensional
internal tensor algebra.

The CLI unit control exports native DOT and reloads it with canonical vector
labels, mixed integer/string scalar-product declarations and a scalar binding.
It rejects an undeclared numerator vector. This is a **same-process** reload;
source inspection of canonical transport is not a substitute for the pending
standalone CLI run.

## Native complex coefficients

The narrow `model_values` correction constructs a numerical parameter as
`real + imaginary * Atom::i()`. Symbolica's `Atom::i()` returns a native complex
rational number. The former parsed expression `real + imaginary * i` could
create an ordinary namespace-qualified symbol instead. In that first correction,
decimal conversion of the two supplied components was unchanged, as were the precedence of
analytic internal definitions, explicit restriction values and inline
overrides. Coupling expressions retain their existing native owner.

The retained diagnostic card
[`gghh-native-generator-1/input/run.toml`](../../output/diagnostics/gghh-native-generator-1/input/run.toml)
contains the concrete unintended `gghh::{}::i^2` terms. It is evidence of the
discovered export problem, not an accepted physical input. The dedicated CLI
regression imports the native model value `[2.0, 3.0]` and checks exact equality
with `2 + 3 * Atom::i()`. This closes the numeric model-parameter transport bug
without introducing custom complex arithmetic or claiming that the whole
generated example has already passed cold import.

## Reviewed evidence

The reviewer read source and retained logs only; no additional build or
scientific process was launched.

- [`tests-4.log`](../../output/diagnostics/gghh-native-input-1/tests-4.log):
  **2 passed, 0 failed**, test runtime 0.10 seconds. The library control now uses
  `Atom::i()` for a genuinely complex mutual Gram product and compares the
  Gaussian result with native `TensorReducer` through epsilon order one. The
  other control checks binding order and native duplicate/loop-label rejection.
  SHA-256: `08c2054807ffafbed793c098e176bb6cc97430fcc7214e82feea7d41d861c87e`.
- [`cli-input-3.log`](../../output/diagnostics/gghh-native-input-1/cli-input-3.log):
  **4 passed, 0 failed**, test runtime 0.08 seconds. Coverage includes literal
  parameter names, analytic/restricted model values, auxiliary-vector loading
  and the native imaginary-coefficient regression.
  SHA-256: `3f7fe3e18e832a7810f38c71bcb1b02c4c16467cff529ce2620c78b78bda932d`.

Reviewed source SHA-256 values:

| File | SHA-256 |
| --- | --- |
| `crates/fastsecdec/src/input/graph.rs` | `80550126d0c7aba1400e899db43c35aab08c14b00ee27e10d3273e760c55085b` |
| `crates/fastsecdec-cli/src/config.rs` | `0aec935b2e72f8a1bca7ce3caea45c44c1322800d144cb2ef0b2c30444943b34` |
| `crates/fastsecdec-cli/src/input.rs` | `182f6122885f11a9e6ad8c4a5fc468f9c7088c499900c0c51893c25373196353` |
| `crates/fastsecdec/tests/auxiliary_momenta.rs` | `20ca3c286e911f883f1ffd18b5308ad8c3b63331217eb65b1228bd3d7f777f21` |

Earlier failed and superseded logs remain intact. This focused acceptance
neither changes benchmark settings nor resumes the paused campaign.

## Exact numeric input and generated fixture follow-up

The subsequent source revision replaces decimal formatting/reparsing with
native `Rational::try_from(f64)` in numerical model parameters, TOML numeric
parameters and the example's wavefunction-component transport. The existing
Numerica implementation preserves the exact finite binary value and rejects
nonfinite input. It introduces no rationalization heuristic or custom numeric
conversion. Analytic model expressions, explicit restriction precedence and
native `Atom::i()` remain intact. Exact binary transport does not claim that an
arbitrary decimal input is its mathematical decimal rational; the CLI README
now explains using an expression string such as `"1/10"` for that intent.

The strengthened CLI regression checks `[2.5, 3.25]` against
`5/2 + (13/4) * Atom::i()` and a numeric TOML mass `172.5` against `345/2`.
[`cli-input-4.log`](../../output/diagnostics/gghh-native-input-1/cli-input-4.log)
records **4 passed, 0 failed**, with a test runtime of 0.07 seconds. Its SHA-256
is `7592d5e57212a0258fb0348ce04c6f2abc07072fa748406c1430359b6d69df3b`.
The reviewed updated CLI input source SHA-256 is
`585c23442fc9f2dc68b2193cacc681ab2bed38fb9663d615077d4da79bc82408`;
the example's `point.rs` SHA-256 is
`f023c7feafa28e33ad14433de3767553d464bb7356a5f49d0e98038016716b7d`.
The earlier source table and input-3 log remain historical evidence.

The generated [`examples/gghh_double_box`](../../examples/gghh_double_box/README.md)
fixture retains raw native diagram and factor data, the full model, parameter
card, selected topology and projection provenance. Source inspection confirms
native diagram generation, native Linnet cycle selection and the required six
top lines plus one gluon joining opposite hexagon vertices. The selected
`FK018` is one of eight matching diagrams among 192 generated diagrams; this
selection is not an amplitude sum. Existing HEPKit wavefunctions supply the
incoming `(+,+)` states, native FourMomentum products supply the Gram entries,
and the color contraction is explicitly unnormalized `delta_ab`. The selected
diagram's couplings, fermion-loop sign and weight remain present once.

The fixture uses the requested 300 GeV center-of-mass energy, 125 GeV Higgs,
172.5 GeV top/Yukawa mass and an explicit cosine of `4/5`. The outgoing Higgs
momenta have nonzero square `125^2`; the incoming gluons have square zero.
The exported card retains dimension `4-2*eps`, order zero and ordinary threshold
admission. It makes no assertion that sub-top-pair energy alone proves that
admission. Exact circular-polarization null products are checked; normalization
and mutual products of the supplied numerical wavefunctions are checked with
the stated numerical tolerance rather than silently replaced by ideal values.

The README and provenance correctly leave parametric generation and numerical
integration incomplete, and make no browser-performance or gauge-invariant
amplitude claim. Review identified one documentation omission: the plan also
requires an explicit gauge label. The native model selects
`g_propFeynman = -i Metric / P^2`; the example owner is adding that Feynman-gauge
label. This does not change the fixture's expressions or runtime settings.
Fresh-process full CLI generation remains unaccepted while the ongoing
contraction/generation attempt has no terminal result. No new runtime was
launched for this follow-up review.
