# ggHH on-shell generation review

This independent review concerns the incoming gluon relations `P(0)^2 = 0`
and `P(1)^2 = 0` in the bundled double-box and planar triple-box-bis inputs.
They are generation-time identities of the chosen process, while the remaining
declared kinematic quantities and contributing model inputs remain runtime
parameters. This review does not run three-loop sector generation or integration.
The separately untracked triple-box example is not used as an acceptance input.

## Native ownership and ordering

The existing run-card schema already expresses the required distinction:
`kinematics.products` accepts exactly one of `value` or `symbol`. In
`crates/fastsecdec-cli/src/input.rs`, the value branch parses and resolves a native
Atom, whereas the symbol branch adds an evaluator input. Both feed native
`Kinematics::with_scalar_product` before `GraphIntegral` construction. Thus
`value = "0"` is an exact relation, not a zero-valued runtime default.

In pinned HEPKit `259df879`, `feynkit-kinematics/src/symbolic.rs` implements
`with_mass_squared(p, m2)` directly through `with_scalar_product(p, p, m2)`.
The latter stores exact local dot-product and metric substitutions. Native
`IntegralFamily::new` applies them to propagator denominators before retaining
the family (`feynkit-graph/src/integrals.rs`). Its `symanzik` method also evaluates
external products in the quadratic-form shifts through that same kinematics
owner (`integrals/parametric.rs`). Symbolica supplies exact determinants,
cofactors and coefficient simplification. No new graph reduction, scalar-product
engine or Symanzik implementation is needed.

FastSecDec's `GraphIntegral::new_with_runtime_scalar_values` constructs this
native family. Both scalar and tensor-numerator parametrization call its native
`symanzik` method before constructing `ParametricIntegrand`. Source-moment
families retain the same kinematics. `generation/mod.rs` then extracts singular
factor supports and builds geometry **before** selecting the symbolic or
numerical-dual continuation. `parametric::polynomial_support` uses the native
exact `AtomField` with `statistical_zero_test = false`. Consequently the fixed
relations reach native F construction, exact support, geometry and both lanes;
there is no separate dual-lane on-shell patch. U normally depends only on the
loop quadratic form; the relevant kinematic support change is in F.

## Why a runtime zero is insufficient

Sector maps and analytic endpoint subtraction are prepared from the exact
generation-time polynomial support. For example, a generic factor `a*x + y`
becomes `x*(a + t)` in one chart. Specializing `a = 0` later creates an additional
endpoint zero in the supposed regular factor. Binding evaluator inputs does not
recompute that chart's valuations or subtraction formula. Agreement of two
generic evaluators at interior points cannot establish the endpoint geometry or
Laurent expansion after such a specialization.

Earlier ggHH benchmark artifacts declared `p0p0` and `p1p1` as generic runtime
inputs and subsequently bound them to zero. Their recorded interior-point parity
and evaluator timings remain historical measurements of those generated
objects. They must not be used as validation of the physical on-shell endpoint
geometry or Laurent poles. Corrected on-shell inputs require fresh generation;
this review does not silently reinterpret or modify previous artifacts.

The correction must apply to the exporter's in-memory kinematics as well as its
written run card, before it parametrizes the graph to retain used model leaves.
The two removed runtime inputs must also be absent from `point.toml`. Other
products must not be frozen merely because a particular physical point happens
to make them zero. No threshold certificate or numerical zero test is introduced.

## Notebook and generator audit

The Rust double-box exporter previously made every one of its fifteen Gram
products a runtime symbol, including the two exact gluon virtualities. Its
physical `Point` owner had already checked on-shell and conservation identities;
the information was lost when serializing the runtime schema. This is the same
issue as the two bundled run-card declarations.

The current notebook helper follows a different path.
`examples/hepkit/showcase/gghh.py::_external_data(raw, None)` constructs generic
symbolic incoming vectors `(e,0,0,+e)` and `(e,0,0,-e)`. Native tensor contraction
in `prepare` retains exactly zero Gram products and creates runtime symbols only
for nonzero expressions. The gluon self-products therefore enter native
kinematics as zero before the binding's Integral owner is constructed. Generic
energy, Higgs mass and scattering angle are used for that structural decision;
the subsequently selected numerical point is not used to decide which products
vanish. Polarization normalization remains a runtime input as intended.
The generated standalone notebook contains the same helper. An explicit
regression asserting these fixed gluon products and their absence from the
runtime schema would strengthen the existing generic-point/schema controls;
no notebook production correction was identified by this source audit.

## Implementation and execution status

