# Native named-coefficient adoption slices

Source-only implementation handoff, 2026-10-05. This narrows
[the production-boundary review](native-named-production-boundaries.md) to files,
interfaces and acceptance steps. No production edit or default change is
authorized by this document. The required three independent original-expression
oracles and complete cold-reader gate must close first; successful generation,
native program construction or one accepted point is insufficient.

The first production patch should extract shared endpoint admission without
changing the physical route. The later named route starts from admitted
`MappedTerm`s, before `subtraction::subtract` constructs its physical Atom, and
returns to the existing assembly before multiplicity and global order padding.
The physical route remains the default throughout the opt-in validation slices.

## Files and ownership

These are proposed implementation ownership boundaries, to be assigned by the
coordinator before editing. They are not a transfer of another active author's
files. The independent auditor should not author the slice being accepted.

| Owner role | Files | Single responsibility |
| --- | --- | --- |
| Symbolic implementation author | `generation/subtraction/endpoints.rs` (new), narrow calls in `subtraction.rs` | Exact affine endpoint admission and checked subtraction degree; shared by both routes |
| Symbolic implementation author | `generation/subtraction/coefficient_first.rs` (new), `coefficient_first/compose.rs` | Native Series attempt controller and endpoint/weight/prefactor composition |
| Symbolic implementation author | `coefficient_first/requests.rs`, `requests/interleaved.rs` | Local coefficient bodies, native derivative/face requests, separate caches and flat native alias lowering |
| Generation integration author | `generation/coefficients.rs` (new), representative block in `generation/mod.rs` | Route selection and a common completed coefficient result; retain existing assembly exactly |
| Generation integration author | `generation/conditioning.rs` (new), relevant `types.rs` definitions/docs | Checked conditioning profiles and honest basis labels; no numeric estimator |
| API/status author | `generation/types.rs`, narrow `context.rs` compatibility, `status/{mod,timings}.rs`, CLI generation presenter | Explicit opt-in, caller limits, cancellation/progress and exclusive stage timing |
| Kernel/persistence reviewer | Existing `kernel/{program,compilation,cancellation}.rs`, artifact tests | Verify reuse and profile semantics; no new evaluator or IR codec |

The existing test-only `series_first.rs` and `series_first/named.rs` are source
material and independent controls, not files to promote wholesale. Keep ignored
capture/replay, JSON records, timers, actual-oracle readers and diagnostic
`Captured*` adapters out of the production module tree. The completed native
`AliasedAtom` and exact evaluator interfaces stay unchanged.

## 1. Extract the endpoint rule once

Move the existing `rational` and `endpoint_power` operations to `endpoints.rs`.
Add one checked `subtraction_count` operation using the existing rule: zero for
constant exponent greater than −1; otherwise native rational
`floor(-constant)` converted to `usize` and checked against
`max_subtractions_per_axis`. Return an internal record with the native rational
constant, slope and count. Use the same helper in physical Taylor, physical IBP,
named composition and mapped-power conditioning. Do not implement a second
affine recognizer or infer powers from expanded coefficients.

This patch changes no mathematical scheduling. Retain the physical exact
derivative-independence pruning and `UnregulatedEndpoint` behavior. Turn count
and degree overflow into the existing typed `GenerationError::ResourceLimit`.
The present representative assembly uses an unchecked sum of cancellation-row
degrees; route that calculation through the shared checked conditioning owner.

Gate: existing analytic Taylor/IBP, fractional endpoint and unregulated/pruned
polynomial controls; add a focused nonintegral rational endpoint count and
count/degree overflow control if those boundaries are not already covered.
No whole-graph trial is needed for this extraction.

## 2. Separate native composition from coefficient requests

`coefficient_first.rs` owns a completed result and the retry controller;
`compose.rs` owns `Series<AtomField>`, pieces and native operations. Move the
prototype's Taylor and IBP loops without changing operation order in this first
slice. Preserve equal-prefactor grouping, native coordinate-weight products,
`map_coeff` remainder behavior and the scalar-independence guard. No inverse,
nonlinear operation or scalar multiplier may contain a formal coefficient name.
Do not combine this move with the unneeded scalar-weight optimization proposal.

