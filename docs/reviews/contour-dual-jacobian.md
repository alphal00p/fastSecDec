# Native dual Jacobian construction

> Scope correction, 2026-10-10: the physical measurements below used
> `GenerationMode::NumericalDual` for endpoint reduction. They are historical
> diagnostics and do **not** measure the subsequently requested comparison of
> symbolic endpoint reduction with symbolic versus contour-only dual Jacobians.
> High-order endpoint-jet costs do not establish a limitation of first-order
> Jacobian dualization. The corrected construction and its measurements require
> separate acceptance; see [the clarification](../../CONTOUR_DEFORMATION_PLAN.md)
> and [the symbolic-endpoint audit](contour-symbolic-endpoints.md).

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
physical measurement of this follow-up initially reached its 600-second limit
for the polynomial recipe without producing an evaluator. The native CSE
follow-up below subsequently completed that case. Sampling cost remains a
separate gate; no overall performance improvement is claimed.

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

## Native CSE follow-up: saved SymJIT source-zero comparisons

The separately reviewed native operand-remapping correction is consumed at
Symbolica/Numerica revision `74225696cd445247fa81c499c5110decd19257ed`.
The candidate3 source archive is
`3a430ad45c0df36a1676b1b29769974064416e7afa54fef23a143ddb89248b53`.
The ignored physical driver links the exact verified release dependency graph,
uses native SymJIT O2, one optimizer core, Horner zero and CPE maximum 1000,
and preserves the original source, IBP construction and all 19 physical
bindings. It introduces no numerical callback or alternative algebra.

Both optional Dual source-zero cases now complete. The fixed case took
33.193 seconds with 3,244,945,408 bytes peak owned RSS; the polynomial case took
224.186 seconds with 8,628,269,056 bytes peak. These include preparation,
construction, compilation, saving and the driver's checked values. Their
native evaluator compilation intervals were 6.801 and 190.550 seconds.
The actual polynomial Symbolic source-zero worker reported a 91.296-second
compilation interval. Thus this evidence does not establish a Dual compilation
speed advantage. Full-artifact worker walltime and isolated source-zero walltime
are not interchangeable measures of generation work.

| Recipe | Symbolic saved source-zero bytes | Dual saved source-zero bytes | Maximum scaled vector difference |
|---|---:|---:|---:|
| Fixed | 2,642,927 | 1,143,353 | 4.075e-15 |
| Polynomial | 5,743,519 | 2,162,020 | 3.708e-15 |

These are actual native exact-plus-stochastic records from source zero, with
the complete real/imaginary pole and finite coefficient layout. Each new Dual
owner is dropped before restoring its saved bytes, and a separate process
repeats the checked evaluation. Fresh restoration reproduces the vectors
exactly. At the two declared interior points, fixed strength 1e-6 is admitted
and 1e-5 retains the identical certified positive-imaginary-F refusal in both
representations. Polynomial strength uses S=0.8, R=1 and admits both declared
caps, 1e-6 and 1e-5. No refused point is treated as a successful value comparison.

The Symbolic reference is selected from newly generated candidate3 native
records. Successful CLI publication removes its temporary staging directory;
an initial attempt to read the deleted fixed receipt failed before output.
The corrected small adapter uses the final native archive catalogue and
`ProgramArchiveWriter::append_record`, verifying source-zero ownership and the
adjacent exact/stochastic record pair before extraction. It does not regenerate
the source or bypass the CLI dependency fence. The adapter and source comparison
were independently reviewed.

All four source-zero owners, resource closures and exact comparison hashes are
recorded in
`target/contour-gghh-double-box-1000-candidate3-dual-probe/cost-handoff.json`
and `parity-fixed-polynomial.json`. Matched native sampling-cost measurements
have been handed to the runtime owner; no runtime advantage or amortization
threshold is inferred from the smaller saved programs alone.

## Historical source-zero runtime costs: NumericalDual endpoints

These completed measurements use **NumericalDual endpoint reduction in both
columns**. They do not satisfy the subsequent request for symbolic endpoint
reduction with an independent choice of Jacobian construction, and they do not
select the revised benchmark's construction.

The four saved source-zero owners above were restored with the verified
candidate3 release graph, Symbolica/Numerica `74225696`, and native SymJIT O2.
The predeclared matched allocation was native Kuo 4096 points × two shifts,
Korobov3, seed `202610100903`, one CPU, and batches of 256. All four actual
coordinate/weight digests agree. Each owner first passed the maintained native
Pilot16 protocol with seed `202610100904`. Fixed strength was 1e-6; polynomial
used S=0.8, L=1e-6, R=1. All 8192 rows per arm completed, with no arbitrary-
precision rescue or optional production causal checks.

| Recipe | Symbolic-J mean µs/point | Dual-J mean µs/point | DD rescues in each arm |
|---|---:|---:|---:|
| Fixed | 29.289 | 37.679 | 27 |
| Polynomial | 57.241 | 70.675 | 28 |

The timer encloses native weighted full-vector evaluation, including precision
rescues and lazy evaluation setup. Artifact loading, binding, causal pilot,
context construction and coordinate hashing have separate timers. Exact offsets
are retained as provenance but excluded from this stochastic cost. Since this
is one source sector, its mean is not an all-sector average, a worst individual
sample, a variance estimate or a full-integral result. Host workloads overlapped;
these bounded single allocations do not establish a universal performance ratio.
The prior 32 × two-shift cold-context feasibility is preserved separately.

The larger run closed in 6.781 seconds with 97,767,424 bytes peak process-group
RSS, within its 120-second/100-GB bound. Raw native summaries, all precision
counts, coordinates and stage timings are under
`target/contour-d05-1000-runtime/cost/candidate3-dual-source-matched_4096x2/`.
The native summary SHA-256 is
`ec045a549d6fcc5f4f2afe6e28631a30f45f02ec51245719c7a062d7d09e7949`;
the frozen plan SHA-256 is
`480977c5b08cc88d23b85804e861dea151bc2b2efd4bb313977f328fa521d031`.
No later final launch used these measurements after the endpoint-mode
clarification.
