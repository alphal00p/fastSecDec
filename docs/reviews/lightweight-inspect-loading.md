# Lightweight inspection and observable loading review

Independent source and execution-evidence review, 2026-10-07. Scope: the CLI
metadata inspection path and compact index, native evaluator restoration and lazy
conditioning, loading presentation, and generation-step elapsed clocks. The
reviewer did not run a large artifact load or another build concurrently with the
implementation owners.

The backend-preparation measurements and descriptions below are historical.
Current optional primary JIT persistence is documented in the
[primary-cache review](symjit-primary-cache.md); the metadata-only inspection
and caller-owned loading boundaries remain applicable.

## Inspection and identity

Previously, inspecting an artifact called the full native loader. That decoded
the saved programs and prepared their execution backends even when the report
only needed the evaluator sizes and layout already stored in JSON.

`Artifact::load_metadata` now reads the JSON sibling and checks the existing
artifact version and mathematical summary identity. Default inspection and
`--sector` use this path without opening the binary. A filesystem size query may
inspect its directory entry. `--deep`, including the implied deep path for
`--expressions`, explicitly restores native data. Missing older chart previews
never cause an implicit deep load. Dependency incompatibility is reported by the
metadata view; strict deep loading still rejects it.

The distinction is explicit in both human and JSON reports: metadata inspection
does **not** validate the binary. Exact coefficients and selected-sector content
IDs unavailable from the summary remain unavailable, rather than becoming zero
or being replaced by the parent ID. The optional producer-recorded preview index
and generation observations remain outside mathematical identity and are not
represented as checked against the unread binary. Deep overview statistics and
chart counts come directly from the restored native owners, including older
artifacts without an index.

The new versioned index reuses retained native chart data and Symbolica's
formatter. It stores at most three pre-subtraction monomial factors and sixteen
coordinate images per representative, with explicit omission counts. Each
native expression and rendered string is capped at 4096 bytes; the aggregate
producer budget is one MiB, with conservative nested-JSON indentation accounting
and a four-KiB envelope reserve. It excludes original source polynomials and
regular mapped bodies. These factors are not asserted to be the complete leading
behavior: their omitted regular body may vanish. Maps and positive measures are
identified as pre-subtraction data. Unknown index versions remain unavailable.
The reader's JSON parse cost still scales with the JSON file itself; the producer
budget is not a general bound on arbitrarily supplied JSON.

Reviewed paths: `crates/fastsecdec-cli/src/artifact.rs`,
`artifact/inspection.rs`, `inspect.rs`, and `inspect/{lightweight,overview,
presentation}.rs`. The review identified and the author corrected a deep-view
legacy fallback that still displayed missing indexed data, and strengthened the
index budget for enclosing pretty-printed JSON indentation.

## Native restoration and numerical ownership

The binary retains optimized exact Symbolica evaluator programs, not directly
runnable machine code. Loading does not regenerate sectors or repeat their
Horner/common-pair optimization. It still decodes and validates native data,
and prepares an eager evaluator or native SymJIT executable. Symbolica's native
JIT serialized representation is also not an instant machine-code cache: its
SymJIT load path reconstructs an application from MIR and prepares executable
code. No new cache format or alternate evaluator was introduced here.

`KernelSet::from_bytes_with_progress` adds caller-owned observations before
decoding, before restoration and after each restored sector, and after successful
native admission. Existing entry points delegate to it. All existing dependency,
digest, semantic identity, layout, branch, runtime-mass and metadata checks remain.
Cancellation at an observation boundary returns an error, never a partially
admitted `KernelSet`.

Roundoff-conditioning evaluators now map lazily through the existing native
`MappingRequirements` implementation. The three states are distinct: unrequested,
unsupported, and ready. Unsupported mappings are remembered, preserving the
existing validated-policy rescue/error behavior instead of retrying or inventing
a value. Distance-policy evaluation does not request this optional conditioning
owner. Worker clones retain the appropriate lazy/unsupported/ready state, with
independent evaluator buffers and timing counters. DD/arbitrary-precision
evaluation, complete Laurent vectors, accepted statistics and covariance are
unchanged. The first validated conditioning request pays the deferred mapping
cost; this is not a claim that all loading work was removed.

