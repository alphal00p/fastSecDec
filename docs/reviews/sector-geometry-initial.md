# Independent sector geometry review

Date: 2026-10-04. Reviewer: the Numerica/QMC implementation agent, independently
of the sector implementation author. Scope: the initial exact geometry slice in
`crates/fastsecdec-sectors`. No implementation files were changed during review.

**Result:** no correctness defect found in the reviewed geometry slice. The
foundation can proceed with the scope limitations below. This is not a claim of
completed sector symmetry reduction, full integral generation, or performance
parity with Pathfinder.

## Mathematical checks

- **Support prefilter.** A vertex of a Minkowski sum has a unique decomposition
  into exposed vertices of its summands. A support sum with distinct
  decompositions cannot be a vertex, so discarding repeated sums is exact.
  Repeating the prefilter after each factor is valid: a discarded nonvertex of
  a partial sum cannot become an exposed vertex after adding another summand.
  The input constructor deduplicates equal monomials first. The implementation
  does not claim that every surviving candidate is a vertex.
- **Homogenization and signs.** Rows `(p, 1)` represent the support points. The
  dual inequalities `p·r + b >= 0` yield supporting normals of minima. At a
  vertex, incident rays satisfy equality. Their first coordinates therefore
  generate the normal cone appropriate to `x_i = product_j t_j^M_ij`, since
  `-log(x) = M(-log(t))`. Cube recession rows `(e_i, 0)` impose nonnegative
  logarithmic coordinates. Omitting them for a positive orthant permits signed
  exponents and the required infinity charts.
- **Double description.** Exact independent rows initialize an enclosing
  simplicial cone. Positive/negative ray pairs use the positive combination
  that cancels the inserted constraint. Common active constraints and the
  absence of a third ray on their common face give the combinatorial adjacency
  test. Primitive reduction preserves ray orientation. For the public inputs,
  the homogenized primal cone is pointed and the dual is full dimensional;
  cube recession or the explicit orthant affine-rank check supplies full rank.
  Thus lower-dimensional intermediate-cone behavior is not silently assumed.
- **Pulling triangulation.** Recursion cones the first ordered ray over facets
  that exclude it. Intersections with the original inequalities provide the
  faces; exact rank selects codimension-one facets and deduplication removes
  repeated incidences. The construction partitions a pointed cone into
  simplices up to boundaries. Exact determinant checks reject singular maps.
- **Jacobian and valuations.** For a full-dimensional monomial map, the
  absolute Jacobian is `abs(det M) product_j t_j^(sum_i M_ij - 1)`; the code
  uses this expression including negative powers. Valuations are coordinate
  minima, and the simultaneous constant-residual check confirms that each
  factor attains them in one original monomial.
- **Projective powers.** Primary charts fix the largest projective parameter
  to one. The returned matrix is a gauge-fixed map, not normalized simplex
  coordinates. For an `n`-parameter density homogeneous of degree `-n`, its
  common scale cancels the normalized-chart measure. The crate enforces
  individual polynomial homogeneity and documents that the caller must check
  the total density degree, including powers and measure.

## Independent execution evidence

Command: `cargo test -p fastsecdec-sectors --locked`, using Rust 1.98.1 on Linux
x86-64 and the native toolchain recorded in `docs/DEVELOPMENT.md`.

All 11 author tests and all 8 new independent tests passed. The new tests live
only in `crates/fastsecdec-sectors/tests/independent_geometry.rs` and check:

1. Repeated Minkowski summands, including interior support points, retain all
   exposed extreme vertices and cover the full logarithmic domain.
2. Individually deficient factors can have a full-rank sum; truly deficient
   orthant input returns a typed error, while deficient cube input remains valid.
3. A four-dimensional cross-polytope exercises nonsimplicial normal cones and
   recursive pulling. Exact inverse-map membership finds neither overlap nor
   gaps at sampled interior points, including mixed-sign logarithms.
4. Twelve varied simultaneous-support fixtures conserve cube volume and
   several independent polynomial moments exactly as rationals. Additional
   exact cone-membership checks detect compensating overlaps and gaps that a
   single volume test could miss.
5. Translating a support leaves the normal fan and Jacobian unchanged while
   changing factor valuations by the exact expected linear amount.
6. Projective charts with secondary sectors integrate normalized simplex
   measure and a first moment to `1/2` and `1/6`, within `2e-12`, using a
   separate fixed Gaussian quadrature over the returned maps.
7. Mixed infinity maps integrate `1/(1+x+y)^3` over the positive orthant to
   `1/2`, within `2e-12`, and pass the exact cone-membership check.
8. Support, ray, and sector resource limits return typed errors; cancellation
   after work has begun remains a cancellation, not an empty success.

The ordinary test run deliberately leaves the nine-dimensional hard probe
ignored. Its author ran the optimized probe separately and reported 266
vertices, 34 facets, and 3,496 sectors in approximately 125 seconds. That result
was not independently rerun here and is not a paired performance baseline.

## Remaining acceptance boundaries

- Finite independent examples and source review are strong checks, not a
  machine-checked proof of the geometry engine. External scientific fixture
  comparisons and the hard-example performance gate remain required.
- The positive-orthant rank error is a geometry diagnostic, not a scalelessness
  certificate. Callers must not convert it into a zero integral.
- The expression layer must verify total projective homogeneity before using
  these gauge charts and preserve numerator/prefactor powers. The independent
  projective test verifies the documented contract, not that future callers
  obey it.
- Cancellation happens at explicit progress points. One exact linear-algebra
  operation or pair-selection scan is not preemptible; no strict latency or
  process-memory bound is promised by the current resource settings.
- Integrand-aware sector equivalence, geometry caching, serialization of
  stable identities, and parallel chart scheduling are outside this initial
  slice. The plan still requires them where applicable.

No source-level fix is requested by this review.
