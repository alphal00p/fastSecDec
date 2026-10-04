# Bounded rank-five generation profile

The ignored unit probe `generation::profiling::rank_five_native_factor_collection` uses the native graph and point from `examples/runs/box_high_rank_numerator.toml`, with measure multiplier one. It invokes the actual production generation path. Test-only instrumentation separates incidence-graph construction from native Graphica canonization and records mapped Atom byte sizes and graph vertex/edge counts. Production code has no new profiling API or altered algebra.

Two development-build variants were measured with SymJIT **2.26.0**: unchanged final Gaussian factors, and native `AtomCore::collect_factors()` on Polynomial-role factors only. The singularity factors and exponents remain unchanged. Exact native sparse-polynomial equality is checked only for these bounded input factors; the full density is never expanded for the comparison. The probe limits input Atom size and chart count, and checks a 120-second budget at existing progress callbacks. A native CAS/canonization call may cross that budget before returning. Both partial runs report `completed: false`, so neither is a complete generation timing. The subsequent release comparison uses SymJIT **2.26.4** and is recorded separately.

| Variant | Charts reached | Generation before cancellation | Mapping | Symmetry |
| --- | ---: | ---: | ---: | ---: |
| Unchanged | 10 / 12 | 140.253 s | 22.196 s | 118.037 s |
| Native final-factor collection | 12 / 12 | 138.009 s | 2.555 s | 135.447 s |

Gaussian parameterization took 2.637 s. Collection itself took 0.002 s and reduced the four numerator factors from 4812/4440/2680/2942 bytes to 2579/1545/1474/1577 bytes. Input validation and the separate equality check took 3.192 s unchanged and 0.635 s after collection. These are diagnostic timings, not a matched reference benchmark.

Matched expensive charts expose the main structural issue: mapped densities are approximately 17.7 MB unchanged and 4.84 MB after final-factor collection, despite their compact Gaussian inputs. Their incidence graphs still have about 2030 vertices and 5200–5500 edges. Encoding falls from about 1.46 s to 0.42 s per such chart, while canonization remains about 19–31 s. Other charts are much smaller and cheap. The stored records contain individual chart measurements, not an average over mismatched chart coverage.

Graphica and FastSecDec generic callers are unoptimized in the development profile; only selected numerical/algebra dependencies have development optimization. Release measurements are required before attributing an algorithmic bottleneck to Graphica or making performance claims. No dependency patch is justified by this profile.

Final-factor collection alone does not solve the mapped-expression growth. The existing signed-polynomial monomial stripping can materialize a large residual. The separate native capability probe passed: collection **after substitution**, removal of the already-proven coordinate monomial, and native recognition that the residual is polynomial in the integration coordinates preserve compact powers. The implemented fast path passed the mapped-factor and scientific regressions, including signed orthant maps and hidden cancellation. The existing signed sparse fallback remains for hidden algebraic cancellation or unsuccessful recognition. See [the native reuse evidence](native-monomial-stripping.md) and [independent review](native-monomial-stripping-independent.md). This is reuse of native algebra, not a new factorization or valuation engine.

Evidence: `output/probes/rank-five-generation-profile.log` and the paired `rank-five-generation-{unchanged,collected}.json` reports. The probe passed, including exact equality checks; cancellation was its explicitly recorded work bound, not a claimed completed generation.

## Controlled release comparison on SymJIT 2.26.4

Both variants use the same release test binary, the unchanged Gaussian input, the same native libraries and all twelve charts. A test-only switch selects the previous signed sparse monomial stripping or the production factored fast path with that same sparse fallback. No other root-team build or symbolic process overlapped these measurements. Unrelated host workloads were neither controlled nor modified. Each run completed, including the native input-factor equality checks.

| Stage | Sparse stripping | Factored stripping |
| --- | ---: | ---: |
| Mapping | 23.599 s | 11.797 s |
| Symmetry | 8.308 s | 0.0107 s |
| Subtraction | 0.853 s | 0.00993 s |
| Laurent expansion | 62.497 s | 0.248 s |
| Complete generation | 95.328 s | 12.066 s |

Gaussian parameterization was 2.234/2.242 seconds and separate input validation was 2.603/2.729 seconds. Those setup stages are outside the generation totals. Both runs used the unchanged final Gaussian factors; neither uses the earlier optional final-factor collection variant. The generation ratio is approximately 7.90, an internal representation comparison rather than a Pathfinder acceptance benchmark. Numerical correctness is established separately by the scientific tests, including the native rank-five reference comparison in the 202-test backend gate.

On the expensive matched charts, mapped densities shrink from approximately 17.7 MB to 7–8 KB, with incidence graphs shrinking from roughly 2030 vertices to 133–137. Release canonization was already much faster than the earlier unoptimized development result, but preserving the factored Atom removes the remaining downstream graph and Laurent blowup as well. Mapping now dominates. Further optimization should measure the native support extraction and repeated per-chart algebra before changing their ownership or algorithms.

Evidence: `output/probes/rank-five-release-symjit-2.26.4-{sparse,factored}.log` and `output/probes/rank-five-generation-release-{sparse,factored}-unchanged.json`. The test-only switch is not a production setting. The binary is `target/release/deps/fastsecdec-22be010dd7f54ed7`; its build log is `output/rank-five-release-build.log`.
