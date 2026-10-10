# Native JIT clone callback ownership (2026-10-10)

The 50-worker D05 preparation exposed a native ownership difference: cloning a
Symbolica eager evaluator clones its external callback implementation, whereas
cloning a JIT evaluator continues to dispatch through the original compiled
callback environment. The numerical values in the focused controls are correct.
This establishes a missing independent-callback cloning capability for the
planned worker-local root scratch, not a measured D05 slowdown or an integration
accuracy defect.

## Source and public-API evidence

The tested owner graph is Symbolica/Numerica `516beb37` with SymJIT `d74993f`,
unchanged from the verified native consumer. Symbolica's
`ExternalFunctionContainer::clone` clones its `imp` and resets its argument
stack; the callback trait supports native `DynClone`. FastSecDec's dynamic
`Workspace::clone` correctly clones scratch into a new mutex and shares only the
immutable helper.

`JITCompiledEvaluator` instead derives `Clone`. This clones its external-function
vector but also clones its SymJIT `Applet`. The applet shares executable memory
and `Config.df: Arc<Defuns>`. SymJIT's sliced-function registration stores the
closure environment pointer in its function definition; the generated native
call embeds that environment. The newly cloned external-function vector is
therefore not the callback environment dispatched by the shared machine code.

The inspected public APIs offer saved-IR restoration with newly supplied
callbacks, but no in-place rebinding of the callback environments of shared
compiled code. Merely replacing the external-function vector does not relocate
the generated native calls. `Defuns`/`RawBox` must not be cloned as a workaround:
their owning raw-pointer clone hazard is already called out in the native backend
source. No alternate callback dispatcher, TLS cache or owner patch has been
introduced in FastSecDec.

## Focused reproductions

Both ignored Rust programs register a deterministic scalar callback
`f(x)=x*x+1`. Its captured workspace has an explicit `Clone` implementation that
creates a fresh mutex and diagnostic identity. The identity is observed without
changing the numerical result.

- `target/contour-d05-1000-runtime/clone-callback.rs`: eager original and clone
  invoke workspace IDs `[2,3]`; JIT original and clone invoke `[6,6]`. Both return
  exactly 5 and 10 at inputs 2 and 3. Source SHA256
  `53d3de5a971f9a38c75c2fdfb2d276a129e2cc59fcb4678d4c9e32924dbaaf85`;
  executable SHA256
  `6827462e7070e428f337c86ee1b39359759533dc3748557c568e37f226a10ba7`.
- `clone-concurrency.rs` in the same directory starts four clones together.
  Eager invokes four distinct workspaces and observes four concurrent critical
  sections; JIT invokes `[9,9,9,9]` and observes one. An artificial 10 ms sleep
  inside each critical section gives 10.815 ms versus 40.563 ms total. This is a
  mechanism control, not a numerical throughput benchmark. Source SHA256
  `ce92cc260a15121d22bf16aefb0c6e4cade875c02dda87cd76e41f237f32b31c`;
  executable SHA256
  `0177771712604372aa3c91678b9c73fbec0fc7c18785358ef88c176f98e0dd9a`.

The direct-rustc probes use the already built pinned native dependency graph;
they do not rebuild or modify owner sources. Initial local harness compilation
errors concerned a missing Cargo macro environment variable and using an eager
convenience method on the JIT type; those logs are retained separately. Final
builds and both executions pass. No benchmark or production source was changed.

The independent foundation review confirms the source chain and public-API gap,
including the architecture-specific embedding of the environment pointer. Its
review agrees that rebuilding a JIT evaluator from saved native IR is a correct
fallback, but may impose substantial setup cost for many sectors and workers.

## Consequence and next gate

FastSecDec's dynamic root callback takes its scratch mutex before either the
quadratic shortcut or generic native root solve. JIT clones of one compiled
sector can consequently contend on that mutex; eager clones do not. The effect
on actual D05 execution depends on root cost and the number of different sectors
active concurrently. The D05 artifact is not yet available, so no 50-worker
speed, variance or memory claim follows.

