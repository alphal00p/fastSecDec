# Native function maps and coordinate aliases

This read-only audit covers Symbolica revision
`98794d0d7337ba2b08e4c046dde584ad7fc1ce10`, including the small recorded local
correctness patches. It establishes available interfaces and a bounded probe
design; it does not establish a performance improvement. The separate dependency
release audit owns the published-version verification and SymJIT update.

## Existing native interfaces

`symbolica/src/evaluate/function_map.rs` provides `FunctionMap`,
`FunctionRegistrationOptions`, `InliningPolicy`, and `EvaluatorBuilder`. Function
bodies are native `Atom` values with native indeterminate arguments; there is no
public `FunctionDefinition` type in this checkout. `examples/nested_evaluation.rs`
demonstrates nested and tagged functions using one native evaluator builder.

| Representation | Native behavior | Consequence for a comparison |
| --- | --- | --- |
| `FunctionMap::add_aliases` | Argument-free aliases always inline in their caller's scope; function-shaped aliases use fixed tags. | This can delay substitution until evaluator construction, but cannot test retained call boundaries. |
| `add_function_with_options(..., Always)` | Explicit arguments shadow matching global evaluator inputs. | Provides the explicit-argument function baseline. |
| `add_function_with_options(..., Never)` | Retains a separate native sub-evaluator. | The builder necessarily selects direct translation because the legacy tree linearizer expands calls. |
| `Auto` | Currently identical to `Always`. | It is not a fourth independent optimization mode. |

The builder also exposes direct translation, Horner/common-pair optimization
controls, hot starts, and an abort check. A timing comparison must record those
settings: `Never` versus the ordinary default changes both call retention and
the translation route. It must not attribute that whole difference to inlining.

## Symbolic boundary

An evaluator `FunctionMap` is not automatically consulted by `AtomCore::derivative`,
polynomial support extraction, domain certification, endpoint substitution, or
complete-density symmetry verification. An opaque alias symbol would incorrectly
look independent of a coordinate to those operations. Symbolica separately
offers `SymbolBuilder::with_derivative_function` (and keyed derivative callbacks),
but that does not supply polynomial support or endpoint values for an arbitrary
evaluator alias.

Consequently, a post-Laurent function-map experiment can establish evaluator
construction and runtime costs while retaining the existing scientific symbolic
path. It cannot demonstrate a reduction in earlier mapping, symmetry, subtraction,
or Laurent work. Carrying aliases through those stages needs a demonstrated native
representation supporting each relevant operation; this audit does not authorize
a custom alias algebra, second evaluator, or alternate U/F path. Existing native
substitution and the current Laurent template cache remain relevant reuse points.

## Numerical backends and persistence

`src/evaluate/evaluator.rs::coefficient_conversion_shares_function_bodies` tests
retained nested functions through real and complex coefficient conversion and a
bincode round trip. Immutable function bodies are shared; numerical stacks are
separate. `merged_functions_share_one_global_table_and_reuse_their_stacks` covers
native evaluator composition. `src/evaluate/backend.rs` registers function bodies
with SymJIT for both real and complex compiled numbers. Compiled evaluator cloning
is native; serialization is explicitly revision dependent.

The native HEPKit OneLOop provider is an existing ecosystem example, not merely a
synthetic API demonstration. `oneloopmaster/src/expressions.rs` exposes shared
`OneLoopExpressions::function_map` and exact/JIT evaluator builders. Its registered
triangle/box bodies use `InliningPolicy::Never`; its builder explicitly selects
direct translation and zero Horner iterations to preserve formula arithmetic.
`JitEvaluator::to_bytes` stores the numerical evaluator including nested function
definitions, and loading recompiles O2 for the host. These interfaces provide a
concrete comparison point for definition ownership and persistence; their
master-specific formulas and private definition wrapper should not be copied.

FastSecDec already derives O2 code, conditioning evaluation, and MPFR rescue from
one exact `ExpressionEvaluator<Complex<Rational>>`. Its worker-local precision
cache uses native `map_coeff_with_prec` for `Float` and `Complex<Float>`. That is
the appropriate common integration point for a function-map probe. The generic
source supports it; an executable nested-function MPFR/complex/weighted-rescue
probe is still required before claiming compatibility in FastSecDec.

`FunctionMap` has native bincode support with Symbolica state mapping. It is not a
public serde-JSON definition table. FastSecDec's current portable artifact stores
canonical coefficient atoms and recompiles them; it does not store function-map
bodies or inlining policy. Persisting calls alone would therefore lose their
definitions on a cold load. Any adopted retained representation must preserve its
definitions, scope, numerical policy and identity across fresh processes, using
native serialization or a thin declarative transport adapter. Internal processing
must continue to use native objects. Hidden complex coefficients in function
bodies must also be accounted for when choosing the real or complex backend.

## Bounded measurement design

Use the same complete Laurent vector and inputs for fully substituted expressions,
always-inline aliases, ordinary always-inline functions, and retained functions.
Control direct translation separately. Record symbolic stage times independently
from exact evaluator construction, O2 compilation, artifact size/load, ordinary
evaluation, and precision rescue. Do not infer generation savings from execution
throughput or compare only scalar outputs when the workload is a complete vector.

Before any production choice, verify the full vector at interior points and
simultaneous boundary approaches, complex coefficients, a fused weighted MPFR
rescue, native worker cloning, and cold-process persistence. Measure both a small
case and the observed rank-five bottleneck; retain identical precision settings
and record checks/replays. Current evidence identifies native APIs to reuse but
does not favor a particular inlining policy.
