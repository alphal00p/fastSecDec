# Native formal-symbol hygiene

Source/API review and accepted disconnected control, 2026-10-05. The public
native APIs provide the required admission without a dependency patch, global
reset, per-generation namespace or additional global registry. The proposed
minimal check is **explicit empty metadata through `SymbolBuilder`, followed
by `Symbol::is_exportable()`**. The compiled probe passed its authorized bounded
small process and [independent outcome review](native-symbol-hygiene-independent.md).
No production code changes here.

## Concrete gap and existing native owners

The test-only `generation/subtraction/series_first/named.rs::fresh` restarts
local `f0`, `a0`, ... counters within `fastsecdec::named_regular` and avoids
symbols already present in its input/coordinate/regulator set. This permits
stable interned handles across independent local jobs. However, bare `symbol!`
uses the default native builder branch: `State::get_symbol` returns an existing
handle without validating its attributes or hooks. A foreign reserved-name
registration absent from the current input can therefore supply semantics the
formal coefficient protocol did not request. This is a source-proven gap;
no failure of the completed isolated scientific readers is inferred.

One detail refines the earlier
[production-slices proposal](native-named-production-slices.md): explicit empty
attributes alone are insufficient to exclude existing hooks. Native
`custom_function_matches` returns true whenever the **requested** hook is
`None`; that branch retains an already registered hook. The composite check
must use the public native `Symbol::is_exportable` predicate as well.

Inspected native source in
`DO_NOT_PUSH_FOR_REFERENCE_ONLY/worktrees/symbolica/`:

| Native source/API | Evidence and consequence |
| --- | --- |
| `src/atom.rs:1187`, `SymbolBuilder::build_with_state` | Default empty builder takes `get_symbol`; explicit `.with_attributes(&[])` selects native metadata comparison |
| `src/state.rs:761`, `get_symbol` | Existing registration is returned unchanged |
| `src/state.rs:813`, `get_symbol_with_attributes` | Compares all nine attributes, tags, aliases and user data against the requested empty metadata |
| `src/state.rs:154`, `custom_function_matches` | Requested absent hook accepts an existing hook; builder alone cannot certify hook absence |
| `src/atom.rs:1693`, `Symbol::is_exportable` | Requires normalization, derivative, series, print and evaluation hooks all absent |
| `src/atom.rs:1329`, `Symbol::get_symbol` | Read-only native name lookup permits existing input-collision skipping before strict admission |
| `src/state.rs:720`, `State::symbol_iter` | Public observation of interned names for the bounded reserved namespace |
| `src/atom/alias.rs:30,158,178` | `AliasedAtom` owns its local definition map; native evaluator builders consume those definitions |

Native tests `state::tests::custom_function_definition_keys_are_opt_in` cover
keyed re-registration and public unkeyed callback redefinition errors;
`atom::tests::{user_data,flat_symbol_encoding_roundtrip}` cover native metadata.
These source/tests support the API choice but do not replace the new focused
empty-request-plus-existing-hook control. Public builders do not expose the
private Python definition keys, and no such keys or callback comparison are
needed here.

## Minimal integration proposal

Keep `Coefficients`' existing local counters, occupied set, exact-body maps and
native alias ownership. In `fresh`, before constructing a function or variable:

1. Compute the next fixed reserved name with checked counter arithmetic. Use
   `Symbol::get_symbol` to skip an already occupied input handle, preserving
   existing declared-symbol collision behavior.
2. Call `SymbolBuilder::new(wrap_symbol!(name)).with_attributes(&[]).build()`.
   Propagate its native metadata conflict rather than using the panicking macro.
3. Require `candidate.is_exportable()`. Otherwise return a typed reserved-name
   conflict before any normalization, derivative, series or evaluator sees it.
4. Insert the admitted handle in the existing local occupied set and continue
   with the existing body/alias construction.

Prefer a typed `GenerationError::ReservedSymbolConflict` carrying the candidate
name and either the native metadata error or the existing-hook category. This
is a caller process-state conflict, not scalelessness, zero, cancellation or an
automatic physical fallback. Fail on an incompatible foreign registration;
an unbounded search through more global names is unnecessary. Local names that
actually occur in the input continue to be skipped as before.

The builder's native comparison executes under the registry lock. Existing
symbol metadata is append-only: an unkeyed attempt to attach a new callback
after plain admission fails, so this design needs no second global lock or
homegrown ownership table. Dropping a local coefficient job drops its bodies
and caches; interned names persist. A larger new request high-water mark can
add names, while repeating the same fixed workload should reuse them. No claim
that total process RSS or unrelated namespaces never grow is appropriate.

## Disconnected probe and limits of its evidence

Owned ignored source is `output/probes/native_symbol_hygiene/`:

