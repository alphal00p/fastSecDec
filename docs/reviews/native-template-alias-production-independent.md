# Independent native alias integration review

Reviewed against `native-template-alias-integration-design.md`, the successful
native template/IR controls, and Symbolica revision
`98794d0d7337ba2b08e4c046dde584ad7fc1ce10` with the four previously recorded
patches. The subsequent native decoder fix and implementation evidence are
recorded below. A complete graph campaign remains a separate scientific gate.

The generation/backend design follows the native reuse contract. Native
`AliasedAtom` owns roots and image bodies; the existing late template selection,
domain certificates, mapping and subtraction retain their scientific roles.
Registering the common image map once avoids the native duplicate-registration
behavior of calling `AliasedAtom::evaluator_multiple` on multiple roots with
identical maps. No callback symbols or process-global polynomial caches enter
the production path. One native exact evaluator supplies the ordinary,
conditioning and MPFR paths, including weighted replay and worker clones.

Preserving `coefficients() -> &[Atom]` with a documented lazy materialization
cache is compatible with existing callers. The new native aliased view must
remain the path used by compilation and persistence. Native alias `Display`
shows only its root, so presentation must not imply that this alone is a fully
restored physical expression. The flat generated map avoids recursive alias
cycles; caller-supplied maps should not acquire an unchecked public construction
path as a side effect of this change.

The implementation gates need used-body-aware coordinate and complex
classification, exact and zero-dimensional offsets, output zero padding with
the same common map, and conservative zero-component flags. A zero root or an
unused complex body has different meaning from a used complex body. Unknown
cancellation is not a symbolic zero certificate. The relative-series cutoff
must cover the requested absolute order, with the already tested exact-zero
case kept distinct from an empty truncated series. Existing lower-level tests
are not substitutes for public generate/compile/cold-load vector tests.

## Native decoder blocker and safe reproduction

Source inspection found no public structural-validation API in native
`src/evaluate`. `ExpressionEvaluator` serde deserialization and bincode
`Decode`/`BorrowDecode` populate raw fields without checking their relationships.
`try_evaluate` checks caller input/output lengths, but its instruction execution
uses unchecked Add/Mul stack indexing. `get_constants` and coefficient mapping
also rely on valid stack partitions and external constant indices.

The non-evaluating Rust probe `output/probes/native_ir_decode.rs` confirms that
all three decoders accept six malformed structures: empty Add operands;
out-of-range operand, destination and result indices; reserved range past the
stack; and parameter count past the reserved range. The probe never executes,
exports, maps or asks for constants from malformed IR. It exited successfully
in 0.73 seconds on CPU8, recording accepted defects in
`output/diagnostics/native-ir-decode/result-final.txt`; executable, source and
linked dependency hashes are retained beside it. This overlaps an independent
guarded C++ reference process and is correctness evidence only.

Checksums, decoded-byte exhaustion and outer layout checks do not fix this:
malformed contents can carry a matching checksum. A trusted-producer restriction
would weaken the CLI loading contract. Do not implement a FastSecDec instruction
validator or duplicate native IR layout to compensate.

The proposed narrow native fix is an additive
`ExpressionEvaluator::validate_structure() -> Result<(), String>`, called by
serde deserialization, bincode Decode and BorrowDecode before returning an
evaluator. One native helper should validate both root and native function
bodies: stack partitions and results; every operand/destination; nonempty
Add/Mul operands and bounded phase prefixes; supported builtin tags; forward
branch targets with matching labels; native function arity and earlier-callee
ordering; and external constant bindings within the constant range. This
establishes representation invariants, not mathematical correctness or a proof
of the originating graph. Fallible native tag parsing should preserve decoder
errors rather than panic. Approximate scope is a small validator module plus
focused native invalid-structure and valid nested-function/branch tests.

Version-three loading should wait for this native ownership gap to close. Its
outer format must also bind the supported native codec/revision contract,
precision/compiler policy, immutable original program bytes and all semantic
metadata. Native instruction serialization has no general promise of stability
across unrelated revisions. Legacy v1/v2 identity and re-emission behavior must
remain covered. No scientific or latency claim follows solely from this design
review.

