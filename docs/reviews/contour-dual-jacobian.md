# Native dual Jacobian construction

The optional `ContourJacobian::Dual` policy constructs the same mathematical
Jacobian with Symbolica's existing evaluator operations. It initially requires
`GenerationMode::NumericalDual` and supports contour charts with one through six
coordinates. Symbolic construction remains the default. Zero-dimensional and
exact-only outputs retain the requested policy without inventing a map; a
stochastic chart requiring symbolic subtraction fallback is rejected explicitly.

## Native reuse and mathematical boundary

The image vector is lowered with its retained native FunctionMap, differentiated
by a first-order native Dualizer, and connected to the existing generic native
Matrix determinant program by EvaluatorComposer. The complete semantic Jacobian
function Atom is an additional input to the regular density evaluator. Composer
connects that input to the determinant result before the existing outer
subtraction Dualizer runs. Thus an order-k subtraction jet includes the required
order-(k+1) image derivatives. No numerical determinant callback, manual chain
rule, or second automatic-differentiation implementation is introduced.

The independent API probe at
`target/foundation-d05-dual-jacobian/probe.rs` covers fixed and genuine implicit
strengths, nine interior/face/vertex points and 40 mixed coordinate/regulator
outputs per point. Native 192-bit differences against the full symbolic density
were at most 2.55e-57 and 5.10e-57. A separate whole-function input probe confirms
native evaluator parameter precedence. These small probes establish API
feasibility; they are not physical D05 performance evidence.

Coordinate derivative seeds remain one on faces. Only non-coordinate inner
seeds are zero; the existing outer subtraction recipe supplies its own face
restriction. Actual-face request lowering is applied consistently to the body,
images and semantic Jacobian before either evaluator is built. The source cache
key retains the exact plan, ordered inputs, native definitions and optimization
settings.

## Semantic storage and exact outputs

The retained definition body is the generic polynomial determinant. Its actual
arguments are the native derivatives of the image Atoms. These first-derivative
Atoms are still constructed for honest metadata, inspection and exact fallback;
the policy does not claim to eliminate that work. The generic determinant body
and coefficient bodies have one uniform native signature. Dynamic strength
calls remain visible in actual arguments rather than hidden inside a callback.

The staged chart has optional serde fields inside the existing schema-4 native
context envelope. Plan indices reuse the chart's native image/Jacobian Atom
table entries and retain ordered coordinate symbols. Restoration requires one
plan for each nonzero-dimensional Dual contour, matching the source index,
ordered context coordinates, images and semantic Jacobian. Old Symbolic records
remain readable. The final evaluator uses the existing saved ExactProgram codec;
only the resolved computational-policy JSON records the new choice.

Native materialization still owns exact-offset cancellation. It is intentionally
allowed at that boundary. The new stochastic NumericalDual path does not call
the symbolic assembly simplifier, which otherwise materializes a function body
when proving coordinate independence.

## Verification status

The coherent native all-target check passed. Two new scientific controls passed
in 4.23 seconds: complete higher endpoint jets with checked saved eager/JIT
owners, and mixed face jets with cache separation. The staged transport filter
passed four controls, including actual fresh-worker restoration with the Dual
choice. All three public policy ownership controls passed, including stepped
compilation, saved restoration, zero-dimensional output and early rejection of
unsupported construction. The current regular core library was rebuilt and its
Cargo artifact record pins the physical probe's dependency identity.

The CLI all-target type check and thin Python binding stub-generation type check
also passed. The two CLI option controls, four existing contour process controls,
and all 82 standalone portable tests passed. Installed Python execution of the
new option remains a separate check; it is not implied by the type checks.
The complete candidate2 workspace test run passed 917 tests with zero failures
and 33 explicit ignores. Earlier focused subsets are included in that count.
Strict workspace all-target Clippy passed in 26.92 seconds. Its sole requested
correction binds existing optional metadata with an `if let` condition instead
of checking and unwrapping it; the archived candidate2 executable predates only
this equivalent lint correction and remains the measured binary.

