# Existing HEPKit prepared threshold owner

The native FastSecDec binding now accepts `threshold_decomposition=True` through
`sd.sector_decompose`, `FeynmanDiagram.sector_decompose`, and
`IntegralFamily.sector_decompose`. Successful preparation returns
`PreparedThreshold`. Its explicit `compile()` action drives the retained native
session and selects the accepted `RecipeArchive`; integration remains a separate
existing QMC/MC action. Ordinary calls continue to return `GeneratedIntegral`.

The pinned FeynKit 8e3a643 owner methods already forward keyword arguments to this
backend. Their source and an actual matching embedded Python host were checked;
no Community/FeynKit runtime implementation or alternate graph/input type was
added. `PyIntegral::new/from_family` still owns native input validation, graph
weights, signed family denominator slots, weighted numerator, scalar bindings,
kinematics, tensor dimension and measure. Native threshold preparation, detached
record compilation and archive publication are unchanged. The wrapper introduces
no CAS, codec, estimator, worker pool or fabricated `GeneratedIntegral`.

A private constructor requires successful preparation by the live retained
session. JSON does not issue an owner. Compilation policy is immutable from
preparation because it enters the native plan identity; default Horner remains
ten. `generation_session()` returns the identical owner. Callback cancellation
preserves its completed records; callback exceptions retain their Python identity.
Checked PyO3 borrows reject simultaneous/reentrant mutable access as Python errors
rather than panic. Successful resident selection is cached. `complete` means
archive publication is complete, independently of whether resident selection has
succeeded; retrying `compile()` can retry that load. Snapshot, repr, receipt and
archive accessors are inert. A caller may step record compilation and use the
archive without loading every kernel through the convenience `compile()`.

Independent root and runtime-agent source reviews passed this ownership/API
boundary. The actual embedded host links the public native Symbolica, FeynKit and
FastSecDec registration APIs, without replacing an installed wheel or server.
Its prepared-owner control passed in 5.34 seconds (one composite Rust test after
73 seconds of cached compilation). It exercises all three existing entrypoints,
preparation cancellation/exception identity, compiler cancellation/resumption,
reentrant borrow refusal, ordinary `GeneratedIntegral` behavior, invalid family
powers/numerator and graph-owned numerator guards, and use after preparation
owners drop. All three paths agree in the full complex Laurent vector with the
exact physical bubble at s=16, m²=3, using the explicit Gamma/rGamma normalization:
pole (1,0), finite (2−1.5 ln 3, π/2). Separate caller-driven QMC uses 4096×8 points,
seed 202610111001; comparisons retain the full vector and uncertainties. The test
also retains prior session raw-checkpoint/no-resolve, archive save/reload and final
callback controls. Maintained pytest controls are supplied; the execution claim
here is the embedded source-matched host, not a separately installed pytest host.

The supported scientific scope remains exact fixed inputs and the admitted
one-dimensional rational-cell path, including affine projective 2→1 preparation.
General algebraic endpoint resolution, runtime physical rebind, represented Float
inputs and global proof replay are not established by this wrapper. Represented
numerical inputs are the next adapter, using the existing native represented-value
owner and its authoritative original/conversion identity. This milestone does not
claim general-resolver completion, browser support or an installed wheel test.

Native and portable `python_stubgen` feature checks passed; native-only classes
are feature-gated. The pinned owner methods' older text signatures do not list the
new kwargs even though actual forwarding works. Backend stub generation is checked
separately below. Exact raw evidence and source pins live under the ignored
`target/no-deformation-hepkit-prepared-owner` directory.

The final source-matched host rerun passed in 5.28 seconds, including the explicit
`archive_complete` repr contract. Native backend stub inventory and rendering
passed in 0.18 seconds; Python `ast.parse` accepted the emitted module and all
three native-only classes were present. Native/portable stub feature checks both
passed. Actual portable stub rendering was not run; no installed-host or browser
claim follows from the portable check. Final strict native all-target Clippy with
`python_stubgen` and formatting checks passed. The private stub harness's Python
3.13 linker annotation is not part of the production patch.

After registration, root's strict all-target `python_stubgen` Clippy and format
checks pass against the current shared core, including the newer AJ/resolution
milestones. The embedded numerical host above used its recorded frozen core;
these two evidence scopes are kept distinct.

Threshold preparation and successful saved-kernel selection/loading now also
mark the existing cumulative citation collector. It retains FastSecDec/pySecDec
and adds the Jones–Olsson–Stone 2025/2026 method papers and symGCAD software.
Primary arXiv metadata and an independent HEPKit source review confirm the
references and call boundaries. Passive import/settings construction, an initial
cancelled action and invalid artifact loads do not mark threshold use. Citation
tracking requires no proof replay, generation or evaluator work.
