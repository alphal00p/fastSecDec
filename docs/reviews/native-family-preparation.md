# Native family preparation before parameterization

This is the design and in-progress implementation record for a production seam,
not an implemented graph rewrite. The
off-shell scalar and rank-two probes already prove the exact original-to-active
family identity and complete numerical coverage. The bounded projected on-shell
experiment subsequently timed out at 310.289 seconds in native Laurent work;
projection alone does not resolve that fixture's generation cost. The preparation
seam still removes the independently demonstrated redundant parameterization and
its off-shell costs; Laurent-depth orchestration is a separate generation slice.

## Existing capability and evidence

Native `feynkit_graph::IntegralFamily` already owns the relevant operations:

- `is_independent()` reads the family's stored exact affine rank and cheaply
  identifies inputs for which no denominator elimination is available.
- `partial_fraction(powers, max_states)` returns exact coefficients and signed
  powers in original denominator order. It performs no loop shift, no scaleless
  removal and no new integration prescription. The budget counts processed and
  queued exponent states; it is not a wall-time bound on a single symbolic solve.
- `sector(powers)` retains the positive-power denominators and reconstructs the
  native family with the original loop momenta, external momenta and kinematics.
- `ParametricIntegrand::from_family` is now the public thin FastSecDec entry into
  the existing native Gaussian construction and strictly positive-power measure.

The source is `feynkit-graph/src/integrals.rs` and
`integrals/partial_fraction.rs` in the pinned local FeynKit checkout. Its native
tests check repeated denominators, mass shifts and exact reconstructed rational
identities. FastSecDec's `tests/family_parametric.rs` adds the raised-power
duplicate-tadpole identity with analytic value two, original-simplex measure
integration, graph weight 210 applied exactly once, and public collision/power/
dimension admission checks. The executed triple-box proof independently checks
the original inverse-denominator product against native `partial_fraction` and
`sector` without expanding the full density. Both off-shell projected inputs
retain all three loops, three independent external momenta and the exact Gram
matrix, while reducing ten Schwinger variables to eight.

Linnet/FeynKit provide graph identification and compatible momentum-routing
operations, but source inspection found no public degree-two contraction that
also transports integral powers, model factors and a fixed tensor numerator into
an equivalent `GraphIntegral`. Constructing such a transport solely for this
optimization would duplicate work already owned by the family API. A manually
collapsed DOT can be a useful independent fixture; it is not needed as the
production entry convention.

## Proposed ownership and public contract

Keep the original native `GraphIntegral` as the physical input and provenance.
Add a small library-owned family-preparation result used by graph
parameterization and by the CLI, with conceptually these fields/accessors:

```
PreparedFamily<'a> {
    family: borrowed original IntegralFamily or owned native sector,
    powers: positive powers in the retained family's native order,
    active_original_indices: ordered indices into original denominators,
    status: Unchanged(reason) | Projected { original_count, active_count },
}
```

The exact Rust representation can use `Cow` or a borrowed/owned enum; it must not
invent another physical family type. The preparer owns only admission,
configuration and metadata, while native `partial_fraction`/`sector` own every
algebraic change. Expose typed settings/status through the library so a future
HEPKit bridge can use the same operation without serializing a graph to DOT or
copying CLI logic. Do not put the algorithm in `fastsecdec-cli`.

The weighted numerator is contracted from the original graph and multiplied by
its graph/projector/measure factor once. It remains in the same scalar-product
basis; preparation admits no momentum shift or basis change. Passing this value
to `from_family` adds only the already documented Gaussian and normalized-loop
measure factors. Raised denominator powers are handled by the existing native
Feynman-parameter measure; no extra factorial, Jacobian or model weight belongs
in the preparer.

For existing graph APIs with a caller-supplied parameter for every original
denominator, validate that original parameter contract first and retain only the
active original labels. Never reinterpret an original label as another edge's
parameter. A CLI that creates its own fresh labels can prepare first and allocate
exactly the active count. Metadata should record the original ordered stable
edge IDs/powers, active original indices and new positive powers, with original
and active denominator counts named distinctly. Generated parameter/map metadata
then describes the actual active family. Kernel identity continues to bind the
actual generated coefficient expressions and maps; input/source identity retains
the original native graph and the preparation policy. Full-integral scope refers
to the same physical integral, not a selected subset of original graph edges.

## Conservative admission and fallbacks

1. Use native `is_independent()` as the first bypass. A family with independent
   denominators incurs no partial-fraction solve.
2. Convert the validated positive input powers to native signed powers using
   checked conversion; retain the old path if this optional optimization cannot
   represent an otherwise admitted power.
3. Request bounded native `partial_fraction`. Admit only exactly one term with
   coefficient canonically one, no negative powers, fewer positive-power entries
   and at least one retained denominator.
