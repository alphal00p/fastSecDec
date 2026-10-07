# Native evaluator optimization review

Date: 2026-10-07. Independent dependency/source review and focused native probes; final FastSecDec policy wiring is reviewed separately below. No optimization algorithm or dependency implementation is changed by this review.

## Public native API and defaults

The pinned Symbolica revision is `58652fabc2f736302a570deaaf8d517679f7fe6e`. `src/evaluate/function_map.rs:389–476` exposes builder configuration directly; `OptimizationSettings` and its native defaults are at lines 501–661. FastSecDec can pass its scalar settings to this existing owner and continue constructing one complete Laurent-vector evaluator, with its existing aliases and ordered input schema.

| Public setting | Native default | Meaning / relevant limitation |
| --- | --- | --- |
| `horner_iterations` | 10 | Variable-order search budget; zero takes the direct unoptimized translation branch. |
| `cores` | 1 | Optimizer-internal worker count; use one for stable caller-owned sector parallelism. |
| `cpe_iterations` | `None` | Common-pair elimination until convergence. `Some(n)` caps rounds; zero skips this stage. FastSecDec's requested default is `Some(1000)`. |
| `max_horner_scheme_variables` | 500 | Caps candidate Horner variables; zero can retain CPE without Horner factoring. |
| `max_common_pair_cache_entries` | 1,000,000 | Native common-pair candidate cache control; zero still retains the native first admitted candidate and can produce an optimization. |
| `max_common_pair_distance` | 1000 | Publicly stored and serialized, but currently **unused by the pinned optimizer**. |
| `verbose` | false | Native optimization logging. |
| `direct_translation` | true | Native translation route; this still performs Horner/CPE when Horner iterations are positive. |

The builder also exposes `hot_start`, `abort_check`, and `abort_level`. `hot_start` carries native expression objects, and the direct route explicitly rejects a supplied hot start. `abort_check` is a caller callback. `abort_level` is transient internal cancellation/early-exit state, reset by optimization, rather than a resource-search setting. These are distinct from the scalar tuning surface.

For the direct route, `src/evaluate/tree.rs:323` returns `linearize_multiple` immediately when Horner iterations are zero. Thus CPE budgets/cache/distance settings do not affect that bypass; retaining this owner behavior is preferable to adding a second optimizer path. With positive Horner iterations, native CPE rounds end early at no changes (`tree.rs:393–406`).

The distance limitation was verified by a whole-source search: all `max_common_pair_distance` references are storage, setters, defaults, serialization and Python parameter plumbing; no optimization reads it. In contrast, the cache setting is read in `src/evaluate/evaluator.rs:963–965`. Configuration may preserve/pass through the distance value for public-API parity, but documentation must not claim it changes instruction locality in this pinned version.

Native `verbose` uses Symbolica's `info!` macro. When needed, that macro initializes its own tracing subscriber with a default stdout writer (`src/lib.rs:259–273`), subject to the environment log filter. A focused `RUST_LOG=info` run produced native optimization records on stdout, not stderr. The CLI therefore requires explicit `--plain` and forbids `--json`/`--status-json` for verbose generation before terminal setup. This preserves native logging behavior without contaminating structured output or installing global library logging policy. The library continues to leave logging ownership to its caller.

## Determinism and caller ownership

The native Horner search uses `MonteCarloRng::new(0, i)` for each optimizer worker (`tree.rs:462–465`), rather than thread randomness. Numerica's existing RNG initializes from a fixed seed and stream (`lib/numerica/src/numerical_integration.rs:1651–1667`). With one native optimizer core, the candidate order and accepted decisions are serial and deterministic; different sectors may still build concurrently in the caller's pool.

The native core count changes iteration partitioning and RNG streams. A four-core control produced a different, repeatable scheme from the one-core control. The shared-best update also compares multiplication counts before a mutex-protected update, and accepts equal multiplication counts without requiring lower additions; this source does not justify a general scheduling-independent guarantee for arbitrary internal core counts. No nondeterministic result was reproduced by the bounded four-core control. Requiring one native optimizer core preserves the established deterministic boundary without forking the dependency, implementing a new search, or inventing a seed API.

## Focused native evidence

The probe builds a three-output, six-input polynomial vector, including a purely imaginary output, through public native `Atom::evaluator_multiple`, `OptimizationSettings`, and exact evaluator serialization. With the requested CPE cap 1000, changing Horner 0 to 10 reduced native counts from **143 additions / 396 multiplications** to **102 additions / 124 multiplications**, and exact evaluator bytes from **3358** to **2042**. Six serial builds, four simultaneous caller threads, and two separate process executions produced identical ten-iteration bytes. Three nontrivial signed/dyadic point controls preserved the full complex vector within their declared floating comparison tolerance. Exact serde/bincode reload preserved bytes and evaluation.

A second native control isolates CPE using the upstream public test pattern of three overlapping sums. Zero rounds retain seven additions; one round reduces this to five. Caps 1000 and unlimited converge to identical exported instructions. Reducing the candidate cache to zero yields six additions, demonstrating an active native control. Distance zero and 1000 produce identical exported instructions, consistent with the source finding. All six configurations return exactly `[10, 8, 13]` on the integer control point.