Reviewed paths: `crates/fastsecdec/src/kernel/artifact{.rs,/binary.rs,/native.rs}`,
the removed expression-only `artifact/legacy.rs`, `compilation.rs`, `complex.rs`, `evaluator.rs`, and
`evaluator/conditioning.rs`. Native API evidence comes from the linked Symbolica
`evaluate/backend.rs` and SymJIT serialization/restoration source. Existing
one-loop master and reduction providers are unaffected; no new algebra,
integration algorithm, graph representation or reference-value provider appears
in this change.

The strengthened no-expression-builder requirement removes the former small
exact-offset kernel and mass-schema evaluator builds as well. Current-format
load paths now contain no expression-to-evaluator construction. Native Atom
evaluation handles saved exact offsets once at a supplied physical point, or
directly at load completion for constant offsets. Saved real/complex layout
determines the numeric domain and flattened output ordering. Nonfinite vectors
retry in native double-double and arbitrary precision, as the former default
Distance helper did. A whole coefficient that numerically becomes zero without
being a literal zero is additionally confirmed using native arbitrary precision,
to guard against different intermediate arithmetic order. This is a bounded
numerical check, not an exact proof of cancellation. Singular or unsupported
evaluation remains an error; no failed value becomes an accepted zero.

Mass schema admission reuses `Atom::get_all_indeterminates(false)`. Its native
implementation visits sums, products and powers but retains entire functions,
avoiding confusion between symbolic callback tags and physical parameters.
Plain undeclared variables still fail load-time schema admission. Function
support, nested argument variables and arity are checked by the existing native
128-bit mass evaluation at binding, together with finite, real and nonzero mass
requirements. This deliberately moves those errors to the actual physical point.
The complete exact-offset vector and mass constraints succeed before sector
inputs, bound values or numerical identity change. The user's final instruction
also removes support for historical v1/v2 caches which stored expressions; the
remaining supported native-IR formats all satisfy the no-expression-builder
requirement in the reviewed source. The legacy expression constructor and sector
expression holder are removed. The common native/portable rejection control
requires v1/v2 compact and pretty encodings to fail after `Decoding`, without
restoration or completion. It passed in both final native and portable artifact
suites. This does
not imply removal of the metadata-only reader for JSON without an optional index.

Additional reviewed paths are `kernel/exact.rs`, `kernel/model_constraints.rs`
and their focused tests. The direct native API dispatches callbacks, tags,
constants and real/complex powers through Symbolica's `atom/core.rs` and
`evaluate/tree.rs`; there is no FastSecDec expression walker. New tests compare
native compiled and direct complex square-root/log/callback values, real and
complex intermediate overflow/underflow, physical-point zeros and poles,
serialized exact-offset rebinding, failed-rebind atomicity, and tagged or invalid
mass functions. All four focused controls passed; they also run in the passing
final native core library suite. The precision ladder uses the existing
`PrecisionClass::bits`: the removed exact kernel always used default Distance
settings, because per-sector stability overrides did not update that helper.

## CLI loading and terminal behavior

The CLI reuses its existing bounded dispatcher with one loading worker, leaving
the coordinator free to poll display and cancellation every 40 ms. No library
pool or integration loop is added. The latest loading snapshot is a small value
behind a short-held mutex. Metadata identity and dependency checks and reference
preflight precede opening the binary. Byte reading polls between eight-MiB chunks.
Native `Complete` is withheld until the outer JSON-to-kernel identity check also
passes; the final label is **Evaluators loaded**, not parameters bound or sampling
complete.

Loading has distinct metadata, byte reading, decoding, restoration and completion
states. Unknown work displays no fabricated fraction or ETA. Sector restoration
at N/N is labeled finalizing while remaining admission work runs. Memory and
process CPU sampling occur only when status is due; phase and cancellation changes
force publication. Cached TUI redraw uses the existing terminal-active recheck
under the stderr lock, preserving interrupt/panic cleanup. Plain and JSON modes
retain their normal output routing and do not enter terminal mode.

