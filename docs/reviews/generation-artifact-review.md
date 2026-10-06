# Independent generation artifact review (2026-10-06–07)

Reviewer: the runtime-parameter implementation agent, independently reviewing the
artifact agent's serialization and CLI persistence changes. Runtime binding is
covered separately and is not self-certified by this review.

## Owners and reuse

The numerical core retains native Symbolica `Atom` and
`ExpressionEvaluator<Complex<Rational>>` objects. The new binary payload derives
context-aware bincode encoding/decoding with `StateMap`; `State::export_partial`
and `State::import` own symbol-state transport. The metadata `StoredAtom` adapter
uses canonical strings for human JSON presentation only and delegates binary
Atom encoding directly to Symbolica. The existing native complex and interpreter
mapping paths reconstruct numeric evaluators after loading. There is no new CAS,
graph model, evaluator instruction format, thread pool, or integration loop.

Source inspection confirms the existing native evaluator serialization owns
instructions, rational constants, parameter count, output positions, external
function containers and optimization settings. A concrete native integer-codec
defect required the evaluator fields to retain their existing serde/bincode
binary codec within the custom context-binserde payload, as detailed below. External function containers
serialize their names/symbol definitions, tags and fixed arguments through their
native owner. Calling `transcendental::gamma()` initializes the shared special
function set, including polygamma and Euler's constant, before cold loading.
One-loop master and reduction APIs remain separate scientific references; this
persistence change does not replace or duplicate them.

## Reviewed boundaries

The CLI derives sibling `.json` and `.dat` names from one basename, stores no
sibling path, and excludes native program arrays from metadata. It writes data
before metadata. This is explicitly a two-file update, not a transactional pair;
a mismatched pair is rejected on load. Native byte integrity, semantic kernel
identity, and CLI provenance identity have separate responsibilities.

The raw digest covers the context and payload bytes; semantic identity includes
the ordered runtime schema, Laurent layout, numerical policy, programs and
retained metadata. Layout, duplicate-coordinate/schema, and metadata ownership
checks remain native. The existing trusted-cache boundary is retained: hashes
and dimensions do not claim to validate malicious native instruction streams.

Sources are stored relative to the artifact directory, while explicit run-card
resume can relocate the input directory and rechecks its scientific fingerprints.
Stored reference paths are resolved against the loaded pair's directory, and
reference identities are checked after runtime parameter binding. Result-output
protection includes both artifact siblings, source files and the reference.

The review found that lexical `..` elimination could change source meaning when
a path passes through a symlink. The artifact author corrected this using transient canonical resolution of the
nearest existing ancestor, retaining only relative persisted paths. No absolute path is needed in the cache.

## Evidence and current acceptance boundary

A manual symbolic bubble was generated through the CLI and loaded in fresh
integration processes. Renaming and moving the pair still permits integration;
original source files are not required by `integrate`. JSON inspection confirms
relative source paths and no program arrays. Flipping one payload byte is rejected
by the native digest. A stored reference bound to `s=-1` is accepted at that point
and rejected at `s=-2`; changing the runtime point also rejects checkpoint resume.
The original examples and committed tests were not edited for these controls.

The first full ggHH cold load exposed a semantic identity mismatch. Independent
focused probing established that Numerica 3.0.1's GMP `Integer::Large` native
binserde writes unsigned `rug::Integer::to_digits` and reads unsigned
`from_digits`: a large negative rational coefficient became positive, as did a
large negative imaginary component. Positive coefficients and gamma/polygamma
callback controls passed. The failed artifact was rejected rather than evaluated.

The correction keeps every explicit Atom on native `Decode<StateMap>` and
uses the established native evaluator serde/bincode bytes with its borrowed
decoder for each evaluator. This preserves coefficients and callback fixed
arguments without introducing a numeric codec, modifying dependencies, or
weakening semantic identity. A format-header change rejects the intermediate
unsafe payload schema.

The independent final-format probe writes whole real and complex KernelSets
with large negative coefficients, starts a fresh child after registering 100
unrelated symbols, and successfully reloads the same identities and evaluates
the expected negative real/imaginary components. A separate native Atom context
round trip preserves a large-negative expression. The established evaluator
binary codec preserves positive, negative, complex, gamma and polygamma cases.
The expanded native runtime-parameter and exact-offset probe also passes.
These controls close the identified coefficient-corruption boundary. The final
ggHH format-5 regeneration completes in 48.34 seconds and a separate cold CLI
inspection succeeds with 30 kernels and 15 runtime parameters. Unbound exact
offsets are displayed as unavailable. The 14,314-byte JSON and 2,113,944-byte data
siblings contain no absolute paths. This closes the artifact review for the
requested generation round. A fresh-process bound ggHH smoke run also completes
122,880 evaluations on eight workers with zero failures, preserving the complete
covariance. It stops at the work limit; this is an integration/transport control,
not a new convergence claim.

The standalone portable-feature runtime probe was rerun for format 5 and passes on the host and establishes
continued use of the Symbolica interpreter and portable arithmetic. It is not
an actual Pyodide, Wasm, browser, or updated notebook execution result.
