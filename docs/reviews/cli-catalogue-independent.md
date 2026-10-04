# Independent CLI catalogue and checkpoint review

Reviewer: Numerica/runtime author, independent of the CLI adapter author.
Scope: `config.rs`, `main.rs`, driver configuration/refinement/checkpoint/
execution/report code, CLI catalogue process tests and driver regression tests.
The underlying native catalogue/design APIs are not claimed as independently
reviewed by their own author here; their separate review is recorded in
`native-cache-and-catalogue-independent.md`.

No actionable correctness finding in the reviewed CLI source.

- Missing historical `lattice` values deserialize as `kuo33002`. Default or
  explicit historical selection is omitted from serialized CLI settings and
  becomes native `RuleSource::Kuo`, preserving existing checkpoint semantics.
  Every alternative becomes a distinct typed native published selection.
- Native `QmcSettings::validate` owns point-count validation; the refinement
  driver obtains the selected catalogue's cap from Numerica. Unsupported
  dimensions are rejected by native session construction without fallback.
  Refinement retains the chosen rule and grows shifts only after reaching its
  published point cap.
- Resume first checks outer artifact/configuration identity, then restores and
  validates the native session. Before creating evaluation contexts or
  requesting work, it compares the session's full numerical settings against
  the expected refinement-round settings and checks the statistical method.
  This binds rule, seed, transform, package size, points, shifts and method.
  Adaptive sector allocations remain native-owned and validated during restore.
- Mismatch regressions deliberately combine valid native sessions with wrong
  outer catalogue, method or round. They require rejection with the checkpoint
  file unchanged. The source places these checks before numerical evaluation,
  not merely before checkpoint writing.
- The final optional native `QmcDesign` records actual allocation counts for
  the final round, including adaptive allocations that differ from initial
  settings. MC reports omit it. It does not change coverage or stopping logic:
  uncertainty and accuracy remain governed by the native complete-production
  checks, and incomplete/cancelled state is not promoted to convergence.
- JSON reporting returns through the structured branch before the human
  `QmcDesign` display. Catalogue process tests parse the entire stdout as one
  JSON document, inspect native rule identities, verify unchanged artifact
  bytes, and resume with another worker count without repeating accepted work.

The existing 26 native-runtime focused tests passed in
`output/runtime-catalogue-tests.log`. The CLI owner reports the completed
focused gate in `output/cli-catalogue-tests.log`: 23 passed (15 unit, two
catalogue processes, four existing CLI processes and two reference processes).
The log was inspected for the matching completed test summaries. No duplicate
numerical runtime or cosmetic test was added for this read-only review.
