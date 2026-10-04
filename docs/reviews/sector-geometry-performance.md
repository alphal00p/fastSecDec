# Exact geometry performance review

Date: 2026-10-04. Baseline: milestone `f559059`, with a diagnostic test harness
that reports phase times and an ordered-map fingerprint. Rust 1.98.1, Linux
x86-64, AMD EPYC 9754. Both Numerica and `fastsecdec-sectors` were compiled by
Cargo in **release mode with thin LTO**. The older approximately 125-second
mixed-profile observation is not used as the baseline.

## Measured bottleneck and exact correction

The unchanged nine-dimensional stress fixture combines 70 U monomials and
105 F monomials. Unique Minkowski sums leave 266 candidate vertices, all of
which are actual vertices in the final decomposition. Extra hull pruning
therefore cannot remove any of these constraints.

A `perf` CPU-clock profile at 99 Hz collected 9,079 samples with none lost.
About 89.2% of leaf samples were in `BTreeSet` subset checks and their iterators;
exact dot products accounted for 1.23%. Facet construction took 99.385 seconds
under profiling, compared with 0.526 seconds for triangulation. Integer/rational
linear algebra was not the measured bottleneck and remains unchanged.

The correction applies an inexpensive necessary condition before the existing
exact double-description adjacency test. Adjacent extreme rays span a
two-dimensional face of a full-dimensional cone. Its common active constraints
have rank `dimension-2`, so their cardinality must be at least that large.
Smaller intersections can be rejected before allocating a set or scanning
every other ray. Cardinality is not used as a sufficient test: the original
third-ray incidence test still decides adjacency for every surviving pair.

Every intermediate cone contains the final full-dimensional cone for the
supported homogenized inputs. Hence the codimension condition applies at each
insertion. The filter changes neither exact arithmetic, constraint ordering,
accepted-ray ordering, resource limits, nor progress/cancellation points.

## Before/after evidence

Unprofiled runs used the same release test harness and no competing project
benchmark. A preserved baseline executable avoids relying on subsequent source
edits to reconstruct the measurement.

| Combined U/F stress case | Baseline | Cardinality filter |
|---|---:|---:|
| Total decomposition | 99.351 s | 11.276 s |
| Facet construction | 98.829 s | 10.748 s |
| Triangulation | 0.517 s | 0.522 s |
| Peak resident memory | 12,288 KiB | 12,288 KiB |
| Vertices | 266 | 266 |
| Sectors | 3,496 | 3,496 |
| Ordered exact-map fingerprint | `37d5b05efde1ff85` | `37d5b05efde1ff85` |

This is an **8.81-fold reduction** for the same geometry input. The fingerprint
includes every ordered exponent matrix, exact determinant, Jacobian power,
factor valuation, and fixed-parameter identity. It uses Rust's `DefaultHasher`
with one fixed toolchain as a diagnostic, not as a portable scientific artifact.

All 11 author geometry tests and all eight independent geometry tests passed in
release mode after the change. These include exact polynomial moments, sampled
cone coverage, nonsimplicial fans, projective measure, infinity charts,
rank-deficient inputs, resource errors, and cancellation.

The coordinator independently checked the cardinality argument and retained
exact subset test. Redundant active constraints can only weaken the filter;
they cannot cause it to discard an adjacent pair. This review and the existing
rank-deficient and nonsimplicial cases did not identify a correctness issue.

A post-filter 199 Hz profile collected 1,894 samples with none lost and took
11.389 seconds. Set iteration/intersection/subset operations still dominate
(about 76% of leaf samples), with exact dot products now 12%. A bitset incidence
representation is a possible later optimization; it is not needed to justify
this small correction and has not been introduced in this milestone.

Commands and ignored raw evidence:

```
cargo test -p fastsecdec-sectors --release --locked
cargo test -p fastsecdec-sectors --release --test hard_geometry --locked --no-run
output/probes/geometry/baseline-hard-geometry --exact hard_four_loop_fan --ignored --nocapture --test-threads=1
target/release/deps/hard_geometry-854c46f7cc35f301 --exact hard_four_loop_fan --ignored --nocapture --test-threads=1
```

Logs, `/usr/bin/time`-compatible resource measurements, profile data, and the
preserved baseline executable are under ignored `output/probes/geometry/`.

## Physical hard fixture versus geometry stress test

The physical hard integral has fixed exponent `U^1`: U is a regular polynomial
numerator and need not refine its singularity fan. F has exponent `eps-3` and
does refine it. The generation layer owns that factor selection; the combined
U/F probe remains useful as a distinct exact-geometry stress test. The separate
ignored `hard_four_loop_singular_f_only` probe checks the 105-monomial F support.

| F-only physical singularity fan | Baseline | Cardinality filter |
|---|---:|---:|
| Total decomposition | 0.298 s | 0.295 s |
| Facet construction | 0.026 s | 0.022 s |
| Triangulation | 0.272 s | 0.273 s |
| Peak resident memory | 9,216 KiB | 9,216 KiB |
| Vertices | 105 | 105 |
| Sectors | 2,760 | 2,760 |
| Ordered exact-map fingerprint | `a408b0879c606dd7` | `a408b0879c606dd7` |

The physical fan is already inexpensive; these subsecond runs do not establish
a meaningful speed change. Their purpose is to distinguish it from the much
larger combined-support stress case and confirm fingerprint agreement.

These measurements are a local geometry optimization result, not a matched
Pathfinder end-to-end performance acceptance. No precision policy or scientific
fixture was relaxed to obtain them.
