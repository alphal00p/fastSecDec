# Parameterized generation and portable artifacts (2026-10-06)

This revision follows the user's gg→HH CLI feedback. The older domain-admission
requirements are superseded: generation and artifact loading do not test signs,
faces or interior points to certify threshold absence. New metadata records
`UserResponsible` and `UncheckedUserResponsibility`. The mapped-polynomial
nonzero constant-term check remains an algebraic sector-map invariant, not a
threshold test. Numerical failures still propagate; no failed evaluation is
replaced with zero. Historical certificate variants and test-only sign helpers
remain solely for old metadata and the separately deferred test migration.

`examples/gghh_double_box/run.toml` declares symbolic scalar products and
`point.toml` supplies their integration-time values. Symbolica's native ordered
inputs carry coordinates followed by runtime scalars through compiled or eager
evaluators, worker clones and precision rescue. Folded exact offsets use the
same parameter point. Missing, unknown and nonfinite values fail explicitly;
checkpoint/result identities distinguish different numerical points.

The CLI takes a basename, such as `output/gghh_double_box.fsd`. Its `.json`
sibling contains human metadata; `.dat` contains native expressions and exact
evaluator programs. Symbolica's `State::export_partial`, `State::import`, native
`Encode` and context-aware `Decode` restore atoms, matching GammaLoop's
`HasStateMap` decoding pattern. Evaluator programs reuse their existing
sign-preserving owner serde/bincode codec: the Numerica 3.0.1 native GMP integer
codec loses the sign of large negative integers, which the full gg→HH cold-load
identity check exposed. A focused probe reproduced that sign change. No scalar
codec, numerical implementation or dependency patch is added, and the semantic
check is retained. A binary digest checks file integrity separately
from semantic identity. Machine code is rebuilt for the selected backend.
Source/reference paths are relative, and the artifact has no stored path to
its data sibling. Loading kernels does not require the original source tree.

The CLI owns the worker pool and dispatches native geometry, mapping, Laurent
expansion and compilation jobs. Native admission validates job ownership,
coverage and deterministic ordering. Symmetry registration/assembly remains
serial. The dashboard shows actual worker activity, stage totals, elapsed time,
percentage and estimated remaining stage time. Later stage sizes are discovered
dynamically. Cancellation joins owned work; interruption within a long native
algebra or compiler call remains cooperative.

The eager portable backend remains separate from native SymJIT O2. This round
checks the portable backend on the host; it does not claim a newly built Wasm
wheel or a Pyodide/marimo execution. Native gg→HH remains SymJIT O2.

## Verification boundary

The user explicitly deferred other-example changes and test/gate migration.
Existing test files are not rewritten in this revision. Temporary probes and
raw outputs stay outside tracked deliverables. Full scientific and performance
gates remain pending; old fixed-point timing/convergence evidence is not
relabeled as a result for the new symbolic workflow.

This host has no `nix-shell`; focused checks use its installed Rust 1.99 and
native build tools. The dependency graph and lockfiles remain unchanged.

Focused evidence collected during this revision:

- A temporary native parameter probe passed ordered inputs, missing/unknown/
  nonfinite rejection, cold binary reload, worker cloning, weighted precision
  replay, changed-point identity isolation and parameterized exact-only offsets.
  The same probe passed with the portable interpreter and Malachite/Astro.
- Five existing portable controls passed unchanged: four complex-kernel controls
  (including QMC integration with covariance and boundary rescue) and the native
  Gamma/endpoint Laurent-vector control. This is eager host execution, not Wasm.
- A CLI bubble was generated once and integrated at two runtime points. The
  finite-part difference agreed with `ln(2)` within `1.86e-13`; pole coefficients
  agreed, and changing the point rejected checkpoint reuse.
- A direct mixed-sign `1/(1-2*x)` input generated and cold-loaded successfully,
  recording the unchecked threshold policy. It was not integrated or reported
  as a finite integral.
- Both physical and native-named six-kernel examples produced identical artifact
  bytes/identities with one and four generation workers. Cancellation during
  active mapping joined workers and published neither artifact sibling.
- Final format-5 gg→HH generation completed in 48.34 seconds on eight workers,
  producing 30 kernels and 15 runtime inputs. Its human metadata is 14,314 bytes
  and its binary payload is 2,113,944 bytes. A fresh process loaded the pair in
  approximately 0.565 seconds; neither sibling contains absolute source paths.
- A subsequent fresh-process gg→HH integration bound `point.toml` and completed
  122,880 evaluations (1,024 points, four shifts, 30 sectors) in 20.523 seconds.
  It retained the complete Laurent covariance, performed 16,320 precision
  rescues up to 256 bits, and reported zero evaluation failures. The imaginary
  pole was `1.4430044188 ± 0.0153710267`; the imaginary finite coefficient was
  `-34.0318589554 ± 0.3630719805`. Both real components were zero. This allocation
  ended at its work limit and is an end-to-end smoke check, not a new convergence
  or performance-parity gate.
- The final release build, strict workspace library/binary Clippy, root and
  Python-binding formatting, Python-binding library check with `python_stubgen`,
  and whitespace checks passed. A controlling-terminal dashboard check exercised
  colored worker rows, aggregate progress and ETA, `NO_COLOR`, and cancellation
  with `q`.
