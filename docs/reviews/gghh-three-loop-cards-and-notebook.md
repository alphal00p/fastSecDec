# Three-loop ggHH cards and notebook defaults

This independent audit covers the two distinct three-loop ggHH inputs and the
shared notebook workflow. The requested defaults are numerical-dual generation
for both three-loop cards and the notebooks; native `dots` contraction for the
original triple box and notebooks; and `minimal` contraction for triple-box-bis.
The double-box card retains its existing symbolic default. There is no global
core or binding-default change. Three-loop numerator contraction, parametrization,
sector generation and integration are outside this validation.

## Native topology and resource provenance

A bounded independent probe strictly imported the actual saved graphs with their
native model and called `validate`, `subgraph`, `connected_components` and
`cycle_basis`. These public HEPKit bindings delegate graph selection and circuit
analysis to the native Linnet owner. No custom graph traversal, parser or
canonicalizer was introduced.

| Input | Native name | Loops | Vertices | Internal edges | Top-only circuits |
| --- | --- | ---: | ---: | ---: | --- |
| Double box | D05 | 2 | 6 | 7 | One six-edge circuit |
| Original triple box | D020 | 3 | 8 | 10 | Two separate four-edge circuits |
| Triple-box-bis | D068 | 3 | 8 | 10 | One eight-edge circuit |

D020 has native ID `d9ae720f395bceaadd28c78a55a7ecaf`. Its top circuits are
`[4,5,9,11]` and `[6,7,8,12]`, with gluons `[10,13]`. D068 has native ID
`cced04ecc95ba7488574be8dc7bb689d`; its single top circuit is
`[4,5,6,7,8,9,10,13]`, with gluons `[11,12]`. Both have incoming PDG-21 ports at
external indices 0/1 and outgoing PDG-25 ports at 2/3. Native probe evidence is
retained under ignored `output/gghh-cards-review/independent-native-topologies.json`.

Initial resource inspection found that the original triple-box `graph.dot` was
the distinct native D020 graph, while its source/raw snapshots, generation
history, provenance and README had been copied from the D05 double box. Those
files cannot document D020's construction. Replacement snapshots must be derived
from the actual supplied native owner and identify unknown original enumeration
history explicitly. They represent that supplied, already projected graph; they
do not recover an unprojected generator output. Its color-delta/polarization
projector is distinct from D068's earlier color-reduced tensor and Lorentz-only
projector. No new symmetry factor, spin/color averaging or tensor reduction is
implied by refreshing these resources.

The final repaired resources satisfy those requirements. D020's
`source-diagram.dot` preserves the supplied file byte-for-byte, with SHA-256
`5fade38408dc235a97c30640b14f48a2750070fbcffa124b81cf7a1e06bd82c7`.
Its refreshed raw DOT, JSON and expression snapshots agree with that native
owner. The copied `generation.json` and unrelated color-reduced numerator have
been removed. The new provenance explicitly records unavailable enumeration
history, and the README describes the actual two-fermion-loop topology.

The final input stores the supplied tensor once in native
`numerator_prefactor`, with unit global/local numerators. Native Atom equality
of `numerator * numerator_prefactor * projector * evaluated_overall_factor`
verifies the unchanged complete weighted product. The native supplied overall
factor evaluates to `+1`; the original projector and complete loop-momentum
basis are unchanged. This storage adjustment performs no tensor contraction.
The final D068 graph, its previously color-reduced tensor and its Lorentz-only
projector retain their separate documented convention.

The independent reviewer reran the bounded final owner checks on the frozen
files; both inputs pass strict import, model/card identity, native validation,
DOT/JSON round trips, routing, raw expression snapshots, topology and relative
resource-path checks. Evidence is in ignored
`output/gghh-cards-review/independent-final-cards.txt`; the reviewed check source
is `output/gghh-three-loop-review/final_cards_check.py`. A separate current-Rust
probe reused the existing native DOT formatter and verified exact payload and
routing round trips for both final/raw DOT files, recorded in
`output/gghh-three-loop-review/format-native.txt`.

## Exact kinematics before geometry

All three run cards must retain exact `value = "0"` declarations for incoming
gluon self-products P0/P1. They must not restore `p0p0`/`p1p1` runtime inputs.
The thirteen remaining Gram declarations stay parametric; each integration point
card supplies them together with six model leaves. Their final three-loop
compiled schema has not been measured because parameterization is out of scope.
The existing native Kinematics → IntegralFamily → Symanzik → exact support path is
reviewed in [the on-shell generation audit](gghh-on-shell-generation.md). Both
symbolic and numerical-dual lanes consume that same prior support discovery.

The notebook constructs incoming momentum vectors with generic symbolic energy
and applies native tensor dot products before selecting runtime Gram symbols.
Their exact self-products vanish before its Integral/generation session exists.
This is a generation-time identity, not an accidentally zero integration-point
value. Other nonzero symbolic products remain runtime symbols; polarization
normalization is not frozen into a large binary-rational constant.

## Notebook execution and native algebra

