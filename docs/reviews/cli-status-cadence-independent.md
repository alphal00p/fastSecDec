# Independent CLI status-cadence review

Reviewed the CLI author's `status_policy.rs`, dashboard adapter, QMC/MC execution
loops, command option and focused process tests after milestone `fe3b72a`.
This reviewer did not author those source changes. No blocking finding emerged
from the source review.

The cadence decision precedes constructing the native snapshot; it therefore
avoids the previously measured repeated full-sector reductions, rather than only
suppressing their rendering. It is a caller-owned display policy. The option
lives on the top-level command and dashboard, outside `IntegrationInput`, native
sampling plans, checkpoint settings and mathematical identities. JSON defaults
to 100 ms, the terminal dashboard to 40 ms and plain progress to one second;
JSON interval zero restores observation after every worker batch.

Both integration lanes still evaluate, merge diagnostics, submit accepted
packages, merge accepted replay state and check cancellation every batch. They
force observations on entry to each stage/refinement round, completed allocation,
cancellation and a recorded numerical failure. The final native report is
rendered unconditionally, including its stopping reason. Native stopping checks
remain unconditional after complete production and native pilot-to-production
allocation remains unchanged. The accepted limitation is that an accumulation
range error discovered only by reducing statistics can be detected at the next
observation deadline, or the next forced boundary. Worker evaluation errors are
still detected immediately at their batch return.

The focused process regression compares interval zero with a long interval on
identical work: means/covariance, native design and serialized checkpoint settings
must agree, while ordinary progress rows decrease. It separately requires pilot
completion, empty production start and final production status despite a long
interval. The existing numerical-failure process test is extended to require a
final failed status. This checks behavior at scientific/output boundaries rather
than screen cosmetics.

The author’s focused gate passed **33 CLI tests** in
`output/status-cadence-cli-tests.log`. The reviewer checked the final source and
execution evidence: interval-zero/long-interval estimate, design, accepted replay
state and checkpoint-settings equality passed; partial-production Ctrl-C,
selected scope, saved failure, and forced pilot/production/final rows passed.
Formatting and all-target Clippy also passed
(`output/status-cadence-clippy.log`). No duplicate Symbolica run was started
while the separate scientific campaign held that runtime.

This review does not
claim a full wall-time speedup from source inspection; the separate caller-status
microprofile is attribution evidence, and a paired end-to-end run remains needed
for a performance claim.
