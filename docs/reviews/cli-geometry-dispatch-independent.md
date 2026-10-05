# Independent review of CLI geometry dispatch

The proposal and applied CLI implementation have no blocking source finding.
The 49-test CLI gate and separate interactive-terminal acceptance passed.
The following records the interface review followed by concrete source and
executed gate evidence.

The native geometry planner and both ordered admission stages remain the only
owners of maps, limits and scientific error precedence. The CLI receives opaque
jobs and returns opaque completions. Finishing a stage after a scientific job
failure preserves native canonical precedence; cancelling for a terminal or
scheduling failure instead stops admission and cannot create an artifact.
Capacity-zero context use and a separate geometry-worker flag avoid introducing
a cache lifetime, changing numerical settings, or altering checkpoint identity.

Installed `rayon-core` 1.13.0 supports the proposed caller-thread coordinator:
`thread_pool/mod.rs` exposes `ThreadPool::in_place_scope`, and
`scope/mod.rs:398–411` invokes its closure without a `Send` bound. Its documented
join guarantee also covers panics. The adapter must still account for a worker
panic before a completion message, because a coordinator waiting for that
message would otherwise prevent the scope from reaching its join. The proposal
explicitly requires this handling and a regression. Mandatory completions must
remain deliverable while the coordinator drains cancelled work; only progress
observations may be dropped.

The existing `crates/fastsecdec-cli/src/generate.rs` owns presentation errors,
terminal polling, generation/compilation cancellation and artifact publication.
Keeping those owners on the calling thread is appropriate. Tagged local job
progress must not replace native accepted map counts. The proposed bounded
receive loop, latched cancellation, forced stage transitions and final native
completion preserve that distinction. A local `RefCell` is sufficient only if
no observer is called while a presentation borrow is live, as specified.

The proposal called for serial/parallel complete artifact and analytic
comparison, reversed completion/cancellation/panic lifecycle controls, empty
stages, original error retention, and terminal cleanup. No speedup, memory bound
for the completed fan, or concurrent Atom evaluation is established by this
proposal review.

## Applied source review

The concrete `generate/geometry_dispatch.rs`, generation/main/display wiring and
new process tests preserve the proposed boundary. The scheduler limits active
jobs, uses nonblocking sends only for progress, and transports worker panics as
mandatory completion failures. Its scope-local receiver drops before Rayon's
join if the coordinator unwinds; the unwind guard also latches cancellation.
Consequently a completion sender cannot remain blocked on a dead receiver.
Ordinary native scientific errors remain opaque completions for ordered native
admission. Presentation errors retain the first message and stop new scheduling.

The dashboard and `RefCell` remain on the coordinator. No mutable presentation
borrow is live while the next native generation observer is invoked. Keyboard
cancellation now latches the same atomic flag used by workers. The original
serial route remains at one geometry worker; resume loads its artifact without
creating a geometry pool. CLI argument and checkpoint inspection show that
geometry workers remain separate from numerical workers and artifact identity.

The authored process control compares serial and parallel artifact contents
after removing only observed generation timings, then checks the complete
analytic vector `[1, 0, pi^2/12]` at orders `[-2,-1,0]` and complete integration
coverage. It also checks unchanged numerical-worker settings, resume equality,
domain/limit error identity and absent artifacts after failure. Scheduler tests
cover mandatory panic transport, empty stages, active cancellation and caller
unwind. This source review introduced no new execution while the author owned
the focused runtime gate; its subsequent result is recorded below.

The author gate subsequently passed all 49 CLI tests across 12 suite summaries
(zero failed, three ignored), including the two scheduler lifecycle controls
and two new generation process tests. I inspected
`output/cli-geometry-dispatch-cli-tests.log`, the corrected checkpoint/artifact
assertions, and the completed CLI all-target Clippy log. Numerical worker counts
are deliberately omitted from checkpoint compatibility settings; the unchanged
value is instead checked under artifact provenance, while geometry workers are
absent from checkpoint settings. The earlier assertions used the wrong field
locations; their retained failures did not reveal a production defect. Format
and CLI Clippy completed successfully; the dependency's existing native warning
is unrelated to this adapter. Interactive-terminal acceptance remains pending
under the coordinator's separate PTY check. No scientific rerun was introduced
by this independent source/log review.

The coordinator's real-PTY checks subsequently passed. I read the retained
terminal protocol and streams, independently compared all three before/after
termios records, and checked exit codes: cancellation exits one; normal and
color-enabled completion exit zero. The cancellation stream contains the
wide/compact/wide geometry sequence and cancellation, with no artifact.
All three streams leave the alternate screen and restore the cursor. The two
monochrome streams contain no explicit foreground colors; the enabled-color
stream contains ten. Normal/color artifacts compare exactly after excluding
only generation timings. Evidence is under
`output/diagnostics/cli-geometry-dispatch-pty-{2,normal-1,color-1}` and the
coordinator's `cli-geometry-dispatch-terminal.md`. The first PTY harness failure
is separately preserved: GNU timeout needed `--foreground`, with no production
change. This closes the bounded CLI caller/terminal acceptance without claiming
parallel speed or full-graph generation performance.
