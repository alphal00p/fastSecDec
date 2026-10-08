# Native evaluator cache loading review

Reviewed on 2026-10-08 against Symbolica
`1deccb8538ccb91dc2c1e58fc0a2e900d2276bf4` and registry SymJIT `2.26.4`.
This review distinguishes the evaluator instruction program, SymJIT's saved
application, and executable machine code. No dependency or artifact-format
change was made for this investigation.

## What the native owners currently save

Symbolica's `JITCompiledEvaluator<T>` serializes its external-function
descriptors, saved SymJIT application bytes, and compilation settings. Its
deserializer restores callback definitions, calls `Application::load`, and seals
the returned application. Non-inlined evaluator bodies are compiled while
reconstructing the callback registry. The field named `compressed_ir` is therefore
not evidence that machine code is present or absent; that is decided by the
SymJIT application codec. See the pinned
[Symbolica backend](https://github.com/ValentinHirschi/symbolica/blob/1deccb8538ccb91dc2c1e58fc0a2e900d2276bf4/src/evaluate/backend.rs#L715).

In SymJIT 2.26.4, `Application::save` writes the program, a presence mask, MIR,
and real-coordinate information. It does not call `MachineCode::save`.
`Application::load` reads those objects and unconditionally calls `with_mir`,
which emits native code through `compile_ty`; saved SIMD/fast flags request
their preparation afterward. This happens on the same architecture too. See
the registry source's `runnable.rs`, lines 767–843, available in the
[published source](https://docs.rs/crate/symjit/2.26.4/source/src/symjit/runnable.rs).

The native eager `ExpressionEvaluator<T>` serializer stores its stack,
dimensions, instruction/result indices, external functions, and settings.
Deserialization restores those fields directly. External callback descriptors
resolve implementations through the numerical domain, and reusable callback
stacks start empty. It does not build an expression evaluator from an Atom.
See the pinned [eager codec](https://github.com/ValentinHirschi/symbolica/blob/1deccb8538ccb91dc2c1e58fc0a2e900d2276bf4/src/evaluate/evaluator.rs#L109)
and [external-function codec](https://github.com/ValentinHirschi/symbolica/blob/1deccb8538ccb91dc2c1e58fc0a2e900d2276bf4/src/evaluate/external.rs#L21).

## Historical and newer upstream source

The author's machine-code serialization description has a historical basis.
Branch `v215`, commit `889e6ea1e07ce821b82fd131db8618c97c05e0b1`, writes and
restores individual `MachineCode` objects in application format version 1.
That is a different codec from the one FastSecDec currently uses. This source
inspection establishes historical machine-code transport, not verified historical
cross-architecture fallback. See the [historical application codec](https://github.com/siravan/symjit/blob/889e6ea1e07ce821b82fd131db8618c97c05e0b1/rust/runnable.rs).

The public changelog mentions MIR save/load infrastructure under version 2.16.
At review time, [docs.rs/latest](https://docs.rs/crate/symjit/latest) resolves to
2.26.4. The development `v227` branch, commit
`83ce2fe185c40956a1ca7f34ab52f7ca1ccd149b`, declares version 2.27.0 and also
loads MIR through `with_mir`; its application format is version 4. An upgrade
to that inspected source would not supply the requested same-architecture
machine-code reuse. See the [development codec](https://github.com/siravan/symjit/blob/83ce2fe185c40956a1ca7f34ab52f7ca1ccd149b/rust/runnable.rs)
and [changelog](https://github.com/siravan/symjit/blob/83ce2fe185c40956a1ca7f34ab52f7ca1ccd149b/CHANGELOG.md).

## Why the lower-level machine codec is insufficient

Current `MachineCode` still implements `Storage`. It maps saved bytes into
executable memory and retains that allocation through an `Arc`. Its header
records an architecture category, lane count, and memory dimensions. A foreign
architecture installs an invalid-call stub; this owner does not recompile MIR.
It does not provide the application-level OS/ABI/CPU-feature and relocation
admission needed here.

This is a concrete limitation for our evaluators: the ARM generator's
`add_func` embeds scalar/SIMD function pointers and callback-environment pointers
in the code. Reloading those bytes in another process cannot reuse those
addresses. Symbolica currently fixes their lifetime and ownership by rebuilding
the callback registry before native compilation. FastSecDec must not copy this
machinery or introduce its own raw executable-memory restoration. See the
[machine owner](https://docs.rs/crate/symjit/2.26.4/source/src/symjit/machine.rs)
and [ARM function-table emission](https://docs.rs/crate/symjit/2.26.4/source/src/symjit/arm/mod.rs).

## Focused native evidence

Two ignored-output Rust probes used existing debug dependency artifacts on
macOS/aarch64, with the Apple Clang linker. No full integral, three-loop input,
or sector generation was run. Sources and logs are in
`output/symjit-persistence-research/` and are intentionally untracked.

The SymJIT probe compiles `x*x + sin(x)`, saves the application and the dedicated
machine object, removes the live machine object, and saves the application
again. The application remains 197 bytes; only the presence-mask byte changes
from 1 to 0. Its payload contains neither the 184-byte raw code nor the machine
codec header. The dedicated machine payload is 216 bytes. Loading either
application produces a native evaluator and the same value
`0.30990395925452296` at `x=0.25`. Together with the inspected load path, this
confirms that the current application codec reconstructs native code from MIR.

The Symbolica probe writes and reads in separate fresh processes. It verifies
JIT/eager agreement at four points and a JIT clone evaluated on another thread.
These measurements cover this tiny real expression only, with no registered
external functions:

| Operation | One observed duration |
|---|---:|
| JIT compilation from an already-built exact evaluator | 3.421500 ms |
| Native JIT serde decode and application restoration | 1.781666 ms |
| Native eager serde decode | 0.248209 ms |

The JIT serde payload is 233 bytes, its nested application 210 bytes, and the
eager payload 116 bytes. Timings exclude file reads and expression construction;
they are functionality evidence, not a benchmark or a three-loop startup
speedup claim. Cross-architecture execution and callback-heavy cache reuse were
not tested.

## Smallest missing operation and FastSecDec boundary

The missing operation belongs in the native owners: a versioned SymJIT
application cache that retains machine code plus relocation information and
portable MIR, admits compatible OS/ABI/ISA code, rebinds function/environment
references, and falls back to MIR compilation when the cached code is unsuitable.
It should expose whether code was reused or recompiled so callers can report and
test the distinction. Symbolica should preserve that native cache, including
non-inlined evaluator bodies, through its existing typed JIT serialization.

Once that exists, FastSecDec can store the primary native evaluator alongside
the exact instruction program needed by DoubleFloat, Arb and eager/portable
consumers. Backend cache bytes must have their own version and integrity
binding, while scientific identity remains tied to the exact program. A cache
miss must not require symbolic substitution, Horner optimization, or CPE.

Using current Symbolica JIT serialization could already skip exact-program
translation into SymJIT, and saving a mapped eager evaluator could skip its
coefficient conversion. Neither change establishes machine-code reuse. This
milestone therefore makes no new JIT cache-format claim; startup improvements
remain separately measurable reductions in redundant loading work.

## Independent startup-change source review

The accompanying FastSecDec loading changes were reviewed independently of their
implementation. Retained encoded program buffers remain aligned with their
decoded native programs, preserve their original bytes, and avoid a second
serialization. The borrowed binary envelope keeps the existing wire layout and
native Symbolica `StateMap` decoding.

With optional validation disabled, restored coordinate metadata uses the saved
images and Jacobian while constructing the same determinant and measure-power
fields as `coordinates_from_parts`. Dimensions, coordinate-domain associations,
numerical policy, evaluator layouts and runtime constraints remain checked.
Explicit validation additionally performs the existing content hashes,
polynomial-support admission, and geometry/image proofs. It never adds threshold
certification. CLI metadata-only inspection stays metadata-only even when its
identity validation is requested; loaded and validated states are reported
separately.

No new CAS, instruction codec, machine-code restoration, callback implementation
or library-owned worker pool was introduced. The reviewed source has no blocking
scientific-equivalence or native-ownership finding.

## Representative release loading measurements

The release binary was measured on macOS/arm64 against the existing on-shell
double-box artifact: 30 sectors, numerical-dual Taylor subtraction, SymJIT O2,
10 Horner iterations and 1000 CPE rounds. Its binary payload is 16,124,198 bytes
(15.4 MiB) and metadata is 17,268 bytes. The content identity is
`b2e444ab75d06d4fcbc54659c7da7740735f8b7d0058e2a57c163b6fa8aef515`.
No generation or integration sampling was performed.

Each command ran in a fresh process with a warm filesystem cache. One initial
run per mode was excluded, then three measured runs per mode were interleaved,
alternating the default/validated order. All commands used `--json --plain
inspect output/on-shell-generation/gghh_double_box.fsd`; the additional options
are shown below. The reported values are medians, with the loading range in
parentheses.

| Additional options | Loader time | Process wall time | Process CPU time | Peak RSS |
|---|---:|---:|---:|---:|
| None: metadata only | 0.119 ms (0.116–0.148 ms) | 9.74 ms | 6.75 ms | 6.42 MiB |
| `--deep` | 1.640 s (1.636–1.647 s) | 2.140 s | 2.131 s | 325 MiB |
| `--deep --validate-artifact` | 2.380 s (2.368–2.402 s) | 2.876 s | 2.870 s | 323 MiB |

`loading_seconds` is the CLI's loading span. Process wall time includes startup,
inspection construction, JSON formatting and writing the output: deep inspection
wrote about 11.5 MB, metadata-only about 17.6 kB. CPU is child user plus system
CPU from `wait4`, including its threads. Peak RSS is the same child's native
`ru_maxrss` (bytes on macOS). These runs do not establish a memory reduction:
the deep-mode RSS ranges overlap (319–330 MiB and 319–328 MiB).

Skipping optional validation reduced median loading time by 31.1%, or 1.45×, for
this artifact. This compares the two new loader modes, not the old release
against the new release, so it does not separately quantify avoided payload
copies or re-encoding. The remaining native SymJIT compilation is still present.
The user's 1.4 GB three-loop artifact was not available for this measurement;
there is no extrapolated three-loop timing claim.

All twelve processes, including warmups, exited successfully. The harness
checked identical content identities, sector counts, orders, components,
dimensions and evaluator statistics across modes, and the expected independent
loaded/validated flags. Full outputs, the benchmark harness and binary/artifact
SHA-256 identities are retained only in ignored
`output/symjit-persistence-research/loading-benchmark/` and its parent directory.

## Final validation evidence

The inspected milestone logs record 146 passing core unit tests (16 ignored),
69 passing CLI unit tests (1 ignored), both inspection process tests, strict
workspace Clippy, and a successful optimized release build. The standalone
portable host suite passed all 72 tests; this is eager/portable host evidence,
not a browser or WASM execution claim. The focused loader tests additionally
cover enabled/disabled identity validation, malformed layouts, native value
parity and restored projective/signed-orthant metadata. Test and build logs remain
untracked in `output/optional-validation-review/`.