The ggHH preparation and shared scalar-numerator view call native
`simplify_algebra(contract="dots").to_dots()`. The reviewed
`showcase/science.py::generation` first reads the prepared input's generation
arguments, then applies explicit caller overrides for mode, subtraction and
coefficient expansion, and passes the result once to the retained native
generation session. `GGHHInput` supplies numerical-dual/Taylor defaults. Other
inputs keep their prior mode default, and compilation remains explicitly eager.
No global Rust/Python API default or worker-pool behavior changes.

The self-contained `gghh_complete.py` is synchronized with those shared helpers;
AST-based tests now check the input, generation and timing helpers as well as
the existing preparation/presentation code. Optional formula timing and counts
come from native snapshots. An absent historical field is omitted, not shown as
a measured zero. This is read-only progress presentation.

Native dot conversion and simplification remain explicit preparation operations.
Passive rich display, progress publication and inspection must not start graph
generation, contraction, compilation or integration. Generation and integration
stay separately authorized actions on retained native owners. One-loop reference
providers and reductions are unchanged; this input/default update introduces no
new numerical integration or reduction algorithm.

## Acceptance status

The final source/resource audit is accepted without an outstanding finding.
Independent native topology and final-file checks pass. The notebook owner ran
93 helper/presentation tests and four native ggHH preparation tests successfully
on the existing host, and Marimo static checks pass for all three notebook
entrypoints. Native preparation tests establish exact incoming gluon self-dot
rules from the routed external ports before session creation, omit their runtime
Gram symbols, and retain every other nonzero Gram entry as a native real input.
The option-forwarding test checks defaults, explicit overrides and nonmutation
of the prepared defaults. Logs are retained under ignored
`output/gghh-cards-review/`.

The refreshed isolated native host subsequently passed **all 102 notebook
tests in 13.45 s**, with zero failures, errors or skipped cases in the independently
read JUnit report. This includes the two previously problematic two-loop graphs
with triple-gluon vertices, native IDs
`6586fc41a2a00087ef7be79f59b61224` and
`bf45cfca79c449b3c03e49aebf39b9dc`. Each is prepared with native dots contraction
and advanced through parametrization into nonempty numerical-dual geometry;
their full compilation and integration are deliberately not part of that
regression. The one-loop top box completes eager numerical-dual generation,
retains its model and polarization runtime inputs, omits the incoming gluon
virtualities, and has no SymJIT IR. The same suite covers retained native pagers,
standalone notebook relocation and explicit option forwarding, using the native
`integrate_by_parts` spelling for the override control.

The final compiled-notebook regression exposed and fixed two issues that direct
helper tests did not cover. Marimo rewrites cell-private names, so methods in
the embedded input class could refer to missing rewritten globals when computing
a later runtime point. The three shared native helper names are now
`external_data`, `scalar_dot` and `tensor_vector`; their underlying tensor/dot
operations are unchanged. Generation progress also reread `session.complete`
while the native observer was inside a mutably borrowed `step`. The view now
reads only the caller's retained `state.kernels`. New generation clears this
field, and the caller refreshes it after the native step returns, so it remains
a valid completion indicator without re-entering the owner.

The added regression copies only the standalone file to a temporary directory,
starts it with isolated Python and actual `app.run()`, explicitly builds the
catalogue and generates a one-loop box, pauses/resumes the same retained owner,
and completes eager numerical-dual generation. It then binds two distinct full
physical points, verifies finite matching runtime schemas and immutable template
bytes, and confirms that no integration session started. This test passes in
the full 102-case run; its separate focused execution also passed in 5.41 s.
The merged suite also preserves the concurrent upstream `11fb3dc` regression
that rejects any progress-view re-entry while the native session is borrowed.
Source review found no new algebra, execution thread or implicit scientific
action in either fix.

All three Marimo entrypoints also pass static checks. Execution evidence is
`output/gghh-cards-review/notebook-native-tests.log` and `.xml`,
`notebook-marimo-check.log`, and `notebook-host-evidence.json`. The validation
host uses the `f777` community source base with the current local FastSecDec
bindings, public Symbolica revision
`1deccb8538ccb91dc2c1e58fc0a2e900d2276bf4`, FeynKit revision
`ec21ce9aedacaa95b36d6af371837539343ac10a`, and registry Numerica/Graphica 3.0.1.
Its native release build took 24 min 23 s. This isolated validation host is
distinct from the separately prepared community delivery; the results do not
claim a new Wasm build or a new interactive-browser run.

The parent's extended cold CLI import regression covering D05, D020 and D068
passes together with the native on/off-shell triangle control (two tests,
0.14 s). It asserts the incoming ports, exact zero declarations, runtime card
counts, loop counts and respective three-loop mode/contraction settings. Its
cold-process boundary retains the native model fingerprint guard, with the
already observed warm-state limitation documented in the linked on-shell audit.
The full CLI suite passes 53 tests with one intentional ignored test in 4.14 s;
strict workspace all-target Clippy passes in 1.96 s and Rust formatting passes.

No three-loop numerator contraction, parameterization, sector generation,
evaluator compilation or integration was performed. No three-loop sector count,
pole order, numerical value or performance claim follows from this review.
