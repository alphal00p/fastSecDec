# Native named coefficient CLI/status bridge

Independently accepted on 2026-10-05 with the coordinated public opt-in: the
workspace gate passes **370 tests**, with 23 intentional ignores; formatting
and all-target Clippy pass. This is the presentation slice of
[the production adoption plan](native-named-production-slices.md). The numerical
coordinator, native composition and request owners remain in the library.

`GenerationInput.coefficient_expansion` directly deserializes the public native
`CoefficientExpansionOptions`. The nested TOML spelling is
`[generation.coefficient_expansion] method = "native_named"`; omitted settings
retain `Physical` and impose no additional named caps. Unknown option fields and
method names are rejected by the native leaf's serde contract. The CLI passes
this value into the existing `GenerationOptions` path for ordinary and
caller-dispatched geometry. It exposes no resolver or evaluator implementation
switch and introduces no symbolic operation.

`CoefficientExpansionSnapshot` stores requested/effective methods, the zero-based
representative, stage, attempt, relative width, formal pieces and the native
request counts. Counters belong to the current attempt and reset on retry.
Zero attempt/width means admission before a named attempt or exact physical
fallback; width never represents absolute Laurent coverage. Physical fallback
retains the requested named method and effective physical method through its
completed event. Its physical piece count is not relabelled as formal pieces.
Untouched physical runs omit this optional field. The outer completed count
stays at the number of finished representatives until the current one completes.

The named phase uses only `coefficient_expansion_seconds`, including exact
physical fallback. The existing physical subtraction/Laurent timing fields are
not populated again for that work. Old timing payloads default the new duration
to zero; old snapshots default the optional coefficient observation to absent.
The existing artifact/report timing transport automatically retains the new
field without changing numeric kernel serialization or content identity.

Named coefficient JSON presentation reuses `StatusCadence` with the snapshot's
caller wall time. Ordinary request changes may be coalesced; first named state,
attempt changes, effective-route changes and completed representatives force
output. Other generation JSON retains its existing unthrottled semantics,
including distinct chart/cone admission observations, compiled kernels and final
saved state. Existing terminal/plain cadence is unchanged. Integration cadence
keeps its own existing state and policy. No new symbolic worker or execution controller
is introduced. Every library callback still reaches cancellation even when
output is suppressed. Existing typed errors, final error reporting and terminal
restoration are unchanged; failure does not manufacture a `Complete` event.

The passing focused tests cover native option/default admission, old timing
payloads, attempt reset/fallback observations, completed-sector semantics,
cancellation while output is suppressed, a real explicit CLI card under both
zero and long status intervals, and a resource error with visible final error,
no completion and no saved artifact. The process test compares output layout,
timings and status, **not numerical coefficient values**. Complete scientific
vectors and cold native artifact/replay checks belong to the coordinated public
generation tests. The actual captured representative and full graph remain
separate gates.

Two presentation corrections preceded acceptance. The resource-failure test
initially looked for the final error in stderr; the existing `--json` contract
puts that error document on stdout, independently of stderr status rows. The
first combined run also exposed lost chart/cone observations when generic
generation coalescing hid their fast transition. Restricting coalescing to named
coefficient JSON restores the existing geometry contract and original
terminal/plain cadence. The five targeted CLI regressions and subsequent full
workspace run pass without weakening the prior geometry assertion. Neither
correction changes numerical work.

The CLI guide documents limits and their scope. Loaded conditioning profiles
retain generic terminology; the fresh result's descriptive basis is not inferred
from saved numeric rows. Physical defaults, full-graph and matched-performance
acceptance remain separate decisions.
