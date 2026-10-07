# Independent review: native gg → HH triple-box input

Scope: the new `examples/gghh_triple_box_bis/` input, its graph topology,
serialization, model point and external projection. This review independently
executed the current CLI/core's native Rust import path and native graph/color
checks. No FastSecDec parameterization, sector generation, evaluator compilation
or integration was run. The existing `examples/gghh_triple_box/` was not edited.

## Native selection and topology

The author enumerated 2,130 native topologies and retained 162 diagrams using
three loops, QCD order 6, QED order 2, exactly one fermion loop, only native
`ttg`/`ttH` interactions, and initial/final symmetrization. This is a targeted
ladder selection, not a catalogue of the complete Standard-Model amplitude.
The unique requested candidate is D068, source ID
`d77f7e0c894002655d2bb58ee0846318`. Applying the physical model card changes the
model fingerprint and derived diagram ID to
`cced04ecc95ba7488574be8dc7bb689d`; the native graph payload and canonical topology
are preserved. The exporter compares the entire native JSON payload after
removing only model identity and diagram ID.

The independent Rust probe calls `FeynmanDiagram::validate`,
`LoopMomentumBasis::validate`, Linnet `is_connected`, `cyclotomatic_number` and
`all_cycles_of(internal, 3)`. The latter enumerates every circuit from the native
Paton basis with an explicit rank limit; no custom graph traversal, planarity
implementation or canonicalizer was added.

Observed native invariants:

- Eight vertices, ten internal edges, four external legs, connected graph, loop
  rank three.
- Top edges `{4,5,6,7,8,9,10,13}` form the unique eight-edge circuit. Gluon rungs
  are `{11,12}`. Thus the fermion propagators form one closed outer loop.
- The complete circuit-length multiset is `[4,4,4,6,6,8]`.
- Three four-edge circuits form the ladder faces:

| Face | Native edge IDs | External particles |
| --- | --- | --- |
| Gluon end | `4,5,9,11` | incoming `g,g` |
| Middle | `10,11,12,13` | none |
| Higgs end | `6,7,8,12` | outgoing `H,H` |

Each top edge occurs in one of these face boundaries; each gluon occurs in two.
The end faces are disjoint and each shares exactly its gluon rung with the middle
face. This gives three adjacent quadrilateral faces bounded by the top circuit,
with uncrossed rungs. The external attachments occupy four distinct vertices:
incoming indices 0 and 1 at vertices 3 and 0, outgoing indices 2 and 3 at vertices
2 and 1. The adjacent pairs belong to opposite end faces, as requested.

## Strict import, routing and runtime model inputs

The enumeration host uses newer HEPKit Python bindings, while the current CLI
pins GammaLoop/HEPKit `259df879`. A focused Rust executable linked to the existing
release libraries verifies the actual older consumer, independently of the
Python export host:

1. Load final `model.json` using `Model::from_json`, load and apply
   `parameters.json`, and verify the model fingerprint is unchanged.
2. Strictly import `graph.dot` through `FeynmanDiagram::from_dot`; validate native
   edge roles, interaction/port data, and the full stored momentum basis.
3. Export and reimport native DOT, requiring exact equality of native JSON.
4. Construct `RuntimeModelBindings` and the native `GraphIntegral` quadratic
   family, without numerator contraction. The family has three loop momenta,
   ten denominators, and ten unit propagator powers.

A separate raw-source check also passes. The scratch enumeration model is kept
with its original defaults; the final model is already at the physical card so
that the CLI's apply-card-before-import sequence remains valid.

The complete uncontracted weighted tensor is substituted through native
`Model::scalar_bindings`/`RuntimeModelBindings` using Symbolica's simultaneous
literal replacement. Its contributing independent model symbols, including the
mass constraint, are exactly `model::Gf`, `model::MT`, `model::MZ`,
`model::aEWM1`, `model::aS`, and `model::ymt`. The full model owner initially
contains 27 candidate leaves; those unused inputs are not exported in the point.
Analytic dependent couplings remain native expressions. The runtime MT mass
constraint is retained; the top mass must be finite, real and nonzero. Exact
zero widths are structural restrictions.

The run card declares fifteen momentum/polarization products as runtime symbols.
Together with the six model leaves they have matching numerical entries in
`point.toml`. Those values are supplied at integration time, not inserted into
the exported tensor. The native point helper checks momentum routing,
conservation and polarization Gram products at √s = 300, mH = 125,
MT = ymt = 172.5 and cosθ = 4/5. External wavefunctions are four-dimensional;
internal contraction is requested at D = 4 − 2ε. No threshold-safety certificate
or contour prescription is inferred from these input checks.

## Projection and single-copy tensor storage

The independent probe reconstructs the incoming external half-edge labels using
native graph ports. Native Idenso closes one unnormalized adjoint `delta_ab`,
with its standard SU(3), T_F = 1/2 convention. Recomputing color-only reduction
with the existing shared helper agrees with the exported tensor. Its numerical
coefficient changes from −1 to −64/9, giving the expected positive color
multiplier 64/9; the original numerator's sign is preserved. This check performs
no Dirac or Lorentz contraction.

The native overall bookkeeping factor evaluates to −1, from the single closed
fermion loop; it remains separate and is included exactly once. There is no
extra symmetry factor, spin/color average or helicity sum. The two (+,+)
Lorentz polarization vectors are reconstructed from the native external ports,
and equal the saved projector.

To avoid duplicating a large tensor at vertex v0, the finalized native graph has
unit aggregate, vertex and edge numerators. Its global `numerator_prefactor`
contains the color-projected tensor multiplied by the original raw prefactor.
This satisfies HEPKit's aggregate/local-fragment invariant. The current native
FastSecDec input owner contracts the complete product
`numerator * numerator_prefactor * projector * overall_factor`; it does not
assume that the named prefactor is a scalar already. The independent probe
requires exact native equality of that complete product with the independently
reconstructed projected raw input, and separately checks unchanged routing and
unit local fragments. Thus the storage change loses no factor and introduces no
double counting.

The native Linnet DOT AST handles parsing and escaping. Its formatter places
graph attributes on separate lines; there is no second DOT parser or altered
mathematical representation. Source and raw artifacts retain their respective
native provenance. Root's independent JSON/TOML/path/hash checks found only
relative references and confirmed the original triple-box folder is unchanged.

## Evidence and limits

Ignored evidence is in `output/gghh-triple-box-bis/`: `enumeration.log`,
`generation-evidence.json`, the native selected render, `native_import_review.rs`,
`native-import-review.txt`, `native-final-review.txt`, and the author's
`export.rs`/`export.log`. The final independent probe completed successfully on
the actual saved assets in under one second after linking.

This accepts a native-compatible input for the user's own FastSecDec run. It does
not establish generation cost, sector counts, integration convergence or a
physical amplitude value. The full eight-propagator Dirac contraction and all
subsequent FastSecDec stages deliberately remain unrun. Existing native one-loop
master and reduction providers are unchanged and supply no new three-loop
reference result here. No numerical or algebraic implementation was added for
this example.
