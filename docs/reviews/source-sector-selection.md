# Original-source selection and Horner defaults

Date: 2026-10-10. This review covers the native generation selection API,
TOML/serial integration, saved scope and result qualification. It introduces no
new algebra, graph representation, evaluator or sampling algorithm.

## User-facing contract

`GenerationOptions::source_sectors` and TOML `[generation] source_sectors =
[2, 5]` select original zero-based charts from the complete native geometry,
before symmetry grouping, endpoint reduction and subtraction. They do not name
post-subtraction numerical kernels. Direct generation, cooperative generation
and native staged/serial preparation use the same resolver. The geometry is
still constructed in full; only the selected charts proceed to mapping,
symmetry, coefficient construction and compilation.

An explicit selection must be nonempty, distinct and in range. The resolver
sorts it. Omission generates all charts; an explicit complete set canonicalizes
to omission, with identical native artifact bytes in the regression fixture.
The original input option remains part of CLI preparation/resume provenance.
Changing that input invalidates the existing generation journal.

Chart IDs used internally and in record receipts remain compact. Symmetry is
computed only among the retained charts; selected multiplicities and exact
terms are retained. A subset can cancel or fold entirely into an exact offset
without losing the declaration of which original charts were selected.

The scalar benchmark helper now inherits native `CompilationSettings` instead
of overriding Horner iterations to zero. Its default is ten, as are the native,
CLI and Python defaults. `--horner-iterations N` is an explicit helper override;
zero remains valid when intentionally requested. Malformed overrides fail
before an output directory is created. Historical cards and measurement
records that explicitly used zero retain that setting and interpretation.

## Native ownership and reuse

`SourceSectorSelection` records the original chart count and canonical selected
original IDs. `GenerationSourceScope` additionally maps the owning metadata's
ordered charts to original IDs. Native `GenerationMetadata`, record receipts,
indexed catalogue accessors and thin Python inspection expose this scope.
There is no inference from record-local chart zero to original chart zero.

The implementation filters existing native sector maps and reuses all existing
mapping, branch-aware symmetry, Symbolica differentiation, subtraction,
compilation and exact aggregation. Staging transports the native scope beside
its existing source/geometry identities. Receipt admission preserves compact
local IDs, ordered chart metadata and complete coverage of the selected set.
Recipe families must have the same original-source selection in every recipe;
mixing partial scopes, or a partial and complete recipe, is rejected.

Proper subsets use a native v14 scope wrapper around the existing v12 payload,
including its native atoms, exact requests and contour definitions. The old
metadata's binary field layout is unchanged. Omitted scope keeps the existing
format and identity. The subset's semantic identity includes the scope even if
two different subsets have equal evaluator arithmetic. Existing v13 optional
native JIT caches can wrap v14 without a second evaluator format or changed
program hashes.

Loading always checks scope shape and receipt/native metadata agreement. As
with other saved semantic identities, recomputing the cryptographic semantic
identity is controlled by `KernelLoadOptions::validate`; trusted loading does
not claim that additional verification. Tests distinguish a valid but
identity-altered scope from an invalid chart map.

The aggregate exact setup owner intentionally releases rich chart bodies. It
retains the declared selection with an empty chart map and serializes directly
as a native exact owner if necessary. It does not fabricate indexed coverage or
claim to contain the stochastic part of the partial integral. Repeated standalone
save/restore preserves its own semantic identity, bytes and exact vector.

## Scientific scope in results

Native result manifests carry the original selection. Requesting
`FullIntegral` from such a manifest canonicalizes to all available numerical
kernel IDs with exact contributions included, under `SelectedSectors` scope.
The ID set is sorted independently of output order, including public manifests
whose IDs are noncontiguous. Exact-only subsets use an empty numerical ID set;
an explicit `ExcludeAll` projection still omits their exact offset.

