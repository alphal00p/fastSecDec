# Named regular-coefficient composition: independent review

Source review, 2026-10-05. The test-only implementation in
`generation/subtraction/series_first/named.rs`, its three focused controls and
the extended existing five series-first controls have no source blocker for a
bounded small execution gate. Production subtraction is unchanged. Compilation
and the bounded eight-control execution now pass, as independently checked
below. The subsequent small native-program/backend compatibility layer also
passes the independent outcome review recorded at the end of this note.

The implementation names only native epsilon-independent regular-series
coefficients, with their actual coordinate dependencies as function arguments.
It reserves input symbols, deduplicates exact native bodies and requires
literal native restoration to reproduce every original coefficient and its
absolute remainder. Native `map_coeff`, derivatives, endpoint substitutions,
addition and multiplication remain the mathematical owners. The extra scalar
guard rejects a formal coefficient in endpoint/prefactor multipliers. No
inverse or nonlinear series function operates on an opaque coefficient.

This linearity matters for the remainder: hidden identities can retain extra
terms and a conservatively lower leading order, but cannot justify discarding
an unknown tail. The existing native final-bound test and checked retries remain
unchanged. The exact unregulated-endpoint fallback still uses ordinary native
subtraction; naming cannot change admission there. Joint regularity is the same
input premise as the prior series-first experiment, not a new claim about
moving singularities.

Ordinary native derivative tags identify body/multiindex/argument requests.
The local caches derive each requested body with native derivatives and perform
simultaneous literal face substitution. No custom derivative, series or chain
rule engine is introduced. Final roots use ordinary native alias handles and
one shared body map; unresolved formal names or derivative requests are
explicitly rejected at that boundary. There are no registered callbacks or
process-lifetime captured polynomial caches.

The controls compare the full union of named and physical-body orders with
bounded exact native restoration, rather than sampling only a finite value.
They exercise composed mixed derivatives, exact zero faces, coefficient-body
deduplication, actual dependency arity, invalid fractional/essential series,
unregulated endpoints, the scalar guard and compact high powers. Existing
Taylor/IBP and Gamma/order controls remain part of the small gate. Native
numerical evaluator, hidden-complex, weighted replay, worker and fresh-process
IR checks are a separate pending layer; no actual captured representative or
production promotion follows from these small source controls.

The concrete `native-named-coefficients-small-1` wrapper and build record also
pass independent preflight: all 15 immutable hashes match, the filter selects
eight normal controls and two ignored earlier replays, and the owned process
has a 180-second plus five-second grace/30-GiB address-space/CPU-8 bound. It
rejects an existing process directory and retains pre/post hashes. The recorded
development build and optimized dependency are correctness provenance, not a
release timing claim. Execution waited for the external reference's
Symbolica-generation exit/reap handoff.

The completed small gate was independently checked: all eight normal controls
passed, both earlier replay probes remained ignored, and the process exited
zero without timeout in 0.125756 s. Native wait4 peak RSS was 15,360 KiB under
the timer's recorded waited-child semantics. All 15 immutable postflight
hashes and the timer's input/build/executable checks pass. The expected scalar
guard panic is a passing explicit rejection control, not a swallowed worker
failure. The process-group SIGKILL after successful leader completion is
the timer's cleanup action; the child itself exited zero. No actual-target,
full-integral, O2/MPFR or cold-IR claim follows from this core gate.

## Small native program and cold-process gate

The separately frozen `output/diagnostics/native-named-program-2` execution
passes independent source/output review. All 17 immutable hashes and all five
writer-output hashes were rechecked; the compact audit is retained as
`independent-review.json` alongside the reports. Attempt one remains preserved:
its writer launch failed in the canonicalized Nix `env` applet before the test
ran. Attempt two exports the test paths before invoking the timer, retaining
the same scientific binary and bounds. This launcher failure is distinct from
the expected passing scalar-guard panic in the symbolic controls.

Taylor and IBP each retain all four orders `[-3,-2,-1,0]` and compare every
restored coefficient exactly against the unchanged production subtraction and
Laurent path. Both native series retries reach absolute remainder one. The
hidden complex-body checks are nonvacuous, and the resulting native IR sizes
are 2,105 and 2,353 bytes. Three prescribed exact rational points independently
require the production baseline to agree at 512 and 1024 bits before recording
reference values. The named native evaluator agrees with those complete complex
vectors at both precisions.

Fresh O2 kernels, native cloned workers, same-process decoded programs and a
separate cold reader each check three points, two weights (`1` and `1e40`) and
ordinary/forced evaluation. These are 96 full-vector calls and 768 scalar
component checks in total. Their retained reports agree exactly across all
four construction paths; 80 calls use rescue, with a maximum observed precision
of 320 bits. Forced calls all report checked and rescued. Weighted comparisons
use MPFR at the same rounded binary64 coordinates as the production call,
separately from exact-rational oracle checks.

The cold process imports only parameter atoms, native exact IR and the frozen
expected values. It constructs neither a formal-coefficient registry nor a
callback, and verifies both strategy identities, full order vectors and program
digests. The symbolic process passes eight controls with two earlier replays
ignored; writer and reader each pass their explicit ignored test. All processes
exit zero without timeout. Observed process durations are 0.126907, 15.663044
and 4.441060 seconds, respectively. They are bounded correctness diagnostics,
not a performance comparison. No difficult representative, full integral,
generation-speed improvement or production subtraction switch is established
by this small compatibility gate.
