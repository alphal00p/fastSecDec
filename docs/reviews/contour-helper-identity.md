# Dynamic root helper identity: independent audit

2026-10-09. This bounded review inspected the actual
`contour/functions/dynamic/program.rs` implementation on
`contour_deformation`, with HEAD `49c9db2ba2de956e4e2a32a6804a8cd92fcefd5f`
and the ongoing uncommitted contour milestone. The finding concerns the use of
native program bytes as a symbolic helper identity; it is not a Symbolica
numerical or serialization defect.

## Finding

`RootProgram::build(n)` constructs the fixed ordered-input function

\[
(-1+\sum_{i=0}^{n-1}c_i r^{2i+2},\quad
 \sum_{i=0}^{n-1}(2i+2)c_i r^{2i+1})
\]

using Symbolica Atoms, native differentiation and a native multiple-output
evaluator. The original implementation uses BLAKE3 of its saved evaluator bytes
both for byte identity and in the symbolic callback tag.

Fresh processes with reversed registration of the same helper symbols generate
different, scientifically equivalent instruction streams. Consequently their
byte digests and callback tags differ for `n = 2, 4, 8, 16` in the executed
probe. The tested `n = 1` cases coincide. This can make symbolic expressions
containing the same mathematical radius callback depend on prior helper-symbol
registration.

| Coefficients | Original registration digest prefix | Reversed helper registration digest prefix | Bytes |
|---|---|---|---:|
| 1 | `cec697218639151a` | `cec697218639151a` | 109 |
| 2 | `ee6fd0ca2299f32d` | `6ee5ff8d1bcee7ae` | 162 |
| 4 | `c37f45ab723c9b1c` | `620e6c94944ca145` | 278 |
| 8 | `e8cb510aa7f6f120` | `9fa5a285d2627daa` | 494 |
| 16 | `d14afce1f306ca48` | `c590266d9248b40b` | 926 |

Registering 37 unrelated symbols or polynomial coefficient rings alone does
not change these program digests. The probe separately confirms that native
`State::export_partial` transport bytes do change in those noisy processes.
Therefore the observed helper difference is not leakage of unrelated symbol
tables into this particular evaluator codec.

For `n = 2`, `export_instructions()` shows derivative multiplication operands
reordered and the native constant entries `2, 4` exchanged with `4, 2`; the
corresponding constant-slot references change consistently. For `n = 4`,
derivative multiplication order and temporary-slot usage change, with constants
`6, 8` exchanged consistently. These differences legitimately require different
byte identities. Exact rational evaluation of both outputs agrees with the
fixed defining polynomial at `r = 1/3, 2, -1/2`, with
`c_i = (i+1)/7`, for every executed case.

Loading original saved bytes in a fresh process after reversed helper-symbol
and ring registration preserves those exact bytes, their digest, their tag,
native callback resolution and all numerical checks. Existing restoration does
not fail in this probe; rebuilding equivalent helpers produces different tags.

## Executable evidence and owner APIs

An isolated `rustc` executable includes the actual private `program.rs` through
`#[path]`; it does not copy the root builder. It ran **120 fresh-process build
cases**: five coefficient counts, eight registration/ring scenarios and three
repetitions. Every fixed scenario is repeatable across those repetitions.
Five additional fresh-process cases restore baseline saved programs under
reversed helper and coefficient-ring registration. All 125 cases pass three
exact rational polynomial/derivative checks and callback resolution checks.
The entire matrix was then rerun with an additional exact symbolic equivalence
check, which also passed all 125 cases: map the native saved evaluator to
`AtomField { statistical_zero_test: false, ..AtomField::new() }`, execute it at
independent symbolic inputs with `try_evaluate_in_ring`, and verify both output
differences using native `expand().is_zero()`. This proves the defining
polynomial identities for each inspected helper, beyond the rational sample
checks. Evidence for this second run is `symbolic-results.log`.

Ignored reproduction assets and full digests are under
`target/contour-helper-identity-probe/`: `probe.rs`, `build.sh`, `run.py`,
`results.log`, `results.json`, and native `.bin`/`.ir` evidence. A pre-fix source
snapshot is retained there as `program-before.rs`. No raw results, executable
or dependency artifacts are tracked.

The executable was linked in `nix-shell` with Rust 1.97.1 and the already-built
native dependencies: Symbolica/Numerica public revision
`7ec1be45ef92ae3b154e0d4ce754c0bdf3d9d0ca`, SymJIT public revision
`33100ae869057f35d9865c933a48bac6699acdd4`. Exact relevant existing rlibs are
`libsymbolica-1302bb9eb6834673`, `libserde-c43b73008ca75bca`,
`libbincode-46c7a2c744a2eb78` and `libblake3-3b7821d873de7bef`. The dependency
fingerprints, rather than newest modification times, select matching Serde
instances. No Cargo build, shared runtime gate, release rebuild or unrelated
workload interruption was needed. Licensing was supplied only through the
process environment.

