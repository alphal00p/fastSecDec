# Caller-driven native recipe families

The native session is implemented; its focused gates are recorded below as they
complete. Dynamic production admission remains separate. The existing
single-recipe native generation and compilation sessions remain the direct path
for ordinary inputs.

## Ownership and proposed surface

`generation::RecipeFamily` owns a checked, canonical set of existing
`ProgramRecipe` values and an artifact default. An optional requested resident
recipe is a separate session setting. It must belong to the family, but need
not equal its artifact default. The CLI's private family request should use
this native type instead of adding a second public representation.

The family session reuses the shared `PreparedRecipeSet` and
`PreparedChartSource` pipeline: one physical input, runtime schema, support union,
geometry and monomial-extracted source per chart. Exact symmetry, subtraction
formulas, root owners and evaluator compilation remain recipe-local. It must
not implement the family by running the existing `GenerationSession` repeatedly.

The caller owns staging lifetime, output storage and each synchronous advance.
The library owns no worker pool. Frozen snapshots report completed native work;
pausing keeps completed units. Completed nonresident programs append to the
existing `ProgramArchiveWriter` and are released. The optional requested
resident `KernelSet` is returned without decoding or translating it again.

Resident assembly extracts and reuses the native output-layout, chart,
descriptor and check-program association logic in
`kernel/artifact/indexed/record_reader.rs`. A generation-side copy would risk
different rules for restored and already resident kernels. The runtime owner
agrees, particularly because the next checker schema carries namespace context
through the same projections. This extraction follows the late numerical-dual
request-lowering gate; it is not part of the current fixed/runtime admission.

## Native interface and independent review

`RecipeFamilySession<W: Write + Seek>` stores the input, caller-owned staging
path and writer without starting work. Builder methods select runtime inputs,
native evaluator settings and an optional resident recipe. `step(max_units,
observer)` advances bounded native units, returning pending/paused/complete.
An observer break pauses after successful current work; any native or storage
failure is terminal. `take_result` exposes only a completed catalogue, writer,
checked family request and optional existing resident owner. The caller retains
responsibility for flushing, filesystem durability and atomic publication.

The CLI now uses the same checked native `RecipeFamily`; its existing journal-v3
field spellings (`recipes`, `default_recipe`) remain unchanged. Resident choice
is neither an artifact default nor a mathematical identity input.

One consuming `ResidentAssembly` now serves selected-record loading and native
compiled-unit assembly. It sums exact expressions through native Atom arithmetic,
preserves complete complex Laurent layouts, remaps chart/check ownership and
adds output projections without recompiling earlier units. `append_unit` and
worker-file output share the existing native partitioner. A requested resident
recipe retains only its own original native records; their selected v2 identity
is verified against the completed family catalogue. The other recipes' bytes
and evaluators are not retained in that owner. Ordinary selected-reader behavior
already provided this portable-identity rule and remains unchanged.

The root and runtime agents independently reviewed the ownership and reuse
boundaries. They found no replacement CAS, graph type, scheduler, serialization
format or algebra implementation. The review required a final source-to-sector
ordering check, a native empty-fixed recipe preservation regression, truthful
live elapsed snapshots and explicit terminal behavior after a partial write.
Those controls are implemented. Elapsed observations include active time within
the current native step and exclude pauses. Complete nonresident units are
released immediately; native allocator/global Symbolica state and a browser's
MEMFS storage are not claimed to disappear from operating-system RSS.

The production `cargo check -p fastsecdec` passes. The focused native gates pass:

- Eight family tests cover checked requests and journal compatibility, both
  symbolic and numerical-dual generation, analytic complex pole/finite vectors,
  one-unit pauses, live elapsed observations, selected-only portable identity,
  no-resident file output, empty input and terminal initial/partial write failure.
- All 43 artifact tests pass, including a mixed exact/stochastic assembly whose
  later units introduce imaginary components and another Laurent order. Native
  resident and restored outputs agree; a noncanonical resident append order is
  rejected rather than assigned the canonical archive identity.
- All 15 recipe tests pass, including real fresh-process certificate restoration
  (one child-only test entry is intentionally ignored in ordinary invocation).
- All ten generation-program tests pass, including complete analytic higher
  endpoint vectors and real native eager/SymJIT evaluation of the tagged root
  requests. The distinct-face diagnostic regression also passes.

