# Proposed production integration of native Laurent aliases

Status: implementation, the complete workspace gate, and the captured
representative's three independent complete-vector oracle checks pass;
whole-graph acceptance remains open. Executed gates
and retained failures are recorded in `native-alias-production-results.md`.
The prerequisite proof is recorded in
`native-template-alias-proposal.md`: the original 241-piece capture has a
complete fresh native series, exact equality of all six coefficients to the
saved inputs, and all three independent six-order MPFR oracle comparisons.
The callback route and the optional Never policy are not part of this design.

## Keep symbolic and numeric ownership native

The insertion point remains `generation/laurent.rs`, after unchanged mapping,
domain/face certification, symmetry and endpoint subtraction. Keep its exact
existing placeholder selection and per-generation template cache. Replace the
absolute-depth retry route with the already controlled native relative-depth
sequence, checking native final absolute coverage before accepting nonempty
coefficients. Exact input zero is a separate structural case; an arbitrary
empty truncated series is not proof of an all-order zero. Negative requested
orders, fractional-order rejection, Gamma poles, prefactor cancellations and
native essential-singularity errors remain acceptance gates.

Return native `AliasedAtom` coefficients with the complete existing image map
attached to each root. This directly exposes a native symbolic object to library
callers rather than inventing a FastSecDec alias graph. Every coefficient is
self-contained. The image maps are identical by construction; the first slice
accepts their small per-order ownership cost rather than introducing a shared
custom symbolic container. Measure that cost explicitly (the tested six-order
case has only 21.5 KB of unique image bodies).

The native multi-output evaluator is constructed from all coefficient roots,
and the common image map is registered **once** through `add_aliases`. Do not
use `AliasedAtom::evaluator_multiple` with repeated identical maps: the native
function map rejects duplicate registrations. Verify identical maps at this
boundary, and retain duplicate/collision errors. Use native direct translation
with the tested settings as a declared compiler policy, not a hidden environment
switch. No alias substitution occurs during worker evaluation.

The public symbolic view remains unambiguous. Add
`GeneratedSector::aliased_coefficients() -> &[AliasedAtom]`. Preserve the existing
`coefficients() -> &[Atom]` contract with an explicitly documented lazy native
materialization cache; never return unresolved placeholder roots under the old
meaning. The selected compatibility implementation uses a
standard `OnceLock<Vec<Atom>>` and native `AliasedAtom::into_inner` only when the
caller requests restored expressions. Generation, compilation, inspection of
layout/metadata and artifact saving must not call that accessor implicitly.
This cache stores native results; it implements no symbolic operation.

Coordinate-dependence and exact-only classification must inspect the native
alias definitions as well as roots. Since the existing images are flat and
each contains coordinates, a used image conservatively keeps that output in a
numerical sector. Preserve existing bounded exact simplifications where they
are cheap. Unknown cancellation is not exact zero, and no sample-based
classification is permitted. Multiplicity is applied exactly once to roots;
all Laurent orders and full-vector cancellation metadata remain attached to the
same chart/representative associations.

## One exact evaluator feeds every numeric path

Factor evaluator construction into a small `kernel/program.rs` boundary using
native `ExpressionEvaluator<Complex<Rational>>`. Real/complex O2, native error
tracking, precision-cache remapping and weighted MPFR replay all derive from
that same exact evaluator. Update the real and complex backend constructors to
consume an already built native evaluator rather than independently rebuilding
from restored coefficient Atoms. Keep native cloned-worker state and the
caller-owned QMC/MC execution APIs unchanged.

Backend/layout and zero-component facts must account for image bodies, not only
outer roots. A missing proof means a conservative complex/unknown-zero path;
never silently project to real. For the admitted real-domain functions, retain
the existing native complex-coefficient reasoning with both roots and bodies in
scope. Test hidden complex images and asymmetric underflow after native weighted
scaling. Flags that suppress precision checks require actual symbolic proof;
loaded IR without that proof may use conservative false flags. A native real
JIT construction failure must remain an error, not trigger coefficient
projection. All three demonstrated points require precision checks, so default
tolerances and lost-bit guards are unchanged.

The selected cold-load policy uses false zero/component facts. With weights
above one, zero-padded outputs or identically zero imaginary components can
therefore request additional MPFR work after reload. This is conservative but
can change rescue counts and timing compared with a fresh source-built kernel;
cold and fresh performance must be reported separately. No serialized claim is
allowed to suppress a check without native proof. Immutable encoded program
bytes use standard `Arc<[u8]>` ownership, so numeric worker clones share them.

`SectorKernel` needs numeric output/layout counts and conservative zero facts,
not copies of the large restored Atoms. Removing those copies from worker
cloning is part of the ownership boundary. The generation object retains the
native symbolic view; numerical workers retain native evaluators and buffers.
No process-global custom SymbolBuilder callbacks are introduced.