Native display, CLI inspection (including selected deep inspection), integration
reports and saved results identify a partial original integral. Integrating all
records in the partial artifact does not produce a full-original-integral
convergence claim. A later runtime numerical-sector restriction remains a
separate scope. No reference value or covariance is altered to compensate for
omitted original charts.

## Validation and independent review

The independent ecosystem/source review accepted the native reuse, identity
ordering, record-local chart handling, selected-only exact/symmetry scope,
recipe-family guard and result projection. It caught an unsorted result-ID
conversion, corrected with a regression using IDs `[7, 99, 42]`. A test-only
CLI Kuo rule initially requested eight points and correctly failed its native
minimum-size check; the corrected test uses 1024 points. Neither failure was
converted into a scientific success.

Completed focused gates:

| Gate | Outcome |
| --- | --- |
| Native source-selection public tests | 6 passed |
| Portable version of the same selection tests | 5 passed; native JIT-only control excluded |
| Result-scope projection tests | 3 passed |
| Saved-result tests | 11 passed |
| Native artifact tests, including v14 and existing codecs/caches | 55 passed, 3 existing ignores |
| Complete cached native core library, documented single test thread | 426 passed, 20 existing ignores |
| CLI normal/serial/recovery/subset/resume process tests | 3 passed |
| Maintained kite TOML/native input and Horner-default control | 1 passed |
| Scalar helper Horner default/explicit override control | 1 passed |
| Workspace all-target strict Clippy and formatting | passed |
| Python isolated `python_stubgen` check and formatting | passed |

The public selection controls compare complementary subsets with the full
complex coefficient vector, direct with cooperative generation, explicit all
with omission, and equal numerical expressions with different original IDs.
They cover nonzero and cancelled exact-only contributions, raw/indexed/family
restoration, selected resident owners, cache refresh, missing or relabelled
lineage, invalid requests and full-versus-partial result qualification. CLI
process controls exercise both endpoint modes and use fresh native worker
processes rather than an alternative staging implementation.

The complete core run passed in 34.19 seconds with the documented
`--test-threads=1` configuration. An initial eight-thread run had 425 passes,
20 ignores and one failure in
`contour::functions::dynamic::tests::prepared_root_callback_preserves_owner_lifetimes_and_clone_independence`:
concurrent tests retained a shared helper while the test expected its weak
reference to expire. Its failed log is preserved; the same binary passed all
tests serially without a production change. The binary preceded only two
semantically equivalent String-borrow lint cleanups in metadata error paths;
the final source passed strict workspace Clippy and formatting.

Raw logs remain ignored under `target/source-selection-*`; no benchmark output
or generated artifact is tracked. The bounded automated process regression
uses a smaller two-chart fixture to isolate scope, symmetry, resume and
exact-owner behavior.

## Maintained kite CLI smoke

A separate end-to-end debug smoke uses the maintained
`examples/contour/scalar_benchmarks/kite.toml`, copied into ignored output with
only absolute asset paths and `source_sectors = [2, 5]` added. Serial generation
with one worker and Horner ten retained two of 22 original charts, their common
representative, and one four-dimensional numerical kernel. Its complete native
generation time was 1.735 seconds; the external monitored process group closed
successfully in 2.429 seconds with 84.20 MB sampled aggregate RSS. These are
debug smoke observations, not optimized performance or speedup measurements.

Lightweight text/JSON inspection and validated deep selected inspection all
retain original `[2, 5]`/22 scope. A native Kuo33002/Korobov3 integration at
`S=.8, L=1, R=.25`, Pilot16 and 1024 points times two shifts completed all 2048
points. The saved native result retains the full complex covariance, exact
offset policy `IncludeAll`, `SelectedSectors([0])` and the original source
selection; the report's full-integral `converged` flag is false. No comparison
with the complete kite reference is made. All five monitored actions exited
successfully with empty process groups and no limit hit. Some very short
inspection/integration processes ended before RSS polling observed them; their
zero sampled RSS is not a zero-memory claim. The pinned inputs, executions and
acceptance checks are retained in ignored
`target/source-selection-kite-smoke/review.json`.
