# LTD fixture provenance and import research

2026-10-09. Provenance and native import review for the multiloop acceptance
suite in [CONTOUR_DEFORMATION_PLAN.md](../../CONTOUR_DEFORMATION_PLAN.md).
The Rust fixture importer and its input-only checks are described below;
multiloop sector generation and numerical integration have not been performed.

## Immutable source pin

The requested [arXiv v2 source](https://arxiv.org/src/1912.09291v2) was fetched
successfully. Its SHA256 is
`23add23135a2431284d5db0af4083a4198f22615cd2e5a554abba9599dcd10cf`.
The source archive contains a nested RAR, not directly exposed YAML files:

| Member | SHA256 |
| --- | --- |
| `ancillary_material/ancillary_material.rar` | `97efb556021a988b999e60c86854f6591942cfe3d674a0b9b228b4cdfedd700d` |
| `paper_topologies.yaml` inside that RAR | `6ad6216da1051a0ff860f80f14cf9b4534cb85aa0f38d48bac06a0e968dd0b68` |

Local raw research copies are under `target/contour_ltd_research/`; the archives
and raw YAML remain untracked. The four admitted numerical records are retained
as a small explicit fixture in `examples/contour/ltd/ancillary-records.json`,
including their original record identities and block hashes. Native fixture
production retains the source URL, these digests and explicit imported values.
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

The implementation in `crates/fastsecdec/examples/ltd_contour/` creates ordinary
HEPKit diagram/family and kinematics fixtures. It reuses
`FeynmanDiagram::from_dot`, `FeynmanDiagram::propagator_family`, native
`IntegralFamily::new`, `Kinematics` and `FourMomentum`; native graph/basis
validation and Linnet incidence supply the topology checks.
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

The implemented reconciliation uses Numerica's exact `Rational::try_from(f64)`
for every stored source component, preserving its binary floating value rather
than declaring all separately rounded decimal identities exact. Only HEPKit's
chosen dependent external vector is replaced by minus the sum of the independent
all-outgoing vectors. Source shifts and conservation are checked to `5e-14`,
and the stated exact mass `2/5` is checked against archived mass-squared within
`5e-15`. The exporter records the actual residuals and exact reconciled vectors.

## Ecosystem reuse evidence

Public API inspection found all required operations in existing owners:

- HEPKit's diagram propagator-family API retains ascending internal edge order
  and delegates quadratic denominators and loop routing to its native basis.
- `LoopMomentumBasis::validate` checks its spanning-tree complement; its
  `route_expression` replaces native `Q(edge)` atoms. The importer uses that
  operation also for the dependent external edge, rather than writing a new
  external-momentum substitution convention.
- `IntegralFamily::symanzik` supplies both graph and independently reconstructed
  archived-denominator U/F. Exact comparison uses Symbolica's native polynomial
  expressions; expansion is confined to these necessary finite identity checks.
- `FourMomentum<Atom>::dot` supplies the mostly-minus metric. Native
  `Kinematics` supplies all formal loop/external scalar products. Symbolica's
  native polynomial coefficient view checks homogeneity and U coefficient signs.
- Source inspection of Numerica's `TryFrom<f64> for Rational` confirms exact
  binary conversion; no decimal-rational approximation helper was introduced.

These were checked in the pinned HEPKit sources `integrals/diagram.rs`,
`integrals/parametric.rs`, `routing.rs`, the graph's native basis validation,
`feynkit-kinematics/src/momentum.rs`, and Numerica's rational conversion source.
The executable third check is the `ltd_contour` example test, matching every
archived denominator, complete U/F and a native DOT export/import round trip.
The finite fixture momentum-shift tables are source provenance, not a router
or general graph-isomorphism algorithm. No dependency patch is required for
this importer.

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

## Import acceptance evidence

The native exporter validated all four inputs and wrote run cards, native DOT
and compact validation manifests under `examples/contour/ltd/`. Input-only CLI
inspection admits every card. No multiloop sector generation was invoked.

| Record | Matched denominators | U terms | F terms | Positive / negative F coefficients |
| --- | ---: | ---: | ---: | ---: |
| `2L4P.b.K1` | 7 | 15 | 31 | 1 / 30 |
| `2L4P.b.K1*` | 7 | 15 | 63 | 33 / 30 |
| `2L6P.a.I` | 9 | 20 | 62 | 0 / 62 |
| `3L4P.K1` | 10 | 56 | 145 | 1 / 144 |

The maximum conservation/archived-shift reconciliation is `4.441e-16` for the
ladder cases and `3.469e-18` for the six-point case. All U/F coefficients have
native definite signs after scalar binding; no unresolved parameter is hidden
in the sign count. DOT export/import preserves each ordered denominator exactly.

The six-point input has exclusively negative F coefficients. It is therefore a
physical causal-branch and real-result control, not evidence of avoiding an
interior F zero. This does not replace the required ladder threshold controls.

Focused native tests pass the four imports and round trips, reject a deliberately
perturbed archived shift, and verify the normalization with an actually
generated/compiled repeated-propagator one-loop integral. Its exact coefficient
is `-1/2`, independently reproduced by the native OneLOop C0 API; applying the
paper measure gives `-i/(32*pi^2)`. This tests FastSecDec's coefficient, not merely
a manually supplied reference constant. The final four-test gate took 1.47s
after compilation, and strict core all-target Clippy passed before the final
parser regression was added. That regression confirms Symbolica's canonical
`ltd::{}::eps` spelling parses back to the exact declared `ltd::eps` symbol;
the measure contains no extra nonbuiltin free variable. Logs remain under
untracked `target/contour-ltd-*`; the combined public-dependency workspace gate
will include the final parser regression too.