4. Call native `sector`, verify unchanged ordered loop/external bases, and retain
   the exact original-index mapping. Native source guarantees kinematic copying.
   The already executed canonical inverse-product equality can be kept as a
   conservative verification using existing Atom operations; failure simply
   declines optimization without inventing a new simplifier.
5. Preserve the original parameterization for multi-term decompositions, nonunit
   coefficients, negative-power terms, no reduction, unsupported checked-power
   conversions and native state-budget exhaustion. Record the typed reason.
   Never silently accept a partial decomposition or a subset of its terms.
6. Do not hide invalid source-state errors, a positive-power identically zero
   denominator or a failed original-input contract as an optimization fallback.
   Distinguish an optional optimization limit from an invalid integral.

This intentionally narrower first admission rule needs no new prefactor
composition or general decomposition-sum orchestration. It can preserve ordinary
graphs unchanged while making the proven repeated-denominator cases natural
inputs to the existing CLI. The state cap and outcome should be observable;
single native symbolic calls remain subject to the existing generation watchdog
rather than a false guarantee of instantaneous cancellation.

## Required implementation checks

Reuse the existing exact raised-power and graph-weight tests through the new
preparation entry. Add an independent-family no-op, a bounded/multi-term fallback
that preserves the complete density, and a numerator case whose active labels
are a noncontiguous subset of original labels. Check that original/active
metadata, powers and full-integral scope survive portable save/load. Compare the
prepared graph route to the already executed direct native-family projection on
the same complete Laurent vector; original unprojected statistical estimates
remain diagnostics, not bitwise-equality targets across changed coordinates.

Before default admission, independently review this seam and measure its no-op
cost on representative ordinary graphs. Do not implement an alternate partial-
fraction algorithm or a graph contraction to improve that measurement. A future
general multi-term family API is a separate extension, not a reason to expand
this first optimization's scope.

## Initial library implementation

The source now exposes `FamilyPreparationPolicy`, a typed fallback/status/report,
`PreparedFamily` and `prepare_family` from `parametric`. The native family is
borrowed for an unchanged result and owned only after a native sector projection.
`ParametricIntegrand::from_graph_prepared` and `from_family_prepared` return the
integrand and report; existing `from_graph` and `from_family` retain their
original-family behavior. No CLI default or portable schema has changed.

Original label admission calls the existing native Symanzik entry because its
standalone label validator is private. A no-op reuses that original U/F, so it
does not compute the same pair twice. A projected input additionally constructs
the active U/F; this bounded extra preparation cost is explicit, rather than
duplicating native label logic or weakening validation of discarded parameters.
Native state exhaustion and optional signed-integer power overflow preserve the
original route; invalid source-family errors still propagate.

Six focused tests are implemented in `tests/family_preparation.rs`: exact original
simplex versus projected raised-power moment, unchanged/multiterm/budget controls,
noncontiguous original labels with loop numerator, collisions in discarded
labels, native integer-range fallback, and a weighted native graph with analytic
value 70. The shared compile passed, followed by all six preparation tests, all
four existing family-parametric tests and all fourteen native-input tests. The
optional-integer-limit test also verifies that a positive-power identically zero
denominator is rejected under both policies, rather than hidden by fallback.
Logs are `output/series-first-compile.log`,
`output/family-preparation-focused-tests.log`,
`output/family-parametric-regression-tests.log` and
`output/family-native-input-regression-tests.log`.

The independent HEPKit review found no remaining source-level blocker; its
executed-evidence closure is recorded separately. No-op cost measurement remains
a prerequisite to later default CLI adoption, not a claim established by these
correctness tests. Portable preparation metadata and an automatic graph route
remain separate implementation work.

## Minimal CLI and portable-record follow-up proposal

This section is a proposal only; no CLI or portable schema change has been made.
The initial wiring can keep a missing run-card setting equivalent to the existing
original-family route while its checks run. Use a small graph-only setting under
`generation`, for
example `family_preparation = "single-unit-term"` and
`family_preparation_max_states = 32`; `"original"` selects the existing route.
The CLI adapter only translates spelling into `FamilyPreparationPolicy` and
delegates validation/preparation to the library. Reject an explicit preparation
request on a direct U/F card, rather than silently treating it as a graph.
Default adoption remains a separately reviewed decision after no-op timing.
The intended natural production design may then make `SingleUnitTerm { max_states:
32 }` the default for newly generated graph inputs: opt-in is not a permanent
compatibility requirement. Keep the explicit `Original` route as a scientific
control. Historical artifact metadata remaining absent/valid is a separate
requirement from the default applied when generating a new artifact; changing
the latter must never reinterpret an already stored artifact or checkpoint.

`input::load` already owns the native `GraphIntegral`, its ordered
`propagator_edges()` and original powers. It should continue allocating the full
original label vector, then call `from_graph_prepared` only for an explicit
preparation request. Keep `LoadedInput.propagators` as the original graph count;
show active count separately. Never use active denominator positions as new
native `EdgeId`s, and never alter graph edges or the user's per-edge powers.
The existing graph weight/measure application in the prepared entry remains the
single owner; the loader adds no prefactor.

