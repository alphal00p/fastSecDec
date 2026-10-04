# Native formal-function and evaluator ownership audit

This is an independent source/API audit for the difficult on-shell generation
case. It complements the reference implementation trace and the test-only
Series-first experiment; it does not select a new production subtraction
strategy. No symbolic process was started for this audit. The proposed early
polynomial-function adapter still needs its focused executable proof.

## Published versions and audited source

At **2026-10-04 22:55 UTC**, fresh downloads of the authoritative crates.io
sparse-index entries report the latest unyanked stable releases as
[Symbolica 3.0.1](https://index.crates.io/sy/mb/symbolica) and
[SymJIT 2.26.4](https://index.crates.io/sy/mj/symjit). These match Cargo.lock.
The downloaded indexes, their SHA-256 hashes and the registry package checksums
are retained in `output/probes/native-formal-functions/release-check.json`.
The Symbolica checkout remains the documented pinned revision
`98794d0d7337ba2b08e4c046dde584ad7fc1ce10`, including six upstream commits
after the 3.0.1 tag and the separately documented local fixes. This is not a
claim that the checkout is byte-identical to the published crate.

All paths below are relative to
`DO_NOT_PUSH_FOR_REFERENCE_ONLY/worktrees/symbolica/` unless stated otherwise.
The three reuse checks are the public API/source, upstream examples/tests, and
the already executed FastSecDec FunctionMap compatibility/persistence probes.
The last check proves the evaluator facilities, not the new early symbolic
adapter.

## Symbolic ownership already exists

`src/derivative.rs:60` recognizes native
`der(depth_1,...,depth_n,f,arg_1,...,arg_n)` calls. Repeated differentiation
increments the selected depth; argument differentiation and the chain rule
remain native. The ordinary unknown-function path constructs these same tags
at line 200. There is no reason to implement a partial-derivative or chain-rule
engine in FastSecDec.

`src/derivative.rs:432` immediately returns a constant native Series when an
expression does not contain the expansion variable. Consequently an admitted
epsilon-independent polynomial represented by `P(x_1,...,x_n)` remains an
epsilon-constant coefficient, while differentiation with respect to its
coordinate arguments remains correct. Upstream `series_user_function` and
`series_derivative_keeps_function_unevaluated` tests at lines 1013 and 1029
exercise native argument composition and derivative tags. Unknown functions
are not coordinate-independent symbols: substituting a coordinate on a face
must change their arguments.

The promising insertion point is **after actual polynomial/domain/map
admission and before repeated endpoint differentiation**. The existing native
polynomials remain authoritative for support, valuation, homogeneity,
positivity and branch certification. Hiding them before those checks would
require expanding their definitions for those operations and would defeat the
existing ownership boundary. Preserve the admitted mapped factor objects if
needed; do not infer a second polynomial factorization scheme from a large
combined expression.

After all symbolic operations, collect only derivative multiindices actually
present. Compute each body through the existing native `Atom::derivative`
operation and cache that result once. This is bookkeeping over native bodies,
not a new differentiation algorithm. Zero bodies should remain native exact
zeros. Native Series bounds, factorials, regulator poles, complete vector
assembly and the current conservative cancellation rules still apply.

## FunctionMap is the Rust definition API

`src/evaluate/function_map.rs` provides `FunctionMap`,
`FunctionRegistrationOptions`, and `InliningPolicy::{Always,Never,Auto}`.
`FunctionDefinition` is a Python wrapper name, not a separate Rust facility.
`Auto` currently behaves as `Always`. Argument-free aliases always inline;
they are not a mechanism for preserving a shared instruction body.

`add_function_with_options` and `add_tagged_function_with_options` accept native
Atoms as bodies. A symbol has one fixed tag count in a map. Definitions see
global evaluator inputs and their own formal arguments, which shadow globals;
they do not capture caller-local arguments. The native implementation rejects
wrong arity, duplicate definitions and recursive definition cycles. Using the
same coordinate arity for every opaque residual in one kernel keeps derivative
tag handling straightforward; different kernels can have separate maps.

There is one source-level limitation to respect. Although the expression
lowerer checks FunctionMap before undefined-function fallback
(`src/evaluate/tree.rs:1287`), a retained `Symbol::DERIVATIVE` call subsequently
passes through the fixed-builtin branch (`tree.rs:749`, `state.rs:732`). The
native derivative symbol is a fixed builtin, and that branch assumes a unary
builtin evaluator. Registering the derivative symbol directly with `Never`
therefore does not establish a working retained derivative body.

The narrow adapter is to leave native derivative tags untouched during algebra,
then use native **literal** substitutions to fresh ordinary function slots
whose bodies are those native partials. An alternative is an Always wrapper
for the derivative tag which calls a fresh Never function. Either option must
be proved with a mixed partial and a composed/face-substituted argument; no
upstream patch is justified before that test. Matching caller symbols as
patterns would reintroduce the previously fixed underscore-name bug.

The builder uses direct translation whenever a map contains a Never function
(`function_map.rs:463`), even if legacy tree optimization was requested. An
Always/Never comparison must report the effective builder path as well as JIT
translation and O2 settings. It must measure expression bytes **plus body
bytes**, native IR size, construction, compilation, loading and evaluation.

## One native IR covers all numeric and portable paths

The output of `Atom::evaluator_multiple(...).function_map(map).build()` is the
existing native `ExpressionEvaluator<Complex<Rational>>`. It contains the full
multioutput instruction program and retained function bodies; FastSecDec does
not need a second evaluator or separate U/F runtime.

`src/evaluate/evaluator.rs:105–219` implements native serde and bincode encode/
decode for this evaluator. `src/evaluate/external.rs` serializes retained
`FunctionBody` instruction data, tags and fixed arguments, and resolves native
external implementations when required. Immutable function bodies are shared
through `Arc`; cloned evaluator containers have independent reusable stacks.
The body serialization has its own native version marker which rejects an old
body format. The enclosing FastSecDec artifact must still bind the exact native
dependency/schema identity and preserve all current layout, domain, precision
and scope metadata. It should transport the native evaluator representation,
not invent another serialization of FunctionMap definitions.

Native `map_coeff_with_prec` provides f64, conditioning and arbitrary-precision
coefficient conversion. Non-inlined polynomial body literals are hoisted into
the common root constant table by `tree.rs:817–927`; `get_constants()` borrows
that table rather than recursively examining arbitrary external callbacks.
Thus a complex coefficient present only in a retained polynomial body must affect
backend selection even if it is absent from the small outer Atom. Existing
output-Atom-only scans are insufficient for such a representation. The native
`is_real()` method checks coefficients, not a general theorem that arbitrary
functions or branches have real outputs; retain the admitted phase-one branch
and native-function contracts. Test a hidden complex body explicitly, including
an identically zero imaginary component and a nonzero component under rescue.

The existing FastSecDec test-only files
`kernel/function_map_probe.rs` and `kernel/function_map_probe/rank_five.rs`
already exercised one exact native IR through portable O2, conditioning,
weighted Float rescue, caller-owned workers, and fresh-process bincode loading.
The 32-case capability matrix and the later 119-case rank-five measurements
are recorded in their existing review documents. Those probes include native
callback initialization on cold load. They prove that retained definitions do
not have to be reconstructed from a second transport schema.

Current production artifacts instead store canonical coefficient Atoms and
rebuild evaluators. They cannot simply start storing formal calls without their
native compiled definitions. Any adopted compact representation must use a
versioned native-IR payload and preserve identity/legacy-load behavior; this
audit does not authorize silently changing the existing artifact contract.

## Other native capability checked

`ExpressionEvaluator::vectorize` and native `Dualizer` already implement numeric
automatic differentiation. The generic external-function dualizer builds
components through native symbolic derivatives and explicitly rejects an
unresolved native derivative tag (`src/evaluate/dual.rs:610`). It is not evidence
that an arbitrary unknown polynomial function will automatically become an
executable derivative callback. A future numeric-AD experiment must use this
native facility; the present proposal has a simpler boundary with explicit
native polynomial partial bodies and one ordinary evaluator.

## Focused proof required before the difficult case

Use a small admitted residual with mixed partials, coordinate composition and
endpoint substitutions. Compare complete Laurent vectors to the unchanged
production path, including a prefactor pole and mixed-sign numerator. Exercise
Always and Never, hidden complex body coefficients, a near-boundary weighted
rescue on an independent worker, and a cold native-IR reload. Missing derivative
definitions and wrong arity must fail explicitly. Check the full coefficient
vector and retained cancellation metadata, not just one interior scalar.

Only after this proof should the captured ordinal-39 difficult case be timed.
Record body construction and size before series/subtraction, coefficient/IR
size, full build/load and numerical costs, including maximum and average
per-sample cost and rescue tails where measured. This is a capability recovery
attempt first; the negative Series-first result is retained, and no performance
claim follows from smaller outer expressions alone.

## Initial proof attempts: rejected empty-vector evidence

The first disconnected writer stopped at its hidden-complex assertion. The
next diagnostic writer revealed that both the formal and restored-baseline
Laurent maps were empty: every case had `orders: []` and an evaluator with zero
outputs. Therefore neither that assertion nor the subsequent successful empty
numeric loop is evidence for or against complex FunctionMap support. Both
attempts remain under `output/diagnostics/formal-functions/small-{1,2}`; neither
is accepted as a compatibility pass. The separate mixed-partial and admitted
face controls did evaluate nonempty vectors.

The follow-up review identified an exact source mechanism to test before any
dependency change. Generic function Series fallback in `src/derivative.rs:514`
and `:534` converts the expansion `Indeterminate` to a replacement pattern.
`src/id.rs:148–174` converts a trailing-underscore symbol to a wildcard, whereas
the proof deliberately uses a regulator ending in `_`. A minimal ordinary-name
versus trailing-underscore Gamma series comparison confirmed the defect:
`Gamma(eps)` has the expected pole and finite term, whereas `Gamma(eps_)`
returns no terms. The two literal substitutions and regression scope are
recorded in
[`symbolica-literal-series-variable.md`](../dependency-patches/symbolica-literal-series-variable.md).
The focused native regression and the FastSecDec Gamma/endpoint complete-vector
regression both pass against the corrected dependency. The guarded
formal-function writer and cold reader still require their own rerun.
No native evaluator classification defect has been established.
The corrected proof must require a nonempty complete vector, the analytic
control's explicit orders `[-3,-2,-1,0]`, and finite nonzero analytic values;
choosing the complex backend from its known source definitions is conservative
and does not rely on sampled realness.

## Corrected small proof: independent outcome review

After the literal-Series correction, `small-3` contains six distinct cases:
analytic complex, rational real and rational complex, each with Always and
Never. Every case retains orders `[-3,-2,-1,0]` and 32 finite values: four
complex coefficients at three ordinary points and one weighted boundary
point. The writer and separate cold reader both exited successfully under
their 180-second caps. Their roughly 4.0- and 2.4-second observations include
debug-dependency execution and coarse process sampling; they are not accepted
generation or throughput benchmarks.

Independent inspection verified the case matrix, nonempty order vectors,
native-IR byte lengths, finite values, policy agreement and source/library
hashes. Hidden complex constants are present for both explicitly complex cases
after the corrected series produces actual coefficients. Thus the original
empty-vector failure did not reveal a constants-table defect. Source inspection
also confirmed nonempty guards, restored exact identities, a composed mixed
partial, missing/wrong-arity errors, admitted positive polynomial faces, cloned
numeric workers, conditioning, O2, and cold native-IR reconstruction without a
separate source-definition table.

An additional numerical check used the independent analytic coefficient vector
of `Gamma(eps)*(2+3i)*(1/(eps*(eps-1))+1/eps^2)`. Its maximum scaled ordinary
binary64 discrepancy is `1.3547764533932096e-11`, within the proof's existing
`2e-10` tolerance. A tighter `2e-14` check does not pass: the unsimplified
endpoint subtraction still incurs ordinary floating-point cancellation.
The weighted multiprecision endpoint agrees with the analytic coefficients to
`2.0849665056442675e-16` after binary64 conversion. These observations do not
establish uniformly accurate raw binary64 evaluation. Full independent
inspection data is retained in `small-3/independent-review.json`.

This accepts the bounded native-API compatibility proof, not a production
representation change or the difficult representative. The proposed actual
adaptation must continue to use admitted epsilon-independent polynomial bodies,
native partials and literal substitutions, preserve all orders and cancellation
metadata, and reject unsupported unregulated captures before introducing
opacity. Its complete-vector comparison remains against the separately
authored point-first oracle.
