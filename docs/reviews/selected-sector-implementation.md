# Qualified selected-sector implementation evidence

This is the implementation author's account; the independent native/CLI review
is recorded separately by the Numerica reviewer.

The accepted proposal is implemented through `KernelResultManifest`:
`canonical_scope` validates unique existing IDs and sorts their declaration;
`integration_problem` selects the existing native `SectorSpec` records and applies
`ExactContributionPolicy`. Saved-result validation delegates to the same method.
No integral, covariance, polynomial, coordinate map or sector identity is rebuilt.
Explicit empty and all-ID selections remain `SelectedSectors`.

The CLI stores that native scope in normalized numerical settings. Omitted full
scope is skipped on serialization, preserving existing full-run settings hashes.
Original compiled IDs survive task scheduling, evaluator lookup, replay and
reporting. The native parent manifest still describes every kernel, while the
native contribution report contains precisely the projected sectors and exact
offset. Outer artifact identity continues to bind accumulation/checkpoint data;
inner kernel identity remains the saved scientific manifest identity.

Worker evaluator contexts are constructed through a map containing only selected
IDs. Full-kernel replay metadata remains in version-three checkpoints for
compatibility; unselected state is untouched. Artifact loading still compiles the
complete kernel set. This change does not claim lazy artifact compilation.

`scope` is present in live integration JSON, terminal/plain status, final output
and saved documents. `scoped_target_reached` describes the scoped tolerance;
`converged` is reserved for a full-integral claim. Existing native comparison and
reference export reject interpreting a selected estimate as a full reference.

Inspection serializes the kernel owner's existing `PortableMetadata`, exported
without changing its artifact schema. Its plain view borrows the retained native
domain/chart/map records. No metadata is reconstructed from compiled expressions
and no valuation row is relabelled as a unique U/F factor. Legacy absence remains
explicit. The first implementation retains the established artifact-loading/JIT
path; numerical result viewing remains independent of it.

Executable evidence: the focused CLI gate passed 31 tests: 18 unit tests, two
catalogue processes, four existing CLI processes, two reference processes, three
saved-result processes and two new selection/inspection processes. The latter
pair passed in 0.81 seconds (`output/selected-scope-process-tests.log`); the other
29 passed in the initial combined invocation
(`output/selected-scope-cli-tests.log`). This includes original nonzero ID1
worker execution in QMC, MC and adaptive MC; partial selected QMC resume with
new work/three workers; exact-only include/exclude in all four methods;
selected-all versus full and reordered-ID resume; native TOML scope and an
explicit `--full-integral` override; unchanged omitted full-scope serialization;
unknown/duplicate rejection; reference-export ineligibility; native metadata
serialization and borrowed plain display; valid legacy metadata absence; the
missing referenced DOT structured-error boundary; and all 24 shipped card loads.

The initial new process fixture incorrectly assumed endpoint constants would be
moved out of mixed nonconstant kernels into the folded exact vector. The runtime
correctly retained those constants inside their representative kernels. No
production change was made to satisfy that assertion. The corrected CLI fixture
uses a two-chart analytic density plus a separate genuinely zero-dimensional
exact artifact. Native pure-data projection tests independently cover the
combined nonzero exact offset plus stochastic subset, including shared-shift
cancellation covariance and original noncontiguous IDs.

A separate measured-performance follow-up is warranted: the fixed-561657b
triple-box campaign emits large full-sector status snapshots after every worker
batch and constructs/reduces snapshots before rendering cadence checks. This
observation is not an attribution or speedup claim. Profile snapshot reduction,
checkpointing and serialization separately before changing caller snapshot
cadence; retain cancellation/failure checks every batch and final/stage snapshots.
No such optimization is included in this selection change.

The follow-up native regression gate passed 17 tests: 11 saved-result contracts,
two projected-scope statistics tests and four retained-generation-metadata tests
(`output/selected-scope-native-regression.log`). The metadata tests include valid
legacy identity preservation and rejection of re-signed semantic tampering.
Compilation took 8.12 seconds and test execution about 0.07 seconds. These gates
check qualification, transport and exact native bookkeeping; they do not certify
new full-integral convergence or performance parity.
