# Phase-two API boundary audit

This is a source audit against the first-phase plan's requirement to retain
domains, coordinate maps, Jacobians, and branch/phase information separately
from numerical evaluation. It implements neither contour deformation nor GCAD.
The initial audit below identified missing retained map and branch metadata.
The subsequent implementation and independent review are recorded first. No
contour deformation or GCAD algorithm is implemented or implied by these types.

## Follow-up implementation and independent review

The phase-one metadata requirement is now represented by native library types
in `generation/metadata.rs`: `CoordinateMap`, `DomainAssessment`,
`FactorAssessment`, `ChartRecord`, and the single supported
`BranchPolicy::NoThresholdReal`. The map contains Symbolica `Atom` images and a
real measure Jacobian, with explicit projective-gauge metadata. These are the
same native objects used by `mapping::coordinates_from_parts` and `map_terms`;
there is no serialization boundary or second algebra implementation between
mapping, generation, and compilation. The Jacobian is documented as the
positive real measure factor used by these charts, not a complex orientation
prescription for a future contour.

Every original chart survives in the metadata, including charts identified by
complete-density symmetry and charts whose contribution is entirely exact.
The representative permutation states its direction explicitly, and the
optional kernel association distinguishes those exact contributions. The
association is computed by a representative lookup and one final chart pass;
an initial quadratic association loop was removed during review.

Domain assessment preserves factor-level certificates and distinguishes an
explicit interior assertion from a coefficient-sign certificate. It reuses the
existing admission checks. The `NoThresholdReal` policy allows the already
supported complex scalar weights while prohibiting continuation of singular
factors. Existing threshold, unresolved-face, and branch errors remain intact.

Portable kernel version two includes these records in scientific identity.
Only this external artifact boundary converts native Atoms to canonical text.
Loading recomputes the identity, parses native expressions, reruns existing
domain certification, calls the shared exact `SectorMap::validate`, rebuilds
the coordinate images and measure Jacobian with the same mapping constructor,
and checks chart associations. Map validation reuses Numerica's determinant;
it validates factor-valuation widths but does not claim to recertify their
values without the original supports. Version-one artifacts remain explicitly
without generation metadata and retain their original identity on reload and
re-emission. Observational timings remain outside scientific identity.

The focused gate passed all four `generation_metadata` tests: exact retained
pullbacks for the three supported domains, projective gauge meaning, symmetry
and exact-piece associations, certified/asserted identity and round trips,
semantic rejection even after a payload is re-signed, legacy preservation,
and zero-dimensional direct densities. The review caught an empty-parameter
rejection that incorrectly excluded valid zero-dimensional unit-cube and
orthant inputs; the constructor was corrected and an exact-seven round-trip
regression now passes. Evidence is `output/weighted-metadata-tests.log`.

No remaining actionable native-ownership or scientific metadata defect was
found in this review. This closes the concrete retained-metadata gap identified
below. It does not supply map composition, contour orientation, a continued
branch policy, GCAD cells, or their local endpoint proofs. The polynomial
admission, subtraction ordering, symmetry compatibility, and evaluator-selection
risks below remain requirements for the later phase.

## Initial audit findings

## Existing boundaries that can remain

- Native HEPKit `GraphIntegral` owns the diagram, `IntegralFamily`, kinematics,
  numerator, powers, and explicit measure multiplier. Phase-one admission
  rejects nonzero widths, complex masses, and unsupported propagator templates
  rather than losing those fields during quadratic-family conversion.
- `ParametricIntegrand` is a validated sum of factored Symbolica densities.
  `ParametricDomain` distinguishes a projective simplex, unit cube, and positive
  orthant. Projective homogeneity is checked before gauge fixing.
- `fastsecdec-sectors` uses exact Numerica geometry and deliberately has no
  Symbolica/CAS dependency. Its monomial `SectorMap` should remain an exact
  geometry result, not become a general complex-expression container.
- Generation separates domain checks, geometry, map substitution, symmetry,
  endpoint subtraction, and Laurent expansion. Symbolica already owns the
  substitutions, derivatives, polynomial arithmetic, and series required at
  these boundaries; phase two must continue to reuse those APIs.
- Kernel expressions already admit complex coefficients. Portable O2 and
  numeric arbitrary-precision evaluators support adjacent real/imaginary
  outputs. Integration carries the complete vector and covariance using real
  unit-cube coordinates, without owning the expression or graph.

Consequently the integration algorithms do not need to know whether a future
coefficient expression arose from a real chart or a complex contour. Any
complex contour Jacobian belongs inside that coefficient expression. The real
periodization or Havana sampling weight remains a separate numerical weight;
it must not replace or take the absolute value of the contour Jacobian.

### Initial gaps and coupling risks

