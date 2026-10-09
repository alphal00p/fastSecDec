# Dynamic contour programs and artifact selection

2026-10-09. Design proposal for review after the accepted fixed-mode gates.
The proposal and subsequent implementation evidence complement
[the dynamic algebra/solver research](contour-dynamic-implementation.md) and
the approved [Phase B plan](../../CONTOUR_DEFORMATION_PLAN.md).

## Recommended contract

Generate independent, preoptimized programs for these recipes:

- `fixed-v1`;
- `dynamic-polynomial-v1`, the independent correctness baseline;
- `dynamic-sign-aware-v1`, the proposed production prescription.

Select one recipe **before native program decoding and backend restoration**.
A loaded `KernelSet` remains one selected mathematical prescription; a
`SectorKernel` remains one exact program and its numeric evaluators. Fixed
sampling must never evaluate a mode switch, dynamic coefficients, a root
callback or local-strength derivatives. Strength/cap rebinding changes the
selected program's scalar inputs, without generation or optimization.

The completed CLI `generate --contour` should prepare all three recipes.
During implementation, advertise only recipes actually present: existing
fixed-only artifacts remain fixed-only and must reject a dynamic request
with a regeneration message. Do not treat `lambda=0`, a sentinel input or a
runtime function switch as a substitute for an explicit capability.

The same artifact layout and recipe selection must serve ordinary and serial
generation/integration. Generation scheduling is not a capability flag.
Undeformed generation keeps a single undeformed recipe and no contour work.

## What the current implementation establishes

| Boundary | Current behavior and implication |
| --- | --- |
| `generation/types.rs` | `GenerationOptions.program_recipe` selects one explicit recipe, defaulting to undeformed. A generated integral contains that recipe's sectors, exact coefficients and native helper owners. Polynomial dynamic generation is staged; dynamic numerical admission remains closed. |
| `generation/work.rs`, `mapping.rs` | Deformation enters before subtraction. The selected smooth density must be built separately for dynamic strength; rebinding the fixed lambda cannot supply its missing derivative terms. |
| `generation/streaming/sector.rs` | One `GeneratedUnit` contains at most one stochastic sector, plus exact terms and original chart IDs. A worker consumes this unit and returns compact receipts. |
| `kernel/compilation.rs`, `program.rs` | One `CompilationJob` lowers/optimizes a complete Laurent-vector sector. Native Symbolica exact IR is the shared source for normal, eager and precision-rescue execution. |
| `kernel/artifact/binary.rs` | Native v9 stores one selected program per sector, exact expressions, runtime inputs, rich metadata and saved contour-check programs. Decoding immediately restores evaluators. |
| `kernel/artifact/indexed/reader.rs` | `load_record` reads one record and immediately decodes/JIT-restores its kernels; `load_sector`, `load_exact` and `load_all` all use this path. |
| `kernel/artifact/indexed/catalogue.rs` | Indexed v1 has one runtime schema and one sector layout. All record ranges must be contiguous, and source-chart IDs are unique within the catalogue. |
| `kernel/contour.rs` | Capability currently follows the reserved fixed-lambda input. Binding, pilot evidence and the selected mathematical identity are already separate from optional validation policy. |

Consequently, adding a vector of alternative heavy programs to every live
`SectorKernel` is the wrong storage boundary. It would load unwanted IR and
could restore unwanted backends before the requested mode is known. Likewise,
putting all recipes into the existing v1 catalogue would violate its schema,
unique-source and contiguous-coverage invariants.

## Indexed v2: a recipe directory over the existing record codec

Prefer a **flat indexed v2 archive**, with independently addressable native
records and a compact footer directory partitioned by recipe. Continue using
the existing immutable generation-specific data file and atomic manifest
publication. No evaluator serializer is replaced.

Conceptual schema, with names subject to implementation review:

```text
ProgramArchiveCatalogue
  version = 2
  source_identity
  archive_identity
  records_end
  recipes[]
    recipe_id + mathematical_recipe_version
    native_recipe_identity
    physics_input_schema
    recipe_input_schema
    orders + components
    records[]
      existing offset + RecordReceipt
      local-to-recipe output projection
      recipe-local stochastic sector ID or exact-only marker
```

