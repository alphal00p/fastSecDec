# LTD fixture provenance and import research

2026-10-09. Research for the multiloop acceptance suite in
[CONTOUR_DEFORMATION_PLAN.md](../../CONTOUR_DEFORMATION_PLAN.md). No fixture
generation or multiloop integration was performed for this review.

## Immutable source pin

The requested [arXiv v2 source](https://arxiv.org/src/1912.09291v2) was fetched
successfully. Its SHA256 is
`23add23135a2431284d5db0af4083a4198f22615cd2e5a554abba9599dcd10cf`.
The source archive contains a nested RAR, not directly exposed YAML files:

| Member | SHA256 |
| --- | --- |
| `ancillary_material/ancillary_material.rar` | `97efb556021a988b999e60c86854f6591942cfe3d674a0b9b228b4cdfedd700d` |
| `paper_topologies.yaml` inside that RAR | `6ad6216da1051a0ff860f80f14cf9b4534cb85aa0f38d48bac06a0e968dd0b68` |

Local research copies and the five selected numerical records are under
`target/contour_ltd_research/`; none should be committed as raw benchmark or
third-party archive data. Native fixture production should retain the source
URL, these digests, selected record identity and the explicit imported values.
Use a standard archive reader and standard YAML parser during one-time data
inspection; neither becomes a FastSecDec production dependency.

## Selected records and independent reference status

The record names below exist in the pinned ancillary file. Counts are internal
propagators, with ordinary unit powers; the source routing fields are
`loop_lines[].signature`, `propagators[].q` and `m_squared`. Use the table's
stated uncertainties, not a zero-filled ancillary reference field.

| Approved case | Propagators | Reference in the paper's measure |
| --- | ---: | --- |
| `2L4P.b.K1` | 7 | `(-1.0840618909337886 + 2.8682065140371712 i) × 10^-6`, ancillary analytic value |
| `2L4P.b.K1*` | 7 | Table 6: `(2.8020 ± 0.0080) × 10^-6 + i (3.3450 ± 0.0080) × 10^-6`, pySecDec |
| `2L6P.a.I` | 9 | Table 2: `-86.080 ± 0.090`, pySecDec; `-86.600 ± 0.800`, independent momentum-space reference |
| `3L4P.K1` | 10 | `(-2.424229927669986 - 3.4003495610895258 i) × 10^-9`, ancillary analytic value |
| `4L4P.a.I` | 12 | **Unreconciled archival mismatch; exclude from validated fixtures for now.** |

The published entries and their interpretation are in
[Tables 2, 3, 6 and 8](https://arxiv.org/html/1912.09291v2#S7.SS1).
The massive K1 record stores a zero analytical-result placeholder. The
four-loop ancillary I record instead has empty deformation data and entirely
spacelike channels, unlike the Table 3 threshold benchmark. Do not relabel
that Euclidean record as the required contour stress test.

This four-loop discrepancy affects an extended case, not the required two-
and three-loop records. It needs reconciliation against the author's original
fixture source before execution. The required six-point numerical references
also carry finite precision and a small published disagreement: retain both
references and uncertainties, never treat the ancillary rounded `-86.07` as
an exact analytic answer or tune a result toward a selected central value.

## Routing identity and the b/c naming issue

The two-loop K1 records have loop-line signature/multiplicity pairs
`([1,0],3)`, `([1,-1],1)`, `([0,1],3)`. This is the seven-propagator planar
double ladder. Its three-loop extension has pairs
`([1,0,0],3)`, `([1,-1,0],1)`, `([0,1,0],2)`,
`([0,1,-1],1)`, `([0,0,1],3)`.

The corresponding archive diagrams are `diagrams/2L_4P.tex` and
`diagrams/3L_4P.tex`. The distinct `diagrams/2L_4P_AB.tex` depicts the massive
quark topology discussed elsewhere in the prose. Names alone therefore cannot
identify a graph. The native importer must verify the full denominator family
and masses, not repair the prose's b/c inconsistency by guessing a label.

For reproducible record selection, these hashes cover each original YAML
block from its `- analytical_result_imag:` line through its `name:` line,
inclusive, with original LF newlines:

| Record | Block SHA256 |
| --- | --- |
| `2L4P.b.K1` | `59bfcbf61d28a140bc3f9215881a7baa70750c26300e16ee76a5807416fe79d9` |
| `2L4P.b.K1*` | `9f90309d10ee3619be6d20e62dc624408a53ff64b920d21023b7fe21e43ecb6e` |
| `2L6P.a.I` | `d54201da0da26ec64c4b00039314f1800b5fb75ea146e438eacf04491c46870d` |
| `3L4P.K1` | `260e69f718f848e3221777c4909245396893c2c7cf56a428442b62ca768f4066` |
| `4L4P.a.I` | `71613ab64143f16575c889fb36c9ed7928e2535fee666ca442d4897841e2f609` |

## Native import protocol

The later implementation should create an ordinary HEPKit diagram/family and
kinematics fixture. Reuse `FeynmanDiagram::from_dot`,
`FeynmanDiagram::propagator_family`, native `IntegralFamily::new` and
`Kinematics`; use Linnet to validate graph incidence, cycles and graph identity.
No separate DOT parser, graph class, loop-routing solver or Symanzik builder is
needed. FastSecDec already accepts these native objects and delegates U/F
construction to them.

For every imported propagator, independently match the native expression
`(sum_a signature[a]*k_a+q)^2-m_squared` to the archived routing, up to an
irrelevant overall sign of its momentum. Preserve source order as a provenance
map even if native graph edge IDs choose another order. Keep unit powers,
scalar numerator and mass bindings explicit. Verify the native loop count and
denominator count before sector generation; then compare native U/F families
under the documented parameter permutation. These are source-validation
checks, not a new graph canonicalization algorithm.

Archived decimal four-vectors contain ordinary binary floating rounding.
Preserve their printed precision and state how native momentum conservation
is enforced. Check the final dependent momentum and archived propagator
shifts against the supplied values at their actual precision. Do not silently
claim every independently rounded decimal identity is exact. Use native
kinematics for Minkowski scalar products and record any necessary roundoff
reconciliation. The massive K1 record's `m_squared=0.16000000000000003`
represents the stated common mass 0.4; preserve this provenance explicitly.

## Measure conversion

FastSecDec's native scalar measure is documented in
`crates/fastsecdec/src/parametric/scalar.rs` as
`d^D k/(i*pi^(D/2))`. Algebraically, conversion to the paper's
`d^4 k/(2*pi)^4` for a finite L-loop result is

`I_paper = (i/(16*pi^2))^L * I_fastsecdec`.

This rotates real and imaginary components at odd loop order and changes the
sign at two loops. Apply the multiplier to the entire complex vector with
native Symbolica, and transform its covariance consistently; never compare
unconverted real and imaginary entries. If higher epsilon orders are later
requested with the dimensionally continued `(2*pi)^(-D)` measure, the
additional factor is `(4*pi)^(L*epsilon)`. No implicit Euler-gamma factor is
part of FastSecDec's default measure.

Before accepting this normalization for multiloop fixtures, test it through
the existing native HEPKit one-loop reference route. The paper's analytic
ladder values are cross-check targets; implement no new special-function
formula without the mandatory HEPKit/Symbolica reuse audit.

The next scientific sequence remains analytic controls, the required two-loop
records, then `3L4P.K1`. Published LTD evaluation timings have no direct
FastSecDec hardware/backend parity interpretation. Fresh pySecDec reference
runs retain the ten-minute/15-GB ceiling.
