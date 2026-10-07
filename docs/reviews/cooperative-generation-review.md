# Independent cooperative-generation review

The root review inspected the retained generation owner separately from its
implementation: geometry job admission, deterministic chart ordering, native
mapping and symmetry registration, coefficient expansion, exact-term folding,
failure handling, and pause boundaries.

`GenerationSession` retains the existing `GeometryPlan`, `GeometryJob`,
`PreparedGeometry`, mapped charts, symmetry registry and native coefficient
outputs. It introduces no executor, graph representation, algebra backend or
numerical estimator. The caller decides when to advance another native unit.
The ordinary generator and the new session share the extracted `Assembly`
implementation, preserving multiplicities, Laurent order padding, exact
contributions and chart-to-kernel associations.

Two review findings were corrected before acceptance: coordinate namespace
selection now uses one shared helper, and snapshot total time tracks active step
time consistently. Paused wall time is excluded. An observer's pause request is
latched until the current native unit completes; successful work remains owned.
Native errors leave a terminal failed session with no partial scientific result.
The ordinary generator retains its existing abort behavior.

The unchanged public-owner comparisons in `generation_session.rs` cover cube
and simplex domains, both expansion methods, several step budgets, repeated
pauses, full metadata/coefficients/cancellation profiles, exact folding, namespace
collisions and terminal geometry failure. The portable validation package reuses
that same test source. Execution evidence belongs in the notebook workflow
results report once the complete milestone checks finish.

The remaining responsiveness limit is explicit: one native chart/cone job,
symmetry preparation, or representative's coefficient expansion is atomic.
The session does not claim interruption inside a Symbolica operation. Native
one-loop master and reduction APIs remain independent reference providers; this
scheduling change adds no replacement for them.
