# Runtime model parameters: independent native API review (2026-10-07)

## Native dependency owner

The pinned HEPKit model API already provides the required analytic dependency
resolution. `Model::scalar_bindings(card, overrides)` resolves parameter and
coupling definitions without changing the model. Internal expression-backed
parameters retain their analytic definitions instead of their cached numerical
values. Explicit restriction-card entries and exact overrides have their
documented precedence. `resolve_scalar_bindings` closes dependencies among the
map's keys using native Symbolica literal replacements and rejects cycles.
Expressions may retain symbols outside that map, which is the required runtime
parameter boundary. A runtime alias must differ from its original UFO key;
self-aliases are correctly rejected as unresolved dependencies.

The source and owner tests were inspected in pinned GammaLoop `259df87`, under
`crates/feynkit-model/src/scalar_bindings.rs` and its tests. Existing tests cover
analytic chains, deliberately stale cached values, card precedence, exact
complex binary64 transport, literal aliases and cyclic definitions. The public
`Model::expand_couplings` alone is insufficient for complete parameter closure:
it expands named couplings but does not replace the full dependency resolver.
No FastSecDec dependency walker or alternative expression evaluator is needed.

## Focused native probe

A temporary Rust probe supplied independent runtime aliases for `aS`, `ymt`
and `MT` to the native Standard Model resolver. At two different parameter
points, substitution into the resulting analytic `GC_11`, `GC_94` and mass
expressions exactly equaled fresh native card resolution. This verifies both
the coupling dependence and the original complex phase, rather than only
comparing absolute numerical magnitudes.

The same probe supplied a complex scalar coupling leaf as `lam_re + i*lam_im`.
The native model retained both components through its `-i*lam` coupling. A real
evaluator-input vector can therefore carry complex model parameters through
explicit real/imaginary components while preserving phases in native Atoms.
The probe did not introduce a new numerical or symbolic arithmetic layer.

## Mass, support and width boundaries

Native `Particle::symbolic_mass` returns the named UFO mass independently of
its cached numerical value. Only the structural parameter name `ZERO` becomes
the literal zero. The native `FeynmanDiagram::propagator_family` therefore
already preserves named mass dependence before U/F construction. The probe
confirmed that a top model with numerical `MT=0` still yields `UFO::MT`, while
the gluon remains exactly massless.

For the existing scalar model, whose default `mt` is zero, the native bubble F
polynomial still contains `mt²(x0+x1)²` before scalar specialization. Promoting
the named mass before family specialization preserves those support monomials.
Promoting it after substituting the default zero would irreversibly erase them.

Conversely, setting a generic runtime mass to zero can change endpoint
singularities and the Laurent expansion. The accepted scope rejects such
zero-mass degenerations and requires a newly generated, explicitly specialized
input. This is a structural validity boundary, separate from the caller's
responsibility for thresholds. The structural `ZERO` parameter remains a fixed
massless declaration.

Native propagator families deliberately do not infer widths or custom UFO
denominator formulas. FastSecDec must therefore continue to reject nonzero or
unresolved internal widths before construction. Widths are explicit zero
constraints, not unsupported runtime controls that are silently discarded.
Masses must remain real and finite at the bound point. Existing numerical-only
mass admission cannot simply accept symbolic aliases without a declared
runtime schema and subsequent point validation.

## Persistence implementation evidence

The review author also implemented the mass-constraint transport; this section
records its focused implementation probe rather than claiming independent
review of that codec. A separate reviewer audits the persistence/binding slice.

Binary version 6 stores each constraint's name and native Atom expression.
Atoms use Symbolica's context-aware binserde and contribute their symbols to the
exported state. The canonical constraint expressions and names enter the
template semantic identity and sector representation identity. The exact
version-5 decoder remains available for the previously validated baseline and
introduces no constraints into old templates. No version guesses omitted
fields. Human JSON lists the mass names and finite-real-nonzero requirement;
model defaults are separately marked metadata and are never implicit bindings.

The focused probe passed a cold version-6 load after registering 100 unrelated
symbols before the runtime mass symbol. The restored constraint matched its
native Atom, template identity and original bytes. Binding a valid mass then
attempting zero failed before changing the numerical identity or worker output.
A worker clone retained the valid bound input. Unknown constraint inputs and a
complex-valued mass were rejected. Adding a constraint changed both template and
sector identities. The complete version-5 D05 baseline also cold-loaded with
its exact original identity and bytes and an empty constraint list.