Receipts retain native content IDs, per-record digests/lengths, compact
statistics, local runtime schema and original source-chart IDs. Record bytes
may appear in worker completion order. Scientific recipe ordering and each
recipe's chart/sector ordering remain canonical and independent of scheduling.

Split the existing validation duties rather than weakening them: validate
coefficient projections, runtime schemas and source uniqueness **within each
recipe**; validate byte-range overflow, overlap, full physical coverage and the
footer extent **over all records**. Existing v1 validation continues unchanged.
Do not manufacture a v1 `KernelCatalogue` whose selected records leave gaps
and then turn validation off to admit it.

Store recipe schemas separately. Fixed programs require physical inputs plus
lambda; dynamic programs require physical inputs plus S, L and R (and any
explicitly exposed regularity setting). Requiring an unused union of these
inputs for every program would burden fixed callers and obscure identity.
Each recipe is independently complete, including its exact/constraint rows.

A top-level reader exposes available recipes without reading any native sector
payload. Selecting a recipe returns a lightweight view implementing the same
selective record/sector/exact/all operations. `load_sector(recipe, id)` reads
only that recipe's record. The ordinary path loops only over selected records;
serial workers receive the archive identity, recipe ID and selected record ID.
The existing numerical owner remains unaware that other recipes are on disk.

Do not deduplicate native payloads across recipes initially. Even equal-looking
exact terms can carry different recipe semantics, validation requirements,
runtime schemas or source mappings. Disk space is preferable to a new proof or
an unsafe cross-recipe alias. Common input geometry/source spools can still be
shared where their existing exact identity establishes equality.

An outer bundle containing several complete indexed-v1 archives is a valid
fallback, but is less attractive: it needs bounded virtual seek readers,
additional archive assembly/copies, and another transport layer for manifests
and inspection. Flat v2 reuses the existing record writer and arbitrary-order
append pattern directly; its principal new operation is recipe-aware indexing.
No raw executable pointers or ad-hoc machine-code format are introduced.

## Native payloads and compatibility

Keep the exact v9 decoder and its fixed metadata semantics. Add a native v10
payload only for the new explicit recipe descriptor / dynamic metadata and
validation-program variants. Preserve `PayloadV9` exactly, as earlier native
versions already preserve their original owned schemas. Do not append fields
to a v9 bincode struct and assume serde defaults make its positional binary
encoding compatible. Do not reorder existing enum discriminants.

The dynamic descriptor must bind the construction version, full-sector
dimension, structural counts, envelope coefficient layout, root specializer,
callback identities/arity, derivative requirements, branch convention and
actual subtraction-face requests. Keep retained symbolic maps and inspection
metadata separate from the already optimized evaluator/check programs.
Register canonical callbacks and derivative hooks before native `State::import`
or program decode, including fresh processes and portable/WASM builds.

The archive reader should accept mixed native record versions: an unchanged
fixed-v9 record is usable inside a new v2 archive, while a dynamic record uses
v10. If simplifying the writer means emitting v10 for all new records, retain
an explicit `FixedV1` descriptor and test that its selected evaluator is the
same fixed program. This writer choice does not change the requirement to read
old v9 artifacts correctly.

Old indexed-v1 and monolithic-v9 files adapt to a **single-recipe view**.
Infer `FixedV1` from the old fixed schema and `Undeformed` from the old ordinary
schema only at this legacy boundary. Dynamic admission must use the explicit
new recipe descriptor, not a guess based on a reserved symbol. Preserve the
existing monolithic restriction on serial integration; no conversion tool is
introduced.

Unknown recipe/callback versions fail before evaluation with a specific
compatibility error. An unavailable dynamic recipe never falls back to fixed
or undeformed evaluation. Metadata-only inspection shows the capabilities and
their local sector counts; selected-sector inspection names its recipe.

## Generation and native API shape