The final example/exporter source changes pass independent review. The exporter
checks that its native `P(0)`/`P(1)` coordinates are incoming gluon ports and that
their physical self-products are exactly zero. It uses `Atom::Zero` in the
in-memory Kinematics and emits `value = "0"` in the run card, while excluding
those entries from the integration point. The other thirteen Gram quantities
remain runtime symbols. Independent TOML parsing confirms two exact self-product
conditions, thirteen runtime Gram declarations and six model entries in each
tracked example's point card. These are input-schema counts, not a claim that a
new three-loop evaluator has been compiled.

An independent, bounded installed-native Python check called the notebook's
actual `_tensor` and `_dot` helpers on both symbolic incoming vectors and
confirmed exact zero before runtime symbol selection. It performs no graph
generation, numerator contraction or integration.

The new CLI regression was also source-reviewed and passed in the owner's
coordinated workspace-feature test run: it admits a native massless
triangle through the actual run-card loader, verifies that fixed on-shell values
reduce F support from three monomials to one, and that binding zero only in the
integration settings leaves generic support intact. Both generation modes must
then expose `[-2,-1,0]` on shell, compared with `[0]` off shell, and retain only
the remaining invariant as an eager evaluator input. A separate fresh-process
native graph import checks both bundled D05/D068 momentum labels and the actual
run/point cards, including parameter-card application. Both controls passed
(two tests, 0.05 seconds); the source and final log were independently reviewed
at `crates/fastsecdec-cli/src/input/on_shell_tests.rs` and
`output/on-shell-generation/input-tests.log`. No build, sector generation or
integration was run by this independent reviewer.

The example owner's separate native Python/TOML proof passed strict imports of
the actual double-box, triple-box-bis and local untracked triple-box graphs. It
confirmed that P0/P1 label incoming PDG-21 ports, and compared each tracked
TOML document with HEAD after only the two intended replacements/removals. Other
scientific values are unchanged. Its actual raw-diagram notebook Gram checks
also passed. The source and log were independently read at
`output/gghh-on-shell-review/cards_native_check.py` and
`output/gghh-on-shell-review/cards-native-check.txt`. The untracked graph remains
a separate input; this import check is not three-loop parametrization or a
generated numerical result.

The strengthened import regression exposed an existing native-owner limitation:
after an unrelated scalar-triangle generation in the same test process, the
first D05 strict import rejected a model fingerprint of `da16…` where the saved
graph expects `1408…`. Both D05 and D068 passed in a fresh process using the same
model/card/graph bytes. Removing point-name parsing did not remove the warm-state
failure, so no particular symbol attribute or parser operation is identified as
its cause here. This establishes process-state/order sensitivity, not a failure
of the new on-shell relation. The import regression uses the existing cold-child
test pattern to exercise ordinary CLI startup. Native fingerprint validation,
model data and graph data remain unchanged; no guard was bypassed or hash
rewritten. The failing warm-state evidence is retained at
`output/on-shell-generation/native-import-warm-state.log`. Fixing that native
owner limitation is outside this input correction.

## Corrected double-box generation acceptance

The coordinator completed an actual corrected D05 run through the current
release CLI with eight caller-owned workers, numerical-dual Taylor subtraction
and SymJIT O2. Independent inspection of the saved artifact confirms 30 charts
and kernels, all using numerical-dual generation; four completed formulas for
30 uses with 26 reuses; and a 19-input evaluator schema containing the thirteen
remaining Gram quantities and six model leaves. Neither `p0p0` nor `p1p1` remains
in that schema. The coordinator also verified exact canonical key agreement
with the revised integration point card.

Saved generation time was 20.055521375 seconds, including 0.195836750 seconds
for formula preparation. The output layout is Real/Imag for epsilon orders -1
and 0 (`[-1,-1,0,0]`). An unchanged sector count does not contradict the fix:
the exact on-shell input now reaches the support analysis, and the separate
massless-triangle regression demonstrates a case where it changes the endpoint
pole structure. No particular sector-count increase is an acceptance condition.

The immediately preceding generic-virtuality numerical-dual Taylor run took
23.302 seconds with the same eight-worker/SymJIT O2 settings. The corrected run
was about 14% faster in these single measurements, a modest observed change
rather than a controlled attribution benchmark. Its largest phases remain
native evaluator construction/compilation (10.664 seconds) and map/valuation
discovery (7.620 seconds). No fresh on-shell evaluator throughput measurement
is inferred from these generation timings.

Evidence is retained under ignored `output/on-shell-generation/`, including
the copied run card, status log, final JSON and `.fsd` artifact pair. The final
workspace all-target strict Clippy and formatting checks passed. The complete
CLI binary gate also passes all 53 tests (one explicitly ignored external-artifact
benchmark), including both new on-shell regressions. These results are recorded
in `output/on-shell-generation/cli-tests.log`. This accepts
the corrected input path and successful D05 evaluator generation; no newly
measured on-shell integral, symbolic/dual pointwise parity, three-loop generation
or physical amplitude value is claimed.
