# Shared external wavefunctions

GammaLoop's numerical scalar, vector and spinor states now have a shared native
owner in `feynkit-kinematics`. The tested extraction is published on GammaLoop's
`feynkit` branch as
[`6c707c6b77a437256eb1180da13d4d327b371d13`](https://github.com/alphal00p/gammaloop/commit/6c707c6b77a437256eb1180da13d4d327b371d13).
The community dependency/stub update is
[HEPKit PR #17](https://github.com/symbolica-dev/symbolica-community/pull/17),
now ready for review after its complete installed-wheel checks passed. Both repositories
were published as `ValentinHirschi <valentin.hirschi@gmail.com>`.

## Ownership and conventions

`FourMomentum<T>::wavefunction` returns the shared `Wavefunction<T>`, with
`Scalar`, `Epsilon`, `EpsilonBar`, `U`, `UBar`, `V` and `VBar` kinds. Native
Numerica complex coefficients retain the existing generic precision. GammaLoop
delegates its numerical construction and adapts only its existing tensor
storage; PyO3 exposes the same implementation in `feynkit-py`. FastSecDec adds
neither wavefunction formulas nor a Python dependency.

The extraction preserves GammaLoop/MadGraph phases, negative-axis limits and
the chiral Dirac adjoint. Numerical external vectors remain four-dimensional,
independently of internal symbolic Lorentz dimension. No spin/color average,
helicity sum or coupling is included. Scalar states have one component.
Invalid kinds/helicities and nonfinite inputs return native errors. The inherited
longitudinal convention is undefined at rest or zero mass; it remains explicitly
unsupported, rather than introducing an arbitrary spin axis. Valid massive
longitudinal states and massive/rest spinors are covered.

The root review checked the moved formulas, GammaLoop delegation, tensor-storage
preservation and Python conversion. The diagram example owner separately
reviewed the phase conventions. No alternate graph, momentum, tensor or numeric
representation is introduced. `PyKinematics::as_kinematics` adds the native
borrow needed by later HEPKit backends; the graph borrow already exists upstream.

## Executed checks

The isolated publication tree starts at upstream `eb744c65`, preserving its
Cargo lock, Symbolica `942bd2c0` and registry Numerica 3.0.1. It excludes the
unrelated renderer edits in the earlier FastSecDec dependency worktree.

| Gate | Result |
| --- | --- |
| Native numerical states | 6 passed: phases, vector transversality/completeness, Dirac equations/spinor completeness, adjoints/scalars, invalid states and generic precision |
| Existing GammaLoop spinors | 2 passed, including degenerate momentum branches |
| Existing GammaLoop vectors | 2 passed, including degenerate momentum branches |
| Existing polarized density | 1 passed |
| Installed Python host | 1 passed, exercising every state kind, native complex transport, immutability and native errors |
| Generated Python stub | Passed; 117 API/documentation lines added |
| Selected Rust formatting and diff whitespace | Passed |

The combined test build exposed six existing ambiguous `collect()` assertions
under the combined feature set. Explicit `BTreeSet<_>` annotations preserve
their meaning and permit the actual GammaLoop tests to compile. Removing the
now-unused private square-root helper avoids leaving a second numerical owner.
Initial failed build and stub-documentation attempts remain in the evidence.

Local reports are in `output/diagnostics/shared-external-states-upstream-1`.
The Symbolica runtime reports an outdated license format; passing serial tests
do not establish that the supplied key was accepted.

The complete community wheel at `956f70fa4ad0` builds with `--locked` and imports
exclusively from the isolated `output/hepkit-showcase-venv` installation. All
fourteen wavefunction cases, the namespace/stub check and eight literal new
documentation examples pass: **23 checks, zero failures**. The build took
1,152 seconds, left Cargo.lock unchanged and all processes were reaped.
Evidence is in `output/diagnostics/community-external-states-1/result.json`.
This validates the native community host, not browser responsiveness or the
separate FastSecDec bridge.

The new documentation also passes the existing structure/parameter checks.
Two broad documentation failures
are reproduced on unchanged community `cd36326`: missing examples for
`IBPFamily.compiled_runtime_arities` and an undocumented `cut` constructor
parameter. They are unrelated to this API update.

The PR's Linux and macOS CI jobs build, then fail at collection under Python
3.10 because unchanged `tests/test_rustred_dot_families.py` imports `tomllib`
unconditionally. The same import is present at base `cd36326`. Neither CI job
reaches its full test suite; the successful installed-wheel checks above use
Python 3.12. See [the CI run](https://github.com/symbolica-dev/symbolica-community/actions/runs/37343874634).