Individual allocation, decoder and native compilation operations remain atomic
to cooperative cancellation; a first cancellation can therefore wait for the
current native operation. The coordinator remains responsive and the existing
repeated-interrupt cleanup remains available. These changes do not establish why
the user's particular remote load was slow, and no remote process profile was
available for this review.

Reviewed paths: `crates/fastsecdec-cli/src/loading.rs`, `artifact.rs`, `main.rs`,
`generate/dispatch.rs`, `display.rs`, and `display/loading_view.rs`.

## Generation clocks

The CLI plan records caller-wall elapsed time at actual coarse-stage transitions,
keeps the active clock advancing on cached redraw, and freezes completed stages.
Repeated lower-level chart events neither reopen old stages nor manufacture
visited work. Saving begins at the caller's fresh elapsed time and completion
freezes the final duration. Future and skipped steps have no invented duration.
These presentation clocks include caller overhead and are separate from saved
native `GenerationTimings`; no numerical status schema or phase measurement was
redefined. An indivisible synchronous input call still cannot repaint itself.

## Validation evidence

The reviewer read the implementation and the following owner-run logs directly:

| Scope | Observed result | Ignored local evidence |
| --- | --- | --- |
| Generation plan/timer controls | 6 passed, 0.18 s | `output/generation-step-timers/focused-tests.log` |
| Final CLI binary unit suite | 68 passed, 1 existing benchmark ignored, 4.23 s | `output/inspect-loading-final-cli-tests.log` |
| Native artifact controls, including ordered restoration/cancellation | 16 passed, 0.50 s | `output/kernel-loading-review/native-artifacts.log` |
| Native runtime branch controls | 9 passed, 0.02 s | `output/kernel-loading-review/native-runtime-branches.log` |
| Lazy real/complex conditioning and unsupported-domain controls | 4 passed, 0.02 s | `output/kernel-loading-review/native-conditioning-tests.log` |
| Portable complete complex-vector round trip | 1 passed, 0.01 s | `output/kernel-loading-review/portable-roundtrip.log` |
| Portable weighted replay controls | 7 passed, 0.02 s | `output/kernel-loading-review/portable-weighted-replay.log` |
| Final CLI process controls | 11 passed across five targets, 3.45 s combined | `output/inspect-loading-final-cli-tests.log` |
| Direct native exact-offset and mass controls | 4 passed, 0.02 s combined | `output/kernel-loading-review/direct-{exact,mass}-tests.log` |
| Final native core library | 141 passed, 16 existing probes ignored, 1.08 s | `output/kernel-loading-review/core-lib-final.log` |
| Final native artifact integration controls | 10 passed, 0.06 s | `output/kernel-loading-review/native-kernel-artifacts-final.log` |
| Final portable artifact and weighted replay controls | 10 + 7 passed, 0.07 s combined | `output/kernel-loading-review/portable-final.log` |
| Portable direct-offset probe | Passed branches, range rescue, immutable bytes and failed-rebind controls | `output/kernel-loading-review/portable-direct-offsets.{rs,log}` |
| Final strict workspace/all-target Clippy | Passed, 10.43 s | `output/inspect-loading-final-clippy.log` |
| Final formatting and diff checks | Passed | `output/inspect-loading-final-fmt.log` and root/reviewer diff checks |

The CLI controls include missing binary and binary-directory cases, a synthetic
1.5-billion-byte sparse sibling, old/future index handling, incompatible native
dependencies, preserved artifact bytes/identity, and preflight/cancellation/final
admission ordering. The sparse sibling is an I/O-boundary control, not a measured
load of the user's artifact. Loading UI controls exercise widths 40/80/120,
unknown progress, byte and sector counters, finalization and cancellation labels.
Native tests compare restored numerical outputs and bytes, boundary cancellation,
lazy worker clones and supported/unsupported conditioning paths.

