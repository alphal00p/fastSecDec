# Artifact inspection: independent native integration review

Date: 2026-10-07. This review is independent of the CLI report implementation
and the saved generation-observation implementation. Other examples and the
committed test/gate migration remain deferred.

## Existing owners and presentation reference

The sibling FastSecDecPathFinder reference at revision
`582d8c7f6dde9bf750750d4c2a2d85a94ce940cd` was inspected read-only, especially
`src/formatting.py::summary_data` and `print_preintegration_summary`. Its sector
table presents IDs, variables, coordinate maps, regular Jacobians, named U/F
monomials, singular axes and endpoint powers, with colored headings and a
bounded sector preview. Its layout is useful context, but its physical
attributions cannot be copied into FastSecDec records that do not retain them.
No reference parser, graph representation or expression formatter was reused.

The pinned Symbolica printer reexports Numerica's public `PrintOptions`.
`Atom::printer` already provides native arithmetic formatting, integer
superscripts, multiplication characters, width controls, namespace visibility
and explicit `ColorMode::Always`/`Never`. Source inspection and a focused native
Rust probe confirmed these APIs before implementation. The probe printed a
retained affine exponent and integer superscript, distinguished `one::x` from
`two::x`, verified explicit ANSI enable/disable, and checked that canonical Atom
content was unchanged. Hiding `fastsecdec` hides that exact namespace; nested
sector namespaces remain explicit. Compact printing can instead collect native
`get_all_symbols(true)` across the entire displayed chart context and compare
`Symbol::get_stripped_name()` before enabling namespace suppression. The context
must include both equation sides; inspecting each side separately would miss
`a::t0 = b::t0`. No alternate algebra or printer is needed.

## Scientific interpretation of retained records

- `KernelResultManifest::from_kernels` assigns numerical sector IDs from the
  zero-based kernel slice. Sorting a preview must preserve those IDs. A chart's
  `source_index` and `representative` are chart IDs; `kernel_sector` associates
  it with a numerical kernel. They are not interchangeable in general.
- Coordinate equations come directly from `CoordinateMap::images`. Projective
  maps fix one source coordinate to one; they are gauge-fixed coordinates,
  not normalized simplex coordinates. The retained Jacobian is a positive real
  density measure, not an oriented or complex determinant.
- A representative permutation maps this chart's target coordinate at index
  `i` to the representative target at `permutation[i]`.
- Each retained pre-subtraction term has its own exact affine endpoint powers.
  Their native product is that term's retained monomial factor. Combining the
  minimum power from different terms would invent a different expression.
  Regular bodies are not retained, only their native Atom storage sizes; they
  may vanish further. The monomial alone is not a full leading-term expansion.
- Pre-subtraction records precede symmetry multiplicity, endpoint subtraction
  and Laurent expansion. Taylor requirements are not surviving pole counts.
  Support valuations follow deduplicated support order and cannot be labeled
  U/F without an explicit retained association.
- Charts with no numerical kernel may be exact, cancelled or truncated at the
  requested orders; the record does not distinguish them or retain individual
  chart exact coefficients. Unbound parameter-dependent exact offsets remain
  unavailable, rather than being presented as zero.

## Evaluator sizes and actual ggHH native probe

`EvaluatorStatistics::exact_program_bytes` is the length of the existing encoded
exact evaluator program for the entire shared Laurent vector. Operations are
Symbolica's exact evaluator operation counts before real/complex lowering and
SymJIT optimization. `symjit_ir_bytes` measures the compressed serialized
SymJIT application, not executable machine code, runtime memory, artifact-file
size or one Laurent coefficient. These existing native observations suffice;
no polynomial expansion or evaluator reconstruction for counting is needed.

The native probe cold-loaded the saved D05 artifact copied to ignored
`output/artifact-inspect/before.fsd.dat`. Its kernel identity was
`66f2af45b499f63bb15feb7148d0941eabb654ec2e83227443c023756208f09d`.
It contained 30 numerical kernels and 2,780,572 total exact-program bytes.
Descending size order, ties broken by original kernel ID, gave top ten IDs
`5, 1, 7, 3, 21, 23, 25, 27, 6, 18`. Every kernel happened to have one retained
chart in this artifact; the general report must not assume this bijection.