Use one native preparation record carried with `ParametricIntegrand`, then
through `GenerationMetadata`, rather than a second CLI-only physical schema:

```
FamilyPreparationMetadata {
    report: FamilyPreparationReport,
    original_parameters: Vec<Symbol>,
    original_edges: Option<Vec<EdgeId>>,
}
```

The report already owns requested policy, fallback/projected status, original
powers, active original indices and active powers. Native `EdgeId` already
implements transparent serde and remains the graph identifier type; a direct
family caller has `original_edges: None`, not invented graph IDs. Prepared
constructors can attach this record without changing their existing tuple
return. Original constructors keep metadata absent. The original graph stays
available to the caller, while input source hashes bind its serialized source in
the CLI envelope. The record does not serialize a replacement graph or copy
coordinate-map definitions.

Add an optional `family_preparation` field to the existing native
`GenerationMetadata` and existing `PortableMetadata`. Use native canonical Symbol
transport already used for the domain/chart parameter lists; do not add another
Atom serializer. `None` must default on read and be omitted on write. This keeps
old version-two payload bytes and content IDs unchanged; `Some` binds the actual
policy and original-to-active correspondence into the existing kernel content
hash. Version-one metadata absence stays explicit. The CLI's outer artifact
already hashes the whole kernel payload and run-card source fingerprint, so a
new second copy of preparation data in `Provenance` is unnecessary. A policy
change must require regeneration before checkpoint resume, even when it falls
back to an algebraically identical original density.

An absent retained preparation record means **not recorded**, not proof that the
original graph parameterization was used: a prior caller may already have passed
a projected native family to `from_family`. Loading and inspection must not infer
an original edge list, policy or active-to-original mapping from dimension alone.

Native portable validation should check the following associations without
rerunning algebra or partial fractions during loading:

- Original powers and original parameter lists have equal nonzero length;
  powers are positive and parameters distinct. If present, original edge IDs
  have that same length and are distinct, in the recorded native order.
- Active original indices are strictly increasing, in range and match the
  active powers' positive entries. Their selected original parameter vector
  equals the retained domain's complete source-parameter vector.
- An `Original` status preserves every index and power. A `Projected` status
  requires the admitting policy, a positive state bound and a strictly smaller
  nonempty active set. The status is preparation provenance, not a new proof
  certificate reconstructed from untrusted text.
- Existing chart/domain/power/Jacobian validation remains authoritative for the
  actual active parameterization. Original source edge count is not substituted
  for the active coordinate dimension. Numerical `ResultScope::FullIntegral`
  still denotes the same entire physical integral.

Live generation status can carry an optional native report (default/omitted
`None`) once preparation finishes; its Display should say, for example,
“10 original propagators → 8 active denominators; native single unit term”.
Fallback should name its native reason. `inspect` should reuse the same retained
metadata Display/transport for edge correspondence, powers and original labels,
with no CLI reconstruction of family algebra. The existing parametrization
timer includes preparation. If a separate preparation timer is useful, it is an
observation excluded from scientific identity, not another policy field.

Implementation would touch the owned preparation/integrand modules, the CLI's
config/input/status adapter, and one coordinated carry-through in
`generation/mod.rs` plus portable metadata. The generation owner must approve
that small seam while subtraction work is in progress. Required checks are an
explicit no-op/original round trip with legacy identities preserved, projected
noncontiguous native IDs/labels/powers after fresh-process load, invalid metadata
associations rejected before compilation, changed-policy resume rejected before
work, and the existing analytic graph weight 210 → integral 70 through the CLI.
No test should replace the independent original-density controls by prepared
ones. A bounded alternating parameterization-only measurement on ordinary
triangle/box inputs should establish no-op cost before changing the default.

For that measurement, use the existing native `triangle`, `box`, `bubble` and
`box_rank2_numerator` cards. First verify each family's native independence and
unchanged status, full original label list and exact resulting term/Atom data.
Keep graph/model/kinematic loading outside the parameterization timer and apply
the same scalar contraction and Gaussian work in both modes; a comparison of
`prepare_family` alone with a complete original parameterization would be
misleading. In each fresh process, run five untimed warm-ups then 100 timed
parameterizations of one fixed native graph, alternating mode order across
seven paired processes per fixture. Retain every batch time, median per-call
cost, source/build identity, policy/status and exact term-identity result. Do not
count JIT/kernel generation or external oracle work in this no-op timer. If a
batch is too short for useful resolution, preserve the initial block and run a
new complete block at a predeclared larger batch size rather than selecting
individual timings. Ordinary-mode timing is a no-op overhead check; projected
benefits are already separate full-generation evidence, and on-shell Laurent
cost remains unresolved.
