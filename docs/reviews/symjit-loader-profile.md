# Historical SymJIT loader profile

Reviewed 2026-10-10. The earlier attribution of whole-artifact startup to
bytecode translation was too broad. The inspected FastSecDec v8 artifact saves
**exact Symbolica evaluators**, and loading rebuilds their SymJIT programs.
Native saved-application restoration avoids part of that work. These measurements
do not establish that executable-code persistence is the next necessary change.

## Artifact, owners and scope

The available `output/gghh_triple_box_bis.fsd.dat` is 3,768,834,294 bytes with
304 sectors. Its metadata declares 3,404,231,368 exact-program bytes and
4,211,405,064 SymJIT IR bytes. The latter is generation-time statistics, **not
proof that these JIT bytes are stored in the artifact**. This is not an identified
copy of the author's quoted 1.4 GB case. No total machine-code size is inferred.

The data SHA-256 is
`48a67d4268df603e8f77b41105743bcdf60c7c066a5a14e3cf89221d41758ce7`;
the manifest SHA-256 is
`6a5d564cf252f22bea888d0e896c51eb32f7ddded3842d980c2a1d4b42224906`.
The native kernel identity is
`d895ef335589ae7c887a863b4688727baf068ac070a325d9628d07a8596c2f9c`.

The probe uses matching historical owner artifacts: Symbolica
`1deccb8538ccb91dc2c1e58fc0a2e900d2276bf4`, Numerica 3.0.1 and SymJIT
**2.26.4**, verified by native version code 22604. A private copy of FastSecDec
`c28871fe164a6963462427fe4353b06211148a24` adds passive timing and extracts
unchanged exact-program bytes *after native StateMap/payload decoding*. There is
no replacement codec. Rust 1.97.1 under `nix-shell`, optimized native dependencies,
and an opt-level 3/thin-LTO wrapper run on Linux x86-64, AMD EPYC 9754.
Concurrent contour work was permitted; this is a diagnostic profile, not an
idle-host benchmark. No generation, evaluation, sampling or parameter binding ran.
The original data and protected `target/release/fastsecdec` were unchanged.

Three positions, 0, 152 and 303, were selected before loading. They are similar
programs, not a claim to cover the largest or median sector. Each has 28 inputs,
two complex outputs and two external-function descriptors. Every representative
process exited successfully with an empty owned process group. Per-run bounds
were 600 seconds and 20 GB; extraction used 120 seconds. Ignored provenance,
source, exact linker inputs, hashes and raw records live in
`target/symjit-loader-profile/`, particularly `representative-result.json`,
`artifact-inventory.json`, `probe.rs` and `runs/`.

## Actual native code sizes and timings

Sizes are bytes returned by native `Application::measure`. Times exclude the
separate `/proc/self/smaps_rollup` memory observations.

| Sector | Saved SymJIT IR | Scalar machine code | SIMD machine code | Exact decode (s) | Exact evaluator → complete JIT (s) | Native `Application::load` (s) |
|---:|---:|---:|---:|---:|---:|---:|
| 0 | 13,813,435 | 23,907,968 | 42,329,920 | 0.1362 | 3.2623 | 1.4737 |
| 152 | 13,813,623 | 23,908,048 | 42,328,064 | 0.1308 | 3.2475 | 1.4813 |
| 303 | 13,790,817 | 23,855,824 | 42,250,368 | 0.1302 | 3.3056 | 1.4672 |

Combined scalar and SIMD code is **4.79–4.80 times** saved IR for these sectors;
fast-kernel size is zero. The saved mask already requests SIMD restoration, so
the measured `Application::load` includes both scalar and SIMD preparation.
These are actual three-sector sizes, not summed/restored sizes for all 304.

Additional native boundaries, observed across the same three processes:

| Operation | Observed range |
|---|---:|
| Native JIT serde field decode | 18.19–18.33 ms |
| Callback/subevaluator registry construction | 10.3–14.6 µs |
| Program native Storage encode **and** decode replay | 0.250–0.261 ms |
| MIR native Storage encode **and** decode replay | 183.7–184.7 ms |
| `with_mir`: complexification, scalar emission and bytecode construction | 680.7–713.2 ms |
| Explicit SIMD preparation on the replayed application | 571.0–578.5 ms |
| Native sealing | 91.8–93.4 µs |
| Optional eager exact-IR clone and coefficient mapping | 23.7–44.0 ms |
| Optional error-tracking coefficient mapping | 0.380–0.404 ms |