Keep single-recipe `GeneratedIntegral`, `CompilationJob` and `KernelSet` as the
inner units. Add an explicit native **program-set generation owner** above
them, with caller-stepped jobs and a requested recipe set. It is not a library
thread/process pool and need not retain all generated expressions. The CLI and
retained HEPKit owner should use this program-set boundary for full contour
generation; existing explicitly fixed-only native entry points can remain as
conveniences that honestly advertise only `FixedV1` capability.

Use an explicit `ContourGenerationCapabilities` or equivalent native enum/set
for this boundary instead of silently broadening the inner boolean's meaning.
At completion, the CLI boolean maps to the full supported recipe set. An
optional native fixed-only recipe request is useful for bounded tests and
users who knowingly want smaller artifacts; no extra CLI flag is required by
the approved interface. The exact public method names should be reviewed with
the HEPKit owner before modifying call sites.

Prepare native graph/parametric input and geometry once. Then perform each
recipe's map, exact symmetry verification, subtraction, Laurent construction
and optimization using the existing native pipeline. Reusable subtraction
formulas may share a disk record only when their complete existing native key
proves equality. Density/source/template caches containing contour expressions
must include the recipe identity. Numerical-dual masks and implicit jets are
part of the selected recipe, not reusable fixed-mode results.

Do not assume recipes have identical representatives, sector counts, local
Laurent support or exact folding. An expression constant in the fixed map may
not be constant under dynamic strength. Map associations through original
source-chart IDs; perform symmetry verification after the full selected
density is constructed. Each recipe gets its own final catalogue and exact
offsets, assembled using the existing local-to-global projection mechanism.

For bounded generation, a job carries `(recipe, representative)` and completes
one recipe-sector record before releasing its process. Another valid scheduling
choice is a source-sector worker that processes its recipes sequentially,
persisting and dropping one before building the next; it must not retain three
compiled owners or their complete symbolic source bodies. The coordinator
holds receipts and indices only. Independent recipes do not justify increasing
the permitted number of active heavyweight worker processes.

Resume receipts and staging source identities must include the requested recipe
set and exact recipe version. A fixed record left by an interrupted run cannot
satisfy a missing dynamic job. Publish the final manifest only when every
advertised recipe is complete. Faults during later recipe writing leave the
previous published artifact usable, as in current universal generation.

Ordinary resident generation may retain more than one selected integral at the
caller's request, but the default bundle writer should drain each compiled
recipe-sector to the common writer. It should not introduce a new requirement
to retain every alternative in memory. This keeps normal and serial artifacts
interchangeable without promising ordinary execution the serial RSS bound.

## Runtime binding, identities and ownership

Separate three identities:

1. **Source identity:** canonical physical parametric input, ordered physical
   runtime schema and mass constraints. It excludes generation options,
   requested expansion orders and the selected contour recipe.
2. **Selected compiled recipe identity:** recipe version, optimized native programs,
   local layouts/constraints and branch semantics.
3. **Bound integrand identity:** selected recipe plus the complete physical
   point and lambda or S/L/R/settings.

The archive content ID additionally binds its advertised recipe directory and
all native record identities. It is appropriate for transport integrity but is
not, alone, a complete checkpoint identity for a selected numerical problem.
Checkpoint compatibility must include the selected recipe and bound settings.
Policies/pilot provenance remain separate; changing validation policy alone
must not invalidate already accepted statistics.

Native v9/v10 record IDs deliberately include optimized evaluator and checker
bytes, compiler settings and native helper transport digests. They identify a
compiled representation, and equivalent optimizer outputs may have different
IDs. Their historical validation rules remain unchanged. This is distinct
from the process-independent physical source identity. New radius callback
tags name the stable defining contract and coefficient arity rather than its
serialized helper bytes; descriptor preparation scopes select the exact saved
owner and never obtain another artifact's same-arity helper from a global
fallback. Saved helper codec v1 retains its original digest-based tag; codec v2
uses the stable contract tag with the same separately verified byte digest.

