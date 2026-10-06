# Independent generation parallelism review (2026-10-06)

Reviewed the native symbolic job dispatch, generation context, compilation
jobs, CLI scheduler and generation dashboard independently of their author.

## Findings resolved during review

The initial symbolic completion stored failures inside an opaque successful
completion. Consequently the CLI scheduler could continue scheduling every
remaining mapping/coefficient job after a native failure. `SymbolicJob::run`
now returns a `Result`, permitting the coordinator to stop the queue, request
cooperative cancellation and join the active workers. The native error remains
an error; no partial or zero integral is constructed. Compilation jobs already
had this contract.

Geometry's existing owner deliberately retains job errors for canonical merge
ordering (`GeometryCompletion::error` and native assembly). This preserves the
native precedence of earlier chart/cone/sector-limit failures. It is not replaced
with completion-order error selection merely to match symbolic job handling.
Cancellation and panics still stop scheduling and join active workers.

Reference identities are checked only after runtime parameters are bound;
generation defers the numerical reference check for parameterized kernels.

## Native ownership and admission

The numerical libraries own opaque work items, input admission, native algebra,
exact geometry, deterministic assembly and evaluator construction. The CLI owns
the Rayon pool, channels, terminal, scheduling, cancellation token and joins.
No library-owned worker pool or integration loop was introduced. Existing serial
entry points remain available, including the interpreter/portable backend.

Symbolic completion admission checks the per-call owner identity, homogeneous
stage, exact count and consecutive indices. It sorts by original index before
symmetry registration or Laurent assembly. Compilation performs the same owner
and exact coverage checks before assembling sector kernels. Source/target
symbols are assigned before dispatch; parallel completion order does not assign
chart labels. Native HEPKit/Symbolica objects stay native throughout.

The scheduler limits outstanding work to its configured worker count and uses a
bounded channel. Activity updates may be coalesced; start and completion messages
are delivered. Native calls are cooperatively cancellable at their established
boundaries, not forcibly preempted inside algebra or compilation. Completed jobs
are admitted only through the owning native stage.

## Presentation and remaining limits

Worker rows reflect actual pool workers and their reported native activity,
completed jobs and elapsed busy time. They do not measure operating-system CPU
utilization or bind threads to physical cores. The top bar aggregates all workers
for the current stage. Its ETA uses observed stage throughput, and both the bar
and footer explicitly label this as a stage estimate; later symbolic work is
not yet known. It does not claim a prediction for the entire generation run.
Colors distinguish active/idle/completed workers, with the existing no-color
policy preserved. Worker rows can be scrolled when the terminal cannot display
all configured workers.

Parallel mapping retains all mapped charts until ordered symmetry assembly;
this can use more memory than the earlier serial traversal. This round does not
establish a performance or memory bound for large examples. Existing numerical
gates and example migration remain deferred at the user's request. Build,
manual native/interpreter probes and any measured ggHH run are recorded in the
round's delivery evidence; static inspection is not a substitute for those runs.
