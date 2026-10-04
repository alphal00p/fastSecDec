# Native formal-function compatibility proof

This is a disconnected correctness experiment under `output/probes/formal_function_series.rs` and `formal_function_series/numeric.rs`. It changes no production generation/evaluator path. The preceding Series-first experiment remains a negative optimization result; its finite-order large-expression comparison is unfinished.

## Scope and native ownership

The proof represents only epsilon-independent polynomial residuals as functions of their actual coordinates. Symbolica performs coordinate differentiation (including a composed-argument mixed partial), Gamma/Laurent series, endpoint substitution, and native derivative-body evaluation. The adapter reads native `der` multiindices and caches the corresponding native partial bodies; it introduces no chain rule or series arithmetic. Native literal replacement maps the resulting calls to fresh user-function slots, avoiding the native fixed-builtin restriction on retaining `der` itself.

Always and Never policies use the same explicit direct translation and O2 settings. A native FunctionMap supplies bodies to one complete-vector evaluator. Definitions and output Atoms are both included in size accounting. Native exact evaluator bincode transports the retained bodies; the fresh reader receives no independent definition table. Intended checks cover eager/O2 agreement, real and explicitly complex definitions, worker clones, native error tracking, 512/1024-bit weighted boundary evaluation with scaling before binary64 conversion, and cold native-IR reconstruction.

The independent analytic control uses `(2+3i)*(1+x)` with endpoint powers `x^(-2+epsilon)*y^(-1+epsilon)` and Gamma(epsilon), giving the coordinate-independent density `Gamma(epsilon)*(2+3i)*(1/(epsilon*(epsilon-1))+1/epsilon^2)`. It must contain all orders −3 through 0. Rational cases retain y dependence and composed polynomial arguments. An explicit admitted face control uses `P=1+x+x*y`: its positive face constant and polynomial partial bodies ensure retained calls at literal zero cannot introduce inactive `0*log(0)` or `0/0`. This is not a claim about hiding arbitrary log-containing coefficient bodies; those would require separate native face specialization/certification.

HEPKit's read-only review corrected a pre-execution oracle error: the initial `(1+x+y)` analytic example had a y-dependent subtracted remainder, so its pointwise coefficients could not equal a fully integrated analytic answer. The revised coordinate-independent control avoids that mismatch. The review also requested and received mixed native derivative tags, missing/wrong-arity builder errors, and explicit nonempty-vector guards.

## Initial executions exposed a native series defect

`output/diagnostics/formal-functions/small-1` retains the first writer failure: a constants-only complex-detection assertion found no imaginary constant. The expanded diagnostic in `small-2` showed that **both formal and concrete coefficient maps were empty**, making its apparent successful numerical loops vacuous. Neither run is accepted as FunctionMap compatibility or performance evidence. There is no demonstrated complex-constant predicate defect from these runs. Subsequent source includes a nonempty guard in the numeric helper and an exact analytic order-list guard, before any value comparisons.

The source mechanism was independently identified in Symbolica `src/derivative.rs`: generic function-series fallback substituted its expansion indeterminate through `.replace(x.clone())` at lines 514 and 534. `src/id.rs` converts an underscored `Indeterminate::Symbol` to a wildcard pattern. The proof deliberately names its regulator `eps_`, exposing this native path when Gamma's regularized function series is evaluated.

The minimal independent program `output/probes/underscore_gamma.rs` confirmed the failure with the same native release library: `Gamma(eps)` expands to `-gamma_E + 1/eps` with two terms, while `Gamma(eps_)` expands to zero with no terms. Raw output is `output/probes/underscore_gamma.txt`. Source/binary/library hashes for the preceding proof runs are retained alongside their captured sources; no dependency patch preceded the reproduction.

The coordinator authorized an essential native correction and assigned HEPKit the minimal literal-substitution patch, native regression, and FastSecDec Gamma/underscored-regulator complete-vector regression. At that point the formal-function proof awaited the correction before rebuilding and rerunning. Complex backend selection in this bounded proof remains explicit from the complete known source definitions; it is never inferred from sampled values or output-only inspection.

Independent review of that patch found the two literal substitutions sufficient for the demonstrated mechanism: both the indeterminate pattern and expansion-point replacement remain literal, including function indeterminates. Native regression cases cover Gamma and an unknown two-argument function around 0 and 1, plus the independently known Gamma pole/constant. The FastSecDec control checks all four orders −2 through +1 of `Gamma(eps_)/eps_` against explicit constants. Both focused regressions passed after the native rebuild. No pole counting, Gamma arithmetic, FunctionMap implementation, or complex-constant predicate is replaced. The corrected compatibility proof links the resulting Symbolica library (`libsymbolica-1f655858224cb588.rlib`, SHA-256 `dda215a39b38358772677beed211e773c9b028f33f0251dc1072135fe37f2e51`).

## Corrected nonempty proof: passed

After the fixed-library workspace gate, `output/diagnostics/formal-functions/small-3` recorded a successful bounded writer (4.015 s) and a distinct cold reader (2.360 s). Both had 180 s limits. These are compatibility diagnostics using an opt2 dependency/debug library and O2 JIT, not release performance comparisons. The directory retains exact source/library/binary hashes, archived Rust source, dependency-patch evidence, native evaluator files, complete numerical output vectors, and process reports. The native revision is `98794d0d7337ba2b08e4c046dde584ad7fc1ce10` with the four recorded local corrections; SymJIT is 2.26.4.

All three cases under both policies contain the complete, nonempty order vector `[-3,-2,-1,0]`. Every restored formal coefficient agrees exactly with the concrete native coefficient, and the analytic case also agrees with its independent Gamma/rational expression. Eager and O2 evaluations agree at all three interior points. Native error tracking, cloned numeric worker evaluation, and weighted boundary evaluation at `x=1e-20` pass. The latter multiplies at native precision before conversion and checks 512/1024-bit results at `1e-50 * max(1, scale)` without reducing the precision comparison to binary64. Native mixed-partial/composed-argument, literal-zero admitted face, and missing/wrong-arity definition controls also pass.

| Case | Definitions | Definition Atom bytes | Formal output Atom bytes | Always native IR bytes | Never native IR bytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| Analytic complex | 2 | 27 | 2,025 | 992 | 1,268 |
| Rational real | 5 | 44 | 9,854 | 1,970 | 2,407 |
| Rational complex | 5 | 64 | 9,854 | 2,022 | 2,433 |

The cold reader imports only native exact-evaluator bincode, without a separate function-definition table, and repeats the full numerical/precision/worker checks. Its successful Never cases verify that retained bodies survive that transport. The diagnostic constants inspection now detects imaginary literals in the two complex-source cases and none in the real case; the earlier failure was entirely explained by empty output vectors. This observation is scoped to these polynomial definitions, not a new general realness classifier.

## Status

The bounded small compatibility proof passed. No production representation change, performance advantage, or actual-representative FunctionMap validation has been established. The approved next actual proof will bind only already admitted polynomial residuals and compare all orders against a separately authored native point-first oracle from the original captured production subtraction, retaining the full domain/face and cancellation context. The earlier failed attempts remain part of the evidence.