## Independent frontend acceptance

The implemented `RuntimeModelBindings` promotes independent model leaves in
stable name order: external parameters and expressionless internal constants.
The review identified the latter case, which was added before acceptance.
Structural `ZERO`, explicitly fixed overrides and internal propagator widths
remain excluded. Complex leaves use ordered real/imaginary inputs; all declared
evaluator inputs have native real-symbol attributes. Used-input filtering
inspects native expressions and retains every mass-constraint dependency.

The numerical card is used for suggested human defaults and the mandatory zero
width restrictions. Runtime definitions call the native analytic resolver
without that card, using the explicit runtime aliases and fixed overrides.
This distinction was verified by deliberately inserting a cached/internal
`yt=123` card entry: runtime analytic expressions were unchanged. Changing the
card's external `aS` changed the suggested default only. Binding two explicit
runtime points still exactly matched independent native card closure for
`GC_11`, `GC_94` and `MT`.

The public runtime graph constructor validates declared real scalar inputs
before native family construction. Independent probes using both external and
expressionless internal named masses with numerical default zero confirmed
that `model::mt` survives in F. The structural ZERO stays literal zero. A
nonzero top width was rejected before construction; an explicit fixed `MT=0`
override instead produced the requested specialized input without a generic
mass constraint. The public API documents that its caller must attach the
matching constraints when compiling, while the CLI does so automatically.

The caller-owned public sequence is explicit: construct
`RuntimeModelBindings::new(diagram, card, fixed)`, pass its `values()` and
`symbols()` into `GraphIntegral::new_with_runtime_scalar_values`, and prepare
the native `ParametricIntegrand`. Then call `retain_used(&integrand)`, compile
with the ordered kinematic inputs followed by the retained model `symbols()`,
and attach `with_runtime_mass_constraints(runtime.mass_constraints().to_vec())`
before `bind_parameters(point)`. This uses the same core owners for native and
portable eager execution; the CLI owns scheduling and presentation.

A further native tadpole probe changed its independent scalar coupling leaf to
complex. The actual `RuntimeModelBindings` implementation emitted adjacent
`model::lam_re`, `model::lam_im` inputs in that order. At two complex points its
complete coupling exactly matched separate native card resolution, including
both phase components.

This frontend review found no remaining native dependency, phase, default-zero
mass, or width-admission blocker. Probe source and output are confined to
ignored `output/runtime-model-review/`.

## Final D05 evaluator comparison

The final version-6 D05 template cold-loaded with 30 sectors and 21 ordered real
inputs: the existing 15 kinematic products plus `model::Gf`, `model::MT`,
`model::MZ`, `model::aEWM1`, `model::aS` and `model::ymt`. Its native runtime mass
constraint is `MT`. The template kernel identity is
`66f2af45b499f63bb15feb7148d0941eabb654ec2e83227443c023756208f09d`.

The independent evaluator probe first required exact agreement between old and
new native chart maps, representative assignments and coordinate permutations.
It then compared the complete output vector at four points per sector,
including coordinate values `1e-4` and `3e-5`, and included the full exact
offset: 121 complete vectors in total. At the supplied default point, the new
runtime-model template agreed with the preserved version-5 fixed-model D05
baseline to a maximum relative infinity-norm difference of
`1.7805820705135412e-14`.

Doubling `model::ymt` or, separately, `model::aS` produced exactly four times
every sampled vector at the returned floating-point precision. Changing `MT`
by 10% changed the evaluator outputs. Binding `MT=0` failed before mutating the
valid bound identity. Physical-point identities changed as expected, while the
serialized template bytes and template identity remained unchanged throughout.

The probe source and result are ignored
`output/runtime-model-review/d05_runtime_compare.rs` and `d05-comparison.txt`.
This is an end-to-end model-parameter regression comparison, including complex
phases and native precision rescue near endpoints; it is not a new independent
two-loop reference or a convergence claim. The coordinator records its
separate bounded integration smoke and parallel-generation control evidence.
