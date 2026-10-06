# Native DOT presentation and runtime-mass review (2026-10-07)

This reviewer implemented the example's DOT presentation adapter and separately
reviewed the runtime model-input, binding and binary-persistence work written by
the other agents. The DOT checks below are implementation evidence; the runtime
section is an independent source and executable review.

## Native DOT presentation

The ggHH `graph.dot` now writes one graph attribute per line. The example helper
`crates/fastsecdec/examples/gghh_double_box/dot.rs` uses native
`FeynmanDiagram::to_dot`, Linnet `DotGraph::from_string`, `GlobalData` formatting
and `DotGraph::write_io`. Linnet owns attribute parsing, escaping, topology,
orientation and identifier serialization. The adapter only encloses the known
global formatter output in a `graph [...]` block; it does not split expressions
on commas or implement another DOT parser. Before returning the text it imports
it through `FeynmanDiagram::from_dot` and compares the complete native JSON
payload exactly.

The resolved GammaLoop `259df87` owner sources and tests were inspected first:
FeynKit's stable DOT importer/exporter and numerator-fragment validation in
`crates/feynkit-graph/src/lib.rs`, its compact dialect in `compact_dot.rs`, and
Linnet's parser/global/vertex/edge serializers and escaping tests. A focused
Rust probe confirmed exact full-payload equality after formatting, including
model identity, local tensor fragments, edge slots, half-edge order, momentum
routing, weights and projector. The example's strict Clippy target passed.

The large numerator remains present globally and at `v0`. This is required by
the current native stable dialect: `with_numerator` places a replacement at
the first interaction vertex, sets other local fragments to one, and retains
the aggregate numerator. Native validation requires their product to equal
that aggregate. A focused probe removed the `v0` numerator using the native
DOT object; reimport failed with **NumeratorFragmentMismatch**. An absent local
fragment defaults to one, not a reference to the global expression.

The compact dialect does not provide a lossless escape from this requirement:
its graph `num` denotes the separate numerator prefactor, and it reconstructs
interaction slots and routing rather than transporting the finalized stable
payload. Moving the expression into that field would change stored semantics.
No lossy omission, reference syntax or alternate importer was introduced.
`source-diagram.dot` and the raw source assets remain unchanged. The exporter
uses this helper only for its projected `graph.dot`; raw export remains native.
Probe evidence is under ignored `output/generation-followup/dot-probe/`.

## Runtime model and native ownership

`RuntimeModelBindings` uses HEPKit `Model::scalar_bindings` for default-point
resolution and analytic dependency closure. It promotes independent scalar leaves
(external parameters or expressionless internal parameters, matching the native
evaluation-request boundary) before propagator-family specialization,
preserving named-mass support even
when the model's cached numerical default is zero. The native structural `ZERO`
and explicit fixed overrides remain specializations. Widths stay fixed to zero;
the existing native quadratic family does not implement width-dependent
propagators. No new dependency walker, propagator builder, CAS or special-function
implementation was added.

Real independent parameters produce declared-real evaluator symbols; complex
leaves produce separate real/imaginary inputs whose native expression retains
the complex phase. Parameter layout follows sorted model names, with real then
imaginary components. Final input pruning retains both integral dependencies
and every runtime mass-constraint dependency. Cached defaults are human
metadata and are not silently bound to evaluators.

The CLI compiles the complete declared schema and then attaches the returned
runtime mass constraints before constructing the artifact. The low-level
`GraphIntegral::new_with_runtime_scalar_values` API is an explicit composition
boundary: its documentation requires callers to attach the matching
`RuntimeModelBindings::mass_constraints()` to the compiled `KernelSet`. The
intermediate graph/parametric types do not carry this obligation automatically,
so independently composed library callers must follow it. That limitation is
explicit; the ordinary CLI path always performs the attachment.

## Binding and persistence audit

Mass constraints keep typed names and native Atom expressions. Shared schema
validation rejects empty/duplicate names and expressions the native evaluator
cannot express in the declared runtime inputs. Review found that a constant
constraint could previously be attached to a kernel with no runtime schema,
whose `parameters_bound()` is already true. The author fixed that bypass by
rejecting nonempty mass constraints with an empty runtime schema in the shared
attachment/load validator. The independent probe confirms that rejection.

Binding first validates complete finite input values and evaluates every mass
through native 128-bit arithmetic. Masses must be finite, real and nonzero.
The zero-mass rejection protects the generic endpoint expansion, independently
of the user's threshold responsibility; a massless specialization must be
generated explicitly. Negative real nonzero masses are permitted by this
stated contract. Failed mass checks precede changes to sector buffers, folded
exact offsets and numerical identity.

Sector coordinates precede the ordered runtime suffix in all evaluator inputs.
Worker cloning copies the bound suffix and keeps independently owned evaluator
state. Weighted precision replay uses that same complete input vector; mass
validation is outside the sample loop. Exact offsets use their own parameterized
native evaluator, including when there are no numerical sectors. Numerical
point identities combine the immutable template identity with all ordered f64
input bits, so replay/checkpoint state cannot cross parameter points. Saved
template bytes continue to exclude the bound values.

Binary version 6 persists each mass-constraint Atom using native context-aware
binserde. Its symbols are included in the exported Symbolica state. Names and
canonical expressions enter both the template semantic identity and sector
representation identity. No process-local symbol ID enters those identities.
The previously reviewed native-evaluator serde/bincode codec is unchanged.
Version 5 retains its exact separate layout, magic, digest and semantic identity;
it yields an empty constraint list and retains its original bytes. There is no
decoder guessing omitted fields or weakening semantic verification.

The codec author's cold-process probe was reviewed: it registers 100 unrelated
symbols before loading the constrained template, checks native constraint
remapping and byte/identity preservation, and loads the complete preceding
version-5 D05 artifact without changing its bytes or identity. Adding a
constraint changes both template and sector identities. Unknown inputs and
nonreal bound masses are rejected.

## Independent executable controls

A separate public-API probe passed on **both native and portable host backends**:

- A derived mass `m²−1` rejects its zero at `m=1`; nonfinite runtime values,
  a mass pole `1/(m−1)` and a nonzero imaginary mass are rejected. A negative
  real nonzero mass is accepted.
- A failed rebind leaves the accepted numerical identity and sector outputs
  unchanged. Loaded template bytes remain unchanged after successful binding.
- A cloned complex worker undergoes forced weighted multiprecision replay and
  reproduces both components with runtime inputs and the weight applied once.
  Replay state from another bound point is rejected.
- An exact-only integral with zero numerical sectors binds its complex offset,
  rejects zero-mass rebinding without changing that offset or identity, and
  cold-loads/rebinds to a different complex offset correctly.
- Nonempty constraints on an empty runtime schema are rejected, closing the
  review finding above.

The probe source and logs are retained only under ignored
`output/generation-followup/runtime-audit/`. Existing tests and other examples
were not edited. This is eager portable-host evidence, not a new Pyodide/Wasm
wheel or browser performance claim. Ordinary one-loop master and reduction APIs
remain the unchanged external scientific reference providers; no new numerical
integration algorithm or independent physics benchmark is claimed here.

The independently reviewed opaque dispatch invariant remains unchanged by this
round: fresh owner identity, stage and complete source-index coverage are
checked before ordered admission. Parallel completion order cannot choose a
different representative or move mass inputs between sector schemas. Focused
native/portable scheduling controls and the isolated D05 byte-identity result
are recorded in [parallel symmetry](parallel-symmetry.md); the full new runtime
model D05 generation/integration has its own acceptance evidence.
