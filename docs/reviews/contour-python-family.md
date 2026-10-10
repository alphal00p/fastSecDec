# Thin HEPKit recipe-family owner

The new binding delegates to native `RecipeFamily`, `RecipeFamilySession`,
`RecipeFamilySnapshot`, `ProgramArchiveReader` and the native resident assembly.
It does not implement recipe scheduling, polynomial operations, graph input
conversion, mathematical identities, coefficient combination or covariance.
`Integral` continues to use its existing HEPKit diagram/kinematics and native
parametrization path. Ordinary singleton generation is unchanged.

```python
work = integral.generation_family_session(
    ["off", "fixed"], default_recipe="off", resident_recipe="fixed"
)
while not work.complete:
    work.step(1, observer=show_progress)
archive = work.result
kernels = archive.select("fixed")
```

Construction is inert, including filesystem effects. The first explicit step
parametrizes the native input and allocates caller-owned temporary storage.
Subsequent units delegate directly to the native family state machine. Frozen
snapshots expose the current recipe, source/persistence counters, completed
native units and the existing generation snapshot. Observer `False` and Python
interruptions pause at native boundaries without discarding completed work.
Runtime generation failures remain terminal, with no partial completed archive.

The generation request's artifact default and the optional retained resident
are independent. Repeated `result` access returns the same completed owner;
repeated selection of the retained resident returns the same Python Kernels
object, backed by its original native evaluators. Selecting another recipe
uses the existing selective native reader and does not accumulate a cache of
all recipe evaluators. Imported/selected kernels own their native programs and
metadata independently of archive temporary storage.

`to_bytes`, `from_bytes`, `save` and `load` expose the existing standalone native
binary archive, not a Python format and not the CLI JSON manifest. `save`
copies into a sibling tempfile, flushes/fsyncs it and atomically replaces the
destination. Import makes an independent temporary copy; metadata admission
uses the existing native reader. Raw archive bytes deliberately contain no
caller default, so imports report `default_recipe=None` unless supplied an
explicit, native-checked override. `select(recipe)` is always explicit.

The only added dependency is `tempfile = "3"`, already resolved in the leaf
lockfile and used in the native ecosystem. All existing lockfile versions and
sources remain unchanged. The native core and CLI retain no Python dependency.

## Verification scope

Native family gates passed 8/8 and archive gates 43/43 before this binding was
added. Root independently reviewed storage/resident ownership and identified
a multi-unit timing bug; the bridge now uses one active-time baseline through
each `step`, normalizes the completed result snapshot and excludes pauses.

The new Python tests cover inert construction, no staging files before work,
native request admission, immutable snapshots, pause/KeyboardInterrupt,
multi-unit monotonic timing, both generation modes, resident reuse, explicit
no-resident behavior, save/reload and detached lifetimes, and fixed/off results
against the analytic massive triangle. **Execution remains pending a rebuilt
private Community host with the new registration/reexports and stubs.** Leaf
Rust/stub-generation compile evidence is recorded separately once complete.

The final live binding sources passed `cargo check --locked` with
`python_stubgen`, followed by binding-only `cargo clippy --locked --all-targets
--no-deps -- -D warnings` with that feature (2026-10-10). The separate
`bindings/python/target` was used with two Cargo jobs and the existing Python
3.11 configuration; no Community installation or running release executable
was replaced. Native runtime drafts still emitted their own pending-work
warnings; the binding itself passed strict Clippy. Rust formatting and Python
test syntax checks also passed. This is compile evidence, not a claim that
the pending Python host tests executed.

The four public recipe labels are capability selectors. Native dynamic
admission now passes its public analytic controls. The complete current leaf
binding also passes a fresh `python_stubgen` check and all-target binding-only
strict Clippy on Symbolica/Numerica `516beb37`; installed-host execution remains
separate. New test cases select
both dynamic recipes, restore saved owners, run native pilots, compare the
triangle with its analytic value, resume after a validation-policy change and
reject changed-strength checkpoints in both generation modes. Those Python
cases still await the refreshed host. Actual Rust WASM family execution also
remains pending; the earlier official Pyodide filesystem probe establishes
available MEMFS operations, not native disk/RSS bounds.