Replace the prototype's unconditional twelve-attempt/width-128 caps with an
explicit caller policy: optional maximum attempts, native relative width and
unique requests. `None` imposes no extra cap; checked native integer range and
the existing subtraction degree/piece limits still apply. The opt-in test
campaign supplies finite limits. Each retry uses the observed native absolute
bound deficit, then checks the new bound again. It never assumes the next call
will cover the maximum. All successful nonliteral-zero outputs require the
native final absolute order to exceed the signed requested maximum.

Before composing, use shared endpoint admission to detect a divergent axis
with zero regulator slope. Route that representative through the existing
physical subtraction plus production Laurent path, with an explicit effective
route observation. This preserves the exact all-order pruning/error decision;
do not reproduce the prototype's separate fallback expansion loop. Resource
limits, cancellation, failed native series or unresolved names must not silently
fall back to another expensive algorithm.

Enforce `max_subtraction_terms` on the initial piece set and every growth push,
including IBP boundary pushes currently missing an immediate check in the
prototype. Keep unknown-zero branches and their native remainders. Literal
input zero and a fully covered vector with no retained coefficients need the
existing separate controls; neither licenses dropping a numerical pole.

Gate: move/reuse the existing complete-vector tests for both strategies,
negative and positive requested maxima, Gamma poles, cancellation, empty input,
fractional/essential errors, and exact fallback after a pruned polynomial.
Inspect compact roots without eagerly restoring large coefficient bodies.

## 3. Lower requests locally, with the reviewed interleaving restriction

`requests.rs` owns a per-attempt registry. It deduplicates exact native coefficient
bodies, selects their actual coordinate dependencies, and reserves all source
symbols, declared coordinates and regulator before using checked monotone name
counters. It uses native function/DER requests, native derivatives and native
literal simultaneous replacement. It introduces no callbacks or global
polynomial/body caches. Drop unsuccessful-attempt registries before retry;
only completed native alias definitions escape the final successful attempt.

Keep two distinct caches: `(body, derivative depths)` for unsubstituted partials,
and `(body, derivative depths, argument tuple)` for face results. The restricted
interleaved helper accepts only each argument's own coordinate or literal zero
or one, applies zero-depth faces first, then each positive-depth derivative
sequence before that axis's face. Its result enters only the full-request cache.
Swaps, composed arguments, other constants and requests without faces use the
original native derivative-then-simultaneous-substitution path.

The domain premise comes from the existing admitted mapped regular factors,
including certified face intersections. Concretely, `generation/mapping.rs`
calls `domain::check_residual` on every singular residual; `generation/domain.rs`
requires a nonzero constant and uniform sign globally, or on every zero/one face
with a nonzero face constant, including their intersections.
`ParametricIntegrand::new` admits polynomial-role factors only with nonnegative
integer powers. Successful existing admission is a prerequisite for this route;
argument shape does not prove regularity and does not authorize singular limits.
Epsilon-dependent polynomial
numerators are expanded by native Series first; do not impose the earlier
polynomial-opacity experiment's epsilon-independent-source restriction on this
different coefficient-first route.

Convert prototype unwrap/panic checks on depths, arity, owned unresolved names
and counter overflow to typed failures. Keep tests that restore small named
coefficients and compare exact native identity. Do not restore the entire
coefficient body in the production `wrap_series` path just to repeat that test.
Preserve the native bound and store the exact body when creating the name;
validate formal scalar independence and resolution at the executable boundary.
Unrelated native function support remains the native evaluator's responsibility.

Lower final requests to one shared flat `AliasedAtom` map per complete vector;
inline native coordinate-independent values. Preserve literal zeros and leave
nontrivial cancellations intact. Reuse `kernel::program::build`, which registers
the shared map once. Do not call the convenience per-coefficient alias builder,
introduce a second FunctionMap serializer, or create a second MPFR evaluator.

Gate: existing mixed/composed, negative-power, exact-zero, early zero-depth face,
fallback and partial-cache contamination tests; reserved-looking declared-symbol
tests; native complex/Gamma full-vector and weighted fresh/clone/cold controls.
The actual representative must then pass again through this module split, with
the already accepted independent oracle records and the complete signed union.

