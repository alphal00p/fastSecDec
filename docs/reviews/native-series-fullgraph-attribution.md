# Full-graph Laurent attribution: source-only follow-up

This note follows the frozen release trial from commit `6332676`. It changes no
production algorithm and launches no additional symbolic process. The trial's
original 1,800-second/30-GiB bound remains fixed. Its final outcome belongs in
`native-alias-production-results.md`. The run has now terminated at the original
deadline; the observations below do not by themselves identify its inner cause.
The binary and archived source/native snapshots are immutable. Live workspace
source may advance independently after that freeze. Later guarded reference
C++/FORM compilation on CPUs 0–7 overlaps the native correctness process on
CPU 8; no Symbolica is initialized by that continuation. These observations are
not isolated timing measurements.

## Observed boundary

The original graph produces 2,112 charts and 1,026 representatives. At the first
79 completed Laurent calls, native CLI phase events record 231.819 cumulative
seconds in that stage. The largest completed calls are displayed ordinals 63
(52.426 seconds), 39 (36.045), 36 (18.821), 70 (16.149), and 47 (15.380).
Displayed ordinal 80 / zero-based representative 79 subsequently completes in
539.1885 seconds. Ordinal 81 begins at elapsed 806.720 seconds with 771.007
cumulative Laurent seconds. It remains active at the fixed deadline and is
terminated after the original five-second grace, without an artifact.
Ordinals identify this generator's traversal only; they are not external
program sector IDs.

The reported Laurent interval includes template construction/cache access,
native series, coefficient wrapping, and bounded simplification. It does not
identify which native operation dominates. The later targeted ordinal-81 export
is recorded below; no sampling profile or isolated series replay has yet been
obtained for these two representatives. Memory growth and the long phase
interval are observations, not proof of an internal cause.

## Existing owner boundaries

The source snapshot is Symbolica 3.0.1 at
`98794d0d7337ba2b08e4c046dde584ad7fc1ce10` plus the five reviewed fixes.

| Boundary | Native/source evidence | What this establishes |
| --- | --- | --- |
| Whole-template reuse | FastSecDec `generation/laurent.rs`, `TemplateCache` and `expand` | One generation owns an exact `(template Atom, requested maximum)` cache. It does not cache native internal subseries, and no hit/miss timing counters are currently exposed. |
| Depth discovery and requested coverage | `generation/laurent.rs::expand_template`; Symbolica `derivative.rs:376` | The production path asks native Relative1, then the width derived from native trailing exponent when needed, and requires final absolute coverage. Each public native call constructs a new field/context. Native internal retries can still double depth if their own requested bound is not reached. |
| Recursive symbolic work | Symbolica `derivative.rs:427`, Add/Mul handling near 607 | The inspected implementation recursively expands function arguments and each Add/Mul child, then combines native series. No public cross-call or subexpression-series memo table appears in this path. Absence of a cache does not quantify duplicated work in the current slow representative. |
| Coefficient construction | Symbolica `poly/series.rs:1091` and `domains/atom.rs:127` | Native series multiplication accumulates through the native coefficient field. `AtomField` creates/normalizes ordinary Atom coefficients. Evaluator common-subexpression sharing happens later. |
| Scoped normalization seam | Symbolica `domains/atom.rs:42`, `derivative.rs:655/726`; earlier `native-series-coefficient-api.md` | A caller-owned field can enter through public Series/Atom operations, and native normalization hooks run after arithmetic. They cannot avoid the allocation that already happened, do not intercept every coefficient store, and are not yet an executed compact-series solution. |
| Existing compact evaluator boundary | Native `AliasedAtom`, builder `add_aliases` and the accepted production program path | The current source aliases keep coordinate images out of coefficient roots and retain one exact evaluator for every numeric backend. They do not create shared storage inside native coefficient arithmetic before those roots exist. |

Common prefactors are already grouped by exact native Atom in
`generation/subtraction.rs`, before the Laurent stage. A future attribution must
not assume the Gamma prefactor is independently expanded once per endpoint
piece without inspecting the actual captured template.

Resident memory also includes retained earlier work: the generation-owned
template cache keeps previous template keys and coefficient roots until
generation ends, while pending generated sectors retain their aliased roots and
definitions. Consequently the process RSS cannot be assigned entirely to the
currently active native series call. Cache payload/hit counters or a captured
single-template replay would be needed to distinguish these contributions.

