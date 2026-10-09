# Independent CLI recipe integration review

2026-10-09. Generation-owner review of the CLI artifact/selection slice, separate
from its implementation owner and the native archive review. This reviews the
registered interfaces, not an assertion that dynamic production is complete.

## Boundaries and identities

The CLI `ProgramStorage` owns the manifest's selected/default recipe and native
directory. `CatalogueView` borrows logical layout data only: it does not invent
v1 record ranges for a v2 archive. Native `ProgramArchiveReader` owns structural
validation, recipe lookup and payload restoration. `open_program_archive` checks
both the archive ID and the complete directory against the manifest, including
when optional payload digest checking is disabled.

There are intentionally distinct identities: CLI scientific provenance, the
complete immutable archive, the selected native recipe catalogue, and the bound
mathematical kernel. Serial jobs carry archive and selected catalogue IDs plus
the recipe. A resident worker checks all three before loading, and refuses a
change of sector, recipe, data file or bound settings without process eviction.
Runtime contour selection occurs before ordinary restoration, isolated exact
setup and serial worker loading. Unknown or absent recipes fail explicitly.

The source ID supplied to a native multi-recipe writer must identify one
mathematical input across all recipes. The CLI consumes that native contract;
it must not synthesize source identity from selected record ranges. Production
recipe-set generation remains a separate pending integration gate.

## Ownership and reuse

Serial workers restore one selected sector and retain its evaluation context;
the temporary reader and KernelSet are dropped. Isolated exact setup selects the
recipe first, validates required exact chart records one at a time, then binds
the compact exact aggregate with checks disabled and reports the preceding
pilot evidence separately. The setup child is reaped before sampling starts.
No CAS, evaluator codec, graph representation, RNG or thread/process ownership
has been moved into the CLI's native library dependencies.

Publication reuses the existing immutable data file plus atomic manifest
transaction. A failed staged replacement leaves the old manifest usable. The
native directory owns global physical range validation; a selected reader may
avoid digest-reading an unrelated recipe's payload without narrowing those
structural checks.

## Inspection and findings

Metadata-only inspection reads no sector payload and explicitly says that
producer observations have not been checked against the binary. Recipe layouts
come from the selected directory. Historical one-recipe expression previews are
suppressed for a multi-recipe archive, rather than misattributed. Selected deep
inspection loads one record and maps local chart/source IDs through its native
receipt before presentation. It reports the limited validation scope.

One presentation gap was sent to the CLI owner and fixed: deep all-sector JSON
initially omitted the selected recipe and native catalogue identity even though
it loaded the correct recipe. Deep, selected and lightweight JSON now expose
`selected_recipe`, `available_recipes` and `selected_catalogue_content_id`;
the terminal overview also shows archive/recipe IDs. Source reinspection
confirmed the correction and a real CLI regression asserts their consistency.
This was a presentation issue, not wrong program selection.

No blocking storage, bounded-residency or ecosystem-reuse duplication was found
in this read-only slice. The native v10 helper descriptor, dynamic validation,
production recipe generation, and dynamic checkpoint/scientific controls are
outside this review and retain their own acceptance gates.

## Verification evidence

The CLI owner reported its artifact unit gate passing 17/17 before this review.
Source inspection confirmed dedicated tests for offline metadata inspection,
selection before loading, corruption in an unselected payload, and interrupted
replacement. Those tests do not establish dynamic contour correctness. After
the presentation fix and native v10 registration, the owner will rerun focused
CLI gates before accepting the combined milestone.

## Final source reinspection before the public-owner milestone

2026-10-09. The final registered CLI archive/selection paths and their tests
were reviewed again without changing Rust source or starting Cargo. The
current workspace lock selects Symbolica/Numerica
`7ec1be45ef92ae3b154e0d4ce754c0bdf3d9d0ca`, SymJIT
`33100ae869057f35d9865c933a48bac6699acdd4`, and Feynkit/Linnet/Spenso
`8e3a643f388b45939d6573a648ef3a509086835e`. Graphica remains registry 3.0.1 in
this workspace; the separate installed-host/owner-consumer matrices must not
be substituted for these identities.