| Boundary | Current implementation | Consequence for the next phase |
| --- | --- | --- |
| Coordinate map | `generation/mapping.rs::map_terms` builds `Vec<Atom>` images from the integer exponent matrix, substitutes them, and discards them. `GeneratedSector::map()` exposes only `SectorMap`. | General affine, algebraic, or complex map expressions cannot currently be retained or inspected through the generated API. This is an unmet plan requirement, not an implemented extension point. |
| Jacobian | `SectorMap` records an absolute integer determinant and coordinate powers; the mapped density incorporates them immediately. | These are correct measure data for the real monomial charts. A complex or oriented map needs its signed/complex Jacobian expression; overloading the existing positive determinant would be incorrect. |
| Projective map meaning | The fixed row maps a parameter to one; total homogeneity supplies the eliminated scale. | A consumer must distinguish a projective gauge chart from normalized simplex coordinates. General map metadata must preserve that gauge interpretation. |
| Domain assessment | Private `domain::check` returns `Result<()>`; errors distinguish a detected threshold, unknown domain, branch rejection, or unresolved boundary. Only the caller's assertion flag is retained in CLI provenance. | The generated result cannot report whether its accepted domain was certified or relied on an explicit assertion, nor which factors were certified. A successful assessment needs a retained structured record. |
| Branch prescription | Negative factors with noninteger/regulator-dependent powers are rejected. No explicit sheet, causal prescription, or continuation metadata exists in the parametric or generated types. | Complex scalar weights are supported, but this is not threshold/branch support. An assertion cannot authorize analytic continuation. Identical expression text can represent different continued branches in phase two. |
| Polynomial admission | Singularity factors must be regulator-independent polynomials; geometry works on their Newton supports. | A GCAD chart with algebraic boundaries or a contour pullback cannot simply be sent back through polynomial-support admission. The chart result needs a separate boundary after original polynomial admission and before local resolved-density processing. |
| Resolved endpoints | Residual checks require a nonzero constant and certified faces; subtraction assumes a regular residual at each lower cube face. Upper-boundary zeros are explicitly rejected pending affine charts. | Future chart providers must supply a valid local endpoint factorization. Applying a deformation after subtraction without preserving the corresponding boundary terms is not automatically correct. The ordering must be established by phase-two mathematics and tests. |
| Symmetry | The current native canonicalization candidate is verified by exact simultaneous substitution of the full scalar density on a cube. | If future cell/sheet/orientation metadata is not fully encoded in that density, it must also enter the symmetry compatibility check. Equality of algebraic expressions alone cannot prove equality on different sheets. |
| Kernel persistence | Kernel payload stores canonical coefficients, parameters, precision policy, and cancellation metadata; it omits maps, domain assessments, and branch data. CLI provenance adds input domain and assertion but no per-chart record. | Pure expression reload is sufficient for present evaluation, but future scientific identity and inspection must include branch/chart semantics or prove they are fully encoded. A versioned change is required before accepting phase-two artifacts. |
| Backend selection | `kernel::has_complex_coefficients` detects literal complex numeric coefficients. | This works for admitted phase-one complex weights. It is not a general complex-domain or logarithm-branch decision. Future branch policy must select the appropriate evaluator explicitly. |

The current errors are appropriate first-phase boundaries. In particular,
`assume_no_threshold` does not override a detected interior zero or unresolved
boundary geometry. Relaxing those checks is not an implementation of either
future strategy.

### Original minimal first-phase metadata proposal

The following is a concrete proposal for a separate coordinated change, not
an assertion that the types already exist. It deliberately adds no unused
contour/GCAD strategy variants or algorithm interfaces.

1. In the main library's parametric or generation metadata module, retain a
   `CoordinateMap` with source parameter symbols, target parameter symbols,
   `images: Vec<Atom>`, `jacobian: Atom`, source `ParametricDomain`, and optional
   projective fixed-parameter metadata. Populate it from the exact `SectorMap`
   using the same images/Jacobian already used by mapping. Keep the original
   exact `SectorMap` as geometry provenance. The general Atom fields establish
   the planned map boundary without changing the sector crate or inventing
   another algebra engine.
2. Return and retain a `DomainAssessment` recording certified acceptance or
   acceptance by explicit assertion, with factor-level outcomes where needed
   to distinguish them. Use a single explicit current branch policy such as
   `NoThresholdReal`, whose documented scope permits complex scalar weights
   but no singular-factor continuation. Do not add unsupported `Contour` or
   `Gcad` enum variants merely to make the API appear complete.
3. Carry those records through generated results and the versioned portable
   metadata/identity boundary. Separate observational timings from scientific
   policy as today. Define whether a kernel-only export intentionally omits
   inspectable source-chart provenance, while still retaining every item that
   changes evaluation semantics.

Ownership should follow existing responsibilities: main-library generation
owns map construction and assessment; `fastsecdec-sectors` stays unchanged;
kernel/artifact ownership handles semantic persistence; the CLI only presents
the retained records. Coordinate map conversion with the generation owner so
that the retained expressions and the expressions actually substituted cannot
drift into separate implementations.

Useful phase-one checks for that metadata change are exact equality between
retained monomial images/Jacobian and the generated pullback; correct
projective-gauge metadata; certified versus asserted-domain round trips; and
artifact identity changes for scientific policy changes while timing changes
remain irrelevant. These exercise the current implementation without adding
phase-two algorithms or synthetic strategy abstractions.

The later phase can then define how its chart provider certifies face behavior,
tracks continuation, and composes maps. Algebraic-root evaluation, contour
selection, and GCAD construction still require their own native ecosystem
API/source/probe audits before any implementation decision.

## Source evidence

- `FIRST_PHASE_PLAN.md`, “Domain assessment and phase-two readiness.”
- `crates/fastsecdec-sectors/src/types.rs`: domains and exact monomial maps.
- `crates/fastsecdec/src/parametric/integrand.rs`: validated polynomial density.
- `crates/fastsecdec/src/generation/{domain,mapping,subtraction,symmetry,types}.rs`:
  admission, local pullback, endpoint assumptions, symmetry and generated API.
- `crates/fastsecdec/src/kernel/{mod,complex,artifact}.rs`: numerical backend
  selection, complex evaluation and portable expression persistence.
- `crates/fastsecdec/src/integration/config.rs`: real/imaginary vector layout and
  caller-owned unit-cube sampling.
- `crates/fastsecdec-cli/src/artifact.rs`: current provenance and complete CLI
  artifact identity.

The initial boundary audit was read-only. The follow-up review inspected the
native metadata implementation and its artifact boundary and ran the focused
regressions described above; it introduced no phase-two algorithm.