The native fix was subsequently implemented by the original reviewer after
the gap was demonstrated; the coordinator independently reviewed that patch.
Its four native tests pass, and all six original malformed byte fixtures are
now rejected by each of serde, native Decode and BorrowDecode without execution.
See `../dependency-patches/symbolica-evaluator-ir-validation.md` for the isolated
patch, old/new library identities and the public-boundary exact-program fixture.
This author/independent-review separation applies to the native fix; the
FastSecDec generation/backend/artifact implementation remains separately owned.

Initial implementation review also found a placeholder collision with declared
but unused parameters or regulator symbols. The generation owner now reserves
all declared symbols before assigning native template handles, with targeted
unused-coordinate and unused-regulator controls. The v3 codec now names the
exact native revision as well as the package and schema versions. Cold-loaded
programs conservatively omit symbolic zero/real proofs; this can add MPFR rescue
work for padded zeros and zero imaginary parts, and must remain visible in
cold-load performance evidence. No replacement symbolic fact analyser is
introduced to hide that difference.

## Concrete implementation and public API review

The implemented `GeneratedSector` retains native `AliasedAtom` values and an
initially empty `OnceLock` for the compatibility accessor `coefficients()`.
Compilation and both generated/compiled portable saving use
`aliased_coefficients()`; neither calls the materializing accessor. The remaining
production restoration is limited to the existing bounded small-expression
simplification and analytically integrated offsets after used-body dependency
classification. A test observes that compilation and both save paths leave the
lazy cache empty, then verifies explicit restoration against the original
expression. Internal numeric workers never parse or restore the alias bodies.

The root/image map is flat, deduplicated by native Atom identity and registered
once for each complete vector. All nonempty maps in the vector must agree;
zero-padding may have an empty map. Used-body checks account for hidden
coordinates and imaginary literal constants, while unused definitions cannot
force a numerical sector or complex layout. Multiplicity is applied to the root
once. Original subtraction, domain and measure ownership remain unchanged.
The relative-series extraction retains integer-order, regulator-exclusion and
absolute-coverage guards, including negative requested maxima and Gamma poles.

The exact native program is encoded before coefficient remapping or evaluation.
Ordinary O2, native roundoff tracking, MPFR rescue and independently cloned
workers all derive from it. Native JIT admission of fixed external constants is
fallible and precedes the infallible conditioning remap. There are no custom
derivative callbacks, process-global polynomial caches, second evaluator
algorithm or native instruction schema in FastSecDec. The existing caller-owned
progress, cancellation, weighted replay and typed precision reports remain the
public numerical interface; no runtime or scheduling ownership moved into the
kernel compiler.

Version three binds the exact native codec revision, compiler/precision policy,
ordered component/input/output layout and existing portable semantic metadata.
The native decoder validates structure before constants or numeric execution
are accessed. Existing chart/domain objects remain the computation owner; their
transport is revalidated without reconstructing geometry from evaluator code.
Legacy expression-only v1/v2 data retains its original bytes and identity after
loading and evaluation. Saved results, selected-sector manifests and replay
state continue to consume the public kernel layout and content identity; no
statistics or result serialization was duplicated for the new representation.

The observed focused scientific log
`output/native-alias-scientific-tests.log` passes 103 tests with nine explicitly
ignored diagnostics. It includes hidden-complex/Gamma/negative-order public
vectors, exact offsets, weighting/rescue, independent workers and cold-process
artifacts. The subsequent five artifact tests pass in
`output/native-ir-artifact-followup-tests.log`, including the native malformed
33-byte program under a correctly recomputed outer identity. The one legacy CLI
inspection test also passes in `output/native-ir-legacy-inspect-tests.log`.
These are coordinated executed gates inspected by this reviewer, not new
duplicate executions. The actual source-bound 241-piece representative and the
full workspace milestone remain separately reported; passing format tests alone
does not establish graph coverage or cold-load performance.

The six-test artifact rerun in `output/native-alias-artifacts-six-tests.log`
also passes, adding fixed-argument complex Gamma rejection under a tampered real
layout. The malformed input still never reaches numerical evaluation.