A correction should reuse native callable ownership or relocation support,
preserve the complete numeric/failure semantics, and avoid an unbounded hidden
thread cache. An explicit fallible independent-callback clone API is another
possible owner boundary; its preparation cost must be measured separately.
Any owner change requires a minimal regression, independent review and the
authorized upstream publication before consumer adoption. In parallel, sharing
FastSecDec's immutable exact evaluator can reduce clone memory locally, but it
does not solve this callback-dispatch contention.

## Published owner correction and acceptance

[Symbolica PR62](https://github.com/symbolica-dev/symbolica/pull/62), authored by
ValentinHirschi at `da82e2ed16aca4b211b1b68cd16d08afa3eebdde`, introduces native
scoped callback tables. Shared machine code dispatches by registry identity and
function index to the current evaluator's cloned callback implementation. The
borrowed frame is stack-local, restored by RAII, and installed on each existing
Rayon chunk for threaded batches. Nested compiled callbacks use their own scope;
public standalone conversion keeps its prior semantics. Callback-free intrinsic
programs retain their existing path. No FastSecDec dispatcher, new thread pool,
unbounded TLS cache, or per-clone recompilation is introduced.

Fourteen new owner controls pass: seven public evaluator tests, three low-level
backend tests, and four typed nesting/unwind tests. Fifteen existing evaluation
tests pass; the pre-existing 1000-variable debug stress test remains ignored.
The independent review additionally executes the actual scope module under
nested/unwind and four-thread controls and strict Clippy. New public tests pass
strict Clippy; the full upstream tree retains 271 existing warnings and an
unrelated `poly/factor.rs` `never_loop` lint (the retained baseline-allow run
passes). Both root and foundation source reviews accept the scoped lifetime,
registry, SIMD/threaded, named-body and original-drop ownership boundaries.

The same synthetic four-thread workspace probe now observes four distinct JIT
workspaces, peak concurrency four, and 10.421 ms total with the artificial 10 ms
critical section. This demonstrates removal of the reproduced serialization;
it is not a D05 throughput measurement.

The tested public consumer revision
`650d9427ae2e20b8f9457b46bdfb02d1fddbb1df` is a clean cherry-pick onto
`516beb37d31af8e3d6ee321a7070f407a0b1b42d`. Its focused integration gates pass
36 tests: seven callback-context, fifteen existing evaluation, six direct-cache,
and eight direct-vector controls, with the same one pre-existing ignored stress
test. SymJIT remains `d74993f`; consumer pin adoption is a separate milestone.
Logs and reproducer scripts remain under `target/contour-d05-1000-runtime/`.

GitHub denied the formal reviewer request because the publishing account lacks
review-request permission. The authorized
[benruijl review invitation](https://github.com/symbolica-dev/symbolica/pull/62#issuecomment-6095045487)
was posted explicitly. No review acceptance is claimed.

## FastSecDec consumer adoption

All three maintained manifests and lockfiles select the public combined owner
`650d9427ae2e20b8f9457b46bdfb02d1fddbb1df`. Locked full metadata passes for the
native, Python and portable consumers, each with one Symbolica/Numerica identity.
Native/Python keep SymJIT `d74993f`; the portable consumer has no SymJIT.

A frozen Git archive of FastSecDec `e029667` plus only those six manifest/lock
overlays passes **406 native library tests**, with 19 intentional ignored tests.
The build/test run takes 782.901 seconds with 6,276,890,624 bytes sampled aggregate
peak RSS; test execution is 30.55 seconds. The monitored process group closes
normally with no limit signal. The source inventory, overlay hashes, metadata
and log are retained in ignored `target/contour-jit-owner-adoption/`.
This consumer gate excludes the evolving dual-option implementation and supplies
no physical production-throughput claim.
