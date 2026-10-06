# Generation summary and live count review

Independent review of the 2026-10-07 presentation follow-up. Scope is the final
CLI generation report and the availability of live coefficient counters. No
symbolic transformations, evaluator layouts, integration statistics, existing
tests or other examples are changed by this review.

## Existing owner contracts

- `KernelSet::orders()` and `KernelSet::components()` are parallel descriptions
  of the complete evaluator output vector. For a complex Laurent vector,
  `[-1, -1, 0, 0]` means real and imaginary parts at orders minus one and zero.
  Presentation must retain both components and must not infer missing orders
  from a minimum/maximum range.
- `GenerationTimings` contains observed caller wall intervals. The combined
  coefficient-expansion interval includes its exact physical fallback. In
  dispatched generation its subtraction and Laurent operations overlap across
  workers, so only the aggregate interval is measured; the individual fields
  remain zero. These are not evidence that those operations took no time.
  Recorded generation total includes input and artifact-metadata preparation
  but excludes the final artifact writes.
- The CLI already owns `ColorPolicy`, which disables color for `--plain`, a
  redirected target stream, or the presence of `NO_COLOR` (including an empty
  value). The final report must inspect stdout, while the dashboard uses
  stderr. Ratatui and Crossterm are existing direct CLI presentation dependencies;
  `tabled` previously appears only as a transitive Spenso dependency.
- Native named-series observations reset on each attempt. Formal-piece counts
  become known at coverage; request observations arrive from the existing
  request owner. Availability must distinguish pending observations from a
  measured zero, survive later stages in the same pass, and reset for a new
  pass or representative. Exact physical fallback has no named-series count.

## Verification

The independent public-event probe in ignored
`output/generation-summary/status-probe.rs` and `status-probe.txt` passed:

- Admission, regular-series expansion, naming, endpoint composition and
  coefficient composition hide unavailable counts, even when deliberately given
  nonzero placeholder values.
- Coverage displays a measured zero subtraction-piece count. Lowering displays
  zero or nonzero resolved coefficient requests and shared expressions as
  running counts ("so far"). Completion preserves measured zeros without the
  running qualifier.
- A new expansion pass or representative hides the preceding counts. Physical
  fallback and its completion show the full-expression route without invented
  named-series counts.

The native request owner increments the distinct counter after resolving a
request, and its alias counter after creating the shared expression. The new
wording therefore describes completed work. Existing structured integer fields
remain unchanged, with their stage-specific availability documented; no
computation or serialization schema change is introduced.

The final renderer uses `tabled` 0.22, already present through Spenso, as an
explicit CLI-only dependency. Its existing `Builder`, `Width::wrap`,
`BorderColor`, `Color` and `Alignment` APIs own wrapping, terminal-cell geometry,
color and alignment. The small CLI policy sets column budgets and chooses a
stacked layout below 32 columns; it introduces no second text-width or table
implementation. Ratatui remains the existing live-dashboard owner. Typed
kernel metadata reaches the renderer directly, without extracting numerical
meaning from formatted JSON.

The independent temporary renderer probe (`output/generation-summary/renderer-probe.rs`)
included the production renderer and color policy directly, with an inert
artifact stub only to satisfy the unused I/O wrapper. Its pure renderer checks
passed at 120, 80, 64 and 40 columns, plus the 31- and 20-column stacked layout:

- Native `tabled` display-width measurement confirms every bordered row aligns
  with its borders and every line fits the requested width. A long unbroken
  relative filename, CJK wide characters and a combining accent survive wrapping
  without character loss.
- Noncontiguous orders minus three, zero and two remain distinct, with real and
  imaginary components grouped only where supplied. No intervening exponent is
  invented. The shortened fingerprint is explicitly labeled `ID (short)`;
  structured output retains the full identity.
- Zero-sector exact-only output, an empty output list, no runtime inputs and
  unavailable timing metadata render without assuming stochastic sectors exist.
- Sub-millisecond, millisecond, second, minute and hour durations are readable.
  Unmeasured zero-valued subphases are omitted, the combined coefficient phase is
  shown once, and the total explicitly excludes writing the artifact files.
- Plain and redirected-stream policies produce identical escape-free output.
  An empty `NO_COLOR` value disables terminal color too. Removing SGR styling
  from the colored output yields the exact plain output, including geometry.

Evidence is in ignored `renderer-probe.txt`, `renderer-no-color-probe.txt`,
`normal-*-plain.txt`, `normal-*-terminal.txt` and `no-color-*-terminal.txt` under
`output/generation-summary/`. Temporary executable sources were removed from the
workspace crates and retained only in that ignored directory. No tests or gates
were added or migrated.

The final I/O wrapper was inspected separately: stdout owns terminal detection
and width, redirected output uses an 80-column default, `ColorPolicy` remains
the single color decision, paths still go through the portable relative-path
owner, and machine JSON keeps its original field types while adding the
one-to-one `components` array. Only the final `generate` report is rerouted;
integration presentation is unchanged. End-to-end CLI runs and release checks
are recorded by the parent/implementation reviewers separately.

Independent acceptance: no blocking correctness, native-reuse or presentation
issue found in this bounded change. This does not claim deferred numerical
test/gate migration or a new scientific benchmark.

## Release CLI acceptance

The coordinator rebuilt `target/release/fastsecdec` and exercised the actual
command with a small complex endpoint-pole integral, whose output layout is
`[-1,-1,0,0]`. A pseudo-terminal captured colored stdout at 80 and 40 columns,
plain output at 64 columns, and `NO_COLOR=""` at 80 columns. Every table row
aligned within its borders and every line fit the terminal width. Colors were
present only when enabled. Redirected stdout contained no escape sequences.

The displayed Laurent table has one row each for epsilon^-1 and epsilon^0,
both explicitly labeled real/imaginary. Phase times use bounded precision and
readable units, and neither the nested metadata dump nor the ambiguous raw
order list appears in the human report. JSON retains the preceding machine
fields and adds the aligned `components` array.

Every generated binary payload was byte-identical to the same input generated
before these presentation changes; scientific identity, orders, sector count
and worker settings matched. These checks cover the public command and do not
require another expensive D05 generation. The existing native-generation event
probe covers the new live count wording and availability at every callback.

Release build, strict scoped core/CLI Clippy, `cargo fmt --all --check` and diff
checks pass. The installed Rust 1.99 toolchain was used because this host has no
`nix-shell`. Evidence is confined to ignored `output/generation-summary/`,
including `check_cli.py`, `cli-checks.json` and captured terminal/plain output.