### Namespace and long-lived caller ownership

The current `named.rs::fresh` restarts its local `f0`, `a0`, ... counters for
each `Coefficients` registry. Native Symbolica interns these names in its
append-only process-wide symbol table, so repeating the same workload reuses
symbols rather than deliberately creating a namespace per generation. New
high-water indices remain interned; dropping the registry does not remove the
symbol names. Do not introduce UUIDs, generation counters or representative IDs
into those names, and never call unsafe native `State::reset` in a library.

The source bodies, unsubstituted partials and faces are ordinary local Atom
maps in `Coefficients::state`. Completed bodies live in each returned native
`AliasedAtom` map and subsequently in the exact native evaluator. No derivative,
normalization, series or evaluation callback is installed by this route. The
same interned handle can consequently identify different bodies in two separate
coefficient vectors: the local definition map supplies its meaning. Never merge
such roots across vectors without their maps; the existing program builder
checks one consistent shared map within each complete vector.

There is one concrete hygiene gap to close during promotion. Bare `symbol!`
uses `SymbolBuilder`'s default branch, and native `State::get_symbol` returns an
already registered name without checking attributes or callbacks. The current
input-symbol collision set therefore does not protect against an incompatible
reserved name registered elsewhere in the process but absent from this input.
Use the native builder's explicit empty-attributes validation **and** require
`Symbol::is_exportable()` before accepting a reusable handle. Empty attributes
alone are insufficient: native `custom_function_matches` treats an omitted
requested hook as unconstrained, whereas `is_exportable()` requires absence of
normalization, derivative, series, print and evaluation hooks. Preserve typed
failure or bounded collision skipping rather than accepting foreign semantics.
Confirm this behavior with a focused Rust control before choosing the exact
adapter. The source evidence is native `atom.rs::SymbolBuilder::build_with_state`
and `state.rs::{get_symbol,get_symbol_with_attributes}`; it is not an observed
failure in the isolated scientific probes.

Add repeated identical-generation/name-reuse and two-live-vectors/different-body
controls, plus a separately pre-registered incompatible reserved symbol control.
Check the reserved namespace with native `State::symbol_iter`, rather than
asserting that unrelated global symbols or process RSS never grow. These tests
must establish local body ownership and stable name reuse without a new global
definition registry or a per-generation callback lifetime problem.

## 4. One private result at the representative boundary

The new `generation/coefficients.rs` should return a Rust-only internal result:

```text
CoefficientOutput {
    coefficients: BTreeMap<i32, AliasedAtom>,
    conditioning: ConditioningProfile,
    observation: CoefficientObservation,
}
```

`CoefficientObservation` contains typed route/basis, native coverage and scalar
counts only; no `serde_json::Value`, diagnostic paths or string-encoded Rational.
The physical branch calls existing `subtract` and `laurent::expand`, retaining
its `TemplateCache`. The named branch calls the new native composition owner.
Neither branch applies multiplicity or assigns global orders/kernel indices.

Replace only the current representative subtraction/Laurent block. Rejoin
before existing root-only multiplicity multiplication, alias-aware dependence
classification, exact-only accumulation, dense order padding and chart/kernel
association. Keep numerator, geometry, symmetry, mapping, domain assessments
and caller-owned geometry reuse outside this module. Complete generated maps
still describe the original pullback before endpoint subtraction.

Gate: cold/warm/uncached and dispatched public generation agree on the complete
scientific output and metadata; multiplicity is applied exactly once; constant
only and zero-only results retain the admitted geometry. Alias names and
subtraction piece counts are not required to match across representations.

## 5. Conditioning, limits and observers are part of the same opt-in

`ConditioningProfile` owns dimension, rows, checked maximum row sum and a typed
basis. Physical results use retained remainder rows. Named results initially
use one conservative row: for each coordinate, the maximum subtraction count
over **all** original mapped terms, before branch pruning or IBP reduction.
Use the shared endpoint rule, not rows copied from a prior physical run. A
representative sent to exact physical fallback uses that route's actual rows.

