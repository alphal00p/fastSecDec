# Dynamic callback failure admission

The original failure-fence increment separated essential numerical failure
handling from optional causal certification while dynamic admission remained
closed. The later public lifecycle and scientific controls below now cover
admitted complete v11 programs. Legacy v10 dynamic programs require regeneration.

## Native execution and ownership

The existing native exported callback metadata identifies contour callbacks
once when constructing an evaluator. Only those programs enter a scoped
callback-failure context during evaluation. Undeformed and fixed evaluators
retain their ordinary execution variants without dynamic thread-local work.

A downstream native or external operation can return a finite value after an
upstream root callback fails. A finite final vector is therefore insufficient
evidence that the evaluation succeeded. The dynamic evaluator wrapper rejects
the entire provisional vector on any callback failure. Since the native matrix
interface does not identify the failing row, it rejects the provisional matrix
and reuses the existing scalar precision-recovery lane for each point. Accepted
complete vectors and their sampling statistics are never fabricated or partially
submitted.

Conditioning has a separate failure scope because it is a diagnostic evaluation,
not the accepted numerical output. Double-float and arbitrary-precision caches
apply the same essential failure admission. Scoped guards restore an enclosing
caller's failure state, including on unwinding, and a failed batch cannot poison
the following healthy batch.

Independent review identified a preparation case too: Symbolica can evaluate
fixed-argument callbacks while mapping or compiling an evaluator. A failure
there now produces a typed construction error even if a later operation masks
its nonfinite value. A mathematically valid constant may still be unrepresentable
in the requested numeric domain; the test verifies that the same constant is
valid in native multiprecision rather than replacing it with zero.

## Evidence and limits

The original four focused controls passed on the failure-fence source. They use an actual
native root callback followed by an opaque consumer returning a finite value,
including mixed healthy/failing rows, real and complex outputs, eager and
SymJIT execution, native multiprecision recovery, a later healthy batch,
preparation-time constants and preservation of outer failure state. The root
coordinator independently reviewed the real, complex, conditioning and precision
cache paths; the foundation agent additionally identified and reviewed the
constant-mapping correction.

This reuses Symbolica's existing evaluator, native precision domains and
FastSecDec's complete-vector recovery. It introduces no replacement algebra,
automatic differentiation, numerical integrator, RNG or library worker pool.
Thin HEPKit callers receive the existing native error and status boundaries.

The separate complete higher-jet test exposed an association error in the IBP
path: exact symbolic restriction makes the endpoint radii equal, but native
jets still execute one callback at each distinct endpoint. Numerical-dual
lowering now tags each actual face separately; symbolic lowering retains the
union when algebra has already merged the restricted root. That increment
retained a strict guard against repeated execution of the same request; the
later direct-translation finding below refined that rule. The corrected complete
vector gate passes for both dynamic constructions and Taylor/IBP, including
eager, SymJIT, double-float and 192-bit execution. It observes exactly two Taylor
requests and three IBP requests per actual point. A focused native control also
checks that the full mathematical callback arguments remain unchanged before
the native derivative seeds. No additional CSE implementation or solver-result
memoization was needed.

## Public execution and terminal diagnostics (2026-10-10)

Public execution exposed valid native direct-translation IR that repeats the
same pure root call. The observer now coalesces only an exactly identical
rational candidate at the same precision within an admitted request bundle.
Conflicting values or precisions, unknown requests and missing required requests
still fail. Each callback returns its own native value and tracked uncertainty;
FastSecDec does not memoize or substitute numerical results. The separate native
owner optimization addresses redundant solves rather than correctness admission.

The public analytic target passes all three tests, including eight complete
complex endpoint cases with the original 8192-point quadrature, and a threshold
bubble exercising saved restoration, real pilots, Always/Pilot/Off transitions
and stationary negative-F behavior. The lifecycle filter first passed 18 tests
covering checked eager/SymJIT execution, atomic binding, actual candidate
certification, policy remapping, private pilot owners and accumulated counters.

The subsequent failure-only context slice passes 20 dynamic tests. A terminal
stationary pole reports F=0 and weighted-gradient magnitude zero only when the
retained native ball program certifies both. Its message qualifies this as
context evidence, not identification of a unique failing density branch or a
Landau classification. A canceled density remains finite and nonzero at that
same point. Root-free exact failure retains the native reason without claiming
a stationary certificate, and failed rebinding preserves the previous value.

Detached sectors retain only their referenced raw certificate bytes and compact
projection/runtime metadata. Raw decoding, ball mapping and diagnostic evaluation
occur only after terminal failure. Successful Off execution performs none of
this work; no hidden root solve is introduced. Missing source coordinates retain
the full [0,1] enclosure. The source and ownership boundary passed independent
foundation review.

The 20-test result is recorded in
`target/contour-dynamic-failure-context-tests.log` on the `1e1cb169` consumer,
before the subsequent `516beb37` owner transition. Full native, CLI and portable
gates on that new owner, actual WASM execution, installed bindings and the full
physical/performance acceptance matrix remain distinct evidence. These focused
results do not establish overall Phase B completion.