The probe independently checked every chart's source/image cardinality,
projective fixed-coordinate image, target/power cardinality and exact equality
of each retained exponent to `b + c epsilon`. It printed native equations and
per-term monomials for kernels 5 and 0 as independent report references.
Temporary source and logs remain under ignored `output/artifact-inspect`.

## Renderer and persistence acceptance

The final source review covers CLI `inspect.rs`, its overview/presentation/table
modules, the shared `math_display` helper and saved generation observations.
There is no numerical generation or integration in inspection. Ordinary native
artifact loading still initializes its evaluator backend; loading time is shown
separately from the historical generation timings.

The shared formatter uses one native symbol context for both coordinate-equation
sides, all monomial groups, term prefactors, representative permutations and
input factors. A separate executable probe of the actual helper verified compact
`t0^(-2+3·eps)`, disambiguated `source::t0 = target::t0`, and preserved distinct
function symbols across separate rows. Native printing owns the mathematics;
tabled owns ANSI-aware wrapping. No manual symbol/text substitution was added.

A minor review finding was fixed: two-column tables initially capped their first
column at twelve characters even on wide terminals, splitting “Parametrization”
and “Positive measure” unnecessarily. The corrected adaptive budget preserves
these labels at ordinary widths and keeps narrow displays within their bounds.

Independent pseudo-terminal runs of the final debug implementation passed:

- Overview at 120, 80 and 40 columns and sector 5 detail at 80 and 40 columns
  stayed within the declared terminal width. Colored terminal reports contained
  ANSI styles; `--plain` and `NO_COLOR` reports contained none. Redirected JSON
  contained no ANSI sequences.
- The overview retained the expected top-ten original kernel IDs and their
  serialized sizes. Sector 5 showed 93,605 exact-program bytes, 2,702 additions,
  9,070 multiplications, twelve inversions and five function calls, matching the
  independent native evaluator statistics.
- Sector 5 equations matched the retained native map: `x0=t5`, `x1=1`,
  `x2=t3·t4`, `x3=t2·t4`, `x4=t4`, `x5=t1`, `x6=t0·t4`, with positive measure
  `t4³`. Its two term groups were `t4^eps` and `t4^(1+eps)`, explicitly attributed
  to representative chart 5 and qualified as pre-subtraction factors.
- Selected JSON preserved the full original native chart record and kernel ID
  five, with no global factor/chart dump. The captured selected document was
  approximately 5.5 KB, compared with approximately 2.86 MB for full inspection.
- Noninteger, negative, overflowing and out-of-range selectors failed clearly.
  `--sector 5` with a nonexistent TOML card returned the artifact-required error
  before attempting input-file loading.
- An exact-only temporary artifact displayed zero numerical kernels and an empty
  ranking while preserving its exact coefficient one in JSON. Selecting kernel
  zero failed without an invalid index range. A temporary wrapper around the
  native historical triangle fixture displayed missing retained metadata
  explicitly and returned `charts: null` for its selected numerical sector.

Saved worker count and requested expansion method are optional typed
observations, written before artifact publication and excluded from scientific
identity. Source inspection confirmed that generation reports and inspection
read the same saved record. Independent cold-load checks verified that old
artifacts report those values as unrecorded, while adding an observation record
changes neither artifact/sector identity nor binary data. “Requested” is accurate
because the series route can use an exact physical fallback for a sector. Saved
phase timings remain historical observations; the current inspection load time
is separate. The generation total explicitly excludes writing the artifact pair.

## Selected-chart helper authorship boundary

This reviewer implemented the narrow public
`PortableMetadata::charts_for_sector` helper requested during review. It selects
existing `PortableChart::from_native` records without renumbering or serializing
unrelated global factors. The hash-oriented `for_sector` helper intentionally
renumbers the kernel to zero and is therefore unsuitable for presentation.
No native graph, codec schema or mathematical identity changed.

The implementation probe compared selected JSON with full native metadata
filtered by kernel ID and preserved source, representative and kernel IDs five.
An independent reviewer then checked this equality for all thirty ggHH kernels,
plus unchanged binary bytes and identities and the empty unmatched-index result.
The CLI owns rejection of an invalid numerical selector. This independently
reviewed helper is distinguished from this author's independent assessment of
the report renderer.

No production test files or other examples were changed. Temporary probes,
terminal captures and synthetic fixtures are confined to ignored
`output/artifact-inspect`. Actual browser rendering, a new numerical reference
calculation and performance improvements are not claimed by this presentation
milestone.