On the open unit cube this row dominates every possible such remainder row in
the existing `Cancellation::lost_bits` calculation. It is scheduling input to
the precision heuristic, not a bound on floating-point error. The native
conditioning owner still validates dimensions/checked degrees and controls
adaptive/forced replay. No sampled zero, reduced rescue policy or invented
exact-real/zero flag is introduced. Cold v3 programs keep their conservative
unknown zero/real facts, including possible extra rescue for padded outputs.

Broaden `cancellation_terms()` documentation to **conditioning profiles**, not a
promise that every row is an actually retained physical remainder. Expose the
fresh result's basis as a typed observation/accessor. Keep the numerical artifact
payload unchanged: its existing rows/degree already carry the conservative
profile and remain content-identity inputs. Artifact inspection must use the
generic profile wording and report an unrecorded basis as unrecorded; it must
not infer physical piece provenance from a loaded row. Any later persistence of
the descriptive basis is a separate metadata change, not a new numeric codec.

Add an explicit coefficient-method option with `Physical` as the default and
`NativeNamed` as the opt-in. Keep `SubtractionStrategy::{Taylor,IntegrateByParts}`
orthogonal. Group the optional named resource limits in a typed options value;
do not expose the private resolver mode as a public numerical option.

Add a typed coefficient-expansion progress variant for regular-series work,
endpoint composition, coverage retries and request lowering. Route it through
the existing `GenerationEvent::Progress` bridge so plain, context and dispatched
callers can all cancel. Poll before/after native series calls, between endpoint
pieces/axes, before each lowered request and final assembly. A native CAS call
is still nonpreemptible. Failure/cancellation returns no partial integral and
never emits `Complete`.

Use one new exclusive coefficient-expansion phase duration for the named route;
do not populate physical Subtraction/Laurent durations with overlapping time.
Update the typed timing/status model and CLI match arms in this same slice.
Formal piece/request counts must be labelled accordingly. Retain unchanged
physical-route progress and timings. This is an explicit public enum/options
migration with caller/test updates, not silent observer event loss.

Gate: cancel at each new boundary, tiny attempt/width/request/piece limits,
degree overflow, no `Complete` after failure, and an observer that distinguishes
requested named from effective physical-unregulated fallback. Verify the bound
row triggers at least the existing physical heuristic on representative cube
points; retain weighted numerical accuracy as the substantive test. CLI work
adapts presentation only and owns no symbolic executor or numerical loop.

## 6. Acceptance and promotion sequence

1. Close the three original-input oracle and cold-reader gates. Preserve every
   failed earlier attempt and the complete extra-leading-order comparison.
2. Land endpoint extraction and meaningful regressions; independently audit the
   shared admission and checked counts before accepting that milestone.
3. Implement the separated native modules, private coordinator and explicit
   physical-default opt-in, including limits/conditioning/status together.
4. Run public small Taylor/IBP generation, native program compilation and
   separate-process artifact reload through the existing v3 codec. Reuse
   `native_alias_kernels`, `gamma_regulator`, `artifact_process`,
   `kernel_artifacts`, `generation_context` and subtraction controls. Preserve
   legacy v1/v2/v3 loading and immutable native-IR/content identity rules.
5. Run the actual representative through the **public** opt-in path and cold
   persistence with all independent oracle points. Require native bound,
   complete signed union, multiplicity metadata, weighted rescue and no
   unresolved coefficient names. Capture-only JSON is not its public result.
6. After a combined workspace/format/Clippy and independent native-ownership
   audit, freeze a release build for a new, predeclared full original on-shell
   graph trial. Keep full coverage/status/resource evidence; representative
   success does not establish whole-graph completion or benchmark parity.
7. Consider a default switch only after full-graph scientific acceptance and
   explicit regression review. Broader convergence and worst/mean sample-cost
   optimization remain after capability coverage. No current speedup or parity
   claim follows from this source proposal.

The independent audit for slices 2–5 must specifically check native Series
coverage/cancellation, admitted face regularity, no per-generation global body
retention, one native alias/IR owner, caller-driven cancellation, and fresh/cold
precision semantics. Review diagnostic scalar counts separately from physical
contributions and preserve full-vector covariance/support in later integration.
