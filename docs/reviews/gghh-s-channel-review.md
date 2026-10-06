# Independent ggHH s-channel diagram review (2026-10-07)

The supplied `D05` diagram is physically different from the previous `FK018`
fixture. The old selector required one gluon and one Higgs on each four-edge
box. The supplied graph places both incoming gluons on one box and both outgoing
Higgs bosons on the other. Their native model-resolved colored topology keys are
unequal. Previous FK018 generation, convergence and reference-feasibility
measurements are historical evidence for that crossed placement, not measurements
of D05.

## Authoritative labeled connectivity and routing

The supplied directed top cycle is
`v0 → v3 → v4 → v1 → v2 → v5 → v0`. The internal gluon joins `v4` and `v5`.
Native Linnet circuit enumeration returns circuits with 4, 4 and 6 edges:

| Circuit edges | External indices | External particles |
|---|---|---|
| 4, 5, 9, 10 | 0, 1 | incoming g, g |
| 6, 7, 8, 10 | 2, 3 | outgoing H, H |
| 4, 5, 6, 7, 8, 9 | 0, 1, 2, 3 | g, g, H, H |

External edge 0 enters `v3`; edge 1 enters `v0`; edge 2 leaves `v2`; edge 3
leaves `v1`. The exact supplied native routing uses loop edges `[4,7]`, tree
edges `[5,6,8,9,10]` and dependent external index 3. With
`p3 = p0 + p1 - p2`, it is:

| Internal edge | Source → target | Particle | Routed momentum |
|---|---|---|---|
| 4 | v0 → v3 | t | k0 |
| 5 | v5 → v0 | t | k0 − p1 |
| 6 | v1 → v2 | t | k1 − p3 |
| 7 | v4 → v1 | t | k1 |
| 8 | v2 → v5 | t | k1 − p0 − p1 |
| 9 | v3 → v4 | t | k0 + p0 |
| 10 | v4 → v5 | g | k0 − k1 + p0 |

There are six massive top propagators and one massless gluon propagator. A change
of arbitrary graph labels or loop basis alone would not select a different
physical graph, but preserving the supplied object also preserves these useful
routing and tensor-index conventions exactly.

## Ecosystem evidence and model identity

The pinned FeynKit `FeynmanDiagram::from_dot` strictly imports the original
source using the original Standard Model. Its model fingerprint is
`834ba7b318f45413c0e10d1c4462c3c2598b92999c23dc366cafbd82a45a87d5`,
which matches the pinned GammaLoop Standard Model asset exactly. FeynKit's public
`Model::standard_model()` owns that embedded definition; its own tests compare it
with the corresponding model fixture and GammaLoop asset. This is preferable
to weakening model fingerprint validation.

The existing physical card produces fingerprint
`1408ea162eab3d6c6d4bd68bc0f01c97525464a8cf3ad93248d03ddf5b17e390`.
The source diagram's native ID is `06593c9910499e9c1a33f9f2ba3a1c5b`; after
explicit rebinding to the existing physical card its ID is
`a4655aa3f84bd42b000eaa16b9c54104`. The identity change records the model
snapshot; it does not change the channel. The original supplied DOT must remain
verbatim. Any rebound export must separately record both model identities.

Public `FeynmanDiagram::canonical_key` delegates colored graph canonicalization
to Symbolica. It retains model-resolved interaction IDs, particles, normalized
particle/antiparticle orientation, directedness, slots and external
index/state/connection. Vertex/diagram names, numerator, weight and routing are
excluded. Existing owner tests cover insertion-order invariance, changed
external-connection sensitivity, interaction sensitivity and equivalent reversed
fermion-edge representation. No FastSecDec DOT parser or graph canonicalizer is
needed. Native `Linnet::DotGraph::from_string`/`write_io` provide any necessary
bounded transport adaptation; `FeynmanDiagram::from_dot` then validates the
model-resolved graph again.

The implemented adapter is deliberately narrower than general model rebinding.
It strictly imports the source against `Model::standard_model()`, clones that
model and applies the existing native `ParameterCard`. The supplied model file
must have either the original fingerprint or this exact physical-card
fingerprint. Linnet changes only the exported `model_fingerprint` attribute;
strict native reimport must then succeed. Complete native diagram JSON,
excluding only the embedded model and derived diagram ID, must compare equal.
Consequently all particle/interaction IDs, local numerator fragments, slots,
half-edge order, global factors and routing survive the adapter. The adapter
does not authorize arbitrary model substitution based on topology alone.

The independent focused Rust probe used only these native graph APIs. It
confirmed the supplied circuit partition and routing, the exact original model
fingerprint, and that the old fixture fails the native colored-key comparison.
Its temporary source and logs are retained only in ignored output.

The source/API audit used the pinned GammaLoop revision `259df87`:
`feynkit-graph::FeynmanDiagram::{canonical_key,from_dot,validate}`,
`feynkit-model::Model::standard_model` and native Linnet `DotGraph`. No missing
algebra or graph operation was replaced. The only missing public operation was
rebinding an already validated diagram to a numerical parameter-card snapshot;
the bounded native-DOT transport above handles that case explicitly.