The owner-source inspection covered Symbolica `evaluate/evaluator.rs`
(`ExpressionEvaluator` Serde, `map_to_ring`, `evaluate_in_ring`),
`coefficient.rs` (`ConvertToRing for AtomField`),
`evaluate/instruction.rs` (`export_instructions`), and
`evaluate/function_map.rs`/`evaluate/tree.rs` (default direct translation and
native linearization). Native `tests/evaluation.rs` also exercises the generic
ring evaluator path in `evaluator_in_finite_field_ring`. The admitted polynomial helper has numeric stacks,
slot-based polynomial instructions and optimization settings, with no external
functions or subevaluators. No replacement evaluator, polynomial algebra or
canonicalizer is needed.

A minimal reproduction, after building the ignored probe and exporting the
license environment, is:

```sh
nix-shell --run 'bash target/contour-helper-identity-probe/build.sh'
nix-shell --run 'target/contour-helper-identity-probe/probe baseline 2'
nix-shell --run 'target/contour-helper-identity-probe/probe reverse 2'
```

`reverse` registers `helper_coefficient_1`, `helper_coefficient_0`, then
`helper_root`, before invoking the unmodified `RootProgram::build(2)`.

## Required correction and validation boundary

Separate the symbolic helper's mathematical identity from its saved native
program identity. A versioned identifier for the fixed coefficient-only helper
contract and ordered coefficient count is sufficient for the callback's
semantic tag. Keep BLAKE3 of saved bytes independently for descriptor integrity,
exact byte identity and transport ownership. Changes to the helper equation or
argument convention must change the contract version.

When admitting two native representations of the same semantic helper, retain
each requested saved byte payload and digest. The registry must not return an
existing different-byte owner when a decoded descriptor expects the newly
supplied bytes. Current admission checks the schema and arithmetic class but
does not prove equality to the fixed helper equation; choosing an arbitrary
global same-arity owner would therefore create a new cross-descriptor coupling.

The coordinating/runtime owners selected a bounded preparation scope: resolve
the stable semantic tag against the caller's actual admitted native helper
owner while preparing callbacks, then retain that exact owner in the callback.
The scope should restore its predecessor on nesting and unwinding; concurrent
caller preparations must remain isolated. Dropping another same-arity owner
must not invalidate or change the prepared callback. Keep explicit legacy
byte-digest routing where needed for existing serialized callbacks. This
preserves saved-program restoration without symbolic reconstruction or
optimization. The demonstrated symbolic ring path is useful for independent
tests, without imposing it on production artifact loading.

The runtime owner has received the reproduction and owns the correction.
Acceptance requires stable semantic callback tags across the process matrix,
unchanged byte-digest distinction and restoration, and regressions with two
different native representations alive simultaneously, exact scoped owner
selection, nesting/unwind restoration and either owner's removal. This review
had not yet validated that correction at the original finding; the independent
correction checks are recorded below. Broader contour scientific/performance
acceptance is separate. Continue using Symbolica to build, store, restore and execute the
native program; no owner-library change is required by this finding.

## Artifact follow-up: mathematical and execution identities

The extended read-only audit inspected `kernel/artifact/binary.rs`,
`kernel/recipe/stored.rs`, `kernel/recipe.rs`, native metadata serialization,
the indexed program catalogue and `kernel/artifact/sector_identity.rs`.
**No descriptor projection or artifact identity change is required for this
increment.** The coordinating owner accepted the distinction below.

| Identity | Current contents and intended boundary |
|---|---|
| `generation::source_identity` | Ordered physical inputs and native canonical expressions; excludes generation scheduling, contour recipe/caps and compiled programs. This is the shared physical source identity. |
| Root callback tag | Names a fixed mathematical helper contract. The original native byte digest was unsuitable here because it entered canonical symbolic expressions. The new contract tag addresses that specific problem. |
| Native v9/v10 `content_id` | Identifies a retained compiled representation, including its mathematical metadata and execution programs. It may distinguish equivalent native instruction streams. |
| Native envelope `digest` | Binds the exact format magic, symbol-state transport and payload bytes. Expensive verification remains controlled by the existing load option. |
| `sector_content_id` | Explicitly documents native representation identity, including program bytes, compiler/precision policy and retained sector semantics; it is not a mathematical equivalence test. |
| Staging receipts | Fence exact immutable execution work and its source context; shared physical source identity remains separate. |

The internal name `semantic_id(payload, 9)` must not be read as a promise of
algebraic equivalence. It hashes native sector program bytes and `CheckProgram`
bytes, compiler/precision policy, ordered inputs/outputs, cancellation data,
endpoint profiles and retained metadata. Exact expressions and mass constraints
use native canonical strings; `PortableMetadata` also serializes its `StoredAtom`
values with `to_canonical_string()`. Thus incidental symbol-state export tables
are excluded from this content projection, while actual optimized program
differences remain included deliberately.