Changing recipes requires a fresh statistical session and pilot, even when
their exact integrals agree. Fixed and dynamic points are different random
integrands and cannot be appended into the same replica epoch. Changing only
lambda or caps similarly invalidates old production/pilot identity. Strength
rebinding on the same selected program needs no JIT or optimization. Selecting
another saved recipe may restore/JIT its backend, but never rebuild symbolic
density, subtraction, Horner or CPE.

Avoid placing dormant heavy program bytes into `KernelSet::try_clone()` or
every serial worker. A caller-owned archive/template handle can select/reload a
different recipe; the active `KernelSet` keeps only its current one. During a
recipe change in bounded execution, release the old worker and confirm its
exit before loading a replacement, as required for sector eviction already.
No new ownership guarantee should equate `drop` with returned process RSS.

Exact-only sectors need explicit recipe capability too. Their contour checks
must be bound and run before compact exact aggregation discards rich metadata;
an empty aggregate check list is not evidence that no preflight was needed.
Retain current handling of real physical inputs and complex amplitudes.

## Prepared root solver: renewed triple-check

The required operation is repeated refinement of a known unique real root on
`0 < u <= 1` from an **already prepared value-and-derivative callback**, with
bounded iterations, finite-value failures and a meaningful termination report.
No evaluator construction, polynomial factorization or general complex-root
search belongs in this sampling callback.

The 2026-10-09 recheck used the actual fixed-gate Symbolica source revision
`3db1607f5acd9669cde747aa048a3ca0c0fcb2e1` and registry Numerica 3.0.1:

| Check | Evidence |
| --- | --- |
| Public API | `AtomCore::nsolve` / `nsolve_system` in `src/atom/core.rs` accept expressions, initial guesses and tolerances. Public rational-polynomial APIs expose interval isolation/refinement and all-root approximation. Numerica's exported modules provide number domains, matrices and integration; no prepared bracketed callback solver was found. |
| Source and tests | `src/solve.rs::nsolve` constructs separate value and derivative evaluators on every call, uses unconstrained Newton steps, and stops on an absolute correction; its API explicitly says this does not certify the root. Existing root tests call that expression API. `poly/univariate/roots.rs::refine_root_interval` performs square-free factorization before rational bisection. Numerica source/test searches found integer-root Newton and finite-field factorization, not the required real callback operation. |
| Executable probe | Existing `target/contour_dynamic_probe/main.rs` compiles native implicit higher jets, native exact closed-form identities and native Hessian-ray coefficients, then calls `nsolve` repeatedly and separately evaluates a prepared H/Hr program. Re-executed successfully during this review without rebuilding or introducing another solver. |

The latest local rerun observed about 227 microseconds per `nsolve` call
(64 calls) and 44 nanoseconds per prepared H/Hr evaluation (8192 calls).
An earlier run observed about 51 microseconds and 44 nanoseconds respectively.
These noisy small measurements demonstrate neither a production speedup nor
the cost of a complete prepared solve; source inspection establishes the
repeated construction. The native implicit-jet/endpoint controls all passed.

This evidence supports proposing a narrow Numerica or Symbolica owner-library
API for prepared safeguarded scalar refinement. It does **not** authorize a
parallel FastSecDec CAS, generic root solver or replacement AD system. Review
the owner API design before implementing it. Keep native closed-form low-degree
specializations as independently verified fast paths; general degree still
requires the owner solver and precision/error-propagation contract.

A floating bracket returned by an ordinary solver is not automatically a
certified enclosure. Under enabled runtime validation, certify the monotone
equation's endpoint signs with native ball arithmetic and escalate precision
if inconclusive. Under `off`, do not retain this extra certification overhead.
Termination/nonfinite checks remain essential in either policy. Error-tracked
coefficient inputs cannot be replaced by their centers and then returned as
an exact tracked root; native implicit uncertainty propagation remains a
separate required gate from symbolic derivative hooks.

## Acceptance for this storage design

- Old fixed-v9 and indexed-v1 artifacts load and bind fixed mode; dynamic
  requests give a precise missing-capability error. Keep actual legacy wire
  fixtures, not only current-type round trips.
- Normal/serial generation and integration consume the same new archive in
  all four combinations, selecting fixed and both dynamic envelopes.
