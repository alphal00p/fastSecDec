# Native reference CLI integration review

The CLI adds file selection and presentation around the existing library
`fastsecdec::reference` adapter. It owns no coefficient alignment, uncertainty
combination, pull calculation, normalization conversion, or independent
reference parser. `read_reference` validates the versioned native envelope or
historical schema; `compare` supplies typed rows, eligibility, and evidence;
plain output delegates to the library's `Display` implementation. Native
objects pass directly between those APIs after the external JSON boundary.

## Scientific and persistence boundaries

The optional root TOML `[reference]` section selects a file and records
normalization, kinematics, and independence evidence. Missing evidence stays
unknown. Evidence is a caller assertion, not automatic validation of a target.
Historical imports retain their unverified status and unknown uncertainties.
Missing orders or components remain missing; a real-only estimate does not
silently supply zero imaginary coefficients. When no numerical estimate is
available, the report preserves the reference and comparison context with
`waiting_for_coverage`, without invented coefficient rows.

Card paths resolve relative to the card. A `--reference` override resolves
relative to the current working directory. Changing the resolved target clears
inherited evidence; selecting the same resolved target retains it. The final
report records the selected file's canonical path and raw-byte digest.

Parsing and validation happen before graph generation or portable-kernel
compilation. A kernel-bound target is checked against a validated outer
artifact's kernel identity during preflight, before recompilation. Newly
generated kernels are checked before their artifact is saved. The loaded
native kernel identity is checked again before integration.

Reference comparison never changes the numerical integration settings,
sampling, stopping, accepted replay state, estimates, or checkpoints. Artifacts
save reference steering separately from scientific identity. Newly generated
run-card source records use the explicit `run_card_without_reference`
fingerprint kind: parse a native TOML table, remove only the root `reference`
key, serialize as native TOML, and hash it. Every other parsed root field,
including unknown fields, remains in the fingerprint. Native TOML serialization
preserves date/string and special-float distinctions. Comments and formatting
are not numerical identity. Model, graph, and polynomial files retain raw-byte
fingerprints and canonical source paths.

Legacy source records deserialize to `bytes`, which is omitted again when
serialized. Thus their original byte-based verification and artifact identity
remain intact; they are not silently upgraded to a new fingerprint rule.

## Review and executable evidence

This document records the implementation owner's evidence. The root agent
independently reviewed the wrapper and persistence changes. That review led to
three concrete corrections: inherited evidence is cleared for a changed target;
known target/kernel mismatches are rejected before JIT compilation; and native
TOML serialization replaces a lossy TOML-to-JSON fingerprint conversion.
The Numerica/reference-adapter agent independently reviewed the final CLI
wrapper, preflight, identity separation, and waiting-for-coverage behavior and
found no additional issue after those corrections.
The library adapter has its separate independent review in
`reference-result-independent.md` and reuse evidence in
`reference-result-reuse-audit.md`.

The focused CLI gate passed 22 tests: 13 unit tests, four existing CLI process
tests, three dependency-provenance tests, and two reference process tests.
Evidence is `output/reference-cli-tests.log`. The reference process coverage
checks a complete analytic unit-interval integral, reference-only run-card
changes during checkpoint resume, unchanged estimates and snapshots, unchanged
regenerated artifact identity, saved default steering, cross-directory path
resolution, same/different-file overrides, strong kernel-identity mismatch,
historical unknown errors, missing imaginary estimates, plain display, and
malformed-reference rejection before a deliberately missing graph is loaded.

After the final native-TOML fingerprint correction, the focused pure regression
passed again, including `nan` versus `inf`, a TOML date versus the same quoted
string, preservation of unknown root fields, changed numerical settings, and
legacy raw-byte behavior. Evidence is `output/reference-fingerprint-tests.log`.
Both reference process tests then passed again against the final fingerprint
and aligned library display, in 0.13 seconds after compilation. Evidence is
`output/reference-cli-process-final.log`.

The tests establish adapter and persistence behavior, not the correctness of
arbitrary external targets or statistical independence asserted by a user.
Reference comparisons remain diagnostic when the library's eligibility checks
are not met.