## Reference distinction and limits

The frozen Pathfinder `src/subtraction_formula.py::_FormulaContext` caches
regular-function coefficients by endpoint subset (`_g_cache`, around line
2025), expands epsilon before extracting the required coordinate Taylor
coefficients, and instantiates a small regular template. Its separate
`_EndpointProjectorContext` builds the endpoint formula in independent regular
coefficient symbols (`_regular_eps_series`, around line 2325) with a reusable
endpoint signature. These are distinct routes with their own caches; they are
not evidence that the current Rust stage has the same expression structure.
The file also contains separate sparse-series helpers. Their presence does not
authorize copying a second series algebra into FastSecDec.

The previous series-first and callback experiments were already bounded and
retained as negative outcomes. The current late-image alias route passes the
full captured-representative precision/oracle gate but has not demonstrated
whole-graph feasibility. Any next capability probe should first retain the
actual slow input and split the existing stage boundaries, then use native APIs
and independent complete-vector controls. No strategy, cache, normalization
policy, custom coefficient ring, or deadline change is proposed from this
partial run.

### Same-topology reference choices

The frozen `examples/runs/dot_triple_box.yaml` explicitly selects
`projector-formula`, enables `ibp-reduce-to-log-endpoint`, and sets
`direct-projector-cache-term-threshold: 0`. With the inspected current CLI
default `sector_evaluator_backend=explicit` (`FSD.py:1245`), the call path is
`_prepare_sector_runtime_artifacts` → `prepare_endpoint_projector_formulas` →
`prepare_explicit_sector_formulas`. The zero threshold prevents the conditional
curated direct-Taylor override in
`integrand.py::_effective_endpoint_projector_uses_ibp`; the preset therefore
retains IBP-to-log for its singular endpoint signatures. This is a source
statement for that preset and default, not an assertion about an unrecorded
historical command.

The explicit singular builder (`integrand.py:7827`) first constructs an
endpoint assembler, then computes the requested regular coefficient bodies and
substitutes them into the final outputs. In IBP mode,
`_two_stage_assembler_expressions` (6845) combines logarithmic child projectors
and derivative requests. The native epsilon `series` call in
`_EndpointProjectorContext::build_outputs` (`subtraction_formula.py:2276`)
receives independent regular-coefficient symbols and relative depth
`coefficient_count`. It does not receive the fully substituted physical
subtraction density. `_two_stage_derivative_fused_components` (6968) groups
requests by boundary/zero face and `_g_coefficients_by_symbolic_diff` (6761)
constructs the requested regular epsilon coefficients before native coordinate
differentiation. There is also derivative-body and endpoint-substitution reuse
within this path. These are algebraic ordering/reuse differences, beyond the
eventual evaluator format.

The bridge separately expands and stores the common Gamma prefactor once
(`pysecdec_bridge.py:1326–1341`). Its helper includes Python analytic-series
arithmetic for an affine Gamma and falls back to native Symbolica series; the
IBP assembler likewise contains Python coefficient arithmetic. These routines
must not be copied as a second series implementation. Rust currently defaults
to Taylor subtraction, already groups identical native prefactors before each
representative's expansion, and includes that prefactor in the native template.
The public native IBP option exists, but these source differences alone do not
establish that switching it reduces the 504/872-piece workload or preserves a
given pointwise integrand. Any comparison requires native complete-vector
integral controls and the actual captured endpoint powers. No alternative
strategy or prefactor-convolution implementation is introduced here.

## Original capture protocol, before the terminal outcome

If this trial does not finish, the already frozen workspace test binary used by
representative attempt three contains the independently tested capture hook.
No production rebuild or replay of the earlier Laurent work is required.
`output/probes/capture_alias_fullgraph_ordinal80.sh` prepares a separate
180-second/30-GiB process for index 79, with mapped-only mode explicitly disabled.
This remains a candidate for the slow completed ordinal 80. If the whole trial
ends at a later ordinal, a separately frozen wrapper must name that actual
zero-based target; the terminal event must be inspected before choosing it.
The hook reproduces mapping/symmetry/subtraction, skips earlier Laurent calls,
exports the actual original expression/template/images and mapped native inputs,
then returns `GenerationError::Cancelled`. A structural barrier also rejects
missing targets, so the diagnostic cannot yield a usable partial integral.