The replay roundtrips include encoding and are **not pure decode measurements**.
The `with_mir` category is **not pure machine-code emission**: this historical
complex path clones/recomplexifies MIR and creates bytecode as well. The explicit
SIMD replay is a separate measurement of work already included in the saved-mask
load. Categories from these different replays must not be added into a purported
single end-to-end load. Error-tracking creation is an optional allocation probe;
FastSecDec creates its conditioning evaluator lazily, so this is not charged as
mandatory startup. Parameter binding and first-use numerical scratch remain
unmeasured.

Whole-file extraction with optional validation disabled measured 13.211 s for
file reading, 0.020 s for native State import and 9.852 s for native payload
decode. The envelope check was 0.364 ms. Extraction deliberately returns a
cancelled sentinel before exact-program restoration/JIT compilation; it is not
a complete 304-sector load. Prior hashing warmed the filesystem cache. The
whole process closed after 24.814 s with a native high-water RSS of 8.050 GB.
A separately bounded optional-validation probe is retained independently; its
outcome is not needed for the representative JIT attribution above.

## Memory interpretation

Native child `ru_maxrss` was 513.0–526.7 MB for the representative processes.
Stage RSS/PSS comes from Linux `smaps_rollup`; process-group monitoring also
records sampled RSS. These processes intentionally retain exact programs,
encoded buffers, decoded JIT fields and later optional eager/error-tracking
objects across sequential replays. Their peaks are **not** minimal resident
memory of one production kernel and must not be summed to estimate full-archive
memory. The discrepancy between sampled peaks and native high-water marks is
retained in the raw evidence. No `memory_states` instrumentation was necessary.

## Interpretation and native reuse

The matching historical FastSecDec loader decodes the native exact evaluator
and invokes `jit_compile`; it does not deserialize a saved `JITCompiledEvaluator`.
The larger 3.25–3.31 s operation therefore includes exact-IR export, coefficient
conversion, translation/optimization, native code construction, application
serialization and sealing. It is not evidence that a saved MIR-to-code pass
alone takes that long. Native application loading still costs 1.47–1.48 s here,
so it is not free either. Callback reconstruction is negligible in this fixture;
callback-heavy or retained subevaluator cases require separate profiling.

The first concrete cache experiment should reuse **existing native Symbolica
JIT serialization**, retaining exact IR for higher precision/portable consumers,
and measure its full-file time/memory tradeoff. The three-sector experiment
shows avoided work, not an accepted artifact-format implementation or promised
whole-artifact speedup. Raw executable-code persistence remains contingent on
further attribution and must preserve native ABI, relocation and callback
ownership. There is no dependency-defect or upstream-fix claim in this profile.

The author's `.o` proposal is also source-supported: upstream development
[`v228` at 4b337fb](https://github.com/siravan/symjit/blob/4b337fb50318313be61bc6e11c9df6712d7f76be/rust/runnable.rs)
contains feature-gated `write_obj`/`write_obj_for` and relocatable ELF/Mach-O
output with a header. That API was not built or benchmarked here, and no `.so`
loading or startup benefit is claimed. These historical 2.26.4 measurements do
not contradict or validate the implementation/performance of current 2.27/2.28.

## Separate validation-enabled bounded result

The validation-enabled extraction reached its 120-second wall limit and was
killed; the owned process group was empty at closure after 121.534 seconds.
The native high-water RSS was 8.050 GB (sampled process-group peak 8.063 GB),
well below its 20 GB cap. Before the unfinished semantic identity stage, it
recorded file read 13.535 s, envelope/digest validation 1.199 s, State import
0.00135 s and payload decoding 10.956 s. Native semantic identity recomputation
was still executing at the cap; no exact sector evaluator or JIT was built.
This is a **failed bounded validation diagnostic**, not a successful full load
or a measured duration for semantic validation. The native implementation hashes
a serde JSON representation of the semantic payload, including program byte
arrays. Optional validation can therefore be material independently of JIT
restoration. This observation does not change the default-disabled policy.
Raw output and closure are retained under `runs/extract-validated/`; the earlier
successful unvalidated extraction and all three successful sector probes remain
unchanged. No further loader run is required for the scoped conclusion above.