The four-native-core diagnostic repeated 30 builds plus four concurrent callers without an observed byte mismatch, but returned 98 additions / 132 multiplications and different IR from one core. This is a limitation diagnostic, not acceptance of arbitrary internal core counts. These small operation-count observations establish that the owner settings are active; they do not establish ggHH generation or runtime performance.

Evidence is ignored under `output/horner-review/`: `native_horner_probe.rs`, `native-horner.txt`, `native-horner-second-process.txt`, `native_cpe_probe.rs`, `native-cpe.txt`, and the four-core diagnostic. No permanent tests or other examples were changed.

## Artifact and portable boundaries

The settings choose the exact evaluator at generation. Native f64 JIT and eager evaluation, DoubleFloat, and arbitrary-precision evaluators continue to derive from that same exact IR; loading must deserialize the saved IR rather than rerun Horner according to a new default. The existing codec already serializes native optimization settings. Compiler-policy metadata must record the requested scalar settings, while accepting the historical zero-Horner spelling under its original backend compatibility checks. Original legacy bytes and content identities must remain intact on load.

Native eager evaluation is exercised by the focused probes, independently of SymJIT compilation. Horner itself operates on Symbolica native Atoms and exact coefficients, so the portable feature need not acquire a native-code dependency. The one-core setting also avoids native optimizer thread creation; Symbolica clamps optimizer thread capability to one on wasm32. This source review is not a Wasm/Pyodide execution test. Full portable-feature integration and actual ggHH performance remain separate coordinating checks.

## FastSecDec implementation inspection

The scalar adapter is accepted on source review. `CompilationSettings` passes every supported setting to native `OptimizationSettings`, defaults to Horner 10/CPE 1000, accepts `max_cpe_rounds` as an alias for `cpe_rounds`, and represents unlimited rounds explicitly. Both deserialization and the Rust entry point reject internal core counts other than one. Serial and dispatched compilation carry the same copied settings; runtime-dependent exact offsets receive them as well. Auxiliary mass predicates and native literal-zero proof helpers retain their intentionally simple zero-Horner configuration.

Binary and legacy loaders read settings from the backend-specific compiler-policy string, preserve saved exact IR, and retain the original artifact bytes/content identity. Old zero-Horner strings map explicitly to legacy settings with unlimited CPE rather than inheriting the new defaults. Fresh policies include the complete scalar settings, and sector representation hashes bind those policies. The existing codec layout remains unchanged because the versioned policy already occupies a string field. Backend/codec checks, input/output validation and content digests remain in force.

The CLI reads the run card's evaluator settings, validates them before generation, sends them through its existing bounded compilation dispatcher, and records the actual settings in optional generation metadata. Older human metadata without that record remains absent rather than acquiring a fictitious new default. Cold loading reconstructs only the runtime exact-offset helper with transient verbosity disabled, then restores the saved settings on the kernel set; this prevents a saved verbose setting from writing native optimization logs during inspection or integration. No numerical integration loop, precision classifier or estimator changed in this slice.

## Portable-host acceptance and bounded limitation

The isolated portable manifest compiled the updated core successfully. Three existing complex controls passed: native canonical complex coefficient precedence, complete complex Laurent layout through portable serialization, and real/imaginary covariance under scalar weighting. The separate non-Gamma native-alias control `an_unused_reserved_looking_regulator_is_not_a_coordinate_image` also passed. A final `cargo check --manifest-path tests/portable-kernel/Cargo.toml --locked` passed after the cold-load logging adjustment.

An ignored portable-feature host probe generated the complex runtime-parameter family `i*x^(-1+eps)/(1+m*x)` and exercised four configurations: default Horner 10/CPE 1000; Horner zero/unlimited CPE; Horner one with zero CPE, variable and cache caps; and the native legacy translation route with CPE two. All four preserved settings and exact evaluator bytes across artifact reload, rebound `m` independently at two values, and matched the complete analytic complex vector at points selecting f64, DoubleFloat 106 bits and arbitrary precision 3322 bits. This tests the new settings through the native eager backend, including zero boundaries and the nondefault translation path. It is a portable-feature host run, not a Wasm/Pyodide execution claim.

The remaining existing complex test, `gamma_constants_and_complex_boundary_rescue_remain_numeric_on_workers`, was intentionally stopped after about 5 minutes 43 seconds under the coordinating time bound. Its sampled stack was performing Symbolica `polygamma_checked` through arbitrary-precision constant mapping in Astro arithmetic at the 3322-bit distance-policy tier. There was no completed assertion or numerical failure before interruption. This high-precision Gamma test and the other Gamma-dependent alias test remain unverified in this milestone; the combined test command therefore does **not** count as a passing suite. No production precision setting was weakened, and no test was edited to avoid that cost.

Evidence is retained under `output/horner-review/`: `portable-compilation.txt` and `portable_compilation_probe.rs`, `portable-host-tests.txt`, `portable-alias-light.txt`, `portable-gamma.sample.txt`, and `portable-final-check.txt`. All probe sources and raw output remain ignored; permanent tests and other examples remain unchanged.
