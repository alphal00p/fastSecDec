# Persisted generation observations

The human artifact JSON now has an optional typed `generation` record. It stores
the configured worker count and `requested_coefficient_expansion`. The latter
is deliberately a requested route: the coefficient-series route can use its
supported full-expression fallback for individual sectors. Human presentation
uses "Coefficient series" or "Full expression" and labels the request.

Existing kernel metadata already retains sector count, Laurent orders and
components, runtime parameter names and evaluator statistics/backends. Existing
artifact fields retain content identity and generation wall timings. Those
facts are reused rather than duplicated into another summary schema. The output
path is derived from the basename being viewed, so moving the artifact pair
does not preserve a stale directory or introduce absolute paths.

`generate` records its observations before saving the pair and reads them back
from the same `Artifact` when rendering the final report. Older artifacts that
lack the record load with `None`; missing worker counts display "Not recorded"
and serialize as null in the final report, without inferring a count from the
current host. No binary codec version or evaluator representation changes.
The optional generation record is excluded from the artifact identity, as are
the existing observational timings and reference settings.

The generation report exposes its existing phase-row formatter for inspection.
Both views therefore share phase names, duration rounding, omission of
unmeasured zero subphases and the generation total, rather than maintaining
independent timing interpretations. The total retains its existing boundary
before writing the artifact files.

Final generation Laurent rows also use the shared native Symbolica printer with
the artifact's actual regulator, rather than a hardcoded epsilon label. Separate
order, basis and component columns distinguish order zero from its constant
basis one. The temporary native-math renderer probe passed at 120, 80, 64 and
40 columns, plus the stacked 31- and 20-column modes, including noncontiguous
orders, real/imaginary components, Unicode paths, exact-only and empty layouts.
Native ANSI coloring stripped back to exactly the plain layout; an empty
`NO_COLOR` value suppressed all styling. Source and captures remain under ignored
`output/artifact-inspect/generation_math_probe.rs` and `generation-math-*`.

## Focused evidence

The temporary CLI probe included the production artifact persistence and final
generation renderer. It used `output/artifact-inspect/before.fsd`, a real
30-sector ggHH artifact produced before this record existed. Its log is
`output/artifact-inspect/record-probe.txt`; the source is retained only in the
ignored `output/artifact-inspect/generation_record_probe.rs`.

- The older artifact loaded with no invented generation record.
- Saving a worker/method record and loading the pair again retained those facts,
  timings, artifact identity, kernel identity and exact binary bytes.
- Changing workers from seven to eight and the requested route from full
  expression to coefficient series left both identities and `.dat` bytes
  unchanged. A future unknown observation field was tolerated on load.
- Absence of both the generation record and timing record remained explicitly
  unavailable after deserialization.
- The production final JSON/human report read the saved worker count, requested
  method, timings, output layout and identity. Its machine values matched the
  human metadata JSON; redirected human output contained no terminal escapes.

The new record does not alter the existing kernel `RawValue` identity contract:
unknown-field checks preserved the kernel JSON verbatim. Reformatting an
identity-bearing kernel object is a separate existing transport constraint.

## Native evaluator size attribution

No additional core API was needed. `SectorKernel::statistics()` already returns
facts captured from the actual compiled shared Laurent-vector program.
`exact_program_bytes` is the length of the established native evaluator
serde/bincode representation. It excludes the artifact envelope, Symbolica state
and chart metadata, mutable evaluation stacks and executable machine code.
`operations` comes from Symbolica's `count_operations()` before real/complex
lowering and SymJIT optimization. `symjit_ir_bytes` measures the compressed
SymJIT application, including the existing complex helper sum, rather than
machine-code size. Source reuse was checked at the compilation, program codec
and statistics owners; the independent native inspection probe is recorded by
the inspection reviewer. No alternate evaluator traversal or size estimator was
added.

The existing tests/gates and other examples remain unchanged. End-to-end release
generation and the independent inspection presentation review are separate
acceptance records.

## Independent selected-chart view review

The separately authored `PortableMetadata::charts_for_sector` helper was
reviewed independently. It filters retained native charts by their saved kernel
index and invokes the existing `PortableChart::from_native` conversion. Unlike
the private hash-boundary `for_sector` view, it preserves the original kernel
index as well as source-chart, representative and permutation identifiers. It
does not construct a partial native generation object, add a decoder, change
the transport schema or include unrelated global factor metadata.

The independent public-API probe compared all 30 selected views of the existing
ggHH artifact to its full native JSON presentation filtered by kernel index.
Every selected array matched exactly, including nonzero IDs. The kernel
template identity and retained binary bytes remained unchanged. An unavailable
index gives an empty presentation array; the CLI separately validates a user's
sector selection. Evidence and temporary source remain only in ignored
`output/artifact-inspect/portable-charts-review.txt` and
`portable_charts_review_probe.rs`.

Native math presentation was also checked at the source boundary. Per-term
monomials use the actual retained target coordinates and endpoint exponents;
no minimum-power inference is made across terms. The chart namespace context
includes all maps, coordinate names, measure factors, regulators, powers and
prefactors; representative permutations combine both chart contexts. This
preserves distinct symbols when shortening their native names would collide.
Review found that the initial input-factor table computed this context per row.
The author changed it to inventory every retained polynomial and exponent in
the table before printing any row, so different symbols in separate factor
rows cannot collapse to one visible short name.