* `admission.rs` is the narrow native builder/predicate composition with typed
  `MetadataConflict` and `ExistingNativeHook` errors.
* `main.rs` pre-registers reserved `f0`–`f8` with all nine attributes; `a0`–`a4`
  with each of the five hook kinds; and three further names with tags, aliases
  and user data. It asserts the current bare-builder reuse, the five
  empty-metadata hook admissions, and rejection by the complete strict check.
  Callback counters must remain zero. Two input-owned conflicting handles are
  skipped by native lookup before admission; an unused foreign hook still
  fails. No poisoned Atom is constructed.
* The same process creates two live two-output native alias vectors with the
  same interned handles and different bodies, builds independent native
  evaluators and checks exact small-integer outputs `[7,-1]` versus `[18,6]`.
  It performs 64 repeated local jobs, checks handle/map equality and observes
  only the reserved namespace via `State::symbol_iter`. Native program values
  remain distinct after another vector is built and after its local map is
  dropped. A late callback-registration attempt must fail.
* `build.sh` archives the source and linked native feature identity, pins the
  existing Rust 1.98.1 toolchain inside the Nix environment, compiles directly
  on CPU9, and records all available-rlib pre/postchecks. It never executes the
  probe or starts Cargo. Failed builds remain separate evidence.
  `prepare.sh`/`run.sh` separately freeze and supervise one process under
  180 seconds plus five seconds grace, 30 GiB address space and CPU8, requiring
  the independent preflight marker before starting.

The native program boundary mirrors `kernel/program.rs`: register one shared
flat map per vector through `Atom::evaluator_multiple(...).add_aliases(...)`.
There is no second alias evaluator, derivative implementation or global body
map. Probe hooks capture only counters and are deliberately hostile fixtures
in a fresh process; the proposed production path installs none.

These controls exercise native symbol admission and local alias ownership,
not repeated full production generation. After promotion, the public opt-in
generation tests still need repeated identical jobs, two simultaneously live
generated vectors and the declared-input collision cases through the real
allocator. Scientific full-vector, cold reload and original-oracle gates remain
separate. This isolated control does not exercise the production allocator.

## Frozen build and bounded control

`output/diagnostics/native-symbol-hygiene-build-1` linked successfully and remains
unchanged. Before any runtime, the focused occupied-input skip control and
separate preparation/runner scripts were added in fresh build two. Independent
HEP source review accepts this final source; no native source or dependency
change was needed. The build has 22 frozen entries and all 474 available-rlib
postchecks pass. That inventory is a superset, not a claim every rlib was linked.

| Build-two item | SHA-256 |
| --- | --- |
| Probe main | `4ea1395dcc3a67f6c8d4cc221cab91c1c1fc581f07ef9e76f019f5fd3f9229c3` |
| Strict admission | `8384e266353471368ccd4736bf6b7e904f03aef806c4d79a5496e6e2e1b217a4` |
| Native Symbolica rlib | `01601b6df1747703693fdbf78818f9bf2bb74f4e4b1a888a5971b678cc60b61a` |
| Executable | `c8ef6d8697041fb903a7d27211b3d2ebbac78c692cfda757f105b0db044b89c5` |
| Build frozen manifest | `e2a29fd047c6803367fa4ff8f7c0dfb9f80143f89290efef209e388198ee50be` |

The fresh prepared attempt is `output/diagnostics/native-symbol-hygiene-1`,
with 27 frozen entries and manifest SHA-256
`2decad748adab3ca47ece4b55535ba91be8e2b5ddd16ef3ab5a004d4c225ae60`.
Its invocation binds the absolute limiter/affinity tools, original build and
180-second/five-second-grace, 30-GiB-address-space, CPU8 bounds. Preparation
started no Symbolica instance. HEP accepted all 22 build and 27 attempt hashes,
source/archive equality, native identities and the fresh bounded invocation.

The one authorized control process exited zero and was reaped after
0.007197841 seconds, with 6,152 KiB peak child RSS and no timeout. The wrapper
exited zero and all 27 posthashes pass. Its result reports all 17 strict
rejections, five empty-request hook admissions caught by `is_exportable`, zero
callback calls, two occupied-input skips, 64 jobs with stable reserved names,
distinct simultaneous alias bodies/evaluators, native program ownership after
local-map drop and rejection of late hook registration. HEP independently
accepts the result and all 27 posthashes: 12 metadata conflicts and five hook
conflicts are rejected through native APIs. Result SHA-256 is
`e52dc33c367e98595f926439690bf316eaeae42d5631aa62a09b1c578d010015`;
independent-review SHA-256 is
`fbc10d6ed3ee77e7e0d5ea2b96dc92cb0c19344aa61a74a4f72b9dcc1426a218`.
Production allocator controls and broader performance acceptance remain
separate from this small observation.
