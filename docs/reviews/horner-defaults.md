# Native evaluator optimization settings

Date: 2026-10-07. Implementation evidence and compatibility controls for the
requested Horner/CPE defaults and exposed evaluator tuning.

## Native reuse and boundaries

The pinned Symbolica revision `58652fabc2f736302a570deaaf8d517679f7fe6e`
provides `EvaluatorBuilder::optimization_settings` and public scalar setters in
`src/evaluate/function_map.rs`. FastSecDec's `CompilationSettings` is a typed,
serializable adapter to that existing owner; it introduces no optimizer,
expression translator or graph representation.

| Run-card field under `[generation.evaluator]` | FastSecDec default | Native mapping |
| --- | --- | --- |
| `horner_iterations` | 10 | `horner_iterations` |
| `cpe_rounds` | 1000 | `cpe_iterations(Some(1000))` |
| `cores` | 1 | `cores` |
| `max_horner_scheme_variables` | 500 | Same-named native setter |
| `max_common_pair_cache_entries` | 1000000 | Same-named native setter |
| `max_common_pair_distance` | 1000 | Same-named native setter; currently stored but unused by the pinned optimizer |
| `verbose` | false | Native tracing output |
| `direct_translation` | true | Native direct/tree route selection |

`cpe_rounds=0` disables CPE rounds; `"unlimited"` maps to native `None`.
`max_cpe_rounds` is a supported alias. Native CPE defaults to unlimited, so the
new explicit 1000 cap is a FastSecDec default, not a claim about an upstream
default. On the direct route, native `horner_iterations=0` immediately
linearizes and bypasses subsequent CSE/CPE optimization. The distance setting
has no effect in this pinned owner: source searches find only its field,
setters and serialization. The cache and variable limits are active native
controls. No missing native distance algorithm was reimplemented here.

Native direct Horner optimization uses its seeded RNG and deterministic initial
variable ordering. More than one internal optimizer core can race between
equal-multiplication candidate schemes with different addition counts; settings
therefore validate `cores=1`. Caller-owned sector dispatch remains parallel and
retains ordered admission. The native owner runs its single-core operation on
the caller; FastSecDec adds no library worker pool. Hot-start expression trees,
function/alias registration and cancellation callbacks are structural inputs,
not TOML scalar knobs. The callback-only abort level is deliberately excluded.

Native verbose logging uses Symbolica's `info!` macro, which can initialize a
stdout tracing subscriber. CLI generation therefore requires plain output and
rejects JSON/status-JSON when verbose is requested. Loading saved evaluators
does not start another optimization-log session: reconstruction of a runtime
exact-offset helper disables only transient logging while retaining the saved
settings and every numerical optimization option.

## Compilation and persistence

Existing `GeneratedIntegral::compile*` and `to_kernel_bytes` methods now use the
new defaults. Configurable serial and caller-dispatch methods receive the same
`CompilationSettings`; `to_kernel_bytes_with_settings` exposes the eager
native-program route without host JIT. Sector programs and runtime-dependent
exact-offset evaluators use those controls. Local SymJIT O2 and portable eager
arithmetic are unchanged. Auxiliary literal-zero recognition and runtime mass
predicate evaluators retain their separate, deliberately fixed zero-Horner
settings.

The existing artifact `compiler_policy` string now carries versioned canonical
JSON containing every scalar setting. No binary payload schema was extended,
and the TOML-facing integer/string CPE representation never goes through an
unsupported untagged bincode decoder. Exact native program bytes remain owned
by Symbolica. Kernel and sector representation identities bind the selected
settings, including settings that may leave arithmetic unchanged.

Historical policy strings decode as Horner 0 and CPE unlimited, with the prior
native defaults for other fields. Loaders retain their original envelope bytes
and content ID. The legacy policy spelling used for sector identity is
preserved. Runtime exact-offset reconstruction uses the decoded settings rather
than the new generation default. Human generation metadata records the actual
settings supplied; older metadata leaves this observational field absent.

## Focused evidence

`cargo check -p fastsecdec -p fastsecdec-cli --locked` and the native debug build
passed. Ignored probes and inputs reside in `output/horner-options/`.

- `core_probe.rs` generated six complex sectors and compared all scalar outputs
  at three interior points under new defaults and explicit legacy settings.
  Serial and reversed parallel completion produced identical bytes and IDs.
  Eager native-program serialization matched the compiled native artifact.
- The same public-API probe exercised zero/bounded/unlimited CPE, reduced
  variable/cache/distance values, and direct/tree routes; every saved setting
  survived load and byte-preserving re-save. Unsupported optimizer core counts
  were rejected. Runtime-parameter sectors and a runtime-dependent exact-only
  offset compiled, loaded and bound successfully.
- CLI generation used default, explicit override and unlimited run cards. Each
  saved `generation.evaluator` value matched the requested settings. Separate
  cold CLI inspection processes recovered those same records. Negative counts,
  unknown fields, unsupported cores and an invalid unlimited spelling failed
  without producing an artifact.
- Both the preserved version-5 ggHH artifact and the current pre-change ggHH
  artifact loaded with legacy settings and byte-identical re-save. Kernel IDs
  matched their saved records; sector 0 identities matched independent output
  from the preserved old executable for both artifacts.
- A copied historical completed checkpoint resumed against the old cheap
  artifact with zero new evaluations. Its whole mean, full covariance and every
  sector estimate were identical to the old report. User artifacts/checkpoints
  were not overwritten.

The independent native-owner probe in `output/horner-review/native-horner.txt`
found a real optimization effect: its three-component complex example changed
from 143 additions/396 multiplications to 102 additions/124 multiplications,
and exact IR size from 3358 to 2042 bytes. Six serial builds and four concurrent
caller builds produced identical optimized IR, and the full vector agreed with
the zero-Horner route. This is a focused example, not a ggHH runtime speedup
claim. The independent portable/HEPKit reuse review is recorded in
`horner-reuse-review.md`; final CLI/release validation is recorded by the root
coordinator. Permanent test migration and other examples remain deferred.
