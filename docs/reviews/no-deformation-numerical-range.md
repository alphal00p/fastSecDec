# Native error-tracking range for algebraic kernels

The generic-degree root tests exposed a numerical-owner limitation independent
of polynomial degree. Numerica's `ErrorPropagatingFloat<Float>` has an arbitrary
precision centre but an `f64` absolute uncertainty. A finite uncertainty around
`1e-153`, multiplied by `1e-200`, could become zero; multiplying back by
`1e200` then retained the nonzero centre with zero reported uncertainty. Native
constant construction at sufficiently high precision had the same range issue.
Two precision evaluations agreeing at a rounded zero therefore did not establish
an acceptable algebraic-kernel result.

## Owner fix and reuse boundary

[Numerica PR #9](https://github.com/symbolica-dev/numerica/pull/9), commit
`ce1388934f0314f89b5f9391889d191f0c9b9531`, adds checked uncertainty arithmetic
and construction. When the existing uncertainty representation cannot retain
the result, it reports invalid tracking through a nonfinite uncertainty. It
does not add a second error estimator, change the stored uncertainty type, or
provide certified ball arithmetic. Consumers must check the uncertainty as
well as the numerical centre. A documented explicit precision reset remains
a reset; consumers must inspect invalid tracking before requesting one.

The checks cover conversions, constants, arithmetic, powers, reciprocals and
native mathematical functions. Review additionally found cancellation in the
`tanh` derivative when its arbitrary-precision value rounded to one in `f64`,
and loss of the exponent uncertainty of `powf` when a base near one was rounded
before taking its logarithm. The fix uses existing native operations before
conversion and rejects unrepresentable tracking instead of fabricating zero.

The public Symbolica consumer commit
`ae0c82b7bda670fdb9f6229226bd47b13bb49a20` incorporates that owner change on
`1ac765fd17e9273706d762b2afe450c3c5bd44f0`. It retains the previously adopted
native `hypot` operation, adapted to the same checked range rules. Excluding
that retained operation, the Numerica implementation matches the PR source.
No local installation-preparation script or FastSecDec algebra implementation
is required. All consuming dependency roots must select this same identity.

The implementation was independently reviewed by the kernel agent and the
coordinator. The PR is authored by ValentinHirschi. Repository permissions
prevented formal reviewer assignment, so the PR explicitly requests review
from `benruijl` in a comment.

## Evidence and acceptance boundary

The unmodified owner fails all nine new regression groups. The corrected
standalone Numerica owner passes 174 native library tests, 14 existing
integration tests and nine new groups; its portable build passes 151 library
tests, the same 14 integration tests and nine new groups.

The combined Symbolica owner passes 211 native and 188 portable tests,
including retained owner controls and six additional `hypot` range controls.
Formatting and diff checks pass. Whole-owner strict Clippy still reports
pre-existing diagnostics; comparison against the unchanged owner found no
additional diagnostics. This is not a claim that upstream strict Clippy is
green.

The unchanged FastSecDec core at `d0536d4` passes its complete native library
suite with the new owner: 678 passed, 21 unchanged ignored, in 187.89 seconds.
The normal core build also passed, with its emitted artifact and exact native
dependency identities retained for subsequent algebraic-kernel probes.
The portable consumer also passes all 88 tests. The source-matched embedded
HEPKit host passes its represented-input control, including the independent
complex bubble oracle, recovery and callbacks. Two saved-artifact controls
pass threshold normal/serial restoration and fixed/polynomial/sign-aware
contour restoration with complete vectors, identities and validation policies.
Standalone native and portable Python-binding checks pass. These checks do
not replace an installed wheel, running community server or browser.
The exact six-file dependency change is limited to
the core, Python binding and portable-test manifests and locks; each lock
changes only the two Symbolica/Numerica source entries. Version numbers and
other package/dependency entries remain unchanged.

After importing the dependency change on `65abaaa`, the coordinator's joined
resolver suite passes all 109 tests in 173.91 seconds with the new owner.
Strict workspace/all-target Clippy with threshold decomposition enabled,
formatting and diff checks also pass. The later resolver milestone is thereby
checked separately from the complete `d0536d4` library baseline above.

The separate algebraic-kernel candidate is not accepted by these core tests.
Its first full QMC replay with the corrected owner refused a sample during
precision escalation. Native probes traced the refusal to unnecessary tracked
construction of two exact controls: a structurally zero imaginary coordinate
in the real root solver and a power-of-two stopping tolerance. Constructing
those controls exactly, while retaining all incoming/computed uncertainty,
lets the same point pass two successive accepted precision tiers. The corrected
complete replay passes independently for Eager and SymJIT: 16,384 samples per
backend, the full six-component Laurent vector and its 36 covariance entries.
Its finite imaginary part agrees with the independent quintic-root oracle.
This supersedes the earlier result on the old owner. No sample is discarded
or replaced by zero, and the integration tolerance is unchanged.

That small private control takes about 48 seconds per backend and rescues
11,504 of 16,384 samples, predominantly at 256 bits. It is correctness evidence,
not a production performance result or an accepted general algebraic factory.
The new callback/artifact implementation still needs its separate joined gates.
Genuine uncertainty-range exhaustion remains an explicit failure.

Local reproducibility records are under `target/no-deformation-epf-owner/`
and `target/no-deformation-ae0-adoption/`; raw logs remain untracked. The frozen
owner handoff has SHA-256
`aed036ff6ac4d2302f8116e9771a8a7c19547e1bbb6cffa453e379ecba0c5b49`.
The frozen consuming-workspace handoff has SHA-256
`c65296efca5f15be9db46ce45211e8302b12d54e283094cba8ecb48672d0760c`;
the coordinator verified all 44 evidence hashes and six unchanged source
preimages before importing it. Two private harness setup failures are retained
in that evidence: a compiler-wrapper launch and omitted citation assets. Both
preceded production tests and required no production source repair.
This numerical range fix does not close general algebraic resolution, certify
floating-point answers, or establish production sampling performance.