The subsequent combined log `output/native-alias-workspace-tests.log` contains
**294 passing tests, zero failures and 18 explicitly ignored probes** across
55 test-result summaries. The final formatting log is empty/successful and
`output/native-alias-clippy-final-2.log` completes successfully; the one existing
native Symbolica warning remains disclosed. These coordinated gates were read,
not rerun, by this reviewer.

## Captured representative gate: comparison contract

The bounded runner binds the original 241-piece expression, all 201 image
definitions, actual production source, copied binary/native dependency,
configuration and three independent original-expression oracle records before
and after execution. It uses the unchanged production Laurent and kernel paths,
followed by all six orders at all three rational points at 512/1024 bits. A
separate fresh/cold kernel comparison uses the same rounded binary64 coordinates
as its MPFR reference, with weights one and 1e40. No multiplicity, full graph
generation or integration claim is added to this representative test.

Two initial diagnostic attempts are retained. The first reaches the series but
fails a direct root-Atom comparison because process-local symbol ordering
renumbers image handles. The second proves equal unique image-body/handle sets
and literal renaming, but encounters different factored coefficient structure
at order -4. Its first order (-5) is structurally identical. Neither attempt
reaches or passes the numerical oracle stage; these are not scientific passes.

The revised test obtains the actual pre-series template from the production
cache key, proves its exact equality to the saved template under the complete
native image bijection, and retains each fresh coefficient plus per-order
structural-equality flags. Literal renaming is simultaneous and applied only to
comparison values; fresh production coefficients are untouched. Laurent
expansion commutes with a bijective rename of epsilon-independent handles, but
factored Atom syntax is not a unique algebraic normal form. Thus exact template
identity plus native series coverage and unchanged complete independent oracle
checks is a meaningful contract without asserting all coefficient trees have
identical structure. Native `together`/`cancel` can invoke polynomial conversion
and GCD work, so they are not a known bounded compact replacement for this test.
No structural difference is silently discarded or promoted to exact
coefficient-tree equality.

### Executed representative outcome

The independently inspected third attempt,
`output/diagnostics/production-alias-representative-3`, passes. Its original
capture/configuration, fresh source/binary/native dependency and oracle inputs
remain bound by **529 pre/post SHA-256 checks**, also rechecked by this reviewer.
The process exits zero without timeout in **78.550794895 seconds**, with
`wait4` peak resident size **532,220 KiB**. These are one-process diagnostic
measurements using development-profile FastSecDec and optimized native
dependencies, not matched release timing acceptance.

The exact 201-body/handle bijection and pre-series template identity pass.
Every fresh coefficient is exported. Only order -5 has identical renamed Atom
structure; the other five flags remain false in the retained diagnostic. All
**18 independent coefficient comparisons** (orders -5 through zero at three
prescribed rational points) pass native 512/1024-bit agreement and the
original-expression point-first oracle at the declared `1e-70 * max(1,scale)`
tolerance, including zero imaginary parts. Thus the finite coefficient is
checked as well as every pole; the earlier structural failures have not been
reclassified as prior successes.

The same exact program also produces **12 complete production weighted
vectors / 72 component checks**, using fresh and decoded kernels at the same
rounded binary64 coordinates as their MPFR reference, with weights one and
1e40. Every vector is checked and rescued: 256 bits for points zero/one and
512 bits for point two. The recorded fresh/decoded vectors are equal in each
pair. This is a native codec decode and backend rebuild within the same
process; separate small/public tests provide fresh-process artifact coverage.
No individual raw O2 stability or rescue-free performance claim follows.

The exact native program is **1,497,695 bytes**. Fresh Laurent expansion takes
45.141 seconds; build/encode/decode/two backend construction takes 6.333 seconds.
The test starts from the original 241-piece subtracted expression, keeps all
32 cancellation tuples, and applies no multiplicity. It does not regenerate
the graph, manufacture chart metadata, or integrate a representative. Full
on-shell generation and integral-level correctness/convergence remain open.

Retained result SHA-256:
`8e71ad4d198929b3bff6aaf606b0dc1c35fbefaa86ba1c6d20a78dfc54ca61e4`;
process report SHA-256:
`2d8f155f5c63f4c6a96fe6bc9539e7a6b3a9d9d39504fde2b1dd15d5c7ede59e`.
