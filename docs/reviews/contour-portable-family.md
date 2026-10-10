# Portable native family and diagnostics acceptance

2026-10-10. The maintained standalone tests are
`tests/portable-kernel/tests/contour_family.rs` and `contour_diagnostics.rs`.
They use the production portable arithmetic graph and public native APIs.
Neither test adds a Python bridge, filesystem abstraction, evaluator, integrator
or callback implementation.

## Reuse and scope

The family test follows the public `RecipeFamilySession` and indexed archive
interfaces, their implementation in `generation/family/session.rs` and
`kernel/artifact/indexed/programs`, and the existing native cooperative family
tests. Actual execution is the focused probe of `tempfile` and `std::fs` support;
availability of these crates alone is not taken as WASM filesystem evidence.

Two cases request all four native recipes: symbolic generation with no resident
owner and numerical-dual generation retaining sign-aware execution independently
of the undeformed archive default. Construction is inert. Caller stepping pauses
after each unit, preserves monotonic active elapsed time, and exposes no partial
completed result. Each shared source is prepared once. The output is an actual
File, and source staging is removed after generation before any archive reload.
The test copies the completed archive to a second file, removes the original,
selects every recipe, restores its complete program and an individual sector,
and closes the archive reader before numerical execution.

Every deformed owner performs a real native homotopy pilot before sampling.
Policy-only transition to Pilot preserves mathematical identity. Native QMC
then integrates the complete complex Laurent vector of
`(2+3i)*x^(-1+eps)/(1+x)`, whose pole is `2+3i` and finite coefficient
`-(2+3i)*log(2)`. The declared work is 1024 Kuo points by four shifts, Korobov3,
seed 294371. Component standard errors must be below `2e-5`, and means must agree
within eight standard errors or `2e-5`, whichever is larger. Native validation
checks the complete covariance. Sampling contexts outlive the generated and
restored KernelSets; Pilot production performs no optional causal checks.

The separate diagnostics test uses genuine three-dimensional cubic envelopes
for both constructions. It exercises actual pilot completion, Off production,
Disabled/Aggregate bitwise value parity, native iterative solver counters,
displacement ranges, local clone/drain semantics and unchanged saved program and
mathematical identities. Diagnostics are observations, not sampling state or
numeric acceptance conditions.

## Execution record

The portable-host gate passed **family 2/2** in 10.07 seconds and
**diagnostics 1/1** in 0.44 seconds, after a 1 minute 9 second incremental build.
The log is `target/contour-portable-family-diagnostics-tests.log`.
Independent foundation review found no source blocker in the family controls.

Actual Emscripten execution passed **family 2/2 in 3.31 seconds** and
**diagnostics 1/1 in 0.38 seconds**, using the existing isolated Rust 1.98.0,
Emscripten 5.0.3 and Node 24.19.0 toolchain under `output/wasm-toolchain`.
The initial cross-build took 1 minute 7 seconds. Logs are
`target/contour-family-wasm-file-owner-tests.log` and
`target/contour-diagnostics-wasm-tests.log`; build records and executable hashes
are retained under `target/contour-family-diagnostics-wasm-*`.

The first WASM family attempt exposed an unsupported operation in test setup:
Rust `File::try_clone` cannot duplicate its descriptor on this target. Temporary
directory and named-file creation already succeeded. The fixture now uses the
existing `NamedTempFile::into_parts` API to transfer the File directly while
retaining its TempPath owner. No production fallback or filesystem wrapper was
added; native family generation and the Python bridge already accept an owned
File without duplication. The initial failed log remains
`target/contour-family-wasm-tests.log`. The final fixture also passed on the
portable host: family **2/2 in 9.87 seconds**, diagnostics **1/1 in 0.41 seconds**,
log `target/contour-portable-family-diagnostics-final-tests.log`. There is no
WASM-only source variant.

Strict standalone Clippy also passed for both maintained targets with
`--locked -- -D warnings`; log
`target/contour-portable-family-diagnostics-clippy.log`.

Successful Node/Emscripten execution establishes native Rust family storage and
numerical behavior in that filesystem environment only. It does not establish
browser responsiveness, bounded disk residency in browser MEMFS,
a Pyodide/HEPKit wheel, caller thread dispatch in WASM, or physical multiloop
performance.
