# Contour recipe ownership and generation recovery

2026-10-09. Review of the native generation selector, helper ownership and
recipe-aware staging increment. Dynamic production admission remains closed;
the saved certified checker and complete runtime integration are separate gates.

## Native ownership and ecosystem reuse

`GenerationOptions.program_recipe` explicitly selects the mathematical program.
The existing CLI/TOML/Python contour flag continues to select the fixed recipe.
Native polynomial-dynamic generation constructs the full smooth map before
Taylor/IBP subtraction, using the existing Symbolica derivative hooks and dual
machinery. Sign-aware generation still fails explicitly until its stable
coefficient callbacks are ready.

Generated charts, sectors, detached compilation jobs and their completions
retain strong native helper owners. Disk-backed chart records restore these
owners before importing the associated Atom context. Selecting or remapping
charts keeps their recipe identity and associated checking source. Exact
contributions retain their referenced helpers independently of stochastic
sector residency. A weak callback registry is not treated as an owner.

The independent helper audit also separates versioned mathematical callback tags
from checksums of saved optimized programs. Numeric callback construction resolves
only the selected descriptor's exact owners. A detached sector retains the owners
needed for later precision mapping; sampling performs no global registry lookup.
Descriptorless legacy loading explicitly clears any surrounding preparation
scope. Nested scopes, unwinding, independent caller threads and legacy restoration
are covered by the [helper identity review](contour-helper-identity.md).

The implementation uses Symbolica's existing context-aware Atom and evaluator
codecs. The checking source is a generation-time collection of native
expressions and an explicit output schema; it is not a second evaluator or
CAS. Production checking will consume optimized saved programs. The new
coordinator identities contain no algebra or graph data.

Current pre-compilation admission checks chart coverage, coordinate identity,
dimensions, factor counts and the complete output schema. Binding the saved
checker and positivity proofs to the actual retained F/U expressions is still
required before dynamic production admission; matching structural counts alone
does not establish that association.

The canonical physical source identity is separate from recipe-specific
execution records. Symmetry admission remains exact and recipe-local. Hashes
continue to locate candidate records, rather than establish equivalence.

The initial staging-byte hash was corrected during review: Symbolica transport
state includes process-local symbol and polynomial-ring tables. The public
`generation::source_identity` helper now streams a structured schema using
Symbolica's existing `to_canonical_string`, as the native kernel identities
already do. It preserves the ordered physical input, roles, branch semantics
and mass constraints without decomposing or expanding it. Only exact reserved
contour input names are excluded; a physics symbol sharing their namespace
remains part of the identity. Fresh workers deliberately register unrelated
symbols and coefficient rings to test this distinction from transport hashes.

## Recovery review and fixes

Native staging and the CLI journal use version two. Resuming an incompatible
version is rejected before rewriting its journal or receipts, with an explicit
instruction to regenerate under a new output basename. Published integral
artifacts use their existing readers; this change concerns incomplete
generation staging.

The independent review identified additional receipt-admission requirements:

- Discovery results must match the issued source, recipe, dimension and index.
  These compact identities travel in the request; the coordinator does not
  reread the complete preparation catalogue for every completion. The worker
  independently checks them against its prepared native context.
- Symmetry assignments must match the source and recipe. Their permutation
  needs the correct length as well as the complete set of coordinates; set
  equality alone admits repeated entries.
- Formula results must match the source, recipe and formula key.
- Compiled results must match the issued source, recipe, sector and output
  path. Every inner native receipt must report the same recipe. Source-chart
  coverage across receipts must match the issued chart set without duplicates,
  and the reported generation modes must cover that same set.

These checks were implemented and independently reviewed. They operate on
compact receipts and existing streamed file digests, without native expression
decoding or JIT in the coordinator. The recovery regression exercises the
durable-data-before-receipt crash window, then injects same-index foreign
sources/recipes, repeated permutations, incorrect source coverage and forged
outer receipts.

## Acceptance scope

Focused descriptor tests cover ownership, exact-helper aggregation, selected
chart projection, zero-dimensional recipes and corrupt staging descriptors.
Generation tests compare analytic complex Laurent coefficients for symbolic
and numerical-dual Taylor/IBP, and exercise native callback restoration in a
fresh worker. The milestone test results are recorded in
[the phase progress ledger](contour-phase-b-progress.md).

This increment does not claim a complete dynamic artifact family, certified
dynamic runtime checks, measured dynamic-versus-fixed variance improvements or
the required physical multiloop acceptance tests.