These are native focused gates on the current combined source. They do not
establish dynamic production admission, the final Python binding, actual Rust
WASM execution, or the later broad milestone checks.

## Actual Pyodide filesystem probe, 2026-10-09

The maintained bridge test harness already uses `pyodide.FS` to install files,
but source inspection alone does not establish the staging operations. An
isolated probe used the official `pyodide` npm distribution **314.0.4**, verified
against its published SHA-512 package integrity before unpacking under ignored
`target/contour-family-pyodide/`:

```text
sha512-fMexKS5P/s8iTjczevLUv/aWq9i5yaGvBTpBW6gM+nzDNelfPpREma/xdDK2AAGebAuKnBJk7fyVP6RBlz4btg==
```

The actual Pyodide runtime reports `sys.platform == "emscripten"` and Python
3.14.2. All of these operations passed in one temporary directory:

- Exclusive file creation, binary writes and file `fsync`.
- Selected-record reads using a nonzero seek offset.
- Keeping the previous manifest readable before replacement, then replacing it
  with `os.replace` after completing the pending file.
- Directory `fsync` and cleanup of the temporary tree.

The inspected mount is **MEMFS**, mounted at `/`. Consequently these operations
support using the existing synchronous filesystem interface; they do not
justify a new storage abstraction. They also do **not** establish native disk
durability or a disk-backed browser memory bound. Staged bytes remain memory
owned by the Pyodide instance, and `fsync` is not a durable external commit.

The probe script and JSON result are intentionally ignored artifacts:
`target/contour-family-filesystem-probe.mjs` and
`target/contour-family-filesystem-probe.json`. The stale preexisting `/tmp`
distribution contained dangling links and was not used for the successful
probe. No shared host checkout or running notebook was modified.

This validates the actual runtime filesystem through its Python/POSIX bridge.
Executing the final Rust family implementation in the published WASM wheel is
a separate required gate, still pending.

## Caller-dispatched batches (in progress, 2026-10-10)

`step_with_dispatch` extends the same native Work/Stage state machine used by
`step`; it does not introduce a second generation pipeline. The caller supplies
existing native geometry dispatch and a family-job dispatcher. Independent
source, recipe mapping, formula, and fused generation/compilation stages issue
at most the minimum of the requested width, remaining step units, and available
stage jobs. Ordered symmetry remains on the coordinator. Job payloads share
immutable prepared context through `Arc`; they do not clone the full chart list.

The dispatcher returns a bounded completion batch after joining issued work.
Every completion's private batch owner, recipe, stage and index is admitted
before any archive mutation. The native coordinator sorts admitted completions
into canonical order. Failed/incomplete/foreign/duplicate batches terminate the
session without exposing a completed archive. An observer pause joins and admits
the current successful batch before returning. Storage failure after a partial
write remains terminal. Residency is the requested resident recipe plus at most
one completed batch; nonresident evaluators drop before another batch is issued.

The old `step` path uses width one. Ordinary undeformed/singleton callers retain
their existing direct generation path. Native preparation exposes only a narrow
geometry-dispatch hook around its existing implementation and codec. No new
algebra, graph transformation, numerical evaluator or library-owned pool is added.
Root and the generation owner independently reviewed this design and its source;
they found no ownership/reuse blocker. Production and all-target compilation
checks pass. All 12 native family tests pass (0.65 seconds). They execute actual
caller threads at widths 1/2/4, reverse completion returns, and compare complete
archive identities and all four recipe values in both generation modes. Separate
controls join a paused batch and refuse missing, foreign, or duplicate completion
IDs before publication. Existing exact-empty, resident lifetime and partial-write
controls also pass. The actual CLI family and cancellation controls pass 2/2;
the complete CLI regression passes 169 tests with eight explicit ignores.
These tests do not establish a performance improvement or browser-thread
availability.


Selected portable archives preserve semantic identity, not a universal byte
ordering. `ProgramResidentAssembly` appends original per-unit records to its
selected-only writer; `SelectedProgramReader::selected_bytes` copies records in
the canonical directory order produced by `finish_layout`. The recipe identity
uses that logical record order and excludes physical offsets. Thus two retained
owners can have different immutable transport bytes with the same identity.
Selection performs no Symbolica re-encoding. The Python lifetime regression now
checks byte stability within its detached owner, re-restores those bytes, and
compares actual complete QMC estimates against the resident recipe. A cross-owner
byte-equality assertion would incorrectly require canonical physical record order.
