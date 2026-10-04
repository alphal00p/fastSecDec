# Native FunctionMap experiment on actual rank-five coefficients

This experiment uses the generated complete Laurent coefficients of the native high-rank box fixture, with orders `[-2,-1,0]`. It selects the representative with the largest total coefficient Atom size and records all twelve representatives' sizes. It then applies the explicit cube bijection `t_i=u_i^2` with real measure factor `product_i(2*u_i)`. This is a controlled additional pullback of real generated coefficients, not a replacement for the original sector maps or an alias-aware subtraction system. The integral is unchanged, but pointwise runtime includes the additional coordinate and measure work.

The unchanged production evaluator is a separate baseline at the corresponding `t` points. Pulled-back modes compare direct Atom substitution, native aliases, native functions with `Always`, and native functions with `Never`. Builder and SymJIT direct-translation settings vary independently; native `Never` internally forces the direct builder. All JIT kernels use O2. Sizes include coefficient Atoms **and definition-body Atoms**, plus the full native evaluator IR; they are representation byte counts, not process-memory estimates.

Initial compatibility and preparation measurements passed 51 cases: three rotated repetitions of sixteen pulled-back settings and the unchanged production baseline. All modes evaluate the same 16,384 fixed interior points and all Laurent outputs. Every scalar and native batch vector agrees with the production interpreted values multiplied by the measure factor; the largest observed scaled difference was `4.91e-14`. Native conditioning evaluators are checked at fixed points. The boundary check doubles each axis's cancellation degree under `t=u^2` and compares native MPFR production values with fused-weight pullback values on caller-owned workers. Exact native IR is then decoded in a fresh process, recompiled, and checked again.

The selected representative is sector 3, dimension three, with 45,305 coefficient Atom bytes. Original symbolic generation took 11.211 seconds in this run and is separate from all evaluator measurements. The numerical representation experiment cannot reduce work that has already happened during parameterization, mapping, subtraction or Laurent expansion.

For the production builder and JIT translation settings (builder direct, JIT indirect), the initial median preparation/build/compile measurements are:

| Representation | Atom bytes including bodies | Native IR bytes | Representation preparation | Native builder | O2 JIT |
| --- | ---: | ---: | ---: | ---: | ---: |
| Unchanged production | 45,305 | 4,939 | 0.014 ms | 15.417 ms | 2.384 ms |
| Direct pullback substitution | 62,417 | 5,019 | 2.080 ms | 18.980 ms | 2.444 ms |
| Native aliases | 45,380 | 5,019 | 0.060 ms | 15.788 ms | 2.423 ms |
| Always-inline functions | 71,015 | 4,997 | 2.028 ms | 18.906 ms | 2.422 ms |
| Retained functions | 71,015 | 5,315 | 2.013 ms | 18.856 ms | 4.414 ms |

This shows that the alias representation can avoid materializing an extra substituted Atom while reaching comparable native evaluator IR. It does not yet justify changing production representation or artifacts. Initial runtime samples lasted only a few milliseconds; their small differences are not evidence of a throughput advantage. A longer probe keeps the same buffers and correctness checks, uses seven rotated repetitions and 64 timing passes (1,048,576 full-vector evaluations per scalar or batch measurement), and records its results in a separate directory.

Initial evidence: `output/function-map-rank-five-release.log` and `output/probes/function-map-rank-five-2.26.4-release/{report.json,medians.json,write.log,read.log}`. All 51 cases, including cold reconstruction, completed in 16.41 seconds. The backend is SymJIT 2.26.4 with the same Symbolica revision and essential local patches documented in [the compatibility review](function-map-compatibility.md). No production alias path or new symbolic algebra was added.

## Longer repeated runtime measurements

The longer probe passed all 119 cases and fresh-process reconstructions in 60.88 seconds. Each scalar or batch measurement performs 1,048,576 complete-vector evaluations on the same fixed buffers, with seven rotated repetitions. Native buffer warmup and numerical comparisons remain outside timed loops. No other root-team build or symbolic workload ran during the measurements; unrelated host workloads were not controlled. Original symbolic generation took 11.294 seconds, separate from evaluator timing, and the maximum numerical difference remains `4.91e-14`.

With the production translation settings, times below are nanoseconds per **complete three-coefficient vector**. Brackets give the minimum and maximum among seven measurements, not statistical confidence intervals.

| Representation | Scalar median [range] | Native batch median [range] |
| --- | ---: | ---: |
| Unchanged production | 169.01 [168.78, 169.42] | 76.09 [75.82, 77.86] |
| Direct pullback substitution | 176.17 [176.02, 178.42] | 77.73 [77.59, 77.97] |
| Native aliases | 171.75 [171.67, 172.08] | 77.64 [77.48, 78.22] |
| Always-inline functions | 170.34 [170.09, 170.57] | 77.16 [76.84, 77.22] |
| Retained functions | 197.11 [196.61, 198.35] | 83.92 [83.36, 85.45] |

Native aliases reduce scalar time by approximately 2.5% relative to direct pullback substitution in this controlled example; their batch timings are essentially the same. Aliases also avoid materializing the larger substituted Atom: median preparation is 0.102 ms versus 2.181 ms, and native building is 15.840 ms versus 19.079 ms. Retaining these very small coordinate functions adds calls and is slower. Forcing SymJIT direct translation yields 274–298 ns scalar and 107–115 ns batch for the four pulled-back representations, despite slightly shorter compilation. Builder translation and JIT translation therefore must not be conflated.

These measurements support native aliases as a compatible compact representation when a numerical pullback is already needed. They do not justify replacing the current phase-one production artifacts or claiming a broad speedup: the current generation has already performed its original symbolic pullback, and most measured generation time is elsewhere. The unchanged production kernel remains the baseline; no extra cube substitution should be introduced solely to obtain these measurements. Native batching is materially faster here, but an eventual production batch lane must retain the existing per-point conditioning, weighted replay and complete-vector semantics.

Long-run evidence: `output/function-map-rank-five-release-long.log` and `output/probes/function-map-rank-five-2.26.4-release-r7-n64/{report.json,medians.json,write.log,read.log}`. All-set coefficient Atom size is 370,078 bytes; all twelve individual sizes, both translation flags, operation counts, full body-inclusive sizes, native IR sizes, precision results and each repeated timing remain in the raw report. The initial three-repeat evidence remains intact.