The process controls completed after an initial disk-space build interruption.
The selected-sector fixture was corrected to retain a numerical sector instead
of folding completely into an exact offset. Status assertions now explicitly
distinguish loading events, before runtime scope selection, from integration
events. Existing scope, vector, covariance and checkpoint checks remain intact;
deep native metadata assertions explicitly request `--deep`.
After expression-only loading was removed, sector-identity fixtures were
migrated to freshly generated native IR while retaining their metadata,
other-sector/offset independence, numerical-policy and layout assertions. A
separate native-IR control retains explicit absent chart-preview coverage.
The final CLI persistence fixture likewise uses newly generated eager native IR
and still checks the complete numerical vector, immutable bytes and identities.
Inspection explicitly verifies that v1/v2 binary siblings are rejected by deep
loading while JSON-only inspection remains available. The final 68-unit and
11-process run includes all direct-offset changes and loader removals.

Root's debug-binary measurements in
`output/light-inspect-review/debug-measurements.json` record a 17,268-byte
double-box JSON overview in 0.0217 s and a 125,650-byte, 304-sector cluster JSON
overview/selected-sector view in 0.0399/0.0392 s. The cluster binary's size changed
during earlier checks; its last inspected directory entry reported 651,788,288
bytes. It was not opened by those inspection commands. These are metadata-inspection observations,
not full binary loading or an asserted 1.4-GB benchmark. Comparison with an older
release binary is cross-profile and does not establish a precise speedup.

No remaining scientific or reuse source finding blocks the reviewed changes.
Native and portable execution confirm the stronger no-expression-builder
behavior, with direct-offset zero confirmation limited to those one-shot exact
offsets and native function-validation errors deliberately occurring at binding.
The independent subsystem review is accepted, including the final CLI rerun,
strict Clippy and formatting checks.

## Release delivery checks

The release CLI built successfully in 3 min 30 s using Rust 1.99 and Apple's
Clang linker (`output/inspect-loading-release-build.log`). Three fresh-process
inspection measurements per view are recorded in
`output/light-inspect-review/release-measurements.json`:

| JSON metadata view | Observed process wall time |
| --- | --- |
| Double-box overview, 17,268-byte JSON | 10.2–40.7 ms |
| Cluster overview, 304 sectors, 125,650-byte JSON | 11.0–21.1 ms |
| Cluster sector 0 | 10.7–20.9 ms |

The earlier release's default double-box inspection took 2.908 s because it
restored the binary. These are local observations, not a general speed guarantee.
The cluster binary remained unopened during these checks;
the large-artifact result establishes JSON-only inspection cost, not native
three-loop restoration performance.

The release also restored the existing on-shell ggHH double-box artifact and ran
QMC across all 30 sectors with 1 worker, 1024 points and 2 shifts: 61,440 admitted
points, complete vector/covariance output, and no numerical failures or cutoff
zeros. This intentionally used a loose absolute tolerance as a workflow smoke
test, not an accuracy benchmark. The invocation took 4.761 s, including 2.515 s
loading; status reported decoding, sector restoration and completion before
sampling. It recorded 60,576 f64 and 864 DoubleFloat classifications, no Arb or
Unstable points, zero conditioning checks and 240 primary matrix invocations.
The command, status and result are under
`output/light-inspect-review/full-load-smoke.*`.

A release CLI in a real macOS PTY was interrupted while displaying native program
decoding. It returned the cancellation error in 1.026 s, after the indivisible
native operation. The alternate screen was left, the cursor shown, and the
entire original terminal settings array restored, including canonical input and
echo. A wrapper session leader stayed alive while those settings were inspected,
avoiding macOS's hung-up slave behavior after session exit. The bounded probe
finished in 1.112 s and cleaned up only its own processes. Evidence is in
`output/light-inspect-review/pty-cancel.{json,log,raw}` and `pty_cancel.py`.
Release build, loading/sampling smoke and terminal cleanup delivery checks pass.