## Numerator and normalization acceptance

A topology key alone does not certify the numerator. The supplied source has
unit numerator prefactor and unit amputated projector. Its authoritative
overall factor contains `AutG(1)^(-1)`, external fermion ordering sign `+1` and
internal fermion-loop sign `−1`; native `evaluate_overall_factor` yields `−1`.
FeynKit's native import checks the aggregate numerator against the product of
its stored vertex/edge fragments and validates routing. The source includes
four `GC_11` and two `GC_94` factors and the six top plus one gluon propagator
numerators. These must remain unchanged during the model rebinding.

The example must retain its existing native color contraction and external
helicity conventions: unnormalized `delta_ab`, no additional spin/color average,
incoming (+,+) wavefunctions from HEPKit's GammaLoop convention, four-dimensional
external states and D-dimensional internal algebra. The loop sign and diagram
weight are included once; the diagnostic symmetry factor is not applied again.
The source's numerator, prefactor, projector, overall factor, local tensor
fragments and supplied routing require explicit equality checks across the
rebinding/export round trip, separately from generated-diagram topology
membership. The old projected numerator cannot be retained after changing the
channel.

## Final independent acceptance

The regenerated fixture preserves the attachment byte-for-byte as
`examples/gghh_double_box/source-diagram.dot`. Its BLAKE3 is
`2f6edc1b54f3eba2f228f7a62b8e9e0052c207f8ec804e877f5e8cba8479f0f2`.
The native generation produced 192 diagrams; four pass the s-channel circuit
selection, at indices `[25,29,42,74]`. The supplied diagram has exactly one
native colored-key match, `FK042` at index 42, whose physical diagram ID is
`a4655aa3f84bd42b000eaa16b9c54104`. The exporter retains the supplied `D05`
object itself; the generated index is evidence of membership rather than a
selection shortcut.

The independent native probe passed against the regenerated files and checked:

- Strict source import against the unmodified embedded Standard Model.
- Complete native JSON equality between the supplied source and physical raw
  export, excluding only the embedded model and derived diagram ID.
- Native colored topology key, raw numerator and every routing signature.
- A fresh Idenso SU(3) color contraction against the exported projected
  numerator, with reduction status `Complete`.
- Rebuilt four-dimensional `eps1`/`eps2` projector from the native incoming
  gluon ports, native overall factor, unit numerator prefactor and unchanged
  momentum routing after projection.

The physical model JSON equals the previous fixture's numerical model exactly.
The runtime point values and symbol declarations are unchanged. The new
projected numerator has scalar prefactor `16*i/3`; this follows from native
color reduction of D05 and is not inherited from the old graph. The native
overall fermion-loop factor remains `-1`, applied separately once by the
existing input convention.

The source, raw DOT and projected DOT copied into the example are byte-identical
to the independently probed generated inputs. Probe code and logs are confined
to ignored `output/gghh-channel-review/`; no permanent test or other example was
added. No graph, algebra, numerator-normalization or ecosystem-reuse blocker
remains for the corrected input. This acceptance establishes the specified
diagram and its projection, not a gauge-invariant sum or an independently
benchmarked two-loop numerical integral.

## Coordinator generation and runtime verification

The ordinary release CLI generated the corrected symbolic input with eight
workers in 133.138 seconds. It produced 30 six-dimensional kernels with
15 runtime scalar inputs and real/imaginary components at orders `[-1,0]`.
All evaluator statistics identify `symjit_o2`. The human JSON is 14,349 bytes
and its sibling data file is 3,628,938 bytes; neither contains absolute machine
paths. A fresh-process inspection loaded the pair in 0.643 seconds.

A separate integration process bound the unchanged `point.toml` and completed
122,880 evaluations: 1,024 points, four shifts and 30 sectors on eight workers,
with seed 20261005. Its active duration was 27.885 seconds, with 16,320 precision
rescues up to 256 bits and zero failures. The complete Laurent covariance was
retained. Imaginary coefficients were `-11.5440353505 ± 0.1229682135` at
`eps^-1` and `356.3642868169 ± 3.8718653013` at `eps^0`; both real components
were zero. Stopping was `WorkLimit`, with convergence false. This is a bounded
runtime smoke check, not a precision target, independent amplitude reference or
performance-parity measurement.

The example's release exporter, strict example Clippy and root formatting pass.
Whitespace checks pass apart from the verbatim source fixture's inherited
trailing blank line, retained to preserve the attachment byte-for-byte.
No production library behavior, dependency lockfile,
existing test or other example changed in this correction. The host still has
no `nix-shell`; these checks used its installed Rust 1.99 toolchain. Actual
Pyodide execution and the previously deferred scientific gates remain outside
this correction. Raw receipts stay in ignored `output/gghh-s-channel/`.
