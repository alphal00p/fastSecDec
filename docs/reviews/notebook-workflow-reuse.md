# Eager notebook workflow and native reuse review

## Scope and reference evidence

This review follows the 2026-10-07 request recorded at the top of
`FIRST_PHASE_PLAN.md`. It covers native graph enumeration and presentation,
cooperative sector generation, and the boundary between native owners and the
Marimo caller. Final native and portable-host gates are recorded below; actual
notebook browser acceptance belongs to the coordinating review. No new Wasm
execution or hosted deployment is claimed.

The current published presentation reference is
[HEPKit's four-loop gluon numerator notebook](https://hepkit.org/gallery/notebooks/four_loop_numerator.html).
Its embedded source was inspected, not approximated from screenshots. It uses
Marimo 0.24.0 and Symbolica 3.0.0, with native `Model`, `FeynmanDiagram` and
`TensorExpression` objects. The public source displays a selected native diagram,
then obtains `diagram.numerator_expression(in_lmb=True)` and the evaluated native
overall factor. Tensor structures, projectors and expressions render themselves.
The contraction calls are `simplify_algebra(contract="minimal", color=True,
gamma=False).to_dots()` and `simplify_algebra(contract="dots")`; no notebook CAS
or handwritten graph renderer is needed. These operations must be explicit
preparation actions in FastSecDec, because passive display must not contract.

The old sibling community checkouts are dirty and were preserved. Public main
was `9960ba17207bc49f3a045e5a1ffea37eb4b8187a`; the caller created a fresh ignored
checkout at `output/notebook-workflow/community`. The pinned FastSecDec native
probe uses FeynKit `259df87`; the fresh community resolves FeynKit `ec21ce9`.
The newer source also changes default diagram labels from `FK` to `D`.
The installed newer host was subsequently probed independently; the count
results and label change below agree with its own native generator.

## Exact process enumeration

The public native process API accepts PDG selectors. The latest inspected
`ec21ce9` source exposes `particle_selection=[6, 21, 25]` directly and includes
antiparticles automatically. This native option is preferred in the notebook.
The pinned `259df87` probe uses the equivalent complement veto derived from
`abs(p.pdg_code) in {6, 21, 25}`, retaining top/antitop, gluon and Higgs. It does not
restrict allowed vertices to `ttg` and `ttH`: that would incorrectly omit allowed
Higgs self-interactions and gluon vertices. Native `coupling_orders={"QED": 2}`
means an exact order. `loops=(1, 2)`, `symmetrize_initial=True` and
`symmetrize_final=True` express the remaining requested conditions directly.

Python's default generation filters are narrower than literal “all diagrams.”
The explicit settings `maximum_bridges=None`, `self_energy=None`,
`tadpoles=None`, `zero_snails=None`, `allow_self_loops=True` and
`allow_zero_flow_edges=True` disable those extra filters while retaining native
graph admission. The last option is independent: omitting it would still reject
internal edges with identically zero momentum flow. Native color-zero filtering
stays off; explicit numerator contraction determines exact zeros later. Numerator grouping stays off;
`projector=S("1")` obtains amputated native tensor numerators for later explicit
polarization/color projection. A restricted-particle collection is not described
as the full gauge-complete Standard Model amplitude.

The focused Rust probe uses `Process`, `GenerationOptions`,
`GenerationFilter::CouplingOrders`, native graph edges/vertices and model vertex
orders. The initial probe left the native zero-flow exclusion at its default and returned
102 diagrams (5 one-loop and 97 two-loop). A final independent source audit
identified that extra exclusion and the corrected literal-all probe enables
zero-flow edges explicitly. It returns **152 diagrams: 5 one-loop and 147
two-loop**, in approximately 0.45 s in the pinned Rust host. One-loop diagrams
carry QCD order 2/QED order 2, and two-loop diagrams QCD order 4/QED order 2.
Every native edge passed the whitelist and each interaction-order sum passed
QED 2. No extra QCD filter was needed. Evidence is ignored
`output/notebook-workflow-review/diagram_zero_flow_probe.rs` and
`diagram-zero-flow-counts.txt`.

A separate direct probe of the installed latest `ec21ce9` HEPKit wheel reproduced
both counts: 102 with zero-flow edges excluded, 152 with them allowed. Under the
requested full catalogue the first one-loop entry is index 35, `D035`; the old
pinned source calls it `FK035`. Its canonical ID is unchanged from the initial
102-diagram collection's `D019`/`FK019`. Therefore default selection searches the
native loop count rather than assuming position or a fixed label. Evidence is
`latest-zero-flow-counts.txt`. Allowed reducible/color-zero graphs remain in the
collection; an unsupported integration is not converted to zero. The separate
interactive one-loop box is chosen from actual native graph content.

## Cooperative native generation

The public geometry owner already supplies `GeometryPlan::charts`, native
`GeometryJob::run`, `prepare`, `PreparedGeometry::cones` and `finish`. Their
opaque completions validate owner, stage, coverage and canonical order.
`GenerationSession` retains those owners and completions across calls. Mapping,
symmetry preparation/admission and coefficient expansion call the existing
FastSecDec functions that delegate algebra/canonization to Symbolica/Graphica.
The multiplicity, exact-term folding and final Laurent assembly were moved
without mathematical changes into a common assembly owner shared with the
existing one-shot/dispatched routes.

Construction stores the native integrand/options only. `step(max_units, observer)`
executes bounded native work units with no pool or hidden thread. A unit is a
geometry chart/cone or merge, chart mapping, symmetry preparation/admission, one
representative's coefficient expansion, or final assembly. `Break` requests a
pause after the current successful unit has been retained. Completed
representatives are never regenerated on resume. Native calls themselves are
atomic: this is a unit bound, not a wall-time/preemption promise. The existing
one-shot cancellation behavior remains available for callers that prefer to
abort inside native progress boundaries. No partial `GeneratedIntegral` is
exposed; failure is terminal, and successful extraction is available only once.

New owner tests compare resumed and one-shot metadata, complete Laurent
coefficients, exact contributions and endpoint profiles across both coefficient
representations and cube/projective domains, plus bounded work, no-work
construction, empty inputs and terminal failure. All four scientific owner tests passed on the native build and on the standalone
portable host build, including exact folding and reserved-coordinate collisions. The coordinating
agent independently reviewed state ownership, native admission and terminal
error behavior, and identified two presentation fixes: intentional callback
return handling and active total timing. Both are fixed. Session elapsed and
total timing count active step calls and exclude paused wall time. Subsequent
installed-host and workspace acceptance is recorded in the final gate closure
below.


## Independent eager and compilation audit

The evaluator selection is a small enum over the existing native
`JITCompiledEvaluator` and `ExpressionEvaluator` owners. Explicit `eager` uses
`MappingRequirements` and the already reviewed native coefficient/callback
mapping; it does not introduce another interpreter. Real and complex paths
retain the same exact program, DoubleFloat and arbitrary-precision caches.
Eager batch fallback visits native scalar evaluations in original row order and
reports zero matrix calls. The native CLI's default `auto` still chooses SymJIT;
the Python settings and notebook choose eager explicitly, with one core.

`CompilationSession` retains each completed native `CompilationJob` evaluator.
Each caller step executes at most its number of sector/final-assembly units;
there is no worker pool or background thread. Pre-unit observer cancellation
performs no work, post-unit cancellation retains the completed evaluator, and
terminal errors expose no partial kernels. Active elapsed time excludes paused
wall time. The composed Python owner retains native parametrization, generation
and compilation objects and captures the completed generated integral separately
for lazy scientific inspection. A native call is still indivisible: one expensive
coefficient or evaluator build can delay servicing a pause until that call ends.

Independent source review found transient observer-false decisions could be lost
within a multi-unit Python call and that errors constructing compilation or
attaching mass constraints needed terminal treatment. The author fixed both.
The pause decision is now latched for the call, callback exceptions propagate
without discarding successfully completed native work, and numerical/algebraic
failures cannot leave an owner that silently continues without a result.

Runtime graph inputs remain native `RuntimeModelBindings`, `GraphIntegral` and
`Kinematics` objects. Declared real symbols are ordered and passed to the exact
program, used independent model leaves retain native dependency expressions,
and finite/nonzero real mass constraints are attached before binding a point.
Kernel binding clones preserve the unbound template, then derive point-specific
identities and exact offsets. No Python dependency solver or tensor/CAS fallback
is introduced. The notebook's Gram construction uses native tensor contraction
on generic symbolic vectors to identify structural zeros; numerical coincidences
at an editable point are not frozen into the generated expression. Explicit
zero-mass specializations in older demonstration inputs remain explicit, while
nonzero model parameters are supplied at integration time.

Native eager artifacts carry a distinct interpreter/GMP/MPFR compiler policy.
Cold loading restores that backend, including parameterized exact-offset helper
programs. Default `auto` is omitted in the existing settings serialization, so
historical native artifact policy/identity is not changed merely by adding this
selection. Portable interpreter arithmetic retains its own Malachite/Astro policy;
this review does not claim binary interchangeability between those backend
policies. Raw exact evaluator loading retains the existing trusted-producer
contract, rather than adding a private instruction validator.

The independent focused Rust probe passed real and complex full Laurent vectors
for `x^(-1+eps)/(1+m*x)` (optionally multiplied by `1+i`) at `m=2` and `m=3`.
It checked direct/stepped byte and identity equality, pause/resume without repeated
sector construction, terminal duplicate-schema failure, missing/nonfinite input
rejection, native eager statistics, cold reload and immutable template bytes.
Scalar evaluations and batches 1, 7 and 256 over 263 rows matched the analytic
complete vectors; no SymJIT dispatch occurred. Distance-routed native DoubleFloat
and 3322-bit arbitrary evaluations also matched those vectors. A zero-sector
parameterized exact result survived stepped final assembly and cold binding.
Evidence is ignored `output/notebook-workflow-review/eager_session_probe.rs`
and `eager-session-probe.txt`. This is native-host acceptance, distinct from the
four portable generation-session tests and from actual browser/Wasm execution.

## Test migration and thin host boundary

All 37 controls in the assigned generation, generation-context,
generation-metadata, independent-generation and regression files pass under the
current caller-responsibility domain contract. Three private domain controls
retain structural residual-polynomial admission without resurrecting threshold
certification. Private metadata admission controls reject altered representative,
permutation, kernel association, Jacobian, determinant, pre-subtraction version
and powers; binary corruption remains covered separately. Scientific vector,
reference and pole-failure assertions remain in place. Formal generation of an
inadmissible endpoint example is not claimed to reproduce its physical poles.

The fresh ignored community host only registers/reexports these native bindings.
The sector-decomposition package already forwards the native module; explicit
root HEPKit reexports now expose the nine new settings/session/observation types,
with matching thin stub-generator imports. No older dirty sibling checkout was
changed. Targeted native stub regeneration remains a host build task.

One upstream documentation limitation is retained explicitly: FeynKit `ec21ce9`
forwards native diagram/family `sector_decompose(**kwargs)` correctly, but its
own static method signature/manual stub inventory still advertises the former
`physical` default and omits the new model/runtime-parameter keyword names.
FastSecDec's own function and session signatures are current. Fixing the owner
signature belongs in a separately validated upstream source change; no cached
Cargo source or alternate wrapper implementation was patched for this milestone.


The assigned nine CLI integration gate files now pass all **19** tests, with the
last two repaired targets rerun separately. Gate migration uses `.fsd` basenames
and removes both paired files when testing result independence. References are
stored relative to the artifact and the default reference resolves after a
working-directory change. Human map inspection now explicitly selects a sector;
selected JSON preserves the same original chart IDs as complete native metadata.
Coordinator coefficient progress is tested as the master workload rather than
pretending concurrent workers share one local expansion pass. Its phase wall
time remains exclusive, and final/stage-entry status survives coalescing.

The boundary cancellation gate revealed a harness pipe deadlock: it took stderr
to synchronize on the first native event, then stopped draining that pipe while
waiting for the child's result. A concurrent drain now permits the final status
burst and stdout result to finish; the native cancellation/partial-coverage
assertions remain unchanged. All three boundary controls then complete in about
0.3 seconds. No scientific tolerances were relaxed. Evidence is
`output/notebook-workflow-review/cli-migration-tests.txt` and
`cli-migration-followup.txt`.


The final notebook caller source review also accepts physical-point chronology:
`Study` archives the previous allocation report before rebinding an independent
kernel clone, and the report captures the previous parameter map and point-specific
kernel identity. Native defaults, native Gram contractions and explicit MT/ymt/MH
choices are combined only at an explicit Integrate action. Provisional views are
read-only and are not admitted into checkpoints. Rust rich presentation uses
retained native getters and native covariance; absent mean/error coverage stays
unavailable rather than becoming zero. These source findings do not replace the
coordinator's installed-wheel numerical and browser checks.


## Frozen reference inputs during runtime-card migration

The remaining reference gate migration preserves all external values, errors,
normalization evidence and recorded source hashes. Eleven changed historical run
cards are retained byte-for-byte under
`crates/fastsecdec-cli/tests/fixtures/historical-run-cards`; frozen reference JSON
is unchanged. The source verifier accepts a migrated run-card path only when its
archived bytes match the original recorded hash. Changes to models, graphs or
other sources still fail the original evidence check.

A native input gate compares every current card, bound at its saved runtime
point, with the historical density under the original fixed-model convention.
Coordinate order, regulator and integration domain must also agree. It reuses
normal input loading and native Atom substitution/equality, without reconstructing
Symanzik polynomials or adding a graph/algebra helper. The default-model leaf test
now separately checks runtime symbolic preservation, two evaluator bindings and
unchanged template bytes, while explicit fixed mode retains its original scalar
restriction assertions. The ignored full multiloop diagnostic additionally checks
paired template files and retained model/kinematic inputs. The targeted
Clang-linked CLI input suite now passes all six tests in 0.47 seconds, including
both new controls. The runtime bubble check compares its pointwise pole density
with the exact combined-primary-chart density `2*scale/(1+t)^2`, not with its
integrated residue. Frozen-card setup uses native TOML table insertion. The
ignored expensive full multiloop diagnostic remains distinct from these passing
input and source-evidence gates.

A separate installed native Symbolica probe now verifies all eleven current
densities against those frozen inputs exactly after binding the saved point.
Floating-point point values are converted to their exact rational ratio, matching
the Rust test's native `Rational::try_from`; parsing decimal text as a floating
Atom would instead test a different coefficient representation. Evidence is
`output/notebook-workflow-review/reference-density-probe.txt`.

## Lazy selected-sector expression viewer

The existing HEPKit four-loop notebook displays `TensorExpression` values using
the native tensor presentation owner. Its explicit public viewer is
`TensorExpression(expression).paged(page_size=25)`, also documented in the fresh
community `examples/hep/README.md` and tensor stubs. An installed-wheel probe
confirms wrapping a scalar Symbolica expression preserves it exactly. Viewer
construction retains the native source and creates neither a rendered page nor
a widget; requesting display renders the first bounded page.

The selected coefficient is restored through Symbolica's existing
`AliasedAtom::clone().into_inner()` operation. The new explicit binding
`CompactCoefficient.expression()` invokes exactly that owner operation and
returns its native Atom. It does not restore other coefficients. This is
preferable to `GeneratedSector::coefficients()` here, because that cached
accessor intentionally materializes the entire coefficient vector. Passive
representations retain compact roots and do not call either operation. Restoring
the selected expression is still an atomic native operation and can cost time
or memory for a very large alias graph; page budgets limit rendering, not the
size of the requested exact expression itself.

The reused implementation is FeynKit `ec21ce9`'s
`crates/spynso3/src/display/paging.rs` and `crates/spynso3/typst/paging.py`/`.js`.
An `Arc` retains the native expression; only the requested immutable subtree is
serialized. A request admits at most 2,000 nodes, depth 32 and 64 KiB of expression
data, then at most 256 KiB/10,000 MathML elements of rendered output. Native
rendering uses a bounded subprocess deadline; the existing Wasm branch uses its
embedded compiler. Nested sums retain their surrounding factors and expose
subexpression/parent navigation. The viewer caches at most three pages, and
`close()` releases its source, cache and widget. FastSecDec needs no alternate
renderer, alias walker or expression parser.

Marimo 0.24.2's `as_html` selects the `_display_` protocol before HTML/MIME reprs.
The installed probe of `mo.as_html(viewer)` produced a 342-byte live
`marimo-anywidget` element with exactly one cached page, rather than a full
expression HTML string. Retaining the viewer in the notebook's explicit Inspect
state and closing it on a new selection scopes its lifetime correctly. Calling
`to_html()` directly would request a full export and is not the inspection path.

Five unchanged upstream paging controls passed in 3.23 seconds: contiguous and
reversible pages, nested factor/subexpression navigation, serialized payload
excluding undisplayed terms, bounded timeout fallback, and cache/budget limits.
Evidence is `output/notebook-workflow-review/native-viewer-tests.txt` and
`native-viewer-marimo.txt`. These installed native controls establish the reuse
boundary; the coordinator's actual notebook browser check remains the acceptance
for FastSecDec's selected-sector wiring. No actual Wasm execution is claimed.

The final notebook source wiring was then independently reviewed: it resolves
the requested signed epsilon order against native coefficients, restores only
that selected expression on explicit Inspect, and retains the Pager on `Study`.
Replacement, a new generation action and inspection failure close the previous
viewer. The label correctly states that the retained symmetry multiplicity is
already included, while the global coordinate-independent exact offset remains
separate. No passive overview or selected metadata representation restores the
full expression.

## Thin community bridge readiness

The earlier community bridge PR 18 is already merged. The fresh local checkout
started at public main `9960ba1`, initially with a temporary path dependency on
this repository's isolated binding crate. For delivery that path was replaced
by the published FastSecDec commit
`73b0442c4ba7e1e9dd3379f985f80dd11b337291`; only the four FastSecDec lockfile source
revisions changed. The active GitHub account was verified as `ValentinHirschi`.
The seven-file bridge follow-up is published in
[community PR 22](https://github.com/symbolica-dev/symbolica-community/pull/22),
commit `882ad55ef41fe9c76bcdec63efe8c831bf2fd7d3`, with that user's configured
name and email. The isolated checkout is clean; older dirty siblings remain
untouched.

Only nine native class aliases are added to the top-level HEPKit Python package;
their `__module__` and object identity remain owned by
`hepkit.sector_decomposition`. The top-level stub and existing generator footer
mirror these aliases. The narrow flat-namespace gate now checks those shared
identities and explicit stub reexports while retaining all original native HEP
class checks; it passes against the installed local wheel.

The public host generator already supports `--fastsecdec-only`. For this local
milestone, an ignored isolated runner uses the same public `StubInfo` inventory,
FastSecDec's `stub_source`, and the host's existing Python-3.9 compatibility pass
to write only `sector_decomposition/__init__.pyi`. It avoids unrelated HEP module
regeneration and leaves the native host build's Cargo target and features alone.
The isolated code-generation build completed in 8 minutes 30 seconds with two
jobs, then produced the 40,109-byte maintained package stub. Repeated extraction
was byte-identical. Python-3.9 AST checks, all nine native alias identities and
five installed method signatures passed, including the selected expression
method, cooperative generation, eager compilation and parameter binding.
Runtime-input keyword declarations and eager/default-progress values were also
checked. The unchanged package-layout controls plus the narrow migrated
flat-namespace control pass all ten tests in 0.16 seconds. Evidence is
`output/notebook-workflow/stub-extraction-result.txt`, `bridge-stub-contract.txt`
and `bridge-stub-tests.txt`. After pinning the published revision, locked Cargo
metadata and the resolved binding owner's native-dependency checker pass.
All ten bridge controls pass again in 0.12 seconds, together with the complete
stub's Python-3.9 grammar and installed signature/alias checks. Evidence is
`bridge-public-metadata.json`, `bridge-public-tests.txt` and
`bridge-public-stub-contract.txt`. The public binding Rust sources match the
validated local binding sources; no fresh host-wheel build is claimed for this
pin-only delivery step.

One upstream documentation limitation remains: FeynKit's diagram/family
`sector_decompose` method text signatures and maintained method stubs list the
older keyword set. The actual public native methods accept `**kwargs` and
forward them unchanged to FastSecDec. An installed family-forwarder probe passed
`runtime_parameters` and `model_parameters` unchanged to a captured backend.
Thus this is not a runtime keyword rejection, and the notebook uses the current
`Integral` API directly. No cached upstream source is edited or forked to disguise
that limitation. Evidence is `output/notebook-workflow/bridge-forwarding.txt` and
`bridge-flat-namespace.txt`.

## Parametric polarization and model-input follow-up

The large displayed fraction
`2535301200456458353478625262249/316912650057057350374175801344`
was traced exactly through native tensor contraction. It equals
`-8*(epsilon1·epsilon2)` when the native f64 helicity components
`±0.7071067811865475` are converted to exact binary rationals. The original
notebook kept that nonzero polarization-only Gram entry fixed. It was not a
frozen model coupling. Python's native `FourMomentum` constructor accepts f64
components; no exact symbolic polarization constructor was found. Reconstructing
a different helicity convention or rounding this rational to eight would bypass
the native owner.

The narrow fix therefore promotes every nonzero native Gram entry, including
the polarization cross product, to a runtime evaluator input. Only the proved
exact zeros remain structural. The native wavefunctions and tensor contraction
supply numerical values at the explicit Integrate action. This matches the CLI's
parameter workflow without introducing polarization algebra or compiled
physical-point coefficients.

An independent installed D150 eager generation/control now retains seven Gram
inputs and the six model leaves `Gf`, `MT`, `MZ`, `aEWM1`, `aS`, `ymt`; only the
explicit zero widths are fixed. HEPKit owns the dependent electroweak/Yukawa
expressions, so all leaves appearing in that native dependency closure remain
available, including ones that could cancel after additional simplification.
Against one immutable evaluator template, doubling `ymt` scales the complete
Laurent mean vector by four and its full covariance by sixteen. Doubling `aS`
or `Gf` scales the vector by two and covariance by four, at relative tolerance
`2e-12`. Bound identities change and template bytes remain identical. Evidence
is `output/notebook-workflow-review/polarization-rational.txt` and
`notebook-parametric-model.txt`. The probe uses an explicit validated numeric
policy to isolate parameter binding; it does not change the notebook's distance
policy or claim statistical convergence from its small allocation.

## Final independent gate closure

The final owned-source workspace gate passes **486 tests with zero failures**
across 73 target results. Its **25 ignored controls remain ignored**: these are
explicit captured-input, resource-intensive numerical/reference or profiling
diagnostics, not successful scientific checks. The separate strict workspace
Clippy gate also exits successfully. Both gates run against the milestone's
owned source without the concurrent, unrelated numerator-contraction feature;
therefore the new test and implementation slices do not rely on that feature.
The ignored output copies are evidence, not additional maintained source trees.

The standalone portable-host gate passes **59 tests with zero failures** across
11 target results. This covers the eager/portable owner on the native host and
must not be described as an actual Pyodide/Wasm run. The final installed native
Python binding suite passes **73 tests in 1.63 seconds**; the notebook suite
passes **59 tests in 8.33 seconds**. These include retained generation and eager
compilation, runtime model/Gram binding, full-vector/covariance observations,
read-only rich representations and the actual selected-expression pager. The
two corrected CLI input fixtures separately pass all six focused input tests;
their exact density and bound-vector assertions are retained.

Evidence was read directly from `output/notebook-workflow/owned-gates.log`,
`owned-clippy.log`, `portable-gates.log`, `python-binding-final.log` and
`cli-input-followup.txt`, plus
`output/notebook-workflow-review/notebook-final-tests.txt`. The ten community
package/alias/stub controls and five unchanged upstream lazy-viewer controls
remain additional, separately attributed checks described above.

This closes the independent native ownership, numerical/API and ecosystem-reuse
portion of the milestone. Native calls remain atomic, including potentially
expensive high-precision special-function preparation; the cooperative API does
not promise mid-call preemption. Actual interactive notebook checks are owned
by the coordinator and are not inferred from headless tests. The seven-file
community bridge subsequently passed its public-pin checks and was published
as PR 22, as recorded above. Historical draft-PR and earlier Wasm observations
retain their original dates/build identities and do not substitute for this
delivery step.

## Live widget lifetime and QMC caller overhead follow-up

The actual browser exposed a missing native expression widget after an
Inspect → QMC → Havana sequence. Retaining the Python Pager alone is
insufficient in Marimo 0.24.2. Its
`_plugins/ui/_impl/anywidget/init.py` registers the comm against the cell that
creates it, and `CommLifecycleItem.dispose` closes that comm when the cell is
invalidated. `_runtime/runtime.py` also disposes affected descendants. The
native Spynso3 Pager independently closes its widget when its last frontend
view detaches (`typst/paging.py::_on_message`). A cached HTML fragment can
therefore retain a model identifier whose native comm has already closed.

An installed native Pager plus Marimo lifecycle fixture reproduced exactly that
condition: the Pager and cached HTML survived while dispatch-cell invalidation
removed its comm. Creating and displaying a replacement in a dedicated
inspection cell survived unrelated dispatch, monitor and integration-cell
invalidations, then closed correctly on intentional inspection replacement.
The fixture uses private runtime setup only to exercise upstream ownership;
production code uses ordinary public cells, `mo.state` and `mo.as_html`, with no
comm revival, monkeypatch or registry manipulation. Evidence is ignored
`output/notebook-workflow-review/marimo_widget_lifecycle_probe.py` and
`marimo-widget-lifecycle.txt`.

The corrected notebook source is independently accepted: dispatch stores the
inspection request; the stable inspection cell creates and displays its Pager.
Prepared numerator views have a separate input revision, and generated/kernel
representations and citations have an artifact revision. Monitor progress no
longer invalidates those symbolic views. An explicit new selection/generation
replaces their owners. Downloads now serialize native objects during an explicit
caller action and retain byte payloads for `mo.download`; display and download
RPCs do not access the unsendable native owners. The source audit confirms this
ownership improvement without claiming that every Marimo lazy-download RPC is
necessarily dispatched on a background thread.

Independent performance measurement also found a separate native cost.
`PyQmcSession::step` always returns `snapshot()`, which calls full
`diagnostic_observation()` even when the Python caller suppresses its additional
observation. Native `QmcAccumulator::shift_estimates` and coverage accounting
scan the retained canonical partials. Repeating that reduction after every
128-point package causes quadratic cumulative work in the number of packages.
A three-second process sample observed 1,921 stack samples in the mandatory
snapshot path versus 91 in worker evaluation; it did not support blaming
evaluator cloning or browser rendering for this dominant native cost.

The unchanged installed eager wheel evaluated the same D150 input, seed and
2,097,152-point allocation through public APIs:

| Package points | Packages per step | Step calls | Timer-free wall | Native worker |
| ---: | ---: | ---: | ---: | ---: |
| 128 | 1 | 16,384 | 57.986 s | 5.125 s |
| 128 | 64 | 256 | 8.458 s | 3.933 s |
| 1,024 | 1 | 2,048 | 4.382 s | 3.640 s |
| 4,096 | 1 | 512 | 4.104 s | 3.976 s |

All four complete mean vectors and full covariance matrices were bit-identical.
The final snapshot averaged 5.95 ms with 128-point packages and 65.4 µs with
4,096-point packages. The public `QmcSettings` accepts any positive package size;
`next_work` clips the remaining range and native packages can span shifts, so
4,096 remains valid when each shift has only 1,024 points. The notebook now uses
that public package size, retains 256-point evaluator batches and checks its
50 ms caller budget between packages. This is an operational grouping change,
not a new accumulator or an uncertainty approximation. Atomic expensive native
calls can still exceed the caller budget. These timings are isolated host
acceptance observations, not browser throughput promises. Evidence is
`qmc_step_overhead_probe.py`, `qmc-step-overhead.txt`,
`qmc-step-overhead.sample.txt`, `qmc_step_4096_probe.py` and `qmc-step-4096.txt`
under ignored `output/notebook-workflow-review/`.
A separate 1,024-point/three-shift control exercised 3,072-point clipped sector
tails with the larger 4,096-point package and again matched the 128-point
package's full mean/covariance exactly (`qmc-package-tail.txt`).

The accompanying human-only number formatter was reviewed independently.
Decimal placement uses the supplied native mean/error without estimating new
statistics; native vectors, history and covariance remain unchanged. Counts use
exact integer four-significant-digit rounding. Missing uncertainty stays
unavailable, and finite exponent extremes remain printable. All 25 focused
presentation controls pass in 0.06 seconds. Browser lifecycle/performance
acceptance for the corrected frontends remains separately owned by the
coordinator.