The source still validates the complete native directory against the CLI
manifest before loading the selected program, even when optional payload
digest checks are disabled. The resident worker retains its selected context
only, fences archive/recipe/catalogue/bound-settings changes, and cannot switch
sector without eviction. Exact-offset setup selects the same recipe before
loading, checks required exact records independently, drops them, and reaps the
setup process before production starts. Pilot provenance is reported separately
from the compact exact aggregate's unchecked binding.

The current real-CLI regression bundles actual native fixed and undeformed
records, then exercises both ordinary and serial integration, full complex
vector covariance, metadata-only and selected/all deep inspection, result
provenance and rejection of cross-recipe checkpoint resume. The separate
archive unit controls cover missing/offline binary data, corruption in an
unselected payload and interrupted manifest replacement. Optional policy-only
checkpoint changes remain separate from mathematical prescription identity.

The focused public-owner recipe CLI log records **one passing integration
test** covering that matrix (`target/contour-public-cli-programs.log`). The
coordinator's complete workspace gate on the final Feynkit revision was still
running at this reinspection; this review does not promote the earlier
focused result into a completed final-matrix gate. Fixed analytic subtraction,
native OneLOop B0/C0/D0 and full physical ggHH evidence remain recorded under
their actual tested source identities in the scientific reviews.

No new commit blocker was found. The transport fixture is deliberately not a
production program-set generator. Dynamic native decoding remains explicitly
gated, and none of these passing storage/CLI controls establishes dynamic
causal validation, dynamic scientific agreement or a variance improvement.

## Shared family and universal publication reinspection

The post-55c3d7c orchestration was reviewed independently against the native
shared-preparation APIs. `RecipeFamily` canonicalizes the requested set and
requires its default to belong to the set. Journal schema 3 persists that entire
family before the completed-resume shortcut. Reordering requests is allowed;
changing the family/default is rejected. Source extraction jobs are shared;
discovery, exact symmetry, formula and sector jobs use recipe-prefixed identities
and retain native execution-source fencing.

The coordinator keeps compact source/sector receipts and copies completed native
records into one `ProgramArchiveWriter`. It does not collect completed evaluators
or reconstruct expressions. Ordinary singleton generation now also uses this
writer through `KernelSet::retain_program_archive`, preserving the resident
evaluators and adopting the selected catalogue identity without recompilation.
Both paths use the same canonical physical source identity, separate from saved
program identities, and the same completed-data/atomic-manifest publication.

Selection precedes numerical loading. Metadata summaries derive their layout
from the selected native recipe directory. Optional bounded inspection previews
are keyed by recipe; an unavailable or unknown-version preview is omitted rather
than substituted from the default recipe. The serial path may lack such a human
preview while retaining its complete native chart records for selective deep
inspection. The complete native directory is compared against the manifest;
selected deep inspection loads one record and maps chart IDs through its receipt.

The new real child-process family controls reported two passing tests, covering
both generation modes, complete complex Laurent vectors, ordinary versus
selective record restoration, canonical family resume and interrupted replacement.
The CLI owner subsequently reported 100 unit tests passing with five ignored,
and 59 integration tests passing with three ignored across 23 executables. That
matrix includes all normal/serial generation/integration combinations, contour
validation policies, exact offsets and killed-coordinator recovery using v2.
The strengthened native-Dualizer scientific filter subsequently passed all five
tests, including complete cubic and higher-endpoint Laurent vectors; its exact
scope is recorded in `contour-sign-aware-generation.md`. The full core library
also passed 297 tests with 16 ignored; Clippy and portable gates remain separate.
No implementation or ecosystem-reuse
blocker was found in this read-only inspection; this is not a dynamic runtime
or performance acceptance claim.