- A selected fixed load reads/decodes/JIT-restores no dynamic record. Instrument
  decode/restore calls and confirm zero root callbacks in fixed sampling.
- Strength/cap/policy rebinding performs no density construction, subtraction,
  Horner or CPE. Policy-only checkpoint resume retains statistics; recipe and
  mathematical-input changes reject old statistical identity.
- Layout tests include different recipe-local real/complex supports, exact
  offsets, symmetry multiplicities and stochastic counts. No missing output is
  silently treated as zero without the native exact layout proof.
- Fresh-process native and portable restoration register every callback before
  decoding, including endpoint implicit jets and complex SIMD batch evaluation.
- Interrupted writes, missing recipe receipts, corrupt selected/unselected
  ranges, unknown recipe versions and generation resume are rejected or
  recovered under the existing integrity-policy distinctions.
- Aggregate RSS during generation and serial loading scales with active
  workers and selected-sector size, not total sectors times recipe count.
- HEPKit uses the native program-set/selected-template objects; community
  remains registration/reexports/stubs, with snapshots detached from active
  native sessions.

This proposal is ready for mathematical, artifact-compatibility and HEPKit API
review. No dynamic production code or owner-library patch was added here.
## Generation ownership transfer (implementation gate in progress)

Generation now transfers the immutable native recipe owner separately from its
symbolic expression tags. The staging adapter uses the existing saved helper
codec; staged restoration registers helper owners before importing expressions.
Detached compilation jobs and their completions retain an explicit strong owner
until numerical callback workspaces have been constructed and the compiled
integral has adopted its descriptor. Normal and caller-stepped compilation use
the same attachment path.

The association gate compares the explicit runtime schema and retained
representative charts, including their full dimension and positive-factor
count, before evaluator construction. Symmetry copies do not duplicate the
executable descriptor, but a projection may omit their own descriptor only when
the actual retained representative is selected and has its descriptor. Exact
aggregation keeps only helpers referenced by its exact expressions.

The generation-only `DynamicCheckSource` captures native polynomial/rational
atoms for the independent future checker: direction norm, leading causal term,
odd-order bounds and spectral data, and positive factors and their ray
coefficients. Its F/U ray uses an independent strength input, with no production
root callback in the source. It is transported through staging; a later slice
will optimize and save the checker programs. No final dynamic numerical
admission is implied by this transfer increment, and binding and restoration
continue to reject dynamic execution explicitly until that checker is complete.

Independent review confirms the ownership and representative selection
boundaries. The transfer gate also validates the complete ordered source-output
schema, source count and coordinate identities against the descriptor. Before
lifting dynamic admission, the saved checker must additionally bind the actual
causal/positive polynomial identities and their proofs. Matching dimensions,
factor counts and output schemas alone is deliberately not treated as a
certificate of those identities or of the causal bounds. New checker fields
will use an explicit new wire layout, preserving existing v10 decoding.

Native acceptance at this increment: descriptor/source tests 9/9, selected
polynomial generation controls 2/2, generation streaming controls 12/12, and the
full core library 271 passed with 16 ignored. The full library includes a detached
compilation job that outlives its generated owner and still evaluates its native
callback correctly. The streamed subprocess additionally evaluates after dropping
both its generated object and its compiled kernel owner. The final
symmetry-candidate lifetime retention passes both the focused streaming rerun
and the complete library gate. Canonical source-identity tests additionally
verify the artifact-writer contract and reproducibility across unrelated native
symbol and coefficient-ring registrations in a fresh process. Stable helper
contract tags pass reverse-registration subprocess controls; scoped owner
routing passes nested/unwind, foreign invalid-body, failed-restore, legacy-codec
and detached precision-mapping tests. The descriptorless native-v3 reader
additionally rejects semantic callbacks under an unrelated outer preparation
and restores that caller's state on failure. The complete CLI package passes
156 tests with 7 ignored on the scoped-helper increment; the subsequent
native-v3 guard is exercised by the final complete core gate. Strict workspace
and all-target Clippy passes with warnings denied.