`semantic_id_v10` hashes that v9 base identity followed by the complete
`SavedProgramDescriptor`. The latter includes native helper bytes,
`charts[*].helper_digest`, `exact_helpers` byte digests and helper list ordering
derived from those digests. Stable callback tags consequently do **not** imply
identical v10 compiled content IDs across the proved byte-distinct helpers.
This is valid execution-content behavior, not a further numerical defect or a
reason to canonicalize native instruction streams. The indexed program archive
already stores its shared `source_identity` separately from recipe-specific
record/content identities.

Preserve legacy identity computation exactly. Native versions 5 through 9 use
their existing layouts and hash domain dispatch. Version 10 validates its
existing descriptor serialization before restoration; legacy root helper
version 1 must retain its original digest tag and saved bytes. A newly generated
version 2 helper can use its stable contract tag without relabeling or rewriting
old artifacts. No new CAS, canonical instruction codec, or restoration-time
symbolic regeneration is needed to maintain those boundaries.

For a future separately requested mathematical recipe identity, a small
explicit projection would retain the recipe version, ordered chart scientific
fields (dimension, derivative orders, positivity proofs, regularity, maximum
order and coefficient count) and semantic helper contract references. It would
exclude helper program bytes and transport digests, translate chart/exact-helper
references through their admitted owners, and sort/deduplicate set-valued
helper references by semantic keys rather than byte digests. Such a projection
would need its own version/domain and a clearly stated chart-order contract;
it would not establish equivalence under arbitrary chart reordering or algebraic
rewriting. Keep exact execution integrity alongside it. If any future change
instead alters persisted v10 content hashing, dispatch a new explicit schema
while preserving the historical v10 validation algorithm. This is a design
note, not work authorized or required by the present correction.

## Scoped v2 correction: independent validation

The corrected sources separate helper storage version 2 and semantic
`root_contract_v1_<coefficient_count>` tags from the unchanged saved-byte
digest. Version 1 helpers retain their original `root_program_<digest>` tags.
Admission continues deduplicating only byte-identical owners. V2 resolution
requires the selected caller's `ProgramScope`; it has no global same-arity
fallback. `MappingRequirements` captures only the strong helper owners needed
by native callback tags and re-enters that captured scope on subsequent native
mapping. Numeric callbacks retain their selected immutable owner themselves.

The independent probe was rebuilt against the actual corrected `program.rs`
using the same existing native rlibs and repeated all **120 fresh-process build
cases plus five v2 restore cases**. All eight registration/ring scenarios now
have one semantic tag per arity. Native byte digests still distinguish ordinary
and reversed registration for `n = 2, 4, 8, 16`, preserving their execution
identities. Native exact symbolic proofs and all rational evaluations pass.
Five further processes restore the **original pre-fix version 1 bytes**, after
reverse helper and ring registration, preserving each byte digest and legacy
tag and successfully resolving it without a v2 scope.

A second isolated executable includes the actual `dynamic/mod.rs` callback
implementation and actual `kernel/evaluator/mapping.rs`. It checks:

- Distinct same-tag owners, including a deliberately altered same-arity
  polynomial, remain isolated between selected caller scopes.
- Mapping without an owner scope fails; inner mapping and caught unwinding
  restore the outer selected owner, and leaving the outer scope restores the
  previous absence of an owner.
- Two simultaneous caller threads perform 40 total mappings using different
  same-tag owners, with each producing its own expected result.
- Captured mapping requirements support lazy 192-bit native `Float` mapping on
  a new thread after the original helper owners are dropped.
- A mapped numeric callback continues evaluating after all explicit helper,
  descriptor and mapping-requirement owners have been dropped.

These checks passed. An additional compile-fail probe rejects both `Send` and
`Sync` bounds for `ProgramPreparation`, specifically through its
`PhantomData<Rc<()>>`; its drop therefore cannot restore another thread's TLS.
The scope object itself remains transferable for caller-owned worker mapping.
Evidence is confined to `target/contour-helper-identity-probe/`:
`v2-results.log`, `v2/results.json`, `scope_probe.rs`, `guard-traits.log` and
`legacy-v1-results.log`. The expected caught panic printed in the scope log is
the unwind test, followed by the successful result.

Source review also checked descriptor retention in compilation jobs/completions,
scope entry in direct/dispatched/session compilation and binary restoration,
empty scope installation for descriptorless calls, exact-offset binding, and
retention through precision/conditioning caches and detached sector clones.
Native evaluator mapping still owns arithmetic; scope entry occurs during
preparation or binding, with no global registry lookup in ordinary sampling.

One additional ownerless-entry omission was reported to the runtime owner:
the supported native JSON v3 loader in `artifact/native.rs` lacked the empty
scope guard already present in binary restoration. The runtime owner added
`NativeProgramDescriptor::enter_optional(None)` at that loader's entry. The
coordinator confirmed the source correction and its real nested-load regression,
`descriptorless_native_json_cannot_inherit_a_callers_dynamic_owner`, in the
final core gate: **271 passed, 16 ignored, zero failures**. The regression checks
both artifact-digest policies and restoration of the caller's scope after the
descriptorless load rejects the callback. No finding from this bounded ownership
audit remains open. Broader contour scientific acceptance remains a separate
gate; the independent probes did not run shared Cargo or modify core files.