## Versioned native IR persistence

Use a new strict kernel artifact version for compact programs. Its envelope
contains ordered parameter names, Laurent/component layout, exact offsets,
cancellation tuples, precision/compiler policy, retained generation metadata,
and native exact-evaluator bincode bytes. Native Symbolica already serializes
instructions, constants, function bodies and external constant bindings. There
is no parallel FastSecDec function-definition serializer and no machine code in
the artifact. Root owns promotion of the existing bincode dependency from
development-only to normal Rust dependencies.

`GeneratedIntegral::to_kernel_bytes` can build portable exact IR without JIT;
document and report that compilation work. `KernelSet::to_bytes` stores the
already built exact program, and cold load rebuilds only host O2 and numeric
buffers. Native builtin registration must precede cold decoding. Preserve
strict codec/version checks, decoded-byte exhaustion, input/output dimensions,
orders, cancellation rows, and all existing chart/domain metadata validation.
Retain the original immutable encoded program for stable save identity rather
than serializing mutated numeric work stacks.

The new content identity binds the native IR, codec/compiler settings and every
semantic metadata field. It does not claim that a checksum mathematically
proves the program came from a particular graph. Native decoding remains the
owner of program representation; do not implement an instruction validator or
disassembler. Reject incompatible encodings explicitly. Preserve version-one
and version-two loading and their original content hashes/re-emission semantics
so historical diagnostic artifacts and checkpoints remain usable. A newly
generated compact program receives a new identity; no old checkpoint is silently
retargeted to it. Saved-result/reference envelopes continue to use that identity.

Current `SectorKernel` has no public restored-expression accessor, so a compact
numeric artifact need not reconstruct symbolic definitions after load. Dimension,
layout, exact offsets and retained scientific metadata stay queryable. The
generation object's native symbolic view provides explicit restored output when
requested. Any new public source-view availability contract must say which
representation exists instead of pretending an IR-only load contains the old
coefficient Atoms.

## Reviewable implementation slices and gates

1. Native aliased coefficient output plus lazy explicit materialization; strict
   native relative-depth handling; unchanged domain/mapping/subtraction. Test
   poles, negative maxima, prefactor-zero cancellation, zero/fractional/error
   cases, compact polynomial factors, aliases hiding coordinate dependence and
   explicit restored symbolic equality. Keep support and precision code frozen.
2. Shared exact-program construction for real/complex backends and weighted
   precision. Exercise hidden complex bodies, zero-component conservatism,
   clone-on-worker MPFR, zero weights/singular endpoints, underflow/subnormal
   scaling, and all three captured complete-vector oracles through the public
   kernel path. No aliases-only raw-f64 accuracy claim follows.
3. Strict versioned native IR artifact and legacy round trips. Cold-process
   Gamma/log/complex programs, malformed/truncated/incompatible codecs, layout
   and metadata mismatches, preserved old identity, replay-state/checkpoint
   binding, CLI generate/inspect/integrate and saved-result flows are required.
4. Rerun bounded full on-shell generation and integration using the reviewed
   production interfaces. The 42.5-second isolated Laurent substage does not
   establish whole-integral generation feasibility, integration convergence or
   sample-throughput performance. Preserve failures and the original graph;
   no coefficient, sector or pole is omitted to meet a deadline.

Independent HEPKit review should settle the native symbolic view, conservative
classification and version-three persistence contracts before implementation.
The existing callback experiments remain isolated diagnostics with their open
global-lifetime limitation; they are not needed for this first production slice.

## Implementation and review boundary

The generation changes retain native aliases and bounded small-expression
simplification. A native symbol-set query classifies coordinate dependence
without repeatedly traversing large roots for every image. Zero padding may
carry an empty alias map; nonempty maps within a vector must agree. The native
builder registers that shared map once. Numerical compilation now has separate
`program` and `compilation` modules; worker state carries exact native IR and
conservative facts rather than restored coefficient Atoms. The artifact slice
has a separate author and preserves immutable legacy payloads.

Independent review found a required native load-safety gate: the pinned native
evaluator decoder accepted unchecked stack/instruction indices. Dimension and
hash checks cannot establish memory-safe IR. The HEPKit reviewer owns a narrow
native structural-validation patch and regression tests; no FastSecDec
instruction parser or validator is introduced. Version-three artifact loading
now passes the owner's native tests and public malformed-transport rejection
tests. It remains a structural safety check, not a mathematical certificate.

The initial scientific gate reuses the existing Gamma, negative-order,
subtraction, complex, weighted-rescue and cold-process suites. Added controls
exercise lazy materialization, a reachable complex image with a complete Gamma
vector and negative requested maximum, literal image-name collisions, and the
native absolute-depth series as an independent comparison to the production
relative-depth sequence. Full captured-representative and whole-graph runs
remain separate gates, with no current performance acceptance claim.