Before execution, a fresh directory must bind an immutable copy of that binary,
the reviewed timer/wrapper, source/native archives and original embedded input
identities. The existing test asserts 2,112 charts and 1,026 representatives and
exports actual source-chart index, multiplicity, canonical parameter/regulator
names, expression/image sizes and preparation duration. Those identities must
be checked before attributing the capture to the observed displayed ordinal.
Execution waits for the original full-graph process to exit and be reaped; no
second Symbolica process is launched during the trial.

## Executed terminal-representative capture

After the original process was reaped, the independently reviewed
`production-alias-capture-81` froze the existing representative-three test
binary and archived source/native inputs. All 37 bound main source/input hashes
and 13 native-source hashes match the archived snapshots; all 441 frozen files
passed before/after verification. The target was changed in a separate wrapper
to index 80, with the same 180-second/five-second-grace/30-GiB address-space limit.

The capture passed and exited normally in **89.2336 seconds**, with peak RSS
801,232 KiB. It identifies original source chart **119**, multiplicity **4**,
nine parameters and one mapped regular factor of **1,351 native Atom bytes**.
The unchanged Taylor subtraction expression is **137,835,408 bytes**; its late
opaque template is **69,451,573 bytes**, with **415 images**. Their bodies total
only **39,913 bytes**, with largest body **962 bytes**; the large representation
is therefore the epsilon-dependent template, not these image definitions.
Template preparation took **8.7180 seconds**. The test requires 2,112 maps/1,026 representatives and
returns `Cancelled`; no Laurent series, evaluator or usable partial integral is
produced. Its canonical density/card/graph identity matches the earlier capture.

This localizes substantial expression growth before Laurent evaluation and
provides the actual failed representative for the next bounded comparison. It
does not establish that native series arithmetic alone consumed the remaining
whole-graph deadline, nor that IBP will be cheaper. The source-bound existing-IBP
comparison is separately specified in `native-triplebox-ibp-proposal.md`.

## Existing IBP preparation did not complete within its bound

The independently reviewed `native-ibp-prepare-1` first reproduced the captured
Taylor expression exactly, including its 872 pieces (32.382 seconds), then
passed the same mapped native fields to the existing production IBP strategy.
Preparation reached its unchanged total 180-second limit during subtraction:
180.483 seconds wall, child SIGINT, wrapper 124, peak RSS 6,020,136 KiB.
All 52 frozen files passed postflight verification and the child was reaped.
No IBP expression or piece count was exported, so its independent oracle,
Laurent, evaluator and integration stages remain unexecuted.

This development/test build used native dependency opt-level 2 and unchanged
allocator features; it is not a controlled release comparison. The negative
result shows that changing the existing subtraction option alone did not
complete this preparation under its declared bound. It does not prove that IBP
is mathematically unsuitable or determine its eventual release cost. Before a
new attempt, the next source question is how to avoid materializing the large
physical derivative/boundary density before epsilon expansion, using native
series and evaluator definition ownership. The reference's coefficient-first
ordering is relevant; its separate series implementation must not be copied.

## Queued allocator control, separate from IBP

The frozen full-graph Cargo artifact record enables Symbolica features
`bincode`, `float-mpfr`, `integer-gmp`, `native_code_generation`, and `serde`.
It does **not** enable native `faster_alloc`. The existing Symbolica feature
selects mimalloc (`Cargo.toml:51–54`), whose process-global allocator is installed
only under that feature (`src/lib.rs:195–197`). Native owned Atom storage is
`Vec<u8>` (`atom/representation.rs:164`), including derived clones of sums.

Together with the recorded high minor-fault/system-CPU counts, this motivates a
separate, bounded native-feature control if the existing IBP strategy is
insufficient. It does not diagnose the present CPU split, establish a benefit,
or justify an allocator default. Allocator ownership is process-global; the
HEPKit caller/application must retain that choice. The IBP comparison keeps the
current native feature set unchanged so mathematical strategy and allocator
changes are not confounded. No custom allocator or new dependency is proposed.