An initial zero-match test invocation reused a binary from a different frozen
checkout and is excluded from this evidence. A narrow FastSecDec package clean
and rebuild produced the current 428-test inventory; an independent review
verified the compiler working directory, source hashes and both new test names.
## Physical fixed source-zero baseline

The first bounded 1000 GeV D05 attempt stopped before discovery because the
ignored driver requested singleton preparation while the original record came
from shared four-recipe preparation. The strict native source comparison caught
the missing opaque factor payload. The corrected driver repeats the original
shared preparation and then selects the requested recipe; all source Atoms,
ordered symbols, declarations, factor payloads and the map agree natively.
The failed setup attempt remains recorded (11.99 seconds, 61.99 MB peak).

On the same current core and original 19 parameter bindings, the fixed Symbolic
and Dual source-zero programs both completed actual IBP construction, eager
compilation, saved publication and separate fresh-process restoration. At
strength 1e-6 both completed their native checked pilot and returned the complete
two-point, four-component Laurent vectors. Their maximum scaled difference is
4.075e-15 (absolute 6.751e-14). Fresh restoration reproduces each vector exactly.
Both retain the identical positive-imaginary-F refusal at strength 1e-5; that
strength is not admitted by this control.

The debug, non-idle feasibility measurements are 103.50 seconds / 2.157 GB peak
for Symbolic and 473.36 seconds / 3.967 GB for the initial Dual implementation.
Dual evaluator compilation alone took 405.64 seconds. Its saved evaluator is
1,143,374 bytes, compared with 2,638,943 bytes for current Symbolic. These are selected
source-zero construction measurements, not integral estimates or optimized
production speed claims. Raw records, native identities, hashes and comparison
are under `target/contour-gghh-double-box-1000-dual-probe`; `parity-fixed.json`
explicitly limits acceptance to the fixed recipe.

## Reuse follow-up

Independent source review identified repeated image-to-Jacobian evaluator
construction for distinct regular density bodies. The follow-up retains a
caller-owned immutable native Jacobian program keyed by the full face-local
plan, ordered inputs, definitions and compiler settings. Each body still gets
its own native composition and complete outer subtraction jets. Cache failures
remain confined to their exact key; the cache lifetime is the existing source
builder's lifetime. The two focused scientific/cache controls passed in 4.05
seconds, including distinct-body reuse, changed-plan separation and isolated
body/prefix errors. Four staged restoration controls also passed. A repeated
physical measurement of this follow-up is pending. Dynamic source-zero
comparisons and sampling cost remain separate gates; no overall performance
improvement is claimed.

## Complete fixed D05 generation

The immutable candidate1 optimized CLI completed the complete fixed-v1 D05
singleton at 1000 GeV in 141.935 seconds, with a sampled owned-process peak of
16,056,111,104 bytes. It used eight caller-owned serial workers, NumericalDual
IBP subtraction, Symbolic Jacobian construction and native SymJIT O2. All 30
six-dimensional source indices produced saved sectors, with 60 indexed records
and the full four-component pole/finite layout. The data file is 79,036,643
bytes. The process group closed normally without reaching the 900-second or
100,000,000,000-byte limit.

The source archive SHA256 is
`74a0747cf9207d47e98c2bcca1a476ea5060f212c340e4fc1e95e62821517889`;
the CLI SHA256 is
`54880e737498e60ab9882efdeea0174bd89a153e7e90b2dc053eafcd075ddcd5`.
Complete raw publication hashes, native source identity and resource closure are
in `target/contour-gghh-double-box-1000-delivery-fixed/handoff.json`. Independent
review confirmed those hashes and the full record/source coverage. Generation
overlapped independent builds, so its elapsed time is feasibility evidence, not
an idle-host benchmark. The artifact was handed to the runtime owner for actual
pilots and sampling; successful generation alone is not a physical estimate.
