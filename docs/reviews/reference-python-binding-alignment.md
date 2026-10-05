# Isolated reference Python binding alignment

The isolated build in `output/diagnostics/matched-python-binding-3/` succeeded
with exit 0 in 390 seconds (recorded UTC endpoints), under a 1,200-second bound on
CPUs 10, 11 with two Cargo jobs. This is reference-environment preparation only;
FastSecDec's Rust dependencies and the working Pathfinder venv are unchanged.

The wheel uses the exact current Symbolica 3.0.1 source and five retained fixes,
current separate Numerica source, Graphica source, and SymJIT 2.26.4 resolution.
The explicitly enabled numerical backends are GMP/MPFR and native code
generation, with serde/bincode. Python API/ABI and necessary binding dependencies
are additional; no community package or default mimalloc allocator is included.
An isolated copied manifest pins SymJIT exactly, points Numerica at the copied
current source, and matches the existing release LTO/codegen settings. No
mathematical source or dependency upgrade was introduced.

The first two metadata-only attempts are preserved. Their overly strict checks
stopped before compilation: one compared inactive/build dependencies in the
whole lockfile, and one treated a `default` feature marker as a numerical change.
The accepted check compares enabled non-macro numerical/runtime dependency
identities and essential backends, and records additional binding, macro and
build dependencies. Root explicitly approved those narrow guard corrections.
There was one compiler build attempt; no API port or alternate executor.

The resulting local Linux ABI3 wheel is 15,516,184 bytes, SHA256
`05fc5dc8f959ca07db6a8b0497e1ad8a3d03479ae8bb7bed855878a39555e0f8`.
The resolved lockfile SHA256 is
`1bbfbebf7c4f87e5290f9bc7bd873b25b52d2bf565e1c2f8873375669814cf52`.
All 159 copied source/packaging postchecks pass. Full enabled-feature and
binding-dependency differences are retained alongside `result.json`. The wheel
was extracted into an isolated import root without installing it or importing
Symbolica. This local wheel carries no cross-platform packaging claim.

The only queued compatibility check is the existing one-loop triangle smoke:
complete `[-2, -1, 0]`, 1,024 points/eight shifts, one worker, O2 complex evaluators,
seed 20261200, and the existing analytic/master-backed five-SE control. A loaded
module path/version check ensures the isolated wheel is actually selected.
The 180-second / 30 GiB process will run after native scientific runtime releases;
its outcome will be retained without starting an API-porting project. Until it
passes, the wheel is compile-ready but reference CLI compatibility is unverified.
No performance comparison has run. The separate original-on-shell correctness
reference deliberately retains its already working older venv and cache inputs.

## Compatibility smoke outcome

The single smoke process selected the intended isolated module and reported
Symbolica 3.0.1, then exited 1 after 1.167 seconds before coefficient generation.
The unchanged Pathfinder `src/symbolic_constants.py:20` unpacks the return from
`Expression.evaluate` into a pair; the current binding returns a native
`ComplexFloat` object. This is a concrete reference API mismatch, not a
numerical disagreement. No full-vector result exists. The process group was
reaped and all frozen hashes remained unchanged; evidence is retained in
`output/diagnostics/matched-python-binding-smoke-1/`.

No compatibility patch, reference port, rebuild or repeated smoke was made.
The existing working reference environment remains the explicitly versioned
benchmark baseline. Exact release alignment is unavailable for this unchanged
caller despite the successful isolated binding build.
