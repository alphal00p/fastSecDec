# Evaluator settings: CLI handoff and presentation review

Review date: 2026-10-07. The reviewer did not author the native optimizer,
configuration handoff or artifact codec changes. The reviewer authored the
shared human settings rows and their use in generation/inspection.

## Configuration and artifact handoff

`GenerationInput.evaluator` uses the public typed `CompilationSettings` under
`[generation.evaluator]`. Missing fields use the declared native-policy defaults:
10 Horner iterations, at most 1000 CPE rounds and one optimizer core. The CLI
passes the exact copy into caller-dispatched sector compilation and records it
as `GenerationRecord.evaluator`. Old generation records omit this optional
field and remain explicitly unrecorded in human output.

The native public settings owner validates deterministic single-core optimizer
execution; parallelism remains across caller-owned sector jobs. TOML supports
nonnegative integer CPE rounds or the explicit string `unlimited`; unknown
fields, negative counts and invalid names are rejected. The native builder owns
Horner/CPE transformations. No CLI optimizer, algebra helper or graph adapter
was introduced.

The binary compiler-policy string retains the complete settings as canonical
JSON, inside the existing context-binserde envelope and semantic identity. This
avoids sending the TOML-facing untagged integer/string representation through a
bincode serde decoder that does not support `deserialize_any`. Native evaluator
bytes retain their existing owner codec. The loader recovers the saved policy;
historical zero-Horner/unlimited-CPE policies remain recognized. Quiet transient
reconstruction of runtime-dependent exact offsets does not replay an old
`verbose=true` setting into an `inspect --json` or integration stdout stream;
the saved settings and original artifact bytes remain intact.

## Operator-facing output

Generation and inspection use one `evaluator_rows` helper. They show saved
Horner iterations, CPE rounds (including Unlimited), deterministic optimizer
cores, Horner-variable cap, CPE-cache cap, pair distance, direct-translation flag
and verbosity. The known upstream-inactive pair-distance control is identified
as inactive. Values are read from saved generation metadata, not inferred from
the current executable's defaults. Native direct translation is a builder
request: upstream can force that route for non-inlined functions.

The native tabled formatter owns widths, wrapping and color handling. A
20-column label budget avoids unnecessary split labels at 80 columns while
retaining narrower wrapping. Inspection still distinguishes kernel IDs from
source-chart IDs, native mathematical formatting, serialized evaluator size
from compressed SymJIT data, and backend lowering. Operation-count prose now
identifies the stored shared program as post-native-symbolic optimization and
pre-backend optimization.

Verbose generation is deliberately limited by CLI preflight to plain human
output, rejecting `--json`, `--status-json` and interactive output before worker
or terminal initialization. This preserves native verbose logs without
corrupting structured stdout. The independent native audit identified the
upstream tracing default's stdout sink and the cold-load reconstruction path;
the implementation owner fixed the latter with a transient quiet builder.

## Focused evidence

The ignored Rust probe in `output/evaluator-settings` passed:

- default 10/1000/one-core values and exact user overrides, including unlimited,
  zero native caps, direct translation false and verbose true;
- invalid integer/name/unknown-field and non-single-core rejection;
- actual `GenerationRecord` JSON round trip, plus old records omitting the new
  optional field;
- shared human rendering at 120, 80, 64, 40 and 24 columns, preserving all saved
  settings and explicit unknown legacy metadata without ANSI in plain output;
- cold loading the existing ggHH artifact under its zero-Horner/unlimited-CPE
  policy, with original `.dat` bytes unchanged.

The probe source was moved out of CLI examples into ignored output. No permanent
examples or tests were migrated. Native optimizer arithmetic/determinism and
portable execution are reviewed independently by the kernel reviewer.

## Final-release CLI checks

The final release generated a native runtime-kinematic bubble artifact with
`verbose=true` and `RUST_LOG=info`. Its generation stdout contains native
optimizer logs, including the exact helper's zero-operation program. Fresh
`--json inspect` and `--json integrate` processes with the same logging level
both produced parseable JSON without optimizer-log contamination. Inspection
retained the saved verbose flag, runtime symbol and byte-identical `.dat` file.
Evidence is under ignored `output/evaluator-settings/verbose-*`.

A separate bounded two-round ggHH discrete-MC run exercised the historical
accepted-reference display on the final release. Both plain output and a native
120-by-32 PTY showed the previous completed allocation during the next pilot,
explicit pilot/batch waiting reasons, and the macOS Free label. The PTY restored
terminal attributes, alternate screen and cursor. Both runs had zero failures;
their final 1024-point means, errors and complete covariance were identical.
This is a presentation/state-transition control, not a convergence or timing
benchmark. The PTY run took about 1.37 seconds while other validation work ran.
The harness and captures are retained under ignored
`output/dashboard-accepted/multiround-*`; it checks the complete explanatory
reference line because Ratatui can update individual title characters rather
than re-emitting the entire title in one terminal write.
