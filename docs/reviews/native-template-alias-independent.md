# Native late-template aliases: independent source audit

The next controlled experiment should start with the original production
241-piece subtraction template and its 201 retained image pairs. This preserves
the successful native relative-series baseline and avoids attributing the
callback experiment's different 384-piece expression to alias construction.
The existing finite template coefficient is still 36,898,483 Atom bytes; its
restored coefficient is 256,030,180 bytes. Avoiding restoration can therefore
reduce a distinct materialization boundary, but does not by itself eliminate
native series cost or establish full generation/runtime performance.

The public native APIs support the intended boundary. `FunctionMap::add_aliases`
registers argument-free variable or fixed-call definitions. In direct native
linearization, aliases always inline in their caller's scope and reuse that
scope's subexpression cache (`evaluate/tree.rs:944–993`). An evaluator built
from all template coefficients can register the exact captured image map once,
then translate directly into one native evaluator without first constructing
the restored large Atom. No symbolic derivative callback, global body cache,
new algebra engine, or separate runtime U/F evaluator is required.

An optional zero-argument ordinary function is a different native policy.
`FunctionMap::add_function_with_options` permits a symbol used as a variable
when its argument list is empty. With `InliningPolicy::Never`, its body sees
root evaluator inputs; direct linearization discovers the surviving global
slots and forwards them to the retained native function body
(`tree.rs:828–930,1055–1083`). Explicit local arguments shadow globals for
ordinary functions, while aliases inherit caller scope. Existing source tests
cover captures, repeated scopes, shadowing and wrong-arity variable use.
That distinction needs an executable local-shadow control, not an assumption
that alias and ordinary-function policies are interchangeable in arbitrary
nested contexts. The original top-level template has no such local bindings.

There is one convenience-API trap. `AliasedAtom::evaluator_multiple` currently
loops through every root's alias map and registers all definitions each time
(`atom/alias.rs:178–190`). The receiving FunctionMap rejects a repeated key,
even with the same body (`function_map.rs:219`). Giving each coefficient the
same 201-image map can therefore produce `FunctionRedefined`. The proposed
`Atom::evaluator_multiple(...).add_aliases(images_once)` route uses the native
API directly and avoids duplicate registration; it needs no dependency patch
or alternate alias resolver. This observation is source evidence, not a newly
executed upstream regression.

Before actual-case timing, small controls must retain nonempty complete vectors,
hidden complex literals, missing globals/recursive definitions, local-shadow
semantics, and native exact-IR cold loading with no symbolic callback registry.
The actual proof must bind original captured source/image identities and compare
the complete order union against the independent original-expression oracle at
all prescribed points. O2, eager and weighted MPFR must derive from the same
native exact evaluator. Count definition bodies and native IR as well as root
Atom sizes, separate series/build/JIT/load/evaluation stages, and retain raw
ordinary-precision discrepancies. This is source approval for a disconnected
capability experiment; no executable or performance acceptance is implied.

The completed author proposal `native-template-alias-proposal.md` has also been
reviewed. Its revised ordering first lowers the six already-validated template
coefficients and exact image map, explicitly labelling this warm reuse rather
than fresh generation. Only successful lowering/cold evaluation permits the
fresh native-series repetition; callback-template composition is a separate
optional last stage. It keeps direct translation and O2 fixed across alias and
ordinary-function policies and preserves independent stage limits/failures.
No source-level blocker remains for the stated small controls and bounded
original-template experiment. Executable evidence remains pending.

## Executed small controls and cached original coefficients

The first writer retained all eight case/policy IR files, then failed at the
exact-zero control's strict absolute-order assertion. Native absolute truncation
sets an empty series' stored order to zero. The correction is confined to the
known input `Atom::Zero`, asserting native zero and empty terms directly; it
does not infer exact zero from an arbitrary truncated series. The shared
nonempty bound guard is unchanged. The fractional control records absolute
order two and still requires the typed fractional-order rejection.

The corrected frozen run at
`output/diagnostics/formal-functions/template-alias-2` passes the writer and
fresh native-IR reader with exactly nine records. Four scientific controls run
under both native aliases and zero-argument Never functions, plus the native
scope control. Their complete order sets are `[-2,-1,0,1]`, `[-2,-1]`, `[-1,0]`
and `[-1,0]`. Hidden complex image coefficients and native exact-IR constants
are explicitly nonzero. Full coefficient equality, native ordinary/MPFR
evaluation, cloned worker, weighting, scope, collision, cycle, missing-global,
zero/fractional and essential-series checks pass. This is nonvacuous evidence;
the earlier failed attempt remains separate.

Independent verification of the frozen hash manifest and retained actual
reports confirms the original 241-piece capture, exact restoration identity,
all six absolute-versus-relative native coefficient identities, all 201 image
pairs, and source/oracle hash binding. The **cached** alias builder completes
in 0.122631 seconds, producing a 1,523,445-byte native exact IR; import and exact
identity checks take 5.757235 seconds. The current imported coefficient Atoms
total approximately 50.7 MB, with 21,513 bytes of image bodies. These observations
exclude fresh Laurent generation and must not be presented as its speedup.

Three separate native-IR reader processes compare every order from -5 through
0 against the independent original-expression oracle at all three prescribed
exact rational points. All 18 real-order comparisons and all paired 512/1024-bit
real/imaginary precision checks pass at the stated scaled `1e-70` criterion.
No representative multiplicity is applied. The numeric reader reconstructs
neither symbolic definitions nor callback state.

Raw ordinary-precision discrepancies remain material and are retained. The
native error-tracking policy reports unstable at **all three points**, and the
retained production cancellation metadata requests checking at all three.
O2 agreeing with eager evaluation is therefore not an accuracy certificate.
Native MPFR weighting tests scale before binary64 conversion, but this isolated
proof does not exercise production adaptive replay selection. Fresh series,
production integration, and complete-integral convergence remain separate gates.

The subsequent `template-alias-fresh-1` stage also passes its frozen hashes and
all six exact native coefficient comparisons. Starting from the captured
14,505,102-byte original post-subtraction expression, it reconstructs the exact
late template and 201 images in 0.408402 seconds, then native relative series
finishes in 38.389408 seconds. The first relative width one exposes leading
order -5 and remainder -4; native width six reaches absolute remainder one,
covering all requested coefficients through zero. Each fresh coefficient is
exactly equal to the previously validated alias-builder input.

The complete isolated process takes 42.538960 seconds with 547,900 KiB peak
resident memory. This closes the fresh **Laurent substage** capability gate
without restoring giant coefficient Atoms. It still excludes a new graph,
geometry or subtraction run and does not establish full on-shell integral
generation, production precision rescue, or integration/convergence acceptance.
