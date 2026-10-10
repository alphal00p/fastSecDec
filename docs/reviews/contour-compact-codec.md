# Compact contour metadata transport

2026-10-10, implementation after `98aa8f9`. This slice retains native coefficient
definitions for inspection and restored chart semantics. It adds no numerical
representation or evaluator reconstruction on loading.

Only artifacts containing nonempty `ContourDefinitions` use native binserde
version 12. The existing versions 5–11 remain readable, and artifacts without
definitions retain their previous v9/v10/v11 emission and identity rules. The
v12 wrapper stores native definitions by strictly increasing **local retained
chart index**. It preserves the exact older nested contour layout rather than
guessing absent fields during decoding. Each legacy contour codec field still
delegates to the existing native Atom/StateMap codec.

Before encoding, native state export visits each definition's function symbol,
formal parameters and body symbols. Canonical semantic metadata includes the
complete definitions before their extraction into the v12 sidecar. Decoding
attaches definitions before semantic validation and native metadata admission.
Unknown/duplicate/out-of-order chart associations, missing contour records,
empty v12 sidecars, unresolved owned coefficient calls and unsupported contour
versions are structural errors even when optional digest verification is off.
Full body/name identity verification remains part of requested validation.

The numerical record still contains the original already-inlined native exact
instructions. Restoration does not construct a `FunctionMap`, differentiate,
optimize, or materialize the retained map. Human-readable metadata delegates
Atoms and symbol spellings to the existing canonical native adapters. Serial
archives continue to copy individual records with compact receipts; no global
definition table is introduced.

The root independently reviewed sidecar association, native symbol export,
semantic identity and historical layout preservation without finding a source
blocker. The combined all-target Rust check passed. The coordinated native test
binary then passed **48 artifact tests** (two subprocess helpers explicitly
ignored in the ordinary invocation) and **3 metadata tests**. The parent tests
actually invoke both subprocess helpers and require completion sentinels.
Logs: `target/contour-compact-artifact-tests.log` and
`target/contour-compact-metadata-tests.log`. The final combined rebuild also
passed **401 native library tests**, with 19 explicitly ignored helpers/stress
controls, including these same codec tests and the corrected definition-admission
and compound-argument controls. These are overlapping counts, not additional
tests. Its log is
`target/generation-agent-ltd-memory/compact-final-core-lib-tests.log`.
The passing codec coverage includes:

- Historical nested contour byte-layout comparison and existing legacy readers.
- Native metadata/JSON identity roundtrip and a fresh-process restore after
  perturbing native symbol insertion order.
- Malformed sidecar and checked versus unchecked body-identity controls.
- Genuine generated compact dynamic descriptors in both constructions and both
  generation modes, comparing complete vectors after direct restore and native
  archive partition/selective-sector restoration with actual checked callbacks.

The small artificial metadata fixture is explicitly a codec control, not a new
scientific integral. Full compact-map higher-jet, symmetry, subtraction and LTD
performance acceptance belong to the coordinated generation/runtime gates.

The independent Python inspection review also passes at source level.
`ContourRecipe` is frozen and unsendable; it retains either the existing generated
`Arc` owner or the compiled `Rc<KernelSet>`, so its local chart index cannot outlive
the immutable metadata it indexes. Native Atom getters return independent values.
The explicit definition getter constructs formal calls and returns existing
bodies without differentiation, substitution, optimization or evaluator creation.
Automatic rendering requests the definition count only; existing map/Jacobian
printing can still produce large text and is not a bounded-size inspection claim.
The new tests exercise independent returned lists and retained views after the
Python kernel owner is dropped. Installed-wheel execution remains a separate gate.
